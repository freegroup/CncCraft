#!/usr/bin/env bash
# Puts music from video_scripts/sounds under every .mp4 in video_scripts/. The picture is not changed.
#   ./add_sound.sh          only videos without audio, only if not done yet
#   ./add_sound.sh --force  every video again, replacing any existing audio with music
# Results: video_scripts/out/<name>.mp4 (originals stay untouched).
set -uo pipefail
cd "$(dirname "$0")"
shopt -s nullglob

if [[ $# -gt 1 || ( $# -eq 1 && "$1" != "--force" ) ]]; then
  echo "usage: $0 [--force]" >&2
  exit 2
fi

failed=()
for video in video_scripts/*.mp4; do
  python3 pipeline/add_sound.py "$video" "$@" || failed+=("$video")
done

if ((${#failed[@]})); then
  printf 'failed: %s\n' "${failed[@]}"
  exit 1
fi
