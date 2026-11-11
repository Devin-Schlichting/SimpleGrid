//! A number stored as text gets Excel's green corner and a warning button whose menu converts it (#1994).

use egui_kittest::kittest::Queryable as _;
use serde_json::json;
use simplegrid_engine::Session;
use simplegrid_ui_egui::SheetApp;

type Harness = egui_kittest::Harness<'static, SheetApp>;

fn harness(active: &str) -> Harness {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("cell.set", json!({"cell": "B2", "input": "'123"})).unwrap();
    s.execute("cell.set", json!({"cell": "B3", "input": "'abc"})).unwrap();
    s.execute("selection.set", json!({"cell": active})).unwrap();
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


#[test]
fn text_that_is_not_a_number_gets_no_button() {
    let h = harness("B3");
    assert!(h.query_by_label("Error checking options").is_none());
}
