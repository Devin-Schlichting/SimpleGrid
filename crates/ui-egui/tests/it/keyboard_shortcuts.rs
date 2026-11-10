//! Excel keyboard shortcuts (crate::keymap) dispatched through real egui key events.

use egui::{Event, Key, Modifiers};
use simplegrid_engine::Session;
use simplegrid_engine::core::CellRef;
use simplegrid_engine::model::BorderStyle;
use simplegrid_ui_egui::SheetApp;
use simplegrid_ui_egui::keymap::{Shortcut, lookup};
use serde_json::json;

type Harness = egui_kittest::Harness<'static, SheetApp>;

fn harness(session: Session) -> Harness {
    crate::support::harness(session, 4)
}

fn book(selected: &str) -> Session {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("range.setValues", json!({"range": "A1", "values": [[1.5, 2], [3, 4], [5, 6]]})).unwrap();
    s.execute("cell.set", json!({"cell": "C1", "input": "=A1*2"})).unwrap();
    s.execute("selection.set", json!({"range": selected})).unwrap();
    s
}

/// A key press as a US keyboard reports it (the physical key is the unshifted one).
fn key(h: &mut Harness, k: Key, m: Modifiers) {
    h.input_mut().events.push(Event::Key { key: k, physical_key: Some(k), pressed: true, repeat: false, modifiers: m });
    h.input_mut().events.push(Event::Key { key: k, physical_key: Some(k), pressed: false, repeat: false, modifiers: m });
    h.run_steps(2);
}

fn style(h: &Harness, a1: &str) -> simplegrid_engine::model::Style {
    let d = h.state().session.active().unwrap();
    let sh = d.wb.active().unwrap();
    d.wb.styles.get(sh.style_id(CellRef::parse(a1).unwrap())).clone()
}

/// Ctrl on Windows and Linux sets both `ctrl` and `command`.
const CTRL: Modifiers = Modifiers { alt: false, ctrl: true, shift: false, mac_cmd: false, command: true };
const CTRL_SHIFT: Modifiers = Modifiers { shift: true, ..CTRL };



#[test]
fn windows_and_mac_variants() {
    let cmd = CTRL;
    let mac_cmd = Modifiers { mac_cmd: true, command: true, ..Modifiers::NONE };
    let ctrl = Modifiers::CTRL;
    let run = |id: &'static str| Some(Shortcut::Run(id, json!({})));
    assert_eq!(lookup(Key::L, Some(Key::L), cmd, false), run("insert.table"));
    assert_eq!(lookup(Key::L, Some(Key::L), mac_cmd, true), run("home.alignLeft"));
    assert_eq!(lookup(Key::E, Some(Key::E), mac_cmd, true), run("home.alignCenter"));
    assert_eq!(lookup(Key::E, Some(Key::E), ctrl, true), run("edit.flashFill"));
    assert_eq!(lookup(Key::Equals, Some(Key::Equals), Modifiers::ALT, false), run("formulas.autoSum"));
    // The shifted character doesn't matter: Ctrl+Shift+1 is the Number format wherever `!` is.
    assert_eq!(
        lookup(Key::Exclamationmark, Some(Key::Num1), CTRL_SHIFT, false),
        Some(Shortcut::Run("home.numberFormat", json!({"code": "#,##0.00"})))
    );
    // Navigation keys go by what they are, not where: the numpad's arrows and Page Down (Num
    // Lock off) report Num6 / Num3 as their physical keys.
    let alt_shift = Modifiers { alt: true, shift: true, ..Modifiers::NONE };
    assert_eq!(lookup(Key::ArrowRight, Some(Key::Num6), alt_shift, false), run("data.group"));
    assert_eq!(lookup(Key::ArrowLeft, Some(Key::Num4), alt_shift, false), run("data.ungroup"));
    assert_eq!(lookup(Key::PageDown, Some(Key::Num3), Modifiers::ALT, false), Some(Shortcut::ScreenRight));
    assert_eq!(lookup(Key::PageDown, Some(Key::Num3), CTRL_SHIFT, false), None);
    // Ctrl+. is Next Corner of Selection, with Control on a Mac too (#1442); Cmd+Shift+. stays
    // Increase Font Size.
    assert_eq!(lookup(Key::Period, Some(Key::Period), cmd, false), run("selection.nextCorner"));
    assert_eq!(lookup(Key::Period, Some(Key::Period), ctrl, true), run("selection.nextCorner"));
    assert_eq!(lookup(Key::Period, Some(Key::Period), Modifiers { shift: true, ..mac_cmd }, true), run("home.increaseFontSize"));
    // Plain keys stay with the grid (typing, navigation).
    assert_eq!(lookup(Key::A, Some(Key::A), Modifiers::NONE, false), None);
    assert_eq!(lookup(Key::F2, Some(Key::F2), Modifiers::NONE, false), None);
}


#[test]
fn space_shortcuts_per_platform() {
    let run = |id: &'static str| Some(Shortcut::Run(id, json!({})));
    let mac_cmd = Modifiers { mac_cmd: true, command: true, ..Modifiers::NONE };
    let space = |m, mac| lookup(Key::Space, Some(Key::Space), m, mac);
    assert_eq!(space(Modifiers::SHIFT, false), run("selection.row"));
    assert_eq!(space(Modifiers::SHIFT, true), run("selection.row"));
    assert_eq!(space(CTRL, false), run("selection.column"));
    assert_eq!(space(Modifiers::CTRL, true), run("selection.column"));
    // Cmd+Space is Spotlight on macOS.
    assert_eq!(space(mac_cmd, true), None);
}




/// #1443: Ctrl+Q (also with Shift or Alt, which is what AltGr+Q reports on Windows) must not
/// close the window and lose unsaved input; macOS keeps Cmd+Q as the app's Quit.
#[test]
fn ctrl_q_does_not_close_the_window_outside_macos() {
    let closes = |h: &mut Harness, k: Key, m: Modifiers| -> bool {
        h.input_mut().events.push(Event::Key { key: k, physical_key: Some(k), pressed: true, repeat: false, modifiers: m });
        h.input_mut().events.push(Event::Key { key: k, physical_key: Some(k), pressed: false, repeat: false, modifiers: m });
        let mut closed = false;
        for _ in 0..3 {
            h.step();
            closed |= h.output().viewport_output.values().any(|v| v.commands.iter().any(|c| matches!(c, egui::ViewportCommand::Close)));
        }
        closed
    };
    let ctrl_alt = Modifiers { alt: true, ..CTRL };
    for os in [egui::os::OperatingSystem::Windows, egui::os::OperatingSystem::Nix] {
        let mut h = harness(book("A1"));
        h.ctx.set_os(os);
        h.run_steps(2);
        for (name, m) in [("Ctrl+Q", CTRL), ("Ctrl+Shift+Q", CTRL_SHIFT), ("Ctrl+Alt+Q", ctrl_alt)] {
            assert!(!closes(&mut h, Key::Q, m), "{os:?}: {name} closed the window");
        }
        assert!(h.ctx.options(|o| o.quit_shortcuts.is_empty()), "{os:?}: no quit shortcut left");
    }
    // macOS: Cmd+Q stays the app's Quit.
    let mut h = harness(book("A1"));
    // The harness's first frame already ran as the host OS; restore egui's default, as a real
    // macOS launch would have it.
    h.ctx.options_mut(|o| o.quit_shortcuts = vec![egui::KeyboardShortcut::new(Modifiers::COMMAND, Key::Q)]);
    h.ctx.set_os(egui::os::OperatingSystem::Mac);
    h.run_steps(2);
    assert!(h.ctx.options(|o| !o.quit_shortcuts.is_empty()), "Cmd+Q is still egui's quit shortcut on macOS");
}

#[test]
fn ctrl_6_is_show_objects_and_nothing_else() {
    let show = Some(Shortcut::Run("view.showObjects", json!({})));
    let mac_control = Modifiers::CTRL;
    assert_eq!(lookup(Key::Num6, Some(Key::Num6), CTRL, false), show);
    assert_eq!(lookup(Key::Num6, Some(Key::Num6), mac_control, true), show);
    // Its neighbours keep theirs: Ctrl+Shift+6 (Ctrl+^) is Scientific, Cmd+6 on a Mac is free.
    assert_eq!(lookup(Key::Num6, Some(Key::Num6), CTRL_SHIFT, false), Some(Shortcut::Run("home.numberFormat", json!({"code": "0.00E+00"}))));
    assert_eq!(lookup(Key::Num6, Some(Key::Num6), Modifiers::MAC_CMD, true), None);
    // No other command claims Ctrl+6.
    let claims: Vec<&str> = Session::new().commands().iter().filter(|c| c.shortcut == Some("Ctrl+6")).map(|c| c.id).collect();
    assert_eq!(claims, ["view.showObjects"]);
}

/// Alt+= as Windows and Linux report it: the key press, then the `=` the OS types with it.
fn alt_equals(h: &mut Harness) {
    h.input_mut().events.push(Event::Key {
        key: Key::Equals,
        physical_key: Some(Key::Equals),
        pressed: true,
        repeat: false,
        modifiers: Modifiers::ALT,
    });
    h.input_mut().events.push(Event::Text("=".into()));
    h.run_steps(2);
}


#[test]
fn ctrl_alt_letter_that_typed_text_is_not_select_all() {
    let mut h = harness(book("B2"));
    let ctrl_alt = Modifiers { alt: true, ..CTRL };
    h.input_mut().events.push(Event::Key { key: Key::A, physical_key: Some(Key::A), pressed: true, repeat: false, modifiers: ctrl_alt });
    h.input_mut().events.push(Event::Text("\u{e4}".into()));
    h.run_steps(2);
    assert_eq!(h.state().session.active().unwrap().selection.current().a1(), "B2");
    assert_eq!(h.state().editor.as_ref().unwrap().text, "\u{e4}");
}
