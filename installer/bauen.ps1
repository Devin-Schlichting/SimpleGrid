# SimpleGrid: Programm bauen, testen, Installer erzeugen und versionen.json schreiben.
#
#   powershell -ExecutionPolicy Bypass -File installer\bauen.ps1 [-Version 1.2.0] [-OhneTests]
#
# Ergebnis in dist\: SimpleGrid-Setup.exe und versionen.json (mit SHA-256 des Installers).
# Voraussetzungen: Rust (cargo) und Inno Setup 6 (ISCC.exe). Es werden keine Zugangsdaten benutzt.
param(
    [string]$Version = "",
    [switch]$OhneTests
)
$ErrorActionPreference = "Stop"
$Wurzel = Split-Path -Parent $PSScriptRoot
Set-Location $Wurzel

if ($Version -eq "") {
    # Version aus Cargo.toml ([workspace.package] version = "x.y.z")
    $Version = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
}
$Version = $Version.TrimStart("v")
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "Version muss Haupt.Neben.Fehlerbehebung sein, z. B. 1.2.0 (war: $Version)" }
Write-Host "SimpleGrid $Version"

# Version in Cargo.toml eintragen, damit das Programm sie selbst kennt.
(Get-Content Cargo.toml) -replace '^version = ".*"', "version = `"$Version`"" | Set-Content Cargo.toml -Encoding utf8

if (-not $OhneTests) {
    cargo test --workspace
    if ($LASTEXITCODE -ne 0) { throw "Tests fehlgeschlagen" }
}
$env:SIMPLEGRID_REQUIRE_WINRES = "1"
cargo build --release -p simplegrid
if ($LASTEXITCODE -ne 0) { throw "Bauen fehlgeschlagen" }

$Iscc = @("${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe", "$env:ProgramFiles\Inno Setup 6\ISCC.exe", "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe") | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $Iscc) { throw "Inno Setup 6 (ISCC.exe) nicht gefunden" }
New-Item -ItemType Directory -Force dist | Out-Null
& $Iscc "/DAppVersion=$Version" "/DQuelle=$Wurzel\target\release" installer\SimpleGrid.iss
if ($LASTEXITCODE -ne 0) { throw "Installer fehlgeschlagen" }

$Hash = (Get-FileHash dist\SimpleGrid-Setup.exe -Algorithm SHA256).Hash.ToLower()
$Json = [ordered]@{ apps = [ordered]@{ SimpleGrid = $Version }; installer = "SimpleGrid-Setup.exe"; sha256 = $Hash } | ConvertTo-Json
[System.IO.File]::WriteAllText("$Wurzel\dist\versionen.json", $Json, (New-Object System.Text.UTF8Encoding $false))
Write-Host "Fertig: dist\SimpleGrid-Setup.exe ($Hash)"
