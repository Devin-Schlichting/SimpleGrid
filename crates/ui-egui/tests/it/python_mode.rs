//! Python mode in the formula bar: entering and leaving it, its keys (applied in input order, also
//! several in one frame), committing through `python.set`, and the PY badge.

use egui::{Event, Key, Modifiers, vec2};
use egui_kittest::kittest::Queryable as _;
use simplegrid_engine::Session;
use simplegrid_engine::core::{CellError, CellRef, Value};
use serde_json::json;

use crate::common::Harness;

fn blank() -> Session {
    let mut s = Session::new();
    s.new_workbook();
    s
}

fn harness(session: Session) -> Harness {
    crate::common::harness_with(crate::common::builder().with_size(vec2(1200.0, 800.0)), crate::common::app(session), 4)
}

fn mac_harness(session: Session) -> Harness {
    crate::common::os_harness(session, egui::os::OperatingSystem::Mac)
}

fn press(key: Key, modifiers: Modifiers) -> [Event; 2] {
    [
        Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers },
        Event::Key { key, physical_key: None, pressed: false, repeat: false, modifiers },
    ]
}

/// Sends `events` in one frame, then lets the app settle.
fn send(h: &mut Harness, events: impl IntoIterator<Item = Event>) {
    h.input_mut().events.extend(events);
    h.run_steps(3);
}

fn type_text(h: &mut Harness, text: &str) {
    send(h, [Event::Text(text.into())]);
}

const CTRL_ALT_SHIFT: Modifiers = Modifiers { alt: true, ctrl: true, shift: true, mac_cmd: false, command: true };
const CMD_OPTION_SHIFT: Modifiers = Modifiers { alt: true, ctrl: false, shift: true, mac_cmd: true, command: true };

fn shortcut(h: &mut Harness) {
    send(h, press(Key::P, CTRL_ALT_SHIFT));
}

fn ctrl_enter(h: &mut Harness) {
    send(h, press(Key::Enter, Modifiers::COMMAND));
}

/// Whether an edit is open, and in Python mode.
fn mode(h: &Harness) -> Option<bool> {
    h.state().editor.as_ref().map(|e| e.python)
}

fn text(h: &Harness) -> String {
    h.state().editor.as_ref().map(|e| e.text.clone()).unwrap_or_default()
}

fn code(h: &Harness, cell: &str) -> Option<String> {
    let d = h.state().session.active()?;
    d.wb.active()?.cell(CellRef::parse(cell)?)?.py.as_ref().map(|p| p.code.clone())
}

fn value(h: &Harness, cell: &str) -> Value {
    let d = h.state().session.active().unwrap();
    d.wb.active().unwrap().value(CellRef::parse(cell).unwrap())
}



#[test]
fn typing_py_then_tab_enters_python_mode_as_in_excel() {
    let mut h = harness(blank());
    type_text(&mut h, "=PY");
    assert_eq!(mode(&h), Some(false), "typing starts an ordinary entry in the cell");
    send(&mut h, press(Key::Tab, Modifiers::NONE));
    assert_eq!(mode(&h), Some(true), "=PY then Tab is Python mode");
    assert_eq!(text(&h), "", "the trigger is not code");
    assert!(h.state().editor.as_ref().is_some_and(|e| e.from_formula_bar));
    let active = h.state().session.active().map(|d| d.selection.active);
    assert_eq!(active, Some(CellRef::new(0, 0)), "Tab did not move on");
    type_text(&mut h, "len('abc')");
    ctrl_enter(&mut h);
    assert_eq!(code(&h, "A1").as_deref(), Some("len('abc')"));
    assert_eq!(value(&h, "A1"), Value::Error(CellError::Blocked), "this build stores the code and doesn't run it");
    // Any other entry still commits with Tab.
    h.state_mut().session.execute("selection.set", json!({"cell": "B2"})).unwrap();
    h.run_steps(2);
    type_text(&mut h, "=PI");
    send(&mut h, press(Key::Tab, Modifiers::NONE));
    assert_eq!(mode(&h), None);
}

/// A cell inside a spill: typing there is an ordinary entry, as on any spill child, and neither
/// `=PY` then Tab nor the shortcut turns that draft into code for the cell, as Python mode starts
/// on no cell inside a spill.
#[test]
fn a_draft_typed_inside_a_spill_does_not_enter_python_mode() {
    let mut s = blank();
    s.execute("cell.set", json!({"cell": "A1", "input": "=SEQUENCE(3)"})).unwrap();
    s.execute("selection.set", json!({"cell": "A2"})).unwrap();
    let mut h = harness(s);
    type_text(&mut h, "=PY");
    assert_eq!(mode(&h), Some(false), "typing starts an ordinary entry");
    send(&mut h, press(Key::Tab, Modifiers::NONE));
    assert_eq!(mode(&h), Some(false), "=PY then Tab");
    shortcut(&mut h);
    assert_eq!(mode(&h), Some(false), "the shortcut");
    ctrl_enter(&mut h);
    assert_eq!(code(&h, "A2"), None, "no code is written into the spill");
}






#[test]
fn ui_python_mode_switches_from_the_control_channel_without_a_dialog() {
    let mut h = harness(blank());
    let r = h.state_mut().run("ui.pythonMode", json!({})).unwrap();
    assert_eq!(r["on"], true);
    assert_eq!(h.state_mut().run("ui.pythonMode", json!({"on": true})).unwrap()["on"], true, "already on");
    assert_eq!(h.state_mut().run("ui.pythonMode", json!({"on": false})).unwrap()["on"], false);
    assert!(h.state().dialog.is_none() && h.state().message.is_none());
    h.state_mut().cancel_edit();
    assert_eq!(h.state_mut().run("ui.pythonMode", json!({"on": false})).unwrap()["on"], false, "off with no edit does nothing");
    assert!(h.state().editor.is_none());
}

#[test]
fn a_refused_ui_python_mode_start_is_an_error_with_the_reason() {
    let mut s = blank();
    s.execute("cell.set", json!({"cell": "A1", "input": "=SEQUENCE(3)"})).unwrap();
    s.execute("python.set", json!({"cell": "C1", "code": "secret = 1"})).unwrap();
    s.execute("home.formatCells", json!({"range": "C1", "style": {"protection": {"locked": false, "hidden": true}}})).unwrap();
    s.execute("review.protectSheet", json!({})).unwrap();
    let mut h = harness(s);
    // A locked cell (B1), a cell inside a spill (A2), code hidden on a protected sheet (C1), and a
    // draft typed inside a spill: each is refused with an error, by `on: true` and by a switch.
    for (cell, typed, says) in [("B1", None, "protected"), ("A2", None, "spill"), ("C1", None, "hidden"), ("A2", Some("5"), "spill")] {
        h.state_mut().cancel_edit();
        h.state_mut().run("ui.message.dismiss", json!({})).unwrap();
        h.state_mut().run("selection.set", json!({"cell": cell})).unwrap();
        if let Some(text) = typed {
            h.state_mut().begin_edit(Some(text.into()), false);
        }
        for params in [json!({"on": true}), json!({})] {
            let e = h.state_mut().run("ui.pythonMode", params.clone()).expect_err(cell);
            assert!(e.contains(says), "{cell} {params}: {e}");
            assert_ne!(mode(&h), Some(true), "{cell}");
        }
    }
}

/// Clicks the middle of `a1` on screen.
fn click_cell(h: &mut Harness, a1: &str) {
    let at = {
        let app = h.state();
        let sh = app.session.active().unwrap().wb.active().unwrap();
        app.grid.geo.as_ref().unwrap().screen_cell_rect(sh, CellRef::parse(a1).unwrap()).center()
    };
    for event in [
        Event::PointerMoved(at),
        Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
        Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE },
    ] {
        h.input_mut().events.push(event);
        h.run_steps(1);
    }
    h.run_steps(2);
}





