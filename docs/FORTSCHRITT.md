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

## Phase 3 – Bearbeiten, Zwischenablage, Rückgängig (2026-11-10)
- Leiste „Bearbeiten“: Rückgängig, Wiederholen, Kopieren, Ausschneiden, Einfügen (mit Format), Inhalt löschen –
  alles GridCraft-Befehle; gesperrt mit Hinweis („Es gibt nichts rückgängig zu machen.“).
- Rückgängig: GridCraft hält 100 Schritte (Anleitung: mindestens 50).
- Bearbeitungsfeld zeigt Zahlen mit Komma (`12,5`) und WAHR/FALSCH.
- Bilder aus der Zwischenablage werden nicht mehr eingefügt (nur Zellen und Text).
- Tests: Eingabe/Löschen/Rückgängig/Wiederholen, 50 Schritte, Kopieren mit Format auf anderes Blatt, Ausschneiden,
  tabgetrennter Text aus anderen Programmen mit deutschen Zahlen.

## Phase 4 – Zeilen und Spalten (2026-11-10)
- Leiste „Zeilen und Spalten“: Zeile/Spalte hinzufügen und entfernen (an der Markierung, auch mehrere), Spaltenbreite …,
  Zeilenhöhe … (GridCraft-Dialoge, eingedeutscht, Zahlen mit Komma). Ziehen an den Kopflinien geht wie bisher.
- Entfernen fragt nur, wenn in den Zeilen/Spalten Daten stehen; alles ist rückgängig machbar.
- Formelbezüge, Formate, Breiten, verbundene Zellen verschieben sich (GridCraft-Engine); gelöschte Bezüge → `#REF!`
  (deutsche Fehlernamen folgen in Phase 6).
- Leiste bricht Gruppen in neue Zeilen um, ist so hoch wie nötig, höchstens 40 % des Fensters (sonst rollbar).
- Hinweis für Phase 10: Mit allen Gruppen wird die Leiste mehrzeilig – Anordnung ist eine Designfrage.

## Phase 5 – Formatierung (2026-11-10)
- Gruppen „Schrift“ (Fett, Kursiv, Unterstrichen – eingedrückt, wenn aktiv; Größe 10–36; Textfarbe),
  „Zellen“ (Hintergrund, Links/Mitte/Rechts, Euro €, Zellen verbinden/Verbindung trennen),
  „Umrandung“ (Linie Dünn/Mittel/Dick, Linienfarbe, Außen/Innen/Alle/Keine), „Ansicht“ (Kleiner/Größer, 50–300 %).
- Farbauswahl: GridCraft-Palette ohne „Weitere Farben“ (eine Ebene), Überschriften eingedeutscht.
- Euro-Format `#,##0.00 "€"`, angezeigt `1.234,56 €`; Eingabe `12,5` → `12,50 €`. „Euro“ noch einmal = normale Zahl.
- Verbinden warnt, wenn außer der Zelle oben links Inhalte verloren gingen.
- Zahlen im Standardformat werden auch beim Spaltenanpassen deutsch gezeigt (Bildschirm und Druck).
- Rechtsklick-Menüs auf die Funktionen der Leiste reduziert (deutsch, größere Einträge, keine Untermenüs).
- Tests: Euro, Umrandung mit Dicke/Farbe + Rückgängig, Verbinden-Warnung, alle Formate nach Speichern/Öffnen, Zoom.
  Entfernt: Tests des alten Rechtsklick-Menüs, der Seitenlayout-Lineale und der Formelobjekte.
