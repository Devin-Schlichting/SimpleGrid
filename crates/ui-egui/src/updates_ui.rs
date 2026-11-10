//! SimpleGrid: Update-Suche in der Oberfläche. Höchstens einmal pro Tag automatisch beim Start
//! (abschaltbar), jederzeit von Hand unter „Info und Einstellungen“. Die Arbeit macht
//! [`crate::update`]; hier laufen nur Hintergrund-Threads, Rückfragen und Meldungen.
//!
//! Ablauf: neue Version → „Es gibt eine neue Version. Jetzt aktualisieren?“ → (ggf. Speichern-
//! Rückfrage) → Installer laden und prüfen → still installieren → Programm startet neu. Ohne
//! Zustimmung wird nie aktualisiert; ohne Internet bleibt die automatische Suche still.

use std::sync::mpsc::{Receiver, channel};

use crate::SheetApp;
use crate::update::{self, Ergebnis, Fehler, GithubQuelle, Release};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub(crate) struct Runtime {
    /// Beim Start prüfen (falls heute noch nicht geschehen).
    startup: bool,
    /// Von Hand prüfen (alle Ergebnisse melden).
    manual: bool,
    /// Laufende Prüfung: (von Hand?, Ergebnis).
    pruefung: Option<(bool, Receiver<Ergebnis>)>,
    /// Laufender Download des Installers.
    download: Option<Receiver<Result<std::path::PathBuf, Fehler>>>,
    /// Die angebotene Version.
    pub(crate) angebot: Option<Release>,
}

/// Prüfung anfordern; der nächste `tick` startet sie.
pub(crate) fn start(app: &mut SheetApp, manual: bool) {
    if manual {
        app.updates_rt.manual = true;
    } else {
        app.updates_rt.startup = true;
    }
}

fn quelle(app: &SheetApp) -> GithubQuelle {
    let repo = if app.ui.update_quelle.trim().is_empty() { update::STANDARD_QUELLE.to_string() } else { app.ui.update_quelle.clone() };
    GithubQuelle { repo }
}

pub(crate) fn tick(app: &mut SheetApp, ctx: &egui::Context) {
    let heute = chrono::Local::now().format("%Y-%m-%d").to_string();
    let manual = std::mem::take(&mut app.updates_rt.manual);
    let automatisch =
        std::mem::take(&mut app.updates_rt.startup) && app.ui.update_automatisch && !update::heute_schon_geprueft(&app.ui.update_zuletzt, &heute);
    if (manual || automatisch) && app.updates_rt.pruefung.is_none() {
        app.ui.update_zuletzt = heute;
        let q = quelle(app);
        let (tx, rx) = channel();
        let ctx2 = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(update::pruefen(&q, VERSION));
            ctx2.request_repaint();
        });
        app.updates_rt.pruefung = Some((manual, rx));
    }
    if let Some((manual, rx)) = &app.updates_rt.pruefung
        && let Ok(ergebnis) = rx.try_recv()
    {
        let manual = *manual;
        app.updates_rt.pruefung = None;
        match ergebnis {
            Ergebnis::Neuer(r) => {
                app.einfach.frage = Some(crate::einfach::Frage::Update { version: r.version.clone() });
                app.updates_rt.angebot = Some(r);
            }
            Ergebnis::Aktuell if manual => {
                app.message = Some(("Updates".into(), format!("Sie haben bereits die neueste Version ({VERSION}).")));
            }
            Ergebnis::Aktuell => {}
            Ergebnis::Fehler(e) => {
                log::info!("Update-Suche: {e:?}");
                if manual {
                    app.message = Some(("Updates".into(), e.text().into()));
                }
            }
        }
    }
    if let Some(rx) = &app.updates_rt.download
        && let Ok(r) = rx.try_recv()
    {
        app.updates_rt.download = None;
        match r {
            Ok(installer) => installieren(app, &installer),
            Err(e) => {
                log::warn!("Update fehlgeschlagen: {e:?}");
                app.message = Some(("Update".into(), e.text().into()));
            }
        }
    }
}

/// „Jetzt aktualisieren“ nach der Speichern-Rückfrage: Installer im Hintergrund laden.
pub(crate) fn herunterladen(app: &mut SheetApp, ctx: &egui::Context) {
    let Some(r) = app.updates_rt.angebot.clone() else { return };
    let q = quelle(app);
    let ordner = std::env::temp_dir().join("SimpleGrid-Update");
    let (tx, rx) = channel();
    let ctx2 = ctx.clone();
    std::thread::spawn(move || {
        let _ = tx.send(update::installer_holen(&q, &r, &ordner));
        ctx2.request_repaint();
    });
    app.updates_rt.download = Some(rx);
    app.toast = Some(("Das Update wird geladen …".into(), crate::now_ms()));
}

/// Startet den geprüften Installer still (er übernimmt die installierten Teile und startet
/// SimpleGrid danach neu) und beendet das Programm.
fn installieren(app: &mut SheetApp, installer: &std::path::Path) {
    let mut cmd = std::process::Command::new(installer);
    cmd.args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/STARTAPP"]);
    match cmd.spawn() {
        Ok(_) => {
            app.allow_close = true;
            app.einfach.jetzt_beenden = true;
        }
        Err(e) => {
            log::warn!("Installer ließ sich nicht starten: {e}");
            app.message = Some(("Update".into(), "Das Update ließ sich nicht starten. Ihre bisherige Version bleibt unverändert.".into()));
        }
    }
}

/// Wird nicht mehr gebraucht (GridCraft's Selbst-Update); bleibt als leerer Einstiegspunkt.
pub(crate) fn start_pending_update(_app: &mut SheetApp) {}

/// Nichts zu zeichnen: Angebot und Meldungen laufen über die einfachen Rückfragen.
pub(crate) fn show(_app: &mut SheetApp, _ctx: &egui::Context) {}
