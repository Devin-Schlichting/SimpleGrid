//! Die einfache Oberfläche von SimpleGrid: eine feste, beschriftete Schaltflächenleiste oben statt
//! des Menübands, große Klickziele und Rückfragen vor allem, was Arbeit kosten kann.
//!
//! Alles läuft über die Befehle der Engine (`app.run_or_alert(id, params)`), damit Rückgängig und
//! Wiederholen überall gelten. Schaltflächen, deren Befehl gerade nicht möglich ist, sind
//! ausgegraut und sagen beim Darüberfahren, warum.

use egui::{RichText, Ui, vec2};
use serde_json::{Value as Json, json};

use crate::SheetApp;
use crate::theme::{self, Tokens};

/// Zentrale Maße der einfachen Oberfläche. Eine Änderung hier wirkt auf alle Schaltflächen.
pub mod masse {
    /// Mindesthöhe einer Schaltfläche in Punkten.
    pub const KNOPF_HOEHE: f32 = 44.0;
    /// Mindestbreite einer Schaltfläche in Punkten.
    pub const KNOPF_BREITE: f32 = 84.0;
    /// Schriftgröße der Beschriftungen.
    pub const SCHRIFT: f32 = 16.0;
    /// Schriftgröße der Gruppenüberschriften.
    pub const GRUPPEN_SCHRIFT: f32 = 12.5;
    /// Abstand zwischen zwei Schaltflächen.
    pub const ABSTAND: f32 = 6.0;
}

/// Zentrale Texte der einfachen Oberfläche, in einfachen Alltagswörtern.
pub mod texte {
    pub const NICHT_MOEGLICH: &str = "Das geht gerade nicht.";
    pub const ABBRECHEN: &str = "Abbrechen";
    pub const SPEICHERN: &str = "Speichern";
    pub const NICHT_SPEICHERN: &str = "Nicht speichern";
    pub const LOESCHEN: &str = "Löschen";
    pub const UNGESPEICHERT_TITEL: &str = "Änderungen speichern?";
    pub const UNGESPEICHERT_TEXT: &str = "Die Tabelle „{}“ hat Änderungen, die noch nicht gespeichert sind. Möchten Sie sie vorher speichern?";
    pub const BLATT_LOESCHEN_TITEL: &str = "Blatt löschen?";
    pub const BLATT_LOESCHEN_TEXT: &str = "Soll das Blatt „{}“ mit allem, was darauf steht, gelöscht werden? Mit „Rückgängig“ können Sie es zurückholen.";
    pub const LETZTES_BLATT: &str = "Das letzte Blatt kann nicht gelöscht werden.";
}

/// Was nach einer Rückfrage zu ungespeicherten Änderungen passieren soll.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Danach {
    /// Eine neue, leere Tabelle anlegen.
    Neu,
    /// Eine Datei öffnen.
    Oeffnen,
}

/// Eine offene Rückfrage der einfachen Oberfläche.
#[derive(Clone, Debug, PartialEq)]
pub enum Frage {
    /// Ungespeicherte Änderungen vor „Neu“ oder „Öffnen“.
    Ungespeichert { danach: Danach },
    /// Ein Tabellenblatt wirklich löschen?
    BlattLoeschen { blatt: usize, name: String },
}

/// Zustand der einfachen Oberfläche.
#[derive(Clone, Debug, Default)]
pub struct Zustand {
    pub frage: Option<Frage>,
}

/// Der Grund, warum Befehl `id` gerade nicht geht, oder `None`, wenn er geht.
pub fn gesperrt(app: &SheetApp, id: &str) -> Option<String> {
    let spec = simplegrid_engine::find_command(id)?;
    (spec.enabled)(&app.session).err()
}

/// Eine große, beschriftete Schaltfläche. `aus` sperrt sie mit Begründung; `an` zeigt sie
/// eingedrückt (z. B. „Fett“, wenn die Zelle fett ist).
fn knopf(ui: &mut Ui, text: &str, tipp: &str, aus: Option<&str>, an: bool) -> bool {
    let beschriftung = RichText::new(text).font(theme::ui_font(masse::SCHRIFT));
    let b = egui::Button::new(beschriftung).selected(an).min_size(vec2(masse::KNOPF_BREITE, masse::KNOPF_HOEHE));
    let r = ui.add_enabled(aus.is_none(), b);
    let r = match aus {
        Some(grund) => r.on_disabled_hover_text(format!("{tipp}\n\n{grund}")),
        None => r.on_hover_text(tipp),
    };
    r.clicked()
}

#[allow(dead_code)] // wird ab Phase 3 von weiteren Gruppen genutzt
/// Eine Schaltfläche, die einen Engine-Befehl ausführt; gesperrt, solange der Befehl nicht geht.
fn befehl(app: &mut SheetApp, ui: &mut Ui, text: &str, tipp: &str, id: &str, params: Json) {
    let aus = gesperrt(app, id).map(|_| texte::NICHT_MOEGLICH.to_string());
    if knopf(ui, text, tipp, aus.as_deref(), false) {
        app.run_or_alert(id, params);
    }
}

/// Eine Gruppe von Schaltflächen mit kleiner Überschrift.
fn gruppe(ui: &mut Ui, titel: &str, inhalt: impl FnOnce(&mut Ui)) {
    let t = Tokens::get(ui.ctx());
    egui::Frame::NONE.fill(t.ribbon).stroke(egui::Stroke::new(1.0, t.ribbon_border)).corner_radius(8.0).inner_margin(6.0).show(ui, |ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(titel).font(theme::ui_font(masse::GRUPPEN_SCHRIFT)).color(t.text_dim));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = masse::ABSTAND;
                inhalt(ui);
            });
        });
    });
}

/// Zeichnet die feste Schaltflächenleiste oben im Fenster.
pub fn leiste(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top("simplegrid_leiste").frame(egui::Frame::NONE.fill(t.window).inner_margin(egui::Margin::same(8))).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            gruppe(ui, "Datei", |ui| datei(app, ui));
            gruppe(ui, "Tabellenblatt", |ui| blaetter(app, ui));
        });
    });
}

fn datei(app: &mut SheetApp, ui: &mut Ui) {
    if knopf(ui, "Neu", "Eine neue, leere Tabelle anfangen", None, false) {
        neu_oder_oeffnen(app, Danach::Neu);
    }
}

fn blaetter(app: &mut SheetApp, ui: &mut Ui) {
    let (aktiv, anzahl, name) = app
        .session
        .active()
        .map(|d| (d.wb.active_sheet.get(), d.wb.sheets.len(), d.wb.active().map(|s| s.name.clone()).unwrap_or_default()))
        .unwrap_or((0, 0, String::new()));
    let ohne = gesperrt(app, "home.insertSheet").map(|_| texte::NICHT_MOEGLICH);
    if knopf(ui, "Blatt hinzufügen", "Ein neues, leeres Tabellenblatt hinten anfügen", ohne, false) {
        app.run_or_alert("home.insertSheet", json!({"before": anzahl}));
    }
    if knopf(ui, "Blatt umbenennen", "Dem aktuellen Blatt einen neuen Namen geben", ohne, false) {
        app.grid.renaming_tab = Some(aktiv);
        app.grid.rename_text = name.clone();
    }
    let loeschen_aus = if anzahl <= 1 { Some(texte::LETZTES_BLATT) } else { ohne };
    if knopf(ui, "Blatt löschen", "Das aktuelle Blatt löschen (mit Rückfrage)", loeschen_aus, false) {
        app.einfach.frage = Some(Frage::BlattLoeschen { blatt: aktiv, name: name.clone() });
    }
    let links_aus = if aktiv == 0 { Some("Das Blatt steht schon ganz links.") } else { ohne };
    if knopf(ui, "◀ Blatt nach links", "Das aktuelle Blatt eine Stelle nach links schieben", links_aus, false) {
        app.run_or_alert("sheet.move", json!({"sheet": aktiv, "to": aktiv.saturating_sub(1)}));
    }
    let rechts_aus = if aktiv + 1 >= anzahl { Some("Das Blatt steht schon ganz rechts.") } else { ohne };
    if knopf(ui, "Blatt nach rechts ▶", "Das aktuelle Blatt eine Stelle nach rechts schieben", rechts_aus, false) {
        app.run_or_alert("sheet.move", json!({"sheet": aktiv, "to": aktiv + 1}));
    }
}

/// „Neu“ oder „Öffnen“: bei ungespeicherten Änderungen erst fragen.
pub fn neu_oder_oeffnen(app: &mut SheetApp, danach: Danach) {
    if !app.commit_edit(0, 0, false, false) {
        return;
    }
    if app.session.active().is_some_and(|d| d.is_dirty()) {
        app.einfach.frage = Some(Frage::Ungespeichert { danach });
    } else {
        ausfuehren(app, danach);
    }
}

/// Ersetzt die aktuelle Tabelle durch eine neue bzw. eine geöffnete. SimpleGrid zeigt immer genau
/// eine Tabelle; das vermeidet versteckte Fenster.
fn ausfuehren(app: &mut SheetApp, danach: Danach) {
    match danach {
        Danach::Neu => {
            if app.session.active().is_some() {
                app.run_or_alert("file.close", json!({"force": true, "document": true}));
            }
            app.run_or_alert("file.new", json!({}));
        }
        Danach::Oeffnen => app.run_or_alert("file.open", json!({})),
    }
}

/// Zeigt eine offene Rückfrage als Fenster mit großen Schaltflächen.
pub fn rueckfragen(app: &mut SheetApp, ctx: &egui::Context) {
    let Some(frage) = app.einfach.frage.clone() else { return };
    let (titel, text, knoepfe): (&str, String, Vec<&str>) = match &frage {
        Frage::Ungespeichert { .. } => {
            let name = app.session.active().map(|d| d.display_title()).unwrap_or_default();
            (texte::UNGESPEICHERT_TITEL, texte::UNGESPEICHERT_TEXT.replace("{}", &name), vec![texte::SPEICHERN, texte::NICHT_SPEICHERN, texte::ABBRECHEN])
        }
        Frage::BlattLoeschen { name, .. } => (texte::BLATT_LOESCHEN_TITEL, texte::BLATT_LOESCHEN_TEXT.replace("{}", name), vec![texte::LOESCHEN, texte::ABBRECHEN]),
    };
    let mut gewaehlt: Option<&str> = None;
    egui::Modal::new(egui::Id::unique("simplegrid.rueckfrage")).show(ctx, |ui| {
        ui.set_width(460.0);
        ui.label(RichText::new(titel).font(theme::ui_bold(masse::SCHRIFT + 2.0)));
        ui.add_space(6.0);
        ui.add(egui::Label::new(RichText::new(&text).font(theme::ui_font(masse::SCHRIFT))).wrap());
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = masse::ABSTAND * 2.0;
            for k in &knoepfe {
                if knopf(ui, k, k, None, false) {
                    gewaehlt = Some(k);
                }
            }
        });
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            gewaehlt = Some(texte::ABBRECHEN);
        }
    });
    let Some(wahl) = gewaehlt else { return };
    app.einfach.frage = None;
    match frage {
        Frage::Ungespeichert { danach } => match wahl {
            texte::SPEICHERN => {
                app.run_or_alert("file.save", json!({}));
                // Nur weiter, wenn wirklich gespeichert wurde (sonst bleibt alles, wie es war).
                if app.session.active().is_some_and(|d| !d.is_dirty()) {
                    ausfuehren(app, danach);
                }
            }
            texte::NICHT_SPEICHERN => ausfuehren(app, danach),
            _ => {}
        },
        Frage::BlattLoeschen { blatt, .. } => {
            if wahl == texte::LOESCHEN {
                app.run_or_alert("home.deleteSheet", json!({"sheet": blatt}));
            }
        }
    }
}

/// Die einzigen Tastenkürzel neben Strg+C/X/V: Strg+Z, Strg+Y und Strg+S. Sie rufen dieselben
/// Befehle wie die Schaltflächen.
pub fn kuerzel(app: &mut SheetApp, key: egui::Key) {
    match key {
        egui::Key::Z => app.run_or_alert("edit.undo", json!({})),
        egui::Key::Y => app.run_or_alert("edit.redo", json!({})),
        egui::Key::S => app.run_or_alert("file.save", json!({})),
        _ => {}
    }
}

/// Deutsche Vorgaben der Engine: neue Blätter heißen „Tabelle1“, neue Mappen „Mappe1“.
pub fn deutsche_vorgaben(session: &mut simplegrid_engine::Session) {
    session.prefs.sheet_name_stem = "Tabelle".into();
    session.prefs.book_name_stem = "Mappe".into();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Services;
    use simplegrid_engine::Session;

    fn app() -> SheetApp {
        let mut session = Session::new();
        deutsche_vorgaben(&mut session);
        session.new_workbook();
        SheetApp::new(session, Services::default())
    }

    fn blaetter(app: &SheetApp) -> Vec<String> {
        app.session.active().map(|d| d.wb.sheets.iter().map(|s| s.name.clone()).collect()).unwrap_or_default()
    }

    #[test]
    fn neue_mappe_heisst_deutsch() {
        let a = app();
        assert_eq!(blaetter(&a), ["Tabelle1"]);
        assert_eq!(a.session.active().map(|d| d.display_title()).as_deref(), Some("Mappe1"));
    }

    #[test]
    fn blatt_hinzufuegen_verschieben_umbenennen_und_rueckgaengig() {
        let mut a = app();
        a.run_or_alert("home.insertSheet", json!({"before": 1}));
        assert_eq!(blaetter(&a), ["Tabelle1", "Tabelle2"]);
        a.run_or_alert("sheet.move", json!({"sheet": 1, "to": 0}));
        assert_eq!(blaetter(&a), ["Tabelle2", "Tabelle1"]);
        a.run_or_alert("sheet.rename", json!({"sheet": 0, "name": "Haushalt"}));
        assert_eq!(blaetter(&a), ["Haushalt", "Tabelle1"]);
        for erwartet in [vec!["Tabelle2", "Tabelle1"], vec!["Tabelle1", "Tabelle2"], vec!["Tabelle1"]] {
            kuerzel(&mut a, egui::Key::Z);
            assert_eq!(blaetter(&a), erwartet);
        }
        kuerzel(&mut a, egui::Key::Y);
        assert_eq!(blaetter(&a), ["Tabelle1", "Tabelle2"]);
    }

    #[test]
    fn letztes_blatt_bleibt() {
        let mut a = app();
        a.run_or_alert("home.deleteSheet", json!({"sheet": 0}));
        assert_eq!(blaetter(&a), ["Tabelle1"]);
    }

    #[test]
    fn blatt_loeschen_laesst_sich_rueckgaengig_machen() {
        let mut a = app();
        a.run_or_alert("home.insertSheet", json!({"before": 1}));
        a.run_or_alert("home.deleteSheet", json!({"sheet": 0}));
        assert_eq!(blaetter(&a), ["Tabelle2"]);
        a.run_or_alert("edit.undo", json!({}));
        assert_eq!(blaetter(&a), ["Tabelle1", "Tabelle2"]);
    }

    #[test]
    fn neu_ohne_aenderungen_ersetzt_die_mappe() {
        let mut a = app();
        neu_oder_oeffnen(&mut a, Danach::Neu);
        assert!(a.einfach.frage.is_none());
        assert_eq!(a.session.documents().len(), 1);
    }

    #[test]
    fn neu_mit_aenderungen_fragt_zuerst() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "42"}));
        neu_oder_oeffnen(&mut a, Danach::Neu);
        assert_eq!(a.einfach.frage, Some(Frage::Ungespeichert { danach: Danach::Neu }));
        // Noch nichts verloren: die Mappe mit 42 ist weiter da.
        assert_eq!(a.session.documents().len(), 1);
        assert!(a.session.active().is_some_and(|d| d.is_dirty()));
        // „Nicht speichern“ ersetzt sie durch eine leere.
        ausfuehren(&mut a, Danach::Neu);
        assert_eq!(a.session.documents().len(), 1);
        assert!(a.session.active().is_some_and(|d| !d.is_dirty()));
    }

    #[test]
    fn grosse_entfernte_zelle() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "Z5000", "input": "fern"}));
        let v = a.session.execute("cell.get", json!({"cell": "Z5000"})).unwrap_or_default();
        assert!(v.to_string().contains("fern"), "{v}");
    }
}
