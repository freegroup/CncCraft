"""Kategorie „Automatisierung“ – Skripte und der eingebaute Server (Pilotfunktion)."""

C = "automatisierung"

PAGES = [
    {
        "slug": "skript-automatisierung", "cat": C, "title": "Skripte und Automatisierung",
        "title_en": "Scripts and automation", "neu": True,
        "lead": "Mit einem kleinen Skript steuern Sie CncCraft Schritt für Schritt – die sichtbare Maus führt jeden Klick vor. Ideal für kurze Videos für TikTok, Instagram & Co.",
        "lead_en": "A small script drives CncCraft step by step – the visible mouse shows every click. Great for short videos for TikTok, Instagram and the like.",
        "body_en": """
<div class="pilot-box"><p><strong>This is a pilot feature (beta).</strong> Scripting and the built-in
automation server are new, still being refined and may change. They are meant mainly for recording
demos; everyday work needs none of it.</p></div>
<p>CncCraft can run a <strong>script</strong>: a plain list of commands, one per line, that it carries
out by moving the <strong>visible mouse</strong> and clicking the real buttons – so it looks exactly
like someone using the program.</p>
<div class="tipp"><p>This is a nice way to build <strong>short demo videos for TikTok, Instagram or
YouTube Shorts</strong>: the big visible cursor, the on-screen captions and the steady, repeatable
moves make clean, hands-free clips. Record the window, set a title and a subtitle, pick the speed,
and let the script draw and mill while the mouse shows every step.</p></div>
<h2>The script window</h2>
<p>Open it with <em>Help ▸ Script…</em>. It is its own window you can move anywhere. It has an
editor with autocomplete and syntax highlighting, a <strong>Run</strong> and a <strong>Stop</strong>
button, a <strong>Command help</strong> window listing every command, and an <strong>agent.md</strong>
button that copies a full guide for an AI assistant.</p>
<h2>How a script reads</h2>
<ul>
<li>One command per line, for example <code>draw_rectangle cx=50 cy=50 width=40 height=30</code>.</li>
<li>Write <code>name = tool …</code> to keep the result under a name, then use it later as
<code>$name.id</code> – so you can draw a shape and give it a toolpath.</li>
<li>Lines like <code>move_to</code>, <code>click</code>, <code>drag</code> and <code>type_text</code>
play through the visible cursor; <code>screenshot</code>, <code>recording_start</code> /
<code>recording_stop</code> and <code>window width=… height=…</code> help with recording.</li>
<li><code>text_title</code> and <code>text_subtitle</code> put a caption on screen,
<code>replay_speed</code> sets how fast the mouse moves.</li>
</ul>
<h2>Snapping from a script</h2>
<p>Every button has a command, so a script can switch the three snap methods
([[einrasten|Snapping]]) on and off like any other: <code>snap_to_geometry</code> (bounding box),
<code>snap_to_center</code> and <code>snap_to_gap</code>. That way a demo can show, step by step,
how a shape lines up on an edge, a centre or an equal gap.</p>
<h2>Automation server</h2>
<p>Behind the scenes the same commands are an <strong>automation server</strong> running inside
CncCraft on your own computer only (localhost). An AI assistant can connect to it and build a drawing
for you while the visible mouse shows every move. Nothing leaves your machine.</p>
<h2>All commands</h2>
<p>These are every command a script (and the automation server) understands – the same set the AI
assistant sees in <code>agent.md</code>.</p>
<!--COMMANDS-->
""",
        "body": """
<div class="pilot-box"><p><strong>Dies ist eine Pilotfunktion (Beta).</strong> Skripte und der
eingebaute Automatisierungs-Server sind neu, werden noch verfeinert und können sich ändern. Sie sind
vor allem zum Aufnehmen von Demos gedacht; für die tägliche Arbeit braucht man sie nicht.</p></div>
<p>CncCraft kann ein <strong>Skript</strong> ausführen: eine einfache Liste von Befehlen, einer pro
Zeile, die es ausführt, indem es die <strong>sichtbare Maus</strong> bewegt und die echten Knöpfe
anklickt – es sieht also genau so aus, als würde jemand das Programm bedienen.</p>
<div class="tipp"><p>Das eignet sich gut, um <strong>kurze Demo-Videos für TikTok, Instagram oder
YouTube Shorts</strong> zu bauen: der große sichtbare Zeiger, die eingeblendeten Beschriftungen und
die ruhigen, wiederholbaren Bewegungen ergeben saubere Clips ganz ohne Hand. Nehmen Sie das Fenster
auf, setzen Sie Titel und Untertitel, wählen Sie das Tempo – und lassen Sie das Skript zeichnen und
fräsen, während die Maus jeden Schritt vorführt.</p></div>
<h2>Das Skriptfenster</h2>
<p>Öffnen mit <em>Hilfe ▸ Skript…</em>. Es ist ein eigenes Fenster, das Sie überallhin schieben
können. Es hat einen Editor mit Autovervollständigung und Syntaxhervorhebung, einen Knopf
<strong>Ausführen</strong> und <strong>Stopp</strong>, ein Fenster <strong>Befehlshilfe</strong> mit
allen Befehlen und einen <strong>agent.md</strong>-Knopf, der eine vollständige Anleitung für einen
KI-Assistenten kopiert.</p>
<h2>Wie ein Skript aussieht</h2>
<ul>
<li>Ein Befehl pro Zeile, zum Beispiel <code>draw_rectangle cx=50 cy=50 width=40 height=30</code>.</li>
<li>Mit <code>name = tool …</code> merken Sie sich das Ergebnis unter einem Namen und verwenden es
später als <code>$name.id</code> – so können Sie eine Form zeichnen und ihr einen Werkzeugweg
geben.</li>
<li>Zeilen wie <code>move_to</code>, <code>click</code>, <code>drag</code> und <code>type_text</code>
laufen über den sichtbaren Zeiger; <code>screenshot</code>, <code>recording_start</code> /
<code>recording_stop</code> und <code>window width=… height=…</code> helfen beim Aufnehmen.</li>
<li><code>text_title</code> und <code>text_subtitle</code> blenden eine Beschriftung ein,
<code>replay_speed</code> stellt ein, wie schnell die Maus sich bewegt.</li>
</ul>
<h2>Einrasten aus dem Skript</h2>
<p>Jeder Knopf hat einen Befehl, also kann ein Skript die drei Einrast-Methoden
([[einrasten|Einrasten]]) wie jeden anderen ein- und ausschalten: <code>snap_to_geometry</code>
(Bounding-Box), <code>snap_to_center</code> und <code>snap_to_gap</code>. So lässt sich in einer
Demo Schritt für Schritt zeigen, wie eine Form an einer Kante, einer Mitte oder einem gleichen
Abstand einrastet.</p>
<h2>Automatisierungs-Server</h2>
<p>Im Hintergrund sind dieselben Befehle ein <strong>Automatisierungs-Server</strong>, der in
CncCraft nur auf Ihrem eigenen Rechner läuft (localhost). Ein KI-Assistent kann sich damit verbinden
und eine Zeichnung für Sie bauen, während die sichtbare Maus jeden Schritt vorführt. Nichts verlässt
Ihren Rechner.</p>
<h2>Alle Befehle</h2>
<p>Dies sind alle Befehle, die ein Skript (und der Automatisierungs-Server) versteht – genau die
Menge, die auch der KI-Assistent in <code>agent.md</code> sieht. Die Befehlsnamen sind englisch.</p>
<!--COMMANDS-->
""",
    },
]
