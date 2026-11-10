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
    pub const HINWEIS_TITEL: &str = "Hinweis";
    pub const OEFFNEN_TITEL: &str = "Datei lässt sich nicht öffnen";
    pub const OEFFNEN_FEHLER: &str = "Die Datei „{}“ konnte nicht geöffnet werden. Vielleicht ist sie beschädigt, oder es ist keine Tabelle im Format .xlsx oder .ods.";
    pub const IMPORT_HINWEIS: &str = "Nicht alles aus dieser Datei kann angezeigt werden (zum Beispiel Diagramme, Bilder oder Filter). Was SimpleGrid nicht kennt, wurde weggelassen.";
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
    if knopf(ui, "Öffnen", "Eine gespeicherte Tabelle öffnen (.xlsx oder .ods)", None, false) {
        neu_oder_oeffnen(app, Danach::Oeffnen);
    }
    let ohne_mappe = app.session.active().is_none().then_some(texte::NICHT_MOEGLICH);
    if knopf(ui, "Speichern", "Die Tabelle speichern", ohne_mappe, false) {
        speichern(app);
    }
    if knopf(ui, "Speichern unter", "Die Tabelle unter einem neuen Namen oder an einem anderen Ort speichern", ohne_mappe, false) {
        speichern_unter(app);
    }
}

/// Ob die aktive Mappe als .xlsx-Datei gespeichert ist (dann speichert „Speichern“ direkt dorthin).
fn ist_xlsx(app: &SheetApp) -> bool {
    app.session.active().and_then(|d| d.path.as_deref()).is_some_and(|p| p.to_ascii_lowercase().ends_with(".xlsx"))
}

/// „Speichern“: in die vorhandene .xlsx-Datei; eine neue oder aus .ods übernommene Tabelle fragt
/// nach Namen und Ort (gespeichert wird immer als .xlsx). Gibt zurück, ob jetzt alles gespeichert ist.
pub fn speichern(app: &mut SheetApp) -> bool {
    if !app.commit_edit(0, 0, false, false) {
        return false;
    }
    if ist_xlsx(app) {
        app.run_or_alert("file.save", json!({}));
    } else {
        speichern_unter(app);
    }
    app.session.active().is_some_and(|d| !d.is_dirty())
}

/// „Speichern unter“: fragt nach Namen und Ort, gespeichert wird als .xlsx.
pub fn speichern_unter(app: &mut SheetApp) {
    if app.commit_edit(0, 0, false, false) {
        app.open_dialog("saveAs", json!({}));
    }
}

/// „Öffnen“: Datei auswählen und anstelle der bisherigen Tabelle anzeigen. Wird keine Datei
/// gewählt oder lässt sie sich nicht lesen, bleibt die bisherige Tabelle unverändert.
fn oeffnen(app: &mut SheetApp) {
    let Some(pfad) = app.services.pick_open.as_ref().and_then(|pick| pick()) else { return };
    oeffnen_pfad(app, &pfad);
}

/// Ob `pfad` eine .xlsx- oder .ods-Datei ist, die auch wirklich wie eine aussieht (ein ZIP-Paket).
/// Andere Dateien liest SimpleGrid gar nicht erst, statt sie als Text zu deuten.
fn ist_tabellendatei(pfad: &str) -> bool {
    let endung = std::path::Path::new(pfad).extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase);
    if !matches!(endung.as_deref(), Some("xlsx" | "ods")) {
        return false;
    }
    let mut kopf = [0u8; 4];
    std::fs::File::open(pfad).and_then(|mut f| std::io::Read::read_exact(&mut f, &mut kopf)).is_ok() && kopf == [b'P', b'K', 3, 4]
}

/// Öffnet `pfad` anstelle der bisherigen Tabelle; verständliche Meldung, wenn das nicht geht.
pub fn oeffnen_pfad(app: &mut SheetApp, pfad: &str) {
    let name = std::path::Path::new(pfad).file_name().and_then(|n| n.to_str()).unwrap_or(pfad).to_string();
    let fehler = |app: &mut SheetApp| app.message = Some((texte::OEFFNEN_TITEL.into(), texte::OEFFNEN_FEHLER.replace("{}", &name)));
    if !ist_tabellendatei(pfad) {
        fehler(app);
        return;
    }
    let alte: Vec<u64> = app.session.documents().iter().map(|d| d.uid).collect();
    app.message = None;
    app.open_path(pfad);
    let neu = app.session.active().map(|d| d.uid);
    if neu.is_none_or(|u| alte.contains(&u)) {
        // Nicht geöffnet (oder Passwortabfrage): die englische Fehlermeldung durch eine deutsche ersetzen.
        if app.message.take().is_some() {
            fehler(app);
        }
        return;
    }
    // Die bisherige Tabelle schließen: SimpleGrid zeigt immer genau eine.
    let mut i = app.session.documents().len();
    while i > 0 {
        i -= 1;
        if app.session.documents().get(i).is_some_and(|d| alte.contains(&d.uid)) {
            app.session.close_document(i);
        }
    }
    app.follow_active_window();
    // Hinweise beim Einlesen (z. B. Diagramme aus einer .ods-Datei) auf Deutsch zusammenfassen:
    // die Engine meldet sie auf Englisch als Meldungsfenster und als Hinweisleiste.
    if let Some((_, text)) = app.message.take() {
        log::info!("Hinweis beim Öffnen von {pfad}: {text}");
        app.message = Some((texte::HINWEIS_TITEL.into(), texte::IMPORT_HINWEIS.into()));
    }
    if let Some(n) = app.notice.as_mut() {
        log::info!("Hinweis beim Öffnen von {pfad}: {}", n.text);
        n.text = texte::IMPORT_HINWEIS.into();
        n.action = None;
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
        Danach::Oeffnen => oeffnen(app),
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
            // Nur weiter, wenn wirklich gespeichert wurde (sonst bleibt alles, wie es war).
            texte::SPEICHERN => {
                if speichern(app) {
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
    // Deutsche Zahlen für diesen Thread; das Programm schaltet sie in `main` für alle Threads ein.
    simplegrid_engine::core::locale::set_german_for_thread(true);
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

    fn temp(name: &str) -> String {
        let dir = std::env::temp_dir().join(format!("simplegrid-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        dir.join(name).to_string_lossy().into_owned()
    }

    fn wert(a: &mut SheetApp, zelle: &str) -> Json {
        a.session.execute("cell.get", json!({"cell": zelle})).map(|v| v["value"].clone()).unwrap_or(Json::Null)
    }

    #[test]
    fn speichern_und_oeffnen_ersetzt_die_mappe() {
        let pfad = temp("rund.xlsx");
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "B2", "input": "12,5"}));
        a.run_or_alert("home.insertSheet", json!({"before": 1}));
        a.run_or_alert("file.save", json!({"path": pfad}));
        assert!(ist_xlsx(&a));
        assert!(a.session.active().is_some_and(|d| !d.is_dirty()));
        // In einer frischen Mappe öffnen: genau eine Mappe, mit Inhalt und beiden Blättern.
        let mut b = app();
        oeffnen_pfad(&mut b, &pfad);
        assert!(b.message.is_none(), "{:?}", b.message);
        assert_eq!(b.session.documents().len(), 1);
        assert_eq!(blaetter(&b), ["Tabelle1", "Tabelle2"]);
        let _ = b.session.execute("sheet.activate", json!({"sheet": 0}));
        assert_eq!(wert(&mut b, "B2"), json!(12.5));
    }

    #[test]
    fn kaputte_datei_gibt_deutsche_meldung_und_behaelt_die_tabelle() {
        let pfad = temp("kaputt.xlsx");
        let _ = std::fs::write(&pfad, b"das ist keine Tabelle");
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "bleibt"}));
        oeffnen_pfad(&mut a, &pfad);
        let (titel, text) = a.message.clone().unwrap_or_default();
        assert_eq!(titel, texte::OEFFNEN_TITEL);
        assert!(text.contains("kaputt.xlsx"), "{text}");
        assert_eq!(a.session.documents().len(), 1);
        assert_eq!(wert(&mut a, "A1"), json!("bleibt"));
    }

    #[test]
    fn ods_wird_als_neue_tabelle_importiert_und_nicht_als_ods_gespeichert() {
        let pfad = temp("import.ods");
        let _ = std::fs::write(&pfad, ods_beispiel(""));
        let mut a = app();
        oeffnen_pfad(&mut a, &pfad);
        assert!(a.message.is_none(), "{:?}", a.message);
        assert_eq!(wert(&mut a, "A1"), json!(3.0));
        assert_eq!(wert(&mut a, "A2"), json!(4.0));
        assert_eq!(wert(&mut a, "A3"), json!(7.0));
        // „Speichern“ würde nach einem .xlsx-Namen fragen, nicht die .ods-Datei überschreiben.
        assert!(!ist_xlsx(&a));
    }

    /// Eine kleine .ods-Datei: 3, 4 und =SUMME darunter.
    fn ods_beispiel(extra: &str) -> Vec<u8> {
        use std::io::Write;
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:of="urn:oasis:names:tc:opendocument:xmlns:of:1.2" office:version="1.2"><office:body><office:spreadsheet><table:table table:name="Haushalt"><table:table-row><table:table-cell office:value-type="float" office:value="3"><text:p>3</text:p></table:table-cell></table:table-row><table:table-row><table:table-cell office:value-type="float" office:value="4"><text:p>4</text:p></table:table-cell></table:table-row><table:table-row><table:table-cell table:formula="of:=SUM([.A1:.A2])" office:value-type="float" office:value="7"><text:p>7</text:p></table:table-cell>EXTRA</table:table-row></table:table></office:spreadsheet></office:body></office:document-content>"#;
        let content = content.replace("EXTRA", extra);
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            let stored = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            let _ = z.start_file("mimetype", stored);
            let _ = z.write_all(b"application/vnd.oasis.opendocument.spreadsheet");
            let _ = z.start_file("content.xml", stored);
            let _ = z.write_all(content.as_bytes());
            let _ = z.finish();
        }
        buf.into_inner()
    }

    #[test]
    fn import_hinweis_ist_deutsch() {
        let pfad = temp("diagramm.ods");
        let extra = r#"<table:table-cell><draw:frame xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0"><draw:object/></draw:frame></table:table-cell>"#;
        let _ = std::fs::write(&pfad, ods_beispiel(extra));
        let mut a = app();
        oeffnen_pfad(&mut a, &pfad);
        assert_eq!(a.message, Some((texte::HINWEIS_TITEL.to_string(), texte::IMPORT_HINWEIS.to_string())));
        assert_eq!(wert(&mut a, "A3"), json!(7.0));
    }

    #[test]
    fn deutsche_zahlen_und_euro() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "1.234,5"}));
        a.run_or_alert("cell.set", json!({"cell": "A2", "input": "12,5 €"}));
        assert_eq!(wert(&mut a, "A1"), json!(1234.5));
        assert_eq!(wert(&mut a, "A2"), json!(12.5));
        let text = |a: &mut SheetApp, c: &str| a.session.execute("cell.get", json!({"cell": c})).map(|v| v["text"].clone()).unwrap_or(Json::Null);
        assert_eq!(text(&mut a, "A1"), json!("1.234,50"));
        assert_eq!(text(&mut a, "A2"), json!("12,50 €"));
    }

    #[test]
    fn grosse_entfernte_zelle() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "Z5000", "input": "fern"}));
        let v = a.session.execute("cell.get", json!({"cell": "Z5000"})).unwrap_or_default();
        assert!(v.to_string().contains("fern"), "{v}");
    }
}
