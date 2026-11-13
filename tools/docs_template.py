"""HTML-Vorlagen der Website: Seite der Hilfe, Startseite der Hilfe, Produktseite, 404."""
import html

SITE = "CncCraft"
SEARCH_SVG = ('<svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">'
              '<circle cx="8.5" cy="8.5" r="5.5"/><path d="M13 13l4 4"/></svg>')
DISCLAIMER = ("CncCraft ist ein eigenständiger, unabhängiger Nachbau, der sich in Aufbau und Bedienung am Programm "
              "Spark von Vectric orientiert. „Spark“ und „Vectric“ sind Marken ihrer Inhaber; CncCraft hat keine "
              "Verbindung zu ihnen. Alle Texte und Abbildungen dieser Seite sind eigens für CncCraft entstanden.")


def e(s):
    return html.escape(s, quote=True)


def rel(depth):
    return "../" * depth if depth else "./"


def head(title, depth, desc=""):
    r = rel(depth)
    return f"""<!doctype html>
<html lang="de">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{e(title)}</title>
<meta name="description" content="{e(desc)}">
<link rel="icon" type="image/png" href="{r}assets/favicon.png">
<link rel="stylesheet" href="{r}assets/site.css">
</head>"""


def search_box(placeholder="Hilfe durchsuchen …"):
    return (f'<div class="suche" role="search">{SEARCH_SVG}<label class="sr-only" for="q">Suche</label>'
            f'<input id="q" type="search" placeholder="{e(placeholder)}" autocomplete="off">'
            f'<div class="treffer" role="listbox"></div></div>')


def top(depth, current):
    """Sprachleiste und Kopfzeile. `current` markiert den aktiven Menüpunkt."""
    r = rel(depth)

    def nav(key, href, label):
        cur = ' aria-current="page"' if key == current else ""
        return f'<a href="{href}"{cur}>{label}</a>'

    return f"""<body data-help-root="{rel(depth - 1) if depth >= 1 else './'}{'' if depth == 0 else ''}">
<div class="sprachleiste"><div class="innen"><span>Deutsch</span><span>Version 0.1 · Onlinehilfe</span></div></div>
<header class="kopf"><div class="innen">
<button class="menue-knopf" type="button" aria-label="Themen ein- oder ausblenden">☰ Themen</button>
<a class="marke" href="{r}index.html"><img src="{r}assets/logo.svg" alt="">{SITE}</a>
<nav aria-label="Hauptmenü">
{nav("produkt", r + "index.html", "Produkt")}
{nav("hilfe", r + "hilfe/Help/page/home/index.html", "Hilfe")}
</nav>
<span class="luecke"></span>
{search_box() if depth else ""}
</div></header>"""


def foot(depth):
    r = rel(depth)
    return f"""<footer class="fuss"><div class="innen">
<div><strong>{SITE}</strong> · Entwerfen, Werkzeugwege berechnen, fräsen.</div>
<div>{e(DISCLAIMER)}</div>
<div><a href="{r}hilfe/Help/page/home/index.html">Hilfe</a> · <a href="{r}PARAMETRIK.md">Hintergrund: parametrisches Zeichnen</a></div>
</div></footer>
<script src="{r}assets/search-index.js"></script>
<script src="{r}assets/site.js"></script>
</body>
</html>"""


def sidebar(categories, order, current_slug, depth, page_url):
    parts = ['<nav class="seitenleiste" aria-label="Themen">']
    r = rel(depth)
    parts.append(f'<ul><li><a href="{r}hilfe/Help/page/home/index.html">Hilfe-Startseite</a></li></ul>')
    for c in categories:
        parts.append(f'<h2>{e(c["title"])}</h2><ul>')
        for p in order:
            if p["cat"] != c["id"]:
                continue
            cur = ' aria-current="page"' if p["slug"] == current_slug else ""
            pill = '<span class="pille">Neu</span>' if p.get("neu") else ""
            parts.append(f'<li><a href="{page_url(p["slug"], depth)}"{cur}>{e(p["title"])}{pill}</a></li>')
        parts.append("</ul>")
    parts.append("</nav>")
    return "\n".join(parts)


def page_html(p, cat, body, categories, order, prev_p, next_p, depth, page_url):
    r = rel(depth)
    neu = '<span class="pille">Neu in CncCraft</span>' if p.get("neu") else ""
    crumbs = (f'<div class="brotkrumen"><a href="{r}hilfe/Help/page/home/index.html">Hilfe</a><span>›</span>'
              f'{e(cat["title"])}<span>›</span>{e(p["title"])}</div>')
    nav = '<div class="blaettern">'
    nav += (f'<a href="{page_url(prev_p["slug"], depth)}"><small>Zurück</small>{e(prev_p["title"])}</a>' if prev_p else "<span></span>")
    nav += (f'<a class="weiter" href="{page_url(next_p["slug"], depth)}"><small>Weiter</small>{e(next_p["title"])}</a>' if next_p else "<span></span>")
    nav += "</div>"
    return (head(f'{p["title"]} – {SITE} Hilfe', depth, p["lead"]) + top(depth, "hilfe") +
            f'<div class="rahmen">{sidebar(categories, order, p["slug"], depth, page_url)}'
            f'<main><article>{crumbs}<h1>{e(p["title"])} {neu}</h1><p class="einleitung">{e(p["lead"])}</p>{body}</article>{nav}</main></div>' +
            foot(depth))


def home_html(categories, order, pages, page_url):
    depth = 4
    cards = []
    for c in categories:
        items = [p for p in order if p["cat"] == c["id"]]
        lis = "".join(f'<li><a href="{page_url(p["slug"], depth)}">{e(p["title"])}</a>'
                      f'{"<span class=pille>Neu</span>" if p.get("neu") else ""}</li>' for p in items[:6])
        more = f'<a class="mehr" href="{page_url(items[0]["slug"], depth)}">Alle {len(items)} Themen →</a>'
        cards.append(f'<div class="karte"><div class="symbol" aria-hidden="true">{c["symbol"]}</div><h3>{e(c["title"])}</h3>'
                     f'<p>{e(c["text"])}</p><ul>{lis}</ul>{more}</div>')
    neue = [p for p in order if p.get("neu")]
    neu_list = "".join(f'<li><a href="{page_url(p["slug"], depth)}">{e(p["title"])}</a> – {e(p["lead"])}</li>' for p in neue)
    return (head(f"{SITE} Hilfe", depth, "Onlinehilfe zu CncCraft: Entwurf, Werkzeugwege, Vorschau und Ausgabe.") + top(depth, "hilfe") +
            f'<section class="held"><div class="innen"><h1>Willkommen in der CncCraft-Hilfe</h1>'
            f'<p>Alles zum Entwerfen, Berechnen von Werkzeugwegen und Ausgeben für Ihre CNC-Fräse – '
            f'von der ersten Zeichnung bis zur fertigen G-Code-Datei.</p>{search_box("Wonach suchen Sie? z. B. Tasche, Variablen, Haltestege …")}</div></section>'
            f'<div class="inhalt"><div class="karten">{"".join(cards)}</div>'
            f'<h2>Neu gegenüber dem Original</h2><p>Diese Funktionen hat CncCraft zusätzlich zu dem, was die Vorlage Spark kann:</p>'
            f'<ul class="liste-neu">{neu_list}</ul></div>' + foot(depth))


def product_html(categories, order, pages, page_url):
    depth = 0
    link = lambda slug: page_url(slug, depth)  # noqa: E731

    def fig(name, alt):
        return f'<figure><img src="assets/img/{name}" alt="{e(alt)}" loading="lazy"><figcaption>{e(alt)}</figcaption></figure>'

    feats = [
        ("2D-Entwurf und Bearbeitung", "Zeichnen Sie Linien, Rechtecke, Kreise, Ellipsen, Polygone und Text, passen Sie Knoten an und formen Sie Vektoren mit Versatz, Spiegeln und Booleschen Operationen um. Ebenen halten größere Zeichnungen übersichtlich.",
         fig("ebenen.png", "Ebenen halten den Entwurf übersichtlich"), link("auswahlwerkzeug")),
        ("Parametrisch entwerfen", "Legen Sie benannte Werte an und rechnen Sie in jedem Maßfeld mit Formeln. Bemaßen Sie Abstände und legen Sie Linien waagrecht oder senkrecht fest – ändert sich ein Wert, folgt die Zeichnung.",
         fig("variablen.png", "Der Variablen-Dialog"), link("variablen")),
        ("Werkzeugwege für die Fräse", "Profil, Ausschnitt, Tasche, Bohren und V-Carving: Werkzeug wählen, Tiefen eintragen, berechnen. Haltestege halten ausgeschnittene Teile an ihrem Platz.",
         fig("tiefe.png", "Ein Werkzeugweg wird eingerichtet"), link("werkzeugwege-uebersicht")),
        ("Vorschau in 2D und 3D", "Spielen Sie die Werkzeugwege ab und sehen Sie am Werkstück, was die Fräse abtragen wird – bevor Material verloren geht. Der Ansichtswürfel und das Achsenkreuz des Nullpunkts helfen bei der Orientierung.",
         fig("ansicht-3d.png", "Die 3D-Vorschau des Werkstücks"), link("vorschau-werkzeugwege")),
        ("Hell und Dunkel", "Das Programm folgt auf Wunsch dem Design Ihres Systems oder bleibt dauerhaft hell oder dunkel.",
         fig("dunkel.png", "CncCraft im dunklen Design"), link("einstellungen-design")),
    ]
    blocks = "".join(
        f'<div class="merkmal"><div class="text"><h3>{e(t)}</h3><p>{e(d)}</p><p><a href="{u}">Mehr dazu →</a></p></div>{f}</div>'
        for t, d, f, u in feats)
    faq = [
        ("Läuft CncCraft auf meinem Computer?",
         "Ja, wenn Sie einen Mac (ab macOS 12, Apple Silicon oder Intel), einen Windows-PC oder einen Linux-Rechner haben. Wie Sie das Programm bauen und installieren, steht unter „Installation“."),
        ("Ist CncCraft mit meiner Fräse kompatibel?",
         "CncCraft schreibt Standard-G-Code in Millimetern oder Zoll sowie einen GRBL-Dialekt. Die meisten Steuerungen, die G-Code lesen, kommen damit zurecht. Prüfen Sie die erste Datei immer mit der Vorschau und einem Probeschnitt in Abfallmaterial."),
        ("In welchem Format speichert CncCraft?",
         "Projekte werden als .cnccraft-Datei gespeichert (Textformat). Für die Fräse schreibt CncCraft G-Code mit den Endungen .nc, .tap oder .gcode."),
        ("Wo finde ich Hilfe zu einer Funktion?",
         "In der Hilfe. Die Suche oben findet Seiten nach Stichwort, die Taste / springt direkt ins Suchfeld."),
    ]
    faq_html = "".join(f"<details><summary>{e(q)}</summary><p>{e(a)}</p></details>" for q, a in faq)
    lists = {
        "Zeichnung": ["auswahlwerkzeug", "linie-polylinie", "rechteck", "kreis", "ellipse", "polygon", "text-erstellen", "knotenbearbeitung"],
        "Vektoren bearbeiten und kombinieren": ["boolesche-operationen", "versatz", "spiegeln", "zentrieren", "in-kurven-umwandeln", "kopieren-einfuegen"],
        "CNC-Werkzeugwege": ["profil-werkzeugweg", "ausschnitt-werkzeugweg", "taschen-werkzeugweg", "bohr-werkzeugweg", "vcarving-werkzeugweg"],
        "Ausgabe": ["vorschau-werkzeugwege", "werkzeugwege-speichern", "werkzeugauswahl"],
    }
    def column(h, slugs):
        items = "".join(f'<li><a href="{link(s)}">{e(pages[s]["title"])}</a></li>' for s in slugs)
        return f"<div><h3>{e(h)}</h3><ul>{items}</ul></div>"

    lst = "".join(column(h, slugs) for h, slugs in lists.items())
    return (head("CncCraft – entwerfen, Werkzeugwege berechnen, fräsen", depth,
                 "CncCraft: 2D-Entwurf, parametrische Maße, Werkzeugwege für CNC-Fräsen und eine Vorschau in 3D.") + top(depth, "produkt") +
            f'<section class="produkt-held"><div class="innen"><div><h1>Das Werkzeugset für den Einstieg ins CNC-Fräsen</h1>'
            f'<p>CncCraft führt Sie vom Entwurf über die Werkzeugwege bis zur fertigen Datei für Ihre Fräse – übersichtlich, mit Vorschau in 2D und 3D und mit Maßen, die mitdenken.</p>'
            f'<div class="knoepfe"><a class="knopf primaer" href="{link("installation")}">Installation</a>'
            f'<a class="knopf" href="hilfe/Help/page/home/index.html">Zur Hilfe</a></div></div>{fig("oberflaeche.png", "CncCraft: Entwurf mit Werkzeugwegen")}</div></section>'
            f'<section class="abschnitt"><h2>Entfalten Sie Ihre Ideen</h2><p class="unter">Von der ersten Skizze bis zum Schnitt: alles in einem Programm.</p>{blocks}</section>'
            f'<section class="abschnitt"><h2>Funktionen im Überblick</h2><div class="liste-zwei" style="columns:1">'
            f'<div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:24px">{lst}</div></div></section>'
            f'<section class="abschnitt"><h2>Häufig gestellte Fragen</h2><div class="faq">{faq_html}</div></section>'
            f'<section class="abschnitt" style="padding-bottom:60px"><h2>Die Hilfe</h2><p class="unter">Jede Funktion ist beschrieben – mit Abbildungen aus dem Programm. '
            f'<a href="hilfe/Help/page/home/index.html">Zur Onlinehilfe →</a></p></section>' + foot(depth))


def not_found_html():
    depth = 0
    return (head("Seite nicht gefunden – CncCraft", depth) + top(depth, "") +
            '<main style="max-width:700px;margin:60px auto;padding:0 20px"><h1>Seite nicht gefunden</h1>'
            '<p>Diese Seite gibt es nicht. Versuchen Sie es mit der <a href="index.html">Startseite</a> oder der '
            '<a href="hilfe/Help/page/home/index.html">Hilfe</a>.</p></main>' + foot(depth))
