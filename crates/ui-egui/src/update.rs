//! SimpleGrid: Update-Dienst (ohne Oberfläche, damit er mit einer Test-Quelle prüfbar ist und
//! später auch vom Textprogramm genutzt werden kann).
//!
//! Jede Veröffentlichung (GitHub-Release) enthält den Installer und eine Datei `versionen.json`:
//!
//! ```json
//! { "apps": { "SimpleGrid": "1.2.0" }, "installer": "SimpleGrid-Setup.exe", "sha256": "…" }
//! ```
//!
//! Abfragen gehen ohne Anmeldung an die öffentliche GitHub-Schnittstelle. Es wird nie ein
//! Zugangsschlüssel verwendet oder gespeichert.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::Digest;

/// Vorgabe für die Update-Quelle (änderbar in den Einstellungen).
pub const STANDARD_QUELLE: &str = "Devin-Schlichting/SimpleGrid";
/// Name dieses Programms in `versionen.json`.
pub const APP: &str = "SimpleGrid";

/// Eine Versionsnummer `Haupt.Neben.Fehlerbehebung`.
pub type Version = (u32, u32, u32);

/// Liest „1.2.0“ oder „v1.2.0“.
pub fn version(s: &str) -> Option<Version> {
    let s = s.trim().trim_start_matches(['v', 'V']);
    let mut it = s.split('.').map(|p| p.parse::<u32>().ok());
    let v = (it.next()??, it.next().flatten().unwrap_or(0), it.next().flatten().unwrap_or(0));
    Some(v)
}

/// Eine angebotene neue Version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    pub installer_url: String,
    pub sha256: String,
}

/// Was schiefgehen kann. Bei der automatischen Prüfung bleibt das Programm in allen Fällen still.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fehler {
    /// Kein Internet oder Server nicht erreichbar.
    KeinNetz(String),
    /// GitHub-Anfragelimit erreicht.
    Limit,
    /// Antwort unbrauchbar (fehlende Datei, falsches Format).
    Unbrauchbar(String),
    /// Der heruntergeladene Installer hat die falsche Prüfsumme.
    Pruefsumme,
}

impl Fehler {
    /// Eine Meldung in einfachen Worten.
    pub fn text(&self) -> &'static str {
        match self {
            Fehler::KeinNetz(_) | Fehler::Limit => "Die Update-Suche hat gerade keine Verbindung. Bitte später noch einmal versuchen.",
            Fehler::Unbrauchbar(_) => "Die Update-Informationen sind nicht lesbar. Bitte später noch einmal versuchen.",
            Fehler::Pruefsumme => "Das heruntergeladene Update ist beschädigt und wurde nicht installiert. Ihre bisherige Version bleibt unverändert.",
        }
    }
}

/// Woher Updates kommen. Die echte Quelle fragt GitHub; Tests nutzen eine Quelle ohne Internet.
pub trait UpdateQuelle: Send {
    /// Die neueste Veröffentlichung.
    fn neueste(&self) -> Result<Release, Fehler>;
    /// Lädt eine Datei (den Installer).
    fn laden(&self, url: &str) -> Result<Vec<u8>, Fehler>;
}

/// Ergebnis einer Prüfung.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ergebnis {
    Neuer(Release),
    Aktuell,
    Fehler(Fehler),
}

/// Prüft, ob es eine neuere Version als `aktuell` gibt.
pub fn pruefen(q: &dyn UpdateQuelle, aktuell: &str) -> Ergebnis {
    match q.neueste() {
        Ok(r) => match (version(&r.version), version(aktuell)) {
            (Some(neu), Some(alt)) if neu > alt => Ergebnis::Neuer(r),
            (Some(_), Some(_)) => Ergebnis::Aktuell,
            _ => Ergebnis::Fehler(Fehler::Unbrauchbar(format!("Version {} / {aktuell}", r.version))),
        },
        Err(e) => Ergebnis::Fehler(e),
    }
}

/// SHA-256 einer Datei als Kleinbuchstaben-Hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let d = sha2::Sha256::digest(bytes);
    d.iter().map(|b| format!("{b:02x}")).collect()
}

/// Lädt den Installer nach `ordner` und prüft die Prüfsumme. Nur ein geprüfter Installer wird
/// gespeichert; bei falscher Prüfsumme bleibt nichts liegen.
pub fn installer_holen(q: &dyn UpdateQuelle, r: &Release, ordner: &Path) -> Result<PathBuf, Fehler> {
    let bytes = q.laden(&r.installer_url)?;
    if !sha256_hex(&bytes).eq_ignore_ascii_case(r.sha256.trim()) {
        return Err(Fehler::Pruefsumme);
    }
    std::fs::create_dir_all(ordner).map_err(|e| Fehler::Unbrauchbar(e.to_string()))?;
    let pfad = ordner.join(format!("SimpleGrid-Setup-{}.exe", r.version));
    std::fs::write(&pfad, &bytes).map_err(|e| Fehler::Unbrauchbar(e.to_string()))?;
    Ok(pfad)
}

/// Inhalt von `versionen.json`.
#[derive(Debug, Deserialize)]
pub struct Versionen {
    pub apps: std::collections::BTreeMap<String, String>,
    pub installer: String,
    pub sha256: String,
}

/// Die echte Quelle: öffentliche GitHub-Releases eines Repositorys (ohne Zugangsschlüssel).
pub struct GithubQuelle {
    /// „Besitzer/Repository“.
    pub repo: String,
}

/// Holt eine Adresse mit dem in Windows enthaltenen `curl.exe` (ohne Konsolenfenster).
fn holen(url: &str) -> Result<Vec<u8>, Fehler> {
    let mut cmd = std::process::Command::new("curl.exe");
    cmd.args(["-sSL", "--max-time", "60", "-H", "User-Agent: SimpleGrid-Update", "-H", "Accept: application/vnd.github+json", "-w", "\n%{http_code}", url]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let out = cmd.output().map_err(|e| Fehler::KeinNetz(e.to_string()))?;
    if !out.status.success() {
        return Err(Fehler::KeinNetz(String::from_utf8_lossy(&out.stderr).into_owned()));
    }
    let mut body = out.stdout;
    let pos = body.iter().rposition(|b| *b == b'\n').ok_or_else(|| Fehler::Unbrauchbar("keine Antwort".into()))?;
    let code: u16 = String::from_utf8_lossy(&body[pos + 1..]).trim().parse().unwrap_or(0);
    body.truncate(pos);
    match code {
        200..=299 => Ok(body),
        403 | 429 => Err(Fehler::Limit),
        0 => Err(Fehler::KeinNetz("keine Antwort".into())),
        c => Err(Fehler::Unbrauchbar(format!("HTTP {c}"))),
    }
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct GhRelease {
    assets: Vec<Asset>,
}

/// Wertet die Release-Antwort und `versionen.json` aus (ohne Netz, für Tests getrennt).
pub fn release_aus(release_json: &[u8], versionen_json: &[u8]) -> Result<Release, Fehler> {
    let gh: GhRelease = serde_json::from_slice(release_json).map_err(|e| Fehler::Unbrauchbar(e.to_string()))?;
    let v: Versionen = serde_json::from_slice(versionen_json).map_err(|e| Fehler::Unbrauchbar(e.to_string()))?;
    let version = v.apps.get(APP).cloned().ok_or_else(|| Fehler::Unbrauchbar("SimpleGrid fehlt in versionen.json".into()))?;
    let url = gh.assets.iter().find(|a| a.name == v.installer).map(|a| a.browser_download_url.clone()).ok_or_else(|| Fehler::Unbrauchbar("Installer fehlt".into()))?;
    Ok(Release { version, installer_url: url, sha256: v.sha256 })
}

impl UpdateQuelle for GithubQuelle {
    fn neueste(&self) -> Result<Release, Fehler> {
        let repo = self.repo.trim();
        if repo.split('/').count() != 2 || repo.chars().any(|c| !(c.is_ascii_alphanumeric() || "-_./".contains(c))) {
            return Err(Fehler::Unbrauchbar(format!("Update-Quelle `{repo}`")));
        }
        let release = holen(&format!("https://api.github.com/repos/{repo}/releases/latest"))?;
        let gh: GhRelease = serde_json::from_slice(&release).map_err(|e| Fehler::Unbrauchbar(e.to_string()))?;
        let url = gh.assets.iter().find(|a| a.name == "versionen.json").map(|a| a.browser_download_url.clone()).ok_or_else(|| Fehler::Unbrauchbar("versionen.json fehlt".into()))?;
        let versionen = holen(&url)?;
        release_aus(&release, &versionen)
    }

    fn laden(&self, url: &str) -> Result<Vec<u8>, Fehler> {
        if !url.starts_with("https://") {
            return Err(Fehler::Unbrauchbar("nur https".into()));
        }
        holen(url)
    }
}

/// Ob heute schon automatisch geprüft wurde (`letzte` und `heute` als „JJJJ-MM-TT“).
pub fn heute_schon_geprueft(letzte: &str, heute: &str) -> bool {
    letzte == heute
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestQuelle {
        neueste: Result<Release, Fehler>,
        datei: Vec<u8>,
    }

    impl UpdateQuelle for TestQuelle {
        fn neueste(&self) -> Result<Release, Fehler> {
            self.neueste.clone()
        }
        fn laden(&self, _url: &str) -> Result<Vec<u8>, Fehler> {
            Ok(self.datei.clone())
        }
    }

    fn release(v: &str, datei: &[u8]) -> Release {
        Release { version: v.into(), installer_url: "https://example.invalid/setup.exe".into(), sha256: sha256_hex(datei) }
    }

    #[test]
    fn neuere_version_wird_erkannt() {
        let q = TestQuelle { neueste: Ok(release("1.2.0", b"x")), datei: b"x".to_vec() };
        assert_eq!(pruefen(&q, "1.1.9"), Ergebnis::Neuer(release("1.2.0", b"x")));
        assert_eq!(pruefen(&q, "1.10.0"), Ergebnis::Aktuell, "1.10 ist neuer als 1.2");
    }

    #[test]
    fn gleiche_version_ist_aktuell() {
        let q = TestQuelle { neueste: Ok(release("v1.2.0", b"x")), datei: vec![] };
        assert_eq!(pruefen(&q, "1.2.0"), Ergebnis::Aktuell);
    }

    #[test]
    fn kein_netz_bleibt_ein_fehler_ohne_absturz() {
        let q = TestQuelle { neueste: Err(Fehler::KeinNetz("offline".into())), datei: vec![] };
        assert_eq!(pruefen(&q, "1.0.0"), Ergebnis::Fehler(Fehler::KeinNetz("offline".into())));
        let q = TestQuelle { neueste: Err(Fehler::Limit), datei: vec![] };
        assert!(matches!(pruefen(&q, "1.0.0"), Ergebnis::Fehler(Fehler::Limit)));
    }

    #[test]
    fn kaputte_pruefsumme_installiert_nichts() {
        let ordner = std::env::temp_dir().join(format!("simplegrid-update-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&ordner);
        let mut r = release("2.0.0", b"echt");
        let q = TestQuelle { neueste: Ok(r.clone()), datei: b"manipuliert".to_vec() };
        assert_eq!(installer_holen(&q, &r, &ordner), Err(Fehler::Pruefsumme));
        assert!(!ordner.join("SimpleGrid-Setup-2.0.0.exe").exists(), "nichts bleibt liegen");
        let q = TestQuelle { neueste: Ok(r.clone()), datei: b"echt".to_vec() };
        r.sha256 = r.sha256.to_uppercase();
        let pfad = installer_holen(&q, &r, &ordner).unwrap_or_default();
        assert_eq!(std::fs::read(pfad).unwrap_or_default(), b"echt");
    }

    #[test]
    fn github_antwort_und_versionen_json() {
        let gh = br#"{"tag_name":"v1.3.0","assets":[{"name":"versionen.json","browser_download_url":"https://x/versionen.json"},{"name":"SimpleGrid-Setup.exe","browser_download_url":"https://x/SimpleGrid-Setup.exe"}]}"#;
        let v = br#"{"apps":{"SimpleGrid":"1.3.0"},"installer":"SimpleGrid-Setup.exe","sha256":"abc"}"#;
        assert_eq!(release_aus(gh, v), Ok(Release { version: "1.3.0".into(), installer_url: "https://x/SimpleGrid-Setup.exe".into(), sha256: "abc".into() }));
        assert!(release_aus(gh, b"{}").is_err());
        assert!(matches!(GithubQuelle { repo: "kein repo; rm -rf".into() }.neueste(), Err(Fehler::Unbrauchbar(_))));
    }

    #[test]
    fn versionen_lesen() {
        assert_eq!(version("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(version("2"), Some((2, 0, 0)));
        assert_eq!(version("abc"), None);
        assert!(heute_schon_geprueft("2026-11-10", "2026-11-10"));
        assert!(!heute_schon_geprueft("2026-11-09", "2026-11-10"));
    }
}
