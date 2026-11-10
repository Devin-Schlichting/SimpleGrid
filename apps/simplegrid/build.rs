//! Windows only: embed the app icon and version info (VERSIONINFO) into `simplegrid.exe`, so it
//! shows in Explorer, the taskbar, the Start menu and Alt-Tab.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `SIMPLEGRID_REQUIRE_WINRES=1` turns it
//! into an error (for release builds).

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/simplegrid.ico");
    println!("cargo:rerun-if-env-changed=SIMPLEGRID_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/simplegrid.ico")
        .set("ProductName", "SimpleGrid")
        .set("FileDescription", "SimpleGrid – einfaches Tabellenprogramm")
        .set("CompanyName", "SimpleGrid")
        .set("LegalCopyright", "Beruht auf GridCraft (c) the GridCraft authors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "simplegrid.exe")
        .set("InternalName", "simplegrid");
    if let Err(e) = res.compile() {
        if std::env::var_os("SIMPLEGRID_REQUIRE_WINRES").is_some() {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=simplegrid.exe built without icon/version resources: {e}");
    }
}
