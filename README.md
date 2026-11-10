# SimpleGrid

Ein einfaches Tabellenprogramm für Windows, gemacht für ältere Menschen: große, beschriftete
Schaltflächen, keine verschachtelten Menüs, nur die Funktionen, die wirklich gebraucht werden.
Nur auf Deutsch.

SimpleGrid ist ein Umbau von [GridCraft](https://github.com/storytold/gridcraft) (MIT OR Apache-2.0).
Die Rechen-, Datei- und Druck-Engine stammt von dort; die Oberfläche ist stark vereinfacht.
Lizenz- und Urheberhinweise: [LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE),
[NOTICE](NOTICE), [ATTRIBUTION.md](ATTRIBUTION.md).

## Bauen

Voraussetzungen: Rust (rustup) und die Visual-Studio-C++-Build-Tools.

```
cargo build --release -p simplegrid
cargo test --workspace
```

Arbeitsanweisung und Stand: [docs/ANLEITUNG.md](docs/ANLEITUNG.md), [docs/FORTSCHRITT.md](docs/FORTSCHRITT.md).
