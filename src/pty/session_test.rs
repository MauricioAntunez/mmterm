use crossbeam_channel::unbounded;

use super::PtySession;

#[test]
fn spawn_with_shell_succeeds_with_bin_true() {
    let (tx, _rx) = unbounded();
    let session = PtySession::spawn_with_shell(80, 24, tx, "/bin/true", None, Box::new(|| {}));
    assert!(
        session.is_ok(),
        "spawn_with_shell failed: {:?}",
        session.err()
    );
}

#[test]
fn write_bytes_after_spawn_does_not_panic() {
    let (tx, _rx) = unbounded();
    let mut session = PtySession::spawn_with_shell(80, 24, tx, "/bin/sh", None, Box::new(|| {}))
        .expect("spawn failed");
    // Writing to a live shell; ignore errors (shell may exit before write).
    let _ = session.write_input(b"exit\n");
}

#[test]
fn resize_after_spawn_does_not_panic() {
    let (tx, _rx) = unbounded();
    let session = PtySession::spawn_with_shell(80, 24, tx, "/bin/sh", None, Box::new(|| {}))
        .expect("spawn failed");
    let result = session.resize(120, 40);
    assert!(result.is_ok(), "resize failed: {:?}", result.err());
}
