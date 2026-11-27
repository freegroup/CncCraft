"""HTML-Vorlagen der Website: Seite der Hilfe, Startseite der Hilfe, Produktseite, 404.

Die Seite ist zweisprachig: Englisch ist die Vorgabe und liegt in docs/, Deutsch spiegelt sie
unter docs/de/. Der EN/DE-Schalter oben wechselt zur gleichen Seite in der anderen Sprache (er
schiebt „de/“ in den Pfad hinein oder heraus). `set_lang` stellt die aktive Sprache ein; `tr`
und `pt` wählen damit den Text.
"""
import html
import re

SITE = "CncCraft"

# the active language of the page being built ("en" or "de"); set by set_lang before rendering
LANG = "en"


def set_lang(lang):
    global LANG
    LANG = lang


def tr(en, de):
    """The English or German string, by the active language."""
    return en if LANG == "en" else de


def pt(page, key):
    """A page's field in the active language: `key` for German, `key_en` for English, with the
    German text as the fallback when no English is given yet."""
    if LANG == "en":
        return page.get(f"{key}_en", page[key])
    return page[key]


SEARCH_SVG = ('<svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">'
              '<circle cx="8.5" cy="8.5" r="5.5"/><path d="M13 13l4 4"/></svg>')
DISCLAIMER = ("CncCraft ist ein eigenständiger, unabhängiger Nachbau, der sich in Aufbau und Bedienung am Programm "
              "Spark von Vectric orientiert. „Spark“ und „Vectric“ sind Marken ihrer Inhaber; CncCraft hat keine "
              "Verbindung zu ihnen. Alle Texte und Abbildungen dieser Seite sind eigens für CncCraft entstanden.")


def e(s):
    return html.escape(s, quote=True)


# Sister apps (PatternMaster, HoruCNC): a link line in every footer and a card section
# on the help home and the product page. Styles: .schwester* in docs_assets/site.css.
PM_URL = "https://freegroup.github.io/patternmaster/"
HORU_URL = "https://freegroup.github.io/HoruCNC/"

# Project links: the GitHub repository and its releases (ready-made packages), plus the public base
# URL of this site (GitHub Pages) for canonical and social-media (Open Graph / Twitter) tags.
GITHUB_URL = "https://github.com/freegroup/CncCraft"
RELEASES_URL = "https://github.com/freegroup/CncCraft/releases"
BASE_URL = "https://freegroup.github.io/CncCraft"
# A small GitHub mark for the header link.
GITHUB_SVG = ('<svg viewBox="0 0 16 16" width="18" height="18" fill="currentColor" aria-hidden="true">'
              '<path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 '
              '0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01'
              '1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 '
              '0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 '
              '1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 '
              '3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0016 8c0-4.42-3.58-8-8-8z"/></svg>')
SISTER_LINE = (f'<div>Schwester-Apps: <a href="{PM_URL}" target="_blank" rel="noopener">PatternMaster</a> – Muster fräsen · '
               f'<a href="{HORU_URL}" target="_blank" rel="noopener">HoruCNC</a> – vom Bild zum G-Code</div>')
SISTER_LINE_EN = (f'<div>Sister apps: <a href="{PM_URL}" target="_blank" rel="noopener">PatternMaster</a> – mill patterns · '
                  f'<a href="{HORU_URL}" target="_blank" rel="noopener">HoruCNC</a> – from image to G-code</div>')
_PM_ICON = ('<svg viewBox="0 0 32 32" aria-hidden="true">' +
            "".join(f'<path d="M{x} {y-5}l5 5-5 5-5-5z"/>' for x in (7, 16, 25) for y in (7, 16, 25)) + '</svg>')
_HORU_ICON = ('<svg viewBox="0 0 32 32" aria-hidden="true" fill="none" stroke-width="2.6">'
              '<rect x="3" y="9" width="26" height="18" rx="4"/><path d="M10 9l2-4h8l2 4" stroke-linejoin="round"/>'
              '<circle cx="16" cy="18" r="5"/></svg>')


def _sister(r, cls, url, icon, name, sub, text, tags, shot, alt):
    lis = "".join(f"<li>{e(t)}</li>" for t in tags)
    return (f'<a class="schwester {cls}" href="{url}" target="_blank" rel="noopener">'
            f'<div class="band"><span class="zeichen">{icon}</span><div><strong>{name}</strong><small>{e(sub)}</small></div></div>'
            f'<img class="bild" src="{r}assets/img/{shot}" alt="{e(alt)}" loading="lazy">'
            f'<div class="koerper"><p>{e(text)}</p><ul>{lis}</ul><span class="los">{tr("Try it now →", "Jetzt ausprobieren →")}</span></div></a>')


def sister_cards(depth):
    """Both sister-app cards in a grid; depth sets the relative path to the assets."""
    r = rel(depth)
    pm = _sister(r, "pm", PM_URL, _PM_ICON, "PatternMaster",
                 tr("Patterns for your CNC router", "Muster für Ihre CNC-Fräse"),
                 tr("Pick a pattern, turn the sliders and see the workpiece in 3D at once. PatternMaster works out the toolpaths for you.",
                    "Muster wählen, an den Reglern drehen und sofort in 3D sehen, wie das Werkstück aussieht. Die Fräsbahnen rechnet PatternMaster für Sie."),
                 tr(["Diamonds, waves, honeycomb", "3D preview", "No account"], ["Rauten, Wellen, Waben", "3D-Vorschau", "Kein Konto"]),
                 "schwester-patternmaster.png",
                 tr("3D preview in PatternMaster: a board with a milled diamond pattern", "3D-Vorschau in PatternMaster: Brett mit gefrästem Rautenmuster"))
    horu = _sister(r, "horu", HORU_URL, _HORU_ICON, "HoruCNC",
                   tr("From image to CNC program", "Vom Bild zum CNC-Programm"),
                   tr("Hold a drawing or photo up to the camera, pick a few filters and download the G-code for GRBL or Marlin.",
                      "Zeichnung oder Foto vor die Kamera halten, ein paar Filter wählen und den G-Code für GRBL oder Marlin herunterladen."),
                   tr(["Camera or image", "3D relief", "GRBL & Marlin"], ["Kamera oder Bild", "3D-Relief", "GRBL & Marlin"]),
                   "schwester-horucnc.png",
                   tr("3D preview in HoruCNC: a milled star in a circle", "3D-Vorschau in HoruCNC: gefräster Stern im Kreis"))
    return f'<div class="schwester-karten">{pm}{horu}</div>'


SISTER_TITLE = "Schwester-Apps für Ihre Fräse"
SISTER_LEAD = "Kostenlos, direkt im Browser und ohne Installation – aus derselben Werkstatt wie CncCraft."
SISTER_TITLE_EN = "Sister apps for your router"
SISTER_LEAD_EN = "Free, right in the browser and with no install – from the same workshop as CncCraft."
DISCLAIMER_EN = ("CncCraft is a standalone, independent re-creation whose layout and handling follow the Spark program "
                 "by Vectric. “Spark” and “Vectric” are trademarks of their owners; CncCraft has no "
                 "connection to them. Every text and image on this site was made specifically for CncCraft.")


def rel(depth):
    return "../" * depth if depth else "./"


def head(title, depth, desc="", url_tail=""):
    r = rel(depth)
    # the public address of this page, for the canonical link and the social-media cards. English
    # lives at BASE_URL/, German at BASE_URL/de/.
    base = BASE_URL.rstrip("/")
    lang_root = base if LANG == "en" else f"{base}/de"
    page_url = f"{lang_root}/{url_tail}" if url_tail else f"{lang_root}/"
    en_url = f"{base}/{url_tail}" if url_tail else f"{base}/"
    de_url = f"{base}/de/{url_tail}" if url_tail else f"{base}/de/"
    social_img = f"{base}/assets/favicon.png"
    og_locale = "en_US" if LANG == "en" else "de_DE"
    return f"""<!doctype html>
<html lang="{LANG}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{e(title)}</title>
<meta name="description" content="{e(desc)}">
<meta name="robots" content="index, follow">
<link rel="canonical" href="{e(page_url)}">
<link rel="alternate" hreflang="en" href="{e(en_url)}">
<link rel="alternate" hreflang="de" href="{e(de_url)}">
<link rel="alternate" hreflang="x-default" href="{e(en_url)}">
<meta property="og:type" content="website">
<meta property="og:site_name" content="{SITE}">
<meta property="og:locale" content="{og_locale}">
<meta property="og:title" content="{e(title)}">
<meta property="og:description" content="{e(desc)}">
<meta property="og:url" content="{e(page_url)}">
<meta property="og:image" content="{e(social_img)}">
<meta name="twitter:card" content="summary">
<meta name="twitter:title" content="{e(title)}">
<meta name="twitter:description" content="{e(desc)}">
<meta name="twitter:image" content="{e(social_img)}">
<link rel="icon" type="image/png" href="{r}assets/favicon.png">
<link rel="stylesheet" href="{r}assets/site.css">
</head>"""


def search_box(placeholder=None):
    placeholder = placeholder if placeholder is not None else tr("Search the help …", "Hilfe durchsuchen …")
    return (f'<div class="suche" role="search">{SEARCH_SVG}<label class="sr-only" for="q">{tr("Search", "Suche")}</label>'
            f'<input id="q" type="search" placeholder="{e(placeholder)}" autocomplete="off">'
            f'<div class="treffer" role="listbox"></div></div>')


def lang_toggle(depth, tail):
    """The EN/DE switch: a link to the same page in the other language. English pages live at the
    site root (docs/), German pages one level deeper (docs/de/) with the same inner structure.
    `tail` is the page's path from its own language root. From an English page, `rel(depth)` reaches
    docs/; from a German page it reaches docs/de/, so one more `../` reaches docs/."""
    r = rel(depth)
    if LANG == "en":
        en_href, de_href = f"{r}{tail}", f"{r}de/{tail}"
    else:
        en_href, de_href = f"{r}../{tail}", f"{r}{tail}"
    cur = ' aria-current="true"'
    en = f'<a href="{en_href}"{cur if LANG == "en" else ""}>EN</a>'
    de = f'<a href="{de_href}"{cur if LANG == "de" else ""}>DE</a>'
    return f'<span class="sprachwahl" aria-label="Language">{en}<span aria-hidden="true">·</span>{de}</span>'


def top(depth, current, tail=""):
    """Language bar with the EN/DE switch and the header. `current` marks the active menu item;
    `tail` is the page's path from its language root (for the switch)."""
    r = rel(depth)

    def nav(key, href, label):
        cur = ' aria-current="page"' if key == current else ""
        return f'<a href="{href}"{cur}>{label}</a>'

    sub = tr("Online help", "Onlinehilfe")
    return f"""<body data-help-root="{rel(depth - 1) if depth >= 1 else './'}">
<div class="sprachleiste"><div class="innen">{lang_toggle(depth, tail)}<span>{sub}</span></div></div>
<header class="kopf"><div class="innen">
<button class="menue-knopf" type="button" aria-label="{tr("Show or hide topics", "Themen ein- oder ausblenden")}">☰ {tr("Topics", "Themen")}</button>
<a class="marke" href="{r}index.html"><img src="{r}assets/logo.svg" alt="">{SITE}</a>
<nav aria-label="{tr("Main menu", "Hauptmenü")}">
{nav("produkt", r + "index.html", tr("Product", "Produkt"))}
{nav("hilfe", r + "hilfe/Help/page/home/index.html", tr("Help", "Hilfe"))}
<a class="github" href="{GITHUB_URL}" target="_blank" rel="noopener" aria-label="GitHub">{GITHUB_SVG}<span>GitHub</span></a>
</nav>
<span class="luecke"></span>
{search_box() if depth else ""}
</div></header>"""


def foot(depth):
    r = rel(depth)
    return f"""<footer class="fuss"><div class="innen">
<div><strong>{SITE}</strong> · {tr("Design, generate toolpaths, mill.", "Entwerfen, Werkzeugwege berechnen, fräsen.")}</div>
<div>{e(tr(DISCLAIMER_EN, DISCLAIMER))}</div>
{tr(SISTER_LINE_EN, SISTER_LINE)}
<div><a href="{GITHUB_URL}" target="_blank" rel="noopener">GitHub</a> · <a href="{RELEASES_URL}" target="_blank" rel="noopener">{tr("Downloads", "Downloads")}</a> · <a href="{r}hilfe/Help/page/home/index.html">{tr("Help", "Hilfe")}</a> · <a href="{r}hilfe/Help/page/parametrisch-uebersicht/index.html">{tr("Parametric design", "Parametrisch entwerfen")}</a></div>
</div></footer>
<script src="{r}assets/search-index.js"></script>
<script src="{r}assets/site.js"></script>
</body>
</html>"""


def ct(cat, key="title"):
    """A category's field in the active language (`key` / `key_en`, German fallback)."""
    if LANG == "en":
        return cat.get(f"{key}_en", cat[key])
    return cat[key]


def sidebar(categories, order, current_slug, depth, page_url):
    parts = ['<nav class="seitenleiste" aria-label="%s">' % e(tr("Topics", "Themen"))]
    r = rel(depth)
    parts.append(f'<ul><li><a href="{r}hilfe/Help/page/home/index.html">{tr("Help home", "Hilfe-Startseite")}</a></li></ul>')
    for c in categories:
        parts.append(f'<h2>{e(ct(c))}</h2><ul>')
        for p in order:
            if p["cat"] != c["id"]:
                continue
            cur = ' aria-current="page"' if p["slug"] == current_slug else ""
            pill = f'<span class="pille">{tr("New", "Neu")}</span>' if p.get("neu") else ""
            parts.append(f'<li><a href="{page_url(p["slug"], depth)}"{cur}>{e(pt(p, "title"))}{pill}</a></li>')
        parts.append("</ul>")
    parts.append("</nav>")
    return "\n".join(parts)


def page_html(p, cat, body, categories, order, prev_p, next_p, depth, page_url, tail=""):
    r = rel(depth)
    neu = f'<span class="pille">{tr("New in CncCraft", "Neu in CncCraft")}</span>' if p.get("neu") else ""
    crumbs = (f'<div class="brotkrumen"><a href="{r}hilfe/Help/page/home/index.html">{tr("Help", "Hilfe")}</a><span>›</span>'
              f'{e(ct(cat))}<span>›</span>{e(pt(p, "title"))}</div>')
    nav = '<div class="blaettern">'
    nav += (f'<a href="{page_url(prev_p["slug"], depth)}"><small>{tr("Back", "Zurück")}</small>{e(pt(prev_p, "title"))}</a>' if prev_p else "<span></span>")
    nav += (f'<a class="weiter" href="{page_url(next_p["slug"], depth)}"><small>{tr("Next", "Weiter")}</small>{e(pt(next_p, "title"))}</a>' if next_p else "<span></span>")
    nav += "</div>"
    return (head(f'{pt(p, "title")} – {SITE} {tr("Help", "Hilfe")}', depth, pt(p, "lead"), tail) + top(depth, "hilfe", tail) +
            f'<div class="rahmen">{sidebar(categories, order, p["slug"], depth, page_url)}'
            f'<main><article>{crumbs}<h1>{e(pt(p, "title"))} {neu}</h1><p class="einleitung">{e(pt(p, "lead"))}</p>{body}</article>{nav}</main></div>' +
            foot(depth))


def home_html(categories, order, pages, page_url):
    depth = 4
    cards = []
    for c in categories:
        items = [p for p in order if p["cat"] == c["id"]]
        lis = "".join(f'<li><a href="{page_url(p["slug"], depth)}">{e(pt(p, "title"))}</a>'
                      f'{"<span class=pille>" + tr("New", "Neu") + "</span>" if p.get("neu") else ""}</li>' for p in items[:6])
        more = f'<a class="mehr" href="{page_url(items[0]["slug"], depth)}">{tr("All %d topics →", "Alle %d Themen →") % len(items)}</a>'
        cards.append(f'<div class="karte"><div class="symbol" aria-hidden="true">{c["symbol"]}</div><h3>{e(ct(c))}</h3>'
                     f'<p>{e(ct(c, "text"))}</p><ul>{lis}</ul>{more}</div>')
    neue = [p for p in order if p.get("neu")]
    neu_list = "".join(f'<li><a href="{page_url(p["slug"], depth)}">{e(pt(p, "title"))}</a> – {e(pt(p, "lead"))}</li>' for p in neue)
    return (head(f"{SITE} {tr('Help', 'Hilfe')}", depth, tr("Online help for CncCraft: design, toolpaths, preview and output.", "Onlinehilfe zu CncCraft: Entwurf, Werkzeugwege, Vorschau und Ausgabe."), "hilfe/Help/page/home/index.html") + top(depth, "hilfe", "hilfe/Help/page/home/index.html") +
            f'<section class="held"><div class="innen"><h1>{tr("Welcome to the CncCraft help", "Willkommen in der CncCraft-Hilfe")}</h1>'
            f'<p>{tr("Everything about designing, generating toolpaths and output for your CNC router – from the first drawing to the finished G-code file.", "Alles zum Entwerfen, Berechnen von Werkzeugwegen und Ausgeben für Ihre CNC-Fräse – von der ersten Zeichnung bis zur fertigen G-Code-Datei.")}</p>{search_box(tr("What are you looking for? e.g. pocket, variables, tabs …", "Wonach suchen Sie? z. B. Tasche, Variablen, Haltestege …"))}</div></section>'
            f'<div class="inhalt"><div class="karten">{"".join(cards)}</div>'
            f'<h2>{tr("New compared with the original", "Neu gegenüber dem Original")}</h2><p>{tr("These features CncCraft adds to what the Spark template can do:", "Diese Funktionen hat CncCraft zusätzlich zu dem, was die Vorlage Spark kann:")}</p>'
            f'<ul class="liste-neu">{neu_list}</ul>'
            f'<h2>{tr(SISTER_TITLE_EN, SISTER_TITLE)}</h2><p>{e(tr(SISTER_LEAD_EN, SISTER_LEAD))}</p>{sister_cards(depth)}</div>' + foot(depth))


def site_figures(order, first):
    """Every picture the site shows, once each: `first` (file, caption, page slug) in that order,
    then the pictures of the help pages ({{img:file|caption}}) in page order."""
    seen, out = set(), []
    for name, alt, slug in first + [(m[0], m[1], p["slug"]) for p in order
                                    for m in re.findall(r"\{\{img:([^|}]+)\|([^}]*)\}\}", p["body"])]:
        if name not in seen:
            seen.add(name)
            out.append((name, alt, slug))
    return out


def carousel(slides, link):
    """A picture carousel: the first slide shows; buttons, arrow keys or a swipe move to the next or
    previous one (never on its own). Without JavaScript only the first picture is there. Hidden
    slides load their picture only once shown (lazy loading skips hidden images)."""
    n = len(slides)
    figs = "".join(
        f'<figure class="dia"{"" if i == 0 else " hidden"} aria-roledescription="{tr("image", "Bild")}" aria-label="{i + 1} {tr("of", "von")} {n}">'
        f'<img src="assets/img/{name}" alt="{e(alt)}" loading="{"eager" if i == 0 else "lazy"}">'
        f'<figcaption><span>{e(alt)}</span><a href="{link(slug)}">{tr("Learn more →", "Mehr dazu →")}</a></figcaption></figure>'
        for i, (name, alt, slug) in enumerate(slides))
    return (f'<div class="karussell" data-karussell tabindex="0" role="region" aria-roledescription="Carousel" '
            f'aria-label="{tr("Pictures from CncCraft", "Bilder aus CncCraft")}">{figs}'
            f'<button type="button" class="karussell-knopf zurueck" data-schritt="-1" aria-label="{tr("Previous image", "Vorheriges Bild")}">‹</button>'
            f'<button type="button" class="karussell-knopf weiter" data-schritt="1" aria-label="{tr("Next image", "Nächstes Bild")}">›</button>'
            f'<div class="karussell-zaehler" aria-live="polite">1 / {n}</div></div>')


def product_html(categories, order, pages, page_url):
    depth = 0
    link = lambda slug: page_url(slug, depth)  # noqa: E731

    def fig(name, alt):
        return f'<figure><img src="assets/img/{name}" alt="{e(alt)}" loading="lazy"><figcaption>{e(alt)}</figcaption></figure>'

    feats = [
        (tr("Parametric design – no CAD degree needed", "Parametrisch entwerfen – ganz ohne CAD-Studium"),
         tr("Create named values like “margin” or “width” and do the maths in any dimension field with formulas. Dimension distances and set lines level or upright – change a value and the drawing follows, and the toolpaths recompute on their own.",
            "Legen Sie benannte Werte wie „rand“ oder „breite“ an und rechnen Sie in jedem Maßfeld mit Formeln. Bemaßen Sie Abstände und legen Sie Linien waagrecht oder senkrecht fest – ändert sich ein Wert, folgt die Zeichnung, und die Werkzeugwege rechnen von selbst neu."),
         ("variablen.png", tr("Variables and bound dimensions", "Variablen und gebundene Maße")), "parametrisch-uebersicht"),
        (tr("Construction lines from the ruler", "Hilfslinien aus dem Lineal"),
         tr("A click in the ruler sets a construction line. Shapes snap to it with an edge or their centre and stay attached: drag the line further – or give it a formula like “platte_b/2” – and the shapes move along. A lock protects all lines from being moved by accident.",
            "Ein Klick ins Lineal setzt eine Hilfslinie. Formen rasten mit einer Kante oder ihrer Mitte daran ein und bleiben hängen: Ziehen Sie die Linie weiter – oder geben Sie ihr eine Formel wie „platte_b/2“ –, wandern die Formen mit. Ein Schloss schützt alle Linien vor versehentlichem Verschieben."),
         ("hilfslinien.png", tr("Construction lines on the ruler, one of them with a formula", "Hilfslinien am Lineal, eine davon mit Formel")), "hilfslinien"),
        (tr("Values that match the material", "Werte, die zum Material passen"),
         tr("Ten materials from softwood to aluminium, each with recommendations for speed, feed, stepdown, cutter and cooling. If a value is too fast or too deep, CncCraft says so right under the field. In 3D the workpiece looks like the material.",
            "Zehn Materialien vom Weichholz bis zum Aluminium, jedes mit Empfehlungen für Drehzahl, Vorschub, Zustellung, Fräser und Kühlung. Ist ein Wert zu schnell oder zu tief, sagt CncCraft es direkt unter dem Feld. In 3D sieht das Werkstück aus wie das Material."),
         ("material-aluminium.png", tr("An aluminium workpiece in the 3D view", "Ein Werkstück aus Aluminium in der 3D-Ansicht")), "materialien"),
        (tr("2D design and editing", "2D-Entwurf und Bearbeitung"),
         tr("Draw lines, splines, rectangles, circles, ellipses, polygons and text, import DXF files, round corners and reshape vectors with offset, mirror, trim and boolean operations. Layers keep larger drawings tidy.",
            "Zeichnen Sie Linien, Splines, Rechtecke, Kreise, Ellipsen, Polygone und Text, importieren Sie DXF-Dateien, runden Sie Ecken ab und formen Sie Vektoren mit Versatz, Spiegeln, Trennen und Booleschen Operationen um. Ebenen halten größere Zeichnungen übersichtlich."),
         ("ebenen.png", tr("Layers keep the design tidy", "Ebenen halten den Entwurf übersichtlich")), "auswahlwerkzeug"),
        (tr("Toolpaths for the router", "Werkzeugwege für die Fräse"),
         tr("Profile, cut-out, pocket, slot, drilling and V-carving: pick a tool, enter the depths, compute. Tabs hold cut-out parts in place; small drawings in the tooltips explain each strategy.",
            "Profil, Ausschnitt, Tasche, Nut, Bohren und V-Carving: Werkzeug wählen, Tiefen eintragen, berechnen. Haltestege halten ausgeschnittene Teile an ihrem Platz; kleine Zeichnungen in den Hilfetexten erklären jede Strategie."),
         ("nut-werkzeugweg.png", tr("A slot toolpath being set up", "Ein Nut-Werkzeugweg wird eingerichtet")), "werkzeugwege-uebersicht"),
        (tr("Preview in 2D and 3D", "Vorschau in 2D und 3D"),
         tr("Play back the toolpaths and see on the workpiece what the router will remove – before any material is wasted. The view cube and the origin axis cross help you keep your bearings.",
            "Spielen Sie die Werkzeugwege ab und sehen Sie am Werkstück, was die Fräse abtragen wird – bevor Material verloren geht. Der Ansichtswürfel und das Achsenkreuz des Nullpunkts helfen bei der Orientierung."),
         ("ansicht-3d.png", tr("The 3D preview of the workpiece", "Die 3D-Vorschau des Werkstücks")), "vorschau-werkzeugwege"),
        (tr("Light and dark", "Hell und Dunkel"),
         tr("The program can follow your system's appearance or stay light or dark for good.",
            "Das Programm folgt auf Wunsch dem Design Ihres Systems oder bleibt dauerhaft hell oder dunkel."),
         ("dunkel.png", tr("CncCraft in the dark theme", "CncCraft im dunklen Design")), "einstellungen-design"),
    ]
    blocks = "".join(
        f'<div class="merkmal"><div class="text"><h3>{e(t)}</h3><p>{e(d)}</p><p><a href="{link(slug)}">{tr("Learn more →", "Mehr dazu →")}</a></p></div>{fig(*im)}</div>'
        for t, d, im, slug in feats)
    hero = ("oberflaeche.png", tr("CncCraft: design with toolpaths", "CncCraft: Entwurf mit Werkzeugwegen"), "einfuehrung")
    slides = site_figures(order, [hero] + [(im[0], im[1], slug) for _, _, im, slug in feats])
    faq = [
        (tr("What does CncCraft cost?", "Was kostet CncCraft?"),
         tr("Nothing. CncCraft is free – no licence key, no subscription and no account. Every feature is unlocked from the start.",
            "Nichts. CncCraft ist kostenlos – ohne Lizenzschlüssel, ohne Abo und ohne Konto. Alle Funktionen sind von Anfang an freigeschaltet.")),
        (tr("Do I need CAD experience?", "Brauche ich Vorkenntnisse in CAD?"),
         tr("No. CncCraft is made for CNC beginners: you draw like in a simple drawing program and make dimensions parametric only when you need to. Every step can be undone, and every tool explains itself with a tooltip.",
            "Nein. CncCraft ist für CNC-Einsteiger gemacht: Sie zeichnen wie in einem einfachen Zeichenprogramm und machen Maße erst dann parametrisch, wenn Sie es brauchen. Jeder Schritt lässt sich rückgängig machen, und jedes Werkzeug erklärt sich mit einem Hilfetext.")),
        (tr("What does “parametric” mean?", "Was heißt „parametrisch“?"),
         tr("A dimension is a value you can change later – and everything tied to it follows: shapes, distances, construction lines and toolpaths. So a design fits another board size with a single number.",
            "Ein Maß ist ein Wert, den Sie später ändern können – und alles, was daran hängt, folgt: Formen, Abstände, Hilfslinien und Werkzeugwege. So passt ein Entwurf mit einer Zahl auf eine andere Plattengröße.")),
        (tr("Does CncCraft run on my computer?", "Läuft CncCraft auf meinem Computer?"),
         tr("Yes, if you have a Mac (macOS 12 or newer, Apple Silicon or Intel), a Windows PC or a Linux machine. How to build and install the program is under “Installation”.",
            "Ja, wenn Sie einen Mac (ab macOS 12, Apple Silicon oder Intel), einen Windows-PC oder einen Linux-Rechner haben. Wie Sie das Programm bauen und installieren, steht unter „Installation“.")),
        (tr("Is CncCraft compatible with my router?", "Ist CncCraft mit meiner Fräse kompatibel?"),
         tr("CncCraft writes standard G-code in millimetres or inches and a GRBL dialect. Most controllers that read G-code cope with it. Always check the first file with the preview and a test cut in scrap material.",
            "CncCraft schreibt Standard-G-Code in Millimetern oder Zoll sowie einen GRBL-Dialekt. Die meisten Steuerungen, die G-Code lesen, kommen damit zurecht. Prüfen Sie die erste Datei immer mit der Vorschau und einem Probeschnitt in Abfallmaterial.")),
        (tr("What format does CncCraft save in?", "In welchem Format speichert CncCraft?"),
         tr("Projects are saved as a .cnccraft file (text format). For the router CncCraft writes G-code with the extensions .nc, .tap or .gcode.",
            "Projekte werden als .cnccraft-Datei gespeichert (Textformat). Für die Fräse schreibt CncCraft G-Code mit den Endungen .nc, .tap oder .gcode.")),
        (tr("Where do I find help on a feature?", "Wo finde ich Hilfe zu einer Funktion?"),
         tr("In the help. The search at the top finds pages by keyword, and the / key jumps straight into the search box.",
            "In der Hilfe. Die Suche oben findet Seiten nach Stichwort, die Taste / springt direkt ins Suchfeld.")),
    ]
    faq_html = "".join(f"<details><summary>{e(q)}</summary><p>{e(a)}</p></details>" for q, a in faq)
    lists = {
        tr("Parametric", "Parametrisch"): ["parametrisch-uebersicht", "variablen", "formeln-in-feldern", "abstand-bemassen", "waagrecht-senkrecht", "hilfslinien"],
        tr("Drawing", "Zeichnung"): ["auswahlwerkzeug", "linie-polylinie", "spline", "rechteck", "kreis", "ellipse", "polygon", "text-erstellen", "knotenbearbeitung", "dxf-importieren", "bild-als-vorlage"],
        tr("Edit and combine vectors", "Vektoren bearbeiten und kombinieren"): ["boolesche-operationen", "versatz", "spiegeln", "zentrieren", "in-kurven-umwandeln", "vervielfaeltigen", "vektoren-trennen-verbinden", "ecken-abrunden", "gruppieren", "kopieren-einfuegen"],
        tr("CNC toolpaths", "CNC-Werkzeugwege"): ["profil-werkzeugweg", "ausschnitt-werkzeugweg", "taschen-werkzeugweg", "nut-werkzeugweg", "bohr-werkzeugweg", "vcarving-werkzeugweg"],
        tr("Output", "Ausgabe"): ["materialien", "werkzeugauswahl", "vorschau-werkzeugwege", "werkzeugwege-speichern"],
    }
    def column(h, slugs):
        items = "".join(f'<li><a href="{link(s)}">{e(pt(pages[s], "title"))}</a></li>' for s in slugs)
        return f"<div><h3>{e(h)}</h3><ul>{items}</ul></div>"

    lst = "".join(column(h, slugs) for h, slugs in lists.items())
    vorteile = [
        (tr("Parametric", "Parametrisch"), tr("Dimensions as values, variables and formulas. Change one number and the whole design follows – all the way to the G-code.", "Maße als Werte, Variablen und Formeln. Ändern Sie eine Zahl, folgt der ganze Entwurf – bis zum G-Code.")),
        (tr("For CNC beginners", "Für CNC-Einsteiger"), tr("Few, clear tools with tooltips, material suggestions, warnings against cuts that are too hard, and undo for every step.", "Wenige, klare Werkzeuge mit Hilfetexten, Vorschläge zum Material, Warnungen vor zu harten Schnitten und Rückgängig für jeden Schritt.")),
        (tr("Free", "Kostenlos"), tr("No licence, no subscription, no account. Every feature is there from the start – for Mac, Windows and Linux.", "Keine Lizenz, kein Abo, kein Konto. Alle Funktionen sind von Anfang an da – für Mac, Windows und Linux.")),
    ]
    vorteile_html = "".join(f'<div class="vorteil"><h3>{e(t)}</h3><p>{e(d)}</p></div>' for t, d in vorteile)
    return (head(tr("CncCraft – parametric CAD/CAM for CNC beginners, free", "CncCraft – parametrisches CAD/CAM für CNC-Einsteiger, kostenlos"), depth,
                 tr("CncCraft: free parametric CAD/CAM for CNC beginners – 2D design with variables and construction lines, toolpaths for CNC routers, a material library and a 3D preview.", "CncCraft: kostenloses parametrisches CAD/CAM für CNC-Einsteiger – 2D-Entwurf mit Variablen und Hilfslinien, Werkzeugwege für CNC-Fräsen, Materialbibliothek und Vorschau in 3D."), "") + top(depth, "produkt", "index.html") +
            f'<section class="produkt-held"><div class="innen"><div><span class="held-marke">{tr("CAD + CAM · Free · for Mac, Windows and Linux", "CAD + CAM · Kostenlos · für Mac, Windows und Linux")}</span><h1>{tr("Parametric CAD/CAM for CNC beginners", "Parametrisches CAD/CAM für CNC-Einsteiger")}</h1>'
            f'<p>{tr("Design (CAD) and generate toolpaths (CAM) in one program: CncCraft takes you from the first drawing to the finished G-code for your CNC router – with dimensions that think along, construction lines from the ruler, cutting values matched to the material and a preview in 2D and 3D. Free, no subscription and no account.", "Entwerfen (CAD) und Werkzeugwege berechnen (CAM) in einem Programm: CncCraft führt Sie von der ersten Zeichnung bis zum fertigen G-Code für Ihre CNC-Fräse – mit Maßen, die mitdenken, Hilfslinien aus dem Lineal, Schnittwerten passend zum Material und Vorschau in 2D und 3D. Kostenlos, ohne Abo und ohne Konto.")}</p>'
            f'<div class="knoepfe"><a class="knopf primaer" href="{link("installation")}">{tr("Installation", "Installation")}</a>'
            f'<a class="knopf" href="hilfe/Help/page/home/index.html">{tr("To the help", "Zur Hilfe")}</a></div></div>{carousel(slides, link)}</div></section>'
            f'<section class="abschnitt"><div class="vorteile">{vorteile_html}</div></section>'
            f'<section class="abschnitt"><h2>{tr("Unfold your ideas", "Entfalten Sie Ihre Ideen")}</h2><p class="unter">{tr("From the first sketch to the cut: all in one program.", "Von der ersten Skizze bis zum Schnitt: alles in einem Programm.")}</p>{blocks}</section>'
            f'<section class="abschnitt"><h2>{tr("Features at a glance", "Funktionen im Überblick")}</h2><div class="liste-zwei" style="columns:1">'
            f'<div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:24px">{lst}</div></div></section>'
            f'<section class="abschnitt"><h2>{tr(SISTER_TITLE_EN, SISTER_TITLE)}</h2><p class="unter">{e(tr(SISTER_LEAD_EN, SISTER_LEAD))}</p>{sister_cards(depth)}</section>'
            f'<section class="abschnitt"><h2>{tr("Frequently asked questions", "Häufig gestellte Fragen")}</h2><div class="faq">{faq_html}</div></section>'
            f'<section class="abschnitt" style="padding-bottom:60px"><h2>{tr("The help", "Die Hilfe")}</h2><p class="unter">{tr("Every feature is described – with pictures from the program. ", "Jede Funktion ist beschrieben – mit Abbildungen aus dem Programm. ")}'
            f'<a href="hilfe/Help/page/home/index.html">{tr("To the online help →", "Zur Onlinehilfe →")}</a></p></section>' + foot(depth))


def not_found_html():
    depth = 0
    return (head(tr("Page not found – CncCraft", "Seite nicht gefunden – CncCraft"), depth) + top(depth, "", "404.html") +
            f'<main style="max-width:700px;margin:60px auto;padding:0 20px"><h1>{tr("Page not found", "Seite nicht gefunden")}</h1>'
            f'<p>{tr("This page does not exist. Try the ", "Diese Seite gibt es nicht. Versuchen Sie es mit der ")}<a href="index.html">{tr("home page", "Startseite")}</a>'
            f'{tr(" or the ", " oder der ")}<a href="hilfe/Help/page/home/index.html">{tr("help", "Hilfe")}</a>.</p></main>' + foot(depth))
