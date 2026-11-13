#!/usr/bin/env python3
"""Prüft die gebaute Website (app/docs): jeder Link und jedes Bild zeigt auf eine vorhandene
Datei, jede Seite hat genau eine Überschrift, keine Seite ist verwaist.

Aufruf:  python3 -I app/tools/check_docs.py
"""
import re
import sys
from pathlib import Path
from urllib.parse import unquote, urldefrag

DOCS = Path(__file__).resolve().parent.parent / "docs"
bad = []
pages = sorted(DOCS.rglob("*.html"))
linked = set()
redirects = set()
for page in pages:
    text = page.read_text(encoding="utf8")
    if 'http-equiv="refresh"' in text:
        redirects.add(page.resolve())
        continue
    if text.count("<h1") != 1:
        bad.append(f"{page.relative_to(DOCS)}: {text.count('<h1')} Überschriften erster Ebene")
    for attr, url in re.findall(r'(href|src)="([^"]+)"', text):
        if re.match(r"(https?:|mailto:|#|data:)", url):
            continue
        target = (page.parent / unquote(urldefrag(url)[0])).resolve()
        if target.is_dir():
            target = target / "index.html"
        if not target.exists():
            bad.append(f"{page.relative_to(DOCS)}: {attr}={url} fehlt")
        else:
            linked.add(target)
for page in pages:
    if page.name == "index.html" and page.parent == DOCS:
        continue
    if page.name == "404.html":
        continue
    if page.resolve() not in linked and page.resolve() not in redirects:
        bad.append(f"{page.relative_to(DOCS)}: von keiner Seite verlinkt")
print(f"{len(pages)} Seiten geprüft")
if bad:
    print("\n".join(bad))
    sys.exit(1)
print("alle Links und Bilder in Ordnung")
