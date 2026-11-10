//! Ribbon layout: the big split button, the scroll arrows of an overflowing tab, and every tab
//! fitting the card's height.

use egui::vec2;
use egui_kittest::kittest::Queryable as _;
use simplegrid_engine::Session;
use simplegrid_ui_egui::icons::Icon;
use simplegrid_ui_egui::ribbon::{RibbonOverflow, TABS};
use simplegrid_ui_egui::{SheetApp, widgets};

type Harness = egui_kittest::Harness<'static, SheetApp>;

fn app_at(width: f32, tab: &str) -> Harness {
    let mut session = Session::new();
    session.new_workbook();
    let mut app = crate::common::app(session);
    app.ui.ribbon_tab = tab.into();
    let mut h = crate::common::builder().with_size(vec2(width, 700.0)).build_ui_state(
        |ui, app: &mut SheetApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(4);
    h
}

#[derive(Default)]
struct Clicks {
    main: u32,
    arrow: u32,
}

#[test]
/// The main button clicks once and the arrow twice separately; the tooltip keeps the platform shortcut, and the icon sits above the full-width label.
fn big_split_button_separates_the_default_action_from_the_menu() {
    // The ribbon picks the shortcut by the OS egui reports, which kittest pins to Nix unless told:
    // pin macOS, so the check is the same on every host (the other platforms' text is in
    // `widgets::tests`).
    let mut h = crate::common::builder().with_size(vec2(300.0, 200.0)).with_os(egui::os::OperatingSystem::Mac).build_ui_state(
        |ui, c: &mut Clicks| {
            let (main, arrow) = widgets::big_split_button(ui, Icon::Sum, "Auto\nSum", "AutoSum");
            c.main += u32::from(main.clicked());
            c.arrow += u32::from(arrow.clicked());
        },
        Clicks::default(),
    );
    h.run_steps(2);
    h.get_by_label("Auto Sum").click();
    h.run_steps(2);
    h.get_by_label("Auto Sum options").click();
    h.run_steps(2);
    h.get_by_label("Auto Sum options").click();
    h.run_steps(2);
    assert_eq!((h.state().main, h.state().arrow), (1, 2));
    // The tooltip carries the shortcut hint, as every ribbon button's does. egui 0.37 waits for
    // the pointer to settle before opening it, so remove the hover delay and let it settle.
    h.ctx.all_styles_mut(|s| s.interaction.tooltip_delay = 0.0);
    h.get_by_label("Auto Sum").hover();
    h.run_steps(6);
    assert!(h.query_by_value("AutoSum (\u{2318}\u{21e7}T)").is_some(), "tooltip with the macOS shortcut");
    // The icon part sits above the label part, across the full width.
    let main = h.get_by_label("Auto Sum").rect();
    let arrow = h.get_by_label("Auto Sum options").rect();
    assert!(main.max.y <= arrow.min.y + 0.5);
    assert!((main.min.x - arrow.min.x).abs() < 0.5 && (main.max.x - arrow.max.x).abs() < 0.5);
}
