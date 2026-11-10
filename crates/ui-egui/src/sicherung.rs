//! SimpleGrid: Schutz vor Datenverlust.
//!
//! - Automatische Sicherung alle [`INTERVALL_SEKUNDEN`] in einen eigenen Ordner (nie in die Datei
//!   des Nutzers), solange es ungespeicherte Änderungen gibt.
//! - Nach einem Absturz bietet der nächste Start die Wiederherstellung an.
//! - Wird regulär gespeichert oder beendet, verschwindet die Sicherung wieder.
//!
//! Der Ordner und das Dateiformat sind bewusst einfach, damit die spätere Textverarbeitung das
//! gleiche Verfahren nutzen kann (eigener Unterordner je Programm).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::SheetApp;

/// Abstand zwischen zwei Sicherungen.
pub const INTERVALL_SEKUNDEN: f64 = 150.0;
const DATEI: &str = "sicherung.xlsx";
const INFO: &str = "sicherung.json";

/// Was zu einer Sicherung gehört (neben den Tabellendaten selbst).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Info {
    /// Titel der Tabelle beim Sichern (z. B. „Haushalt.xlsx“ oder „Mappe1“).
    pub titel: String,
    /// Wo die Tabelle gespeichert war (leer bei einer neuen).
    pub pfad: String,
    /// Wann gesichert wurde, als Text für die Anzeige.
    pub zeit: String,
}

/// Der Sicherungsordner: `%LocalAppData%\SimpleGrid\Sicherung`.
pub fn standard_ordner() -> Option<PathBuf> {
    let basis =
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from).or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(basis.join("SimpleGrid").join("Sicherung"))
}

/// Zustand der automatischen Sicherung.
#[derive(Clone, Debug, Default)]
pub struct Sicherung {
    /// Ordner für die Sicherung; `None` schaltet sie ab (z. B. in Tests ohne Ordner).
    pub ordner: Option<PathBuf>,
    /// Zeitpunkt (monotone ms) der letzten Sicherung bzw. der letzten Prüfung.
    letzte: f64,
    /// Die Sicherung im Ordner stammt aus dieser Sitzung (eine aus einem Absturz bleibt liegen,
    /// bis über die Wiederherstellung entschieden ist).
    eigene: bool,
}

impl Sicherung {
    pub fn neu(ordner: Option<PathBuf>) -> Sicherung {
        Sicherung { ordner, letzte: crate::monotonic_ms(), eigene: false }
    }

    /// Ob eine Sicherung im Ordner liegt (z. B. nach einem Absturz).
    pub fn vorhanden(&self) -> Option<Info> {
        let o = self.ordner.as_ref()?;
        if !o.join(DATEI).is_file() {
            return None;
        }
        let info = std::fs::read(o.join(INFO)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        Some(info)
    }

    /// Löscht die Sicherung (nach regulärem Speichern oder Beenden).
    pub fn loeschen(&self) {
        if let Some(o) = &self.ordner {
            let _ = std::fs::remove_file(o.join(DATEI));
            let _ = std::fs::remove_file(o.join(INFO));
        }
    }

    fn pfad(&self) -> Option<PathBuf> {
        self.ordner.as_ref().map(|o| o.join(DATEI))
    }
}

/// Schreibt die aktive Tabelle sofort in den Sicherungsordner. Gibt zurück, ob es geklappt hat.
pub fn jetzt_sichern(app: &mut SheetApp) -> bool {
    let Some(ordner) = app.einfach.sicherung.ordner.clone() else { return false };
    let Some(d) = app.session.active() else { return false };
    let info = Info {
        titel: d.display_title(),
        pfad: d.path.clone().unwrap_or_default(),
        zeit: chrono::Local::now().format("%d.%m.%Y um %H:%M Uhr").to_string(),
    };
    let Ok(r) = app.session.run("file.saveBytes", json!({"format": "xlsx"})) else { return false };
    let Some(bytes) = r.get("base64").and_then(serde_json::Value::as_str).and_then(simplegrid_engine::io::base64_decode) else { return false };
    if std::fs::create_dir_all(&ordner).is_err() {
        return false;
    }
    // Erst in eine Hilfsdatei schreiben, dann umbenennen: eine halbe Sicherung gibt es nie.
    let neu = ordner.join("sicherung.tmp");
    let ok = std::fs::write(&neu, &bytes).is_ok() && std::fs::rename(&neu, ordner.join(DATEI)).is_ok();
    if ok && let Ok(b) = serde_json::to_vec_pretty(&info) {
        let _ = std::fs::write(ordner.join(INFO), b);
        app.einfach.sicherung.eigene = true;
    }
    if !ok {
        log::warn!("Automatische Sicherung nach {} fehlgeschlagen", ordner.display());
    }
    ok
}

/// Einmal pro Bild aufgerufen: sichert alle [`INTERVALL_SEKUNDEN`], solange es ungespeicherte
/// Änderungen gibt, und räumt die Sicherung weg, sobald alles gespeichert ist.
pub fn takt(app: &mut SheetApp, ctx: &egui::Context) {
    if app.einfach.sicherung.ordner.is_none() || app.einfach.frage.is_some() {
        return;
    }
    let jetzt = crate::monotonic_ms();
    let dirty = app.session.active().is_some_and(|d| d.is_dirty());
    if !dirty {
        // Gespeichert oder nichts geändert: keine Sicherung nötig.
        if app.einfach.sicherung.eigene {
            app.einfach.sicherung.loeschen();
            app.einfach.sicherung.eigene = false;
        }
        app.einfach.sicherung.letzte = jetzt;
        return;
    }
    if jetzt - app.einfach.sicherung.letzte >= INTERVALL_SEKUNDEN * 1000.0 && app.editor.is_none() {
        app.einfach.sicherung.letzte = jetzt;
        jetzt_sichern(app);
    } else {
        ctx.request_repaint_after(std::time::Duration::from_secs(10));
    }
}

/// Öffnet die Sicherung als Tabelle (anstelle der aktuellen). Sie bleibt ungespeichert, damit
/// „Speichern“ nach Ort und Namen fragt.
pub fn wiederherstellen(app: &mut SheetApp) -> bool {
    let Some(pfad) = app.einfach.sicherung.pfad() else { return false };
    let Some(info) = app.einfach.sicherung.vorhanden() else { return false };
    let Ok(bytes) = std::fs::read(&pfad) else { return false };
    let alte: Vec<u64> = app.session.documents().iter().map(|d| d.uid).collect();
    let name = if info.titel.to_ascii_lowercase().ends_with(".xlsx") { info.titel.clone() } else { format!("{}.xlsx", info.titel) };
    let ok = app.run("file.open", json!({"name": name, "base64": simplegrid_engine::io::base64_encode(&bytes)})).is_ok();
    app.after_engine();
    if !ok {
        return false;
    }
    let mut i = app.session.documents().len();
    while i > 0 {
        i -= 1;
        if app.session.documents().get(i).is_some_and(|d| alte.contains(&d.uid)) {
            app.session.close_document(i);
        }
    }
    app.follow_active_window();
    // Als geändert markieren, damit nichts verloren geht, bevor gespeichert wurde.
    if let Some(d) = app.session.active_mut() {
        d.mark_dirty();
    }
    app.message = None;
    true
}

/// Nach einem unerwarteten Fehler: sofort sichern und freundlich Bescheid geben.
pub fn nach_fehler(app: &mut SheetApp) {
    let gesichert = jetzt_sichern(app);
    let text = if gesichert {
        "Es ist ein unerwarteter Fehler aufgetreten. Ihre Tabelle wurde vorsorglich gesichert. Sie können weiterarbeiten – speichern Sie am besten gleich."
    } else {
        "Es ist ein unerwarteter Fehler aufgetreten. Sie können weiterarbeiten – speichern Sie am besten gleich."
    };
    app.message = Some(("Hinweis".into(), text.into()));
}

/// Für Tests: ein frischer, eigener Ordner.
pub fn test_ordner(name: &str) -> PathBuf {
    let o = std::env::temp_dir().join(format!("simplegrid-sicherung-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&o);
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Services;
    use crate::einfach::{Danach, Frage};
    use simplegrid_engine::Session;

    fn app(ordner: &str) -> SheetApp {
        let mut session = Session::new();
        crate::einfach::deutsche_vorgaben(&mut session);
        session.new_workbook();
        let mut a = SheetApp::new(session, Services::default());
        a.einfach.sicherung = Sicherung::neu(Some(test_ordner(ordner)));
        a
    }

    fn wert(a: &mut SheetApp, zelle: &str) -> serde_json::Value {
        a.session.execute("cell.get", json!({"cell": zelle})).map(|v| v["value"].clone()).unwrap_or_default()
    }

    #[test]
    fn sicherung_und_wiederherstellung_nach_absturz() {
        let mut a = app("absturz");
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "nicht verlieren"}));
        assert!(jetzt_sichern(&mut a));
        let info = a.einfach.sicherung.vorhanden().unwrap_or_default();
        assert_eq!(info.titel, "Mappe1");
        // „Absturz“: neues Programm mit demselben Ordner, die Sicherung liegt noch da.
        let ordner = a.einfach.sicherung.ordner.clone();
        drop(a);
        let mut session = Session::new();
        crate::einfach::deutsche_vorgaben(&mut session);
        session.new_workbook();
        let mut b = SheetApp::new(session, Services::default());
        b.einfach.sicherung = Sicherung::neu(ordner);
        crate::einfach::beim_start(&mut b);
        assert!(matches!(b.einfach.frage, Some(Frage::Wiederherstellen { .. })));
        assert!(wiederherstellen(&mut b));
        assert_eq!(wert(&mut b, "A1"), json!("nicht verlieren"));
        assert!(b.session.active().is_some_and(|d| d.is_dirty()), "wiederhergestellt heißt: noch nicht gespeichert");
        assert_eq!(b.session.documents().len(), 1);
    }

    #[test]
    fn takt_sichert_nur_bei_aenderungen_und_raeumt_nach_dem_speichern_auf() {
        let mut a = app("takt");
        let ctx = egui::Context::default();
        a.einfach.sicherung.letzte = -1.0e12;
        takt(&mut a, &ctx);
        assert!(a.einfach.sicherung.vorhanden().is_none(), "ohne Änderung keine Sicherung");
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "1"}));
        a.einfach.sicherung.letzte = -1.0e12;
        takt(&mut a, &ctx);
        assert!(a.einfach.sicherung.vorhanden().is_some(), "Änderung wird gesichert");
        let pfad = test_ordner("takt-datei").with_extension("xlsx");
        a.run_or_alert("file.save", json!({"path": pfad.to_string_lossy()}));
        takt(&mut a, &ctx);
        assert!(a.einfach.sicherung.vorhanden().is_none(), "nach dem Speichern weg");
    }

    #[test]
    fn schliessen_mit_aenderungen_fragt_zuerst() {
        let mut a = app("schliessen");
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "1"}));
        crate::einfach::neu_oder_oeffnen(&mut a, Danach::Beenden);
        assert_eq!(a.einfach.frage, Some(Frage::Ungespeichert { danach: Danach::Beenden }));
        assert!(!a.allow_close);
    }

    #[test]
    fn schliessen_ohne_aenderungen_ohne_frage() {
        let mut a = app("ohne");
        crate::einfach::neu_oder_oeffnen(&mut a, Danach::Beenden);
        assert!(a.einfach.frage.is_none());
        assert!(a.allow_close && a.einfach.jetzt_beenden);
    }

    #[test]
    fn nach_fehler_wird_gesichert_und_freundlich_gemeldet() {
        let mut a = app("fehler");
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "x"}));
        nach_fehler(&mut a);
        assert!(a.einfach.sicherung.vorhanden().is_some());
        assert!(a.message.as_ref().is_some_and(|(_, t)| t.contains("gesichert")));
    }

    #[test]
    fn letzte_datei_wird_beim_start_geoeffnet_und_ist_abschaltbar() {
        let pfad = test_ordner("zuletzt").with_extension("xlsx").to_string_lossy().into_owned();
        let mut a = app("zuletzt-a");
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "zuletzt"}));
        a.run_or_alert("file.save", json!({"path": pfad}));
        crate::einfach::zuletzt_merken(&mut a);
        let recent = a.ui.recent.clone();
        let mut b = app("zuletzt-b");
        b.ui.recent = recent.clone();
        crate::einfach::beim_start(&mut b);
        assert_eq!(wert(&mut b, "A1"), json!("zuletzt"));
        let mut c = app("zuletzt-c");
        c.ui.recent = recent;
        c.ui.letzte_datei_oeffnen = false;
        crate::einfach::beim_start(&mut c);
        assert_eq!(wert(&mut c, "A1"), serde_json::Value::Null);
    }

    #[test]
    fn auch_zeile_mit_daten_loeschen_ist_rueckgaengig_machbar() {
        let mut a = app("zeile");
        a.run_or_alert("cell.set", json!({"cell": "A2", "input": "wichtig"}));
        a.run_or_alert("selection.set", json!({"range": "A2"}));
        a.run_or_alert("home.deleteRows", json!({}));
        assert_eq!(wert(&mut a, "A2"), serde_json::Value::Null);
        a.run_or_alert("edit.undo", json!({}));
        assert_eq!(wert(&mut a, "A2"), json!("wichtig"));
    }
}
