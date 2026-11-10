//! SimpleGrid: deutsche Zahlenschreibweise (Dezimalkomma, Tausenderpunkt) für Eingabe und Anzeige.
//!
//! Die Engine rechnet und speichert weiter in der neutralen Schreibweise (`12.5`, Dateien wie bisher).
//! Nur an den Rändern – was getippt wird und was angezeigt wird – gilt die deutsche Schreibweise,
//! wenn sie eingeschaltet ist. Das Programm schaltet sie beim Start für alle Threads ein
//! ([`set_german`]); Tests schalten sie nur für ihren eigenen Thread ein ([`set_german_for_thread`]),
//! damit die übrigen (englischen) Tests unberührt bleiben.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::date::{DateSystem, days_in_month, serial_from_ymd};

static GLOBAL: AtomicBool = AtomicBool::new(false);

thread_local! {
    static LOCAL: Cell<Option<bool>> = const { Cell::new(None) };
}

/// Das Euro-Zahlenformat von SimpleGrid: zwei Nachkommastellen, Tausendertrennung, „ €“ dahinter.
/// Mit deutscher Schreibweise angezeigt als `1.234,56 €`.
pub const EURO_FORMAT: &str = "#,##0.00\\ \"€\"";

/// Schaltet die deutsche Schreibweise für das ganze Programm ein oder aus.
pub fn set_german(on: bool) {
    GLOBAL.store(on, Ordering::Relaxed);
}

/// Schaltet die deutsche Schreibweise nur für den aktuellen Thread ein oder aus (für Tests).
pub fn set_german_for_thread(on: bool) {
    let _ = LOCAL.try_with(|l| l.set(Some(on)));
}

/// Ob gerade die deutsche Schreibweise gilt.
pub fn german() -> bool {
    LOCAL.try_with(Cell::get).ok().flatten().unwrap_or_else(|| GLOBAL.load(Ordering::Relaxed))
}

/// Tauscht in einer fertig formatierten Zahl Punkt und Komma, wo sie zwischen Ziffern stehen:
/// `1,234.56` → `1.234,56`. Text wie „Stk.“ bleibt unverändert.
pub fn localize_number_text(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    for (i, &c) in chars.iter().enumerate() {
        let between_digits = i > 0 && chars[i - 1].is_ascii_digit() && chars.get(i + 1).is_some_and(char::is_ascii_digit);
        // Auch „0.5“ am Anfang und „-.5“ (Ziffer nur danach) gelten als Dezimaltrenner.
        let leading_decimal = c == '.' && chars.get(i + 1).is_some_and(char::is_ascii_digit) && (i == 0 || matches!(chars[i - 1], '-' | ' ' | '('));
        out.push(match c {
            '.' if between_digits || leading_decimal => ',',
            ',' if between_digits => '.',
            _ => c,
        });
    }
    out
}

/// Eine Zahl so, wie sie im Bearbeitungsfeld stehen soll (`12.5` → `12,5`), ohne Tausenderpunkte.
pub fn edit_number_text(s: &str) -> String {
    if german() { s.replace('.', ",") } else { s.to_string() }
}

/// Liest eine Eingabe in deutscher Schreibweise: `12,5`, `-1.234,56`, `12,50 €`, `€ 3`, `7,5 %`,
/// `10.11.2026`, `1.2.26`. Gibt den Wert und das passende Zahlenformat zurück; `None`, wenn die
/// Eingabe keine deutsche Zahl und kein deutsches Datum ist.
pub fn parse_german(t: &str, sys: DateSystem) -> Option<(f64, Option<&'static str>)> {
    let t = t.trim();
    if let Some(d) = parse_german_date(t, sys) {
        return Some(d);
    }
    let mut s = t;
    let mut euro = false;
    let mut percent = false;
    if let Some(r) = s.strip_suffix('€') {
        euro = true;
        s = r.trim_end();
    } else if let Some(r) = s.strip_prefix('€') {
        euro = true;
        s = r.trim_start();
    }
    if !euro && let Some(r) = s.strip_suffix('%') {
        percent = true;
        s = r.trim_end();
    }
    let (neg, body) = match s.strip_prefix('-') {
        Some(r) => (true, r.trim_start()),
        None => (false, s.strip_prefix('+').map_or(s, str::trim_start)),
    };
    // Nach dem Vorzeichen darf das Euro-Zeichen auch noch stehen: „-€ 5“.
    let body = if !euro && let Some(r) = body.strip_prefix('€') {
        euro = true;
        r.trim_start()
    } else {
        body
    };
    let n = german_number(body)?;
    let n = if neg { -n } else { n };
    if euro {
        return Some((n, Some(EURO_FORMAT)));
    }
    if percent {
        let decimals = body.contains(',');
        return Some((n / 100.0, Some(if decimals { "0.00%" } else { "0%" })));
    }
    // Mit Tausenderpunkt getippt: Format mit Tausendertrennung, wie beim Standardleser.
    if body.contains('.') {
        return Some((n, Some(if body.contains(',') { "#,##0.00" } else { "#,##0" })));
    }
    Some((n, None))
}

/// `1.234,56` → 1234.56; `12,5` → 12.5; `1234` → 1234. Tausenderpunkte nur in Dreiergruppen.
fn german_number(s: &str) -> Option<f64> {
    if s.is_empty() || !s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') {
        return None;
    }
    let (int, frac) = match s.split_once(',') {
        Some((a, b)) => (a, Some(b)),
        None => (s, None),
    };
    if frac.is_some_and(|f| f.is_empty() || !f.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }
    let int_digits = if int.contains('.') {
        let mut groups = int.split('.');
        let first = groups.next()?;
        if first.is_empty() || first.len() > 3 {
            return None;
        }
        let mut digits = first.to_string();
        for g in groups {
            if g.len() != 3 {
                return None;
            }
            digits.push_str(g);
        }
        digits
    } else {
        if int.is_empty() && frac.is_none() {
            return None;
        }
        int.to_string()
    };
    let text = match frac {
        Some(f) => format!("{}.{f}", if int_digits.is_empty() { "0" } else { &int_digits }),
        None => int_digits,
    };
    text.parse::<f64>().ok().filter(|n| n.is_finite())
}

/// `10.11.2026`, `1.2.26` → Datum mit Format `dd.mm.yyyy`.
fn parse_german_date(t: &str, sys: DateSystem) -> Option<(f64, Option<&'static str>)> {
    let mut parts = t.split('.');
    let d = parts.next()?;
    let m = parts.next()?;
    let y = parts.next()?;
    if parts.next().is_some() || d.is_empty() || m.is_empty() || d.len() > 2 || m.len() > 2 || !(y.len() == 2 || y.len() == 4) {
        return None;
    }
    if ![d, m, y].iter().all(|p| p.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }
    let day: u32 = d.parse().ok()?;
    let month: u32 = m.parse().ok()?;
    let mut year: i32 = y.parse().ok()?;
    if y.len() == 2 {
        year += if year < 30 { 2000 } else { 1900 };
    }
    if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) || !(1900..=9999).contains(&year) {
        return None;
    }
    serial_from_ymd(sys, year.into(), month.into(), day.into()).map(|n| (n, Some("dd.mm.yyyy")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Option<(f64, Option<&'static str>)> {
        parse_german(s, DateSystem::D1900)
    }

    #[test]
    fn zahlen() {
        assert_eq!(p("12,5"), Some((12.5, None)));
        assert_eq!(p("1.234,56"), Some((1234.56, Some("#,##0.00"))));
        assert_eq!(p("-1.234"), Some((-1234.0, Some("#,##0"))));
        assert_eq!(p("1.000.000"), Some((1_000_000.0, Some("#,##0"))));
        assert_eq!(p(",5"), Some((0.5, None)));
        assert_eq!(p("12.5"), None, "kein gültiger Tausenderpunkt: der Standardleser entscheidet");
        assert_eq!(p("1,2,3"), None);
        assert_eq!(p("abc"), None);
    }

    #[test]
    fn euro_und_prozent() {
        assert_eq!(p("12,50 €"), Some((12.5, Some(EURO_FORMAT))));
        assert_eq!(p("€ 3"), Some((3.0, Some(EURO_FORMAT))));
        assert_eq!(p("-5 €"), Some((-5.0, Some(EURO_FORMAT))));
        assert_eq!(p("7,5 %"), Some((0.075, Some("0.00%"))));
        assert_eq!(p("20%"), Some((0.2, Some("0%"))));
    }

    #[test]
    fn datum() {
        let (n, f) = p("10.11.2026").unwrap_or_default();
        assert_eq!(f, Some("dd.mm.yyyy"));
        assert_eq!(serial_from_ymd(DateSystem::D1900, 2026, 11, 10), Some(n));
        assert!(p("1.2.26").is_some());
        assert_eq!(p("31.2.2026"), None);
    }

    #[test]
    fn anzeige() {
        assert_eq!(localize_number_text("1,234.56 €"), "1.234,56 €");
        assert_eq!(localize_number_text("12.5"), "12,5");
        assert_eq!(localize_number_text("0.5"), "0,5");
        assert_eq!(localize_number_text("3 Stk."), "3 Stk.");
        assert_eq!(localize_number_text("-1,000"), "-1.000");
    }

    #[test]
    fn schalter_pro_thread() {
        set_german_for_thread(true);
        assert!(german());
        set_german_for_thread(false);
        assert!(!german());
    }
}
