# SimpleGrid – Hinweise für Claude Code

Verbindliche Arbeitsanweisung: [docs/ANLEITUNG.md](docs/ANLEITUNG.md). Fortschritt: [docs/FORTSCHRITT.md](docs/FORTSCHRITT.md).

- SimpleGrid ist ein Umbau von GridCraft (Rust/egui). Die alten GridCraft-Dokumente liegen nur zum
  Nachschlagen unter `docs/herkunft-gridcraft/`; ihre Hausregeln (storytold, PR-Pflicht, Parität mit
  Excel usw.) gelten hier **nicht**.
- Weiter gültig aus GridCraft: nie abstürzen (kein `unwrap`/`expect`/`panic` außerhalb von Tests,
  kein `unsafe`), alles ist ein Befehl in `crates/engine/src/cmd/*`, die Oberfläche ruft nur
  `app.run(id, params)` auf, Farben aus `theme::Tokens`.
- Bauen: `cargo build -p simplegrid`, testen: `cargo test --workspace`.
- Offscreen-Bild der Oberfläche: `cargo run -p simplegrid-ui-egui --example snapshot -- out.png`.
- Nur Deutsch, nur Windows, nur die sechs Tastenkürzel aus der Anleitung.
