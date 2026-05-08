use super::*;

const W: u32 = 800;
const H: u32 = 600;

#[test]
fn new_layout_has_single_leaf() {
    let layout = Layout::new(0, W, H);
    assert_eq!(layout.leaves(), vec![0]);
}

#[test]
fn split_h_creates_two_panes() {
    let mut layout = Layout::new(0, W, H);
    layout.split(0, 1, SplitDir::H);
    let leaves = layout.leaves();
    assert_eq!(leaves.len(), 2);
    assert!(leaves.contains(&0));
    assert!(leaves.contains(&1));
}

#[test]
fn split_v_creates_two_panes() {
    let mut layout = Layout::new(0, W, H);
    layout.split(0, 1, SplitDir::V);
    assert_eq!(layout.leaves().len(), 2);
}

#[test]
fn split_h_rects_widths_sum_to_total() {
    let mut layout = Layout::new(0, W, H);
    layout.split(0, 1, SplitDir::H);
    let rects = layout.rects();
    let total: u32 = rects.iter().map(|(_, r)| r[2]).sum::<u32>() + SEP;
    assert_eq!(total, W);
}

#[test]
fn split_v_rects_heights_sum_to_usable() {
    let mut layout = Layout::new(0, W, H);
    layout.split(0, 1, SplitDir::V);
    let rects = layout.rects();
    let usable = H - STATUS_BAR_H - TAB_BAR_H;
    let total: u32 = rects.iter().map(|(_, r)| r[3]).sum::<u32>() + SEP;
    assert_eq!(total, usable);
}

#[test]
fn single_pane_rect_spans_full_width() {
    let layout = Layout::new(0, W, H);
    let rects = layout.rects();
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].1[0], 0);
    assert_eq!(rects[0].1[2], W);
}

#[test]
fn remove_returns_sibling_id() {
    let mut layout = Layout::new(0, W, H);
    layout.split(0, 1, SplitDir::H);
    let sibling = layout.remove(0);
    assert_eq!(sibling, Some(1));
    assert_eq!(layout.leaves(), vec![1]);
}

#[test]
fn remove_last_pane_returns_none() {
    let mut layout = Layout::new(0, W, H);
    let result = layout.remove(0);
    assert_eq!(result, None);
}

#[test]
fn separators_empty_for_single_pane() {
    let layout = Layout::new(0, W, H);
    assert!(layout.separators().is_empty());
}

#[test]
fn separators_one_for_split() {
    let mut layout = Layout::new(0, W, H);
    layout.split(0, 1, SplitDir::H);
    assert_eq!(layout.separators().len(), 1);
}

#[test]
fn focus_dir_right() {
    let mut layout = Layout::new(0, W, H);
    layout.split(0, 1, SplitDir::H);
    assert_eq!(layout.focus_dir(0, 1, 0), Some(1));
}

#[test]
fn focus_dir_left() {
    let mut layout = Layout::new(0, W, H);
    layout.split(0, 1, SplitDir::H);
    assert_eq!(layout.focus_dir(1, -1, 0), Some(0));
}

#[test]
fn focus_dir_no_pane_returns_none() {
    let mut layout = Layout::new(0, W, H);
    layout.split(0, 1, SplitDir::H);
    assert_eq!(layout.focus_dir(0, -1, 0), None);
}

#[test]
fn resize_updates_dimensions() {
    let mut layout = Layout::new(0, W, H);
    layout.resize(1024, 768);
    assert_eq!(layout.width, 1024);
    assert_eq!(layout.height, 768);
}

#[test]
fn focus_dir_up_and_down() {
    let mut layout = Layout::new(0, W, H);
    layout.split(0, 1, SplitDir::V);
    assert_eq!(layout.focus_dir(0, 0, 1), Some(1));
    assert_eq!(layout.focus_dir(1, 0, -1), Some(0));
}

#[test]
fn v_split_separator_is_horizontal() {
    let mut layout = Layout::new(0, W, H);
    layout.split(0, 1, SplitDir::V);
    let seps = layout.separators();
    assert_eq!(seps.len(), 1);
    // horizontal separator spans full width
    assert_eq!(seps[0][2], W);
    assert_eq!(seps[0][3], SEP);
}

#[test]
fn remove_second_pane_leaves_first() {
    let mut layout = Layout::new(0, W, H);
    layout.split(0, 1, SplitDir::H);
    let sibling = layout.remove(1);
    assert_eq!(sibling, Some(0));
    assert_eq!(layout.leaves(), vec![0]);
}

#[test]
fn nested_split_has_three_panes() {
    let mut layout = Layout::new(0, W, H);
    layout.split(0, 1, SplitDir::H);
    layout.split(1, 2, SplitDir::V);
    assert_eq!(layout.leaves().len(), 3);
    assert_eq!(layout.separators().len(), 2);
}
