#!/usr/bin/env bash
#
# sync.sh — Spiegelt die relevanten Quellen aus dem Original-Projekt
# ("Craft Anything/spark/app") in dieses Git-Repository (CncCraft).
#
# Ablauf:
#   1. Prüfen, dass das Ziel-Repo ein sauberer Git-Stand ist (alles committed).
#   2. Die relevanten Ziel-Ordner/-Dateien löschen (unkritisch, da in Git).
#   3. Quellen aus dem Original kopieren.
#
# Danach die Änderungen mit `git status` prüfen und committen.

set -euo pipefail

SRC="/Users/D023280/Documents/Craft Anything/spark/app"
DST="/Users/D023280/Documents/workspace/CncCraft"

# Relevante Ordner und Dateien (relativ zur Projektwurzel)
ITEMS=(
  assets
  crates
  data
  docs
  locales
  src
  tools
  Cargo.lock
  Cargo.toml
  build.rs
  Makefile
)

# Hinweis: README.md wird bewusst NICHT synchronisiert — sie ist CncCraft-eigen
# (GitHub-Projektseite) und existiert im Original nicht.

# --- Farben für Ausgabe -----------------------------------------------------
if [[ -t 1 ]]; then
  BOLD=$'\033[1m'; RED=$'\033[31m'; GREEN=$'\033[32m'; YELLOW=$'\033[33m'; RESET=$'\033[0m'
else
  BOLD=''; RED=''; GREEN=''; YELLOW=''; RESET=''
fi

info()  { echo "${GREEN}==>${RESET} $*"; }
warn()  { echo "${YELLOW}==>${RESET} $*"; }
die()   { echo "${RED}FEHLER:${RESET} $*" >&2; exit 1; }

# --- Vorbedingungen ---------------------------------------------------------
[[ -d "$SRC" ]] || die "Quellordner nicht gefunden: $SRC"
[[ -d "$DST" ]] || die "Zielordner nicht gefunden: $DST"
command -v rsync >/dev/null 2>&1 || die "rsync wird benötigt, ist aber nicht installiert."

# --- 1. Git-Stand des Ziels prüfen -----------------------------------------
cd "$DST"

git rev-parse --is-inside-work-tree >/dev/null 2>&1 \
  || die "Ziel ist kein Git-Repository: $DST"

if [[ -n "$(git status --porcelain)" ]]; then
  echo
  git status --short
  echo
  die "Das Ziel-Repo hat uncommittete Änderungen. Bitte erst committen, dann erneut ausführen."
fi

info "Git-Stand ist sauber (alles committed)."

# Die Version gehört semantic-release (der Release-Workflow bumpt und checkt Cargo.toml/Cargo.lock
# ein). Das Original trägt eine eigene, alte Version – die darf sie nicht überschreiben.
KEEP_VERSION="$(sed -n 's/^version *= *"\(.*\)"/\1/p' Cargo.toml | head -1)"

# --- 2. Ziel-Ordner/-Dateien löschen ---------------------------------------
info "Lösche vorhandene Ziele..."
for item in "${ITEMS[@]}"; do
  target="$DST/$item"
  if [[ -e "$target" ]]; then
    echo "   - entferne $item"
    rm -rf "$target"
  fi
done

# --- 3. Quellen kopieren ----------------------------------------------------
info "Kopiere Quellen aus dem Original..."
for item in "${ITEMS[@]}"; do
  source="$SRC/$item"
  if [[ ! -e "$source" ]]; then
    warn "   - Quelle fehlt, übersprungen: $item"
    continue
  fi

  if [[ -d "$source" ]]; then
    echo "   - kopiere Ordner $item/"
    # Trailing slash: Inhalt von source nach DST/item spiegeln
    rsync -a --delete "$source/" "$DST/$item/"
  else
    echo "   - kopiere Datei $item"
    rsync -a "$source" "$DST/$item"
  fi
done

# --- 4. Version des Repos beibehalten ---------------------------------------
# gleiche Muster wie in .releaserc.json (Paket-Version in Cargo.toml und Cargo.lock)
if [[ -n "$KEEP_VERSION" ]]; then
  sed -i '' -e 's/^version *= *".*"/version = "'"$KEEP_VERSION"'"/' "$DST/Cargo.toml"
  sed -i '' -e '/^name = "cnccraft"$/{n;s/^version = ".*"/version = "'"$KEEP_VERSION"'"/;}' "$DST/Cargo.lock"
  info "Version $KEEP_VERSION beibehalten (nicht die des Originals)."
fi

# --- Abschluss --------------------------------------------------------------
echo
info "Fertig. Neuer Stand im Ziel:"
echo
git -C "$DST" status --short
echo
info "Prüfen und committen:  ${BOLD}cd \"$DST\" && git add -A && git commit${RESET}"
