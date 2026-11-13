#!/usr/bin/env bash
# CncCraft: die macOS-Pakete lokal bauen und an die passende GitHub-Release hängen.
#
# Hintergrund: Die GitHub-Action (.github/workflows/release.yml) baut Windows und Linux und
# lässt semantic-release die Version in Cargo.toml/Cargo.lock bumpen, einchecken und taggen.
# Für macOS gibt es keinen passenden GitHub-Runner, also werden die Apple-Pakete hier auf
# einem Mac gebaut und an dieselbe Release gehängt.
#
#   ./build_and_release.sh
#
# Die Version wird NICHT ausgewählt: Cargo.toml gibt sie vor. Das Skript holt zuerst den
# aktuellen main-Stand (git pull), damit der von semantic-release gebumpte Versionsstand
# lokal vorliegt, baut genau diese Version und hängt die Pakete an die Release "v<version>".
#
# Läuft nur auf einem Mac und braucht die GitHub-CLI (`gh`, am freegroup-Konto angemeldet).
set -euo pipefail
cd "$(dirname "$0")"

NAME="CncCraft"
DIST="dist"

say() { printf '\n==> %s\n' "$*"; }
die() { echo "FEHLER: $*" >&2; exit 1; }

# -------------------------------------------------- Voraussetzungen
[ "$(uname -s)" = "Darwin" ] || die "Dieses Skript baut macOS-Pakete und läuft nur auf einem Mac."
command -v gh >/dev/null 2>&1 || die "GitHub-CLI 'gh' nicht gefunden (brew install gh)."
gh auth status >/dev/null 2>&1 || die "Bitte zuerst mit 'gh auth login' anmelden."

# -------------------------------------------------- 1) Aktuellen Stand holen (Cargo.toml gibt die Version vor)
branch="$(git rev-parse --abbrev-ref HEAD)"
[ "$branch" = "main" ] || die "Bitte auf dem Branch 'main' bauen (aktuell: '$branch')."
[ -z "$(git status --porcelain)" ] || die "Arbeitsverzeichnis nicht sauber – bitte erst committen/stashen, dann erneut ausführen."

say "Aktuellen main-Stand holen (git pull)"
git pull --ff-only

VERSION="$(sed -n 's/^version *= *"\(.*\)"/\1/p' Cargo.toml | head -1)"
[ -n "$VERSION" ] || die "Konnte die Version nicht aus Cargo.toml lesen."
TAG="v$VERSION"
REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || echo "freegroup/$NAME")"

# Die Release muss schon existieren (semantic-release legt sie beim Push auf main an).
gh release view "$TAG" >/dev/null 2>&1 \
  || die "Release '$TAG' existiert nicht. Läuft der Release-Workflow für diese Version noch? (Cargo.toml = $VERSION)"

say "Version $VERSION → Release $TAG"

# -------------------------------------------------- 2) alte macOS-Pakete entfernen und neu bauen
shopt -s nullglob
rm -f "$DIST"/*apple-silicon.dmg "$DIST"/*apple-silicon.zip "$DIST"/*intel-mac.dmg "$DIST"/*intel-mac.zip
shopt -u nullglob

say "macOS (Apple Silicon) bauen"
./build.sh mac
say "macOS (Intel) bauen"
./build.sh mac-intel || echo "Intel-Build übersprungen (läuft nur mit Rosetta)."

# genau die frisch gebauten macOS-Pakete einsammeln (dist wurde vorher geleert)
shopt -s nullglob
MAC_ASSETS=( "$DIST"/*apple-silicon.dmg "$DIST"/*apple-silicon.zip "$DIST"/*intel-mac.dmg "$DIST"/*intel-mac.zip )
shopt -u nullglob
[ "${#MAC_ASSETS[@]}" -gt 0 ] || die "Keine macOS-Pakete in $DIST/ gefunden – Build fehlgeschlagen?"

# -------------------------------------------------- 3) Artefakte anhängen
say "Diese macOS-Pakete an Release '$TAG' hängen:"
printf '  %s\n' "${MAC_ASSETS[@]}"
gh release upload "$TAG" "${MAC_ASSETS[@]}" --clobber

say "Fertig: https://github.com/$REPO/releases/tag/$TAG"
