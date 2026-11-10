; SimpleGrid – Installer (Inno Setup 6)
;
; Ein Installer für die ganze Programmfamilie: heute die Tabellenkalkulation, später das
; Textprogramm als weitere Komponente. Eine weitere App hinzufügen heißt: einen Eintrag unter
; [Components], ihre Dateien unter [Files] und ihre Verknüpfungen unter [Icons] – der Aufbau bleibt.
;
; Bauen: installer\bauen.ps1 (setzt Version und Quellordner).
; Installation pro Benutzer, ohne Administratorrechte, nach %LocalAppData%\Programs\SimpleGrid.

#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif
#ifndef Quelle
  #define Quelle "..\target\release"
#endif
#define AppName "SimpleGrid"

[Setup]
; Die AppId bleibt für immer gleich: so erkennt der Installer frühere Installationen (Update).
AppId={{3A96D016-7637-42E8-8567-0FDC70D7A7BC}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppName}
DefaultDirName={localappdata}\Programs\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
DisableDirPage=yes
DisableReadyPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=
OutputDir=..\dist
OutputBaseFilename=SimpleGrid-Setup
SetupIconFile=..\assets\app-icon\simplegrid.ico
UninstallDisplayIcon={app}\simplegrid.exe
UninstallDisplayName={#AppName}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=yes
RestartApplications=no
UsePreviousTasks=yes
UsePreviousSetupType=yes
ChangesAssociations=yes

[Languages]
Name: "de"; MessagesFile: "compiler:Languages\German.isl"

[Types]
Name: "voll"; Description: "Alle Programme"
Name: "eigene"; Description: "Auswahl"; Flags: iscustom

[Components]
; Gemeinsame Dateien (Lizenzen, später gemeinsame Bausteine): immer dabei.
Name: "basis"; Description: "Gemeinsame Dateien"; Types: voll eigene; Flags: fixed
Name: "tabellen"; Description: "SimpleGrid (Tabellen)"; Types: voll eigene
; Später: Name: "text"; Description: "SimpleOffice Text"; Types: voll eigene

[Tasks]
Name: "desktop"; Description: "Verknüpfung auf dem Desktop anlegen"; Components: tabellen
Name: "xlsx"; Description: ".xlsx-Dateien mit SimpleGrid öffnen"; Components: tabellen; Flags: unchecked

[Files]
Source: "..\LICENSE-MIT"; DestDir: "{app}"; Components: basis; Flags: ignoreversion
Source: "..\LICENSE-APACHE"; DestDir: "{app}"; Components: basis; Flags: ignoreversion
Source: "..\NOTICE"; DestDir: "{app}"; Components: basis; Flags: ignoreversion
Source: "{#Quelle}\simplegrid.exe"; DestDir: "{app}"; Components: tabellen; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\SimpleGrid"; Filename: "{app}\simplegrid.exe"; Components: tabellen
Name: "{autodesktop}\SimpleGrid"; Filename: "{app}\simplegrid.exe"; Components: tabellen; Tasks: desktop

[Registry]
; Dateizuordnung nur, wenn gewählt (Standard: aus). Pro Benutzer (HKCU).
Root: HKA; Subkey: "Software\Classes\.xlsx\OpenWithProgids"; ValueType: string; ValueName: "SimpleGrid.xlsx"; ValueData: ""; Flags: uninsdeletevalue; Tasks: xlsx
Root: HKA; Subkey: "Software\Classes\SimpleGrid.xlsx"; ValueType: string; ValueName: ""; ValueData: "Tabelle"; Flags: uninsdeletekey; Tasks: xlsx
Root: HKA; Subkey: "Software\Classes\SimpleGrid.xlsx\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\simplegrid.exe,0"; Tasks: xlsx
Root: HKA; Subkey: "Software\Classes\SimpleGrid.xlsx\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\simplegrid.exe"" ""%1"""; Tasks: xlsx

[Run]
; Nach einer normalen Installation: Häkchen „SimpleGrid starten“.
Filename: "{app}\simplegrid.exe"; Description: "SimpleGrid jetzt starten"; Flags: nowait postinstall skipifsilent; Components: tabellen
; Nach einem stillen Update aus dem Programm heraus (/STARTAPP): SimpleGrid wieder starten.
Filename: "{app}\simplegrid.exe"; Flags: nowait; Check: NachUpdateStarten; Components: tabellen

[UninstallDelete]
; Saubere Deinstallation: automatische Sicherungen und Update-Reste entfernen.
Type: filesandordirs; Name: "{localappdata}\SimpleGrid"
Type: filesandordirs; Name: "{userappdata}\SimpleGrid"
Type: filesandordirs; Name: "{%TEMP}\SimpleGrid-Update"

[Code]
function NachUpdateStarten: Boolean;
begin
  Result := WizardSilent and (Pos('/STARTAPP', UpperCase(GetCmdTail)) > 0);
end;
