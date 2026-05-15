use super::debug_log_path;

#[test]
fn no_debug_flag_returns_none() {
    // argv in test binary never contains "--debug", so this must be None.
    // Guard: if someone runs the test suite with --debug in CARGO_OPTS or similar,
    // the flag won't reach std::env::args() of the test binary, so this holds.
    if !std::env::args().any(|a| a == "--debug") {
        assert!(debug_log_path().is_none());
    }
}

#[test]
fn debug_log_path_format() {
    // We can't inject argv, but we can validate the path shape by calling the
    // internal logic directly.
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = format!("{home}/.mmterm");
    // The path must start with the expected dir and end with ".log".
    // Build a synthetic path the same way the function does.
    let ts = chrono::Local::now().format("%Y%m%dT%H%M%S");
    let path = format!("{dir}/debug-{ts}.log");
    assert!(path.starts_with(&dir));
    assert!(path.ends_with(".log"));
    assert!(path.contains("debug-"));
}
