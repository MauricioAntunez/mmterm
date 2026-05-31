use std::num::NonZeroU32;
use std::time::{Duration, Instant};

use chrono::Local;

use crate::input::InputMode;
use crate::renderer::Renderer;
use crate::{AppState, command_palette, screenshot, statusbar, views};
use winit::event::Modifiers;
use winit::event_loop::ActiveEventLoop;

use super::App;

pub(crate) fn bell_flash_intensity(start: Option<Instant>) -> Option<f32> {
    const BELL_DURATION_MS: f32 = 150.0;
    let start = start?;
    let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;
    if elapsed_ms >= BELL_DURATION_MS {
        None
    } else {
        let t = elapsed_ms / BELL_DURATION_MS;
        Some(1.0 - t * t)
    }
}

pub(super) fn draw_overlays(
    renderer: &mut Renderer,
    state: &AppState,
    pixels: &mut [u32],
    w: u32,
    h: u32,
) {
    if let Some(panel) = &state.config_panel {
        renderer.draw_config_panel(pixels, w, h, panel);
    }
    if let InputMode::CommandPalette { query, selected } = &state.mode {
        let filtered = command_palette::filter(query);
        let entries: Vec<(&str, &str)> = filtered
            .iter()
            .map(|&i| {
                (
                    command_palette::entry_label(i),
                    command_palette::entry_shortcut(i),
                )
            })
            .collect();
        renderer.draw_command_palette(pixels, w, h, query, &entries, *selected);
    }
    if state.quit_pending {
        renderer.draw_quit_confirm(pixels, w, h, &state.theme);
    }
    if matches!(state.mode, InputMode::QuitSave) {
        renderer.draw_save_session_confirm(pixels, w, h, &state.theme);
    }
    if let InputMode::Screenshot {
        cx,
        cy,
        half_w,
        half_h,
    } = state.mode
    {
        renderer.draw_screenshot_selector(pixels, w, h, cx, cy, half_w, half_h);
    }
    if let InputMode::ScreenshotName {
        cx,
        cy,
        half_w,
        half_h,
        ref name,
    } = state.mode
    {
        renderer.draw_screenshot_name_input(pixels, w, h, cx, cy, half_w, half_h, name);
    }
}

impl App {
    pub(crate) fn redraw(&mut self) {
        if self.state.blink_last.elapsed()
            >= Duration::from_millis(self.state.config.window.cursor_blink_ms as u64)
        {
            self.state.blink_last = Instant::now();
            self.state.cursor_blink = !self.state.cursor_blink;
        }

        let Some(surface) = &mut self.surface else {
            return;
        };
        let Some(window) = &self.window else { return };
        let size = window.inner_size();
        let (w, h) = (size.width, size.height);
        if w == 0 || h == 0 {
            return;
        }

        if self.surface_size != (w, h)
            && let (Ok(wn), Ok(hn)) = (NonZeroU32::try_from(w), NonZeroU32::try_from(h))
        {
            let _ = surface.resize(wn, hn);
            self.surface_size = (w, h);
        }

        let mut buf = surface.buffer_mut().unwrap();
        let pixels: &mut [u32] = &mut buf;

        if self.state.tabs.is_empty() {
            buf.present().unwrap();
            return;
        }

        self.state.tabs[self.state.active_tab].has_activity = false;

        let (separators, zoomed, active_id) = {
            let tab = &self.state.tabs[self.state.active_tab];
            (tab.layout.separators(), tab.zoomed, tab.active)
        };

        // Clone grid Arcs so guards are independent of &self.state lifetime.
        // This allows &mut self.state after the rendering block (e.g. screenshot clipboard).
        let grid_arcs: Vec<(
            usize,
            std::sync::Arc<std::sync::RwLock<crate::terminal::Grid>>,
        )> = {
            let tab = &self.state.tabs[self.state.active_tab];
            tab.panes
                .iter()
                .map(|(id, e)| (*id, e.pane.grid.clone()))
                .collect()
        };

        let screenshot_outcome = {
            let guards: Vec<(usize, std::sync::RwLockReadGuard<crate::terminal::Grid>)> = grid_arcs
                .iter()
                .map(|(id, arc)| (*id, arc.read().unwrap()))
                .collect();
            let views = views::collect_pane_views(&self.state, &guards, w, h);
            let tab_titles = views::build_tab_titles(&self.state);

            let metrics = self.state.tabs[self.state.active_tab].metrics.clone();
            let draw_separators: &[[u32; 4]] = if zoomed { &[] } else { &separators };
            let home = std::env::var("HOME").unwrap_or_default();
            let cwd_owned: Option<String> = self.state.tabs[self.state.active_tab]
                .panes
                .get(&active_id)
                .and_then(|e| e.pane.grid.read().unwrap().cwd.clone())
                .map(|p| statusbar::shorten_home(&p, &home));
            let right_text = statusbar::resolve(
                &self.state.config.status_bar.right,
                cwd_owned.as_deref(),
                &Local::now(),
            );
            let bell_flash_intensity =
                bell_flash_intensity(self.state.tabs[self.state.active_tab].bell_flash_start);
            let is_logging = self.state.tabs[self.state.active_tab]
                .panes
                .get(&active_id)
                .is_some_and(|e| e.log_file.lock().unwrap().is_some());
            let pane_osc_title_owned: Option<String> = self.state.tabs[self.state.active_tab]
                .panes
                .get(&active_id)
                .and_then(|e| e.pane.grid.read().unwrap().osc_title.clone());
            let pane_title_raw = pane_osc_title_owned.as_deref();
            let pwd_in_right = self.state.config.status_bar.right.contains("%pwd");
            let pane_title = statusbar::pane_title_for_display(
                pane_title_raw,
                pwd_in_right,
                cwd_owned.as_deref(),
            );
            self.renderer.draw(
                pixels,
                w,
                h,
                &views,
                draw_separators,
                &self.state.mode,
                self.state.tabs[self.state.active_tab].passthrough,
                &tab_titles,
                &metrics,
                self.state.search_matches.len(),
                self.state.search_current,
                right_text.as_deref(),
                pane_title,
                self.state.config.window.inactive_dim,
                bell_flash_intensity,
                self.state.config.general.visual_bell,
                is_logging,
                &self.state.theme,
            );

            // Capture screenshot before overlays; views/guards still alive here.
            self.pending_screenshot
                .take()
                .map(|([px, py, pw, ph], name)| {
                    screenshot::save_screenshot(
                        pixels,
                        w,
                        [px, py, pw, ph],
                        &self.state.config.general.screenshot_dir,
                        &name,
                    )
                })
            // guards, views dropped at end of block
        };

        // Apply screenshot result after views/guards are dropped (needs &mut self.state).
        if let Some(result) = screenshot_outcome {
            match result {
                Ok(path) => self
                    .state
                    .copy_text_to_clipboard(path.to_string_lossy().into_owned()),
                Err(e) => log::warn!("screenshot save failed: {e}"),
            }
        }

        draw_overlays(&mut self.renderer, &self.state, pixels, w, h);
        buf.present().unwrap();
    }

    pub(crate) fn handle_focus_changed(&mut self, gained: bool) {
        if gained {
            self.state.swallow_next_tab = true;
        } else {
            self.modifiers = Modifiers::default();
        }
        let active_tab = self.state.active_tab;
        let tab_active = self.state.tabs[active_tab].active;
        self.send_pane_focus_seq(active_tab, tab_active, gained);
    }

    pub(crate) fn handle_redraw_requested(&mut self, event_loop: &ActiveEventLoop) {
        let exited = self.drain_effects();
        for (tab_idx, pane_id) in exited {
            self.close_pane_on_tab(tab_idx, pane_id, event_loop);
        }
        self.redraw();
    }
}
