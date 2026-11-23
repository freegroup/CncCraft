#!/usr/bin/env python3
"""Baut die Website (GitHub Pages) nach app/docs/.

Aufbau wie die Onlinehilfe des Originals: Startseite mit Suche und Kategorien, je Seite
Brotkrumen, Seitenleiste mit dem Themenbaum, Inhalt, Zurück/Weiter und Fußzeile; dazu eine
Produktseite als Einstieg. Alle Texte stehen in tools/docs_pages/*.py und sind eigens
geschrieben; Bilder sind Aufnahmen von CncCraft selbst (`cnccraft --screenshot`).

Aufruf:  python3 -I app/tools/build_docs.py      (aus dem Ordner app/)
"""
import html
import json
import re
import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
APP = HERE.parent
OUT = APP / "docs"
sys.path.insert(0, str(HERE))

from docs_pages import CATEGORIES, PAGES  # noqa: E402
import docs_template as T  # noqa: E402
from docs_template import page_html, home_html, product_html, not_found_html  # noqa: E402

HELP = "hilfe"          # Ordner der Onlinehilfe in docs/
PAGE_DIR = "Help/page"   # docs/hilfe/Help/page/<slug>/index.html, wie die Adressen des Originals
OLD_DIR = "seite"       # frühere Adressen: bleiben als Weiterleitung bestehen


def slug_of(title):
    """Wie das Original eine Seite benennt (sub_10059ee2c, 0x10059ee2c): alles klein, jedes Zeichen,
    das kein Buchstabe oder keine Ziffer ist, wird „-“, mehrere „-“ werden eins, am Ende keins."""
    out = ""
    for ch in title.lower():
        out += ch if ch.isascii() and ch.isalnum() else "-"
    return re.sub(r"-+", "-", out).strip("-")


def redirect_html(target):
    return ('<!doctype html><html lang="de"><head><meta charset="utf-8">'
            f'<meta http-equiv="refresh" content="0; url={target}"><link rel="canonical" href="{target}">'
            f'<title>Weiterleitung</title></head><body><a href="{target}">Weiter zur Hilfe</a></body></html>')


def by_slug():
    d = {}
    for p in PAGES:
        if p["slug"] in d:
            raise SystemExit(f"doppelter Slug: {p['slug']}")
        d[p["slug"]] = p
    return d


def rel_to_root(depth):
    return "../" * depth if depth else "./"


def page_url(slug, from_depth):
    """Relative URL einer Hilfeseite von einer Seite in der Tiefe `from_depth` unter docs/."""
    return f"{rel_to_root(from_depth)}{HELP}/{PAGE_DIR}/{slug}/index.html"


def resolve(body, pages, depth):
    """[[slug]] und [[slug|Text]] zu Links, {{img:datei|Beschreibung}} zu Abbildungen."""

    def link(m):
        slug, _, text = m.group(1).partition("|")
        if slug not in pages:
            raise SystemExit(f"unbekannter Verweis [[{slug}]]")
        return f'<a href="{page_url(slug, depth)}">{html.escape(text or pages[slug]["title"])}</a>'

    def img(m):
        name, _, alt = m.group(1).partition("|")
        if not (OUT / "assets" / "img" / name).exists():
            raise SystemExit(f"Bild fehlt: {name}")
        src = f"{rel_to_root(depth)}assets/img/{name}"
        return (f'<figure><a href="{src}"><img src="{src}" alt="{html.escape(alt)}" loading="lazy"></a>'
                f"<figcaption>{html.escape(alt)}</figcaption></figure>")

    body = re.sub(r"\[\[([^\]]+)\]\]", link, body)
    return re.sub(r"\{\{img:([^}]+)\}\}", img, body)


def plain(body):
    t = re.sub(r"\{\{img:[^}]*\}\}", " ", body)
    t = re.sub(r"\[\[(?:[^\]|]*\|)?([^\]]+)\]\]", r"\1", t)
    t = re.sub(r"<[^>]+>", " ", t)
    return re.sub(r"\s+", " ", html.unescape(t)).strip()


def field(p, key, lang):
    """A page/category field in `lang` (English falls back to German when no *_en is given)."""
    if lang == "en":
        return p.get(f"{key}_en", p[key])
    return p[key]


def build_lang(lang, root, pages, cats, order):
    """Write the whole site for one language into `root` (English at docs/, German at docs/de/)."""
    T.set_lang(lang)
    index = []
    for i, p in enumerate(order):
        depth = 4  # <root>/hilfe/Help/page/<slug>/index.html
        if slug_of(p["slug"]) != p["slug"] or p["slug"] == "home":
            raise SystemExit(f"Seitenname {p['slug']!r} folgt nicht der Namensregel des Originals")
        body = resolve(field(p, "body", lang), pages, depth)
        prev_p = order[i - 1] if i > 0 else None
        next_p = order[i + 1] if i + 1 < len(order) else None
        tail = f"{HELP}/{PAGE_DIR}/{p['slug']}/index.html"
        out = root / HELP / PAGE_DIR / p["slug"] / "index.html"
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(page_html(p, cats[p["cat"]], body, CATEGORIES, order, prev_p, next_p, depth, page_url, tail), encoding="utf-8")
        index.append({
            "t": field(p, "title", lang),
            "c": field(cats[p["cat"]], "title", lang),
            "u": f"{PAGE_DIR}/{p['slug']}/index.html",
            "x": (field(p, "lead", lang) + " " + plain(field(p, "body", lang)))[:2400],
            "n": bool(p.get("neu")),
        })

    home = root / HELP / PAGE_DIR / "home" / "index.html"
    home.parent.mkdir(parents=True, exist_ok=True)
    home.write_text(home_html(CATEGORIES, order, pages, page_url), encoding="utf-8")
    (root / HELP / "index.html").write_text(redirect_html(f"{PAGE_DIR}/home/index.html"), encoding="utf-8")
    for p in order:
        old = root / HELP / OLD_DIR / p["slug"] / "index.html"
        old.parent.mkdir(parents=True, exist_ok=True)
        old.write_text(redirect_html(f"../../{PAGE_DIR}/{p['slug']}/index.html"), encoding="utf-8")
    (root / "assets").mkdir(parents=True, exist_ok=True)
    (root / "assets" / "search-index.js").write_text(
        "window.SEARCH_INDEX=" + json.dumps(index, ensure_ascii=False, separators=(",", ":")) + ";", encoding="utf-8")
    (root / "index.html").write_text(product_html(CATEGORIES, order, pages, page_url), encoding="utf-8")
    (root / "404.html").write_text(not_found_html(), encoding="utf-8")


def main():
    pages = by_slug()
    cats = {c["id"]: c for c in CATEGORIES}
    for p in PAGES:
        if p["cat"] not in cats:
            raise SystemExit(f"Kategorie fehlt: {p['cat']}")

    # shared assets live once at docs/assets; the German tree under docs/de/ links up to them
    (OUT / "assets").mkdir(parents=True, exist_ok=True)
    for name in ("site.css", "site.js", "logo.svg"):
        shutil.copy(HERE / "docs_assets" / name, OUT / "assets" / name)
    icon = APP / "assets" / "icon.png"
    if icon.exists():
        shutil.copy(icon, OUT / "assets" / "favicon.png")
    (OUT / ".nojekyll").write_text("")

    order = [p for c in CATEGORIES for p in PAGES if p["cat"] == c["id"]]
    # English is the default at docs/; German mirrors it at docs/de/ with its own copy of the
    # assets, so every relative link resolves inside its own language root
    build_lang("en", OUT, pages, cats, order)
    de_root = OUT / "de"
    if (de_root / "assets").exists():
        shutil.rmtree(de_root / "assets")
    shutil.copytree(OUT / "assets", de_root / "assets")
    build_lang("de", de_root, pages, cats, order)
    known = {p["slug"] for p in order}
    stale = [d.name for d in (OUT / HELP / PAGE_DIR).iterdir() if d.is_dir() and d.name not in known | {"home"}]
    if stale:
        print("Hinweis: nicht mehr benutzte Seitenordner (von Hand löschen):", ", ".join(sorted(stale)))
    print(f"{len(order)} Seiten × 2 Sprachen, {len(CATEGORIES)} Kategorien -> {OUT}")


if __name__ == "__main__":
    main()
