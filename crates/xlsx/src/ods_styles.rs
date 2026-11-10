//! SimpleGrid: Formate aus OpenDocument-Tabellen (.ods) übernehmen.
//!
//! Liest `style:style`-Einträge (Zell-, Spalten- und Zeilenstile) und Währungs-Zahlenformate aus
//! `styles.xml` und `content.xml`. Übernommen werden fett, kursiv, unterstrichen, Schriftgröße,
//! Textfarbe, Hintergrundfarbe, Ausrichtung, Umrandung, Euro-Format, Spaltenbreiten und
//! Zeilenhöhen. Alles andere wird still ignoriert.
//! Referenz: ODF 1.3, Teil 3, Abschnitte 16 (Stile) und 20 (Formatierungsattribute).

use std::collections::HashMap;

use quick_xml::NsReader;
use quick_xml::XmlVersion;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;
use simplegrid_model::{BorderLine, BorderStyle, Color, HAlign, NumFmt, Style, StyleId, Underline, Workbook};

const STYLE: &[u8] = b"urn:oasis:names:tc:opendocument:xmlns:style:1.0";
const FO: &[u8] = b"urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";
const NUMBER: &[u8] = b"urn:oasis:names:tc:opendocument:xmlns:datastyle:1.0";
/// Höchstzahl gelesener Stile, damit eine präparierte Datei nicht beliebig Speicher belegt.
const MAX_STYLES: usize = 100_000;
/// Höchstens so tief wird `style:parent-style-name` verfolgt.
const MAX_PARENTS: usize = 16;

/// Das Euro-Format, das SimpleGrid für Währungszellen verwendet (Anzeige `1.234,56 €`).
pub const EURO_FORMAT: &str = simplegrid_core::locale::EURO_FORMAT;

/// Die gelesenen Eigenschaften eines Stils; `None` heißt „vom Elternstil oder Standard“.
#[derive(Clone, Debug, Default)]
pub(crate) struct OdsStyle {
    parent: Option<String>,
    data_style: Option<String>,
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    size: Option<f32>,
    color: Option<Color>,
    fill: Option<Option<Color>>,
    h_align: Option<HAlign>,
    border_left: Option<BorderLine>,
    border_right: Option<BorderLine>,
    border_top: Option<BorderLine>,
    border_bottom: Option<BorderLine>,
    /// Spaltenbreite bzw. Zeilenhöhe in Pixeln bei 96 dpi.
    pub(crate) column_width: Option<f32>,
    pub(crate) row_height: Option<f32>,
    /// Zeilenhöhe passt sich dem Inhalt an (dann keine feste Höhe übernehmen).
    pub(crate) optimal_row_height: bool,
}

/// Alle Stile einer .ods-Datei plus die schon angelegten `StyleId`s.
#[derive(Default)]
pub(crate) struct OdsStyles {
    styles: HashMap<String, OdsStyle>,
    /// Namen der Zahlenformate mit Euro-Zeichen.
    euro: Vec<String>,
    ids: HashMap<String, StyleId>,
}

fn is_ns(ns: &[u8], expected: &[u8]) -> bool {
    ns == expected
}

/// Alle Attribute eines Elements als (Namensraum, lokaler Name, Wert).
fn attrs(reader: &mut NsReader<&[u8]>, e: &BytesStart<'_>) -> Vec<(Vec<u8>, Vec<u8>, String)> {
    let mut out = Vec::new();
    for a in e.attributes().flatten() {
        let (ns, local) = reader.resolver_mut().resolve_attribute(a.key);
        let ns = match ns {
            ResolveResult::Bound(n) => n.as_ref().to_vec(),
            _ => Vec::new(),
        };
        if let Ok(v) = a.normalized_value(XmlVersion::Explicit1_0) {
            out.push((ns, local.as_ref().to_vec(), v.into_owned()));
        }
    }
    out
}

fn get<'a>(list: &'a [(Vec<u8>, Vec<u8>, String)], ns: &[u8], name: &[u8]) -> Option<&'a str> {
    list.iter().find(|(n, l, _)| n == ns && l == name).map(|(_, _, v)| v.as_str())
}

/// Eine ODF-Länge („2.258cm“, „0.1665in“, „12pt“, „5mm“) in Pixeln bei 96 dpi.
pub(crate) fn length_px(s: &str) -> Option<f32> {
    let s = s.trim();
    let split = s.find(|c: char| c.is_ascii_alphabetic())?;
    let (n, unit) = s.split_at(split);
    let n: f32 = n.trim().parse().ok().filter(|n: &f32| n.is_finite() && *n >= 0.0)?;
    let px = match unit {
        "cm" => n * 96.0 / 2.54,
        "mm" => n * 96.0 / 25.4,
        "in" => n * 96.0,
        "pt" => n * 96.0 / 72.0,
        "pc" => n * 16.0,
        "px" => n,
        _ => return None,
    };
    (px <= 10_000.0).then_some(px)
}

/// „#rrggbb“ als Farbe; `transparent` und Unlesbares als `None`.
fn color(s: &str) -> Option<Color> {
    let hex = s.trim().strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    u32::from_str_radix(hex, 16).ok().map(Color::Rgb)
}

/// Eine ODF-Umrandung („0.74pt solid #000000“ oder „none“).
fn border(s: &str) -> Option<BorderLine> {
    let s = s.trim();
    if s.is_empty() || s == "none" || s == "hidden" {
        return Some(BorderLine { style: BorderStyle::None, color: Color::Auto });
    }
    let mut width = 0.75_f32;
    let mut style = BorderStyle::Thin;
    let mut c = Color::Auto;
    for part in s.split_whitespace() {
        if let Some(px) = length_px(part) {
            width = px * 72.0 / 96.0;
        } else if let Some(col) = color(part) {
            c = col;
        } else {
            style = match part {
                "double" => BorderStyle::Double,
                "dotted" => BorderStyle::Dotted,
                "dashed" => BorderStyle::Dashed,
                _ => BorderStyle::Thin,
            };
        }
    }
    if style == BorderStyle::Thin {
        // Dicke in Punkt: bis 1 pt dünn, bis 2 pt mittel, darüber dick.
        style = if width > 2.0 {
            BorderStyle::Thick
        } else if width > 1.0 {
            BorderStyle::Medium
        } else {
            BorderStyle::Thin
        };
    }
    Some(BorderLine { style, color: c })
}

impl OdsStyles {
    /// Liest die Stile aus einem XML-Teil (`styles.xml` oder `content.xml`). Fehlerhafte Teile
    /// werden übersprungen: Formate sind schön, aber nie ein Grund, die Datei abzulehnen.
    pub(crate) fn read(&mut self, bytes: &[u8]) {
        let mut reader = NsReader::from_reader(bytes);
        let mut current: Option<(String, OdsStyle)> = None;
        // Name des gerade gelesenen Zahlenformats, solange es offen ist.
        let mut number_style: Option<String> = None;
        let mut depth = 0usize;
        let mut number_depth = 0usize;
        let mut style_depth = 0usize;
        loop {
            let Ok(event) = reader.read_event() else { return };
            let (e, empty) = match &event {
                Event::Start(e) => (Some(e.clone()), false),
                Event::Empty(e) => (Some(e.clone()), true),
                Event::End(_) => {
                    depth = depth.saturating_sub(1);
                    if current.is_some() && depth < style_depth {
                        self.finish(current.take());
                    }
                    if number_style.is_some() && depth < number_depth {
                        number_style = None;
                    }
                    continue;
                }
                Event::Text(t) => {
                    // Text eines number:currency-symbol (z. B. „€“).
                    if let Some(name) = &number_style
                        && let Ok(text) = t.xml_content(XmlVersion::Explicit1_0)
                        && text.contains('€')
                        && !self.euro.contains(name)
                    {
                        self.euro.push(name.clone());
                    }
                    continue;
                }
                Event::Eof => break,
                _ => continue,
            };
            let Some(e) = e else { continue };
            let (ns, local) = reader.resolver_mut().resolve_element(e.name());
            let ns: Vec<u8> = match ns {
                ResolveResult::Bound(n) => n.as_ref().to_vec(),
                _ => Vec::new(),
            };
            let local = local.as_ref().to_vec();
            let list = attrs(&mut reader, &e);
            if is_ns(&ns, STYLE) && local == b"style" {
                if self.styles.len() >= MAX_STYLES {
                    return;
                }
                let name = get(&list, STYLE, b"name").unwrap_or_default().to_string();
                let style = OdsStyle {
                    parent: get(&list, STYLE, b"parent-style-name").map(str::to_string),
                    data_style: get(&list, STYLE, b"data-style-name").map(str::to_string),
                    ..Default::default()
                };
                if empty {
                    self.finish(Some((name, style)));
                } else {
                    current = Some((name, style));
                    style_depth = depth + 1;
                }
            } else if let Some((_, st)) = current.as_mut() {
                apply_properties(st, &ns, &local, &list);
            } else if is_ns(&ns, NUMBER) && local.ends_with(b"-style") {
                let name = get(&list, STYLE, b"name").unwrap_or_default().to_string();
                if !empty {
                    number_style = Some(name);
                    number_depth = depth + 1;
                }
            } else if is_ns(&ns, NUMBER)
                && local == b"currency-symbol"
                && let Some(name) = &number_style
            {
                // Das Symbol steht als Text im Element; manche Schreiber setzen nur die Sprache.
                if get(&list, NUMBER, b"country").is_some_and(|c| matches!(c, "DE" | "AT" | "FR" | "IT" | "ES" | "NL" | "BE" | "LU" | "FI" | "IE"))
                    && !self.euro.contains(name)
                {
                    self.euro.push(name.clone());
                }
            }
            if !empty {
                depth += 1;
            }
        }
    }

    fn finish(&mut self, entry: Option<(String, OdsStyle)>) {
        if let Some((name, style)) = entry
            && !name.is_empty()
        {
            self.styles.insert(name, style);
        }
    }

    /// Der Stil `name` mit allen Eigenschaften seiner Elternstile.
    pub(crate) fn resolved(&self, name: &str) -> Option<OdsStyle> {
        let mut chain = Vec::new();
        let mut next = Some(name.to_string());
        while let Some(n) = next {
            if chain.len() >= MAX_PARENTS {
                break;
            }
            let Some(st) = self.styles.get(&n) else { break };
            next = st.parent.clone();
            chain.push(st);
        }
        if chain.is_empty() {
            return None;
        }
        let mut out = OdsStyle::default();
        for st in chain.into_iter().rev() {
            macro_rules! take {
                ($($f:ident),*) => { $( if st.$f.is_some() { out.$f = st.$f.clone(); } )* };
            }
            take!(data_style, bold, italic, underline, size, color, fill, h_align, border_left, border_right, border_top, border_bottom, column_width, row_height);
            out.optimal_row_height = st.optimal_row_height;
        }
        Some(out)
    }

    /// Die `StyleId` für den Zellstil `name` (einmal angelegt, dann wiederverwendet).
    pub(crate) fn cell_style(&mut self, wb: &mut Workbook, name: &str) -> StyleId {
        if let Some(id) = self.ids.get(name) {
            return *id;
        }
        let id = match self.resolved(name) {
            Some(st) => {
                let euro = st.data_style.as_ref().is_some_and(|d| self.euro.contains(d));
                wb.styles.derive(StyleId::DEFAULT, |s: &mut Style| {
                    if let Some(b) = st.bold {
                        s.font.bold = b;
                    }
                    if let Some(i) = st.italic {
                        s.font.italic = i;
                    }
                    if let Some(u) = st.underline {
                        s.font.underline = if u { Underline::Single } else { Underline::None };
                    }
                    if let Some(size) = st.size {
                        s.font.size = size;
                    }
                    if let Some(c) = st.color {
                        s.font.color = c;
                    }
                    if let Some(fill) = st.fill {
                        s.fill = match fill {
                            Some(c) => simplegrid_model::Fill::solid(c),
                            None => simplegrid_model::Fill::default(),
                        };
                    }
                    if let Some(h) = st.h_align {
                        s.align.h = h;
                    }
                    for (src, dst) in [
                        (st.border_left, &mut s.borders.left),
                        (st.border_right, &mut s.borders.right),
                        (st.border_top, &mut s.borders.top),
                        (st.border_bottom, &mut s.borders.bottom),
                    ] {
                        if let Some(b) = src {
                            *dst = b;
                        }
                    }
                    if euro {
                        s.num_fmt = NumFmt::new(EURO_FORMAT);
                    }
                })
            }
            None => StyleId::DEFAULT,
        };
        self.ids.insert(name.to_string(), id);
        id
    }
}

/// Überträgt die Formatierungsattribute eines Kind-Elements von `style:style`.
fn apply_properties(st: &mut OdsStyle, ns: &[u8], local: &[u8], list: &[(Vec<u8>, Vec<u8>, String)]) {
    if !is_ns(ns, STYLE) {
        return;
    }
    match local {
        b"text-properties" => {
            if let Some(w) = get(list, FO, b"font-weight") {
                st.bold = Some(w == "bold" || w.parse::<u32>().is_ok_and(|n| n >= 600));
            }
            if let Some(i) = get(list, FO, b"font-style") {
                st.italic = Some(i == "italic" || i == "oblique");
            }
            if let Some(u) = get(list, STYLE, b"text-underline-style") {
                st.underline = Some(u != "none");
            }
            if let Some(size) = get(list, FO, b"font-size").and_then(length_px) {
                st.size = Some(size * 72.0 / 96.0);
            }
            if let Some(c) = get(list, FO, b"color").and_then(color) {
                st.color = Some(c);
            }
        }
        b"table-cell-properties" => {
            if let Some(bg) = get(list, FO, b"background-color") {
                st.fill = Some(color(bg));
            }
            if let Some(all) = get(list, FO, b"border").and_then(border) {
                st.border_left = Some(all);
                st.border_right = Some(all);
                st.border_top = Some(all);
                st.border_bottom = Some(all);
            }
            for (name, slot) in [
                (&b"border-left"[..], &mut st.border_left),
                (b"border-right", &mut st.border_right),
                (b"border-top", &mut st.border_top),
                (b"border-bottom", &mut st.border_bottom),
            ] {
                if let Some(b) = get(list, FO, name).and_then(border) {
                    *slot = Some(b);
                }
            }
        }
        b"paragraph-properties" => {
            if let Some(a) = get(list, FO, b"text-align") {
                st.h_align = match a {
                    "center" => Some(HAlign::Center),
                    "end" | "right" => Some(HAlign::Right),
                    "start" | "left" => Some(HAlign::Left),
                    "justify" => Some(HAlign::Justify),
                    _ => None,
                };
            }
        }
        b"table-column-properties" => {
            st.column_width = get(list, STYLE, b"column-width").and_then(length_px);
        }
        b"table-row-properties" => {
            st.row_height = get(list, STYLE, b"row-height").and_then(length_px);
            st.optimal_row_height = get(list, STYLE, b"use-optimal-row-height") == Some("true");
        }
        _ => {}
    }
}

/// Wandelt eine OpenFormula (`of:=SUM([.A1:.A5])`) in die interne Schreibweise (`SUM(A1:A5)`).
/// `None`, wenn die Formel nicht sicher umgewandelt werden kann.
pub(crate) fn convert_formula(src: &str) -> Option<String> {
    let body = src.strip_prefix("of:").or_else(|| src.strip_prefix("oooc:")).unwrap_or(src);
    let body = body.strip_prefix('=')?;
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                // Text in Anführungszeichen unverändert übernehmen ("" ist ein Anführungszeichen).
                out.push('"');
                loop {
                    let n = chars.next()?;
                    out.push(n);
                    if n == '"' {
                        if chars.peek() == Some(&'"') {
                            out.push(chars.next()?);
                        } else {
                            break;
                        }
                    }
                }
            }
            '[' => {
                let mut r = String::new();
                loop {
                    let n = chars.next()?;
                    if n == ']' {
                        break;
                    }
                    r.push(n);
                }
                out.push_str(&convert_reference(&r)?);
            }
            ';' => out.push(','),
            _ => out.push(c),
        }
    }
    Some(out)
}

/// `.A1`, `$Tabelle2.A1`, `.A1:.B5`, `'Mein Blatt'.A1:'Mein Blatt'.B2` → `A1`, `Tabelle2!A1` …
fn convert_reference(r: &str) -> Option<String> {
    let mut parts = r.splitn(2, ':');
    let first = parts.next()?;
    let (sheet, a) = split_sheet(first)?;
    let mut out = match sheet {
        Some(s) => format!("{}!{}", quote_sheet(&s), a),
        None => a.to_string(),
    };
    if let Some(second) = parts.next() {
        let (_, b) = split_sheet(second)?;
        out.push(':');
        out.push_str(b);
    }
    Some(out)
}

/// Trennt `$Blatt.A1` in (Some("Blatt"), "A1") und `.A1` in (None, "A1").
fn split_sheet(s: &str) -> Option<(Option<String>, &str)> {
    let dot = s.rfind('.')?;
    let (sheet, cell) = (&s[..dot], &s[dot + 1..]);
    if cell.is_empty() || cell.contains('#') {
        return None;
    }
    let sheet = sheet.trim_start_matches('$');
    if sheet.is_empty() {
        return Some((None, cell));
    }
    let name = sheet.strip_prefix('\'').and_then(|x| x.strip_suffix('\'')).map(|x| x.replace("''", "'")).unwrap_or_else(|| sheet.to_string());
    Some((Some(name), cell))
}

fn quote_sheet(name: &str) -> String {
    if name.chars().all(|c| c.is_alphanumeric() || c == '_') && !name.starts_with(|c: char| c.is_ascii_digit()) {
        name.to_string()
    } else {
        format!("'{}'", name.replace('\'', "''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formeln_werden_umgewandelt() {
        assert_eq!(convert_formula("of:=SUM([.A1:.A5])").as_deref(), Some("SUM(A1:A5)"));
        assert_eq!(convert_formula("of:=[.A1]+[.A2]*2.5").as_deref(), Some("A1+A2*2.5"));
        assert_eq!(convert_formula("of:=IF([.A1]>0;\"ja; gut\";\"nein\")").as_deref(), Some("IF(A1>0,\"ja; gut\",\"nein\")"));
        assert_eq!(convert_formula("of:=[$Tabelle2.B3]").as_deref(), Some("Tabelle2!B3"));
        assert_eq!(convert_formula("of:=SUM(['Mein Blatt'.A1:'Mein Blatt'.B2])").as_deref(), Some("SUM('Mein Blatt'!A1:B2)"));
        assert_eq!(convert_formula("of:=[.$A$1]").as_deref(), Some("$A$1"));
        assert_eq!(convert_formula("of:=[.#REF!]"), None);
        assert_eq!(convert_formula("of:=SUM([.A1"), None);
    }

    #[test]
    fn laengen_in_pixeln() {
        assert!((length_px("2.54cm").unwrap_or(0.0) - 96.0).abs() < 0.01);
        assert!((length_px("72pt").unwrap_or(0.0) - 96.0).abs() < 0.01);
        assert_eq!(length_px("1in"), Some(96.0));
        assert_eq!(length_px("viel"), None);
    }

    #[test]
    fn umrandungen() {
        assert_eq!(border("0.74pt solid #ff0000"), Some(BorderLine { style: BorderStyle::Thin, color: Color::Rgb(0xff0000) }));
        assert_eq!(border("1.5pt solid #000000").map(|b| b.style), Some(BorderStyle::Medium));
        assert_eq!(border("2.5pt solid #000000").map(|b| b.style), Some(BorderStyle::Thick));
        assert_eq!(border("none").map(|b| b.style), Some(BorderStyle::None));
    }
}
