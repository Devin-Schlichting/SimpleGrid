//! Ctrl+Shift+= (or Ctrl with the numpad's +) and Ctrl+- on whole rows or columns insert or delete them at once, as in Excel;
//! any other selection opens the Insert or Delete dialog.

use egui::{Key, Modifiers};
use serde_json::json;
use simplegrid_engine::Session;
use simplegrid_engine::core::{CellRef, Value};
use simplegrid_ui_egui::SheetApp;

type Harness = egui_kittest::Harness<'static, SheetApp>;

fn harness(selection: &str) -> Harness {
    let mut session = Session::new();
    session.new_workbook();
    session.execute("range.setValues", json!({"range": "A1", "values": [["a1", "b1"], ["a2", "b2"], ["a3", "b3"]]})).unwrap();
    session.execute("selection.set", json!({"range": selection})).unwrap();
    let mut h = crate::common::builder().with_size(egui::vec2(1200.0, 800.0)).build_ui_state(
        |ui, app: &mut SheetApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        crate::common::app(session),
    );
    h.run_steps(4);
    h
}

fn value(h: &Harness, a1: &str) -> Value {
    h.state().session.active().unwrap().wb.active().unwrap().value(CellRef::parse(a1).unwrap())
}

fn press(h: &mut Harness, key: Key, shift: bool) {
    let modifiers = Modifiers { shift, ..Modifiers::COMMAND };
    h.key_press_modifiers(modifiers, key);
    h.run_steps(3);
}

#[test]
fn unshifted_equals_inserts_nothing() {
    let mut h = harness("2:2");
    press(&mut h, Key::Equals, false);
    assert_eq!(value(&h, "A2"), Value::from("a2"));
}
