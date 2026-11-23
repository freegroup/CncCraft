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


# Sister apps (PatternMaster, HoruCNC): a link line in every footer and a card section
# on the help home and the product page. Styles: .schwester* in docs_assets/site.css.
PM_URL = "https://freegroup.github.io/patternmaster/"
HORU_URL = "https://freegroup.github.io/HoruCNC/"
SISTER_LINE = (f'<div>Schwester-Apps: <a href="{PM_URL}" target="_blank" rel="noopener">PatternMaster</a> – Muster fräsen · '
               f'<a href="{HORU_URL}" target="_blank" rel="noopener">HoruCNC</a> – vom Bild zum G-Code</div>')
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
            f'<div class="koerper"><p>{e(text)}</p><ul>{lis}</ul><span class="los">Jetzt ausprobieren →</span></div></a>')


def sister_cards(depth):
    """Both sister-app cards in a grid; depth sets the relative path to the assets."""
    r = rel(depth)
    return ('<div class="schwester-karten">' +
                _sister(r, "pm", PM_URL, _PM_ICON, "PatternMaster", "Muster für Ihre CNC-Fräse",
                        "Muster wählen, an den Reglern drehen und sofort in 3D sehen, wie das Werkstück aussieht. "
                        "Die Fräsbahnen rechnet PatternMaster für Sie.",
                        ["Rauten, Wellen, Waben", "3D-Vorschau", "Kein Konto"],
                        "schwester-patternmaster.png", "3D-Vorschau in PatternMaster: Brett mit gefrästem Rautenmuster") +
                _sister(r, "horu", HORU_URL, _HORU_ICON, "HoruCNC", "Vom Bild zum CNC-Programm",
                        "Zeichnung oder Foto vor die Kamera halten, ein paar Filter wählen und den G-Code "
                        "für GRBL oder Marlin herunterladen.",
                        ["Kamera oder Bild", "3D-Relief", "GRBL & Marlin"],
                        "schwester-horucnc.png", "3D-Vorschau in HoruCNC: gefräster Stern im Kreis") +
                '</div>')
SISTER_TITLE = "Schwester-Apps für Ihre Fräse"
SISTER_LEAD = "Kostenlos, direkt im Browser und ohne Installation – aus derselben Werkstatt wie CncCraft."


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
{SISTER_LINE}
<div><a href="{r}hilfe/Help/page/home/index.html">Hilfe</a> · <a href="{r}hilfe/Help/page/parametrisch-uebersicht/index.html">Parametrisch entwerfen</a></div>
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
            f'<ul class="liste-neu">{neu_list}</ul>'
            f'<h2>{SISTER_TITLE}</h2><p>{e(SISTER_LEAD)}</p>{sister_cards(depth)}</div>' + foot(depth))


def product_html(categories, order, pages, page_url):
    depth = 0
    link = lambda slug: page_url(slug, depth)  # noqa: E731

    def fig(name, alt):
        return f'<figure><img src="assets/img/{name}" alt="{e(alt)}" loading="lazy"><figcaption>{e(alt)}</figcaption></figure>'

    feats = [
        ("Parametrisch entwerfen – ganz ohne CAD-Studium", "Legen Sie benannte Werte wie „rand“ oder „breite“ an und rechnen Sie in jedem Maßfeld mit Formeln. Bemaßen Sie Abstände und legen Sie Linien waagrecht oder senkrecht fest – ändert sich ein Wert, folgt die Zeichnung, und die Werkzeugwege rechnen von selbst neu.",
         fig("variablen.png", "Variablen und gebundene Maße"), link("parametrisch-uebersicht")),
        ("Hilfslinien aus dem Lineal", "Ein Klick ins Lineal setzt eine Hilfslinie. Formen rasten mit einer Kante oder ihrer Mitte daran ein und bleiben hängen: Ziehen Sie die Linie weiter – oder geben Sie ihr eine Formel wie „platte_b/2“ –, wandern die Formen mit. Ein Schloss schützt alle Linien vor versehentlichem Verschieben.",
         fig("hilfslinien.png", "Hilfslinien am Lineal, eine davon mit Formel"), link("hilfslinien")),
        ("Werte, die zum Material passen", "Zehn Materialien vom Weichholz bis zum Aluminium, jedes mit Empfehlungen für Drehzahl, Vorschub, Zustellung, Fräser und Kühlung. Ist ein Wert zu schnell oder zu tief, sagt CncCraft es direkt unter dem Feld. In 3D sieht das Werkstück aus wie das Material.",
         fig("material-aluminium.png", "Ein Werkstück aus Aluminium in der 3D-Ansicht"), link("materialien")),
        ("2D-Entwurf und Bearbeitung", "Zeichnen Sie Linien, Splines, Rechtecke, Kreise, Ellipsen, Polygone und Text, importieren Sie DXF-Dateien, runden Sie Ecken ab und formen Sie Vektoren mit Versatz, Spiegeln, Trennen und Booleschen Operationen um. Ebenen halten größere Zeichnungen übersichtlich.",
         fig("ebenen.png", "Ebenen halten den Entwurf übersichtlich"), link("auswahlwerkzeug")),
        ("Werkzeugwege für die Fräse", "Profil, Ausschnitt, Tasche, Nut, Bohren und V-Carving: Werkzeug wählen, Tiefen eintragen, berechnen. Haltestege halten ausgeschnittene Teile an ihrem Platz; kleine Zeichnungen in den Hilfetexten erklären jede Strategie.",
         fig("nut-werkzeugweg.png", "Ein Nut-Werkzeugweg wird eingerichtet"), link("werkzeugwege-uebersicht")),
        ("Vorschau in 2D und 3D", "Spielen Sie die Werkzeugwege ab und sehen Sie am Werkstück, was die Fräse abtragen wird – bevor Material verloren geht. Der Ansichtswürfel und das Achsenkreuz des Nullpunkts helfen bei der Orientierung.",
         fig("ansicht-3d.png", "Die 3D-Vorschau des Werkstücks"), link("vorschau-werkzeugwege")),
        ("Hell und Dunkel", "Das Programm folgt auf Wunsch dem Design Ihres Systems oder bleibt dauerhaft hell oder dunkel.",
         fig("dunkel.png", "CncCraft im dunklen Design"), link("einstellungen-design")),
    ]
    blocks = "".join(
        f'<div class="merkmal"><div class="text"><h3>{e(t)}</h3><p>{e(d)}</p><p><a href="{u}">Mehr dazu →</a></p></div>{f}</div>'
        for t, d, f, u in feats)
    faq = [
        ("Was kostet CncCraft?",
         "Nichts. CncCraft ist kostenlos – ohne Lizenzschlüssel, ohne Abo und ohne Konto. Alle Funktionen sind von Anfang an freigeschaltet."),
        ("Brauche ich Vorkenntnisse in CAD?",
         "Nein. CncCraft ist für CNC-Einsteiger gemacht: Sie zeichnen wie in einem einfachen Zeichenprogramm und machen Maße erst dann parametrisch, wenn Sie es brauchen. Jeder Schritt lässt sich rückgängig machen, und jedes Werkzeug erklärt sich mit einem Hilfetext."),
        ("Was heißt „parametrisch“?",
         "Ein Maß ist ein Wert, den Sie später ändern können – und alles, was daran hängt, folgt: Formen, Abstände, Hilfslinien und Werkzeugwege. So passt ein Entwurf mit einer Zahl auf eine andere Plattengröße."),
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
        "Parametrisch": ["parametrisch-uebersicht", "variablen", "formeln-in-feldern", "abstand-bemassen", "waagrecht-senkrecht", "hilfslinien"],
        "Zeichnung": ["auswahlwerkzeug", "linie-polylinie", "spline", "rechteck", "kreis", "ellipse", "polygon", "text-erstellen", "knotenbearbeitung", "dxf-importieren", "bild-als-vorlage"],
        "Vektoren bearbeiten und kombinieren": ["boolesche-operationen", "versatz", "spiegeln", "zentrieren", "in-kurven-umwandeln", "vektoren-trennen-verbinden", "ecken-abrunden", "gruppieren", "kopieren-einfuegen"],
        "CNC-Werkzeugwege": ["profil-werkzeugweg", "ausschnitt-werkzeugweg", "taschen-werkzeugweg", "nut-werkzeugweg", "bohr-werkzeugweg", "vcarving-werkzeugweg"],
        "Ausgabe": ["materialien", "werkzeugauswahl", "vorschau-werkzeugwege", "werkzeugwege-speichern"],
    }
    def column(h, slugs):
        items = "".join(f'<li><a href="{link(s)}">{e(pages[s]["title"])}</a></li>' for s in slugs)
        return f"<div><h3>{e(h)}</h3><ul>{items}</ul></div>"

    lst = "".join(column(h, slugs) for h, slugs in lists.items())
    vorteile = [
        ("Parametrisch", "Maße als Werte, Variablen und Formeln. Ändern Sie eine Zahl, folgt der ganze Entwurf – bis zum G-Code."),
        ("Für CNC-Einsteiger", "Wenige, klare Werkzeuge mit Hilfetexten, Vorschläge zum Material, Warnungen vor zu harten Schnitten und Rückgängig für jeden Schritt."),
        ("Kostenlos", "Keine Lizenz, kein Abo, kein Konto. Alle Funktionen sind von Anfang an da – für Mac, Windows und Linux."),
    ]
    vorteile_html = "".join(f'<div class="vorteil"><h3>{e(t)}</h3><p>{e(d)}</p></div>' for t, d in vorteile)
    return (head("CncCraft – parametrisches CAD/CAM für CNC-Einsteiger, kostenlos", depth,
                 "CncCraft: kostenloses parametrisches CAD/CAM für CNC-Einsteiger – 2D-Entwurf mit Variablen und Hilfslinien, Werkzeugwege für CNC-Fräsen, Materialbibliothek und Vorschau in 3D.") + top(depth, "produkt") +
            f'<section class="produkt-held"><div class="innen"><div><span class="held-marke">CAD + CAM · Kostenlos · für Mac, Windows und Linux</span><h1>Parametrisches CAD/CAM für CNC-Einsteiger</h1>'
            f'<p>Entwerfen (CAD) und Werkzeugwege berechnen (CAM) in einem Programm: CncCraft führt Sie von der ersten Zeichnung bis zum fertigen G-Code für Ihre CNC-Fräse – mit Maßen, die mitdenken, Hilfslinien aus dem Lineal, Schnittwerten passend zum Material und Vorschau in 2D und 3D. Kostenlos, ohne Abo und ohne Konto.</p>'
            f'<div class="knoepfe"><a class="knopf primaer" href="{link("installation")}">Installation</a>'
            f'<a class="knopf" href="hilfe/Help/page/home/index.html">Zur Hilfe</a></div></div>{fig("oberflaeche.png", "CncCraft: Entwurf mit Werkzeugwegen")}</div></section>'
            f'<section class="abschnitt"><div class="vorteile">{vorteile_html}</div></section>'
            f'<section class="abschnitt"><h2>Entfalten Sie Ihre Ideen</h2><p class="unter">Von der ersten Skizze bis zum Schnitt: alles in einem Programm.</p>{blocks}</section>'
            f'<section class="abschnitt"><h2>Funktionen im Überblick</h2><div class="liste-zwei" style="columns:1">'
            f'<div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:24px">{lst}</div></div></section>'
            f'<section class="abschnitt"><h2>{SISTER_TITLE}</h2><p class="unter">{e(SISTER_LEAD)}</p>{sister_cards(depth)}</section>'
            f'<section class="abschnitt"><h2>Häufig gestellte Fragen</h2><div class="faq">{faq_html}</div></section>'
            f'<section class="abschnitt" style="padding-bottom:60px"><h2>Die Hilfe</h2><p class="unter">Jede Funktion ist beschrieben – mit Abbildungen aus dem Programm. '
            f'<a href="hilfe/Help/page/home/index.html">Zur Onlinehilfe →</a></p></section>' + foot(depth))


def not_found_html():
    depth = 0
    return (head("Seite nicht gefunden – CncCraft", depth) + top(depth, "") +
            '<main style="max-width:700px;margin:60px auto;padding:0 20px"><h1>Seite nicht gefunden</h1>'
            '<p>Diese Seite gibt es nicht. Versuchen Sie es mit der <a href="index.html">Startseite</a> oder der '
            '<a href="hilfe/Help/page/home/index.html">Hilfe</a>.</p></main>' + foot(depth))
