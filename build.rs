fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");

    let pkg = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let version = build_version(&pkg);
    println!("cargo:rustc-env=MMTERM_VERSION={version}");
}

fn build_version(pkg: &str) -> String {
    // If we are sitting exactly on a release tag, emit the bare semver so
    // release binaries are clean.  In every other situation (local dev,
    // commits after a tag, detached HEAD, no git) append the short hash so
    // the binary is self-identifying without needing external tooling.
    let on_exact_tag = std::process::Command::new("git")
        .args(["describe", "--exact-match", "HEAD"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if on_exact_tag {
        return pkg.to_string();
    }

    let short_hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    match short_hash {
        Some(hash) => format!("{pkg}+{hash}"),
        None => pkg.to_string(),
    }
}
