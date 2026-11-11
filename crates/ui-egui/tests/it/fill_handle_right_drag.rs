//! Dragging the fill handle with the right button fills nothing on release: it opens Excel's fill
//! menu (Copy Cells, Fill Series, Fill Formatting Only, Fill Without Formatting, Series…, and from
//! a date Fill Days, Fill Weekdays, Fill Months and Fill Years).

use egui::{Event, Key, Modifiers, PointerButton, Pos2, vec2};
use egui_kittest::kittest::Queryable;
use serde_json::json;
use simplegrid_engine::Session;
use simplegrid_engine::core::{CellRef, RangeRef, Value};
use simplegrid_ui_egui::{SheetApp, grid::Drag, grid::Geo};

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
    geo.cell_rect(sheet, CellRef::parse(cell).unwrap()).right_bottom() - vec2(1.0, 1.0)
}

fn center(h: &Harness, cell: &str) -> Pos2 {
    let app = h.state();
    let sheet = app.session.active().unwrap().wb.active().unwrap();
    let geo = Geo::new(sheet, app.grid.rect.unwrap(), app.view().scroll);
    geo.cell_rect(sheet, CellRef::parse(cell).unwrap()).center()
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

fn escape(h: &mut Harness) {
    h.input_mut().events.push(Event::Key { key: Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE });
    h.step();
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

fn fills(h: &Harness) -> usize {
    h.state().session.journal.iter().filter(|(id, _)| id == "edit.autoFill").count()
}

fn selection(h: &Harness) -> String {
    h.state().session.active().unwrap().selection.a1()
}

fn n(x: f64) -> Value {
    Value::Number(x)
}





/// B1:B2 = 1, 2 in bold, B4 = 9 in italic.
fn formatted(s: &mut Session) {
    s.execute("range.setValues", json!({"range": "B1", "values": [[1], [2], [null], [9]]})).unwrap();
    s.execute("home.bold", json!({"range": "B1:B2", "on": true})).unwrap();
    s.execute("home.italic", json!({"range": "B4", "on": true})).unwrap();
    s.execute("selection.set", json!({"range": "B1:B2"})).unwrap();
}









#[test]
fn right_click_on_the_handle_opens_the_context_menu() {
    let mut h = harness(one_two);
    let at = corner(&h, "B2");
    press(&mut h, at, PointerButton::Secondary);
    release(&mut h, at, PointerButton::Secondary);

    assert!(h.state().grid.context_menu.is_some());
    assert!(h.state().grid.fill_menu.is_none());
}
