//! The File tab opens the Backstage view, which exposes the file operations without a native menu.

use std::sync::{Arc, Mutex};

use egui_kittest::kittest::{NodeT as _, Queryable as _};
use simplegrid_engine::{Session, core::CellRef, core::Value};
use simplegrid_ui_egui::i18n::Language;
use simplegrid_ui_egui::{Services, SheetApp};

fn harness(services: Services) -> egui_kittest::Harness<'static, SheetApp> {
    crate::support::services_harness(services, 4)
}

/// Opens File (the Backstage view) and clicks a navigation entry or page button.
fn choose(h: &mut egui_kittest::Harness<'static, SheetApp>, label: &str) {
    if h.state().backstage.is_none() {
        h.get_by_label("File").click();
        h.run_steps(3);
    }
    click(h, label);
}

fn click(h: &mut egui_kittest::Harness<'static, SheetApp>, label: &str) {
    h.get_by_role_and_label(egui::accesskit::Role::Button, label).click();
    h.run_steps(3);
}

fn page(h: &egui_kittest::Harness<'static, SheetApp>) -> Option<&'static str> {
    h.state().backstage.as_ref().map(|b| b.page.id())
}

/// Esc first cancels the focused File-page field, then closes File on the next press.
#[test]
fn escape_leaves_file_fields_before_closing_the_view() {
    use egui::accesskit::Role;

    for (page_id, label) in [
        ("info", "Title"),
        ("info", "Subject"),
        ("info", "Author"),
        ("info", "Company"),
        ("info", "Keywords"),
        ("info", "Description"),
        ("options", "Maximum Iterations:"),
        ("options", "Maximum Change:"),
        ("print", "Copies"),
    ] {
        let services = Services {
            printers: Some(Box::new(|| {
                let (tx, rx) = std::sync::mpsc::channel();
                tx.send(Vec::new()).unwrap();
                rx
            })),
            ..Default::default()
        };
        let mut session = Session::new();
        session.new_workbook();
        let mut h = crate::common::harness_with(
            crate::common::builder().with_size(egui::vec2(1200.0, 1400.0)),
            crate::common::prepare(SheetApp::new(session, services)),
            4,
        );
        h.state_mut().run("formulas.calculationOptions", serde_json::json!({"iterative": true})).unwrap();
        h.state_mut().run("ui.backstage", serde_json::json!({"page": page_id})).unwrap();
        h.run_steps(3);
        let role = if page_id == "print" { Role::SpinButton } else { Role::TextInput };
        h.get_by_role_and_label(role, label).click();
        h.run_steps(2);
        assert!(h.ctx.text_edit_focused(), "{label} has the caret");
        let original = h.get_by_role_and_label(role, label).value();
        let revision = h.state().session.active().unwrap().revision;
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        h.run_steps(1);
        h.input_mut().events.push(egui::Event::Text("7".into()));
        h.run_steps(2);
        if page_id == "print" {
            assert_eq!(h.state().printing.settings.copies, 7, "Copies applies the entered value while editing");
        } else {
            assert_eq!(h.get_by_role_and_label(Role::TextInput, label).value().as_deref(), Some("7"));
        }
        let copies = h.state().printing.settings.copies;

        h.key_press(egui::Key::Escape);
        h.run_steps(3);
        assert_eq!(page(&h), Some(page_id), "Esc in {label} keeps File open");
        assert!(!h.ctx.text_edit_focused(), "Esc leaves {label}");
        assert_eq!(h.state().session.active().unwrap().revision, revision, "Esc does not commit {label}");
        if page_id == "print" {
            assert_eq!(h.state().printing.settings.copies, copies, "leaving Copies preserves its existing live value");
        } else {
            assert_eq!(h.get_by_role_and_label(Role::TextInput, label).value(), original, "Esc discards the {label} draft");
        }

        h.key_press(egui::Key::Escape);
        h.run_steps(3);
        assert_eq!(page(&h), None, "the next Esc returns to the sheet");
    }
}

#[test]
/// Options opens, named Backstage pages are reachable and accessible, an invalid page errors, and both close and Back leave Backstage.
fn file_options_opens_the_options_page_and_pages_are_reachable_by_command() {
    let mut h = harness(Services::default());
    let r = h.state_mut().run("file.options", serde_json::json!({})).unwrap();
    assert_eq!(r["calculation"]["mode"], "automatic");
    h.run_steps(3);
    h.check_accessibility();
    assert_eq!(page(&h), Some("options"));
    click(&mut h, "Info");
    h.check_accessibility();
    assert_eq!(page(&h), Some("info"));
    for id in ["print", "export", "saveAs", "new", "open", "home"] {
        h.state_mut().run("ui.backstage", serde_json::json!({"page": id})).unwrap();
        h.run_steps(2);
        h.check_accessibility();
        assert_eq!(page(&h), Some(id));
    }
    assert!(h.state_mut().run("ui.backstage", serde_json::json!({"page": "feedback"})).is_err());
    h.state_mut().run("ui.backstage", serde_json::json!({"page": "close"})).unwrap();
    assert!(h.state().backstage.is_none());
    // The back arrow returns to the sheet too.
    h.state_mut().run("ui.backstage", serde_json::json!({})).unwrap();
    h.run_steps(2);
    click(&mut h, "Back");
    h.check_accessibility();
    assert!(h.state().backstage.is_none());
}

#[test]
fn export_as_pdf_is_delivered_as_a_download() {
    let files = Arc::new(Mutex::new(Vec::<(String, Vec<u8>)>::new()));
    let capture = files.clone();
    let mut h = harness(Services {
        download: Some(Box::new(move |name, bytes| capture.lock().unwrap().push((name.to_string(), bytes.to_vec())))),
        ..Default::default()
    });
    h.state_mut().run("file.exportPdf", serde_json::json!({})).unwrap();
    let got = files.lock().unwrap();
    assert_eq!(got.len(), 1, "Export as PDF hands a file to the browser");
    assert!(got[0].0.ends_with(".pdf"));
    assert!(got[0].1.starts_with(b"%PDF-"));
}

/// The window's close button (or Alt+F4, or the OS quitting the app): steps until the request
/// has been handled and returns the viewport commands the app answered with.
fn request_window_close(h: &mut egui_kittest::Harness<'static, SheetApp>) -> Vec<egui::ViewportCommand> {
    h.input_mut().viewports.entry(egui::ViewportId::ROOT).or_default().events.push(egui::ViewportEvent::Close);
    step_commands(h)
}

/// Runs a few frames and gathers the commands sent to the window.
fn step_commands(h: &mut egui_kittest::Harness<'static, SheetApp>) -> Vec<egui::ViewportCommand> {
    let mut out = Vec::new();
    for _ in 0..3 {
        h.step();
        out.extend(h.output().viewport_output.get(&egui::ViewportId::ROOT).map(|v| v.commands.clone()).unwrap_or_default());
    }
    out
}

fn click_commands(h: &mut egui_kittest::Harness<'static, SheetApp>, label: &str) -> Vec<egui::ViewportCommand> {
    h.get_by_role_and_label(egui::accesskit::Role::Button, label).click();
    step_commands(h)
}

/// Book1 has a cell being typed when File › New runs (the macOS menu runs the command directly).
fn typing_in_book1_then_new(h: &mut egui_kittest::Harness<'static, SheetApp>) -> u64 {
    h.input_mut().events.push(egui::Event::Text("hello".into()));
    h.run_steps(2);
    assert!(h.state().editor.is_some());
    let book1 = h.state().session.doc().unwrap().uid;
    h.state_mut().run_or_alert("file.new", serde_json::json!({}));
    h.run_steps(3);
    book1
}

fn a1(h: &egui_kittest::Harness<'static, SheetApp>, doc: usize) -> Value {
    h.state().session.documents()[doc].wb.active().unwrap().value(CellRef::new(0, 0))
}

#[test]
fn a_draft_its_workbook_refuses_stops_the_switch_and_stays_open() {
    let mut h = harness(Services::default());
    let book1 = typing_in_book1_then_new(&mut h);
    let book2 = h.state().session.doc().unwrap().uid;
    h.state_mut().session.execute("data.validation", serde_json::json!({"range": "A1", "type": "list", "formula1": "\"Red,Green\""})).unwrap();
    h.input_mut().events.push(egui::Event::Text("Purple".into()));
    h.run_steps(2);
    h.state_mut().run_or_alert("view.switchWindows", serde_json::json!({"doc": book1}));
    h.run_steps(2);
    assert_eq!(h.state().session.doc().unwrap().uid, book2, "the switch doesn't happen");
    assert_eq!(h.state().editor.as_ref().map(|ed| ed.text.as_str()), Some("Purple"), "the draft stays open to fix");
    assert_eq!(a1(&h, 1), Value::Empty, "Book2 is unchanged");
    assert!(h.state().message.is_some(), "the refusal is reported");

    // Esc drops the draft; then the switch goes through.
    h.state_mut().message = None;
    h.state_mut().cancel_edit();
    h.state_mut().run_or_alert("view.switchWindows", serde_json::json!({"doc": book1}));
    h.run_steps(2);
    assert_eq!(h.state().session.doc().unwrap().uid, book1);
    assert!(h.state().message.is_none());
}

fn pdf_picker_harness(dir: &std::path::Path) -> (Arc<Mutex<Vec<String>>>, egui_kittest::Harness<'static, SheetApp>) {
    let asked = Arc::new(Mutex::new(Vec::<String>::new()));
    let (capture, target) = (asked.clone(), dir.to_path_buf());
    let h = harness(Services {
        pick_save: Some(Box::new(move |suggested| {
            capture.lock().unwrap().push(suggested.to_string());
            Some(target.join("Report.xlsx").to_string_lossy().into_owned())
        })),
        ..Default::default()
    });
    (asked, h)
}

#[test]
fn the_palette_export_as_pdf_asks_where_to_save() {
    let dir = std::env::temp_dir().join(format!("simplegrid-export-pdf-palette-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (asked, mut h) = pdf_picker_harness(&dir);
    h.state_mut().run("cell.set", serde_json::json!({"cell": "A1", "input": "42"})).unwrap();
    // The palette and menus go through run_or_alert.
    h.state_mut().run_or_alert("file.exportPdf", serde_json::json!({}));
    assert!(asked.lock().unwrap().first().is_some_and(|s| s.ends_with(".pdf")), "the picker is offered a .pdf name");
    assert!(std::fs::read(dir.join("Report.pdf")).unwrap().starts_with(b"%PDF-"), "the PDF is written, whatever extension the picker returned");
    assert!(h.state().message.is_none());
    let d = h.state().session.active().unwrap();
    assert!(d.path.is_none(), "a PDF is a copy, not the workbook's file");
    // Other params survive the round trip through the picker.
    h.state_mut().run_or_alert("file.exportPdf", serde_json::json!({"range": "nope"}));
    assert!(h.state().message.is_some(), "a bad range is reported, not swallowed");
    // A path in the params skips the picker.
    let before = asked.lock().unwrap().len();
    let named = dir.join("Named.pdf");
    h.state_mut().run_or_alert("file.exportPdf", serde_json::json!({"path": named.to_string_lossy()}));
    assert_eq!(asked.lock().unwrap().len(), before);
    assert!(named.exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_cancelled_export_pdf_picker_writes_nothing() {
    let mut h = harness(Services { pick_save: Some(Box::new(|_| None)), ..Default::default() });
    h.state_mut().run_or_alert("file.exportPdf", serde_json::json!({}));
    assert!(h.state().message.is_none());
    assert!(h.state_mut().session.take_ui_requests().is_empty());
}

/// Options › Formulas: Maximum Iterations and Maximum Change, with the checkbox that enables them.
#[test]
fn options_sets_maximum_iterations_and_change_only_while_iteration_is_on() {
    use egui::accesskit::Role;
    let mut session = Session::new();
    session.new_workbook();
    // Tall enough that the Formulas section is on screen without scrolling.
    let mut h = crate::common::builder().with_size(egui::vec2(1200.0, 1400.0)).build_ui_state(
        |ui, app: &mut SheetApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        crate::common::prepare(SheetApp::new(session, Services::default())),
    );
    h.run_steps(4);
    h.state_mut().run("file.options", serde_json::json!({})).unwrap();
    h.run_steps(3);
    let calc = |h: &egui_kittest::Harness<'static, SheetApp>| h.state().session.active().unwrap().wb.calc.clone();
    // Resolve the field by its accessible name in both the disabled and enabled states.
    let field = |h: &egui_kittest::Harness<'static, SheetApp>, label: &str| h.get_by_role_and_label(Role::TextInput, label).rect();
    let disabled = |h: &egui_kittest::Harness<'static, SheetApp>, rect: egui::Rect| {
        h.get_all_by_role(Role::TextInput).find(|n| n.rect() == rect).unwrap().accesskit_node().is_disabled()
    };
    let retype = |h: &mut egui_kittest::Harness<'static, SheetApp>, rect: egui::Rect, text: &str| {
        h.input_mut().events.push(egui::Event::PointerMoved(rect.center()));
        h.run_steps(1);
        h.get_all_by_role(Role::TextInput).find(|n| n.rect() == rect).unwrap().click();
        h.run_steps(2);
        h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        h.run_steps(1);
        for c in text.chars() {
            h.input_mut().events.push(egui::Event::Text(c.to_string()));
            h.run_steps(2);
        }
        h.key_press(egui::Key::Enter);
        h.run_steps(2);
    };

    // Iteration is off: the two fields show the defaults, greyed out.
    let (iterations, change) = (field(&h, "Maximum Iterations:"), field(&h, "Maximum Change:"));
    assert!(disabled(&h, iterations) && disabled(&h, change), "the fields are disabled while iteration is off");
    assert_eq!(h.get_all_by_role(Role::TextInput).find(|n| n.rect() == iterations).unwrap().value().as_deref(), Some("100"));
    assert_eq!(h.get_all_by_role(Role::TextInput).find(|n| n.rect() == change).unwrap().value().as_deref(), Some("0.001"));

    h.get_by_role_and_label(Role::CheckBox, "Enable iterative calculation").click();
    h.run_steps(3);
    assert!(calc(&h).iterative);
    let (iterations, change) = (field(&h, "Maximum Iterations:"), field(&h, "Maximum Change:"));
    assert!(!disabled(&h, iterations) && !disabled(&h, change));

    retype(&mut h, iterations, "500");
    assert_eq!(calc(&h).max_iterations, 500);
    retype(&mut h, change, "0.01");
    assert_eq!(calc(&h).max_change, 0.01);
    assert!(h.state().message.is_none());

    // Out-of-range entries are refused with a message and change nothing (the command alone would clamp them).
    for bad in ["0", "32768", "12.5", "lots"] {
        retype(&mut h, iterations, bad);
        assert_eq!(calc(&h).max_iterations, 500, "{bad}");
        assert!(h.state_mut().message.take().is_some(), "{bad} is reported");
        h.run_steps(2);
    }
    retype(&mut h, iterations, "32767");
    assert_eq!(calc(&h).max_iterations, 32767);
    for bad in ["-1", "x"] {
        retype(&mut h, change, bad);
        assert_eq!(calc(&h).max_change, 0.01, "{bad}");
        assert!(h.state_mut().message.take().is_some(), "{bad} is reported");
        h.run_steps(2);
    }
    retype(&mut h, change, "0");
    assert_eq!(calc(&h).max_change, 0.0);

    // Undo puts the old value back in the field.
    h.state_mut().run("edit.undo", serde_json::json!({})).unwrap();
    h.run_steps(3);
    assert_eq!(calc(&h).max_change, 0.01);
    let change = field(&h, "Maximum Change:");
    assert_eq!(h.get_all_by_role(Role::TextInput).find(|n| n.rect() == change).unwrap().value().as_deref(), Some("0.01"));

    // A value typed but not confirmed with Enter is dropped when the checkbox switches iteration off.
    let iterations = field(&h, "Maximum Iterations:");
    h.input_mut().events.push(egui::Event::PointerMoved(iterations.center()));
    h.run_steps(1);
    h.get_all_by_role(Role::TextInput).find(|n| n.rect() == iterations).unwrap().click();
    h.run_steps(2);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.input_mut().events.push(egui::Event::Text("777".into()));
    h.run_steps(2);
    h.get_by_role_and_label(Role::CheckBox, "Enable iterative calculation").click();
    h.run_steps(3);
    assert!(!calc(&h).iterative);
    assert_eq!(calc(&h).max_iterations, 32767, "an unconfirmed entry is not committed once iteration is off");
    assert!(h.state().message.is_none());

    // Spanish accepts a decimal comma in Maximum Change, and still a dot.
    h.get_by_role_and_label(Role::CheckBox, "Enable iterative calculation").click();
    h.run_steps(3);
    h.state_mut().run("app.language.set", serde_json::json!({"language": Language::Es.code()})).unwrap();
    h.state_mut().run("file.options", serde_json::json!({})).unwrap();
    h.run_steps(3);
    let change = field(&h, "Cambio máximo:");
    retype(&mut h, change, "0,02");
    assert_eq!(calc(&h).max_change, 0.02);
    retype(&mut h, change, "0.5");
    assert_eq!(calc(&h).max_change, 0.5);
    assert!(h.state().message.is_none());
}

/// Share › Export as Web Page (and the native File menu's Save as Web Page) asks for a name in its
/// own dialog. A bare name has no folder: written as typed it lands in the process's working
/// directory, where nobody looks for it (#1212). The save picker chooses the folder instead.
#[test]
fn save_a_copy_with_a_bare_name_asks_where_to_save_it() {
    let dir = std::env::temp_dir().join(format!("simplegrid-save-copy-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let asked = Arc::new(Mutex::new(Vec::<String>::new()));
    let (capture, target) = (asked.clone(), dir.clone());
    let mut h = harness(Services {
        pick_save: Some(Box::new(move |suggested| {
            capture.lock().unwrap().push(suggested.to_string());
            Some(target.join("Chosen.html").to_string_lossy().into_owned())
        })),
        ..Default::default()
    });
    h.state_mut().run("cell.set", serde_json::json!({"cell": "A1", "input": "42"})).unwrap();
    let save_copy = |h: &mut egui_kittest::Harness<'static, SheetApp>, path: &str| {
        h.state_mut().open_dialog("saveCopy", serde_json::json!({"format": "html"}));
        h.run_steps(3);
        h.state_mut().dialog.as_mut().unwrap().values.insert("path".into(), serde_json::json!(path));
        h.run_steps(2);
        h.get_by_label("   OK   ").click();
        h.run_steps(3);
    };
    let name = format!("simplegrid-save-copy-{}", std::process::id());
    save_copy(&mut h, &name);
    let stray = std::path::PathBuf::from(format!("{name}.html"));
    let written_in_cwd = stray.exists();
    let _ = std::fs::remove_file(&stray);
    assert!(!written_in_cwd, "a bare name is not written to the working directory");
    assert_eq!(*asked.lock().unwrap(), vec![format!("{name}.html")], "the picker is offered the typed name");
    assert!(std::fs::read_to_string(dir.join("Chosen.html")).unwrap().contains("42"), "the page is written where the picker says");
    assert!(h.state().message.is_none());
    // A full path skips the picker; a write that fails is reported, not swallowed.
    save_copy(&mut h, &dir.join("Named.html").to_string_lossy());
    assert!(dir.join("Named.html").exists());
    save_copy(&mut h, &dir.join("missing").join("Lost.html").to_string_lossy());
    assert!(h.state().message.is_some(), "a failed write is reported");
    assert_eq!(asked.lock().unwrap().len(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn control_requests_route_to_a_window_and_list_windows() {
    use simplegrid_ui_egui::control::{Outcome, handle};
    let mut h = harness(Services::default());
    let first = h.state().session.active_window().unwrap();
    h.state_mut().run_or_alert("file.new", serde_json::json!({}));
    h.run_steps(2);
    let ctx = h.ctx.clone();
    let call = |h: &mut egui_kittest::Harness<'static, SheetApp>, method: &str, p: serde_json::Value| match handle(h.state_mut(), &ctx, method, &p) {
        Outcome::Done(v) => v,
        Outcome::Screenshot { .. } => serde_json::Value::Null,
    };
    let r = call(&mut h, "engine.execute", serde_json::json!({"command": "cell.set", "params": {"cell": "A1", "input": "one"}, "window": first}));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(a1(&h, 0), Value::text("one"));
    assert_eq!(a1(&h, 1), Value::Empty);
    assert_eq!(h.state().session.active_index(), 1, "Book2 stays in front");
    let r = call(&mut h, "cell.get", serde_json::json!({"cell": "A1", "window": first}));
    assert!(r["result"].to_string().contains("one"), "{r}");

    let r = call(&mut h, "window.list", serde_json::json!({}));
    let rows = r["result"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "{r}");
    assert_eq!((rows[0]["window"].as_u64(), rows[0]["title"].as_str(), rows[0]["dirty"].as_bool()), (Some(first), Some("Book1"), Some(true)));
    assert_eq!(rows[1]["active"].as_bool(), Some(true));
    let r = call(&mut h, "engine.execute", serde_json::json!({"command": "cell.get", "params": {"cell": "A1"}, "window": 999}));
    assert_eq!(r["ok"], false, "{r}");
}
