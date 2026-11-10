# Anleitung: SimpleGrid – Tabellenprogramm für Senioren (Windows)

Arbeitsanweisung für die Entwicklung. Ursprünglich als C#/WPF-Neubau „SimpleOffice Tabellen“
geschrieben. Am 2026-11-10 hat der Auftraggeber entschieden, stattdessen das Open-Source-Projekt
**GridCraft** (Rust/egui, MIT OR Apache-2.0) umzubauen. Die Abschnitte unten sind darauf angepasst.

## Verbindliche Abweichungen von der ursprünglichen Anleitung

| Thema | Ursprünglich | Gilt jetzt |
|---|---|---|
| Technik | C#/WPF, .NET 8, MVVM | Rust + egui (aus GridCraft übernommen) |
| Name | SimpleOffice Tabellen | **SimpleGrid** (Programm, Fenster, Installer, Verknüpfung) |
| Repository | Devin-Schlichting/SimpleOffice | **Devin-Schlichting/SimpleGrid** (öffentlich), Ordner `D:\GitHub\SimpleGrid` |
| Git-Historie | – | Frischer Start, ohne GridCraft-Historie; Lizenz/NOTICE von GridCraft bleiben |
| Gemeinsame Bibliotheken `Suite.*` | C#-Projekte | Rust-Crates; was eine spätere Textverarbeitung braucht, kommt in eigene, UI-unabhängige Crates |
| xlsx/ods | ClosedXML / eigener ods-Leser | GridCraft-Crates `simplegrid-xlsx` (xlsx, ods-Import vorhanden) |
| Formeln | eigener kleiner Parser | GridCraft-Formel-Engine; sichtbar nur, was hier verlangt ist |
| Installer | Inno Setup | **Inno Setup** (der GridCraft-WiX-Installer wird ersetzt) |
| Branding | ZIP im Repo | ZIP wird vom Auftraggeber **später nachgelegt** (Logo, Farben). Bis dahin neutrales Platzhalter-Symbol |
| Arbeitsrhythmus | nach jeder Phase warten | **Phasen am Stück**, je Phase ein Commit, Prüfung am Ende |

Grundidee des Umbaus: Die Rechen-, Datei- und Druck-Engine von GridCraft bleibt (sie ist ausgereift
und getestet). Die Oberfläche wird radikal vereinfacht: statt Menüband/Ribbon eine feste,
beschriftete Schaltflächenleiste mit genau den Funktionen unten. Alles andere wird entfernt oder
unsichtbar gemacht.

## 1. Ziel und Rahmen
- Windows-Programm, das wie eine Tabellenkalkulation funktioniert, aber nur die unten genannten
  Funktionen zeigt. Keine verschachtelten Menüs, keine unnötigen Funktionen.
- Für eine bestimmte ältere Person. Im Zweifel: weniger Funktionen, dafür eindeutig und fehlertolerant.
- Alles lokal, ohne Internet (Ausnahme: Update-Suche). Nur Deutsch (Oberfläche, Zahlenformat, Dezimalkomma).
- Die Namen „Excel“ und „Word“ oder Anklänge daran dürfen in der Oberfläche nirgends vorkommen.
  (Dateiformat-Bezeichnungen wie „.xlsx“ sind erlaubt.)
- Ausblick: später eine einfache Textverarbeitung als Schwesterprogramm. Gemeinsames (Rückgängig,
  Einstellungen, Autosicherung, Update, Design, Dialoge, Druckvorschau) möglichst wiederverwendbar halten.

## 2. Bedienkonzept
- Alles per beschrifteter Schaltfläche in einer festen Leiste oben (Text, nicht nur Symbol).
- Keine verschachtelten Menüs. Varianten höchstens als kleines Auswahlfeld mit wenigen großen Einträgen.
- Erlaubte Tastenkürzel ausschließlich: `Strg+C`, `Strg+X`, `Strg+V`, `Strg+Z`, `Strg+Y`, `Strg+S`,
  dazu normale Bedienung (Pfeiltasten, Enter, Tab, Entf, Eingabe). Alle anderen Kürzel von GridCraft
  werden abgeschaltet. Keine Funktion nur per Tastatur.
- Jede Funktion ist ein Befehl der Engine; Schaltfläche und Kürzel rufen denselben Befehl.
  Nicht mögliche Aktionen: Schaltfläche ausgegraut mit Hinweistext.
- Große Klickziele (zentral einstellbar), deutliche Beschriftung, sichtbarer Fokus.
- Zerstörerische Aktionen vorher bestätigen; alles rückgängig machbar.

## 3. Funktionsumfang (Phasen)
Jede Phase: Funktion in der vereinfachten Oberfläche verfügbar, durch Tests belegt, Commit.

- **Phase 0 – Gerüst:** GridCraft übernehmen, umbenennen, nicht benötigte Teile entfernen
  (Web, Kommandozeile, Linux/Mac-Pakete, fremde CI), Bau und Tests grün.
- **Phase 1 – Grundansicht:** feste Schaltflächenleiste statt Ribbon; Raster mit Zeilen-/Spaltenköpfen;
  Blätter als Reiter unten mit Schaltflächen Hinzufügen, Löschen (Rückfrage, nicht das letzte),
  Umbenennen, Verschieben; Schaltfläche „Neu“. Nur Deutsch.
- **Phase 2 – Dateien:** „Speichern“, „Speichern unter“ (.xlsx), „Öffnen“ (.xlsx, Import .ods).
  Defekte Dateien: verständliche deutsche Meldung, nie Absturz. Nicht Unterstütztes beim Import melden.
- **Phase 3 – Bearbeiten:** Auswahl, Eingabe, Entf; „Kopieren“, „Ausschneiden“, „Einfügen“ (auch mit
  anderen Programmen, tabgetrennt); „Rückgängig“/„Wiederholen“ (mindestens 50 Schritte).
- **Phase 4 – Zeilen/Spalten:** hinzufügen/entfernen (auch mehrere), Formelbezüge passen sich an
  (`#BEZUG!` bei gelöschten Bezügen); Spaltenbreite/Zeilenhöhe per Eingabe und Ziehen.
- **Phase 5 – Formatierung:** fett, kursiv, unterstrichen, Textfarbe, Schriftgröße (feste Stufen),
  Hintergrundfarbe, Ausrichtung links/mittig/rechts, Umrandung (außen/innen/alle/keine; dünn/mittel/dick;
  Farbe), Euro-Format `1.234,56 €`, Zellen verbinden/trennen (Warnung bei Inhaltsverlust),
  Zoom 50–300 % per „Größer“/„Kleiner“.
- **Phase 6 – Formeln:** Eingabe deutsch (`SUMME`, Dezimalkomma, `;` als Trenner), gespeichert in
  englischer Schreibweise. Schaltfläche „Summe“ (Bereich darunter/rechts, automatische Bereichserkennung).
  Fehlerwerte mit Klartext-Erklärung (z. B. „Durch Null geteilt“), Zirkelbezug `#ZIRKEL!`-Hinweis.
- **Phase 7 – Drucken:** „Druckbereich festlegen/aufheben“, Seitenansicht mit Blättern und Zoom,
  Hoch/Quer, „Auf eine Seite anpassen“, Seitenzahlen unten, Farbe/Schwarz-Weiß, Umbruch nur an
  Zellgrenzen.
- **Phase 8 – Schutz vor Fehlern:** Rückfrage „Speichern / Nicht speichern / Abbrechen“; automatische
  Sicherung alle 2–3 Minuten und Wiederherstellung nach Absturz; zuletzt geöffnete Datei beim Start
  (abschaltbar); freundliche Fehlermeldungen, nie einfach schließen.
- **Phase 9 – Installer und Updates:** Inno Setup, pro Benutzer ohne Adminrechte, nach
  `%LocalAppData%\Programs\SimpleGrid\`, Komponenten-Aufbau (später Textprogramm), Verknüpfungen
  Desktop/Startmenü, Dateizuordnung .xlsx optional (Standard aus), saubere Deinstallation, deutsch.
  Update-Suche über GitHub-Releases von `Devin-Schlichting/SimpleGrid` (höchstens 1× täglich,
  still ohne Netz, `versionen.json` mit SHA-256, nur nach Zustimmung, vorher Speichern-Rückfrage).
  Einstellung zum Abschalten, Schaltfläche „Nach Updates suchen“. Niemals Zugangsschlüssel einchecken.
  Release-Skript/GitHub-Aktion mit Version per Git-Tag.
- **Phase 10 – Design:** erst nach Freigabe, mit dem Branding-Paket des Auftraggebers. Entwürfe zeigen
  (mehrere Varianten), dann zentral umsetzen. Große Schrift, hoher Kontrast, große Schaltflächen.

## 4. Arbeitsweise
- Nach jeder Phase: `cargo build`, `cargo test`, kurzer Eintrag in `docs/FORTSCHRITT.md`, Commit.
- Keine zusätzlichen Funktionen (Sortieren, Filter, Diagramme, weitere sichtbare Formeln, weitere Kürzel).
  GridCraft-Funktionen, die nicht in der Liste stehen, werden nicht angezeigt.
- Texte der Oberfläche in einfachen Alltagswörtern, zentral abgelegt.
- Bezeichner im Code Englisch (wie GridCraft), Kommentare zu neuem Code und alles Sichtbare Deutsch.
- Keine echten Daten des Opas, keine Zugangsdaten im öffentlichen Repository.
- Lizenz: GridCraft-Hinweise (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, `ATTRIBUTION.md`) bleiben erhalten.
  Die ArtCraft-Markendateien sind entfernt und dürfen nicht zurückkommen.

## 5. Noch offen (mit dem Auftraggeber klären)
- Branding-Paket (Logo, Farben, Schrift)
- Beispieldateien (.ods) zum Testen des Imports
- Welche Formelfunktionen außer SUMME genutzt werden
- Typischer Drucker, Bildschirmgröße, Windows-Skalierung
- Signaturzertifikat für den Installer

## 6. Abnahme-Checkliste (am Ende)
- [ ] Neue Datei, Tabellenblätter anlegen/umbenennen/löschen
- [ ] Zeilen und Spalten einfügen/löschen, Formeln passen sich an
- [ ] Summen-Button, eigene Formeln, automatische Neuberechnung
- [ ] Zellfarbe, Textfarbe, fett, kursiv, unterstrichen, Schriftgröße, Ausrichtung
- [ ] Umrandung mit Dicke und Farbe
- [ ] Euro-Format korrekt angezeigt und gespeichert
- [ ] Kopieren/Ausschneiden/Einfügen per Schaltfläche und Strg+C/X/V, auch zwischen Blättern und Programmen
- [ ] Rückgängig/Wiederholen per Schaltfläche, mindestens 50 Schritte
- [ ] Spaltenbreite, Zeilenhöhe, Zoom, Zellen verbinden
- [ ] Speichern, Speichern unter, Öffnen (.xlsx), Import (.ods)
- [ ] Druckbereich, Seitenansicht, Hoch/Quer, Auf eine Seite, Seitenzahlen, Farbe/Schwarz-Weiß
- [ ] Jede Funktion per Schaltfläche, keine weiteren Tastenkürzel
- [ ] Rückfrage beim Schließen, automatische Sicherung, Wiederherstellung
- [ ] Installer ohne Adminrechte, saubere Deinstallation, Komponenten
- [ ] Update-Suche funktioniert, ohne Internet bleibt alles still
- [ ] Alle Tests grün, Build ohne Warnungen
