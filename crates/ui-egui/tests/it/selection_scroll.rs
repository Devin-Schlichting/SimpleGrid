//! Extending the selection from the keyboard scrolls its moving end into view, not the anchor.
//! Go To and Select over the control channel scroll the active cell into view.

use egui::{Event, Key, Modifiers};
use serde_json::{Value as Json, json};
use simplegrid_engine::Session;
use simplegrid_engine::core::{CellRef, RangeRef};
use simplegrid_ui_egui::control::{Outcome, handle};
use simplegrid_ui_egui::{SheetApp, grid::Geo};

type Harness = egui_kittest::Harness<'static, SheetApp>;

/// Ctrl on Windows and Linux sets both `ctrl` and `command`.
const CTRL: Modifiers = Modifiers { alt: false, ctrl: true, shift: false, mac_cmd: false, command: true };
const CTRL_SHIFT: Modifiers = Modifiers { shift: true, ..CTRL };

/// Column A filled down to A200 and row 1 filled across to CV1, far past one screen each way.
fn harness() -> Harness {
    let mut s = Session::new();
    s.new_workbook();
    let down: Vec<Vec<i64>> = (1..=200).map(|r| vec![r]).collect();
    s.execute("range.setValues", json!({"range": "A1", "values": down})).unwrap();
    let across: Vec<i64> = (1..=100).collect();
    s.execute("range.setValues", json!({"range": "A1", "values": [across]})).unwrap();
    s.execute("selection.set", json!({"cell": "A1"})).unwrap();
    let mut h = crate::common::builder().with_size(egui::vec2(1200.0, 800.0)).build_ui_state(
        |ui, app: &mut SheetApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        crate::common::app(s),
    );
    h.run_steps(4);
    h
}

fn key(h: &mut Harness, k: Key, m: Modifiers) {
    h.input_mut().events.push(Event::Key { key: k, physical_key: Some(k), pressed: true, repeat: false, modifiers: m });
    h.input_mut().events.push(Event::Key { key: k, physical_key: Some(k), pressed: false, repeat: false, modifiers: m });
    h.run_steps(2);
}

/// Whether the cell's centre is on screen (a whole cell can sit a few pixels under the edge when
/// scrolling far down widens the row headings).
fn visible(h: &Harness, a1: &str) -> bool {
    let app = h.state();
    let sheet = app.session.active().unwrap().wb.active().unwrap();
    let geo = Geo::new(sheet, app.grid.rect.unwrap(), app.view().scroll);
    geo.cells.contains(geo.cell_rect(sheet, CellRef::parse(a1).unwrap()).center())
}

fn assert_extended(h: &Harness, anchor: &str, moving_end: &str) {
    let selection = &h.state().session.active().unwrap().selection;
    let (anchor_cell, end_cell) = (CellRef::parse(anchor).unwrap(), CellRef::parse(moving_end).unwrap());
    assert_eq!(selection.current(), RangeRef::new(anchor_cell, end_cell));
    assert_eq!(selection.anchor, anchor_cell);
    assert_eq!(selection.active, anchor_cell, "extending keeps the active cell on the anchor");
    assert!(visible(h, moving_end), "the moving end {moving_end} is scrolled into view");
}

#[test]
fn shift_arrows_scroll_the_moving_end_into_view() {
    let mut h = harness();
    for _ in 0..60 {
        key(&mut h, Key::ArrowDown, Modifiers::SHIFT);
    }
    assert_extended(&h, "A1", "A61");
    for _ in 0..30 {
        key(&mut h, Key::ArrowRight, Modifiers::SHIFT);
    }
    assert_extended(&h, "A1", "AE61");
}

fn control(h: &mut Harness, method: &str, params: Json) -> Json {
    let ctx = h.ctx.clone();
    let out = match handle(h.state_mut(), &ctx, method, &params) {
        Outcome::Done(v) => v,
        Outcome::Screenshot { .. } => json!(null),
    };
    h.run_steps(2);
    out
}

fn active(h: &Harness) -> CellRef {
    h.state().session.active().unwrap().selection.active
}

#[test]
fn go_to_over_the_control_channel_scrolls_the_target_into_view() {
    let mut h = harness();
    assert!(!visible(&h, "Z300"));
    let r = control(&mut h, "engine.execute", json!({"command": "edit.goTo", "params": {"reference": "Z300"}}));
    assert_eq!(r["ok"], json!(true), "{r}");
    assert_eq!(active(&h), CellRef::parse("Z300").unwrap());
    assert!(visible(&h, "Z300"), "Go To scrolls Z300 into view");
    assert!(!visible(&h, "A1"));
    // A command named as the method runs the same way.
    control(&mut h, "edit.goTo", json!({"reference": "A1"}));
    assert!(visible(&h, "A1") && !visible(&h, "Z300"));
}

#[test]
fn select_over_the_control_channel_scrolls_the_active_cell_into_view() {
    let mut h = harness();
    control(&mut h, "engine.execute", json!({"command": "selection.set", "params": {"range": "BZ150:CA160"}}));
    assert_eq!(active(&h), CellRef::parse("BZ150").unwrap());
    assert!(visible(&h, "BZ150") && !visible(&h, "A1"), "selecting a far range scrolls its active cell into view");
}

fn scroll(h: &Harness) -> egui::Vec2 {
    h.state().view().scroll
}

#[test]
fn page_down_and_back_up_from_row_10_shows_rows_2_to_9_again() {
    // The reported steps: row 1 frozen, row 10 selected, Page Down, Page Up, Page Up.
    let mut h = harness();
    h.state_mut().session.execute("view.freezePanes", json!({"cell": "A2"})).unwrap();
    h.state_mut().session.execute("selection.set", json!({"cell": "A10"})).unwrap();
    h.run_steps(2);
    key(&mut h, Key::PageDown, Modifiers::NONE);
    key(&mut h, Key::PageUp, Modifiers::NONE);
    assert_eq!(active(&h), CellRef::parse("A10").unwrap());
    key(&mut h, Key::PageUp, Modifiers::NONE);
    assert_eq!(active(&h), CellRef::parse("A1").unwrap());
    assert_eq!(scroll(&h).y, 0.0);
    assert!(visible(&h, "A2") && visible(&h, "A9"));
}

#[test]
fn other_reveals_onto_a_frozen_cell_leave_the_scrolled_rows_where_they_are() {
    // Only arrow and paging keys bring the scrolled rows back; selecting a header cell from the
    // control channel (as Find, Go To or Undo would) keeps the place the user scrolled to.
    let mut h = harness();
    h.state_mut().session.execute("view.freezePanes", json!({"cell": "A2"})).unwrap();
    key(&mut h, Key::ArrowDown, CTRL);
    let down = scroll(&h).y;
    control(&mut h, "engine.execute", json!({"command": "selection.set", "params": {"cell": "B1"}}));
    assert_eq!(active(&h), CellRef::parse("B1").unwrap());
    assert_eq!(scroll(&h).y, down);
}

#[test]
fn down_from_a_frozen_header_row_goes_to_row_2_at_the_top_from_far_down() {
    // Excel's quick way back to the top: select a cell in the frozen header row while far down,
    // press Down, and the view jumps to row 2 under the header. Selecting the header cell alone
    // doesn't scroll.
    let mut h = harness();
    h.state_mut().session.execute("view.freezePanes", json!({"cell": "A2"})).unwrap();
    h.state_mut().session.execute("selection.set", json!({"cell": "A100000"})).unwrap();
    h.state_mut().grid.ensure_visible = true;
    h.run_steps(2);
    assert!(!visible(&h, "A2") && visible(&h, "A100000"));
    h.state_mut().session.execute("selection.set", json!({"cell": "A1"})).unwrap();
    h.run_steps(2);
    assert!(visible(&h, "A100000"));
    key(&mut h, Key::ArrowDown, Modifiers::NONE);
    assert_eq!(active(&h), CellRef::parse("A2").unwrap());
    assert_eq!(scroll(&h).y, 0.0);
    assert!(visible(&h, "A2"));
}

#[test]
fn shift_arrows_on_a_whole_column_leave_the_rows_where_they_are() {
    // A whole column's moving end is on the sheet's last row: Shift+Right from Ctrl+Space must
    // widen it to the next columns without scrolling down to the bottom of the sheet.
    let mut h = harness();
    h.state_mut().session.execute("selection.set", json!({"cell": "A100"})).unwrap();
    h.state_mut().grid.ensure_visible = true;
    h.run_steps(2);
    let down = scroll(&h).y;
    assert!(down > 0.0 && visible(&h, "A100"));
    h.state_mut().session.execute("selection.column", json!({})).unwrap();
    h.run_steps(2);
    key(&mut h, Key::ArrowRight, Modifiers::SHIFT);
    assert_eq!(h.state().session.active().unwrap().selection.current(), RangeRef::parse("A:B").unwrap());
    assert_eq!(scroll(&h).y, down, "the view stays on the rows it showed");
    // The columns still scroll to follow the extension.
    for _ in 0..30 {
        key(&mut h, Key::ArrowRight, Modifiers::SHIFT);
    }
    assert!(visible(&h, "AF100") && !visible(&h, "A100"));
    assert_eq!(scroll(&h).y, down);
}

#[test]
fn shift_arrows_on_a_whole_row_leave_the_columns_where_they_are() {
    let mut h = harness();
    h.state_mut().session.execute("selection.set", json!({"cell": "CA1"})).unwrap();
    h.state_mut().grid.ensure_visible = true;
    h.run_steps(2);
    let right = scroll(&h).x;
    assert!(right > 0.0 && visible(&h, "CA1"));
    h.state_mut().session.execute("selection.row", json!({})).unwrap();
    h.run_steps(2);
    key(&mut h, Key::ArrowDown, Modifiers::SHIFT);
    assert_eq!(h.state().session.active().unwrap().selection.current(), RangeRef::parse("1:2").unwrap());
    assert_eq!(scroll(&h).x, right, "the view stays on the columns it showed");
    // The rows still scroll to follow the extension.
    for _ in 0..60 {
        key(&mut h, Key::ArrowDown, Modifiers::SHIFT);
    }
    assert!(visible(&h, "CA62") && !visible(&h, "CA1"));
    assert_eq!(scroll(&h).x, right);
}
