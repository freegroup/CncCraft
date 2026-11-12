#!/usr/bin/env bash
# CncCraft: die macOS-Pakete lokal bauen und an eine GitHub-Release hängen.
#
# Hintergrund: Die GitHub-Action (.github/workflows/release.yml) baut Windows und Linux.
# Für macOS gibt es keinen passenden GitHub-Runner, also werden die Apple-Pakete hier auf
# einem Mac gebaut und mit diesem Skript an die gewünschte Release gehängt.
#
#   ./build_and_release.sh
#
# Läuft nur auf einem Mac und braucht die GitHub-CLI (`gh`, am freegroup-Konto angemeldet).
set -euo pipefail
cd "$(dirname "$0")"

NAME="CncCraft"
VERSION="$(sed -n 's/^version *= *"\(.*\)"/\1/p' Cargo.toml | head -1)"
DIST="dist"

say() { printf '\n==> %s\n' "$*"; }

# -------------------------------------------------- Voraussetzungen
[ "$(uname -s)" = "Darwin" ] || { echo "Dieses Skript baut macOS-Pakete und läuft nur auf einem Mac."; exit 1; }
command -v gh >/dev/null 2>&1 || { echo "GitHub-CLI 'gh' nicht gefunden (brew install gh)."; exit 1; }
gh auth status >/dev/null 2>&1 || { echo "Bitte zuerst mit 'gh auth login' anmelden."; exit 1; }

REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || echo "freegroup/$NAME")"

# -------------------------------------------------- 1) macOS bauen
say "macOS (Apple Silicon) bauen"
./build.sh mac
say "macOS (Intel) bauen"
./build.sh mac-intel || echo "Intel-Build übersprungen (läuft nur mit Rosetta)."

# macOS-Pakete einsammeln, die gerade entstanden sind
shopt -s nullglob
MAC_ASSETS=( "$DIST"/*apple-silicon.dmg "$DIST"/*apple-silicon.zip "$DIST"/*intel-mac.dmg "$DIST"/*intel-mac.zip )
shopt -u nullglob
[ "${#MAC_ASSETS[@]}" -gt 0 ] || { echo "Keine macOS-Pakete in $DIST/ gefunden – Build fehlgeschlagen?"; exit 1; }

# -------------------------------------------------- 2) Release-Version wählen (die letzten 5, latest vorselektiert)
TAGS=()
while IFS= read -r line; do [ -n "$line" ] && TAGS+=("$line"); done \
  < <(gh release list --limit 5 --json tagName -q '.[].tagName' 2>/dev/null || true)

if [ "${#TAGS[@]}" -eq 0 ]; then
  # noch keine Releases: Tag eingeben und anlegen
  DEFAULT_TAG="v$VERSION"
  printf 'Keine Releases gefunden. Welchen Tag anlegen? [%s] ' "$DEFAULT_TAG"
  read -r TAG
  TAG="${TAG:-$DEFAULT_TAG}"
  gh release create "$TAG" --title "$TAG" --notes "CncCraft $TAG"
else
  echo
  echo "Welche Release? (die letzten ${#TAGS[@]} von GitHub)"
  i=1
  for t in "${TAGS[@]}"; do
    if [ "$i" -eq 1 ]; then printf '  %d) %s  (latest)\n' "$i" "$t"; else printf '  %d) %s\n' "$i" "$t"; fi
    i=$((i + 1))
  done
  printf 'Auswahl [1] '
  read -r sel
  sel="${sel:-1}"
  case "$sel" in ''|*[!0-9]*) echo "Ungültige Auswahl."; exit 1 ;; esac
  { [ "$sel" -ge 1 ] && [ "$sel" -le "${#TAGS[@]}" ]; } || { echo "Auswahl außerhalb des Bereichs."; exit 1; }
  TAG="${TAGS[$((sel - 1))]}"
fi

# -------------------------------------------------- 3) Artefakte anhängen
say "Diese macOS-Pakete an Release '$TAG' hängen:"
printf '  %s\n' "${MAC_ASSETS[@]}"
gh release upload "$TAG" "${MAC_ASSETS[@]}" --clobber

say "Fertig: https://github.com/$REPO/releases/tag/$TAG"
