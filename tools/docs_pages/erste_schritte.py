"""Kategorie „Erste Schritte“."""

C = "erste-schritte"

PAGES = [
    {
        "slug": "einfuehrung", "cat": C, "title": "Einführung",
        "title_en": "Introduction",
        "lead": "CncCraft ist ein kostenloses, parametrisches CAD/CAM-Programm für Einsteiger: Sie entwerfen Teile für eine CNC-Fräse mit Maßen, die sich ändern lassen, berechnen die Werkzeugwege und geben sie als Datei an die Maschine.",
        "lead_en": "CncCraft is a free, parametric CAD/CAM program for beginners: you design parts for a CNC router with dimensions you can change, compute the toolpaths and hand them to the machine as a file.",
        "body_en": """
<div class="neu-box"><p><strong>Parametric means:</strong> a dimension is not a fixed line but a value you can change at any time – and everything tied to it follows. Write the board width as the variable <code>breite</code>, for example, and holes, distances, [[hilfslinien|construction lines]] and toolpaths move along when you change <code>breite</code>. What professional CAD programs like Fusion 360 can do, CncCraft brings in a simple form – no licence, no subscription, no account.</p></div>
<p>The path from idea to milled part has three stages in CncCraft, all in one window:</p>
<ol>
<li><strong>Design:</strong> you draw shapes or type dimensions – numbers, [[variablen|variables]] or [[formeln-in-feldern|formulas]]. Layers, snapping, construction lines and constraints help you work precisely (start here: [[parametrisch-uebersicht]]).</li>
<li><strong>Generate toolpaths:</strong> you select shapes and say what the router should do with them – cut out, clear a pocket, mill a slot, drill or carve lettering with a V-bit. For the chosen [[materialien|material]] CncCraft suggests matching cutting values and warns when one is too high.</li>
<li><strong>Output:</strong> after a preview in 2D or 3D you save the toolpaths as G-code.</li>
</ol>
{{img:oberflaeche.png|The main window: tools on the left, the drawing in the middle, the toolpaths on the right}}
<h2>The window at a glance</h2>
<table>
<tr><th>Area</th><th>What you find there</th></tr>
<tr><td>Menu bar (top left)</td><td>File, Edit, View, Toolpaths, Help.</td></tr>
<tr><td>Layer picker (top centre)</td><td>The active layer and the list of all layers. See [[ebenenliste]].</td></tr>
<tr><td>Toolbar (left)</td><td>Drawing and editing tools, plus job setup and variables.</td></tr>
<tr><td>Header above the drawing</td><td>Decoration (origin, grid, snapping, dimensions, rulers) and constraints; the view buttons on the right.</td></tr>
<tr><td>Drawing area (centre)</td><td>The workpiece with rulers, construction lines and your shapes – or the 3D preview.</td></tr>
<tr><td>Toolpaths (right)</td><td>Board setup, the buttons for new toolpaths, the list, preview and output.</td></tr>
<tr><td>Status line (bottom)</td><td>Size of the selection, pointer position and the layer of the selection.</td></tr>
</table>
<p>More on the [[entwurfsansicht]] page. To dive straight in, open the “sign with lettering, pocket and holes” example on the [[startseite|start page]] and see what a finished job looks like.</p>
<div class="hinweis"><p>CncCraft is a standalone re-creation that follows Spark by Vectric and extends it in places. Only what CncCraft can do is documented here.</p></div>
""",
        "body": """
<div class="neu-box"><p><strong>Parametrisch heißt:</strong> Ein Maß ist kein fester Strich, sondern ein Wert, den Sie jederzeit ändern – und alles, was daran hängt, folgt. Schreiben Sie etwa die Breite der Platte als Variable <code>breite</code>, wandern Bohrungen, Abstände, [[hilfslinien|Hilfslinien]] und Werkzeugwege mit, wenn Sie <code>breite</code> ändern. Was Profi-CAD-Programme wie Fusion 360 können, bringt CncCraft in einfacher Form mit – ohne Lizenz, ohne Abo, ohne Konto.</p></div>
<p>Der Weg vom Einfall zum gefrästen Teil hat bei CncCraft drei Stationen, die sich in einem Fenster abspielen:</p>
<ol>
<li><strong>Entwerfen:</strong> Sie zeichnen Formen oder tippen Maße ein – Zahlen, [[variablen|Variablen]] oder [[formeln-in-feldern|Formeln]]. Ebenen, Einrasten, Hilfslinien und Bedingungen helfen, genau zu arbeiten (Anfang: [[parametrisch-uebersicht]]).</li>
<li><strong>Werkzeugwege berechnen:</strong> Sie wählen Formen aus und sagen, was die Fräse damit tun soll – ausschneiden, eine Tasche ausräumen, eine Nut fräsen, bohren oder eine Schrift mit V-Fräser schnitzen. Zum gewählten [[materialien|Material]] schlägt CncCraft passende Schnittwerte vor und warnt, wenn ein Wert zu hoch ist.</li>
<li><strong>Ausgeben:</strong> Nach einer Vorschau in 2D oder 3D speichern Sie die Werkzeugwege als G-Code.</li>
</ol>
{{img:oberflaeche.png|Das Hauptfenster: links die Werkzeuge, in der Mitte die Zeichnung, rechts die Werkzeugwege}}
<h2>Das Fenster im Überblick</h2>
<table>
<tr><th>Bereich</th><th>Was Sie dort finden</th></tr>
<tr><td>Menüleiste (oben links)</td><td>Datei, Bearbeiten, Ansicht, Werkzeugwege, Hilfe.</td></tr>
<tr><td>Ebenenwahl (oben Mitte)</td><td>Die aktive Ebene und die Liste aller Ebenen. Siehe [[ebenenliste]].</td></tr>
<tr><td>Werkzeugleiste (links)</td><td>Zeichen- und Bearbeitungswerkzeuge, darunter Job-Einrichtung und Variablen.</td></tr>
<tr><td>Kopfleiste über der Zeichnung</td><td>Dekoration (Nullpunkt, Raster, Einrasten, Maße, Lineale) und Constraints; rechts die Ansichtsknöpfe.</td></tr>
<tr><td>Zeichenfläche (Mitte)</td><td>Das Werkstück mit Linealen, Hilfslinien und Ihren Formen – oder die 3D-Vorschau.</td></tr>
<tr><td>Werkzeugpfade (rechts)</td><td>Platten-Setup, die Knöpfe für neue Werkzeugwege, die Liste, Vorschau und Ausgabe.</td></tr>
<tr><td>Statuszeile (unten)</td><td>Größe der Auswahl, Zeigerposition und die Ebene der Auswahl.</td></tr>
</table>
<p>Mehr dazu auf der Seite [[entwurfsansicht]]. Wer gleich loslegen möchte, öffnet auf der [[startseite]] das Beispiel „Schild mit Schrift, Tasche und Bohrungen“ und sieht, wie ein fertiger Job aussieht.</p>
<div class="hinweis"><p>CncCraft ist ein eigenständiger Nachbau, der sich an Spark von Vectric orientiert und einiges erweitert. Dokumentiert ist hier nur, was CncCraft kann.</p></div>
""",
    },
    {
        "slug": "installation", "cat": C, "title": "Installation",
        "title_en": "Installation",
        "lead": "Fertige Pakete für macOS, Windows und Linux gibt es bei den GitHub-Releases – oder bauen Sie CncCraft selbst aus dem Quelltext.",
        "lead_en": "Download ready-made packages for macOS, Windows and Linux from the GitHub releases – or build CncCraft yourself from source.",
        "body_en": """
<h2>Download (recommended)</h2>
<p>The quickest way is a ready-made package from the releases page. Pick the file for your system,
open it and start CncCraft – no compiler needed.</p>
<p><a href="https://github.com/freegroup/CncCraft/releases"><strong>Downloads on GitHub →</strong></a></p>
<table>
<tr><th>System</th><th>File</th></tr>
<tr><td>macOS (Apple Silicon / Intel)</td><td><code>.dmg</code> or <code>.zip</code></td></tr>
<tr><td>Windows</td><td>the installer (<code>.exe</code>) or <code>.zip</code></td></tr>
<tr><td>Linux (x86_64)</td><td><code>.deb</code> or <code>.tar.gz</code></td></tr>
</table>
<div class="hinweis"><p>The Mac app is only “ad hoc” signed. On other Macs a security warning
appears on first launch; without an Apple developer account it cannot be avoided.</p></div>
<p>The source code lives on GitHub too:
<a href="https://github.com/freegroup/CncCraft">github.com/freegroup/CncCraft</a>.</p>
<h2>Building from source</h2>
<p>All packages are produced with one call to <code>build.sh</code> (or just <code>make</code>) in the program's folder. Finished files land in the <code>dist/</code> subfolder.</p>
<table>
<tr><th>Command</th><th>Result</th><th>Runs on</th></tr>
<tr><td><code>./build.sh mac</code></td><td><code>CncCraft.app</code>, <code>.zip</code> and <code>.dmg</code> for Apple Silicon</td><td>a Mac</td></tr>
<tr><td><code>./build.sh mac-intel</code></td><td>the same for Intel Macs</td><td>a Mac</td></tr>
<tr><td><code>./build.sh windows</code></td><td><code>CncCraft.exe</code>, <code>.zip</code> and – with Inno Setup – an installer</td><td>Windows</td></tr>
<tr><td><code>./build.sh linux</code></td><td><code>.tar.gz</code> and <code>.deb</code> (x86_64)</td><td>Linux</td></tr>
<tr><td><code>./build.sh all</code></td><td>everything possible on this machine</td><td></td></tr>
</table>
<h2>Requirements</h2>
<ul>
<li>The Rust language with <code>cargo</code> (for compiling).</li>
<li>macOS 12 or newer for the Mac packages.</li>
<li>For the Windows installer: Inno Setup.</li>
</ul>
<h2>Building packages on GitHub</h2>
<p>If you have no Windows or Linux machines of your own, you can have all packages built on GitHub: the included workflow <code>.github/workflows/release.yml</code> starts as soon as a version tag like <code>v0.1.0</code> is pushed, and attaches the packages to the <a href="https://github.com/freegroup/CncCraft/releases">release</a>.</p>
<h2>Checking that it runs</h2>
<p>With <code>cnccraft --selftest</code> the program runs a self-test and reports every step. With <code>cnccraft --screenshot image.png</code> it saves a picture of the window without any input from you – that is how the pictures in this help were made.</p>
""",
        "body": """
<h2>Herunterladen (empfohlen)</h2>
<p>Am schnellsten geht es mit einem fertigen Paket von der Releases-Seite. Wählen Sie die Datei für
Ihr System, öffnen Sie sie und starten Sie CncCraft – kein Compiler nötig.</p>
<p><a href="https://github.com/freegroup/CncCraft/releases"><strong>Downloads bei GitHub →</strong></a></p>
<table>
<tr><th>System</th><th>Datei</th></tr>
<tr><td>macOS (Apple Silicon / Intel)</td><td><code>.dmg</code> oder <code>.zip</code></td></tr>
<tr><td>Windows</td><td>das Installationsprogramm (<code>.exe</code>) oder <code>.zip</code></td></tr>
<tr><td>Linux (x86_64)</td><td><code>.deb</code> oder <code>.tar.gz</code></td></tr>
</table>
<div class="hinweis"><p>Die Mac-App ist nur „ad hoc“ signiert. Auf fremden Macs erscheint daher beim
ersten Start eine Sicherheitswarnung; ohne Apple-Entwicklerkonto lässt sie sich nicht vermeiden.</p></div>
<p>Der Quelltext liegt ebenfalls bei GitHub:
<a href="https://github.com/freegroup/CncCraft">github.com/freegroup/CncCraft</a>.</p>
<h2>Selbst aus dem Quelltext bauen</h2>
<p>Alle Pakete entstehen mit einem Aufruf von <code>build.sh</code> (oder kurz <code>make</code>) im Ordner des Programms. Fertige Dateien landen im Unterordner <code>dist/</code>.</p>
<table>
<tr><th>Befehl</th><th>Ergebnis</th><th>Läuft auf</th></tr>
<tr><td><code>./build.sh mac</code></td><td><code>CncCraft.app</code>, <code>.zip</code> und <code>.dmg</code> für Apple Silicon</td><td>einem Mac</td></tr>
<tr><td><code>./build.sh mac-intel</code></td><td>dasselbe für Intel-Macs</td><td>einem Mac</td></tr>
<tr><td><code>./build.sh windows</code></td><td><code>CncCraft.exe</code>, <code>.zip</code> und – mit Inno Setup – ein Installationsprogramm</td><td>Windows</td></tr>
<tr><td><code>./build.sh linux</code></td><td><code>.tar.gz</code> und <code>.deb</code> (x86_64)</td><td>Linux</td></tr>
<tr><td><code>./build.sh all</code></td><td>alles, was auf diesem Rechner möglich ist</td><td></td></tr>
</table>
<h2>Voraussetzungen</h2>
<ul>
<li>Die Programmiersprache Rust mit <code>cargo</code> (zum Übersetzen).</li>
<li>macOS 12 oder neuer für die Mac-Pakete.</li>
<li>Für das Windows-Installationsprogramm: Inno Setup.</li>
</ul>
<h2>Pakete bei GitHub bauen lassen</h2>
<p>Wer keine eigenen Windows- oder Linux-Rechner hat, kann alle Pakete bei GitHub bauen lassen: Die mitgelieferte Arbeitsanweisung <code>.github/workflows/release.yml</code> startet, sobald ein Versions-Tag wie <code>v0.1.0</code> hochgeladen wird, und hängt die Pakete an die <a href="https://github.com/freegroup/CncCraft/releases">Veröffentlichung</a>.</p>
<h2>Prüfen, ob es läuft</h2>
<p>Mit <code>cnccraft --selftest</code> führt das Programm einen Selbsttest aus und meldet jeden Schritt. Mit <code>cnccraft --screenshot bild.png</code> speichert es ein Bild des Fensters, ohne dass Sie etwas bedienen müssen – so sind die Abbildungen dieser Hilfe entstanden.</p>
""",
    },
    {
        "slug": "startseite", "cat": C, "title": "Startseite",
        "title_en": "Start page",
        "lead": "Beim Start zeigt CncCraft drei Kacheln: neue Datei, Datei öffnen und das Beispiel.",
        "lead_en": "At launch CncCraft shows three tiles: new file, open file and the example.",
        "body_en": """
<p>The start page appears as long as no job is open yet. You can reach it later at any time via <em>File ▸ Start page</em>.</p>
<table>
<tr><th>Tile</th><th>What happens</th></tr>
<tr><td><strong>Create new file</strong></td><td>Opens the [[neuer-job]] dialog, where you set the material size and thickness.</td></tr>
<tr><td><strong>Open file</strong></td><td>Loads a saved project (extension <code>.cnccraft</code>). See [[dateien-speichern]].</td></tr>
<tr><td><strong>Open example</strong></td><td>Loads a sign with lettering, a pocket and holes, with toolpaths – a good starting point to try the preview.</td></tr>
</table>
<p>The example is also available at any time via <em>File ▸ Open example</em>.</p>
""",
        "body": """
<p>Die Startseite erscheint, solange noch kein Job offen ist. Sie erreichen sie später jederzeit über <em>Datei ▸ Startseite</em>.</p>
<table>
<tr><th>Kachel</th><th>Was passiert</th></tr>
<tr><td><strong>Neue Datei erstellen</strong></td><td>Öffnet den Dialog [[neuer-job]], in dem Sie Materialgröße und Dicke festlegen.</td></tr>
<tr><td><strong>Datei öffnen</strong></td><td>Lädt ein gespeichertes Projekt (Endung <code>.cnccraft</code>). Siehe [[dateien-speichern]].</td></tr>
<tr><td><strong>Beispiel öffnen</strong></td><td>Lädt ein Schild mit Schrift, Tasche und Bohrungen samt Werkzeugwegen – ein guter Ausgangspunkt, um die Vorschau auszuprobieren.</td></tr>
</table>
<p>Das Beispiel ist auch jederzeit über <em>Datei ▸ Beispiel öffnen</em> erreichbar.</p>
""",
    },
    {
        "slug": "neuer-job", "cat": C, "title": "Neuen Job erstellen und einrichten",
        "title_en": "Create and set up a new job",
        "lead": "Der Job beschreibt das Werkstück: Größe, Material und Nullpunkt der Fräse.",
        "lead_en": "The job describes the workpiece: size, material and the router's origin.",
        "body_en": """
<p>The dialog is called “Create job” when you begin a new file, and “Set up job” when you open it later via <em>Edit ▸ Set up job …</em>, the button in the toolbar or the “Board setup” row on the right.</p>
{{img:job-dialog.png|The “Create job” dialog}}
<h2>Job size</h2>
<ul>
<li><strong>Width (X), length (Y), thickness (Z):</strong> the dimensions of the blank. The small sketch shows which side matches which axis.</li>
<li><strong>Units:</strong> inches or millimetres. On switching, the numbers are converted; internally the file is stored in millimetres.</li>
</ul>
<h2>Material</h2>
<p>No job can be created without a material – the <strong>Create</strong> button stays grey until one is chosen. The list holds ten materials from softwood to aluminium, each with a small swatch. It determines the tools and cutting values (see [[werkzeugauswahl]]), the warnings for cuts that are too fast or too deep, and the look in the [[ansicht-3d|3D view]].</p>
<ul>
<li><strong>Recommendations</strong> (collapsed): a click shows spindle speed, maximum feed, stepdown, suitable cutters and cooling for the material.</li>
<li><strong>Manage …</strong> at the end of the list opens the material settings, where you adjust values and add your own materials ([[materialien]]).</li>
</ul>
<h2>Origin</h2>
<ul>
<li><strong>Z zero:</strong> <em>Surface</em> – the top of the material is height 0, cut depths count downward. <em>Table</em> – the bottom is height 0.</li>
<li><strong>XY zero:</strong> <em>Corner</em> (lower left) or <em>Centre</em> of the workpiece. That is where the origin sits in the G-code file later too.</li>
<li><strong>Safe Z</strong> and <strong>Start Z:</strong> the heights the router travels at between cuts, and the machine's start height.</li>
</ul>
<div class="tipp"><p>Size and thickness are then available as the variables <code>platte_b</code>, <code>platte_l</code> and <code>platte_d</code>. So a cut depth can be entered as <code>platte_d/2</code>, for example – see [[variablen]].</p></div>
<p>If you change an existing job, CncCraft recomputes all toolpaths with the new values.</p>
<div class="tipp"><p>A new job always starts with your last settings – size, units, material and origin. If you mostly mill the same board, you only have to click <strong>Create</strong>.</p></div>
""",
        "body": """
<p>Der Dialog heißt „Job erstellen“, wenn Sie eine neue Datei beginnen, und „Job einrichten“, wenn Sie ihn später über <em>Bearbeiten ▸ Job einrichten …</em>, den Knopf in der Werkzeugleiste oder die Zeile „Platten-Setup“ rechts öffnen.</p>
{{img:job-dialog.png|Der Dialog „Job erstellen“}}
<h2>Auftragsgröße</h2>
<ul>
<li><strong>Breite (X), Länge (Y), Dicke (Z):</strong> die Maße des Rohlings. Die kleine Skizze zeigt, welche Seite welcher Achse entspricht.</li>
<li><strong>Einheiten:</strong> Zoll oder Millimeter. Beim Umschalten werden die Zahlen umgerechnet; gespeichert wird intern in Millimetern.</li>
</ul>
<h2>Material</h2>
<p>Ohne Material lässt sich kein Job anlegen – der Knopf <strong>Erstellen</strong> bleibt grau, bis eines gewählt ist. Die Liste enthält zehn Materialien vom Weichholz bis zum Aluminium, jedes mit einem kleinen Muster. Es bestimmt die Werkzeuge und Schnittwerte (siehe [[werkzeugauswahl]]), die Warnungen bei zu schnellen oder zu tiefen Schnitten und das Aussehen in der [[ansicht-3d|3D-Ansicht]].</p>
<ul>
<li><strong>Empfehlungen</strong> (zugeklappt): ein Klick zeigt Drehzahl, höchsten Vorschub, Zustellung, passende Fräser und Kühlung für das Material.</li>
<li><strong>Verwalten …</strong> am Ende der Liste öffnet die Materialeinstellungen, in denen Sie Werte anpassen und eigene Materialien anlegen ([[materialien]]).</li>
</ul>
<h2>Nullpunkt</h2>
<ul>
<li><strong>Z-Null:</strong> <em>Oberfläche</em> – die Oberseite des Materials ist Höhe 0, Schnitttiefen zählen nach unten. <em>Tisch</em> – die Unterseite ist Höhe 0.</li>
<li><strong>XY-Null:</strong> <em>Ecke</em> (unten links) oder <em>Mitte</em> des Werkstücks. Dort sitzt später auch der Nullpunkt in der G-Code-Datei.</li>
<li><strong>Sicher Z</strong> und <strong>Start Z:</strong> die Höhen, auf die die Fräse zwischen den Schnitten fährt, und die Starthöhe der Maschine.</li>
</ul>
<div class="tipp"><p>Größe und Dicke stehen danach als Variablen <code>platte_b</code>, <code>platte_l</code> und <code>platte_d</code> zur Verfügung. Damit lässt sich zum Beispiel eine Schnitttiefe als <code>platte_d/2</code> eintragen – siehe [[variablen]].</p></div>
<p>Wenn Sie einen bestehenden Job ändern, berechnet CncCraft alle Werkzeugwege mit den neuen Werten neu.</p>
<div class="tipp"><p>Ein neuer Job beginnt immer mit Ihren letzten Einstellungen – Größe, Einheiten, Material und Nullpunkt. Wer meist dieselbe Platte fräst, muss nur noch auf <strong>Erstellen</strong> klicken.</p></div>
""",
    },
    {
        "slug": "entwurfsansicht", "cat": C, "title": "Die Entwurfsansicht",
        "title_en": "The design view",
        "lead": "In der 2D-Ansicht zeichnen und bearbeiten Sie; Lineale, Raster und Einrasthilfen unterstützen Sie dabei.",
        "lead_en": "In the 2D view you draw and edit; rulers, grid and snapping help you.",
        "body_en": """
<p>The drawing area shows the workpiece from above: the board with rulers along the top and left edge and the coordinate cross at the origin. The value in the ruler is in the job's unit. A click in the ruler adds a [[hilfslinien|construction line]].</p>
<h2>Moving around</h2>
<ul>
<li><strong>Zoom:</strong> with the mouse wheel or a two-finger gesture (toward the pointer).</li>
<li><strong>Pan:</strong> drag with the middle or right mouse button.</li>
<li><strong>Whole view</strong> (<kbd>F4</kbd>): shows the whole workpiece. <strong>Zoom to selection:</strong> shows the selected shapes.</li>
</ul>
<h2>The header</h2>
<p>Above the drawing are two groups of switches:</p>
<ul>
<li><strong>Decoration:</strong> show the origin, snap to the grid, snap to geometry (see [[einrasten]]), show the dimensions, and show or hide the <strong>rulers</strong> with the construction lines. Each decoration is also a tool: the grid is only visible while snapping to it is on; the rulers are only there while you can set construction lines with them.</li>
<li><strong>Constraints:</strong> <em>Distance</em> (<kbd>D</kbd>) and <em>Level/upright</em> (<kbd>H</kbd>) – see [[abstand-bemassen]] and [[waagrecht-senkrecht]].</li>
</ul>
<p>Top right, the <strong>2D/3D</strong> button switches between design and the [[ansicht-3d|3D view]]; next to it a button turns the origin (axis cross) on or off.</p>
<h2>The toolbar</h2>
<p>On the left the tools sit in groups: mode (select, transform, node edit), lines, shapes, text, edit objects, align and mirror, offset, pattern, boolean operations and – set apart – job setup and the variables. The small arrow at the top left shrinks the bar; each group then shows only the last tool used.</p>
{{img:werkzeugleiste-eingeklappt.png|The shrunk toolbar}}
<p>Hover over a tool and a short tooltip appears. The status line at the bottom also says what to do next (for example “Pick the second point”).</p>
<h2>Worksheet decorations</h2>
<p>What the drawing area shows in addition you switch one by one in <em>View ▸ Worksheet</em>: board, rulers, coordinate cross and hints. The grid is not there – it follows the “Snap to grid” switch.</p>
<h2>Colours</h2>
<p>Shapes, their toolpaths in the list and the drawn cutter paths all carry the colour of their layer. Change a layer's colour ([[ebenenliste]]) and all three change with it.</p>
""",
        "body": """
<p>Die Zeichenfläche zeigt das Werkstück von oben: die Platte mit Linealen am oberen und linken Rand und das Koordinatenkreuz am Nullpunkt. Der Wert im Lineal ist in der Einheit des Jobs. Ein Klick ins Lineal legt eine [[hilfslinien|Hilfslinie]] an.</p>
<h2>Sich bewegen</h2>
<ul>
<li><strong>Zoomen:</strong> mit dem Mausrad oder einer Zwei-Finger-Geste (zum Zeiger hin).</li>
<li><strong>Verschieben:</strong> mit der mittleren oder rechten Maustaste ziehen.</li>
<li><strong>Ganze Ansicht</strong> (<kbd>F4</kbd>): zeigt das ganze Werkstück. <strong>Auf die Auswahl zoomen:</strong> zeigt die gewählten Formen.</li>
</ul>
<h2>Die Kopfleiste</h2>
<p>Über der Zeichnung liegen zwei Gruppen von Schaltern:</p>
<ul>
<li><strong>Dekoration:</strong> Nullpunkt zeigen, am Raster einrasten, an Geometrie einrasten (siehe [[einrasten]]), die Maße anzeigen und die <strong>Lineale</strong> mit den Hilfslinien ein- oder ausblenden. Jede Dekoration ist zugleich Werkzeug: Das Raster ist nur zu sehen, solange daran eingerastet wird; die Lineale sind nur da, solange man mit ihnen Hilfslinien setzen kann.</li>
<li><strong>Constraints:</strong> <em>Abstand</em> (<kbd>D</kbd>) und <em>Waagrecht/Senkrecht</em> (<kbd>H</kbd>) – siehe [[abstand-bemassen]] und [[waagrecht-senkrecht]].</li>
</ul>
<p>Rechts oben wechselt der Knopf <strong>2D/3D</strong> zwischen Entwurf und [[ansicht-3d]]; daneben schaltet ein Knopf den Nullpunkt (Achsenkreuz) ein oder aus.</p>
<h2>Die Werkzeugleiste</h2>
<p>Links liegen die Werkzeuge in Gruppen: Modus (Auswählen, Transformieren, Knoten bearbeiten), Linien, Formen, Text, Objekte bearbeiten, Ausrichten und Spiegeln, Versatz, Vervielfältigen, Boolesche Operationen sowie – abgesetzt – die Job-Einrichtung und die Variablen. Mit dem kleinen Pfeil oben links lässt sich die Leiste verkleinern; dann zeigt jede Gruppe nur das zuletzt benutzte Werkzeug.</p>
{{img:werkzeugleiste-eingeklappt.png|Die verkleinerte Werkzeugleiste}}
<p>Fahren Sie mit der Maus über ein Werkzeug, erscheint ein kurzer Hilfetext. Die Statuszeile unten nennt außerdem, was als Nächstes zu tun ist (zum Beispiel „Zweiten Punkt wählen“).</p>
<h2>Dekorationen der Arbeitsfläche</h2>
<p>Was die Zeichenfläche zusätzlich zeigt, schalten Sie einzeln im Menü <em>Ansicht ▸ Arbeitsfläche</em>: Platte, Lineale, Koordinatenkreuz und Hinweise. Das Raster steht dort nicht – es folgt dem Schalter „Am Raster einrasten“.</p>
<h2>Farben</h2>
<p>Formen, ihre Werkzeugwege in der Liste und die gezeichneten Fräsbahnen tragen die Farbe ihrer Ebene. Ändern Sie die Farbe einer Ebene ([[ebenenliste]]), ändern sich alle drei mit.</p>
""",
    },
    {
        "slug": "ansicht-3d", "cat": C, "title": "Die 3D-Ansicht",
        "title_en": "The 3D view",
        "lead": "Die 3D-Ansicht zeigt das Werkstück, so wie es nach dem Fräsen aussehen wird.",
        "lead_en": "The 3D view shows the workpiece as it will look after milling.",
        "body_en": """
<p>With the <strong>2D/3D</strong> button top right (or <kbd>F2</kbd> for 2D and <kbd>F3</kbd> for 3D) you switch the view. In 3D you see the material as a solid; what the toolpaths remove is cut away.</p>
{{img:ansicht-3d.png|The 3D view with the view cube and the origin axis cross}}
<h2>Controls</h2>
<ul>
<li><strong>Rotate:</strong> drag with the left mouse button.</li>
<li><strong>Pan:</strong> drag with the middle or right mouse button.</li>
<li><strong>Zoom:</strong> mouse wheel or two-finger gesture.</li>
<li><strong>Perspective:</strong> in <em>View ▸ Perspective (3D)</em> you switch between perspective and parallel projection.</li>
</ul>
<h2>Orientation aids</h2>
<div class="neu-box"><p>Two decorations help you get your bearings in 3D; both can be switched off individually in <em>View ▸ 3D view</em>:</p>
<ul>
<li><strong>View cube:</strong> shows which side you are looking from.</li>
<li><strong>Origin axis cross:</strong> draws the router's axis cross (X red, Y green, Z blue) at the job's origin. It follows the chosen origin (corner or centre, surface or table).</li>
</ul></div>
<p>The origin button shows or hides the axis cross in both views at once.</p>
<p>The toolpaths themselves you overlay with the “Show G-code motion” option — here in 3D as the machine's moves, and in the 2D view as a band in the cutter's width; more under [[vorschau-werkzeugwege]].</p>
""",
        "body": """
<p>Mit dem Knopf <strong>2D/3D</strong> rechts oben (oder <kbd>F2</kbd> für 2D und <kbd>F3</kbd> für 3D) wechseln Sie die Ansicht. In 3D sehen Sie das Material als Körper; was die Werkzeugwege abtragen, ist ausgespart.</p>
{{img:ansicht-3d.png|Die 3D-Ansicht mit Ansichtswürfel und Achsenkreuz des Nullpunkts}}
<h2>Bedienung</h2>
<ul>
<li><strong>Drehen:</strong> mit der linken Maustaste ziehen.</li>
<li><strong>Verschieben:</strong> mit der mittleren oder rechten Maustaste ziehen.</li>
<li><strong>Zoomen:</strong> Mausrad oder Zwei-Finger-Geste.</li>
<li><strong>Perspektive:</strong> im Menü <em>Ansicht ▸ Perspektivische Darstellung (3D)</em> wechseln Sie zwischen perspektivischer und paralleler Darstellung.</li>
</ul>
<h2>Orientierungshilfen</h2>
<div class="neu-box"><p>Zwei Dekorationen helfen in 3D, sich zurechtzufinden; beide lassen sich im Menü <em>Ansicht ▸ 3D-Ansicht</em> einzeln abschalten:</p>
<ul>
<li><strong>Ansichtswürfel:</strong> zeigt, von welcher Seite Sie schauen.</li>
<li><strong>Nullpunkt-Achsenkreuz:</strong> zeichnet das Achsenkreuz der Fräse (X rot, Y grün, Z blau) an den Nullpunkt des Jobs. Es folgt dem eingestellten Nullpunkt (Ecke oder Mitte, Oberfläche oder Tisch).</li>
</ul></div>
<p>Der Knopf für den Nullpunkt blendet das Achsenkreuz in beiden Ansichten zugleich ein oder aus.</p>
<p>Die Werkzeugwege selbst blenden Sie mit der Option „G-Code-Fahrt einblenden“ ein — hier in 3D als die Fahrten der Maschine, und in der 2D-Ansicht als Band in Fräserbreite; mehr dazu unter [[vorschau-werkzeugwege]].</p>
""",
        "neu": True,
    },
    {
        "slug": "ebenenliste", "cat": C, "title": "Ebenen",
        "title_en": "Layers",
        "lead": "Ebenen gruppieren Formen: Sie lassen sich ein- und ausblenden, sperren und färben.",
        "lead_en": "Layers group shapes: they can be shown and hidden, locked and coloured.",
        "body_en": """
<p>Top centre, the layer picker shows which layer you are drawing on. A click opens the list of all layers.</p>
{{img:ebenen.png|The open layer list}}
<table>
<tr><th>Element</th><th>Meaning</th></tr>
<tr><td>Eye</td><td><strong>Visible / hidden.</strong> Hidden shapes are not shown, not milled and not exported.</td></tr>
<tr><td>Lock</td><td><strong>Locked / unlocked.</strong> Locked shapes stay visible but cannot be clicked or changed.</td></tr>
<tr><td>Name</td><td>A click makes the layer active – new shapes are created on it. Double-click renames it.</td></tr>
<tr><td>Colour dot</td><td>The layer's colour; a click changes it.</td></tr>
</table>
<h2>At the bottom of the list</h2>
<ul>
<li><strong>+</strong> adds a new layer ([[ebene-neu]]), <strong>−</strong> deletes the active layer ([[ebene-loeschen]]).</li>
<li><strong>Selection to here</strong> moves the selected shapes onto the active layer.</li>
</ul>
<div class="hinweis"><p>A drawing always has at least one layer; the last one cannot be deleted.</p></div>
""",
        "body": """
<p>Oben in der Mitte zeigt der Ebenenwähler, auf welcher Ebene Sie gerade zeichnen. Ein Klick öffnet die Liste aller Ebenen.</p>
{{img:ebenen.png|Die geöffnete Ebenenliste}}
<table>
<tr><th>Element</th><th>Bedeutung</th></tr>
<tr><td>Auge</td><td><strong>Sichtbar / ausgeblendet.</strong> Ausgeblendete Formen werden nicht gezeigt, nicht gefräst und nicht exportiert.</td></tr>
<tr><td>Schloss</td><td><strong>Gesperrt / entsperrt.</strong> Gesperrte Formen bleiben sichtbar, lassen sich aber nicht anklicken oder ändern.</td></tr>
<tr><td>Name</td><td>Ein Klick macht die Ebene aktiv – neue Formen entstehen auf ihr. Doppelklick benennt sie um.</td></tr>
<tr><td>Farbpunkt</td><td>Die Farbe der Ebene; ein Klick wechselt sie.</td></tr>
</table>
<h2>Unten in der Liste</h2>
<ul>
<li><strong>+</strong> legt eine neue Ebene an ([[ebene-neu]]), <strong>−</strong> löscht die aktive Ebene ([[ebene-loeschen]]).</li>
<li><strong>Auswahl hierher</strong> verschiebt die gewählten Formen auf die aktive Ebene.</li>
</ul>
<div class="hinweis"><p>Eine Zeichnung hat immer mindestens eine Ebene; die letzte lässt sich nicht löschen.</p></div>
""",
    },
    {
        "slug": "ebene-neu", "cat": C, "title": "Neue Ebene anlegen",
        "title_en": "Add a new layer",
        "lead": "Eine neue Ebene entsteht mit einem Klick auf das Plus in der Ebenenliste.",
        "lead_en": "A new layer is created with a click on the plus in the layer list.",
        "body_en": """
<ol>
<li>Open the layer list (click the layer picker, top centre).</li>
<li>Click <strong>+</strong> at the bottom. The new layer gets a name and a colour from the palette and becomes active at once.</li>
<li>Give it your own name by double-click if you like.</li>
</ol>
<p>To put shapes you have already drawn onto the new layer, select them and click <strong>Selection to here</strong>.</p>
<div class="tipp"><p>One layer per work step is handy, for example “outline”, “lettering” and “holes”. With the eye you then switch off parts of the drawing on purpose, for example to preview the lettering only.</p></div>
""",
        "body": """
<ol>
<li>Öffnen Sie die Ebenenliste (Klick auf den Ebenenwähler oben in der Mitte).</li>
<li>Klicken Sie unten auf <strong>+</strong>. Die neue Ebene bekommt einen Namen und eine Farbe aus der Palette und wird sofort aktiv.</li>
<li>Geben Sie ihr bei Bedarf per Doppelklick einen eigenen Namen.</li>
</ol>
<p>Wollen Sie bereits gezeichnete Formen in die neue Ebene legen, wählen Sie sie aus und klicken Sie auf <strong>Auswahl hierher</strong>.</p>
<div class="tipp"><p>Praktisch ist eine Ebene je Arbeitsschritt, etwa „Außenkontur“, „Beschriftung“ und „Bohrungen“. Mit dem Auge schalten Sie dann gezielt Teile der Zeichnung ab, zum Beispiel für eine Vorschau nur der Beschriftung.</p></div>
""",
    },
    {
        "slug": "ebene-loeschen", "cat": C, "title": "Ebene löschen",
        "title_en": "Delete a layer",
        "lead": "Beim Löschen einer Ebene entscheiden Sie, was mit ihren Formen geschieht.",
        "lead_en": "When deleting a layer you decide what happens to its shapes.",
        "body_en": """
<ol>
<li>Make the layer active with a click.</li>
<li>Click <strong>−</strong> in the layer list.</li>
<li>If the layer holds shapes, CncCraft asks (“Layer … holds n shape(s)”): you can move the shapes to another layer or delete them too.</li>
</ol>
<p>The last remaining layer cannot be deleted. Deleting can be undone with <kbd>⌘Z</kbd> (Windows/Linux: <kbd>Ctrl+Z</kbd>).</p>
""",
        "body": """
<ol>
<li>Machen Sie die Ebene durch Klick aktiv.</li>
<li>Klicken Sie in der Ebenenliste auf <strong>−</strong>.</li>
<li>Enthält die Ebene Formen, fragt CncCraft („Ebene … enthält n Form(en)“): Sie können die Formen auf eine andere Ebene verschieben oder mit löschen.</li>
</ol>
<p>Die letzte verbliebene Ebene lässt sich nicht löschen. Das Löschen können Sie mit <kbd>⌘Z</kbd> (Windows/Linux: <kbd>Strg+Z</kbd>) rückgängig machen.</p>
""",
    },
    {
        "slug": "einstellungen-design", "cat": C, "title": "Design: hell, dunkel oder wie das System",
        "title_en": "Theme: light, dark or like the system",
        "lead": "CncCraft kann hell oder dunkel erscheinen oder dem Design Ihres Betriebssystems folgen.",
        "lead_en": "CncCraft can appear light or dark, or follow your operating system's theme.",
        "body_en": """
<p>In <em>View ▸ Theme</em> you choose between <strong>System</strong>, <strong>Light</strong> and <strong>Dark</strong>. Top right in the title bar a button (moon or sun) switches between light and dark. The choice is saved and used again next time.</p>
{{img:dunkel.png|The dark theme}}
<p>The drawing sheet stays light in both themes, so the colours of shapes and toolpaths look the same. Text, input fields and buttons are easy to read in both themes – an automatic test checks the contrast ratio of every surface.</p>
""",
        "body": """
<p>Im Menü <em>Ansicht ▸ Design</em> wählen Sie zwischen <strong>System</strong>, <strong>Hell</strong> und <strong>Dunkel</strong>. Rechts oben in der Titelleiste wechselt ein Knopf (Mond bzw. Sonne) zwischen hell und dunkel. Die Wahl wird gespeichert und beim nächsten Start wieder verwendet.</p>
{{img:dunkel.png|Das dunkle Design}}
<p>Das Zeichenblatt bleibt in beiden Designs hell, damit die Farben der Formen und Werkzeugwege gleich aussehen. Texte, Eingabefelder und Knöpfe sind in beiden Designs gut lesbar – ein automatischer Test prüft das Kontrastverhältnis aller Oberflächen.</p>
""",
    },
    {
        "slug": "tastenkuerzel", "cat": C, "title": "Tastenkürzel",
        "title_en": "Keyboard shortcuts",
        "lead": "Die wichtigsten Befehle haben Tasten. Die Liste „Tastenkürzel“ im Hilfe-Menü zeigt sie im Programm.",
        "lead_en": "The most important commands have keys. The “Keyboard shortcuts” list in the Help menu shows them in the program.",
        "body_en": """
<p>On the Mac <kbd>⌘</kbd> (Command) applies, on Windows and Linux <kbd>Ctrl</kbd>. <em>Help ▸ Keyboard shortcuts</em> (<kbd>⌘/</kbd>) opens the list in the program; the same keys also stand next to the menu entries.</p>
<h2>Tools</h2>
<table>
<tr><th>Key</th><th>Action</th></tr>
<tr><td><kbd>S</kbd></td><td>Select</td></tr>
<tr><td><kbd>L</kbd></td><td>Draw line</td></tr>
<tr><td><kbd>P</kbd></td><td>Draw spline</td></tr>
<tr><td><kbd>R</kbd></td><td>Draw rectangle</td></tr>
<tr><td><kbd>C</kbd></td><td>Draw circle</td></tr>
<tr><td><kbd>O</kbd></td><td>Offset vectors</td></tr>
<tr><td><kbd>T</kbd></td><td>Trim vectors at crossings</td></tr>
<tr><td><kbd>N</kbd></td><td>Edit selected vector in node mode / back</td></tr>
<tr><td><kbd>D</kbd></td><td>Dimension distance</td></tr>
<tr><td><kbd>H</kbd></td><td>Set level/upright</td></tr>
</table>
<h2>While drawing a line</h2>
<table>
<tr><td><kbd>Enter</kbd></td><td>Finish line</td></tr>
<tr><td><kbd>Tab</kbd></td><td>Close line</td></tr>
<tr><td><kbd>Space</kbd></td><td>Finish line and start a new one</td></tr>
<tr><td><kbd>Esc</kbd></td><td>Cancel</td></tr>
</table>
<h2>Edit and file</h2>
<table>
<tr><td><kbd>⌘Z</kbd> / <kbd>⇧⌘Z</kbd></td><td>Undo / redo</td></tr>
<tr><td><kbd>⌘C</kbd>, <kbd>⌘X</kbd>, <kbd>⌘V</kbd></td><td>Copy, cut, paste</td></tr>
<tr><td><kbd>⌘D</kbd></td><td>Duplicate</td></tr>
<tr><td><kbd>⌘G</kbd> / <kbd>⇧⌘G</kbd></td><td>Group / ungroup</td></tr>
<tr><td><kbd>⌘A</kbd></td><td>Select all</td></tr>
<tr><td><kbd>Del</kbd> / <kbd>⌫</kbd></td><td>Delete</td></tr>
<tr><td><kbd>⌘N</kbd>, <kbd>⌘O</kbd></td><td>New job, open</td></tr>
<tr><td><kbd>⌘S</kbd> / <kbd>⇧⌘S</kbd></td><td>Save / save as</td></tr>
</table>
<h2>View</h2>
<table>
<tr><td><kbd>F2</kbd> / <kbd>F3</kbd></td><td>2D view / 3D view</td></tr>
<tr><td><kbd>F4</kbd></td><td>Whole view</td></tr>
</table>
<h2>Mouse and keys together</h2>
<table>
<tr><td><kbd>⇧</kbd> + click</td><td>select and deselect several objects</td></tr>
<tr><td><kbd>⇧</kbd> while dragging</td><td>snapping off</td></tr>
<tr><td><kbd>⌥</kbd> or <kbd>⌃</kbd> while moving</td><td>make a copy</td></tr>
<tr><td>Arrow keys</td><td>move the selection or nodes (<kbd>⇧</kbd>: ten times)</td></tr>
<tr><td><kbd>↑</kbd> <kbd>↓</kbd> in dimension fields</td><td>switch between the fields</td></tr>
<tr><td>Formula and <kbd>=</kbd></td><td>evaluate the value</td></tr>
<tr><td><kbd>Tab</kbd> in dimension fields</td><td>accept the suggested variable</td></tr>
<tr><td>Long-press a value</td><td>type a dimension, radius or construction-line distance</td></tr>
</table>
<div class="neu-box"><p>The single keys for tools (<kbd>S</kbd>, <kbd>L</kbd>, <kbd>P</kbd>, <kbd>R</kbd>, <kbd>C</kbd>, <kbd>O</kbd>, <kbd>T</kbd>, <kbd>D</kbd>, <kbd>H</kbd>) follow programs like Fusion 360, so drawing needs no trip to the toolbar.</p></div>
""",
        "body": """
<p>Auf dem Mac gilt <kbd>⌘</kbd> (Befehl), unter Windows und Linux <kbd>Strg</kbd>. <em>Hilfe ▸ Tastenkürzel</em> (<kbd>⌘/</kbd>) öffnet die Liste im Programm; dieselben Tasten stehen auch neben den Menüeinträgen.</p>
<h2>Werkzeuge</h2>
<table>
<tr><th>Taste</th><th>Wirkung</th></tr>
<tr><td><kbd>S</kbd></td><td>Auswählen</td></tr>
<tr><td><kbd>L</kbd></td><td>Linie zeichnen</td></tr>
<tr><td><kbd>P</kbd></td><td>Spline zeichnen</td></tr>
<tr><td><kbd>R</kbd></td><td>Rechteck zeichnen</td></tr>
<tr><td><kbd>C</kbd></td><td>Kreis zeichnen</td></tr>
<tr><td><kbd>O</kbd></td><td>Vektoren versetzen</td></tr>
<tr><td><kbd>T</kbd></td><td>Vektoren an Kreuzungen trennen</td></tr>
<tr><td><kbd>N</kbd></td><td>Gewählten Vektor im Knotenmodus bearbeiten / zurück</td></tr>
<tr><td><kbd>D</kbd></td><td>Abstand bemaßen</td></tr>
<tr><td><kbd>H</kbd></td><td>Waagrecht/Senkrecht festlegen</td></tr>
</table>
<h2>Beim Zeichnen einer Linie</h2>
<table>
<tr><td><kbd>Enter</kbd></td><td>Linie beenden</td></tr>
<tr><td><kbd>Tab</kbd></td><td>Linie schließen</td></tr>
<tr><td><kbd>Leertaste</kbd></td><td>Linie beenden und eine neue beginnen</td></tr>
<tr><td><kbd>Esc</kbd></td><td>Abbrechen</td></tr>
</table>
<h2>Bearbeiten und Datei</h2>
<table>
<tr><td><kbd>⌘Z</kbd> / <kbd>⇧⌘Z</kbd></td><td>Rückgängig / Wiederholen</td></tr>
<tr><td><kbd>⌘C</kbd>, <kbd>⌘X</kbd>, <kbd>⌘V</kbd></td><td>Kopieren, Ausschneiden, Einfügen</td></tr>
<tr><td><kbd>⌘D</kbd></td><td>Duplizieren</td></tr>
<tr><td><kbd>⌘G</kbd> / <kbd>⇧⌘G</kbd></td><td>Gruppieren / Gruppe auflösen</td></tr>
<tr><td><kbd>⌘A</kbd></td><td>Alles auswählen</td></tr>
<tr><td><kbd>Entf</kbd> / <kbd>⌫</kbd></td><td>Löschen</td></tr>
<tr><td><kbd>⌘N</kbd>, <kbd>⌘O</kbd></td><td>Neuer Job, Öffnen</td></tr>
<tr><td><kbd>⌘S</kbd> / <kbd>⇧⌘S</kbd></td><td>Speichern / Speichern unter</td></tr>
</table>
<h2>Ansicht</h2>
<table>
<tr><td><kbd>F2</kbd> / <kbd>F3</kbd></td><td>2D-Ansicht / 3D-Ansicht</td></tr>
<tr><td><kbd>F4</kbd></td><td>Ganze Ansicht</td></tr>
</table>
<h2>Maus und Tasten zusammen</h2>
<table>
<tr><td><kbd>⇧</kbd> + Klick</td><td>mehrere Objekte aus- und abwählen</td></tr>
<tr><td><kbd>⇧</kbd> beim Ziehen</td><td>Einrasten aus</td></tr>
<tr><td><kbd>⌥</kbd> oder <kbd>⌃</kbd> beim Verschieben</td><td>eine Kopie erstellen</td></tr>
<tr><td>Pfeiltasten</td><td>Auswahl oder Knoten verschieben (<kbd>⇧</kbd>: zehnfach)</td></tr>
<tr><td><kbd>↑</kbd> <kbd>↓</kbd> in Maßfeldern</td><td>zwischen den Feldern wechseln</td></tr>
<tr><td>Formel und <kbd>=</kbd></td><td>Wert ausrechnen</td></tr>
<tr><td><kbd>Tab</kbd> in Maßfeldern</td><td>vorgeschlagene Variable übernehmen</td></tr>
<tr><td>Lange drücken auf einen Wert</td><td>Maß, Radius oder Hilfslinien-Abstand eintippen</td></tr>
</table>
<div class="neu-box"><p>Die Einzeltasten für Werkzeuge (<kbd>S</kbd>, <kbd>L</kbd>, <kbd>P</kbd>, <kbd>R</kbd>, <kbd>C</kbd>, <kbd>O</kbd>, <kbd>T</kbd>, <kbd>D</kbd>, <kbd>H</kbd>) sind an Programme wie Fusion 360 angelehnt, damit das Zeichnen ohne Mausweg zur Werkzeugleiste auskommt.</p></div>
""",
        "neu": True,
    },
    {
        "slug": "dateien-speichern", "cat": C, "title": "Dateien speichern und öffnen", "neu": True,
        "title_en": "Saving and opening files",
        "lead": "Projekte werden als <code>.cnccraft</code>-Dateien gespeichert und enthalten Zeichnung, Ebenen, Variablen, Bedingungen und Werkzeugwege.",
        "lead_en": "Projects are saved as <code>.cnccraft</code> files and hold the drawing, layers, variables, constraints and toolpaths.",
        "body_en": """
<table>
<tr><th>Command</th><th>Effect</th></tr>
<tr><td><em>File ▸ New …</em></td><td>Begins a new job ([[neuer-job]]).</td></tr>
<tr><td><em>File ▸ Open …</em></td><td>Loads a project (<code>.cnccraft</code>).</td></tr>
<tr><td><em>File ▸ Open recent</em></td><td>The projects opened and saved most recently, newest at the top – as in word processors. <em>Clear list</em> at the end tidies them up. Until something has been opened, the entry is grey.</td></tr>
<tr><td><em>File ▸ Open example</em></td><td>Loads the included example.</td></tr>
<tr><td><em>File ▸ Save / Save as …</em></td><td>Writes the project to a file.</td></tr>
<tr><td><em>File ▸ Save toolpaths …</em></td><td>Writes the machine file (G-code), see [[werkzeugwege-speichern]].</td></tr>
</table>
<h2>Unsaved changes</h2>
<div class="neu-box"><p>If the drawing has changes that are not saved yet, a dot (●) appears after the file name at the top right. If you then want to close the program, begin a new job, or open another file or the example, CncCraft first asks “Do you want to save the changes …?”:</p>
{{img:ungespeichert.png|The question before closing with unsaved changes}}
<ul>
<li><strong>Save:</strong> saves (for a new file CncCraft asks for the name) and then continues. Cancel the save and the question stays open.</li>
<li><strong>Don't save:</strong> continues without saving; the changes are lost.</li>
<li><strong>Cancel</strong> (or <kbd>Esc</kbd>): nothing happens, you keep working.</li>
</ul></div>
<p>The [[hilfslinien|construction lines]] are saved with the project too, with their formulas, attached shapes and lock. The project is a text file in JSON format; older files with the earlier extension <code>.funke</code> can still be opened.</p>
""",
        "body": """
<table>
<tr><th>Befehl</th><th>Wirkung</th></tr>
<tr><td><em>Datei ▸ Neu …</em></td><td>Beginnt einen neuen Job ([[neuer-job]]).</td></tr>
<tr><td><em>Datei ▸ Öffnen …</em></td><td>Lädt ein Projekt (<code>.cnccraft</code>).</td></tr>
<tr><td><em>Datei ▸ Zuletzt verwendet öffnen</em></td><td>Die zuletzt geöffneten und gespeicherten Projekte, das neueste oben – wie in Textverarbeitungen. <em>Liste leeren</em> am Ende räumt sie auf. Solange noch nichts geöffnet wurde, ist der Eintrag grau.</td></tr>
<tr><td><em>Datei ▸ Beispiel öffnen</em></td><td>Lädt das mitgelieferte Beispiel.</td></tr>
<tr><td><em>Datei ▸ Speichern / Speichern unter …</em></td><td>Schreibt das Projekt in eine Datei.</td></tr>
<tr><td><em>Datei ▸ Werkzeugwege speichern …</em></td><td>Schreibt die Maschinendatei (G-Code), siehe [[werkzeugwege-speichern]].</td></tr>
</table>
<h2>Ungespeicherte Änderungen</h2>
<div class="neu-box"><p>Hat die Zeichnung Änderungen, die noch nicht gespeichert sind, steht hinter dem Dateinamen oben rechts ein Punkt (●). Wollen Sie dann das Programm schließen, einen neuen Job beginnen, eine andere Datei oder das Beispiel öffnen, fragt CncCraft zuerst „Möchten Sie die Änderungen … speichern?“:</p>
{{img:ungespeichert.png|Die Frage vor dem Schließen mit ungespeicherten Änderungen}}
<ul>
<li><strong>Speichern:</strong> speichert (bei einer neuen Datei fragt CncCraft nach dem Namen) und macht dann weiter. Brechen Sie das Speichern ab, bleibt die Frage offen.</li>
<li><strong>Nicht speichern:</strong> macht ohne Speichern weiter; die Änderungen gehen verloren.</li>
<li><strong>Abbrechen</strong> (oder <kbd>Esc</kbd>): nichts passiert, Sie arbeiten weiter.</li>
</ul></div>
<p>Mit dem Projekt werden auch die [[hilfslinien|Hilfslinien]] gespeichert, samt Formeln, angehängten Formen und Sperre. Das Projekt ist eine Textdatei im JSON-Format; ältere Dateien mit der früheren Endung <code>.funke</code> lassen sich weiterhin öffnen.</p>
""",
    },
    {
        "slug": "rueckgaengig-wiederholen", "cat": C, "title": "Rückgängig und Wiederholen",
        "title_en": "Undo and redo",
        "lead": "Jeder Arbeitsschritt lässt sich zurücknehmen und wiederholen.",
        "lead_en": "Every step can be taken back and redone.",
        "body_en": """
<p><kbd>⌘Z</kbd> takes back the last step, <kbd>⇧⌘Z</kbd> redoes it (also in the <em>Edit</em> menu). Up to 100 steps are remembered.</p>
<p>Not only changes to shapes can be taken back, but every action one by one:</p>
<ul>
<li>drawing, moving, changing and deleting shapes,</li>
<li>[[ebenenliste|layers]]: add, duplicate, rename, reorder, show and hide, lock, change colour and opacity, merge, flatten and delete,</li>
<li>[[werkzeugwegliste|toolpaths]]: reorder and delete,</li>
<li>[[hilfslinien|construction lines]]: add, move, change their distance and delete.</li>
</ul>
<p>A drag with the mouse is always <em>one</em> step – even if a shape or construction line was moved for a long time.</p>
""",
        "body": """
<p><kbd>⌘Z</kbd> nimmt den letzten Schritt zurück, <kbd>⇧⌘Z</kbd> wiederholt ihn (auch im Menü <em>Bearbeiten</em>). Bis zu 100 Schritte werden gemerkt.</p>
<p>Zurücknehmen lassen sich nicht nur Änderungen an Formen, sondern jede Aktion einzeln:</p>
<ul>
<li>Formen zeichnen, verschieben, ändern, löschen,</li>
<li>[[ebenenliste|Ebenen]] anlegen, duplizieren, umbenennen, umsortieren, ein- und ausblenden, sperren, Farbe und Deckkraft ändern, zusammenführen, alle zusammenfassen und löschen,</li>
<li>[[werkzeugwegliste|Werkzeugwege]] umsortieren und löschen,</li>
<li>[[hilfslinien|Hilfslinien]] anlegen, verschieben, ihren Abstand ändern und löschen.</li>
</ul>
<p>Ein Zug mit der Maus ist immer <em>ein</em> Schritt – auch wenn eine Form oder Hilfslinie dabei lange bewegt wurde.</p>
""",
    },
]
