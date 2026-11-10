//! SimpleGrid: die Seitenansicht. Zeigt groß, was gedruckt wird (Seite für Seite, mit Zoom), und
//! bietet die wenigen Druck-Einstellungen als Schaltflächen: Hoch-/Querformat, Auf eine Seite,
//! Farbe/Schwarz-Weiß. Seitenaufteilung und Druck kommen aus der GridCraft-Engine
//! (`file.printPreview`, `file.print`, `file.exportPdf`); gedruckt wird immer mit Seitenzahl unten.

use egui::{RichText, Ui, vec2};
use serde_json::{Value as Json, json};

use crate::SheetApp;
use crate::einfach::masse;
use crate::theme::{self, Tokens};

/// Zustand der offenen Seitenansicht.
#[derive(Clone, Debug, Default)]
pub struct Seitenansicht {
    /// Angezeigte Seite (ab 0).
    pub seite: usize,
    /// Vergrößerung der Vorschau (1.0 = ganze Seite passt ins Fenster).
    pub zoom: f32,
    /// Schwarz-Weiß drucken.
    pub schwarz_weiss: bool,
    /// Zwischengespeicherte Seitenaufteilung: (Mappe, Stand, Schwarz-Weiß) → Ergebnis.
    vorschau: Option<((u64, u64, bool), Json)>,
}

/// Die Druck-Parameter, die Vorschau, Drucken und PDF gemeinsam nutzen.
pub fn druck_parameter(schwarz_weiss: bool) -> Json {
    json!({"sheets": "active", "pageNumbers": true, "blackWhite": schwarz_weiss})
}

/// Öffnet die Seitenansicht.
pub fn oeffnen(app: &mut SheetApp) {
    if app.commit_edit(0, 0, false, false) {
        app.einfach.seitenansicht = Some(Seitenansicht { zoom: 1.0, ..Default::default() });
    }
}

/// Die Seiten, wie sie gedruckt werden (aus der Engine).
pub fn seiten(app: &mut SheetApp, sa: &mut Seitenansicht) -> Vec<Json> {
    let Some(key) = app.session.active().map(|d| (d.uid, d.revision, sa.schwarz_weiss)) else { return vec![] };
    // Ein leeres Blatt ohne Druckbereich: nichts drucken (statt einer leeren Seite).
    let leer = app.session.active().and_then(|d| d.wb.active()).is_none_or(|sh| sh.used_range().is_none() && sh.print.print_area.is_none());
    if leer {
        return vec![];
    }
    if sa.vorschau.as_ref().is_none_or(|(k, _)| *k != key) {
        let v = app.session.run("file.printPreview", druck_parameter(sa.schwarz_weiss)).unwrap_or(Json::Null);
        sa.vorschau = Some((key, v));
    }
    sa.vorschau.as_ref().and_then(|(_, v)| v.get("pageList")).and_then(Json::as_array).cloned().unwrap_or_default()
}

fn knopf(ui: &mut Ui, text: &str, tipp: &str, an: bool, aus: Option<&str>) -> bool {
    let b = egui::Button::new(RichText::new(text).font(theme::ui_bold(masse::SCHRIFT)))
        .selected(an)
        .corner_radius(10.0)
        .min_size(vec2(masse::KNOPF_BREITE, masse::KNOPF_HOEHE));
    let r = ui.add_enabled(aus.is_none(), b);
    let r = match aus {
        Some(g) => r.on_disabled_hover_text(format!("{tipp}\n\n{g}")),
        None => r.on_hover_text(tipp),
    };
    r.clicked()
}

/// Zeichnet die Seitenansicht anstelle der Tabelle.
pub fn zeigen(app: &mut SheetApp, ui: &mut Ui) {
    let Some(mut sa) = app.einfach.seitenansicht.take() else { return };
    let t = Tokens::get(ui.ctx());
    let mut schliessen = false;
    let seiten = seiten(app, &mut sa);
    let (hochformat, eine_seite) = app
        .session
        .active()
        .and_then(|d| d.wb.active())
        .map(|sh| (sh.print.orientation != simplegrid_engine::model::Orientation::Landscape, sh.print.fit_to == Some((1, 1))))
        .unwrap_or((true, false));
    // Kopf: zurück zur Tabelle und Überschrift.
    egui::Panel::top("simplegrid_seitenansicht_kopf")
        .resizable(false)
        .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin::symmetric(20, 10)).stroke(egui::Stroke::new(1.0, t.ribbon_border)))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if knopf(ui, "◀ Zurück zur Tabelle", "Die Seitenansicht schließen", false, None) {
                    schliessen = true;
                }
                ui.add_space(12.0);
                ui.label(RichText::new("Seitenansicht – so wird gedruckt").font(theme::ui_bold(26.0)).color(t.text));
            });
        });
    // Links: die Einstellungen als Schritte, darunter „Jetzt drucken“.
    let lang = app.ui.language;
    egui::Panel::left("simplegrid_seitenansicht_schritte")
        .resizable(false)
        .exact_size(380.0)
        .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin::same(22)).stroke(egui::Stroke::new(1.0, t.ribbon_border)))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
                let schritt = |ui: &mut Ui, text: &str| {
                    ui.add_space(6.0);
                    ui.label(RichText::new(text).font(theme::ui_bold(18.0)).color(t.text));
                };
                let breit = |ui: &mut Ui, text: &str, tipp: &str, an: bool| -> bool {
                    let b = egui::Button::new(RichText::new(text).font(if an { theme::ui_bold(19.0) } else { theme::ui_font(19.0) }).color(if an {
                        t.accent_dark
                    } else {
                        t.text
                    }))
                    .fill(if an { t.accent_soft } else { egui::Color32::WHITE })
                    .stroke(egui::Stroke::new(if an { 3.0 } else { 2.0 }, if an { t.accent } else { t.ribbon_border }))
                    .corner_radius(12.0)
                    .min_size(vec2(160.0, 62.0));
                    ui.add(b).on_hover_text(tipp).clicked()
                };
                schritt(ui, "1. Wie soll das Blatt liegen?");
                ui.horizontal(|ui| {
                    if breit(ui, "Hochformat", "Das Blatt hochkant drucken", hochformat) && !hochformat {
                        app.run_or_alert("pageLayout.orientation", json!({"orientation": "portrait"}));
                    }
                    if breit(ui, "Querformat", "Das Blatt quer drucken", !hochformat) && hochformat {
                        app.run_or_alert("pageLayout.orientation", json!({"orientation": "landscape"}));
                    }
                });
                if breit(
                    ui,
                    "Alles auf eine Seite",
                    "Alles verkleinert auf eine einzige Seite drucken (noch einmal klicken: normale Größe)",
                    eine_seite,
                ) {
                    auf_eine_seite(app, !eine_seite);
                }
                schritt(ui, "2. In Farbe?");
                ui.horizontal(|ui| {
                    if breit(ui, "Farbe", "In Farbe drucken", !sa.schwarz_weiss) {
                        sa.schwarz_weiss = false;
                    }
                    if breit(ui, "Schwarz-Weiß", "Ohne Farbe drucken: Hintergründe werden hellgrau, Schrift und Linien schwarz", sa.schwarz_weiss) {
                        sa.schwarz_weiss = true;
                    }
                });
                if let Some(suche) = &app.services.printers {
                    schritt(ui, "3. Drucker");
                    crate::printing::show(&mut app.printing, ui, suche, |label| lang.tr(label).to_owned());
                }
                ui.add_space(18.0);
                let nichts = seiten.is_empty().then_some("Auf diesem Blatt steht nichts zum Drucken.");
                let kein_drucker = (app.services.print_native.is_some() && !app.printing.ready()).then_some("Bitte zuerst einen Drucker wählen.");
                let aus = nichts.or(kein_drucker);
                let drucken = egui::Button::new(RichText::new("Jetzt drucken").font(theme::ui_bold(24.0)).color(egui::Color32::WHITE))
                    .fill(if aus.is_some() { t.text_disabled } else { t.accent })
                    .stroke(egui::Stroke::new(3.0, if aus.is_some() { t.text_disabled } else { t.accent }))
                    .corner_radius(14.0)
                    .min_size(vec2(330.0, 80.0));
                let r = ui.add_enabled(aus.is_none(), drucken);
                let r = match aus {
                    Some(g) => r.on_disabled_hover_text(g),
                    None => r.on_hover_text("Auf dem Drucker ausgeben, so wie rechts gezeigt"),
                };
                if r.clicked() {
                    app.print_now(druck_parameter(sa.schwarz_weiss));
                }
                if knopf(ui, "Als PDF speichern", "Eine PDF-Datei speichern (zum Ansehen oder Verschicken)", false, nichts) {
                    app.export_pdf_with_picker(druck_parameter(sa.schwarz_weiss));
                }
            });
        });
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.desk)).show(ui, |ui| {
        if seiten.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Auf diesem Blatt steht nichts zum Drucken.").font(theme::ui_font(masse::SCHRIFT + 4.0)))
            });
            return;
        }
        let letzte = seiten.len() - 1;
        sa.seite = sa.seite.min(letzte);
        // Untere Leiste: Blättern und Zoom.
        egui::Panel::bottom("simplegrid_seitenansicht_blaettern")
            .resizable(false)
            .frame(egui::Frame::NONE.fill(t.window).inner_margin(egui::Margin::same(8)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = masse::ABSTAND;
                    if knopf(ui, "◀ Vorige Seite", "Eine Seite zurückblättern", false, (sa.seite == 0).then_some("Das ist die erste Seite.")) {
                        sa.seite -= 1;
                    }
                    ui.label(RichText::new(format!("Seite {} von {}", sa.seite + 1, seiten.len())).font(theme::ui_bold(masse::SCHRIFT)));
                    if knopf(ui, "Nächste Seite ▶", "Eine Seite weiterblättern", false, (sa.seite >= letzte).then_some("Das ist die letzte Seite."))
                    {
                        sa.seite += 1;
                    }
                    ui.separator();
                    if knopf(ui, "– Kleiner", "Die Vorschau kleiner zeigen", false, (sa.zoom <= 1.0).then_some("Die ganze Seite ist schon zu sehen."))
                    {
                        sa.zoom = (sa.zoom - 0.5).max(1.0);
                    }
                    ui.label(RichText::new(format!("{} %", (sa.zoom * 100.0).round())).font(theme::ui_bold(masse::SCHRIFT)));
                    if knopf(ui, "+ Größer", "Die Vorschau vergrößern", false, (sa.zoom >= 3.0).then_some("Größer geht es nicht.")) {
                        sa.zoom = (sa.zoom + 0.5).min(3.0);
                    }
                });
            });
        let Some(page) = seiten.get(sa.seite) else { return };
        let w = page.get("width").and_then(Json::as_f64).filter(|w| w.is_finite() && *w > 1.0).unwrap_or(595.0) as f32;
        let h = page.get("height").and_then(Json::as_f64).filter(|h| h.is_finite() && *h > 1.0).unwrap_or(842.0) as f32;
        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            let platz = ui.available_size() - vec2(40.0, 40.0);
            let scale = (platz.y / h).min(platz.x / w).max(0.1) * sa.zoom;
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                ui.add_space(((ui.available_width() - w * scale) / 2.0).max(20.0));
                let (rect, _) = ui.allocate_exact_size(vec2(w * scale, h * scale), egui::Sense::hover());
                let p = ui.painter();
                p.rect_filled(rect.translate(vec2(3.0, 4.0)), 2.0, t.shadow);
                p.rect_filled(rect, 0.0, egui::Color32::WHITE);
                if let Some(d) = app.session.active() {
                    crate::backstage::paint_page(p, rect, w, page, &d.wb, sa.schwarz_weiss);
                }
                // Seitenzahl unten in der Mitte, wie sie gedruckt wird.
                let fuss = egui::pos2(rect.center().x, rect.bottom() - 0.45 * 72.0 * rect.width() / w);
                p.text(
                    fuss,
                    egui::Align2::CENTER_CENTER,
                    format!("Seite {} von {}", sa.seite + 1, seiten.len()),
                    theme::ui_font((10.0 * rect.width() / w).clamp(4.0, 30.0)),
                    egui::Color32::BLACK,
                );
            });
            ui.add_space(20.0);
        });
    });
    if !schliessen && !ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.einfach.seitenansicht = Some(sa);
    }
}

/// „Auf eine Seite“: an → alles auf eine Seite verkleinern; aus → normale Größe (100 %).
/// Wird im Blatt gespeichert, also auch in der .xlsx-Datei.
pub fn auf_eine_seite(app: &mut SheetApp, an: bool) {
    let p = if an { json!({"width": 1, "height": 1}) } else { json!({"scale": 100}) };
    app.run_or_alert("pageLayout.scaleToFit", p);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Services;
    use simplegrid_engine::Session;

    fn app_mit_zeilen(n: u32) -> SheetApp {
        let mut session = Session::new();
        crate::einfach::deutsche_vorgaben(&mut session);
        session.new_workbook();
        let mut a = SheetApp::new(session, Services::default());
        for r in 1..=n {
            a.run_or_alert("cell.set", json!({"cell": format!("A{r}"), "input": format!("Zeile {r}")}));
            a.run_or_alert("cell.set", json!({"cell": format!("H{r}"), "input": r.to_string()}));
        }
        a
    }

    fn seitenzahl(a: &mut SheetApp) -> usize {
        let mut sa = Seitenansicht { zoom: 1.0, ..Default::default() };
        seiten(a, &mut sa).len()
    }

    /// Die Seiten in Druckreihenfolge decken jede Zeile genau einmal ab (Umbruch nur an Zellgrenzen).
    fn zeilen_lueckenlos(a: &mut SheetApp, n: u32) {
        let mut sa = Seitenansicht { zoom: 1.0, ..Default::default() };
        let mut gesehen = vec![0u32; n as usize];
        for p in seiten(a, &mut sa) {
            let r = p["range"].as_str().and_then(simplegrid_engine::core::RangeRef::parse).unwrap_or_else(|| unreachable!());
            if r.start.col == 0 {
                for row in r.start.row..=r.end.row.min(n - 1) {
                    gesehen[row as usize] += 1;
                }
            }
        }
        assert!(gesehen.iter().all(|x| *x == 1), "{gesehen:?}");
    }

    #[test]
    fn hoch_quer_und_auf_eine_seite() {
        let mut a = app_mit_zeilen(150);
        let hoch = seitenzahl(&mut a);
        assert!(hoch >= 3, "150 Zeilen brauchen im Hochformat mehrere Seiten: {hoch}");
        zeilen_lueckenlos(&mut a, 150);
        a.run_or_alert("pageLayout.orientation", json!({"orientation": "landscape"}));
        let quer = seitenzahl(&mut a);
        assert!(quer > hoch, "quer passen weniger Zeilen auf eine Seite: {quer} / {hoch}");
        zeilen_lueckenlos(&mut a, 150);
        auf_eine_seite(&mut a, true);
        assert_eq!(seitenzahl(&mut a), 1);
        auf_eine_seite(&mut a, false);
        assert_eq!(seitenzahl(&mut a), quer);
    }

    #[test]
    fn druckbereich_festlegen_und_aufheben() {
        let mut a = app_mit_zeilen(150);
        a.run_or_alert("selection.set", json!({"range": "A1:B10"}));
        a.run_or_alert("pageLayout.printArea", json!({}));
        let mut sa = Seitenansicht { zoom: 1.0, ..Default::default() };
        let s = seiten(&mut a, &mut sa);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0]["range"], json!("A1:B10"));
        a.run_or_alert("pageLayout.printArea", json!({"clear": true}));
        assert!(seitenzahl(&mut a) > 1);
    }

    #[test]
    fn leeres_blatt_hat_nichts_zu_drucken() {
        let mut a = app_mit_zeilen(0);
        assert_eq!(seitenzahl(&mut a), 0);
    }

    #[test]
    fn seitenzahl_unten_und_keine_kopfzeile() {
        let mut a = app_mit_zeilen(5);
        a.run_or_alert("pageLayout.headerFooter", json!({"header": "Geheim", "footer": "Alt"}));
        let d = a.session.active().unwrap_or_else(|| unreachable!());
        let mut opts = simplegrid_engine::cmd::print::PrintOptions::default();
        opts.page_numbers = true;
        let lay = simplegrid_engine::cmd::print::layout_sheet(&d.wb, 0, &opts, None).unwrap_or_else(|| unreachable!());
        assert_eq!(lay.settings.header, "");
        assert_eq!(lay.settings.footer, "&CSeite &P von &N");
    }

    #[test]
    fn druck_einstellungen_werden_gespeichert() {
        let dir = std::env::temp_dir().join(format!("simplegrid-druck-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let pfad = dir.join("druck.xlsx").to_string_lossy().into_owned();
        let mut a = app_mit_zeilen(20);
        a.run_or_alert("selection.set", json!({"range": "A1:H20"}));
        a.run_or_alert("pageLayout.printArea", json!({}));
        a.run_or_alert("pageLayout.orientation", json!({"orientation": "landscape"}));
        auf_eine_seite(&mut a, true);
        a.run_or_alert("file.save", json!({"path": pfad}));
        let mut b = app_mit_zeilen(0);
        crate::einfach::oeffnen_pfad(&mut b, &pfad);
        let sh = b.session.active().and_then(|d| d.wb.active().cloned()).unwrap_or_else(|| unreachable!());
        assert_eq!(sh.print.print_area.map(|r| r.a1()).as_deref(), Some("A1:H20"));
        assert_eq!(sh.print.orientation, simplegrid_engine::model::Orientation::Landscape);
        assert_eq!(sh.print.fit_to, Some((1, 1)));
    }

    #[test]
    fn pdf_mit_schwarz_weiss_wird_erzeugt() {
        let mut a = app_mit_zeilen(3);
        a.run_or_alert("home.fillColor", json!({"range": "A1", "color": "#FF0000"}));
        let farbe = a.session.run("file.print", druck_parameter(false)).unwrap_or_default();
        let sw = a.session.run("file.print", druck_parameter(true)).unwrap_or_default();
        let text = |v: &Json| {
            v["base64"].as_str().and_then(simplegrid_engine::io::base64_decode).map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default()
        };
        assert!(text(&farbe).contains("1 0 0 rg"), "rote Fläche in Farbe");
        assert!(!text(&sw).contains("1 0 0 rg"), "keine rote Fläche in Schwarz-Weiß");
        assert!(text(&sw).contains("Seite"), "Seitenzahl im Druck");
    }
}
