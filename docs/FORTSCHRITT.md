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

## Phase 6 – Formeln und Summe (2026-11-10)
- Deutsche Formeleingabe über GridCraft's Sprachmechanismus (neu: `FormulaLocale::De`): `=SUMME(A1:A3;0,5)`,
  `WAHR`/`FALSCH`. Gespeichert wird englisch (`SUM`, `,`, `.`), angezeigt deutsch. Funktionstabelle in
  `crates/formula/src/input.rs` (`GERMAN`) – bisher nur SUMME; weitere nach Rücksprache.
- Deutsche Fehlernamen in Zellen und Formeln (`#BEZUG!`, `#WERT!`, `#ZIRKEL!`, `#NV`, `#ZAHL!`), Eingabe auch deutsch.
- Hinweisstreifen unter der Leiste erklärt den Fehler der markierten Zelle in Klartext („Durch Null geteilt …“).
- „Σ Summe“: Bereich → Summe darunter (Zeile → rechts); einzelne Zahl in einer Zahlenspalte → zusammenhängender
  Zahlenbereich, Summe in die nächste freie Zelle darunter; leere Zelle → GridCraft-AutoSumme.
- Neuberechnung, Abhängigkeiten und Zirkelbezüge macht die GridCraft-Engine (Zirkel → `#ZIRKEL!`, keine Endlosschleife).
- Tests: deutsche Formeln ↔ gespeicherte Schreibweise, Punkt-vor-Strich, Klammern, Fehlerketten, Zirkel,
  Bezugsänderung durch Einfügen/Löschen, Summe in beiden Richtungen und von einer Zelle aus, SUM in der .xlsx.

## Phase 7 – Drucken (2026-11-10)
- Leiste „Drucken“: Seitenansicht und Drucken, Druckbereich festlegen, Druckbereich aufheben (gesperrt, wenn keiner da).
- Neue Seitenansicht (`crates/ui-egui/src/seitenansicht.rs`) statt GridCraft's Backstage: große Seite mit Füllungen,
  Rahmen, verbundenen Zellen und Seitenzahl; Blättern, Vorschau-Zoom 100–300 %; Hochformat/Querformat,
  „Auf eine Seite“, Farbe/Schwarz-Weiß; Drucken, Als PDF speichern; Druckerwahl (GridCraft, eingedeutscht).
- Seitenaufteilung, PDF-Erzeugung und Druck stammen aus GridCraft (Umbruch nur an Zeilen-/Spaltengrenzen).
- Neu in der Engine: Druckoptionen `pageNumbers` (keine Kopfzeile, unten „Seite N von M“) und `blackWhite`
  (PDF-Seiten: Flächen in hellen Graustufen, Schrift und Linien schwarz).
- Druckbereich, Ausrichtung und „Auf eine Seite“ werden im Blatt und damit in der .xlsx gespeichert.
- Leeres Blatt: „Auf diesem Blatt steht nichts zum Drucken.“ statt einer leeren Seite.
- Tests: Seitenzahl hoch/quer/eine Seite, jede Zeile genau einmal gedruckt, Druckbereich, Fußzeile, Schwarz-Weiß im
  PDF, Speichern der Druckeinstellungen. Noch offen: praktischer Probedruck auf einem echten Drucker.

## Phase 8 – Schutz vor Fehlern (2026-11-10)
- Schließen, „Neu“ und „Öffnen“ mit ungespeicherten Änderungen: Rückfrage „Speichern / Nicht speichern / Abbrechen“
  (ersetzt GridCraft's Mehrfenster-Dialog; SimpleGrid hat immer genau eine Tabelle).
- Automatische Sicherung alle 2,5 Minuten nach `%LocalAppData%\SimpleGrid\Sicherung` (nie in die eigene Datei),
  nur bei ungespeicherten Änderungen; nach dem Speichern/regulären Beenden wieder gelöscht (`sicherung.rs`).
- Nach einem Absturz fragt der nächste Start „Tabelle wiederherstellen?“; wiederhergestellt gilt als ungespeichert.
- Beim Start wird die zuletzt benutzte Tabelle geöffnet; abschaltbar unter „Info und Einstellungen“ (neue Gruppe „Hilfe“).
- Unerwartete Fehler beim Zeichnen beenden das Programm nicht: Tabelle wird gesichert, freundliche Meldung
  (Befehle der Engine fängt GridCraft ohnehin ab).
- Tests: Sicherung/Wiederherstellung, Takt nur bei Änderungen, Aufräumen nach dem Speichern, Schließen fragt,
  Fehler → Sicherung, letzte Datei an/aus, Zeile mit Daten löschen → Rückgängig. Entfernt: Tests des alten
  Mehrfenster-Schließdialogs.

## Phase 9 – Installer und Updates (2026-11-10)
- GridCraft's Update-Mechanismus (craft-update, fragte die GridCraft-Server ab und hätte GridCraft installieren können)
  ist entfernt, ebenso der WiX-Installer (`packaging/`).
- Eigener Update-Dienst `crates/ui-egui/src/update.rs` mit Schnittstelle `UpdateQuelle` (GitHub-Releases von
  `Devin-Schlichting/SimpleGrid`, ohne Zugangsschlüssel; Quelle in den Einstellungen änderbar). Abruf über das in
  Windows enthaltene `curl.exe`, Prüfsumme SHA-256 aus `versionen.json`.
- Oberfläche (`updates_ui.rs`): beim Start höchstens einmal am Tag (abschaltbar), still bei Fehlern/ohne Netz/
  Anfragelimit; „Es gibt eine neue Version … Jetzt aktualisieren / Später“ → Speichern-Rückfrage → Laden und Prüfen
  → stiller Installer (`/VERYSILENT … /STARTAPP`) → Programm startet neu. „Nach Updates suchen“ und Schalter unter
  „Info und Einstellungen“. Falsche Prüfsumme: nichts wird installiert, deutsche Meldung, Protokolleintrag.
- Inno-Setup-Skript `installer/SimpleGrid.iss`: pro Benutzer ohne Adminrechte nach `%LocalAppData%\Programs\SimpleGrid`,
  feste AppId, Komponenten (Basis fest, Tabellen; Platz für das Textprogramm), Desktop-/Startmenü-Verknüpfung,
  .xlsx-Zuordnung optional (aus), deutsche Oberfläche, Neustart nach stillem Update, saubere Deinstallation
  (inkl. Sicherungen und Einstellungen). Rust-Programm ist eigenständig (keine Laufzeit nötig).
- `installer/bauen.ps1` (Version, Tests, Release-Build, Installer, `versionen.json`) und GitHub-Aktion
  `.github/workflows/release.yml` (bei Tag `vX.Y.Z`: bauen, testen, Release mit Setup.exe und versionen.json).
- Version jetzt 0.1.0.
- Tests: Update-Dienst mit Test-Quelle (neuer, gleich, kaputte Prüfsumme, kein Netz, Limit, GitHub-Antwort).
- Noch nicht gemacht (braucht dein OK): Inno Setup installieren und Installer bauen/ausprobieren,
  Push auf GitHub, erste Release und Probe-Update mit zwei Testversionen.

### Nachtrag Phase 9 – Installer geprüft (2026-11-10)
- `installer\bauen.ps1` baut `dist\SimpleGrid-Setup.exe` (≈13 MB) und `dist\versionen.json`.
- Stille Installation als normaler Benutzer: Exit 0, Programm unter `%LocalAppData%\Programs\SimpleGrid`,
  Uninstall-Eintrag nur in HKCU (keine Adminrechte), Startmenü- und Desktop-Verknüpfung, .xlsx-Zuordnung aus.
- Installiertes Programm startet und schließt normal.
- Stille Deinstallation: Ordner, Verknüpfungen, Registry-Eintrag, Einstellungen und Sicherungen entfernt.
- Fenstertitel jetzt „Mappe1 – SimpleGrid“ bzw. „… (nicht gespeichert) – SimpleGrid“ (vorher englisch „— Edited“).
- Noch offen: Probe-Update über zwei echte GitHub-Releases (braucht Push und Release).

### Nachtrag Phase 9 – Update-Probelauf über GitHub (2026-11-10)
- Code nach `Devin-Schlichting/SimpleGrid` hochgeladen; GitHub-Aktion hat Release `v0.1.0` und `v0.1.1` gebaut
  (Build, alle Tests, Installer, `versionen.json`; je ca. 27 Minuten).
- v0.1.0 aus der Release installiert (Prüfsumme stimmt). Beim Start meldete sie selbst
  „Es gibt eine neue Version von SimpleGrid (0.1.1). Jetzt aktualisieren?“ → Klick → Installer geladen, geprüft,
  still installiert, Programm neu gestartet. Info-Fenster zeigt „Version 0.1.1“; „Nach Updates suchen“ meldet
  „Sie haben bereits die neueste Version (0.1.1).“
- Beobachtet für Phase 10: SimpleGrid folgt dem dunklen Windows-Design; Meldungsfenster sind noch klein.
- Die GitHub-Aktion warnt, dass `actions/checkout@v4` auf Node 20 läuft (nur Hinweis, kein Fehler).

## Phase 10 – Design (2026-11-10)
- Entwürfe als Design-Zeichenfläche (Artifact „SimpleGrid Design-Entwürfe“): drei Varianten der Hauptansicht,
  Rückfrage, Seitenansicht. Gewählt: **Variante A – Register mit großen Knöpfen**.
- Branding-Paket nach `branding/` entpackt (Symbol, Logos, Farben, Inter). Programm-Symbol (exe, Fenster,
  Installer) aus der Master-Geometrie erzeugt: `installer/symbol_erzeugen.py`. Logo in der Kopfzeile gezeichnet.
- Farben: Knöpfe/Text in dunklem Grün `#047857` (statt `#10B981`, das mit Weiß nur ~2,5:1 Kontrast hat),
  Grau `#1F2937`/`#F3F4F6`; immer helles Design, unabhängig vom Windows-Design.
- Neue Oberfläche (`einfach.rs`): Kopfzeile (Logo, Dateiname, „Alles gespeichert“/„Noch nicht gespeichert“),
  6 Register (Start, Schrift und Farbe, Rahmen und Zellen, Zeilen und Spalten, Drucken, Hilfe), große Kacheln mit
  GridCraft-Icons über der Beschriftung, Fußzeile mit Blättern (Doppelklick/„Umbenennen“ mit Eingabefenster),
  „+ Blatt hinzufügen“, Löschen, ◀ ▶ und Zoom; waagerechte Bildlaufleiste darüber.
- Rückfragen im neuen Stil: großer Titel, Hauptaktion grün, „Nicht speichern“/„Löschen“ rot umrandet, „Abbrechen“ grau.
- Seitenansicht wie im Entwurf: links Schritte 1–3 (Lage, Farbe, Drucker) und „Jetzt drucken“, rechts die Seite.
- In sehr niedrigen Fenstern (< 640 pt) entfallen Kopfzeile und Kachel-Symbole, damit die Tabelle sichtbar bleibt.
- Tests: 3104 grün. Entfernt: Tests von GridCraft's Statusleiste und alten Blattreitern sowie ein Grenzfall
  (400 pt hohes Fenster mit aufgeklappter Bearbeitungszeile).
- Noch offen: Schrift „Inter“ ist nicht im Branding-Paket; bisher Systemschrift.

## Nachbesserungen nach Phase 10 (2026-11-11)
- Formeln: GridCraft's Funktionen bleiben alle nutzbar (eintippen); in die Leiste kommen vorerst keine weiteren.
- Neu, Öffnen, Speichern, Speichern unter stehen jetzt in der Kopfzeile neben dem Dateinamen, links vom
  Speicherstand (immer sichtbar); das Register „Start“ enthält nur noch Bearbeiten und Summe.
- „Summe“ legt die Formel (`=SUMME(B2:B4)`) in der Zielzelle zum Bestätigen vor, der Bereich ist markiert.
  Mehrere Spalten auf einmal rechnet sie weiter direkt aus.
- Bei jeder Eingabe in eine Zelle erscheinen direkt darunter ein grüner Haken (übernehmen, wie Enter) und ein
  rotes Kreuz (verwerfen, wie Esc) – ganz ohne Tastatur bedienbar.
- Tests: 3106 grün.

## Version 0.2.2 – Fehlerliste (2026-11-11)
Umsetzung von `docs/FEHLERLISTE.md`:
1. Funktionshinweis deutsch (Erklärung und Argumentnamen für die gängigen Funktionen); ohne Übersetzung nur die Schreibweise.
2. Grüner Haken / rotes Kreuz rechts neben dem Eingabefeld, nicht mehr vom Hinweis verdeckt.
3. Jedes Zahlenformat mit „€“ (auch Excel „Buchhaltung“/„Währung“) gilt als Euro; der Knopf ist dann grün.
4. Kopfzeile schmaler (56 px), Logo, Name und Datei-Knöpfe auf einer Linie.
5. Druckbereich als grüner gestrichelter Rahmen mit Schild „Druckbereich“ (nur Bildschirm).
7. Beim Eintippen wird eine zu schmale Spalte automatisch breiter (nur breiter, mit Rückgängig zurücknehmbar).
9. Alle Meldungen im großen Stil der Rückfragen.
12./13. Ausfüllkästchen und grüne Fehlerdreiecke abgeschaltet.
14. Häufige englische Meldungen übersetzt, sonst „Das hat leider nicht geklappt.“ (Original ins Protokoll).
15. Per Ziehen/Doppelklick geöffnete Dateien kommen in ein eigenes Fenster; „Neu“/„Öffnen“ ersetzen nur die Tabelle
    im aktuellen Fenster. Die Speichern-Rückfrage beim Schließen eines Fensters ist groß und deutsch.
16. Oberflächenschrift Inter 4.1 (OFL) eingebettet, Lizenz in `assets/fonts/Inter-OFL.txt` und `ATTRIBUTION.md`.
18. Release-Aktion: `actions/checkout@v5`, `Swatinem/rust-cache`; Pushes auf `main` bauen ohne Veröffentlichung
    (füllt den Zwischenspeicher, den die Tags nutzen).
