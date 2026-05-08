use super::*;

fn make_grid(cols: usize, rows: usize) -> Grid {
    Grid::with_colors(
        cols, rows,
        Color::WHITE, Color::BLACK,
        Color::CURSOR, Color::SELECTION,
        [Color::BLACK; 16],
    )
}

#[test]
fn write_char_stores_and_advances_cursor() {
    let mut g = make_grid(10, 5);
    g.write_char('A');
    assert_eq!(g.cell(0, 0).c, 'A');
    assert_eq!(g.cursor_col, 1);
}

#[test]
fn write_char_wraps_to_next_row() {
    let mut g = make_grid(3, 5);
    g.write_char('A');
    g.write_char('B');
    g.write_char('C');
    // cursor_col is now 3; next write wraps
    g.write_char('D');
    assert_eq!(g.cell(0, 1).c, 'D');
    assert_eq!(g.cursor_col, 1);
    assert_eq!(g.cursor_row, 1);
}

#[test]
fn scroll_up_pushes_line_to_scrollback() {
    let mut g = make_grid(4, 3);
    g.write_char('A');
    assert_eq!(g.scrollback_len(), 0);
    g.scroll_up(1);
    assert_eq!(g.scrollback_len(), 1);
    assert_eq!(g.scrollback[0][0].c, 'A');
}

#[test]
fn scroll_down_shifts_content() {
    let mut g = make_grid(4, 3);
    g.write_char('A');
    g.cursor_col = 0;
    g.cursor_row = 1;
    g.write_char('B');
    g.scroll_down(1);
    assert_eq!(g.cell(0, 0).c, ' ');
    assert_eq!(g.cell(0, 1).c, 'A');
}

#[test]
fn resize_preserves_existing_content() {
    let mut g = make_grid(5, 5);
    g.write_char('X');
    g.resize(8, 8);
    assert_eq!(g.cell(0, 0).c, 'X');
    assert_eq!(g.cols, 8);
    assert_eq!(g.rows, 8);
}

#[test]
fn selected_text_single_row() {
    let mut g = make_grid(10, 5);
    for c in "hello".chars() { g.write_char(c); }
    assert_eq!(g.selected_text(0, 0, 4, 0), "hello");
}

#[test]
fn selected_text_reversed_selection() {
    let mut g = make_grid(10, 5);
    for c in "hello".chars() { g.write_char(c); }
    assert_eq!(g.selected_text(4, 0, 0, 0), "hello");
}

#[test]
fn selected_text_trims_trailing_spaces() {
    let mut g = make_grid(10, 5);
    g.write_char('H');
    g.write_char('i');
    assert_eq!(g.selected_text(0, 0, 9, 0), "Hi");
}

#[test]
fn alternate_screen_saves_and_restores() {
    let mut g = make_grid(10, 5);
    g.write_char('A');
    g.enter_alternate_screen();
    assert_eq!(g.cell(0, 0).c, ' ');
    g.write_char('B');
    g.exit_alternate_screen();
    assert_eq!(g.cell(0, 0).c, 'A');
}

#[test]
fn alternate_screen_double_enter_is_noop() {
    let mut g = make_grid(10, 5);
    g.write_char('A');
    g.enter_alternate_screen();
    g.enter_alternate_screen();
    g.exit_alternate_screen();
    assert_eq!(g.cell(0, 0).c, 'A');
}

#[test]
fn clear_line_blanks_row() {
    let mut g = make_grid(5, 3);
    g.write_char('X');
    g.clear_line(0);
    assert_eq!(g.cell(0, 0).c, ' ');
}

#[test]
fn clear_screen_blanks_all_cells() {
    let mut g = make_grid(5, 3);
    g.write_char('X');
    g.clear_screen();
    for row in 0..3 {
        for col in 0..5 {
            assert_eq!(g.cell(col, row).c, ' ');
        }
    }
}

#[test]
fn scrollback_len_increments_on_scroll_up() {
    let mut g = make_grid(4, 2);
    assert_eq!(g.scrollback_len(), 0);
    g.scroll_up(1);
    assert_eq!(g.scrollback_len(), 1);
    g.scroll_up(2);
    assert_eq!(g.scrollback_len(), 3);
}
