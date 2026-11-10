//! Ctrl+[ / Ctrl+] select precedent and dependent cells (#1865).

use egui::{Event, Key, Modifiers};
use serde_json::json;
use simplegrid_engine::Session;
use simplegrid_engine::core::CellRef;
use simplegrid_ui_egui::SheetApp;

type Harness = egui_kittest::Harness<'static, SheetApp>;

fn harness(session: Session) -> Harness {
    crate::support::harness(session, 4)
}

fn active(h: &Harness) -> String {
    let d = h.state().session.active().unwrap();
    d.selection.active.a1()
}

fn ranges(h: &Harness) -> Vec<String> {
    let d = h.state().session.active().unwrap();
    d.selection.ranges.iter().map(|r| r.a1()).collect()
}

fn key(h: &mut Harness, k: Key, m: Modifiers) {
    h.input_mut().events.push(Event::Key { key: k, physical_key: Some(k), pressed: true, repeat: false, modifiers: m });
    h.input_mut().events.push(Event::Key { key: k, physical_key: Some(k), pressed: false, repeat: false, modifiers: m });
    h.run_steps(2);
}

/// A1 and B1 feed C1; C1 feeds D1.
fn chain() -> Session {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("range.setValues", json!({"range": "A1", "values": [[1, 2, 3]]})).unwrap();
    s.execute("cell.set", json!({"cell": "C1", "input": "=A1+B1"})).unwrap();
    s.execute("cell.set", json!({"cell": "D1", "input": "=C1*2"})).unwrap();
    s
}

#[test]
fn no_precedents_leaves_selection_untouched() {
    let mut h = harness(chain());
    h.state_mut().session.execute("selection.set", json!({"range": "A1"})).unwrap();
    key(&mut h, Key::OpenBracket, Modifiers::CTRL);
    assert_eq!(active(&h), "A1", "a plain entry cell has no precedents to select");
}
