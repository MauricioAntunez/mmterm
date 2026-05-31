use std::io::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Instant;

use base64::Engine as _;
use crossbeam_channel::{Receiver, Sender};

use crate::app_state::TabState;
use crate::terminal::TerminalParser;
use crate::terminal::grid::Grid;

use super::App;

#[cfg(test)]
#[path = "drain_test.rs"]
mod tests;

// ── ParseEffect ───────────────────────────────────────────────────────────────

/// Side-effects produced by a parser thread batch and consumed on the main thread.
pub enum ParseEffect {
    PtyResponse(Vec<u8>),
    ClipboardWrite(String),
    ClipboardRead,
    Bell,
    /// Scrollback length changed; main thread adjusts scroll_offset to match.
    /// `old` may be greater than `new` on alternate screen entry (clamp case).
    ScrollbackChanged {
        old: usize,
        new: usize,
    },
    /// Parser thread's PTY EOF — pane should be closed.
    Disconnected,
}

// ── Parser thread ─────────────────────────────────────────────────────────────

/// Bytes drained from the PTY channel per parser iteration.
/// Caps write-lock duration at ~36 ms (32 KiB / 885 KiB/s).
const PARSE_BATCH_MAX: usize = 32 * 1024;

/// Spawn a per-pane parser thread that owns the VTE state machine.
/// The thread reads PTY bytes from `rx`, parses them into `grid` (write lock),
/// and sends side-effects to `effects_tx`. Responds to `discard_signal` by
/// draining the channel without parsing (instant Ctrl+C response).
pub fn spawn_parser_thread(
    rx: Receiver<Vec<u8>>,
    grid: Arc<RwLock<Grid>>,
    log_file: Arc<Mutex<Option<std::fs::File>>>,
    effects_tx: Sender<ParseEffect>,
    discard_signal: Arc<AtomicBool>,
    wakeup: Box<dyn Fn() + Send + 'static>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut parser = TerminalParser::new();
        loop {
            // Block until first chunk arrives from PTY reader thread.
            let first = match rx.recv() {
                Ok(b) => b,
                Err(_) => {
                    let _ = effects_tx.send(ParseEffect::Disconnected);
                    return;
                }
            };

            // Drain any further immediately available chunks (up to batch cap).
            let mut batch = first;
            while batch.len() < PARSE_BATCH_MAX {
                match rx.try_recv() {
                    Ok(more) => batch.extend_from_slice(&more),
                    Err(_) => break,
                }
            }

            // Ctrl+C / Ctrl+\: discard entire queue without parsing.
            if discard_signal.swap(false, Ordering::AcqRel) {
                while rx.try_recv().is_ok() {}
                wakeup();
                continue;
            }

            // Log raw bytes before parsing.
            if let Ok(mut guard) = log_file.lock()
                && let Some(f) = guard.as_mut()
            {
                let _ = f.write_all(&batch);
            }

            // Parse bytes → mutate grid.
            let old_sb = grid.read().unwrap().scrollback_len();
            {
                let mut g = grid.write().unwrap();
                parser.process(&batch, &mut g);
                g.scan_urls();
            }

            // Scrollback changed → scroll_offset compensation on main thread.
            let new_sb = grid.read().unwrap().scrollback_len();
            if new_sb != old_sb {
                let _ = effects_tx.send(ParseEffect::ScrollbackChanged {
                    old: old_sb,
                    new: new_sb,
                });
            }

            // Extract side-effects accumulated during parsing.
            {
                let mut g = grid.write().unwrap();
                let resp = std::mem::take(&mut g.pending_responses);
                if !resp.is_empty() {
                    let _ = effects_tx.send(ParseEffect::PtyResponse(resp));
                }
                if let Some(t) = g.pending_clipboard_write.take() {
                    let _ = effects_tx.send(ParseEffect::ClipboardWrite(t));
                }
                if std::mem::take(&mut g.pending_clipboard_read) {
                    let _ = effects_tx.send(ParseEffect::ClipboardRead);
                }
                if std::mem::take(&mut g.bell_pending) {
                    let _ = effects_tx.send(ParseEffect::Bell);
                }
            }

            wakeup();
        }
    })
}

// ── Main-thread drain ─────────────────────────────────────────────────────────

impl App {
    /// Consume side-effects from all pane parser threads.
    /// Returns (tab_idx, pane_id) pairs for panes whose PTY disconnected.
    pub(super) fn drain_effects(&mut self) -> Vec<(usize, usize)> {
        // Phase 1: drain per-pane effects that only touch the pane/PTY.
        // Defer clipboard and bell effects (need self-level access) for phase 2.
        struct Deferred {
            tab_idx: usize,
            pane_id: usize,
            kind: DeferredKind,
        }
        enum DeferredKind {
            ClipboardWrite(String),
            ClipboardRead,
            Disconnected,
        }
        let mut deferred: Vec<Deferred> = Vec::new();
        let mut bell_tabs: std::collections::HashSet<usize> = Default::default();

        for tab_idx in 0..self.state.tabs.len() {
            let pane_ids: Vec<usize> = self.state.tabs[tab_idx].panes.keys().copied().collect();
            for pane_id in pane_ids {
                loop {
                    let effect = self.state.tabs[tab_idx]
                        .panes
                        .get_mut(&pane_id)
                        .and_then(|e| e.effects_rx.try_recv().ok());
                    match effect {
                        None => break,
                        Some(ParseEffect::PtyResponse(r)) => {
                            if let Some(e) = self.state.tabs[tab_idx].panes.get_mut(&pane_id) {
                                let _ = e.pty.write_input(&r);
                            }
                        }
                        Some(ParseEffect::ScrollbackChanged { old, new }) => {
                            if let Some(e) = self.state.tabs[tab_idx].panes.get_mut(&pane_id)
                                && e.pane.scroll_offset > 0
                            {
                                let added = new.saturating_sub(old);
                                e.pane.scroll_offset = (e.pane.scroll_offset + added).min(new);
                            }
                        }
                        Some(ParseEffect::Bell) => {
                            bell_tabs.insert(tab_idx);
                        }
                        Some(ParseEffect::ClipboardWrite(t)) => {
                            deferred.push(Deferred {
                                tab_idx,
                                pane_id,
                                kind: DeferredKind::ClipboardWrite(t),
                            });
                        }
                        Some(ParseEffect::ClipboardRead) => {
                            deferred.push(Deferred {
                                tab_idx,
                                pane_id,
                                kind: DeferredKind::ClipboardRead,
                            });
                        }
                        Some(ParseEffect::Disconnected) => {
                            deferred.push(Deferred {
                                tab_idx,
                                pane_id,
                                kind: DeferredKind::Disconnected,
                            });
                            break;
                        }
                    }
                }
            }
        }

        // Phase 2: apply deferred effects (clipboard / disconnect).
        let mut exited = Vec::new();
        let now = Instant::now();
        for d in deferred {
            match d.kind {
                DeferredKind::ClipboardWrite(t) => {
                    if let Some(cb) = self.state.clipboard.as_mut() {
                        let _ = cb.set_text(t);
                    }
                }
                DeferredKind::ClipboardRead => {
                    let text = self
                        .state
                        .clipboard
                        .as_mut()
                        .and_then(|cb| cb.get_text().ok())
                        .unwrap_or_default();
                    let encoded = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
                    let resp = format!("\x1b]52;c;{encoded}\x1b\\");
                    if let Some(e) = self.state.tabs[d.tab_idx].panes.get_mut(&d.pane_id) {
                        let _ = e.pty.write_input(resp.as_bytes());
                    }
                }
                DeferredKind::Disconnected => exited.push((d.tab_idx, d.pane_id)),
            }
        }
        for tab_idx in bell_tabs {
            if let Some(tab) = self.state.tabs.get_mut(tab_idx) {
                trigger_bell(tab, now);
            }
        }
        exited
    }
}

fn trigger_bell(tab: &mut TabState, now: Instant) {
    let cooled = tab.bell_cooldown_until.is_none_or(|until| now >= until);
    if cooled {
        tab.bell_flash_start = Some(now);
        tab.bell_flash_until = Some(now + std::time::Duration::from_millis(150));
        tab.bell_cooldown_until = Some(now + std::time::Duration::from_millis(500));
    }
}
