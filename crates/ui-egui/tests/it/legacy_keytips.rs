//! Excel 2003 menu access keys that open a ribbon control rather than run a command (#941).

use egui::os::OperatingSystem;
use egui::{Event, Key, Modifiers};
use egui_kittest::kittest::Queryable as _;
use simplegrid_engine::Session;
use simplegrid_ui_egui::SheetApp;
use serde_json::json;

fn press(h: &mut egui_kittest::Harness<'static, SheetApp>, key: Key) {
    let ev = |pressed| Event::Key { key, physical_key: None, pressed, repeat: false, modifiers: Modifiers::NONE };
    h.input_mut().events.extend([ev(true), Event::Text(key.name().to_ascii_lowercase()), ev(false)]);
    h.run_steps(2);
}



#[test]
fn a_waiting_key_tip_request_stays_with_its_window() {
    // A request waiting for the frame that draws its control belongs to the window whose keys
    // made it: another window drawn first must not take it.
    let mut s = Session::new();
    s.new_workbook();
    let mut h = crate::common::os_harness(s, OperatingSystem::Nix);
    h.run_steps(2);
    let first = h.state().session.active_window().unwrap();
    h.state_mut().run("view.newWindow", json!({})).unwrap();
    let second = h.state().session.active_window().unwrap();
    h.state_mut().show_window(second);
    h.state_mut().ui.keytip_invoke_next = Some("insert.symbol");
    h.state_mut().show_window(first);
    assert_eq!(h.state().ui.keytip_invoke_next, None, "the first window doesn't see the second window's request");
    h.state_mut().show_window(second);
    assert_eq!(h.state().ui.keytip_invoke_next, Some("insert.symbol"));
}

#[test]
fn a_hidden_window_drops_its_waiting_key_tip_request() {
    // A hidden window isn't drawn, so its waiting request would fire when it was shown again,
    // long after the keys: hiding the window drops it.
    let mut s = Session::new();
    s.new_workbook();
    let mut h = crate::common::os_harness(s, OperatingSystem::Nix);
    h.run_steps(2);
    let first = h.state().session.active_window().unwrap();
    h.state_mut().run("view.newWindow", json!({})).unwrap();
    let second = h.state().session.active_window().unwrap();
    h.state_mut().show_window(second);
    h.state_mut().ui.keytip_invoke_next = Some("insert.symbol");
    h.state_mut().run("view.hideWindow", json!({"window": second})).unwrap();
    h.state_mut().show_window(first);
    h.state_mut().run("view.unhideWindow", json!({"window": second})).unwrap();
    h.state_mut().show_window(second);
    assert_eq!(h.state().ui.keytip_invoke_next, None);
}

#[test]
fn a_window_hidden_from_another_window_drops_its_parked_request() {
    // Hidden while another window is in front (its request already parked), the window still
    // loses the request, so showing it again doesn't open the control.
    let mut s = Session::new();
    s.new_workbook();
    let mut h = crate::common::os_harness(s, OperatingSystem::Nix);
    h.run_steps(2);
    let first = h.state().session.active_window().unwrap();
    h.state_mut().run("view.newWindow", json!({})).unwrap();
    let second = h.state().session.active_window().unwrap();
    h.state_mut().show_window(second);
    h.state_mut().ui.keytip_invoke_next = Some("insert.symbol");
    h.state_mut().enter_window(first);
    h.state_mut().run("view.hideWindow", json!({"window": second})).unwrap();
    h.state_mut().show_window(first);
    h.state_mut().run("view.unhideWindow", json!({"window": second})).unwrap();
    h.state_mut().show_window(second);
    assert_eq!(h.state().ui.keytip_invoke_next, None);
}
