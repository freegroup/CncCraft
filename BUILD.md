# CncCraft bauen und verteilen

Alles läuft über `build.sh` (oder kurz `make`). Fertige Pakete landen in `dist/`.

| Befehl | Ergebnis | Wo es läuft |
|---|---|---|
| `./build.sh mac` | `CncCraft.app`, `CncCraft-<Version>-apple-silicon.zip` und `.dmg` (Apple Silicon) | auf einem Mac |
| `./build.sh mac-intel` | dasselbe für Intel-Macs | auf einem Mac |
| `./build.sh windows` | `CncCraft.exe`, `.zip` und – mit Inno Setup – `CncCraft-<Version>-windows-x86_64-setup.exe` | unter Windows (Git Bash), sonst mit `cargo xwin` oder `cross` |
| `./build.sh linux` | `.tar.gz` und `.deb` (x86_64) | unter Linux, sonst mit `cross` (Docker) |
| `./build.sh all` | alles, was auf diesem Rechner geht | |

**Releases (automatisch per Semantic Versioning):** `.github/workflows/release.yml` nutzt
[semantic-release](https://semantic-release.gitbook.io/) mit [Conventional Commits](https://www.conventionalcommits.org/).
Bei jedem Push auf `main` wird die Version aus den Commit-Messages abgeleitet:
`fix:` → Patch, `feat:` → Minor, `feat!:`/`BREAKING CHANGE:` → Major; `docs:`/`chore:`/`ci:`/`refactor:` lösen kein Release aus.
Ein passender Commit bumpt `Cargo.toml`, pflegt `CHANGELOG.md`, taggt, legt die GitHub-Release an und baut Windows + Linux dran.
Die **macOS-Pakete** werden lokal auf einem Mac gebaut und mit `./build_and_release.sh` an die Release gehängt –
GitHubs gehostete Apple-Silicon-Runner sind nicht auf jedem Konto verfügbar.

Hinweise
- Die Version steht in `Cargo.toml` (`version = …`) und geht in alle Dateinamen ein.
- Das Programmsymbol wird im Programm selbst gezeichnet: `cnccraft --export-icon bild.png` (macOS-`.icns` und
  Linux-Symbol entstehen daraus) und `cnccraft --export-ico bild.ico` (Windows).
- macOS-Signatur: Ohne Zusatz ist die App nur „ad hoc" signiert (läuft nur auf dem Build-Mac; andere Macs
  zeigen „nicht verifiziert"). Für die Weitergabe mit einem Apple-Developer-Account signiert und notarisiert
  `build.sh` automatisch, wenn diese Umgebungsvariablen gesetzt sind:
  `CODESIGN_IDENTITY` (z. B. `Developer ID Application: Dein Name (TEAMID)`) und `NOTARY_PROFILE`
  (ein mit `xcrun notarytool store-credentials` angelegtes Schlüsselbund-Profil). Einrichtung:
  1. „Developer ID Application"-Zertifikat anlegen (Xcode → Einstellungen → Accounts → Team → Manage Certificates → +),
  2. App-spezifisches Passwort auf appleid.apple.com erstellen,
  3. `xcrun notarytool store-credentials "CncCraft-Notary" --apple-id DEINE@MAIL --team-id TEAMID --password APP-PASSWORT`,
  4. `export CODESIGN_IDENTITY="Developer ID Application: …" NOTARY_PROFILE="CncCraft-Notary"` und dann `./build_and_release.sh`.
- Windows: Der Installer braucht Inno Setup (`iscc`); `packaging/cnccraft.iss` beschreibt ihn.
- Geprüft ist bisher nur der Mac-Weg (Apple Silicon); die Windows- und Linux-Wege habe ich hier nicht ausführen können.

## Website (GitHub Pages)

Die Onlinehilfe liegt fertig in `docs/` (Produktseite `docs/index.html`, Hilfe unter `docs/hilfe/`). Sie ist reines
HTML, CSS und JavaScript ohne Server; die Suche läuft im Browser.

- **Veröffentlichen:** In den GitHub-Einstellungen des Repositories unter *Pages* als Quelle den Zweig `main` und den
  Ordner `/docs` wählen. `docs/.nojekyll` verhindert, dass GitHub die Dateien mit Jekyll umbaut.
- **Texte ändern:** Die Seiten stehen in `tools/docs_pages/*.py`, das Aussehen in `tools/docs_assets/` und
  `tools/docs_template.py`. Danach `make docs` (baut neu und prüft alle Links und Bilder).
- **Abbildungen** sind Aufnahmen von CncCraft selbst: `cnccraft --screenshot docs/assets/img/<name>.png`
  (mit den `CNCCRAFT_SHOW_…`-Schaltern aus `src/app.rs` für Dialoge).
- Beschrieben wird nur, was CncCraft kann; Neues gegenüber dem Original trägt in der Hilfe das Zeichen „Neu“
  (`"neu": True` an der Seite).
