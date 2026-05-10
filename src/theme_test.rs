use super::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn install_bundled_themes_creates_nine_files() {
    let dir = tempdir().unwrap();
    install_bundled_themes(dir.path());
    let count = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("toml"))
        .count();
    assert_eq!(count, BUNDLED.len());
}

#[test]
fn install_bundled_themes_does_not_overwrite_existing() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("catppuccin-mocha.toml");
    fs::write(&path, "sentinel").unwrap();
    install_bundled_themes(dir.path());
    assert_eq!(fs::read_to_string(&path).unwrap(), "sentinel");
}

#[test]
fn load_theme_catppuccin_mocha_parses() {
    let dir = tempdir().unwrap();
    install_bundled_themes(dir.path());
    let theme = load_theme("catppuccin-mocha", dir.path()).unwrap();
    assert_eq!(theme.background.r, 0x1e);
    assert_eq!(theme.background.g, 0x1e);
    assert_eq!(theme.background.b, 0x2e);
    assert_eq!(theme.foreground.r, 0xcd);
}

#[test]
fn load_theme_unknown_name_returns_error() {
    let dir = tempdir().unwrap();
    let result = load_theme("nonexistent", dir.path());
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(
        msg.contains("nonexistent"),
        "error should name the theme: {msg}"
    );
}

#[test]
fn load_theme_without_ui_fields_uses_palette_defaults() {
    let dir = tempdir().unwrap();
    let minimal = concat!(
        "foreground = \"#ffffff\"\n",
        "background = \"#000000\"\n",
        "color0  = \"#111111\"\n",
        "color1  = \"#ff0000\"\n",
        "color2  = \"#00ff00\"\n",
        "color3  = \"#ffff00\"\n",
        "color4  = \"#0000ff\"\n",
        "color5  = \"#ff00ff\"\n",
        "color6  = \"#00ffff\"\n",
        "color7  = \"#cccccc\"\n",
        "color8  = \"#888888\"\n",
        "color9  = \"#ff5555\"\n",
        "color10 = \"#55ff55\"\n",
        "color11 = \"#ffff55\"\n",
        "color12 = \"#5555ff\"\n",
        "color13 = \"#ff55ff\"\n",
        "color14 = \"#55ffff\"\n",
        "color15 = \"#eeeeee\"\n",
    );
    fs::write(dir.path().join("minimal.toml"), minimal).unwrap();
    let theme = load_theme("minimal", dir.path()).unwrap();
    // search_match defaults to palette[3] = yellow
    assert_eq!(theme.search_match.r, 0xff);
    assert_eq!(theme.search_match.g, 0xff);
    assert_eq!(theme.search_match.b, 0x00);
    // scrollbar defaults to palette[8] = gray
    assert_eq!(theme.scrollbar.r, 0x88);
}

#[test]
fn list_themes_returns_sorted_names() {
    let dir = tempdir().unwrap();
    install_bundled_themes(dir.path());
    let names = list_themes(dir.path());
    assert!(!names.is_empty());
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

#[test]
fn list_themes_empty_dir_returns_empty() {
    let dir = tempdir().unwrap();
    let names = list_themes(dir.path());
    assert!(names.is_empty());
}

#[test]
fn default_theme_has_default_background() {
    let theme = default_theme();
    // default background is #121212
    assert_eq!(theme.background.r, 0x12);
    assert_eq!(theme.background.g, 0x12);
    assert_eq!(theme.background.b, 0x12);
}

#[test]
fn all_bundled_themes_parse_without_error() {
    let dir = tempdir().unwrap();
    install_bundled_themes(dir.path());
    for (name, _) in BUNDLED {
        let result = load_theme(name, dir.path());
        assert!(result.is_ok(), "theme {name} failed to parse: {:?}", result);
    }
}

#[test]
fn palette_has_16_entries() {
    let theme = default_theme();
    assert_eq!(theme.palette.len(), 16);
}
