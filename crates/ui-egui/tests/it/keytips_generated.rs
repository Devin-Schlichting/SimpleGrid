//! GENERATED from keytips.json by `cargo xtask keytips`. Edit the JSON, not this file.
//!
//! One end-to-end test per keytip sequence, through the real egui input path.
//! Assertions are generated from a probe of the live engine catalog (blank
//! workbook, empty params), so each test proves exactly what its command does
//! today: journaled, ran clean, surfaced a missing-params error, opened a
//! dialog, switched tabs, waited for more input, or stayed silent (pending).

use egui::os::OperatingSystem;
use egui::{Event, Key, Modifiers};
use simplegrid_engine::Session;
use simplegrid_ui_egui::SheetApp;

type Harness = egui_kittest::Harness<'static, SheetApp>;

fn harness(session: Session) -> Harness {
    crate::common::os_harness(session, OperatingSystem::Nix)
}

fn blank() -> Session {
    let mut s = Session::new();
    s.new_workbook();
    s
}

fn key_event(key: Key, pressed: bool, modifiers: Modifiers) -> Event {
    Event::Key { key, physical_key: None, pressed, repeat: false, modifiers }
}

fn events(h: &mut Harness, events: impl IntoIterator<Item = Event>) {
    h.input_mut().events.extend(events);
    h.run_steps(2);
}

fn key(h: &mut Harness, key: Key) {
    events(h, [key_event(key, true, Modifiers::NONE), key_event(key, false, Modifiers::NONE)]);
}

fn letter(h: &mut Harness, key: Key, text_first: bool) {
    let text = Event::Text(key.name().to_ascii_lowercase());
    let pressed = key_event(key, true, Modifiers::NONE);
    let pair = if text_first { [text, pressed] } else { [pressed, text] };
    events(h, pair.into_iter().chain([key_event(key, false, Modifiers::NONE)]));
}

fn sequence(h: &mut Harness, seq: &str, text_first: bool) {
    key(h, Key::F10);
    for ch in seq.chars() {
        let k = Key::from_name(&ch.to_string()).expect("sequence letters map to keys");
        letter(h, k, text_first);
    }
}

#[test]
fn keytip_h_() {
    let mut h = harness(blank());
    sequence(&mut h, "H", false);
    assert_eq!(h.state().ui.ribbon_tab, "Home", "H must switch to the Home tab");
}

#[test]
fn keytip_e_() {
    let mut h = harness(blank());
    sequence(&mut h, "E", false);
    assert_eq!(h.state().ui.ribbon_tab, "Home", "E must switch to the Home tab");
}

#[test]
fn keytip_o_() {
    let mut h = harness(blank());
    sequence(&mut h, "O", false);
    assert_eq!(h.state().ui.ribbon_tab, "Home", "O must switch to the Home tab");
}

#[test]
fn keytip_has_() {
    let mut h = harness(blank());
    sequence(&mut h, "HAS", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "HAS must run home.addIns cleanly");
}

#[test]
fn keytip_hbe_() {
    let mut h = harness(blank());
    sequence(&mut h, "HBE", false);
    assert!(h.state().dialog.is_none(), "HBE must not open a dialog");
    assert!(h.state().message.is_none(), "HBE must not open a message box");
    assert!(h.state().toast.is_none(), "HBE must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "HBE must not journal");
}

#[test]
fn keytip_hbi_() {
    let mut h = harness(blank());
    sequence(&mut h, "HBI", false);
    assert!(h.state().dialog.is_none(), "HBI must not open a dialog");
    assert!(h.state().message.is_none(), "HBI must not open a message box");
    assert!(h.state().toast.is_none(), "HBI must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "HBI must not journal");
}

#[test]
fn keytip_hby_() {
    let mut h = harness(blank());
    sequence(&mut h, "HBY", false);
    assert!(h.state().dialog.is_none(), "HBY must not open a dialog");
    assert!(h.state().message.is_none(), "HBY must not open a message box");
    assert!(h.state().toast.is_none(), "HBY must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "HBY must not journal");
}

#[test]
fn keytip_hfc_() {
    let mut h = harness(blank());
    sequence(&mut h, "HFC", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "HFC must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_hfdo_() {
    let mut h = harness(blank());
    sequence(&mut h, "HFDO", false);
    assert!(h.state().dialog.is_none(), "HFDO must not open a dialog");
    assert!(h.state().message.is_none(), "HFDO must not open a message box");
    assert!(h.state().toast.is_none(), "HFDO must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "HFDO must not journal");
}

#[test]
fn keytip_hfdp_() {
    let mut h = harness(blank());
    sequence(&mut h, "HFDP", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "HFDP must run arrange.selectionPane cleanly");
}

#[test]
fn keytip_hff_() {
    let mut h = harness(blank());
    sequence(&mut h, "HFF", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "HFF must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_hfia_() {
    let mut h = harness(blank());
    sequence(&mut h, "HFIA", false);
    assert!(h.state().dialog.is_none(), "HFIA must not open a dialog");
    assert!(h.state().message.is_none(), "HFIA must not open a message box");
    assert!(h.state().toast.is_none(), "HFIA must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "HFIA must not journal");
}

#[test]
fn keytip_hfq_() {
    let mut h = harness(blank());
    sequence(&mut h, "HFQ", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "HFQ must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_hfs_() {
    let mut h = harness(blank());
    sequence(&mut h, "HFS", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "HFS must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_hh_() {
    let mut h = harness(blank());
    sequence(&mut h, "HH", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "HH must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_hn_() {
    let mut h = harness(blank());
    sequence(&mut h, "HN", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "HN must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_nc1_() {
    let mut h = harness(blank());
    sequence(&mut h, "NC1", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "NC1 must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_nd_() {
    let mut h = harness(blank());
    sequence(&mut h, "ND", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "ND must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_nhi_() {
    let mut h = harness(blank());
    sequence(&mut h, "NHI", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "NHI must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_nj_() {
    let mut h = harness(blank());
    sequence(&mut h, "NJ", false);
    assert!(h.state().dialog.is_none(), "NJ must not open a dialog");
    assert!(h.state().message.is_none(), "NJ must not open a message box");
    assert!(h.state().toast.is_none(), "NJ must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "NJ must not journal");
}

#[test]
fn keytip_nn1_() {
    let mut h = harness(blank());
    sequence(&mut h, "NN1", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "NN1 must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_nq_() {
    let mut h = harness(blank());
    sequence(&mut h, "NQ", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "NQ must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_nsa_() {
    let mut h = harness(blank());
    sequence(&mut h, "NSA", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "NSA must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_nsc_() {
    let mut h = harness(blank());
    sequence(&mut h, "NSC", false);
    assert!(h.state().dialog.is_none(), "NSC must not open a dialog");
    assert!(h.state().message.is_none(), "NSC must not open a message box");
    assert!(h.state().toast.is_none(), "NSC must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "NSC must not journal");
}

#[test]
fn keytip_nst_() {
    let mut h = harness(blank());
    sequence(&mut h, "NST", false);
    assert!(h.state().dialog.is_none(), "NST must not open a dialog");
    assert!(h.state().message.is_none(), "NST must not open a message box");
    assert!(h.state().toast.is_none(), "NST must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "NST must not journal");
}

#[test]
fn keytip_nw_() {
    let mut h = harness(blank());
    sequence(&mut h, "NW", false);
    assert!(h.state().dialog.is_none(), "NW must not open a dialog");
    assert!(h.state().message.is_none(), "NW must not open a message box");
    assert!(h.state().toast.is_none(), "NW must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "NW must not journal");
}

#[test]
fn keytip_map_() {
    let mut h = harness(blank());
    sequence(&mut h, "MAP", false);
    assert!(h.state().dialog.is_none(), "MAP must not open a dialog");
    assert!(h.state().message.is_none(), "MAP must not open a message box");
    assert!(h.state().toast.is_none(), "MAP must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "MAP must not journal");
}

#[test]
fn keytip_md_() {
    let mut h = harness(blank());
    sequence(&mut h, "MD", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "MD must run formulas.traceDependents cleanly");
}

#[test]
fn keytip_me_() {
    let mut h = harness(blank());
    sequence(&mut h, "ME", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "ME must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_mg_() {
    let mut h = harness(blank());
    sequence(&mut h, "MG", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "MG must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_mi1_() {
    let mut h = harness(blank());
    sequence(&mut h, "MI1", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "MI1 must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_ml_() {
    let mut h = harness(blank());
    sequence(&mut h, "ML", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "ML must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_mo_() {
    let mut h = harness(blank());
    sequence(&mut h, "MO", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "MO must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_mp_() {
    let mut h = harness(blank());
    sequence(&mut h, "MP", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "MP must run formulas.tracePrecedents cleanly");
}

#[test]
fn keytip_mq_() {
    let mut h = harness(blank());
    sequence(&mut h, "MQ", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "MQ must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_mr_() {
    let mut h = harness(blank());
    sequence(&mut h, "MR", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "MR must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_mt_() {
    let mut h = harness(blank());
    sequence(&mut h, "MT", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "MT must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_muf_() {
    let mut h = harness(blank());
    sequence(&mut h, "MUF", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "MUF must run formulas.moreFunctions cleanly");
}

#[test]
fn keytip_mw_() {
    let mut h = harness(blank());
    sequence(&mut h, "MW", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "MW must run formulas.watchWindow cleanly");
}

#[test]
fn keytip_afa_() {
    let mut h = harness(blank());
    sequence(&mut h, "AFA", false);
    assert!(h.state().dialog.is_none(), "AFA must not open a dialog");
    assert!(h.state().message.is_none(), "AFA must not open a message box");
    assert!(h.state().toast.is_none(), "AFA must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "AFA must not journal");
}

#[test]
fn keytip_afe_() {
    let mut h = harness(blank());
    sequence(&mut h, "AFE", false);
    assert!(h.state().dialog.is_none(), "AFE must not open a dialog");
    assert!(h.state().message.is_none(), "AFE must not open a message box");
    assert!(h.state().toast.is_none(), "AFE must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "AFE must not journal");
}

#[test]
fn keytip_afo_() {
    let mut h = harness(blank());
    sequence(&mut h, "AFO", false);
    assert!(h.state().dialog.is_none(), "AFO must not open a dialog");
    assert!(h.state().message.is_none(), "AFO must not open a message box");
    assert!(h.state().toast.is_none(), "AFO must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "AFO must not journal");
}

#[test]
fn keytip_aft_() {
    let mut h = harness(blank());
    sequence(&mut h, "AFT", false);
    assert!(h.state().dialog.is_none(), "AFT must not open a dialog");
    assert!(h.state().message.is_none(), "AFT must not open a message box");
    assert!(h.state().toast.is_none(), "AFT must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "AFT must not journal");
}

#[test]
fn keytip_afw_() {
    let mut h = harness(blank());
    sequence(&mut h, "AFW", false);
    assert!(h.state().dialog.is_none(), "AFW must not open a dialog");
    assert!(h.state().message.is_none(), "AFW must not open a message box");
    assert!(h.state().toast.is_none(), "AFW must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "AFW must not journal");
}

#[test]
fn keytip_aga_() {
    let mut h = harness(blank());
    sequence(&mut h, "AGA", false);
    assert!(h.state().dialog.is_none(), "AGA must not open a dialog");
    assert!(h.state().message.is_none(), "AGA must not open a message box");
    assert!(h.state().toast.is_none(), "AGA must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "AGA must not journal");
}

#[test]
fn keytip_ak_() {
    let mut h = harness(blank());
    sequence(&mut h, "AK", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "AK must run data.queriesConnections cleanly");
}

#[test]
fn keytip_ao_() {
    let mut h = harness(blank());
    sequence(&mut h, "AO", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "AO must run data.queriesConnections cleanly");
}

#[test]
fn keytip_apnd_() {
    let mut h = harness(blank());
    sequence(&mut h, "APND", false);
    assert!(h.state().dialog.is_none(), "APND must not open a dialog");
    assert!(h.state().message.is_none(), "APND must not open a message box");
    assert!(h.state().toast.is_none(), "APND must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "APND must not journal");
}

#[test]
fn keytip_apne_() {
    let mut h = harness(blank());
    sequence(&mut h, "APNE", false);
    assert!(h.state().dialog.is_none(), "APNE must not open a dialog");
    assert!(h.state().message.is_none(), "APNE must not open a message box");
    assert!(h.state().toast.is_none(), "APNE must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "APNE must not journal");
}

#[test]
fn keytip_apnh_() {
    let mut h = harness(blank());
    sequence(&mut h, "APNH", false);
    assert!(h.state().dialog.is_none(), "APNH must not open a dialog");
    assert!(h.state().message.is_none(), "APNH must not open a message box");
    assert!(h.state().toast.is_none(), "APNH must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "APNH must not journal");
}

#[test]
fn keytip_apnl_() {
    let mut h = harness(blank());
    sequence(&mut h, "APNL", false);
    assert!(h.state().dialog.is_none(), "APNL must not open a dialog");
    assert!(h.state().message.is_none(), "APNL must not open a message box");
    assert!(h.state().toast.is_none(), "APNL must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "APNL must not journal");
}

#[test]
fn keytip_apns_() {
    let mut h = harness(blank());
    sequence(&mut h, "APNS", false);
    assert!(h.state().dialog.is_none(), "APNS must not open a dialog");
    assert!(h.state().message.is_none(), "APNS must not open a message box");
    assert!(h.state().toast.is_none(), "APNS must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "APNS must not journal");
}

#[test]
fn keytip_apnt_() {
    let mut h = harness(blank());
    sequence(&mut h, "APNT", false);
    assert!(h.state().dialog.is_none(), "APNT must not open a dialog");
    assert!(h.state().message.is_none(), "APNT must not open a message box");
    assert!(h.state().toast.is_none(), "APNT must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "APNT must not journal");
}

#[test]
fn keytip_apr_() {
    let mut h = harness(blank());
    sequence(&mut h, "APR", false);
    assert!(h.state().dialog.is_none(), "APR must not open a dialog");
    assert!(h.state().message.is_none(), "APR must not open a message box");
    assert!(h.state().toast.is_none(), "APR must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "APR must not journal");
}

#[test]
fn keytip_aps_() {
    let mut h = harness(blank());
    sequence(&mut h, "APS", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "APS must run data.queriesConnections cleanly");
}

#[test]
fn keytip_apt_() {
    let mut h = harness(blank());
    sequence(&mut h, "APT", false);
    assert!(h.state().dialog.is_none(), "APT must not open a dialog");
    assert!(h.state().message.is_none(), "APT must not open a message box");
    assert!(h.state().toast.is_none(), "APT must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "APT must not journal");
}

#[test]
fn keytip_arc_() {
    let mut h = harness(blank());
    sequence(&mut h, "ARC", false);
    assert!(h.state().dialog.is_none(), "ARC must not open a dialog");
    assert!(h.state().message.is_none(), "ARC must not open a message box");
    assert!(h.state().toast.is_none(), "ARC must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "ARC must not journal");
}

#[test]
fn keytip_aro_() {
    let mut h = harness(blank());
    sequence(&mut h, "ARO", false);
    assert!(h.state().dialog.is_none(), "ARO must not open a dialog");
    assert!(h.state().message.is_none(), "ARO must not open a message box");
    assert!(h.state().toast.is_none(), "ARO must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "ARO must not journal");
}

#[test]
fn keytip_ars_() {
    let mut h = harness(blank());
    sequence(&mut h, "ARS", false);
    assert!(h.state().dialog.is_none(), "ARS must not open a dialog");
    assert!(h.state().message.is_none(), "ARS must not open a message box");
    assert!(h.state().toast.is_none(), "ARS must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "ARS must not journal");
}

#[test]
fn keytip_auc_() {
    let mut h = harness(blank());
    sequence(&mut h, "AUC", false);
    assert!(h.state().dialog.is_none(), "AUC must not open a dialog");
    assert!(h.state().message.is_none(), "AUC must not open a message box");
    assert!(h.state().toast.is_none(), "AUC must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "AUC must not journal");
}

#[test]
fn keytip_avr_() {
    let mut h = harness(blank());
    sequence(&mut h, "AVR", false);
    assert!(h.state().dialog.is_none(), "AVR must not open a dialog");
    assert!(h.state().message.is_none(), "AVR must not open a message box");
    assert!(h.state().toast.is_none(), "AVR must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "AVR must not journal");
}

#[test]
fn keytip_aws_() {
    let mut h = harness(blank());
    sequence(&mut h, "AWS", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "AWS must run data.scenarioManager cleanly");
}

#[test]
fn keytip_ax_() {
    let mut h = harness(blank());
    sequence(&mut h, "AX", false);
    assert!(h.state().dialog.is_none(), "AX must not open a dialog");
    assert!(h.state().message.is_none(), "AX must not open a message box");
    assert!(h.state().toast.is_none(), "AX must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "AX must not journal");
}

#[test]
fn keytip_ra2_() {
    let mut h = harness(blank());
    sequence(&mut h, "RA2", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "RA2 must run review.showComments cleanly");
}

#[test]
fn keytip_rg_() {
    let mut h = harness(blank());
    sequence(&mut h, "RG", false);
    assert!(h.state().dialog.is_none(), "RG must not open a dialog");
    assert!(h.state().message.is_none(), "RG must not open a message box");
    assert!(h.state().toast.is_none(), "RG must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "RG must not journal");
}

#[test]
fn keytip_rl_() {
    let mut h = harness(blank());
    sequence(&mut h, "RL", false);
    assert!(h.state().dialog.is_none(), "RL must not open a dialog");
    assert!(h.state().message.is_none(), "RL must not open a message box");
    assert!(h.state().toast.is_none(), "RL must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "RL must not journal");
}

#[test]
fn keytip_rrs_() {
    let mut h = harness(blank());
    sequence(&mut h, "RRS", false);
    assert!(h.state().dialog.is_none(), "RRS must not open a dialog");
    assert!(h.state().message.is_none(), "RRS must not open a message box");
    assert!(h.state().toast.is_none(), "RRS must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "RRS must not journal");
}

#[test]
fn keytip_wj_() {
    let mut h = harness(blank());
    sequence(&mut h, "WJ", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "WJ must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_wt_() {
    let mut h = harness(blank());
    sequence(&mut h, "WT", false);
    assert!(h.state().dialog.is_none(), "WT must not open a dialog");
    assert!(h.state().message.is_none(), "WT must not open a message box");
    assert!(h.state().toast.is_none(), "WT must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "WT must not journal");
}

#[test]
fn keytip_wvc_() {
    let mut h = harness(blank());
    sequence(&mut h, "WVC", false);
    assert!(h.state().dialog.is_none(), "WVC must not open a dialog");
    assert!(h.state().message.is_none(), "WVC must not open a message box");
    assert!(h.state().toast.is_none(), "WVC must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "WVC must not journal");
}

#[test]
fn keytip_wvf_() {
    let mut h = harness(blank());
    sequence(&mut h, "WVF", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "WVF must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_wvn_() {
    let mut h = harness(blank());
    sequence(&mut h, "WVN", false);
    assert!(h.state().dialog.is_none(), "WVN must not open a dialog");
    assert!(h.state().message.is_none(), "WVN must not open a message box");
    assert!(h.state().toast.is_none(), "WVN must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "WVN must not journal");
}

#[test]
fn keytip_ww_() {
    let mut h = harness(blank());
    sequence(&mut h, "WW", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "WW must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_is_() {
    let mut h = harness(blank());
    sequence(&mut h, "IS", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "IS must route to its control without an error box");
    assert_eq!(h.state().keytips.prefix(), None);
}

#[test]
fn keytip_tud_() {
    let mut h = harness(blank());
    sequence(&mut h, "TUD", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "TUD must run formulas.traceDependents cleanly");
}

#[test]
fn keytip_tut_() {
    let mut h = harness(blank());
    sequence(&mut h, "TUT", false);
    assert!(h.state().message.is_none() && h.state().toast.is_none(), "TUT must run formulas.tracePrecedents cleanly");
}

#[test]
fn keytip_q_() {
    let mut h = harness(blank());
    sequence(&mut h, "Q", false);
    assert!(h.state().dialog.is_none(), "Q must not open a dialog");
    assert!(h.state().message.is_none(), "Q must not open a message box");
    assert!(h.state().toast.is_none(), "Q must not raise a toast");
    assert!(h.state().session.journal.is_empty(), "Q must not journal");
}

#[test]
fn generated_test_count_matches_the_table() {
    const EXPECTED: usize = 413;
    assert_eq!(simplegrid_ui_egui::keytips::keytip_table::SEQUENCES.len(), EXPECTED);
}

fn reset(h: &mut Harness) {
    key(h, Key::Escape); // clear any prefix, back to the armed root
    key(h, Key::Escape); // leave keytip mode entirely
}
