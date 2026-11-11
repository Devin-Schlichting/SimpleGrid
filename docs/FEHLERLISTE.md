# Fehlerliste und Lösungsansätze (2026-11-11)

Gesammelt nach Version 0.2.1. Entscheidungen des Auftraggebers sind eingetragen.

| Nr. | Fehler / Wunsch | Lösung |
|---|---|---|
| 1 | Funktionshinweis ist englisch (`number1`, „Adds all the numbers…“) | Deutsche Tabelle für Argumentnamen und Erklärungen (wie GridCraft's spanische); ohne Übersetzung nur die Schreibweise zeigen |
| 2 | Funktionshinweis verdeckt grünen Haken / rotes Kreuz | ✓/✗ rechts neben das Eingabefeld, Hinweis bleibt darunter |
| 3 | Euro-Format aus Excel („Buchhaltung“/„Währung“) wird nicht als Euro erkannt | Jedes Zahlenformat mit „€“ gilt als Euro (Knopf grün, Klick → normale Zahl); Anzeige deutsch prüfen |
| 4 | Kopfzeile zu hoch, Datei-Knöpfe nicht auf einer Linie | Weniger Innenabstand, Knöpfe 40 px, alles senkrecht mittig |
| 5 | Druckbereich in der Tabelle sichtbar machen | Kräftiger grüner gestrichelter Rahmen mit Schild „Druckbereich“ (nur Bildschirm) |
| 7 | `########` bei zu schmaler Spalte | Beim Eintippen automatisch verbreitern (nur breiter, rückgängig machbar); geöffnete Dateien unverändert |
| 9 | Kleine GridCraft-Meldungsfenster | Alle Meldungen im großen Stil der Rückfragen |
| 12 | Ausfüllkästchen mit englischem Menü | **Abschalten** |
| 13 | Grüne Dreiecke „Zahl als Text“ mit englischem Menü | **Abschalten** |
| 14 | Englische Fehlermeldungen der Engine | Übersetzungstabelle für häufige Meldungen, sonst „Das hat leider nicht geklappt.“ (Original ins Protokoll) |
| 15 | Datei per Ziehen/Doppelklick bei laufendem Programm | **Zweites Fenster** (GridCraft `doc_windows`); „Neu“/„Öffnen“ in der Leiste **ersetzen wie bisher** |
| 16 | Schriftart Inter fehlt | Inter Regular/Bold (OFL) herunterladen, einbinden, Lizenz in ATTRIBUTION |
| 18 | Release-Aktion langsam, Node-20-Hinweis | `Swatinem/rust-cache`, neueres `actions/checkout` |
