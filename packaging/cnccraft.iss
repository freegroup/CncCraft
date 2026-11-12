; Inno Setup: Windows-Installer für CncCraft.
; Aufruf:  iscc /DAppVersion=0.1.0 /DSourceDir=dist\CncCraft-0.1.0-windows-x86_64 /DOutDir=dist packaging\cnccraft.iss
#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif
#ifndef SourceDir
  #define SourceDir "..\dist\CncCraft-windows-x86_64"
#endif
#ifndef OutDir
  #define OutDir "..\dist"
#endif

[Setup]
AppId={{6C1F1B5A-4C5E-4B0E-9E8A-2D6F3B7A0C11}
AppName=CncCraft
AppVersion={#AppVersion}
AppPublisher=CncCraft
DefaultDirName={autopf}\CncCraft
DefaultGroupName=CncCraft
UninstallDisplayIcon={app}\CncCraft.exe
OutputDir={#OutDir}
OutputBaseFilename=CncCraft-{#AppVersion}-windows-x86_64-setup
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
PrivilegesRequiredOverridesAllowed=dialog

[Languages]
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Symbol auf dem Desktop anlegen"; GroupDescription: "Zusätzliche Symbole:"
Name: "assoc"; Description: "Dateien .cnccraft mit CncCraft öffnen"; GroupDescription: "Dateien:"

[Files]
Source: "{#SourceDir}\CncCraft.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\CncCraft.ico"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{group}\CncCraft"; Filename: "{app}\CncCraft.exe"
Name: "{autodesktop}\CncCraft"; Filename: "{app}\CncCraft.exe"; Tasks: desktopicon

[Registry]
Root: HKA; Subkey: "Software\Classes\.cnccraft"; ValueType: string; ValueData: "CncCraft.Project"; Flags: uninsdeletevalue; Tasks: assoc
Root: HKA; Subkey: "Software\Classes\CncCraft.Project\shell\open\command"; ValueType: string; ValueData: """{app}\CncCraft.exe"" ""%1"""; Flags: uninsdeletekey; Tasks: assoc

[Run]
Filename: "{app}\CncCraft.exe"; Description: "CncCraft starten"; Flags: nowait postinstall skipifsilent
