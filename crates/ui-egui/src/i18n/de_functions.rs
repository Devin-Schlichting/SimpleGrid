//! SimpleGrid: deutsche Hilfe für die gängigen Tabellenfunktionen (Hinweis beim Eintippen einer
//! Formel). Funktionen ohne Eintrag zeigen nur ihre Schreibweise, keine englische Erklärung.
//! Beide Tabellen müssen nach dem englischen Text sortiert bleiben (Test in `i18n.rs`).

/// Beschreibungen der Funktionen.
pub(super) const DESCRIPTIONS: &[(&str, &str)] = &[
    ("Adds all the numbers in the arguments.", "Zählt alle Zahlen zusammen."),
    ("Adds the cells that meet a condition.", "Zählt die Zellen zusammen, die eine Bedingung erfüllen."),
    ("Arithmetic mean of the numbers.", "Der Durchschnitt der Zahlen."),
    ("Counts the cells that meet a condition.", "Zählt die Zellen, die eine Bedingung erfüllen."),
    ("Counts the non-empty values.", "Zählt die Zellen, die nicht leer sind."),
    ("Counts the numbers.", "Zählt, wie viele Zahlen es sind."),
    ("Largest number (0 when there are none).", "Die größte Zahl."),
    ("Returns the current date.", "Das heutige Datum."),
    ("Rounds a number to a given number of digits, halves away from zero.", "Rundet eine Zahl auf die gewünschte Anzahl Nachkommastellen."),
    ("Smallest number (0 when there are none).", "Die kleinste Zahl."),
];

/// Argumentnamen ohne Klammern und Nummer (`number`, nicht `[number2]`).
pub(super) const ARGUMENTS: &[(&str, &str)] = &[
    ("criteria", "Bedingung"),
    ("num_digits", "Stellen"),
    ("number", "Zahl"),
    ("range", "Bereich"),
    ("sum_range", "Summenbereich"),
    ("value", "Wert"),
];
