#!/usr/bin/env python3
"""Put music under a CncCraft recording, picture untouched: add_sound.py video_scripts/pockets.mp4"""
import argparse
import hashlib
import re
import subprocess
import sys
from pathlib import Path

META_RE = re.compile(r"^\s*#\s*@([A-Za-z_][\w-]*)\s*:\s*(.*?)\s*$")
MUSIC_EXT = {".mp3", ".m4a", ".wav", ".aac", ".flac", ".ogg"}


def fail(msg):
    sys.exit(f"error: {msg}")


def read_meta(path):
    if not path.is_file():
        return {}
    return dict(m.groups() for m in map(META_RE.match, path.read_text(encoding="utf-8").splitlines()) if m)


def probe(path, *args):
    out = subprocess.run(["ffprobe", "-v", "error", *args, "-of", "csv=p=0", str(path)],
                         capture_output=True, text=True, check=True)
    return out.stdout.strip()


def pick_music(music_dir, choice, name):
    if choice and choice.lower() in ("off", "none"):
        return None
    pool_dir = music_dir
    if choice:
        direct = music_dir / choice
        if direct.is_file():
            return direct
        if not direct.is_dir():
            fail(f"@music '{choice}' is neither a file nor a folder in {music_dir}")
        pool_dir = direct
    pool = sorted(p for p in pool_dir.rglob("*") if p.suffix.lower() in MUSIC_EXT)
    if not pool:
        fail(f"no music in {pool_dir} (add tracks or set '# @music: off')")
    # stable per video, so a re-run picks the same track
    return pool[int(hashlib.sha1(name.encode()).hexdigest(), 16) % len(pool)]


def add_music(video, music, out):
    duration = float(probe(video, "-show_entries", "format=duration"))
    af = (
        f"[1:a]atrim=0:{duration:.3f},asetpts=PTS-STARTPTS,"
        f"afade=t=in:d=1,afade=t=out:st={max(duration - 2, 0):.3f}:d=2,"
        f"loudnorm=I=-14:TP=-1.5:LRA=11,aresample=48000[aout]"
    )
    subprocess.run([
        "ffmpeg", "-y", "-v", "error", "-i", str(video), "-stream_loop", "-1", "-i", str(music),
        "-filter_complex", af, "-map", "0:v", "-map", "[aout]",
        "-c:v", "copy", "-c:a", "aac", "-b:a", "192k", "-t", f"{duration:.3f}",
        "-movflags", "+faststart", str(out),
    ], check=True)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("video", type=Path, help="the recording; a .txt with the same name may set '# @music: ...'")
    p.add_argument("--music-dir", type=Path, help="music library (default: sounds/ next to the video)")
    p.add_argument("--out", type=Path, help="output folder (default: out/ next to the video)")
    p.add_argument("--force", action="store_true",
                   help="redo even if done, and replace the video's own audio with music")
    a = p.parse_args()

    video = a.video.resolve()
    if not video.is_file():
        fail(f"{video} not found")
    outdir = a.out or video.parent / "out"
    outdir.mkdir(parents=True, exist_ok=True)
    out = outdir / video.name

    if not a.force:
        if probe(video, "-select_streams", "a", "-show_entries", "stream=index"):
            print(f"skip: {video.name} already has audio")
            return
        if out.exists() and out.stat().st_mtime > video.stat().st_mtime:
            print(f"skip: {out} is up to date")
            return

    music = pick_music(a.music_dir or video.parent / "sounds",
                       read_meta(video.with_suffix(".txt")).get("music"), video.stem)
    if music is None:
        print(f"skip: {video.name} has '# @music: off'")
        return
    print(f"{video.name} + {music.name}")
    add_music(video, music, out)
    print(f"-> {out}")


if __name__ == "__main__":
    main()
