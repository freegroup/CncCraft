#!/usr/bin/env bash
# CncCraft: Programm und Installationspakete bauen.
#
#   ./build.sh mac        Apple Silicon: CncCraft.app, .zip und .dmg      (auf einem Mac)
#   ./build.sh mac-intel  dasselbe für Intel-Macs                         (auf einem Mac)
#   ./build.sh linux      Linux x86_64: .tar.gz und, wenn möglich, .deb   (auf Linux, sonst mit `cross`)
#   ./build.sh windows    Windows x86_64: .exe, .zip und Installer         (auf Windows, sonst mit `cargo xwin`/`cross`)
#   ./build.sh all        alles, was auf diesem Rechner geht
#   ./build.sh clean      dist/ und target/ entfernen
#
# Fertige Pakete landen in dist/. Für alle Systeme ohne eigene Rechner: siehe
# .github/workflows/release.yml (baut bei jedem Versions-Tag auf macOS, Windows und Linux).
set -euo pipefail
cd "$(dirname "$0")"

NAME="CncCraft"
BIN="cnccraft"
VERSION="$(sed -n 's/^version *= *"\(.*\)"/\1/p' Cargo.toml | head -1)"
DIST="dist"
mkdir -p "$DIST"

say() { printf '\n==> %s\n' "$*"; }
have() { command -v "$1" >/dev/null 2>&1; }

build_target() { # $1 = Rust-Ziel
  rustup target add "$1" >/dev/null 2>&1 || true
  cargo build --release --target "$1"
}

# ---------------------------------------------------------------- macOS
mac_app() { # $1 = Rust-Ziel, $2 = Name im Dateinamen
  local target="$1" tag="$2"
  [ "$(uname -s)" = "Darwin" ] || { echo "macOS-Pakete lassen sich nur auf einem Mac bauen."; return 1; }
  say "macOS ($tag): bauen"
  build_target "$target"
  local app="$DIST/$NAME.app"
  rm -rf "$app"
  mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
  cp "target/$target/release/$BIN" "$app/Contents/MacOS/$BIN"
  # das Symbol: PNG -> .icns
  local set
  set="$(mktemp -d)/AppIcon.iconset"
  mkdir -p "$set"
  "target/$target/release/$BIN" --export-icon "$set/base.png" >/dev/null
  for s in 16 32 64 128 256 512; do
    sips -z $s $s "$set/base.png" --out "$set/icon_${s}x${s}.png" >/dev/null
    sips -z $((s * 2)) $((s * 2)) "$set/base.png" --out "$set/icon_${s}x${s}@2x.png" >/dev/null
  done
  rm "$set/base.png"
  iconutil -c icns "$set" -o "$app/Contents/Resources/AppIcon.icns"
  cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>$NAME</string>
  <key>CFBundleDisplayName</key><string>$NAME</string>
  <key>CFBundleIdentifier</key><string>de.cnccraft.app</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleExecutable</key><string>$BIN</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>CFBundleDocumentTypes</key><array><dict>
    <key>CFBundleTypeName</key><string>CncCraft-Projekt</string>
    <key>CFBundleTypeExtensions</key><array><string>cnccraft</string><string>funke</string></array>
    <key>CFBundleTypeRole</key><string>Editor</string>
  </dict></array>
</dict></plist>
PLIST
  # Signieren. Mit "Developer ID Application" (Umgebungsvariable CODESIGN_IDENTITY) für die
  # Weitergabe an andere Macs; sonst ad-hoc, was nur auf diesem Mac ohne Meldung läuft.
  if [ -n "${CODESIGN_IDENTITY:-}" ]; then
    say "macOS ($tag): mit Developer ID signieren"
    codesign --force --options runtime --timestamp --sign "$CODESIGN_IDENTITY" "$app/Contents/MacOS/$BIN"
    codesign --force --options runtime --timestamp --sign "$CODESIGN_IDENTITY" "$app"
  else
    codesign --force --deep --sign - "$app" >/dev/null 2>&1 || true
  fi

  local notarize=""
  if [ -n "${CODESIGN_IDENTITY:-}" ] && [ -n "${NOTARY_PROFILE:-}" ]; then notarize=1; fi

  # DMG zuerst bauen, damit eine einzige Notarisierung App und DMG abdeckt.
  local dmg="$DIST/$NAME-$VERSION-$tag.dmg" stage
  if have hdiutil; then
    stage="$(mktemp -d)"
    cp -R "$app" "$stage/"
    ln -s /Applications "$stage/Applications"
    rm -f "$dmg"
    hdiutil create -quiet -volname "$NAME" -srcfolder "$stage" -ov -format UDZO "$dmg"
  fi

  # Notarisieren (NOTARY_PROFILE = mit `xcrun notarytool store-credentials` angelegtes Profil).
  # Das DMG enthält die App, also deckt seine Einreichung beide ab; danach beide stapeln,
  # damit Gatekeeper auch offline zufrieden ist.
  if [ -n "$notarize" ]; then
    say "macOS ($tag): notarisieren (kann einige Minuten dauern)"
    if [ -f "$dmg" ]; then
      xcrun notarytool submit "$dmg" --keychain-profile "$NOTARY_PROFILE" --wait
      xcrun stapler staple "$dmg"
      xcrun stapler staple "$app"
    else
      local nzip; nzip="$(mktemp -d)/$NAME.zip"
      ditto -c -k --keepParent "$app" "$nzip"
      xcrun notarytool submit "$nzip" --keychain-profile "$NOTARY_PROFILE" --wait
      xcrun stapler staple "$app"
    fi
  elif [ -n "${CODESIGN_IDENTITY:-}" ]; then
    echo "(NOTARY_PROFILE nicht gesetzt – signiert, aber nicht notarisiert.)"
  fi

  # ZIP aus der – jetzt ggf. gestapelten – App erzeugen.
  local zip="$DIST/$NAME-$VERSION-$tag.zip"
  rm -f "$zip"
  (cd "$DIST" && ditto -c -k --keepParent "$NAME.app" "$(basename "$zip")")
  say "macOS ($tag): $zip"
  if [ -f "$dmg" ]; then say "macOS ($tag): $dmg"; fi
}

# ---------------------------------------------------------------- Linux
linux_pkg() {
  local target="x86_64-unknown-linux-gnu" out="$DIST/$NAME-$VERSION-linux-x86_64"
  say "Linux: bauen"
  if [ "$(uname -s)" = "Linux" ]; then
    build_target "$target"
  elif have cross; then
    cross build --release --target "$target" # braucht Docker
  else
    echo "Auf diesem System braucht Linux entweder 'cross' (cargo install cross, plus Docker) oder den GitHub-Workflow."
    return 1
  fi
  rm -rf "$out"
  mkdir -p "$out"
  cp "target/$target/release/$BIN" "$out/$BIN"
  cp assets/icon.png "$out/$BIN.png"
  cp packaging/cnccraft.desktop "$out/"
  cat > "$out/README.txt" <<TXT
CncCraft $VERSION
Starten:  ./$BIN
Als Anwendung eintragen: $BIN nach ~/.local/bin, $BIN.png nach ~/.local/share/icons,
cnccraft.desktop nach ~/.local/share/applications kopieren.
Benötigt: OpenGL (Mesa) und X11 oder Wayland.
TXT
  tar -C "$DIST" -czf "$out.tar.gz" "$(basename "$out")"
  say "Linux: $out.tar.gz"
  if have dpkg-deb; then
    local deb
    deb="$(mktemp -d)"
    mkdir -p "$deb/DEBIAN" "$deb/usr/bin" "$deb/usr/share/applications" "$deb/usr/share/icons/hicolor/512x512/apps"
    cp "$out/$BIN" "$deb/usr/bin/$BIN"
    cp "$out/$BIN.png" "$deb/usr/share/icons/hicolor/512x512/apps/$BIN.png"
    sed 's|^Exec=.*|Exec=/usr/bin/cnccraft|' packaging/cnccraft.desktop > "$deb/usr/share/applications/cnccraft.desktop"
    cat > "$deb/DEBIAN/control" <<CTRL
Package: cnccraft
Version: $VERSION
Section: graphics
Priority: optional
Architecture: amd64
Depends: libgl1, libx11-6, libxkbcommon0
Maintainer: CncCraft <openjacob@gmail.com>
Description: CNC-Entwurfs- und Fräsprogramm
 Entwerfen, Werkzeugwege berechnen und G-Code für die CNC-Fräse ausgeben.
CTRL
    dpkg-deb --build "$deb" "$DIST/cnccraft_${VERSION}_amd64.deb" >/dev/null
    say "Linux: $DIST/cnccraft_${VERSION}_amd64.deb"
  fi
}

# ---------------------------------------------------------------- Windows
windows_pkg() {
  local target="x86_64-pc-windows-msvc" out="$DIST/$NAME-$VERSION-windows-x86_64"
  say "Windows: bauen"
  case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN*) build_target "$target" ;;
    *)
      if have cargo-xwin; then
        rustup target add "$target" >/dev/null 2>&1 || true
        cargo xwin build --release --target "$target"
      elif have cross; then
        target="x86_64-pc-windows-gnu"
        cross build --release --target "$target" # braucht Docker
      else
        echo "Windows-Pakete brauchen entweder Windows, 'cargo xwin' (cargo install cargo-xwin), 'cross' (plus Docker) oder den GitHub-Workflow."
        return 1
      fi
      ;;
  esac
  rm -rf "$out"
  mkdir -p "$out"
  cp "target/$target/release/$BIN.exe" "$out/$NAME.exe"
  cp assets/icon.png "$out/$NAME.png"
  # das Windows-Symbol (.ico) aus dem Programm selbst, wenn es hier laufen kann
  "target/$target/release/$BIN.exe" --export-ico "$out/$NAME.ico" >/dev/null 2>&1 || true
  if have zip; then
    (cd "$DIST" && zip -qr "$NAME-$VERSION-windows-x86_64.zip" "$(basename "$out")")
  fi
  say "Windows: $out/$NAME.exe"
  # der Installer: Inno Setup (unter Windows oder mit `iscc` im Pfad)
  if have iscc; then
    # In Git Bash die Pfade nach Windows-Form wandeln und die /D-Argumente vor der
    # MSYS-Pfadumwandlung schützen – sonst meldet iscc "more than one script filename".
    local src_iss out_iss iss_file
    if have cygpath; then
      src_iss="$(cygpath -w "$(cd "$out" && pwd)")"
      out_iss="$(cygpath -w "$(pwd)/$DIST")"
      iss_file="$(cygpath -w "$(pwd)/packaging/cnccraft.iss")"
    else
      src_iss="$(cd "$out" && pwd)"; out_iss="$(pwd)/$DIST"; iss_file="packaging/cnccraft.iss"
    fi
    MSYS2_ARG_CONV_EXCL='*' MSYS_NO_PATHCONV=1 \
      iscc "/DAppVersion=$VERSION" "/DSourceDir=$src_iss" "/DOutDir=$out_iss" "$iss_file"
    say "Windows: Installer in $DIST/"
  else
    echo "(Für einen Installer: Inno Setup installieren und packaging/cnccraft.iss mit 'iscc' ausführen; der GitHub-Workflow macht das automatisch.)"
  fi
}

case "${1:-help}" in
  mac) mac_app aarch64-apple-darwin apple-silicon ;;
  mac-intel) mac_app x86_64-apple-darwin intel-mac ;;
  linux) linux_pkg ;;
  windows) windows_pkg ;;
  all)
    if [ "$(uname -s)" = "Darwin" ]; then mac_app aarch64-apple-darwin apple-silicon || true; fi
    linux_pkg || true
    windows_pkg || true
    ;;
  clean) rm -rf "$DIST" target ;;
  *) sed -n '2,12p' "$0" ;;
esac
