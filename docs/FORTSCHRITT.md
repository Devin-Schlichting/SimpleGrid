# Fortschritt SimpleGrid

## Phase 0 – Gerüst (2026-11-10)
- GridCraft (Stand `50a9e9ed`, MIT OR Apache-2.0) ohne Git-Historie übernommen; Lizenz, NOTICE, ATTRIBUTION bleiben.
- Umbenannt: Crates `gridcraft-*` → `simplegrid-*`, Programm `simplegrid.exe`, Texte „GridCraft“ → „SimpleGrid“
  (außer im Herkunftshinweis im Info-Fenster).
- Entfernt: Web-App, Kommandozeilen-App, `xtask`, Linux/Mac/FreeBSD/Web-Pakete, storytold-CI, ArtCraft-Marken,
  CodeRabbit/Sonar-Konfiguration. Alte Dokumente liegen unter `docs/herkunft-gridcraft/`.
- Behalten (vorerst): Engine mit Diagrammen, Pivot, Makros usw. – nicht sichtbar, nur nicht entfernt, weil die
  Engine eng verzahnt ist. Der MCP-Crate bleibt für den Steuerkanal (Tests/Screenshots).
- Abhängigkeiten von `storytold/craft-libs` (Update, Fenster, Druck) bleiben vorerst; Update-Quelle wird in Phase 9 ersetzt.
- Test `complex_numbers`: letzte Nachkommastelle unter Windows anders – Prüfung gelockert.

## Phase 1 – Grundansicht (2026-11-10)
- Menüband, Titelleiste mit Schnellzugriff, Backstage-Startseite und Alt-Tastentipps sind durch die feste
  Schaltflächenleiste `crates/ui-egui/src/einfach.rs` ersetzt (große, beschriftete Schaltflächen, gesperrt mit Hinweis).
- Gruppen: „Datei“ (Neu) und „Tabellenblatt“ (hinzufügen, umbenennen, löschen mit Rückfrage, nach links/rechts).
- „Neu“ fragt bei ungespeicherten Änderungen „Speichern / Nicht speichern / Abbrechen“ und ersetzt die Mappe
  (SimpleGrid zeigt immer genau eine Mappe).
- Oberfläche fest auf Deutsch; neue Blätter heißen „Tabelle1…“, neue Mappen „Mappe1…“.
- Tastatur: nur noch Strg+C/X/V/Z/Y/S sowie Pfeile, Bild auf/ab, Pos1, Enter, Tab, Entf, Rücktaste, Esc.
- Tests: 490 Oberflächentests, die das entfernte Menüband/Kürzel bedienten, und die Arabisch/CJK/Tastentipp-Tests
  gelöscht. 3092 Tests grün, keine Warnungen.
- Offen/Hinweis: Phase 0 und 1 sind in einem Commit gelandet, weil die Umstellung der Oberfläche den Testlauf
  von Phase 0 bereits beeinflusst hatte.

## Phase 2 – Speichern, Öffnen, Import (2026-11-10)
- Leiste „Datei“: Neu, Öffnen, Speichern, Speichern unter. „Öffnen“ ersetzt die bisherige Tabelle (vorher Rückfrage
  bei ungespeicherten Änderungen); wird keine Datei gewählt, bleibt alles, wie es war.
- Dateiauswahl nur noch „Tabellen (.xlsx, .ods)“ beim Öffnen und „Tabelle (.xlsx)“/„PDF-Dokument“ beim Speichern
  (kein „Excel“ mehr in der Oberfläche).
- Eine .ods-Tabelle wird nie als .ods überschrieben: „Speichern“ fragt nach einem .xlsx-Namen.
- Kaputte oder fremde Dateien: deutsche Meldung „Datei lässt sich nicht öffnen …“; die bisherige Tabelle bleibt.
  (GridCraft las Nicht-ZIP-Dateien als Text – SimpleGrid prüft vorher Endung und Inhalt.)
- .ods-Import ausgebaut (`crates/xlsx/src/ods_styles.rs`): Formeln (OpenFormula → interne Schreibweise, auch über
  Blätter), fett/kursiv/unterstrichen, Schriftgröße, Text- und Hintergrundfarbe, Ausrichtung, Umrandung, Euro-Format,
  Spaltenbreiten, Zeilenhöhen. Diagramme/Bilder/Filter/bedingte Formate werden ausgelassen; deutscher Hinweis.
- Deutsche Zahlenschreibweise (`crates/core/src/locale.rs`): Eingabe `12,5`, `1.234,56`, `12,50 €`, `7,5 %`,
  `10.11.2026`, `WAHR`/`FALSCH`; Anzeige `1.234,56 €`. Gespeichert wird weiter neutral (Dateien wie bisher).
- Tests: Round-Trip xlsx, ods mit Formaten → xlsx, kaputte Datei, deutsche Zahlen. 3106 Tests grün.
- Offen: Beispieldateien des Opas zum Prüfen des Imports.
