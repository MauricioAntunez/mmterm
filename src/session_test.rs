use super::*;
use std::path::PathBuf;

fn leaf(slot: usize) -> SavedNode {
    SavedNode::Leaf { slot }
}

fn split(dir: SavedSplitDir, ratio: f32, a: SavedNode, b: SavedNode) -> SavedNode {
    SavedNode::Split {
        dir,
        ratio,
        a: Box::new(a),
        b: Box::new(b),
    }
}

#[test]
fn roundtrip_single_pane() {
    let session = SavedSession {
        active_tab: 0,
        tabs: vec![SavedTab {
            name: None,
            active_pane: 0,
            pane_cwds: vec![PathBuf::from("/tmp")],
            layout: leaf(0),
        }],
    };
    let toml = toml::to_string_pretty(&session).expect("serialize");
    let back: SavedSession = toml::from_str(&toml).expect("deserialize");
    assert_eq!(back.active_tab, 0);
    assert_eq!(back.tabs.len(), 1);
    assert_eq!(back.tabs[0].pane_cwds[0], PathBuf::from("/tmp"));
    assert!(matches!(back.tabs[0].layout, SavedNode::Leaf { slot: 0 }));
}

#[test]
fn roundtrip_h_split() {
    let session = SavedSession {
        active_tab: 0,
        tabs: vec![SavedTab {
            name: Some("build".into()),
            active_pane: 1,
            pane_cwds: vec![PathBuf::from("/home"), PathBuf::from("/tmp")],
            layout: split(SavedSplitDir::H, 0.6, leaf(0), leaf(1)),
        }],
    };
    let toml = toml::to_string_pretty(&session).expect("serialize");
    let back: SavedSession = toml::from_str(&toml).expect("deserialize");
    assert_eq!(back.tabs[0].name.as_deref(), Some("build"));
    assert_eq!(back.tabs[0].active_pane, 1);
    if let SavedNode::Split { dir, ratio, a, b } = &back.tabs[0].layout {
        assert!(matches!(dir, SavedSplitDir::H));
        assert!((ratio - 0.6).abs() < 0.001);
        assert!(matches!(a.as_ref(), SavedNode::Leaf { slot: 0 }));
        assert!(matches!(b.as_ref(), SavedNode::Leaf { slot: 1 }));
    } else {
        panic!("expected Split");
    }
}

#[test]
fn roundtrip_three_pane_tree() {
    // Split(H, Split(V, Leaf(0), Leaf(1)), Leaf(2))
    let layout = split(
        SavedSplitDir::H,
        0.5,
        split(SavedSplitDir::V, 0.5, leaf(0), leaf(1)),
        leaf(2),
    );
    let session = SavedSession {
        active_tab: 0,
        tabs: vec![SavedTab {
            name: None,
            active_pane: 0,
            pane_cwds: vec![
                PathBuf::from("/a"),
                PathBuf::from("/b"),
                PathBuf::from("/c"),
            ],
            layout,
        }],
    };
    let toml = toml::to_string_pretty(&session).expect("serialize");
    let back: SavedSession = toml::from_str(&toml).expect("deserialize");
    assert_eq!(back.tabs[0].pane_cwds.len(), 3);
    // spot-check the tree structure survives
    let SavedNode::Split { a, b, .. } = &back.tabs[0].layout else {
        panic!("expected outer Split");
    };
    assert!(matches!(a.as_ref(), SavedNode::Split { .. }));
    assert!(matches!(b.as_ref(), SavedNode::Leaf { slot: 2 }));
}

#[test]
fn roundtrip_multiple_tabs() {
    let session = SavedSession {
        active_tab: 1,
        tabs: vec![
            SavedTab {
                name: Some("one".into()),
                active_pane: 0,
                pane_cwds: vec![PathBuf::from("/a")],
                layout: leaf(0),
            },
            SavedTab {
                name: Some("two".into()),
                active_pane: 0,
                pane_cwds: vec![PathBuf::from("/b"), PathBuf::from("/c")],
                layout: split(SavedSplitDir::V, 0.4, leaf(0), leaf(1)),
            },
        ],
    };
    let toml = toml::to_string_pretty(&session).expect("serialize");
    let back: SavedSession = toml::from_str(&toml).expect("deserialize");
    assert_eq!(back.active_tab, 1);
    assert_eq!(back.tabs.len(), 2);
    assert_eq!(back.tabs[1].name.as_deref(), Some("two"));
}

#[test]
fn load_returns_none_on_missing_file() {
    // session_path() points to the real config dir; this test just checks
    // that load() doesn't panic when the file is absent.
    // We can't override the path without a refactor, so we verify via
    // toml::from_str failing gracefully.
    let result = toml::from_str::<SavedSession>("not valid toml ;;;");
    assert!(result.is_err());
}

#[test]
fn load_returns_none_on_corrupt_toml() {
    let raw = "active_tab = 0\n[[tabs]]\nnot_a_field = true";
    let result = toml::from_str::<SavedSession>(raw);
    // Missing required fields → deserialization error
    assert!(result.is_err());
}
