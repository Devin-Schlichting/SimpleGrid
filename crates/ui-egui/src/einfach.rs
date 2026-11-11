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

/// Das kleine Ausfüllkästchen an der Markierung (Ziehen füllt Zellen weiter). Abgeschaltet: zu klein
/// und leicht aus Versehen gezogen; sein englisches Menü entfällt damit auch.
pub const AUSFUELLKAESTCHEN: bool = false;
/// GridCraft's Hintergrund-Fehlerprüfung (grüne Ecken bei „Zahl als Text“ mit englischem Menü). Abgeschaltet.
pub const FEHLERPRUEFUNG: bool = false;

/// Zentrale Maße der einfachen Oberfläche. Eine Änderung hier wirkt auf alle Schaltflächen.
pub mod masse {
    /// Mindesthöhe einer Schaltfläche in Punkten.
    pub const KNOPF_HOEHE: f32 = 52.0;
    /// Mindestbreite einer Schaltfläche in Punkten.
    pub const KNOPF_BREITE: f32 = 84.0;
    /// Schriftgröße der Beschriftungen.
    pub const SCHRIFT: f32 = 17.0;
    /// Schriftgröße der Gruppenüberschriften.
    pub const GRUPPEN_SCHRIFT: f32 = 12.5;
    /// Abstand zwischen zwei Schaltflächen.
    pub const ABSTAND: f32 = 8.0;
    /// Kacheln (Symbol über Beschriftung) in den Registern.
    pub const KACHEL_BREITE: f32 = 92.0;
    pub const KACHEL_HOEHE: f32 = 88.0;
    /// Größe der Symbole auf den Kacheln.
    pub const SYMBOL: f32 = 30.0;
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
    pub const BLATT_LOESCHEN_TEXT: &str =
        "Soll das Blatt „{}“ mit allem, was darauf steht, gelöscht werden? Mit „Rückgängig“ können Sie es zurückholen.";
    pub const LETZTES_BLATT: &str = "Das letzte Blatt kann nicht gelöscht werden.";
    pub const HINWEIS_TITEL: &str = "Hinweis";
    pub const OEFFNEN_TITEL: &str = "Datei lässt sich nicht öffnen";
    pub const OEFFNEN_FEHLER: &str =
        "Die Datei „{}“ konnte nicht geöffnet werden. Vielleicht ist sie beschädigt, oder es ist keine Tabelle im Format .xlsx oder .ods.";
    pub const IMPORT_HINWEIS: &str = "Nicht alles aus dieser Datei kann angezeigt werden (zum Beispiel Diagramme, Bilder oder Filter). Was SimpleGrid nicht kennt, wurde weggelassen.";
}

/// Was nach einer Rückfrage zu ungespeicherten Änderungen passieren soll.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Danach {
    /// Eine neue, leere Tabelle anlegen.
    Neu,
    /// Eine Datei öffnen.
    Oeffnen,
    /// Das Programm beenden.
    Beenden,
    /// Das angebotene Update laden und installieren.
    Aktualisieren,
}

/// Eine offene Rückfrage der einfachen Oberfläche.
#[derive(Clone, Debug, PartialEq)]
pub enum Frage {
    /// Ungespeicherte Änderungen vor „Neu“ oder „Öffnen“.
    Ungespeichert { danach: Danach },
    /// Ein Tabellenblatt wirklich löschen?
    BlattLoeschen { blatt: usize, name: String },
    /// Eine Aktion, die Daten entfernt, erst nach Bestätigung ausführen.
    Bestaetigen { titel: String, text: String, knopf: String, befehl: String, params: Json },
    /// Nach einem Absturz: die Sicherung wiederherstellen?
    Wiederherstellen { titel: String, zeit: String },
    /// Eine neue Version ist da: jetzt aktualisieren?
    Update { version: String },
    /// Ein Tabellenblatt umbenennen (mit Eingabefeld).
    Umbenennen { blatt: usize, name: String },
}

/// Zustand der einfachen Oberfläche.
#[derive(Clone, Debug, Default)]
pub struct Zustand {
    pub frage: Option<Frage>,
    /// Gewählte Dicke der Umrandung (`thin`, `medium`, `thick`); leer heißt dünn.
    pub rahmen_dicke: String,
    /// Gewählte Farbe der Umrandung (`#RRGGBB`); leer heißt schwarz.
    pub rahmen_farbe: String,
    /// Die offene Seitenansicht (Druckvorschau), falls gezeigt.
    pub seitenansicht: Option<crate::seitenansicht::Seitenansicht>,
    /// Automatische Sicherung (Ordner wird vom Programm gesetzt; ohne Ordner aus).
    pub sicherung: crate::sicherung::Sicherung,
    /// Nach „Speichern“/„Nicht speichern“ beim Schließen: das Fenster jetzt schließen.
    pub jetzt_beenden: bool,
    /// Das Fenster „Info und Einstellungen“ ist offen.
    pub info_offen: bool,
    /// Nach der Speichern-Rückfrage: das Update jetzt laden.
    pub update_laden: bool,
    /// Das gewählte Register der Leiste.
    pub register: Register,
    /// Zelle, in der eine Summe zum Bestätigen steht (grüner Haken / rotes Kreuz daneben).
    pub summe_ziel: Option<simplegrid_engine::core::CellRef>,
}

/// Schriftgrößen zur Auswahl (feste Stufen).
pub const SCHRIFTGROESSEN: [f32; 10] = [10.0, 11.0, 12.0, 14.0, 16.0, 18.0, 20.0, 24.0, 28.0, 36.0];
/// Zoomstufen für „Größer“ und „Kleiner“ (Prozent).
pub const ZOOMSTUFEN: [u32; 9] = [50, 75, 100, 125, 150, 175, 200, 250, 300];
/// Dicken der Umrandung: (Engine-Name, Anzeige).
pub const RAHMEN_DICKEN: [(&str, &str); 3] = [("thin", "Dünn"), ("medium", "Mittel"), ("thick", "Dick")];

/// Deutsche Texte für übernommene GridCraft-Dialoge, die die allgemeine Übersetzung nicht kennt
/// oder anders (einfacher) heißen sollen. Wird vor `i18n::de` gefragt.
pub fn uebersetzung(text: &str) -> Option<&'static str> {
    Some(match text {
        "Column Width" => "Spaltenbreite",
        "Column width (characters):" => "Spaltenbreite (Zeichen):",
        "Row Height" => "Zeilenhöhe",
        "Row height (points):" => "Zeilenhöhe (Punkt):",
        "Cancel" => "Abbrechen",
        "OK" => "OK",
        "Save" => "Speichern",
        "Don't Save" => "Nicht speichern",
        "Close" => "Schließen",
        "Ready" => "Bereit",
        "Theme Colors" => "Farben",
        // Häufige Meldungen der Engine (Nr. 14).
        "The workbook is protected and cannot be changed." => "Die Tabelle ist geschützt und kann nicht geändert werden.",
        "A workbook must contain at least one visible worksheet." => "Das letzte Blatt kann nicht gelöscht werden.",
        "A sheet name can't be blank." => "Der Blattname darf nicht leer sein.",
        "A sheet name can't be longer than 31 characters." => "Der Blattname darf höchstens 31 Zeichen lang sein.",
        "A sheet name can't contain any of these characters: : \\ / ? * [ ]" => "Der Blattname darf diese Zeichen nicht enthalten: : \\ / ? * [ ]",
        "A sheet name can't begin or end with an apostrophe." => "Der Blattname darf nicht mit einem Apostroph beginnen oder enden.",
        "That name is already taken. Try a different one." => "Diesen Namen gibt es schon. Bitte einen anderen wählen.",
        "Reference isn't valid." => "Der Bezug ist ungültig.",
        "Nothing to paste." => "Es wurde noch nichts kopiert.",
        "This value doesn't match the data validation restrictions defined for this cell." => "Dieser Wert ist in dieser Zelle nicht erlaubt.",
        "This action won't work on multiple selections." => "Das geht nur, wenn ein einziger Bereich markiert ist.",
        "The cell or chart you're trying to change is on a protected sheet." => "Die Zelle ist auf einem geschützten Blatt und kann nicht geändert werden.",
        "To prevent possible loss of data, SimpleGrid cannot shift nonblank cells off of the worksheet." => "Das geht nicht: Dabei würden Zellen mit Inhalt über den Rand der Tabelle geschoben.",
        "To prevent possible loss of data, columns cannot be shifted off of the worksheet." => "Das geht nicht: Dabei würden Spalten mit Inhalt über den Rand der Tabelle geschoben.",
        "The copied cells don't fit on the sheet at this position." => "Das Kopierte passt an dieser Stelle nicht auf das Blatt.",
        "The copied cells don't fit here: paste them where the whole copied area fits on the sheet." => "Das Kopierte passt hier nicht hin. Bitte weiter oben oder links einfügen.",
        "The cut rows or columns can't be inserted inside themselves." => "Ausgeschnittene Zeilen oder Spalten können nicht in sich selbst eingefügt werden.",
        "The number must be between 1 and 409." => "Die Zahl muss zwischen 1 und 409 liegen.",
        "Select an available printer in File > Print before submitting." => "Bitte in der Seitenansicht zuerst einen Drucker wählen.",
        "Printing is unavailable here; export a PDF instead." => "Drucken geht hier nicht. Bitte „Als PDF speichern“ benutzen.",
        "Standard Colors" => "Kräftige Farben",
        "Recent Colors" => "Zuletzt benutzt",
        "Printer" => "Drucker",
        "Select a printer" => "Drucker wählen",
        "Refresh printers" => "Drucker neu suchen",
        "Copies" => "Anzahl",
        "Two-sided printing" => "Beidseitig drucken",
        "Printer default" => "Wie am Drucker eingestellt",
        "One-sided" => "Einseitig",
        "Two-sided: long edge" => "Beidseitig (lange Kante)",
        "Two-sided: short edge" => "Beidseitig (kurze Kante)",
        "Grayscale" => "Graustufen",
        "Finding printers…" => "Drucker werden gesucht …",
        "No printers found. Configure a system printer or export a PDF." => {
            "Kein Drucker gefunden. Bitte einen Drucker in Windows einrichten oder als PDF speichern."
        }
        "Printer discovery failed." => "Die Druckersuche hat nicht geklappt.",
        "Submitting print job…" => "Wird an den Drucker geschickt …",
        "Print job submitted." => "Wurde an den Drucker geschickt.",
        "Print job submitted: {receipt}" => "Wurde an den Drucker geschickt: {receipt}",
        "Print failed: {error}" => "Drucken hat nicht geklappt: {error}",
        "Print failed: the printing worker stopped." => "Drucken hat nicht geklappt.",
        _ => return None,
    })
}

/// Der Grund, warum Befehl `id` gerade nicht geht, oder `None`, wenn er geht.
pub fn gesperrt(app: &SheetApp, id: &str) -> Option<String> {
    let spec = simplegrid_engine::find_command(id)?;
    (spec.enabled)(&app.session).err()
}

/// Das Symbol über der Beschriftung einer Kachel: GridCraft's gezeichnete Icons (keine
/// Schriftzeichen, die einer Schrift fehlen könnten). Zentral, damit alle Knöpfe gleich aussehen.
fn symbol(text: &str) -> Option<crate::icons::Icon> {
    use crate::icons::Icon;
    Some(match text {
        "Neu" => Icon::NewFile,
        "Öffnen" => Icon::OpenFolder,
        "Speichern" | "Speichern unter" => Icon::Save,
        "Rückgängig" => Icon::Undo,
        "Wiederholen" => Icon::Redo,
        "Kopieren" => Icon::Copy,
        "Ausschneiden" => Icon::Cut,
        "Einfügen" => Icon::Paste,
        "Inhalt löschen" => Icon::Eraser,
        "Σ Summe" | "Summe" => Icon::Sum,
        "Fett" => Icon::Bold,
        "Kursiv" => Icon::Italic,
        "Unterstrichen" => Icon::Underline,
        "Links" => Icon::AlignLeft,
        "Mitte" => Icon::AlignCenter,
        "Rechts" => Icon::AlignRight,
        "Euro €" => Icon::Currency,
        "Zellen verbinden" | "Verbindung trennen" => Icon::Merge,
        "Außen" => Icon::BorderOutside,
        "Innen" => Icon::Borders,
        "Alle" => Icon::BorderAll,
        "Keine" => Icon::BorderNone,
        "Zeile hinzufügen" | "Spalte hinzufügen" => Icon::Insert,
        "Zeile entfernen" | "Spalte entfernen" => Icon::Delete,
        "Spaltenbreite …" | "Zeilenhöhe …" => Icon::Format,
        "Seitenansicht und Drucken" => Icon::Print,
        "Druckbereich festlegen" | "Druckbereich aufheben" => Icon::PrintArea,
        "Info und Einstellungen" => Icon::Info,
        "Nach Updates suchen" => Icon::Search,
        "Textfarbe" => Icon::FontColor,
        "Hintergrund" => Icon::Fill,
        "Linienfarbe" => Icon::Borders,
        _ => return None,
    })
}

/// Eine große, beschriftete Schaltfläche (Kachel: Symbol über der Beschriftung, sonst nur Text).
/// `aus` sperrt sie mit Begründung; `an` zeigt sie eingedrückt (z. B. „Fett“, wenn die Zelle fett ist).
fn knopf(ui: &mut Ui, text: &str, tipp: &str, aus: Option<&str>, an: bool) -> bool {
    let t = Tokens::get(ui.ctx());
    let farbe = if aus.is_some() {
        t.text_disabled
    } else if an {
        t.accent_dark
    } else {
        t.text
    };
    if let Some(sym) = symbol(text) {
        return kachel(ui, sym, text.trim_start_matches("Σ "), tipp, aus, an);
    }
    let b = egui::Button::new(RichText::new(text).font(theme::ui_bold(masse::SCHRIFT)).color(farbe))
        .selected(an)
        .min_size(vec2(masse::KNOPF_BREITE, masse::KNOPF_HOEHE))
        .corner_radius(10.0);
    let r = ui.add_enabled(aus.is_none(), b);
    let r = match aus {
        Some(grund) => r.on_disabled_hover_text(format!("{tipp}\n\n{grund}")),
        None => r.on_hover_text(tipp),
    };
    r.clicked()
}

/// Eine Kachel: großes Symbol, darunter die Beschriftung, beides mittig; so breit wie nötig.
fn kachel(ui: &mut Ui, sym: crate::icons::Icon, text: &str, tipp: &str, aus: Option<&str>, an: bool) -> bool {
    let r = kachel_antwort(ui, sym, text, tipp, aus, an);
    aus.is_none() && r.clicked()
}

/// Wie [`kachel`], gibt aber die egui-Antwort zurück (z. B. um ein Auswahlfenster daran zu hängen).
fn kachel_antwort(ui: &mut Ui, sym: crate::icons::Icon, text: &str, tipp: &str, aus: Option<&str>, an: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let schrift = theme::ui_bold(masse::SCHRIFT);
    let farbe = if aus.is_some() {
        t.text_disabled
    } else if an {
        t.accent_dark
    } else {
        t.text
    };
    let symbol_farbe = if aus.is_some() { t.text_disabled } else { t.accent };
    let galley = ui.painter().layout_no_wrap(text.to_string(), schrift.clone(), farbe);
    let breite = (galley.size().x + 20.0).max(masse::KACHEL_BREITE);
    let sense = if aus.is_some() { egui::Sense::hover() } else { egui::Sense::click() };
    // In sehr niedrigen Fenstern: kompakte Kacheln ohne Symbol, damit die Tabelle sichtbar bleibt.
    let kompakt = ui.ctx().content_rect().height() < 640.0;
    let hoehe = if kompakt { masse::KNOPF_HOEHE } else { masse::KACHEL_HOEHE };
    let (rect, r) = ui.allocate_exact_size(vec2(breite, hoehe), sense);
    let (fill, rand) = if an {
        (t.accent_soft, t.accent)
    } else if aus.is_none() && r.is_pointer_button_down_on() {
        (t.pressed, t.accent)
    } else if aus.is_none() && r.hovered() {
        (t.accent_soft, t.accent)
    } else {
        (egui::Color32::WHITE, t.ribbon_border)
    };
    let p = ui.painter();
    p.rect(rect, 10.0, fill, egui::Stroke::new(if an || r.hovered() { 2.5 } else { 1.5 }, rand), egui::StrokeKind::Inside);
    if r.has_focus() {
        p.rect_stroke(rect.expand(3.0), 12.0, egui::Stroke::new(3.0, egui::Color32::from_rgb(0xF5, 0x9E, 0x0B)), egui::StrokeKind::Outside);
    }
    if kompakt {
        p.galley(rect.center() - galley.size() / 2.0, galley, farbe);
    } else {
        crate::icons::paint(
            p,
            egui::Rect::from_center_size(egui::pos2(rect.center().x, rect.top() + 30.0), vec2(masse::SYMBOL, masse::SYMBOL)),
            sym,
            symbol_farbe,
        );
        p.galley(egui::pos2(rect.center().x - galley.size().x / 2.0, rect.bottom() - 16.0 - galley.size().y), galley, farbe);
    }
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, aus.is_none(), text));
    let r = match aus {
        Some(grund) => r.on_hover_text(format!(
            "{tipp}

{grund}"
        )),
        None => r.on_hover_text(tipp),
    };
    if aus.is_none() && r.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    r
}

/// Eine Schaltfläche, die einen Engine-Befehl ausführt; gesperrt, solange der Befehl nicht geht.
fn befehl(app: &mut SheetApp, ui: &mut Ui, text: &str, tipp: &str, id: &str, params: Json) {
    let aus = gesperrt(app, id).map(|_| texte::NICHT_MOEGLICH.to_string());
    if knopf(ui, text, tipp, aus.as_deref(), false) {
        app.run_or_alert(id, params);
    }
}

/// Senkrechter Trennstrich zwischen zwei Knopfgruppen eines Registers.
fn trenner(ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let (r, _) = ui.allocate_exact_size(vec2(14.0, masse::KACHEL_HOEHE), egui::Sense::hover());
    ui.painter()
        .line_segment([egui::pos2(r.center().x, r.top() + 8.0), egui::pos2(r.center().x, r.bottom() - 8.0)], egui::Stroke::new(2.0, t.ribbon_border));
}

/// Das SimpleGrid-Symbol aus dem Branding (grünes Blatt mit Eselsohr und 3×3-Raster), gezeichnet.
pub fn logo(ui: &mut Ui, hoehe: f32) {
    let (r, _) = ui.allocate_exact_size(vec2(hoehe * 512.0 / 452.0, hoehe), egui::Sense::hover());
    let p = ui.painter();
    let k = r.height() / 452.0;
    let pt = |x: f32, y: f32| egui::pos2(r.left() + x * k, r.top() + y * k);
    let gruen = egui::Color32::from_rgb(0x10, 0xB9, 0x81);
    // Blatt mit abgeschnittener Ecke oben rechts.
    let blatt = vec![pt(81.0, 38.0), pt(335.0, 38.0), pt(452.0, 156.0), pt(452.0, 414.0), pt(81.0, 414.0)];
    p.add(egui::Shape::convex_polygon(blatt, gruen, egui::Stroke::NONE));
    p.add(egui::Shape::convex_polygon(
        vec![pt(335.0, 38.0), pt(452.0, 156.0), pt(335.0, 156.0)],
        egui::Color32::from_rgb(0x6E, 0xE7, 0xB7),
        egui::Stroke::NONE,
    ));
    for zeile in 0..3 {
        for spalte in 0..3 {
            let x = 139.0 + spalte as f32 * 90.0;
            let y = 166.0 + zeile as f32 * 64.0;
            p.rect_filled(egui::Rect::from_min_max(pt(x, y), pt(x + 78.0, y + 52.0)), 1.0, egui::Color32::WHITE);
        }
    }
}

/// Die Register der Leiste (Variante A aus Phase 10).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Register {
    #[default]
    Start,
    Schrift,
    Rahmen,
    ZeilenSpalten,
    Drucken,
    Hilfe,
}

impl Register {
    pub const ALLE: [Register; 6] =
        [Register::Start, Register::Schrift, Register::Rahmen, Register::ZeilenSpalten, Register::Drucken, Register::Hilfe];
    pub fn name(self) -> &'static str {
        match self {
            Register::Start => "Start",
            Register::Schrift => "Schrift und Farbe",
            Register::Rahmen => "Rahmen und Zellen",
            Register::ZeilenSpalten => "Zeilen und Spalten",
            Register::Drucken => "Drucken",
            Register::Hilfe => "Hilfe",
        }
    }
}

/// Kopfzeile, Register und die Knöpfe des gewählten Registers.
pub fn leiste(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    // Kopfzeile: Logo, Name, Datei, Speicherstand. In sehr niedrigen Fenstern entfällt sie,
    // damit von der Tabelle genug sichtbar bleibt.
    let niedrig = ui.ctx().content_rect().height() < 640.0;
    if !niedrig {
        egui::Panel::top("simplegrid_kopf")
            .resizable(false)
            .exact_size(56.0)
            .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin::symmetric(20, 0)))
            .show(ui, |ui| {
                // Alles auf einer Linie: senkrecht mittig in der 56 px hohen Kopfzeile.
                ui.horizontal_centered(|ui| {
                    logo(ui, 32.0);
                    ui.add_space(4.0);
                    let mut job = egui::text::LayoutJob::default();
                    job.append("Simple", 0.0, egui::TextFormat { font_id: theme::ui_bold(24.0), color: t.text, ..Default::default() });
                    job.append("Grid", 0.0, egui::TextFormat { font_id: theme::ui_bold(24.0), color: t.accent, ..Default::default() });
                    ui.label(job);
                    let (titel, ungespeichert) = app.session.active().map(|d| (d.display_title(), d.is_dirty())).unwrap_or_default();
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);
                    ui.label(RichText::new(titel).font(theme::ui_font(20.0)).color(t.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (text, farbe) = if ungespeichert {
                            ("Noch nicht gespeichert", egui::Color32::from_rgb(0xB4, 0x53, 0x09))
                        } else {
                            ("Alles gespeichert", t.accent)
                        };
                        ui.label(RichText::new(text).font(theme::ui_bold(17.0)).color(farbe));
                        ui.add_space(16.0);
                        // Neu, Öffnen, Speichern, Speichern unter: immer sichtbar, unabhängig vom Register.
                        // Direkt in der rechtsbündigen Zeile, damit die Knöpfe mittig sitzen.
                        ui.spacing_mut().item_spacing.x = 8.0;
                        datei(app, ui);
                    });
                });
            });
    }
    // Register (Reiter).
    egui::Panel::top("simplegrid_register")
        .resizable(false)
        .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin { left: 20, right: 20, top: 4, bottom: 0 }))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 4.0);
                for reg in Register::ALLE {
                    let an = app.einfach.register == reg;
                    let text = RichText::new(reg.name()).font(if an { theme::ui_bold(19.0) } else { theme::ui_font(19.0) }).color(if an {
                        egui::Color32::WHITE
                    } else {
                        t.text
                    });
                    let b = egui::Button::new(text)
                        .fill(if an { t.accent } else { t.window })
                        .stroke(egui::Stroke::new(2.0, if an { t.accent } else { t.ribbon_border }))
                        .corner_radius(egui::CornerRadius { nw: 10, ne: 10, sw: 0, se: 0 })
                        .min_size(vec2(120.0, 50.0));
                    if ui.add(b).on_hover_text(format!("Knöpfe für „{}“ zeigen", reg.name())).clicked() {
                        app.einfach.register = reg;
                    }
                }
            });
        });
    // Die Knöpfe des gewählten Registers in einer Reihe (bricht bei schmalen Fenstern um).
    egui::Panel::top("simplegrid_knoepfe")
        .resizable(false)
        .frame(
            egui::Frame::NONE
                .fill(t.ribbon)
                .inner_margin(egui::Margin { left: 20, right: 20, top: 14, bottom: 14 })
                .stroke(egui::Stroke::new(1.0, t.ribbon_border)),
        )
        .show(ui, |ui| {
            // Grüne Linie unter den Reitern.
            let oben = ui.max_rect().top() - 14.0;
            ui.painter().line_segment(
                [egui::pos2(ui.max_rect().left() - 20.0, oben + 1.5), egui::pos2(ui.max_rect().right() + 20.0, oben + 1.5)],
                egui::Stroke::new(3.0, t.accent),
            );
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(masse::ABSTAND, 10.0);
                ui.spacing_mut().interact_size.y = if ui.ctx().content_rect().height() < 640.0 { masse::KNOPF_HOEHE } else { masse::KACHEL_HOEHE };
                ui.spacing_mut().button_padding = vec2(14.0, 8.0);
                match app.einfach.register {
                    Register::Start => {
                        bearbeiten(app, ui);
                        trenner(ui);
                        rechnen(app, ui);
                    }
                    Register::Schrift => {
                        schrift(app, ui);
                        trenner(ui);
                        zellen(app, ui);
                    }
                    Register::Rahmen => {
                        umrandung(app, ui);
                        trenner(ui);
                        verbinden_knoepfe(app, ui);
                    }
                    Register::ZeilenSpalten => zeilen_spalten(app, ui),
                    Register::Drucken => drucken(app, ui),
                    Register::Hilfe => {
                        if knopf(ui, "Info und Einstellungen", "Version, Einstellungen und Updates", None, false) {
                            app.einfach.info_offen = true;
                        }
                        if knopf(ui, "Nach Updates suchen", "Jetzt im Internet nachsehen, ob es eine neue Version gibt", None, false) {
                            app.check_for_updates();
                        }
                    }
                }
            });
        });
    fehler_hinweis(app, ui);
}

/// Die Fußzeile: Tabellenblätter als große Reiter, Blatt-Knöpfe und Zoom.
pub fn fusszeile(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::bottom("simplegrid_fuss")
        .resizable(false)
        .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin::symmetric(20, 8)).stroke(egui::Stroke::new(1.0, t.ribbon_border)))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 6.0);
                let (aktiv, namen) = app
                    .session
                    .active()
                    .map(|d| (d.wb.active_sheet.get(), d.wb.sheets.iter().map(|s| s.name.clone()).collect::<Vec<_>>()))
                    .unwrap_or_default();
                for (i, name) in namen.iter().enumerate() {
                    let an = i == aktiv;
                    let b = egui::Button::new(RichText::new(name).font(if an { theme::ui_bold(18.0) } else { theme::ui_font(18.0) }).color(if an {
                        t.accent_dark
                    } else {
                        t.text
                    }))
                    .fill(if an { t.accent_soft } else { t.ribbon })
                    .stroke(egui::Stroke::new(if an { 3.0 } else { 2.0 }, if an { t.accent } else { t.ribbon_border }))
                    .corner_radius(10.0)
                    .min_size(vec2(110.0, 48.0));
                    let r = ui.add(b).on_hover_text("Dieses Blatt zeigen (Doppelklick: umbenennen)");
                    if r.double_clicked() {
                        app.einfach.frage = Some(Frage::Umbenennen { blatt: i, name: name.clone() });
                    } else if r.clicked() && !an {
                        app.run_or_alert("sheet.activate", json!({"sheet": i}));
                    }
                }
                blaetter(app, ui);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    ansicht_rechts(app, ui);
                });
            });
        });
    // Waagerechte Bildlaufleiste der Tabelle (GridCraft), über der Fußzeile, gut greifbar.
    app.grid.hscroll = None;
    egui::Panel::bottom("simplegrid_bildlauf").resizable(false).exact_size(26.0).frame(egui::Frame::NONE.fill(t.window)).show(ui, |ui| {
        let platz = ui.available_rect_before_wrap();
        let links = app.grid.rect.map_or(platz.left(), |r| r.left()).max(platz.left());
        let rechts = app.grid.rect.map_or(platz.right(), |r| r.right()).min(platz.right());
        let spur = egui::Rect::from_min_max(egui::pos2(links + 40.0, platz.center().y - 9.0), egui::pos2(rechts, platz.center().y + 9.0));
        crate::grid::horizontal_scrollbar(app, ui, spur, &t);
    });
}

/// Neu, Öffnen, Speichern, Speichern unter. Die Kopfzeile zeichnet sie von rechts nach links,
/// daher werden sie hier in umgekehrter Reihenfolge angelegt (sichtbar bleibt: Neu … Speichern unter).
fn datei(app: &mut SheetApp, ui: &mut Ui) {
    let ohne_mappe = app.session.active().is_none().then_some(texte::NICHT_MOEGLICH);
    if kopfknopf(ui, "Speichern unter", "Die Tabelle unter einem neuen Namen oder an einem anderen Ort speichern", ohne_mappe) {
        speichern_unter(app);
    }
    if kopfknopf(ui, "Speichern", "Die Tabelle speichern", ohne_mappe) {
        speichern(app);
    }
    if kopfknopf(ui, "Öffnen", "Eine gespeicherte Tabelle öffnen (.xlsx oder .ods)", None) {
        neu_oder_oeffnen(app, Danach::Oeffnen);
    }
    if kopfknopf(ui, "Neu", "Eine neue, leere Tabelle anfangen", None) {
        neu_oder_oeffnen(app, Danach::Neu);
    }
}

/// Ein Knopf der Kopfzeile: Icon links, Text rechts, gut greifbar, aber flacher als die Kacheln.
fn kopfknopf(ui: &mut Ui, text: &str, tipp: &str, aus: Option<&str>) -> bool {
    let t = Tokens::get(ui.ctx());
    let farbe = if aus.is_some() { t.text_disabled } else { t.text };
    let galley = ui.painter().layout_no_wrap(text.to_string(), theme::ui_bold(masse::SCHRIFT), farbe);
    let icon = 24.0;
    let breite = 14.0 + icon + 8.0 + galley.size().x + 16.0;
    let sense = if aus.is_some() { egui::Sense::hover() } else { egui::Sense::click() };
    let (rect, r) = ui.allocate_exact_size(vec2(breite, 40.0), sense);
    let (fill, rand) = if aus.is_none() && r.is_pointer_button_down_on() {
        (t.pressed, t.accent)
    } else if aus.is_none() && r.hovered() {
        (t.accent_soft, t.accent)
    } else {
        (egui::Color32::WHITE, t.ribbon_border)
    };
    let p = ui.painter();
    p.rect(rect, 10.0, fill, egui::Stroke::new(if r.hovered() { 2.5 } else { 1.5 }, rand), egui::StrokeKind::Inside);
    if r.has_focus() {
        p.rect_stroke(rect.expand(3.0), 12.0, egui::Stroke::new(3.0, egui::Color32::from_rgb(0xF5, 0x9E, 0x0B)), egui::StrokeKind::Outside);
    }
    if let Some(sym) = symbol(text) {
        let mitte = egui::pos2(rect.left() + 14.0 + icon / 2.0, rect.center().y);
        crate::icons::paint(p, egui::Rect::from_center_size(mitte, vec2(icon, icon)), sym, if aus.is_some() { t.text_disabled } else { t.accent });
    }
    p.galley(egui::pos2(rect.left() + 14.0 + icon + 8.0, rect.center().y - galley.size().y / 2.0), galley, farbe);
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, aus.is_none(), text));
    let r = match aus {
        Some(grund) => r.on_hover_text(format!("{tipp}\n\n{grund}")),
        None => r.on_hover_text(tipp),
    };
    aus.is_none() && r.clicked()
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
    let vorher = app.session.active().map(|d| d.uid);
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
    // Die neue Tabelle ersetzt die bisherige im selben Fenster; andere Fenster bleiben.
    if let (Some(v), Some(n)) = (vorher, neu) {
        app.session.replace_document(v, n);
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

fn bearbeiten(app: &mut SheetApp, ui: &mut Ui) {
    let undo_aus = gesperrt(app, "edit.undo").map(|_| "Es gibt nichts rückgängig zu machen.");
    if knopf(ui, "Rückgängig", "Den letzten Schritt zurücknehmen (auch Strg+Z)", undo_aus, false) {
        app.run_or_alert("edit.undo", json!({}));
    }
    let redo_aus = gesperrt(app, "edit.redo").map(|_| "Es gibt nichts zu wiederholen.");
    if knopf(ui, "Wiederholen", "Den zurückgenommenen Schritt wiederholen (auch Strg+Y)", redo_aus, false) {
        app.run_or_alert("edit.redo", json!({}));
    }
    let ohne = gesperrt(app, "edit.copy").map(|_| texte::NICHT_MOEGLICH);
    if knopf(ui, "Kopieren", "Die markierten Zellen kopieren (auch Strg+C)", ohne, false) {
        let ctx = ui.ctx().clone();
        app.copy_to_clipboard(&ctx, "edit.copy");
    }
    if knopf(ui, "Ausschneiden", "Die markierten Zellen ausschneiden, um sie woanders einzufügen (auch Strg+X)", ohne, false) {
        let ctx = ui.ctx().clone();
        app.copy_to_clipboard(&ctx, "edit.cut");
    }
    if knopf(ui, "Einfügen", "Das Kopierte an der markierten Stelle einfügen, mit Formatierung (auch Strg+V)", ohne, false) {
        app.paste_from_clipboard();
    }
    befehl(app, ui, "Inhalt löschen", "Den Inhalt der markierten Zellen löschen (auch Entf)", "edit.clearContents", json!({}));
}

/// Das Format der aktiven Zelle (für eingedrückte Schaltflächen wie „Fett“).
fn aktiver_stil(app: &SheetApp) -> simplegrid_engine::model::Style {
    app.session
        .active()
        .and_then(|d| {
            let sh = d.wb.active()?;
            Some(d.wb.styles.get(sh.style_id(d.selection.active)).clone())
        })
        .unwrap_or_default()
}

/// Eine Schaltfläche mit Farbfeld, die beim Klick die Farbauswahl (eine Ebene, keine Untermenüs)
/// öffnet. Gibt die gewählte Farbe (`#RRGGBB` oder `none`) zurück.
fn farbknopf(app: &SheetApp, ui: &mut Ui, text: &str, tipp: &str, ohne_farbe: &str, aus: Option<&str>) -> Option<String> {
    let id = ui.scope_id().with(("simplegrid_farbe", text));
    let sym = symbol(text).unwrap_or(crate::icons::Icon::Fill);
    let r = kachel_antwort(ui, sym, text, tipp, aus, false);
    let farben = app.session.active().map(|d| d.wb.theme.colors).unwrap_or(simplegrid_engine::model::Theme::default().colors);
    let mut gewaehlt = None;
    egui::Popup::menu(&r)
        .id(id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .open_memory((aus.is_none() && r.clicked()).then_some(egui::SetOpenCommand::Toggle))
        .show(|ui| {
            if let Some(c) = crate::widgets::color_palette_fixed(ui, &farben, ohne_farbe) {
                gewaehlt = Some(c);
                ui.close();
            }
        });
    gewaehlt
}

fn schrift(app: &mut SheetApp, ui: &mut Ui) {
    let st = aktiver_stil(app);
    let ohne = gesperrt(app, "home.bold").map(|_| texte::NICHT_MOEGLICH);
    if knopf(ui, "Fett", "Die markierten Zellen fett schreiben (noch einmal klicken: wieder normal)", ohne, st.font.bold) {
        app.run_or_alert("home.bold", json!({}));
    }
    if knopf(ui, "Kursiv", "Die markierten Zellen schräg schreiben (noch einmal klicken: wieder normal)", ohne, st.font.italic) {
        app.run_or_alert("home.italic", json!({}));
    }
    let unterstrichen = st.font.underline != simplegrid_engine::model::Underline::None;
    if knopf(ui, "Unterstrichen", "Die markierten Zellen unterstreichen (noch einmal klicken: wieder normal)", ohne, unterstrichen) {
        app.run_or_alert("home.underline", json!({}));
    }
    // Schriftgröße: ein Auswahlfeld mit festen Stufen.
    let mut groesse = st.font.size;
    egui::ComboBox::from_id_salt("simplegrid_schriftgroesse")
        .selected_text(
            RichText::new(format!("Größe {}", simplegrid_engine::core::locale::localize_number_text(&format!("{groesse}"))))
                .font(theme::ui_font(masse::SCHRIFT)),
        )
        .width(masse::KNOPF_BREITE + 30.0)
        .height(400.0)
        .show_ui(ui, |ui| {
            for g in SCHRIFTGROESSEN {
                if ui.selectable_value(&mut groesse, g, RichText::new(format!("{g}")).font(theme::ui_font(masse::SCHRIFT))).clicked() {
                    app.run_or_alert("home.fontSize", json!({"size": g}));
                }
            }
        })
        .response
        .on_hover_text("Die Schriftgröße der markierten Zellen wählen");
    if let Some(c) = farbknopf(app, ui, "Textfarbe", "Die Schriftfarbe der markierten Zellen wählen", "Automatisch", ohne) {
        app.run_or_alert("home.fontColor", json!({"color": if c == "none" { "auto".to_string() } else { c }}));
    }
}

fn zellen(app: &mut SheetApp, ui: &mut Ui) {
    use simplegrid_engine::model::HAlign;
    let st = aktiver_stil(app);
    let ohne = gesperrt(app, "home.fillColor").map(|_| texte::NICHT_MOEGLICH);
    if let Some(c) = farbknopf(app, ui, "Hintergrund", "Die Hintergrundfarbe der markierten Zellen wählen", "Keine Farbe", ohne) {
        app.run_or_alert("home.fillColor", json!({"color": c}));
    }
    for (text, tipp, h, id) in [
        ("Links", "Den Inhalt links ausrichten", HAlign::Left, "home.alignLeft"),
        ("Mitte", "Den Inhalt in die Mitte setzen", HAlign::Center, "home.alignCenter"),
        ("Rechts", "Den Inhalt rechts ausrichten", HAlign::Right, "home.alignRight"),
    ] {
        if knopf(ui, text, tipp, ohne, st.align.h == h) {
            app.run_or_alert(id, json!({}));
        }
    }
    let euro = ist_euro_format(st.num_fmt.as_str());
    if knopf(ui, "Euro €", "Zahlen als Geldbetrag zeigen, z. B. 1.234,56 € (noch einmal klicken: normale Zahl)", ohne, euro) {
        euro_umschalten(app, euro);
    }
}

/// „Zellen verbinden“ und „Verbindung trennen“.
fn verbinden_knoepfe(app: &mut SheetApp, ui: &mut Ui) {
    let ohne = gesperrt(app, "home.mergeCells").map(|_| texte::NICHT_MOEGLICH);
    if knopf(ui, "Zellen verbinden", "Die markierten Zellen zu einer großen Zelle verbinden", ohne, false) {
        verbinden(app);
    }
    let in_verbund = app
        .session
        .active()
        .and_then(|d| d.wb.active().map(|sh| (sh.merges.clone(), d.selection.current())))
        .is_some_and(|(m, r)| m.iter().any(|x| x.intersects(&r)));
    let trennen_aus = if in_verbund { ohne } else { Some("Hier sind keine verbundenen Zellen markiert.") };
    if knopf(ui, "Verbindung trennen", "Verbundene Zellen wieder in einzelne Zellen teilen", trennen_aus, false) {
        app.run_or_alert("home.unmergeCells", json!({}));
    }
}

/// Ob ein Zahlenformat Euro zeigt: SimpleGrid's eigenes, aber auch Excel's „Buchhaltung“ und
/// „Währung“ mit € (z. B. `_-* #,##0.00 €_-;…` oder `[$€-407]`).
pub fn ist_euro_format(code: &str) -> bool {
    code.contains('€')
}

/// „Euro“: an → Euro-Format; noch einmal → normale Zahl.
pub fn euro_umschalten(app: &mut SheetApp, ist_euro: bool) {
    let code = if ist_euro { "General" } else { simplegrid_engine::core::locale::EURO_FORMAT };
    app.run_or_alert("home.numberFormat", json!({"code": code}));
}

/// „Zellen verbinden“: gehen dabei Inhalte verloren (nur die obere linke Zelle bleibt), erst fragen.
pub fn verbinden(app: &mut SheetApp) {
    if !app.commit_edit(0, 0, false, false) {
        return;
    }
    let verlust = app.session.active().and_then(|d| {
        let r = d.selection.current();
        let sh = d.wb.active()?;
        Some(sh.cells.iter_range(r).any(|(c, cell)| c != r.start && !cell.is_blank()))
    });
    if verlust == Some(true) {
        app.einfach.frage = Some(Frage::Bestaetigen {
            titel: "Zellen verbinden?".into(),
            text: "Beim Verbinden bleibt nur der Inhalt der Zelle oben links erhalten. Die anderen Inhalte gehen verloren. Mit „Rückgängig“ können Sie es zurückholen.".into(),
            knopf: "Verbinden".into(),
            befehl: "home.mergeCells".into(),
            params: json!({}),
        });
    } else {
        app.run_or_alert("home.mergeCells", json!({}));
    }
}

fn umrandung(app: &mut SheetApp, ui: &mut Ui) {
    let ohne = gesperrt(app, "home.borders").map(|_| texte::NICHT_MOEGLICH);
    let dicke = if app.einfach.rahmen_dicke.is_empty() { "thin".to_string() } else { app.einfach.rahmen_dicke.clone() };
    let anzeige = RAHMEN_DICKEN.iter().find(|(k, _)| *k == dicke).map_or("Dünn", |(_, a)| *a);
    egui::ComboBox::from_id_salt("simplegrid_rahmendicke")
        .selected_text(RichText::new(format!("Linie: {anzeige}")).font(theme::ui_font(masse::SCHRIFT)))
        .width(masse::KNOPF_BREITE + 40.0)
        .show_ui(ui, |ui| {
            for (k, a) in RAHMEN_DICKEN {
                if ui.selectable_label(dicke == k, RichText::new(a).font(theme::ui_font(masse::SCHRIFT))).clicked() {
                    app.einfach.rahmen_dicke = k.to_string();
                }
            }
        })
        .response
        .on_hover_text("Wie dick die Linien werden");
    if let Some(c) = farbknopf(app, ui, "Linienfarbe", "Die Farbe der Linien wählen", "Schwarz", ohne) {
        app.einfach.rahmen_farbe = if c == "none" { String::new() } else { c };
    }
    for (text, tipp, preset) in [
        ("Außen", "Einen Rahmen um die markierten Zellen ziehen", "outside"),
        ("Innen", "Linien zwischen den markierten Zellen ziehen", "inside"),
        ("Alle", "Alle Linien der markierten Zellen ziehen", "all"),
        ("Keine", "Alle Linien der markierten Zellen entfernen", "none"),
    ] {
        if knopf(ui, text, tipp, ohne, false) {
            umranden(app, preset);
        }
    }
}

/// Zieht die Umrandung `preset` mit der gewählten Dicke und Farbe.
pub fn umranden(app: &mut SheetApp, preset: &str) {
    let dicke = if app.einfach.rahmen_dicke.is_empty() { "thin" } else { app.einfach.rahmen_dicke.as_str() };
    let farbe = if app.einfach.rahmen_farbe.is_empty() { "#000000" } else { app.einfach.rahmen_farbe.as_str() };
    let params = json!({"preset": preset, "style": dicke, "color": farbe});
    app.run_or_alert("home.borders", params);
}

/// Zoom „− Kleiner  120 %  + Größer“ in der Fußzeile (rechtsbündig, daher von rechts gezeichnet).
fn ansicht_rechts(app: &mut SheetApp, ui: &mut Ui) {
    let zoom = app.session.active().and_then(|d| d.wb.active().map(|s| u32::from(s.zoom))).unwrap_or(100);
    let ohne = gesperrt(app, "view.zoom").map(|_| texte::NICHT_MOEGLICH);
    let kleiner = ZOOMSTUFEN.iter().rev().find(|z| **z < zoom).copied();
    let groesser = ZOOMSTUFEN.iter().find(|z| **z > zoom).copied();
    if knopf(
        ui,
        "+ Größer",
        "Die Tabelle größer anzeigen (ändert nicht den Druck)",
        if groesser.is_none() { Some("Größer geht es nicht.") } else { ohne },
        false,
    ) && let Some(z) = groesser
    {
        app.run_or_alert("view.zoom", json!({"percent": z}));
    }
    ui.label(RichText::new(format!("{zoom} %")).font(theme::ui_bold(masse::SCHRIFT + 1.0)));
    if knopf(
        ui,
        "– Kleiner",
        "Die Tabelle kleiner anzeigen (ändert nicht den Druck)",
        if kleiner.is_none() { Some("Kleiner geht es nicht.") } else { ohne },
        false,
    ) && let Some(z) = kleiner
    {
        app.run_or_alert("view.zoom", json!({"percent": z}));
    }
}

/// Der Fenstertitel: „Haushalt.xlsx – SimpleGrid“, mit Hinweis, solange etwas nicht gespeichert ist.
pub fn fenstertitel(titel: &str, ungespeichert: bool) -> String {
    if ungespeichert { format!("{titel} (nicht gespeichert) – SimpleGrid") } else { format!("{titel} – SimpleGrid") }
}

/// Merkt sich die gespeicherte Datei der aktiven Tabelle als „zuletzt benutzt“.
pub fn zuletzt_merken(app: &mut SheetApp) {
    let Some(pfad) = app.session.active().and_then(|d| d.path.clone()) else { return };
    if !pfad.to_ascii_lowercase().ends_with(".xlsx") || app.ui.recent.first() == Some(&pfad) {
        return;
    }
    app.ui.recent.retain(|p| p != &pfad);
    app.ui.recent.insert(0, pfad);
    app.ui.recent.truncate(20);
}

/// Beim Start: Sicherung nach Absturz anbieten, sonst (falls eingeschaltet) die zuletzt
/// benutzte Tabelle öffnen.
pub fn beim_start(app: &mut SheetApp) {
    if let Some(info) = app.einfach.sicherung.vorhanden() {
        app.einfach.frage = Some(Frage::Wiederherstellen { titel: info.titel, zeit: info.zeit });
        return;
    }
    let unberuehrt = app.session.documents().len() == 1 && app.session.active().is_some_and(|d| d.path.is_none() && !d.is_dirty());
    if app.ui.letzte_datei_oeffnen
        && unberuehrt
        && let Some(pfad) = app.ui.recent.first().cloned()
        && std::path::Path::new(&pfad).is_file()
    {
        oeffnen_pfad(app, &pfad);
    }
}

/// Das kleine Fenster „Info und Einstellungen“.
fn info_fenster(app: &mut SheetApp, ctx: &egui::Context) {
    if !app.einfach.info_offen {
        return;
    }
    let mut offen = true;
    egui::Modal::new(egui::Id::unique("simplegrid.info")).show(ctx, |ui| {
        ui.set_width(520.0);
        ui.label(RichText::new("SimpleGrid").font(theme::ui_bold(masse::SCHRIFT + 6.0)));
        ui.label(RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION"))).font(theme::ui_font(masse::SCHRIFT)));
        ui.add_space(4.0);
        ui.label(
            RichText::new("Ein einfaches Tabellenprogramm. Beruht auf GridCraft (MIT OR Apache-2.0).").font(theme::ui_font(masse::SCHRIFT - 2.0)),
        );
        ui.add_space(12.0);
        ui.label(RichText::new("Einstellungen").font(theme::ui_bold(masse::SCHRIFT)));
        let mut an = app.ui.letzte_datei_oeffnen;
        if ui.checkbox(&mut an, RichText::new("Beim Start die zuletzt benutzte Tabelle öffnen").font(theme::ui_font(masse::SCHRIFT))).changed() {
            app.ui.letzte_datei_oeffnen = an;
        }
        let mut auto = app.ui.update_automatisch;
        if ui.checkbox(&mut auto, RichText::new("Automatisch nach Updates suchen (einmal am Tag)").font(theme::ui_font(masse::SCHRIFT))).changed() {
            app.ui.update_automatisch = auto;
        }
        ui.add_space(8.0);
        if knopf(ui, "Nach Updates suchen", "Jetzt im Internet nachsehen, ob es eine neue Version gibt", None, false) {
            app.check_for_updates();
        }
        ui.add_space(16.0);
        if knopf(ui, "Schließen", "Dieses Fenster schließen", None, false) || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            offen = false;
        }
    });
    app.einfach.info_offen = offen;
}

/// Ein festgelegter Druckbereich bekommt am Bildschirm einen kräftigen grünen, gestrichelten
/// Rahmen und oben links ein Schild „Druckbereich“. Gedruckt wird beides nicht.
pub(crate) fn druckbereich_rahmen(ui: &Ui, geo: &crate::grid::Geo, sh: &simplegrid_engine::model::Sheet) {
    let Some(bereich) = sh.print.print_area else { return };
    let r = geo.range_rect(sh, bereich);
    if !r.is_positive() || !r.intersects(geo.cells) {
        return;
    }
    let gruen = egui::Color32::from_rgb(0x04, 0x78, 0x57);
    let p = ui.painter().with_clip_rect(geo.cells);
    let ecken = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
    for paar in ecken.windows(2) {
        p.extend(egui::Shape::dashed_line(&[paar[0], paar[1]], egui::Stroke::new(3.0, gruen), 10.0, 6.0));
    }
    let schrift = theme::ui_bold(13.0);
    let galley = p.layout_no_wrap("Druckbereich".to_string(), schrift, egui::Color32::WHITE);
    let schild = egui::Rect::from_min_size(egui::pos2(r.left(), r.top() - galley.size().y - 6.0), galley.size() + vec2(12.0, 6.0));
    let schild = if schild.top() < geo.cells.top() { schild.translate(vec2(0.0, galley.size().y + 8.0)) } else { schild };
    p.rect_filled(schild, 4.0, gruen);
    p.galley(schild.min + vec2(6.0, 3.0), galley, egui::Color32::WHITE);
}

/// Die Klartext-Erklärung zum Fehlerwert der markierten Zelle, z. B. „Durch Null geteilt …“.
pub fn fehler_erklaerung(app: &SheetApp) -> Option<String> {
    let d = app.session.active()?;
    let sh = d.wb.active()?;
    let zelle = d.selection.active;
    match sh.value(zelle) {
        simplegrid_engine::core::Value::Error(e) => Some(format!(
            "{} in Zelle {}: {}",
            simplegrid_engine::core::locale::error_name(e),
            zelle.a1(),
            simplegrid_engine::core::locale::error_explanation(e)
        )),
        _ => None,
    }
}

/// Ein gut sichtbarer Hinweisstreifen unter der Leiste, solange eine Fehlerzelle markiert ist.
fn fehler_hinweis(app: &SheetApp, ui: &mut Ui) {
    let Some(text) = fehler_erklaerung(app) else { return };
    let t = Tokens::get(ui.ctx());
    egui::Panel::top("simplegrid_fehler")
        .resizable(false)
        .frame(
            egui::Frame::NONE
                .fill(t.notice_warn_bg)
                .stroke(egui::Stroke::new(1.0, t.notice_warn_border))
                .inner_margin(egui::Margin::symmetric(12, 8)),
        )
        .show(ui, |ui| {
            ui.label(RichText::new(format!("Achtung – {text}")).font(theme::ui_font(masse::SCHRIFT)).color(t.text));
        });
}

fn rechnen(app: &mut SheetApp, ui: &mut Ui) {
    let ohne = gesperrt(app, "formulas.autoSum").map(|_| texte::NICHT_MOEGLICH);
    if knopf(
        ui,
        "Σ Summe",
        "Zahlen zusammenzählen: Bereich markieren und klicken – die Summe kommt darunter (bei einer Zeile rechts daneben)",
        ohne,
        false,
    ) {
        summe(app);
    }
}

/// Wohin die Summe kommt und welcher Bereich zusammengezählt wird – oder `None`, wenn das nicht
/// eindeutig ist (dann rechnet GridCraft's AutoSumme direkt, z. B. für mehrere Spalten auf einmal).
pub fn summe_ziel(app: &SheetApp) -> Option<(simplegrid_engine::core::CellRef, simplegrid_engine::core::RangeRef)> {
    use simplegrid_engine::core::{CellRef, MAX_COLS, MAX_ROWS, RangeRef, Value};
    let d = app.session.active()?;
    let sh = d.wb.active()?;
    let r = d.selection.current();
    let zahl = |c: CellRef| matches!(sh.value(c), Value::Number(_));
    let leer = |c: CellRef| sh.value(c) == Value::Empty;
    // Nächste freie Zelle unterhalb von `zeile` bzw. rechts von `spalte`.
    let frei_unter = |zeile: u32, spalte: u32| (zeile + 1..MAX_ROWS).map(|z| CellRef::new(z, spalte)).find(|c| leer(*c));
    let frei_rechts = |zeile: u32, spalte: u32| (spalte + 1..MAX_COLS).map(|s| CellRef::new(zeile, s)).find(|c| leer(*c));
    if r.is_single() {
        let c = r.start;
        if zahl(c) {
            // Eine Zahl in einer Zahlenspalte: den zusammenhängenden Zahlenbereich erkennen.
            let mut oben = c.row;
            while oben > 0 && zahl(CellRef::new(oben - 1, c.col)) {
                oben -= 1;
            }
            let mut unten = c.row;
            while unten + 1 < MAX_ROWS && zahl(CellRef::new(unten + 1, c.col)) {
                unten += 1;
            }
            let ziel = CellRef::new(unten + 1, c.col);
            return (unten + 1 < MAX_ROWS && leer(ziel)).then(|| (ziel, RangeRef::new(CellRef::new(oben, c.col), CellRef::new(unten, c.col))));
        }
        if !leer(c) {
            return None;
        }
        // Leere Zelle: Zahlen direkt darüber, sonst direkt links davon.
        if c.row > 0 && zahl(CellRef::new(c.row - 1, c.col)) {
            let mut oben = c.row - 1;
            while oben > 0 && zahl(CellRef::new(oben - 1, c.col)) {
                oben -= 1;
            }
            return Some((c, RangeRef::new(CellRef::new(oben, c.col), CellRef::new(c.row - 1, c.col))));
        }
        if c.col > 0 && zahl(CellRef::new(c.row, c.col - 1)) {
            let mut links = c.col - 1;
            while links > 0 && zahl(CellRef::new(c.row, links - 1)) {
                links -= 1;
            }
            return Some((c, RangeRef::new(CellRef::new(c.row, links), CellRef::new(c.row, c.col - 1))));
        }
        return None;
    }
    if r.start.col == r.end.col {
        // Eine Spalte markiert: Summe darunter.
        return frei_unter(r.end.row, r.start.col).map(|z| (z, r));
    }
    if r.start.row == r.end.row {
        // Eine Zeile markiert: Summe rechts daneben.
        return frei_rechts(r.start.row, r.end.col).map(|z| (z, r));
    }
    None
}

/// „Summe“: Die Formel (z. B. `=SUMME(B2:B4)`) wird in die Zielzelle geschrieben und zum
/// Bestätigen vorgelegt – mit Enter oder dem grünen Haken übernehmen, mit Esc oder dem roten
/// Kreuz verwerfen. Ist das Ziel nicht eindeutig (mehrere Spalten), rechnet GridCraft's AutoSumme direkt.
pub fn summe(app: &mut SheetApp) {
    if !app.commit_edit(0, 0, false, false) {
        return;
    }
    match summe_ziel(app) {
        Some((zelle, bereich)) => {
            app.run_or_alert("selection.set", json!({"cell": zelle.a1()}));
            // Funktionsname in der Sprache der Oberfläche (deutsch: SUMME), gespeichert wird SUM.
            let name = app.ui.language.formula_locale().function_name("SUM").to_string();
            app.begin_edit(Some(format!("={name}({})", bereich.a1())), false);
            app.einfach.summe_ziel = app.editor.as_ref().map(|e| e.cell);
        }
        None => app.run_or_alert("formulas.autoSum", json!({})),
    }
}

/// Grüner Haken und rotes Kreuz neben der Zelle, die gerade bearbeitet wird (jede Eingabe, auch
/// eine vorgelegte Summe): übernehmen wie Enter, verwerfen wie Esc – ganz ohne Tastatur.
fn eingabe_knoepfe(app: &mut SheetApp, ctx: &egui::Context) {
    let Some(ziel) = app.editor.as_ref().filter(|e| !e.from_formula_bar).map(|e| e.cell) else {
        app.einfach.summe_ziel = None;
        return;
    };
    // Rechts neben dem tatsächlichen Eingabefeld (es wächst mit dem Text); darunter bleibt Platz
    // für den Funktionshinweis.
    let feld: Option<egui::Rect> = ctx.data(|d| d.get_temp(egui::Id::unique("simplegrid.cell_editor.rect")));
    let Some(rect) = feld.or_else(|| app.grid.geo.as_ref().and_then(|g| app.session.active().and_then(|d| d.wb.active()).map(|sh| g.cell_rect(sh, ziel)))) else { return };
    let t = Tokens::get(ctx);
    let mut uebernehmen = false;
    let mut verwerfen = false;
    egui::Area::new(egui::Id::unique("simplegrid.summe_knoepfe"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(rect.right() + 8.0, rect.center().y - 30.0))
        .show(ctx, |ui| {
            egui::Frame::NONE.fill(egui::Color32::WHITE).stroke(egui::Stroke::new(1.5, t.ribbon_border)).corner_radius(12.0).inner_margin(6.0).shadow(egui::epaint::Shadow { offset: [0, 3], blur: 10, spread: 0, color: egui::Color32::from_black_alpha(40) }).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let gruen = egui::Color32::from_rgb(0x04, 0x78, 0x57);
                    let rot = egui::Color32::from_rgb(0xB9, 0x1C, 0x1C);
                    for (icon, farbe, tipp, ja) in [
                        (crate::icons::Icon::Check, gruen, "Eingabe übernehmen (auch Enter)", true),
                        (crate::icons::Icon::Close, rot, "Eingabe verwerfen (auch Esc)", false),
                    ] {
                        let (r, resp) = ui.allocate_exact_size(vec2(48.0, 48.0), egui::Sense::click());
                        let fill = if ja { gruen } else { egui::Color32::WHITE };
                        let fill = if resp.hovered() { fill.gamma_multiply(0.85) } else { fill };
                        ui.painter().rect(r, 10.0, fill, egui::Stroke::new(3.0, farbe), egui::StrokeKind::Inside);
                        crate::icons::paint(ui.painter(), r.shrink(10.0), icon, if ja { egui::Color32::WHITE } else { rot });
                        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tipp));
                        if resp.on_hover_text(tipp).clicked() {
                            if ja {
                                uebernehmen = true;
                            } else {
                                verwerfen = true;
                            }
                        }
                    }
                });
            });
        });
    if uebernehmen {
        app.einfach.summe_ziel = None;
        app.commit_edit(0, 0, false, false);
    } else if verwerfen {
        app.einfach.summe_ziel = None;
        app.cancel_edit();
    }
}

fn drucken(app: &mut SheetApp, ui: &mut Ui) {
    let ohne = gesperrt(app, "pageLayout.printArea").map(|_| texte::NICHT_MOEGLICH);
    if knopf(ui, "Seitenansicht und Drucken", "Zeigen, wie die Seiten gedruckt aussehen, und dann drucken", ohne, false) {
        crate::seitenansicht::oeffnen(app);
    }
    if knopf(ui, "Druckbereich festlegen", "Nur die markierten Zellen drucken", ohne, false) {
        app.run_or_alert("pageLayout.printArea", json!({}));
    }
    let bereich = app.session.active().and_then(|d| d.wb.active().and_then(|s| s.print.print_area)).map(|r| r.a1());
    let aufheben_aus =
        if bereich.is_some() { ohne } else { Some("Es ist kein Druckbereich festgelegt: gedruckt wird alles, was auf dem Blatt steht.") };
    let tipp = match &bereich {
        Some(b) => format!("Wieder alles drucken statt nur {b}"),
        None => "Wieder alles drucken".into(),
    };
    if knopf(ui, "Druckbereich aufheben", &tipp, aufheben_aus, false) {
        app.run_or_alert("pageLayout.printArea", json!({"clear": true}));
    }
}

fn zeilen_spalten(app: &mut SheetApp, ui: &mut Ui) {
    let ohne = gesperrt(app, "home.insertRows").map(|_| texte::NICHT_MOEGLICH);
    let (zeilen, spalten) = auswahl_zeilen_spalten(app);
    let mehrzahl = |n: u32, eins: &str, viele: &str| if n > 1 { format!("{n} {viele}") } else { eins.to_string() };
    if knopf(
        ui,
        "Zeile hinzufügen",
        &format!("{} über der Markierung einfügen", mehrzahl(zeilen.1 - zeilen.0 + 1, "Eine leere Zeile", "leere Zeilen")),
        ohne,
        false,
    ) {
        app.run_or_alert("home.insertRows", json!({}));
    }
    if knopf(ui, "Zeile entfernen", "Die markierten Zeilen entfernen (mit Rückfrage, wenn Daten darin stehen)", ohne, false) {
        entfernen(app, true);
    }
    if knopf(
        ui,
        "Spalte hinzufügen",
        &format!("{} links neben der Markierung einfügen", mehrzahl(spalten.1 - spalten.0 + 1, "Eine leere Spalte", "leere Spalten")),
        ohne,
        false,
    ) {
        app.run_or_alert("home.insertColumns", json!({}));
    }
    if knopf(ui, "Spalte entfernen", "Die markierten Spalten entfernen (mit Rückfrage, wenn Daten darin stehen)", ohne, false) {
        entfernen(app, false);
    }
    if knopf(
        ui,
        "Spaltenbreite …",
        "Die Breite der markierten Spalten als Zahl eingeben (man kann auch die Linie im Spaltenkopf ziehen)",
        ohne,
        false,
    ) {
        app.open_dialog("columnWidth", json!({}));
    }
    if knopf(ui, "Zeilenhöhe …", "Die Höhe der markierten Zeilen als Zahl eingeben (man kann auch die Linie im Zeilenkopf ziehen)", ohne, false) {
        app.open_dialog("rowHeight", json!({}));
    }
}

/// Erste und letzte markierte Zeile sowie Spalte (nullbasiert).
fn auswahl_zeilen_spalten(app: &SheetApp) -> ((u32, u32), (u32, u32)) {
    app.session
        .active()
        .map(|d| {
            let r = d.selection.current();
            ((r.start.row, r.end.row), (r.start.col, r.end.col))
        })
        .unwrap_or(((0, 0), (0, 0)))
}

/// „Zeile entfernen“ / „Spalte entfernen“: stehen Daten darin, erst fragen.
pub(crate) fn entfernen(app: &mut SheetApp, zeilen: bool) {
    if !app.commit_edit(0, 0, false, false) {
        return;
    }
    let ((r0, r1), (c0, c1)) = auswahl_zeilen_spalten(app);
    let hat_daten = app.session.active().and_then(|d| d.wb.active()).is_some_and(|sh| {
        sh.cells.iter().any(|(c, cell)| !cell.is_blank() && if zeilen { (r0..=r1).contains(&c.row) } else { (c0..=c1).contains(&c.col) })
    });
    let befehl = if zeilen { "home.deleteRows" } else { "home.deleteColumns" };
    if !hat_daten {
        app.run_or_alert(befehl, json!({}));
        return;
    }
    let was = if zeilen {
        if r0 == r1 { format!("Die Zeile {}", r0 + 1) } else { format!("Die Zeilen {} bis {}", r0 + 1, r1 + 1) }
    } else {
        let name = |c: u32| simplegrid_engine::core::CellRef::new(0, c).a1().trim_end_matches(char::is_numeric).to_string();
        if c0 == c1 { format!("Die Spalte {}", name(c0)) } else { format!("Die Spalten {} bis {}", name(c0), name(c1)) }
    };
    app.einfach.frage = Some(Frage::Bestaetigen {
        titel: if zeilen { "Zeilen entfernen?".into() } else { "Spalten entfernen?".into() },
        text: format!(
            "{was} {} Daten. Wirklich entfernen? Mit „Rückgängig“ können Sie es zurückholen.",
            if (zeilen && r0 == r1) || (!zeilen && c0 == c1) { "enthält" } else { "enthalten" }
        ),
        knopf: "Entfernen".into(),
        befehl: befehl.into(),
        params: json!({}),
    });
}

fn blaetter(app: &mut SheetApp, ui: &mut Ui) {
    let (aktiv, anzahl, name) = app
        .session
        .active()
        .map(|d| (d.wb.active_sheet.get(), d.wb.sheets.len(), d.wb.active().map(|s| s.name.clone()).unwrap_or_default()))
        .unwrap_or((0, 0, String::new()));
    let ohne = gesperrt(app, "home.insertSheet").map(|_| texte::NICHT_MOEGLICH);
    if knopf(ui, "+ Blatt hinzufügen", "Ein neues, leeres Tabellenblatt hinten anfügen", ohne, false) {
        app.run_or_alert("home.insertSheet", json!({"before": anzahl}));
    }
    if knopf(ui, "Umbenennen", "Dem aktuellen Blatt einen neuen Namen geben", ohne, false) {
        app.einfach.frage = Some(Frage::Umbenennen { blatt: aktiv, name: name.clone() });
    }
    let loeschen_aus = if anzahl <= 1 { Some(texte::LETZTES_BLATT) } else { ohne };
    if knopf(ui, "Löschen", "Das aktuelle Blatt löschen (mit Rückfrage)", loeschen_aus, false) {
        app.einfach.frage = Some(Frage::BlattLoeschen { blatt: aktiv, name: name.clone() });
    }
    let links_aus = if aktiv == 0 { Some("Das Blatt steht schon ganz links.") } else { ohne };
    if knopf(ui, "◀", "Das aktuelle Blatt eine Stelle nach links schieben", links_aus, false) {
        app.run_or_alert("sheet.move", json!({"sheet": aktiv, "to": aktiv.saturating_sub(1)}));
    }
    let rechts_aus = if aktiv + 1 >= anzahl { Some("Das Blatt steht schon ganz rechts.") } else { ohne };
    if knopf(ui, "▶", "Das aktuelle Blatt eine Stelle nach rechts schieben", rechts_aus, false) {
        app.run_or_alert("sheet.move", json!({"sheet": aktiv, "to": aktiv + 1}));
    }
}

/// Ob ein Meldungstext (noch) englisch ist: nur ASCII-Zeichen und typisch englische Wörter.
fn ist_englisch(text: &str) -> bool {
    let t = format!(" {} ", text.to_ascii_lowercase());
    text.is_ascii() && [" the ", " is ", " can't ", " cannot ", " must ", " not ", " no ", " to ", " a ", " an ", " select ", " couldn't ", " failed", " invalid", " unable"].iter().any(|w| t.contains(w))
}

/// Eine Meldung auf Deutsch: übersetzt, sonst (falls noch englisch) eine allgemeine deutsche
/// Meldung; der Originaltext geht ins Protokoll.
pub fn deutsche_meldung(app: &SheetApp, text: &str) -> String {
    let t = crate::i18n::Language::De.tr_error(text);
    if ist_englisch(&t) {
        log::info!("Meldung (englisch, ersetzt): {text}");
        let _ = app;
        "Das hat leider nicht geklappt.".to_string()
    } else {
        t
    }
}

/// Das große Meldungsfenster (ersetzt GridCraft's kleines): Titel, Text, grüner „OK“-Knopf.
pub fn meldungsfenster(app: &mut SheetApp, ctx: &egui::Context) {
    let Some((titel, text)) = app.message.clone() else { return };
    let titel = match crate::i18n::Language::De.tr(&titel) {
        "SimpleGrid" | "" => texte::HINWEIS_TITEL.to_string(),
        t if ist_englisch(t) => texte::HINWEIS_TITEL.to_string(),
        t => t.to_string(),
    };
    let text = deutsche_meldung(app, &text);
    let mut zu = false;
    egui::Modal::new(egui::Id::unique("simplegrid.meldung"))
        .frame(egui::Frame::NONE.fill(egui::Color32::WHITE).stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0xD1, 0xD5, 0xDB))).inner_margin(egui::Margin::same(36)).corner_radius(18.0))
        .show(ctx, |ui| {
            ui.set_width(600.0);
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(56.0, 56.0), egui::Sense::hover());
                ui.painter().circle(r.center(), 27.0, egui::Color32::from_rgb(0xFE, 0xF3, 0xC7), egui::Stroke::new(3.0, egui::Color32::from_rgb(0xD9, 0x77, 0x06)));
                ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, "i", theme::ui_bold(32.0), egui::Color32::from_rgb(0x92, 0x40, 0x0E));
                ui.add_space(10.0);
                ui.label(RichText::new(&titel).font(theme::ui_bold(28.0)));
            });
            ui.add_space(14.0);
            ui.add(egui::Label::new(RichText::new(&text).font(theme::ui_font(21.0))).wrap());
            ui.add_space(20.0);
            if dialogknopf(ui, "OK", 0) || ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Escape)) {
                zu = true;
            }
        });
    if zu {
        app.message = None;
        // Zurück zur Eingabe, um die es ging (wie bei GridCraft).
        if let Some(ed) = &mut app.editor {
            ed.request_focus = true;
        }
    }
}

/// Die Speichern-Rückfrage beim Schließen eines Fensters (GridCraft's „saveChanges“) im großen,
/// deutschen Stil. Gibt `false` zurück, wenn sie beantwortet ist.
pub fn speichern_frage(app: &mut SheetApp, ctx: &egui::Context, titel: &str, quitting: bool) -> bool {
    let mut antwort = None;
    egui::Modal::new(egui::Id::unique("simplegrid.speichernfrage"))
        .frame(
            egui::Frame::NONE
                .fill(egui::Color32::WHITE)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0xD1, 0xD5, 0xDB)))
                .inner_margin(egui::Margin::same(36))
                .corner_radius(18.0),
        )
        .show(ctx, |ui| {
            ui.set_width(640.0);
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(56.0, 56.0), egui::Sense::hover());
                ui.painter().circle(r.center(), 27.0, egui::Color32::from_rgb(0xFE, 0xF3, 0xC7), egui::Stroke::new(3.0, egui::Color32::from_rgb(0xD9, 0x77, 0x06)));
                ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, "!", theme::ui_bold(32.0), egui::Color32::from_rgb(0x92, 0x40, 0x0E));
                ui.add_space(10.0);
                ui.label(RichText::new(texte::UNGESPEICHERT_TITEL).font(theme::ui_bold(30.0)));
            });
            ui.add_space(14.0);
            ui.add(egui::Label::new(RichText::new(texte::UNGESPEICHERT_TEXT.replace("{}", titel)).font(theme::ui_font(21.0))).wrap());
            ui.add_space(20.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(14.0, 10.0);
                for (i, (k, a)) in [
                    (texte::SPEICHERN, crate::SaveAnswer::Save),
                    (texte::NICHT_SPEICHERN, crate::SaveAnswer::DontSave),
                    (texte::ABBRECHEN, crate::SaveAnswer::Cancel),
                ]
                .into_iter()
                .enumerate()
                {
                    if dialogknopf(ui, k, i) {
                        antwort = Some(a);
                    }
                }
            });
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                antwort = Some(crate::SaveAnswer::Cancel);
            }
        });
    match antwort {
        Some(a) => !app.answer_save_changes(a, quitting, ctx),
        None => true,
    }
}

/// Ein Knopf in einer Rückfrage: der erste ist die Hauptaktion (grün gefüllt), „Nicht speichern“
/// und „Löschen“/„Entfernen“ sind rot umrandet, „Abbrechen“ und „Später“ grau.
fn dialogknopf(ui: &mut Ui, text: &str, nr: usize) -> bool {
    let t = Tokens::get(ui.ctx());
    let rot = egui::Color32::from_rgb(0xB9, 0x1C, 0x1C);
    let gefaehrlich = matches!(text, "Nicht speichern" | "Löschen" | "Entfernen" | "Nicht wiederherstellen");
    let (fill, rand, farbe) = if nr == 0 && !gefaehrlich {
        (t.accent, t.accent, egui::Color32::WHITE)
    } else if gefaehrlich {
        (egui::Color32::WHITE, rot, rot)
    } else {
        (egui::Color32::from_rgb(0xF9, 0xFA, 0xFB), egui::Color32::from_rgb(0x9C, 0xA3, 0xAF), t.text)
    };
    let b = egui::Button::new(RichText::new(text).font(theme::ui_bold(21.0)).color(farbe))
        .fill(fill)
        .stroke(egui::Stroke::new(3.0, rand))
        .corner_radius(14.0)
        .min_size(vec2(190.0, 68.0));
    ui.add(b).clicked()
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

/// Ersetzt die Tabelle im aktuellen Fenster durch eine neue bzw. eine geöffnete. Weitere Fenster
/// (per Ziehen oder Doppelklick geöffnet) bleiben unberührt.
fn ausfuehren(app: &mut SheetApp, danach: Danach) {
    match danach {
        Danach::Neu => {
            let vorher = app.session.active().map(|d| d.uid);
            app.run_or_alert("file.new", json!({}));
            let neu = app.session.active().map(|d| d.uid);
            if let (Some(v), Some(n)) = (vorher, neu) {
                app.session.replace_document(v, n);
            }
            app.follow_active_window();
        }
        Danach::Oeffnen => oeffnen(app),
        Danach::Beenden => {
            app.allow_close = true;
            app.einfach.jetzt_beenden = true;
        }
        Danach::Aktualisieren => app.einfach.update_laden = true,
    }
}

/// Zeigt eine offene Rückfrage als Fenster mit großen Schaltflächen.
pub fn rueckfragen(app: &mut SheetApp, ctx: &egui::Context) {
    if app.einfach.jetzt_beenden {
        app.einfach.jetzt_beenden = false;
        app.einfach.sicherung.loeschen();
        ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
    }
    eingabe_knoepfe(app, ctx);
    if std::mem::take(&mut app.einfach.update_laden) {
        #[cfg(not(target_arch = "wasm32"))]
        crate::updates_ui::herunterladen(app, ctx);
    }
    info_fenster(app, ctx);
    let Some(frage) = app.einfach.frage.clone() else { return };
    let (titel, text, knoepfe): (&str, String, Vec<&str>) = match &frage {
        Frage::Ungespeichert { .. } => {
            let name = app.session.active().map(|d| d.display_title()).unwrap_or_default();
            (
                texte::UNGESPEICHERT_TITEL,
                texte::UNGESPEICHERT_TEXT.replace("{}", &name),
                vec![texte::SPEICHERN, texte::NICHT_SPEICHERN, texte::ABBRECHEN],
            )
        }
        Frage::BlattLoeschen { name, .. } => {
            (texte::BLATT_LOESCHEN_TITEL, texte::BLATT_LOESCHEN_TEXT.replace("{}", name), vec![texte::LOESCHEN, texte::ABBRECHEN])
        }
        Frage::Bestaetigen { titel, text, knopf, .. } => (titel.as_str(), text.clone(), vec![knopf.as_str(), texte::ABBRECHEN]),
        Frage::Update { version } => (
            "Neue Version",
            format!("Es gibt eine neue Version von SimpleGrid ({version}). Jetzt aktualisieren? Das Programm startet danach von selbst neu."),
            vec!["Jetzt aktualisieren", "Später"],
        ),
        Frage::Umbenennen { .. } => ("Blatt umbenennen", "Wie soll das Blatt heißen?".to_string(), vec!["Umbenennen", texte::ABBRECHEN]),
        Frage::Wiederherstellen { titel, zeit } => (
            "Tabelle wiederherstellen?",
            format!(
                "SimpleGrid wurde beim letzten Mal nicht richtig beendet. Es gibt eine Sicherung von „{titel}“ vom {zeit}. Möchten Sie sie wiederherstellen?"
            ),
            vec!["Wiederherstellen", "Nicht wiederherstellen"],
        ),
    };
    let mut gewaehlt: Option<&str> = None;
    let mut neuer_name: Option<String> = None;
    egui::Modal::new(egui::Id::unique("simplegrid.rueckfrage"))
        .frame(
            egui::Frame::NONE
                .fill(egui::Color32::WHITE)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0xD1, 0xD5, 0xDB)))
                .inner_margin(egui::Margin::same(36))
                .corner_radius(18.0),
        )
        .show(ctx, |ui| {
            ui.set_width(640.0);
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(56.0, 56.0), egui::Sense::hover());
                ui.painter().circle(
                    r.center(),
                    27.0,
                    egui::Color32::from_rgb(0xFE, 0xF3, 0xC7),
                    egui::Stroke::new(3.0, egui::Color32::from_rgb(0xD9, 0x77, 0x06)),
                );
                ui.painter().text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    if matches!(frage, Frage::Umbenennen { .. } | Frage::Update { .. }) { "i" } else { "!" },
                    theme::ui_bold(32.0),
                    egui::Color32::from_rgb(0x92, 0x40, 0x0E),
                );
                ui.add_space(10.0);
                ui.label(RichText::new(titel).font(theme::ui_bold(30.0)));
            });
            ui.add_space(14.0);
            ui.add(egui::Label::new(RichText::new(&text).font(theme::ui_font(21.0))).wrap());
            if let Frage::Umbenennen { name, .. } = &frage {
                ui.add_space(12.0);
                let mut n = name.clone();
                let feld = ui.add(egui::TextEdit::singleline(&mut n).font(theme::ui_font(22.0)).desired_width(560.0).margin(vec2(12.0, 10.0)));
                if ui.memory(|m| m.focused().is_none()) {
                    feld.request_focus();
                }
                if n != *name {
                    neuer_name = Some(n);
                }
                if feld.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    gewaehlt = Some("Umbenennen");
                }
            }
            ui.add_space(20.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(14.0, 10.0);
                for (i, k) in knoepfe.iter().enumerate() {
                    if dialogknopf(ui, k, i) {
                        gewaehlt = Some(k);
                    }
                }
            });
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                gewaehlt = Some(texte::ABBRECHEN);
            }
        });
    if let (Some(n), Some(Frage::Umbenennen { name, .. })) = (neuer_name, app.einfach.frage.as_mut()) {
        *name = n;
    }
    let Some(wahl) = gewaehlt.map(str::to_string) else {
        return;
    };
    // Mit dem aktuellen Text antworten (beim Umbenennen der eingegebene Name).
    let frage = app.einfach.frage.clone().unwrap_or(frage);
    app.einfach.frage = None;
    match frage {
        Frage::Ungespeichert { danach } => match wahl.as_str() {
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
        Frage::Bestaetigen { knopf, befehl, params, .. } => {
            if wahl == knopf.as_str() {
                app.run_or_alert(&befehl, params);
            }
        }
        Frage::Update { .. } => {
            if wahl == "Jetzt aktualisieren" {
                neu_oder_oeffnen(app, Danach::Aktualisieren);
            }
        }
        Frage::Umbenennen { blatt, name } => {
            let name = name.trim().to_string();
            if wahl == "Umbenennen" && !name.is_empty() {
                app.run_or_alert("sheet.rename", json!({"sheet": blatt, "name": name}));
            }
        }
        Frage::Wiederherstellen { .. } => {
            if wahl == "Wiederherstellen" && !crate::sicherung::wiederherstellen(app) {
                app.message = Some((texte::HINWEIS_TITEL.into(), "Die Sicherung ließ sich leider nicht öffnen.".into()));
            }
            // Ab jetzt gehört die Sicherung dieser Sitzung (oder sie ist erledigt).
            app.einfach.sicherung.loeschen();
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
        let mut a = SheetApp::new(session, Services::default());
        a.ui.language = crate::i18n::Language::De;
        a
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
    fn neu_ersetzt_nur_die_mappe_im_aktuellen_fenster() {
        let mut a = app();
        let erstes = a.session.active_window();
        a.run_or_alert("file.new", json!({}));
        let zweites = a.session.active_window();
        assert_ne!(erstes, zweites);
        a.run_or_alert("cell.set", json!({"cell": "A1", "value": "x"}));
        let alt = a.session.active().map(|d| d.uid);
        ausfuehren(&mut a, Danach::Neu);
        assert_eq!(a.session.documents().len(), 2, "das andere Fenster bleibt");
        assert_eq!(a.session.active_window(), zweites, "gleiches Fenster");
        assert_ne!(a.session.active().map(|d| d.uid), alt, "neue Mappe darin");
        assert!(a.session.active().is_some_and(|d| !d.is_dirty()));
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
    fn eingeben_loeschen_rueckgaengig_wiederholen() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "Hallo"}));
        a.run_or_alert("selection.set", json!({"cell": "A1"}));
        a.run_or_alert("edit.clearContents", json!({}));
        assert_eq!(wert(&mut a, "A1"), Json::Null);
        a.run_or_alert("edit.undo", json!({}));
        assert_eq!(wert(&mut a, "A1"), json!("Hallo"));
        a.run_or_alert("edit.redo", json!({}));
        assert_eq!(wert(&mut a, "A1"), Json::Null);
    }

    #[test]
    fn fuenfzig_schritte_zurueck() {
        let mut a = app();
        for i in 1..=60 {
            a.run_or_alert("cell.set", json!({"cell": "A1", "input": i.to_string()}));
        }
        for _ in 0..50 {
            a.run_or_alert("edit.undo", json!({}));
        }
        assert_eq!(wert(&mut a, "A1"), json!(10.0));
        for _ in 0..50 {
            a.run_or_alert("edit.redo", json!({}));
        }
        assert_eq!(wert(&mut a, "A1"), json!(60.0));
    }

    #[test]
    fn kopieren_und_einfuegen_mit_format_auch_auf_anderes_blatt() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "12,5 €"}));
        a.run_or_alert("home.bold", json!({"range": "A1"}));
        a.run_or_alert("selection.set", json!({"range": "A1"}));
        a.run_or_alert("edit.copy", json!({}));
        a.run_or_alert("home.insertSheet", json!({"before": 1}));
        a.run_or_alert("selection.set", json!({"cell": "C3"}));
        a.run_or_alert("edit.paste", json!({}));
        assert_eq!(wert(&mut a, "C3"), json!(12.5));
        let st = a.session.execute("cell.get", json!({"cell": "C3"})).unwrap_or_default();
        assert_eq!(st["text"], json!("12,50 €"));
        assert_eq!(st["style"]["font"]["bold"], json!(true), "{st}");
        a.run_or_alert("edit.undo", json!({}));
        assert_eq!(wert(&mut a, "C3"), Json::Null);
    }

    #[test]
    fn ausschneiden_verschiebt() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "weg"}));
        a.run_or_alert("selection.set", json!({"range": "A1"}));
        a.run_or_alert("edit.cut", json!({}));
        a.run_or_alert("selection.set", json!({"cell": "B2"}));
        a.run_or_alert("edit.paste", json!({}));
        assert_eq!(wert(&mut a, "A1"), Json::Null);
        assert_eq!(wert(&mut a, "B2"), json!("weg"));
    }

    #[test]
    fn text_aus_anderem_programm_mit_tabulatoren() {
        let mut a = app();
        a.run_or_alert("selection.set", json!({"cell": "A1"}));
        a.paste_external(
            "1,5	Apfel
2.000	3,25 €"
                .into(),
        );
        assert_eq!(wert(&mut a, "A1"), json!(1.5));
        assert_eq!(wert(&mut a, "B1"), json!("Apfel"));
        assert_eq!(wert(&mut a, "A2"), json!(2000.0));
        assert_eq!(wert(&mut a, "B2"), json!(3.25));
    }

    #[test]
    fn bearbeitungstext_mit_komma() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "12,5"}));
        let v = a.session.execute("cell.get", json!({"cell": "A1"})).unwrap_or_default();
        assert_eq!(v["text"], json!("12,5"));
        let d = a.session.active().map(|d| {
            simplegrid_engine::display::edit_text(&d.wb, d.wb.active().unwrap_or_else(|| unreachable!()), simplegrid_engine::core::CellRef::new(0, 0))
        });
        assert_eq!(d.as_deref(), Some("12,5"));
    }

    fn formel(a: &mut SheetApp, zelle: &str) -> Json {
        a.session.execute("cell.get", json!({"cell": zelle})).map(|v| v["formula"].clone()).unwrap_or(Json::Null)
    }

    #[test]
    fn zeile_einfuegen_passt_formeln_an_und_laesst_sich_zuruecknehmen() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "1"}));
        a.run_or_alert("cell.set", json!({"cell": "A2", "input": "2"}));
        a.run_or_alert("cell.set", json!({"cell": "B1", "input": "=A1+A2"}));
        a.run_or_alert("selection.set", json!({"range": "A1:A2"}));
        a.run_or_alert("home.insertRows", json!({}));
        assert_eq!(formel(&mut a, "B3"), json!("=A3+A4"));
        assert_eq!(wert(&mut a, "B3"), json!(3.0));
        a.run_or_alert("edit.undo", json!({}));
        assert_eq!(formel(&mut a, "B1"), json!("=A1+A2"));
    }

    #[test]
    fn geloeschter_bezug_ergibt_bezugsfehler() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "5"}));
        a.run_or_alert("cell.set", json!({"cell": "C1", "input": "=A1*2"}));
        a.run_or_alert("selection.set", json!({"range": "A1"}));
        a.run_or_alert("home.deleteColumns", json!({}));
        assert_eq!(formel(&mut a, "B1"), json!("=#REF!*2"));
    }

    #[test]
    fn entfernen_mit_daten_fragt_ohne_daten_nicht() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A2", "input": "Daten"}));
        a.run_or_alert("selection.set", json!({"range": "A5:B6"}));
        entfernen(&mut a, true);
        assert!(a.einfach.frage.is_none(), "leere Zeilen werden ohne Rückfrage entfernt");
        a.run_or_alert("selection.set", json!({"range": "A2:A3"}));
        entfernen(&mut a, true);
        match &a.einfach.frage {
            Some(Frage::Bestaetigen { text, befehl, .. }) => {
                assert!(text.starts_with("Die Zeilen 2 bis 3 enthalten Daten"), "{text}");
                assert_eq!(befehl, "home.deleteRows");
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(wert(&mut a, "A2"), json!("Daten"), "noch nichts entfernt");
    }

    #[test]
    fn spalte_verschiebt_breite_verbund_und_format() {
        let mut a = app();
        a.run_or_alert("home.columnWidth", json!({"cols": "B", "chars": 20}));
        a.run_or_alert("cell.set", json!({"cell": "B1", "input": "x"}));
        a.run_or_alert("home.bold", json!({"range": "B1"}));
        a.run_or_alert("home.mergeCells", json!({"range": "B2:C2"}));
        let breite = |a: &SheetApp, c: u32| a.session.active().and_then(|d| d.wb.active().map(|s| s.col_width(c))).unwrap_or(0.0);
        let b = breite(&a, 1);
        a.run_or_alert("selection.set", json!({"range": "A1"}));
        a.run_or_alert("home.insertColumns", json!({}));
        assert_eq!(breite(&a, 2), b);
        assert_eq!(wert(&mut a, "C1"), json!("x"));
        let st = a.session.execute("cell.get", json!({"cell": "C1"})).unwrap_or_default();
        assert_eq!(st["style"]["font"]["bold"], json!(true));
        let merges = a.session.active().and_then(|d| d.wb.active().map(|s| s.merges.iter().map(|m| m.a1()).collect::<Vec<_>>())).unwrap_or_default();
        assert_eq!(merges, ["C2:D2"]);
    }

    #[test]
    fn spaltenbreite_und_zeilenhoehe_rueckgaengig() {
        let mut a = app();
        let breite = |a: &SheetApp| a.session.active().and_then(|d| d.wb.active().map(|s| s.col_width(0))).unwrap_or(0.0);
        let vorher = breite(&a);
        a.run_or_alert("home.columnWidth", json!({"cols": "A", "chars": 30}));
        assert!(breite(&a) > vorher);
        a.run_or_alert("edit.undo", json!({}));
        assert_eq!(breite(&a), vorher);
        a.run_or_alert("home.rowHeight", json!({"rows": "1", "points": 40}));
        let h = a.session.active().and_then(|d| d.wb.active().map(|s| s.row_height(0))).unwrap_or(0.0);
        assert!((h - 40.0 / 0.75).abs() < 1.0, "{h}");
    }

    #[test]
    fn euro_anzeige_und_umschalten() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "1234,56"}));
        a.run_or_alert("selection.set", json!({"range": "A1"}));
        euro_umschalten(&mut a, false);
        let text = |a: &mut SheetApp| a.session.execute("cell.get", json!({"cell": "A1"})).map(|v| v["text"].clone()).unwrap_or(Json::Null);
        assert_eq!(text(&mut a), json!("1.234,56 €"));
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "12,5"}));
        assert_eq!(text(&mut a), json!("12,50 €"), "Eingabe 12,5 erscheint als 12,50 €");
        assert_eq!(wert(&mut a, "A1"), json!(12.5), "intern bleibt es eine Zahl");
        euro_umschalten(&mut a, true);
        assert_eq!(text(&mut a), json!("12,5"));
    }

    #[test]
    fn umrandung_mit_dicke_und_farbe() {
        let mut a = app();
        a.run_or_alert("selection.set", json!({"range": "B2:C3"}));
        a.einfach.rahmen_dicke = "thick".into();
        a.einfach.rahmen_farbe = "#FF0000".into();
        umranden(&mut a, "outside");
        let st = a.session.execute("cell.get", json!({"cell": "B2"})).unwrap_or_default();
        assert_eq!(st["style"]["borders"]["top"]["style"], json!("Thick"), "{st}");
        assert_eq!(st["style"]["borders"]["left"]["style"], json!("Thick"));
        assert_eq!(st["style"]["borders"]["right"]["style"], json!("None"), "innen keine Linie");
        a.run_or_alert("edit.undo", json!({}));
        let st = a.session.execute("cell.get", json!({"cell": "B2"})).unwrap_or_default();
        assert_eq!(st["style"]["borders"]["top"]["style"], json!("None"));
    }

    #[test]
    fn verbinden_warnt_bei_datenverlust() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "oben"}));
        a.run_or_alert("selection.set", json!({"range": "A1:B1"}));
        verbinden(&mut a);
        assert!(a.einfach.frage.is_none(), "B1 ist leer: ohne Rückfrage");
        a.run_or_alert("home.unmergeCells", json!({}));
        a.run_or_alert("cell.set", json!({"cell": "B2", "input": "geht verloren"}));
        a.run_or_alert("selection.set", json!({"range": "A2:B2"}));
        verbinden(&mut a);
        assert!(matches!(&a.einfach.frage, Some(Frage::Bestaetigen { befehl, .. }) if befehl == "home.mergeCells"));
        assert_eq!(wert(&mut a, "B2"), json!("geht verloren"));
    }

    #[test]
    fn formate_ueberstehen_speichern_und_oeffnen() {
        let pfad = temp("formate.xlsx");
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "99,9"}));
        a.run_or_alert("selection.set", json!({"range": "A1:B2"}));
        for (id, p) in [
            ("home.bold", json!({})),
            ("home.italic", json!({})),
            ("home.underline", json!({})),
            ("home.fontSize", json!({"size": 16})),
            ("home.fontColor", json!({"color": "#0000FF"})),
            ("home.fillColor", json!({"color": "#FFFF00"})),
            ("home.alignCenter", json!({})),
        ] {
            a.run_or_alert(id, p);
        }
        euro_umschalten(&mut a, false);
        a.einfach.rahmen_dicke = "medium".into();
        umranden(&mut a, "all");
        a.run_or_alert("home.mergeCells", json!({"range": "C1:D1"}));
        a.run_or_alert("file.save", json!({"path": pfad}));
        let mut b = app();
        oeffnen_pfad(&mut b, &pfad);
        let st = b.session.execute("cell.get", json!({"cell": "A1"})).unwrap_or_default();
        let s = &st["style"];
        assert_eq!(s["font"]["bold"], json!(true));
        assert_eq!(s["font"]["italic"], json!(true));
        assert_eq!(s["font"]["underline"], json!("Single"));
        assert_eq!(s["font"]["size"], json!(16.0));
        assert_eq!(s["align"]["h"], json!("Center"));
        assert_eq!(s["borders"]["bottom"]["style"], json!("Medium"));
        assert_eq!(s["num_fmt"], json!(simplegrid_engine::core::locale::EURO_FORMAT));
        assert_eq!(st["text"], json!("99,90 €"));
        let merges = b.session.active().and_then(|d| d.wb.active().map(|s| s.merges.iter().map(|m| m.a1()).collect::<Vec<_>>())).unwrap_or_default();
        assert_eq!(merges, ["C1:D1"]);
    }

    #[test]
    fn zoom_stufen() {
        let mut a = app();
        a.run_or_alert("view.zoom", json!({"percent": 300}));
        a.run_or_alert("view.zoom", json!({"percent": 50}));
        let z = a.session.active().and_then(|d| d.wb.active().map(|s| s.zoom)).unwrap_or_default();
        assert_eq!(z, 50);
        assert!(ZOOMSTUFEN.contains(&50) && ZOOMSTUFEN.contains(&300));
    }

    #[test]
    fn deutsche_formeln_werden_gespeichert_wie_excel_sie_kennt() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "1,5"}));
        a.run_or_alert("cell.set", json!({"cell": "A2", "input": "2"}));
        let deutsch = |a: &SheetApp, t: &str| {
            let d = a.session.active().unwrap_or_else(|| unreachable!());
            crate::formula_locale::canonical(t, simplegrid_engine::formula::FormulaLocale::De, &d.wb, 0, simplegrid_engine::core::CellRef::new(0, 2))
                .unwrap_or_default()
        };
        let f = deutsch(&a, "=SUMME(A1:A2;0,5)*2");
        assert_eq!(f, "=SUM(A1:A2,0.5)*2");
        a.run_or_alert("cell.set", json!({"cell": "C1", "input": f}));
        assert_eq!(wert(&mut a, "C1"), json!(8.0));
        let d = a.session.active().unwrap_or_else(|| unreachable!());
        let shown = crate::formula_locale::display("=SUM(A1:A2,0.5)*2", simplegrid_engine::formula::FormulaLocale::De, &d.wb, 0);
        assert_eq!(shown, "=SUMME(A1:A2;0,5)*2");
        assert_eq!(deutsch(&a, "=#BEZUG!+1"), "=#REF!+1");
    }

    #[test]
    fn punkt_vor_strich_klammern_und_fehler() {
        let mut a = app();
        for (zelle, eingabe, erwartet) in [
            ("B1", "=2+3*4", json!(14.0)),
            ("B2", "=(2+3)*4", json!(20.0)),
            ("B3", "=10/4", json!(2.5)),
            ("B4", "=-2^2", json!(4.0)),
            ("B5", "=1/0", json!({"error": "#DIV/0!"})),
            ("B6", "=B5+1", json!({"error": "#DIV/0!"})),
            ("B7", "=GIBTSNICHT(1)", json!({"error": "#NAME?"})),
            ("B8", "=\"a\"*2", json!({"error": "#VALUE!"})),
        ] {
            a.run_or_alert("cell.set", json!({"cell": zelle, "input": eingabe}));
            let v = a.session.execute("cell.get", json!({"cell": zelle})).unwrap_or_default();
            assert_eq!(v["value"], erwartet, "{zelle} {eingabe}");
        }
        let text = |a: &mut SheetApp, c: &str| a.session.execute("cell.get", json!({"cell": c})).map(|v| v["text"].clone()).unwrap_or(Json::Null);
        assert_eq!(text(&mut a, "B8"), json!("#WERT!"));
        a.run_or_alert("selection.set", json!({"cell": "B5"}));
        let erkl = fehler_erklaerung(&a).unwrap_or_default();
        assert!(erkl.contains("Durch Null geteilt"), "{erkl}");
    }

    #[test]
    fn zirkelbezug_haengt_nicht() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "=B1+1"}));
        a.run_or_alert("cell.set", json!({"cell": "B1", "input": "=A1+1"}));
        let v = a.session.execute("cell.get", json!({"cell": "A1"})).unwrap_or_default();
        assert_eq!(v["text"], json!("#ZIRKEL!"));
        a.run_or_alert("selection.set", json!({"cell": "A1"}));
        assert!(fehler_erklaerung(&a).is_some_and(|t| t.contains("Zirkelbezug")));
    }

    #[test]
    fn neuberechnung_und_bezugsaenderung() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "5"}));
        a.run_or_alert("cell.set", json!({"cell": "A2", "input": "=A1*2"}));
        a.run_or_alert("cell.set", json!({"cell": "A3", "input": "=A2+$A$1"}));
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "10"}));
        assert_eq!(wert(&mut a, "A3"), json!(30.0));
        a.run_or_alert("selection.set", json!({"range": "A1"}));
        a.run_or_alert("home.insertRows", json!({}));
        assert_eq!(formel(&mut a, "A4"), json!("=A3+$A$2"));
        a.run_or_alert("selection.set", json!({"range": "A3"}));
        a.run_or_alert("home.deleteRows", json!({}));
        assert_eq!(wert(&mut a, "A3"), json!({"error": "#REF!"}));
        let text = a.session.execute("cell.get", json!({"cell": "A3"})).map(|v| v["text"].clone()).unwrap_or(Json::Null);
        assert_eq!(text, json!("#BEZUG!"));
    }

    #[test]
    fn summe_unter_markiertem_bereich_und_rechts_bei_zeile() {
        let mut a = app();
        for (c, v) in [("A1", "1"), ("A2", "2"), ("A3", "3"), ("C1", "4"), ("D1", "5")] {
            a.run_or_alert("cell.set", json!({"cell": c, "input": v}));
        }
        a.run_or_alert("selection.set", json!({"range": "A1:A3"}));
        summe(&mut a);
        assert!(a.commit_edit(0, 0, false, false));
        assert_eq!(formel(&mut a, "A4"), json!("=SUM(A1:A3)"));
        assert_eq!(wert(&mut a, "A4"), json!(6.0));
        a.run_or_alert("selection.set", json!({"range": "C1:D1"}));
        summe(&mut a);
        assert!(a.commit_edit(0, 0, false, false));
        assert_eq!(wert(&mut a, "E1"), json!(9.0));
    }

    #[test]
    fn summe_erkennt_zahlenspalte_von_einer_zelle_aus() {
        let mut a = app();
        for (c, v) in [("B1", "Ausgaben"), ("B2", "10"), ("B3", "20,5"), ("B4", "30")] {
            a.run_or_alert("cell.set", json!({"cell": c, "input": v}));
        }
        a.run_or_alert("selection.set", json!({"cell": "B3"}));
        summe(&mut a);
        assert!(a.commit_edit(0, 0, false, false));
        assert_eq!(formel(&mut a, "B5"), json!("=SUM(B2:B4)"));
        assert_eq!(wert(&mut a, "B5"), json!(60.5));
        a.run_or_alert("cell.set", json!({"cell": "D1", "input": "1"}));
        a.run_or_alert("cell.set", json!({"cell": "D2", "input": "2"}));
        a.run_or_alert("selection.set", json!({"cell": "D3"}));
        summe(&mut a);
        assert!(a.commit_edit(0, 0, false, false));
        assert_eq!(wert(&mut a, "D3"), json!(3.0));
    }

    #[test]
    fn summe_wird_zum_bestaetigen_vorgelegt_und_laesst_sich_verwerfen() {
        let mut a = app();
        for (c, v) in [("B1", "10"), ("B2", "20")] {
            a.run_or_alert("cell.set", json!({"cell": c, "input": v}));
        }
        a.run_or_alert("selection.set", json!({"range": "B1:B2"}));
        summe(&mut a);
        let ed = a.editor.as_ref().map(|e| (e.cell.a1(), e.text.clone()));
        assert_eq!(ed, Some(("B3".to_string(), "=SUMME(B1:B2)".to_string())), "Formel deutsch vorgelegt");
        assert_eq!(wert(&mut a, "B3"), Json::Null, "noch nichts eingetragen");
        assert_eq!(a.einfach.summe_ziel.map(|c| c.a1()).as_deref(), Some("B3"));
        a.cancel_edit();
        assert!(a.editor.is_none());
        assert_eq!(wert(&mut a, "B3"), Json::Null, "rotes Kreuz / Esc: nichts eingetragen");
    }

    #[test]
    fn summe_fuer_mehrere_spalten_rechnet_direkt() {
        let mut a = app();
        for (c, v) in [("A1", "1"), ("A2", "2"), ("B1", "3"), ("B2", "4")] {
            a.run_or_alert("cell.set", json!({"cell": c, "input": v}));
        }
        a.run_or_alert("selection.set", json!({"range": "A1:B2"}));
        summe(&mut a);
        assert!(a.editor.is_none());
        assert_eq!(wert(&mut a, "A3"), json!(3.0));
        assert_eq!(wert(&mut a, "B3"), json!(7.0));
    }

    #[test]
    fn summe_wird_in_xlsx_englisch_gespeichert() {
        let pfad = temp("summe.xlsx");
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "A1", "input": "1"}));
        a.run_or_alert("cell.set", json!({"cell": "A2", "input": "=SUM(A1)"}));
        a.run_or_alert("file.save", json!({"path": pfad}));
        let bytes = std::fs::read(&pfad).unwrap_or_default();
        let (wb, _) = simplegrid_xlsx::read_xlsx(&bytes).unwrap_or_else(|_| unreachable!());
        let f = wb.sheets[0].cell(simplegrid_engine::core::CellRef::new(1, 0)).and_then(|c| c.formula.clone()).map(|f| f.text.clone());
        assert_eq!(f.as_deref(), Some("SUM(A1)"));
    }

    #[test]
    fn grosse_entfernte_zelle() {
        let mut a = app();
        a.run_or_alert("cell.set", json!({"cell": "Z5000", "input": "fern"}));
        let v = a.session.execute("cell.get", json!({"cell": "Z5000"})).unwrap_or_default();
        assert!(v.to_string().contains("fern"), "{v}");
    }
}
