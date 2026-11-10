//! Copy/cut route rich tables to the host without a later egui text write replacing them.

use std::cell::RefCell;
use std::rc::Rc;

use egui::accesskit::Role;
use egui::{Event, Modifiers, OutputCommand, PointerButton};
use egui_kittest::kittest::Queryable as _;
use serde_json::json;
use simplegrid_engine::Session;
use simplegrid_engine::core::{CellRef, Value};
use simplegrid_ui_egui::{Services, SheetApp, grid};

fn harness(services: Services) -> egui_kittest::Harness<'static, SheetApp> {
    let mut session = Session::new();
    session.new_workbook();
    session.execute("range.setValues", json!({"range": "A1", "values": [["Name", "Score"], ["Ada", 42]]})).unwrap();
    session.execute("selection.set", json!({"range": "A1:B2"})).unwrap();
    let mut h = crate::common::builder().with_size(egui::vec2(1200.0, 800.0)).build_ui_state(
        |ui, app: &mut SheetApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        crate::common::prepare(SheetApp::new(session, services)),
    );
    h.run_steps(4);
    h
}

#[test]
fn copy_and_cut_publish_html_and_text_without_overwriting_rich_clipboard() {
    for event in [Event::Copy, Event::Cut] {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let received = calls.clone();
        let mut h = harness(Services {
            copy_html: Some(Box::new(move |html, text| {
                received.borrow_mut().push((html.to_string(), text.to_string()));
                Ok(())
            })),
            ..Default::default()
        });
        h.input_mut().events.push(event.clone());
        h.step();
        let calls = calls.borrow();
        assert_eq!(calls.len(), 1);
        assert!(calls[0].0.contains("<table"));
        assert!(calls[0].0.contains("Ada"));
        assert_eq!(calls[0].1, "Name\tScore\nAda\t42\n");
        assert!(!h.output().platform_output.commands.iter().any(|c| matches!(c, OutputCommand::CopyText(_))));
        assert_eq!(h.state().session.clipboard.as_ref().map(|c| c.cut), Some(matches!(event, Event::Cut)));
    }
}

#[test]
fn missing_or_failed_rich_clipboard_falls_back_to_plain_text() {
    for fail in [false, true] {
        let mut services = Services::default();
        if fail {
            services.copy_html = Some(Box::new(|_, _| Err("clipboard unavailable".into())));
        }
        let mut h = harness(services);
        h.input_mut().events.push(Event::Copy);
        h.step();
        assert!(
            h.output().platform_output.commands.iter().any(|c| { matches!(c, OutputCommand::CopyText(text) if text == "Name\tScore\nAda\t42\n") })
        );
    }
}

#[test]
fn programmatic_copy_does_not_write_to_the_system_clipboard() {
    let calls = Rc::new(RefCell::new(0));
    let received = calls.clone();
    let mut h = harness(Services {
        copy_html: Some(Box::new(move |_, _| {
            *received.borrow_mut() += 1;
            Ok(())
        })),
        ..Default::default()
    });
    let copied = h.state_mut().run("edit.copy", json!({})).unwrap();
    assert!(copied.get("html").is_none(), "HTML is only built when requested");
    let copied = h.state_mut().run("edit.copy", json!({"html": true})).unwrap();
    assert!(copied["html"].is_string());
    assert_eq!(*calls.borrow(), 0);
    assert!(!h.output().platform_output.commands.iter().any(|c| matches!(c, OutputCommand::CopyText(_))));
}

#[test]
fn pasting_reads_the_clipboard_html_for_the_numbers_behind_the_text() {
    // #257: currency text from another spreadsheet stays a number when the host offers the HTML.
    let mut h = harness(Services {
        paste_html: Some(Box::new(|| Some(r#"<table><tr><td x:num="10000">Rp10,000.00</td></tr></table>"#.to_string()))),
        ..Default::default()
    });
    h.state_mut().run("selection.set", json!({"cell": "D5"})).unwrap();
    h.input_mut().events.push(Event::Paste("Rp10,000.00".into()));
    h.run_steps(2);
    let value = h.state().session.active().and_then(|d| d.wb.active().map(|s| s.value(simplegrid_engine::core::CellRef::new(4, 3))));
    assert_eq!(value, Some(simplegrid_engine::core::Value::number(10000.0)));
}

#[test]
fn copying_multiple_areas_copies_lined_up_areas_and_explains_the_rest() {
    // Issue #348: lined-up areas copy as one block; others get Excel's message instead of nothing.
    let mut h = harness(Services::default());
    h.state_mut().session.execute("selection.set", json!({"range": "B1:B2,A1:A2"})).unwrap();
    h.input_mut().events.push(Event::Copy);
    h.step();
    assert!(h.output().platform_output.commands.iter().any(|c| matches!(c, OutputCommand::CopyText(text) if text == "Name\tScore\nAda\t42\n")));
    assert!(h.state().message.is_none());
    assert_eq!(h.state().session.clipboard.as_ref().map(|c| c.areas.len()), Some(2));

    h.state_mut().session.execute("selection.set", json!({"range": "A1,B2"})).unwrap();
    h.input_mut().events.push(Event::Copy);
    h.step();
    assert!(!h.output().platform_output.commands.iter().any(|c| matches!(c, OutputCommand::CopyText(_))));
    assert_eq!(h.state().message.as_ref().map(|(_, text)| text.as_str()), Some("This action won't work on multiple selections."));
    assert_eq!(h.state().session.clipboard.as_ref().map(|c| c.areas.len()), Some(2), "the earlier copy stays");
}

type Harness = egui_kittest::Harness<'static, SheetApp>;

fn value_at(h: &Harness, a1: &str) -> Option<Value> {
    let at = simplegrid_engine::core::CellRef::parse(a1).unwrap();
    h.state().session.active().and_then(|d| d.wb.active().map(|s| s.value(at)))
}

/// A host clipboard holding `text`, and how many times the app read it.
fn system_clipboard(text: &str) -> (Rc<RefCell<(String, usize)>>, Services) {
    let clipboard = Rc::new(RefCell::new((text.to_string(), 0)));
    let read = clipboard.clone();
    let services = Services {
        paste_text: Some(Box::new(move || {
            let mut clipboard = read.borrow_mut();
            clipboard.1 += 1;
            Some(clipboard.0.clone())
        })),
        ..Default::default()
    };
    (clipboard, services)
}

fn press(h: &mut Harness, key: egui::Key) {
    for pressed in [true, false] {
        h.input_mut().events.push(Event::Key { key, physical_key: None, pressed, repeat: false, modifiers: Modifiers::NONE });
        if pressed && key != egui::Key::F10 {
            h.input_mut().events.push(Event::Text(key.name().to_ascii_lowercase()));
        }
    }
    h.run_steps(2);
}

fn has_formula(h: &Harness, a1: &str) -> bool {
    let at = CellRef::parse(a1).unwrap();
    h.state().session.active().and_then(|d| d.wb.active().and_then(|s| s.cell(at)).map(|c| c.formula.is_some())).unwrap_or(false)
}

/// Close the "Nothing to paste." alert so the ribbon takes clicks again.
fn dismiss(h: &mut Harness) {
    h.state_mut().message = None;
    h.state_mut().toast = None;
    h.run_steps(2);
}

fn click_paste(h: &mut Harness) {
    h.get_by_role_and_label(Role::Button, "Paste").click();
    h.run_steps(2);
}

#[test]
fn cmd_v_after_a_pasted_cut_does_not_paste_the_cut_text_again() {
    // Cmd/Ctrl+V hands the app the system clipboard's text, which still holds the cut after the
    // move. Excel pastes a cut once: the second Cmd/Ctrl+V must leave the moved formulas alone.
    let mut h = harness(Services::default());
    h.state_mut().run("cell.set", json!({"cell": "B3", "input": "=B2*2"})).unwrap();
    h.state_mut().run("selection.set", json!({"range": "A1:B3"})).unwrap();
    h.input_mut().events.push(Event::Cut);
    h.step();
    let text = h
        .output()
        .platform_output
        .commands
        .iter()
        .find_map(|c| match c {
            OutputCommand::CopyText(text) => Some(text.clone()),
            _ => None,
        })
        .expect("the cut is published");
    h.run_steps(2);
    h.state_mut().run("selection.set", json!({"cell": "D4"})).unwrap();
    h.input_mut().events.push(Event::Paste(text.clone()));
    h.run_steps(2);
    assert!(has_formula(&h, "E6"), "the cut moves the formula");

    // The host may hand the text back with CRLF line endings.
    h.input_mut().events.push(Event::Paste(text.replace('\n', "\r\n")));
    h.run_steps(2);
    assert!(has_formula(&h, "E6"), "a second Cmd/Ctrl+V leaves the moved formula alone");
    assert_eq!(value_at(&h, "E6"), Some(Value::number(84.0)));
    assert_eq!(value_at(&h, "D4"), Some(Value::from("Name")));
    assert!(h.state().toast.is_some() || h.state().message.is_some(), "Paste reports there is nothing to paste");
}

#[test]
fn the_cut_text_pastes_again_once_the_clipboard_has_held_other_text() {
    // Once other text has been pasted, the clipboard has moved on from the cut: the same text
    // copied again later (from another app, or a cell editor) is a new copy and pastes.
    let mut h = harness(Services::default());
    h.state_mut().run("selection.set", json!({"cell": "A1"})).unwrap();
    h.input_mut().events.push(Event::Cut);
    h.step();
    let text = h
        .output()
        .platform_output
        .commands
        .iter()
        .find_map(|c| match c {
            OutputCommand::CopyText(text) => Some(text.clone()),
            _ => None,
        })
        .expect("the cut is published");
    h.run_steps(2);
    h.state_mut().run("selection.set", json!({"cell": "D4"})).unwrap();
    h.input_mut().events.push(Event::Paste(text.clone()));
    h.run_steps(2);
    assert_eq!(value_at(&h, "D4"), Some(Value::from("Name")), "the cut moves A1");
    h.state_mut().run("selection.set", json!({"cell": "G1"})).unwrap();
    h.input_mut().events.push(Event::Paste("other".into()));
    h.run_steps(2);
    assert_eq!(value_at(&h, "G1"), Some(Value::from("other")));
    h.state_mut().run("selection.set", json!({"cell": "H1"})).unwrap();
    h.input_mut().events.push(Event::Paste(text.replace('\n', "\r\n")));
    h.run_steps(2);
    assert_eq!(value_at(&h, "H1"), Some(Value::from("Name")), "the same text copied afresh pastes");
}

/// The floating pictures on the active sheet: (anchor cell, width, height, mime).
fn pictures(h: &Harness) -> Vec<(String, f32, f32, String)> {
    let sh = h.state().session.active().unwrap().wb.active().unwrap();
    sh.images.iter().map(|i| (i.anchor.cell.a1(), i.anchor.width, i.anchor.height, i.mime.clone())).collect()
}
