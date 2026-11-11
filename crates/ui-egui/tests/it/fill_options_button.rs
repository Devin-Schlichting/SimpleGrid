//! After a left-drag of the fill handle, the Auto Fill Options button redoes the fill as Copy Cells,
//! Fill Series, Fill Formatting Only or Fill Without Formatting (#1403), and from a date as Fill
//! Days, Fill Weekdays, Fill Months or Fill Years.

use egui::{Event, Modifiers, PointerButton, Pos2, vec2};
use egui_kittest::kittest::Queryable;
use serde_json::json;
use simplegrid_engine::Session;
use simplegrid_engine::core::{CellRef, Value};
use simplegrid_ui_egui::{SheetApp, grid::Geo};

type Harness = egui_kittest::Harness<'static, SheetApp>;

fn harness(setup: impl FnOnce(&mut Session)) -> Harness {
    let mut session = Session::new();
    session.new_workbook();
    setup(&mut session);
    crate::support::stepped_harness(session)
}

/// B1:B2 holds 1, 2 and is selected.
fn one_two(s: &mut Session) {
    s.execute("range.setValues", json!({"range": "B1", "values": [[1], [2]]})).unwrap();
    s.execute("selection.set", json!({"range": "B1:B2"})).unwrap();
}

fn corner(h: &Harness, cell: &str) -> Pos2 {
    let app = h.state();
    let sheet = app.session.active().unwrap().wb.active().unwrap();
    let geo = Geo::new(sheet, app.grid.rect.unwrap(), app.view().scroll);
    geo.mirror_pos(geo.cell_rect(sheet, CellRef::parse(cell).unwrap()).right_bottom() - vec2(1.0, 1.0))
}

fn center(h: &Harness, cell: &str) -> Pos2 {
    let app = h.state();
    let sheet = app.session.active().unwrap().wb.active().unwrap();
    let geo = Geo::new(sheet, app.grid.rect.unwrap(), app.view().scroll);
    geo.screen_cell_rect(sheet, CellRef::parse(cell).unwrap()).center()
}

fn press(h: &mut Harness, pos: Pos2, button: PointerButton) {
    h.input_mut().events.extend([Event::PointerMoved(pos), Event::PointerButton { pos, button, pressed: true, modifiers: Modifiers::NONE }]);
    h.step();
}

fn move_to(h: &mut Harness, from: Pos2, to: Pos2) {
    for i in 1..=10 {
        h.input_mut().events.push(Event::PointerMoved(from + (to - from) * (i as f32 / 10.0)));
        h.step();
    }
}

fn release(h: &mut Harness, pos: Pos2, button: PointerButton) {
    h.input_mut().events.push(Event::PointerButton { pos, button, pressed: false, modifiers: Modifiers::NONE });
    h.run_steps(3);
}

/// Drags the handle of the current selection (whose bottom-right cell is `from`) to `to`.
fn drag_handle(h: &mut Harness, from: &str, to: &str, button: PointerButton) {
    let (start, end) = (corner(h, from), center(h, to));
    press(h, start, button);
    move_to(h, start, end);
    release(h, end, button);
}

fn value(h: &Harness, cell: &str) -> Value {
    h.state().session.active().unwrap().wb.active().unwrap().value(CellRef::parse(cell).unwrap())
}

fn values(h: &Harness, cells: &[&str]) -> Vec<Value> {
    cells.iter().map(|c| value(h, c)).collect()
}

fn bold(h: &Harness, cell: &str) -> bool {
    let d = h.state().session.active().unwrap();
    let sh = d.wb.active().unwrap();
    d.wb.styles.get(sh.style_id(CellRef::parse(cell).unwrap())).font.bold
}

fn italic(h: &Harness, cell: &str) -> bool {
    let d = h.state().session.active().unwrap();
    let sh = d.wb.active().unwrap();
    d.wb.styles.get(sh.style_id(CellRef::parse(cell).unwrap())).font.italic
}

fn selection(h: &Harness) -> String {
    h.state().session.active().unwrap().selection.a1()
}

fn n(x: f64) -> Value {
    Value::Number(x)
}






fn formatted(s: &mut Session) {
    s.execute("range.setValues", json!({"range": "B1", "values": [[1], [2], [null], [9]]})).unwrap();
    s.execute("home.bold", json!({"range": "B1:B2", "on": true})).unwrap();
    s.execute("home.italic", json!({"range": "B4", "on": true})).unwrap();
    s.execute("selection.set", json!({"range": "B1:B2"})).unwrap();
}



#[test]
fn right_drag_makes_no_button() {
    let mut h = harness(one_two);
    drag_handle(&mut h, "B2", "B6", PointerButton::Secondary);
    assert!(h.query_by_label("Auto Fill Options").is_none());
}



/// Right-to-left sheet with column A made `chars` wide (so column B's right edge, where the fill
/// range B1:B6 ends, sits far enough right to be scrolled to either edge of the cells area).
fn rtl_wide_a(chars: u32) -> Harness {
    harness(|s| {
        one_two(s);
        s.execute("home.columnWidth", json!({"cols": "A:A", "chars": chars})).unwrap();
        s.execute("view.rightToLeft", json!({"on": true})).unwrap();
    })
}

/// Logical right edge of the filled column B (scroll-dependent) and the current geometry.
fn b_right(h: &Harness) -> (f32, Geo) {
    let app = h.state();
    let sheet = app.session.active().unwrap().wb.active().unwrap();
    let geo = Geo::new(sheet, app.grid.rect.unwrap(), app.view().scroll);
    (geo.cell_rect(sheet, CellRef::parse("B6").unwrap()).right(), geo)
}

fn scroll_b_right_to(h: &mut Harness, logical_x: f32) {
    let (now, geo) = b_right(h);
    h.state_mut().view_mut().unwrap().scroll.x += (now - logical_x) / geo.z;
    h.run_steps(3);
    let got = b_right(h).0;
    assert!((got - logical_x).abs() < 1.0, "setup: column B ends at logical x {got}, wanted {logical_x}");
}


