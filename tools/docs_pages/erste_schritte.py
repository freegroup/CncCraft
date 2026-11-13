"""Kategorie „Erste Schritte“."""

C = "erste-schritte"

PAGES = [
    {
        "slug": "einfuehrung", "cat": C, "title": "Einführung",
        "lead": "CncCraft ist ein Programm, mit dem Sie Teile für eine CNC-Fräse entwerfen, Werkzeugwege berechnen und als Datei an die Maschine geben.",
        "body": """
<p>Der Weg vom Einfall zum gefrästen Teil hat bei CncCraft drei Stationen, die sich in einem Fenster abspielen:</p>
<ol>
<li><strong>Entwerfen:</strong> Sie zeichnen Formen oder tippen Maße ein. Ebenen, Einrasten und Bedingungen helfen, genau zu arbeiten (Anfang: [[auswahlwerkzeug|die Entwurfswerkzeuge]]).</li>
<li><strong>Werkzeugwege berechnen:</strong> Sie wählen Formen aus und sagen, was die Fräse damit tun soll – ausschneiden, eine Tasche ausräumen, bohren oder eine Schrift mit V-Fräser schnitzen.</li>
<li><strong>Ausgeben:</strong> Nach einer Vorschau in 2D oder 3D speichern Sie die Werkzeugwege als G-Code.</li>
</ol>
{{img:oberflaeche.png|Das Hauptfenster: links die Werkzeuge, in der Mitte die Zeichnung, rechts die Werkzeugwege}}
<h2>Das Fenster im Überblick</h2>
<table>
<tr><th>Bereich</th><th>Was Sie dort finden</th></tr>
<tr><td>Menüleiste (oben links)</td><td>Datei, Bearbeiten, Ansicht, Werkzeugwege, Hilfe.</td></tr>
<tr><td>Ebenenwahl (oben Mitte)</td><td>Die aktive Ebene und die Liste aller Ebenen. Siehe [[ebenenliste]].</td></tr>
<tr><td>Werkzeugleiste (links)</td><td>Zeichen- und Bearbeitungswerkzeuge, darunter Job-Einrichtung und Variablen.</td></tr>
<tr><td>Kopfleiste über der Zeichnung</td><td>Dekoration (Raster, Einrasten, Maße) und Constraints; rechts die Ansichtsknöpfe.</td></tr>
<tr><td>Zeichenfläche (Mitte)</td><td>Das Werkstück mit Linealen und Ihren Formen – oder die 3D-Vorschau.</td></tr>
<tr><td>Werkzeugpfade (rechts)</td><td>Platten-Setup, die Knöpfe für neue Werkzeugwege, die Liste, Vorschau und Ausgabe.</td></tr>
<tr><td>Statuszeile (unten)</td><td>Größe der Auswahl, Zeigerposition und die Ebene der Auswahl.</td></tr>
</table>
<p>Mehr dazu auf der Seite [[entwurfsansicht]]. Wer gleich loslegen möchte, öffnet auf der [[startseite]] das Beispiel „Schild mit Schrift, Tasche und Bohrungen“ und sieht, wie ein fertiger Job aussieht.</p>
<div class="hinweis"><p>CncCraft ist ein eigenständiger Nachbau, der sich an Spark von Vectric orientiert und einiges erweitert. Dokumentiert ist hier nur, was CncCraft kann.</p></div>
""",
    },
    {
        "slug": "installation", "cat": C, "title": "Installation",
        "lead": "CncCraft wird aus dem Quelltext gebaut; das Skript <code>build.sh</code> erzeugt daraus die Pakete für macOS, Windows und Linux.",
        "body": """
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
<p>Wer keine eigenen Windows- oder Linux-Rechner hat, kann alle Pakete bei GitHub bauen lassen: Die mitgelieferte Arbeitsanweisung <code>.github/workflows/release.yml</code> startet, sobald ein Versions-Tag wie <code>v0.1.0</code> hochgeladen wird, und hängt die Pakete an die Veröffentlichung.</p>
<div class="hinweis"><p>Die Mac-App ist nur „ad hoc“ signiert. Auf fremden Macs erscheint daher beim ersten Start eine Sicherheitswarnung; ohne Apple-Entwicklerkonto lässt sie sich nicht vermeiden. Die Wege für Windows und Linux sind beschrieben, wurden aber bisher nicht auf allen Systemen ausprobiert.</p></div>
<h2>Prüfen, ob es läuft</h2>
<p>Mit <code>cnccraft --selftest</code> führt das Programm einen Selbsttest aus und meldet jeden Schritt. Mit <code>cnccraft --screenshot bild.png</code> speichert es ein Bild des Fensters, ohne dass Sie etwas bedienen müssen – so sind die Abbildungen dieser Hilfe entstanden.</p>
""",
    },
    {
        "slug": "startseite", "cat": C, "title": "Startseite",
        "lead": "Beim Start zeigt CncCraft drei Kacheln: neue Datei, Datei öffnen und das Beispiel.",
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
        "lead": "Der Job beschreibt das Werkstück: Größe, Material und Nullpunkt der Fräse.",
        "body": """
<p>Der Dialog heißt „Job erstellen“, wenn Sie eine neue Datei beginnen, und „Job einrichten“, wenn Sie ihn später über <em>Bearbeiten ▸ Job einrichten …</em>, den Knopf in der Werkzeugleiste oder die Zeile „Platten-Setup“ rechts öffnen.</p>
{{img:job-dialog.png|Der Dialog „Job erstellen“}}
<h2>Auftragsgröße</h2>
<ul>
<li><strong>Breite (X), Länge (Y), Dicke (Z):</strong> die Maße des Rohlings. Die kleine Skizze zeigt, welche Seite welcher Achse entspricht.</li>
<li><strong>Einheiten:</strong> Zoll oder Millimeter. Beim Umschalten werden die Zahlen umgerechnet; gespeichert wird intern in Millimetern.</li>
<li><strong>Material:</strong> Weichholz, Hartholz, MDF oder Acryl. Das Material bestimmt, welche Werkzeuge mit passenden Vorschub- und Drehzahlwerten zur Auswahl stehen (siehe [[werkzeugauswahl]]).</li>
</ul>
<h2>Nullpunkt</h2>
<ul>
<li><strong>Z-Null:</strong> <em>Oberfläche</em> – die Oberseite des Materials ist Höhe 0, Schnitttiefen zählen nach unten. <em>Tisch</em> – die Unterseite ist Höhe 0.</li>
<li><strong>XY-Null:</strong> <em>Ecke</em> (unten links) oder <em>Mitte</em> des Werkstücks. Dort sitzt später auch der Nullpunkt in der G-Code-Datei.</li>
<li><strong>Sicher Z</strong> und <strong>Start Z:</strong> die Höhen, auf die die Fräse zwischen den Schnitten fährt, und die Starthöhe der Maschine.</li>
</ul>
<div class="tipp"><p>Größe und Dicke stehen danach als Variablen <code>platte_b</code>, <code>platte_l</code> und <code>platte_d</code> zur Verfügung. Damit lässt sich zum Beispiel eine Schnitttiefe als <code>platte_d/2</code> eintragen – siehe [[variablen]].</p></div>
<p>Wenn Sie einen bestehenden Job ändern, berechnet CncCraft alle Werkzeugwege mit den neuen Werten neu.</p>
""",
    },
    {
        "slug": "entwurfsansicht", "cat": C, "title": "Die Entwurfsansicht",
        "lead": "In der 2D-Ansicht zeichnen und bearbeiten Sie; Lineale, Raster und Einrasthilfen unterstützen Sie dabei.",
        "body": """
<p>Die Zeichenfläche zeigt das Werkstück von oben: die Platte mit Linealen am oberen und linken Rand, ein Raster und das Koordinatenkreuz am Nullpunkt. Der Wert im Lineal ist in der Einheit des Jobs.</p>
<h2>Sich bewegen</h2>
<ul>
<li><strong>Zoomen:</strong> mit dem Mausrad oder einer Zwei-Finger-Geste (zum Zeiger hin).</li>
<li><strong>Verschieben:</strong> mit der mittleren oder rechten Maustaste ziehen.</li>
<li><strong>Ganze Ansicht</strong> (<kbd>F4</kbd>): zeigt das ganze Werkstück. <strong>Auf die Auswahl zoomen:</strong> zeigt die gewählten Formen.</li>
</ul>
<h2>Die Kopfleiste</h2>
<p>Über der Zeichnung liegen zwei Gruppen von Schaltern:</p>
<ul>
<li><strong>Dekoration:</strong> am Raster einrasten, an Geometrie einrasten (siehe [[einrasten]]) und die Maße anzeigen oder ausblenden.</li>
<li><strong>Constraints:</strong> <em>Abstand</em> (<kbd>D</kbd>) und <em>Waagrecht/Senkrecht</em> (<kbd>H</kbd>) – siehe [[abstand-bemassen]] und [[waagrecht-senkrecht]].</li>
</ul>
<p>Rechts oben wechselt der Knopf <strong>2D/3D</strong> zwischen Entwurf und [[ansicht-3d]]; daneben schaltet ein Knopf den Nullpunkt (Achsenkreuz) ein oder aus.</p>
<h2>Die Werkzeugleiste</h2>
<p>Links liegen die Werkzeuge in Gruppen: Modus (Auswählen, Transformieren, Knoten bearbeiten), Linien, Formen, Text, Objekte bearbeiten, Ausrichten und Spiegeln, Versatz, Boolesche Operationen sowie – abgesetzt – die Job-Einrichtung und die Variablen. Mit dem kleinen Pfeil oben links lässt sich die Leiste verkleinern; dann zeigt jede Gruppe nur das zuletzt benutzte Werkzeug.</p>
{{img:werkzeugleiste-eingeklappt.png|Die verkleinerte Werkzeugleiste}}
<p>Fahren Sie mit der Maus über ein Werkzeug, erscheint ein kurzer Hilfetext. Die Statuszeile unten nennt außerdem, was als Nächstes zu tun ist (zum Beispiel „Zweiten Punkt wählen“).</p>
<h2>Dekorationen der Arbeitsfläche</h2>
<p>Was die Zeichenfläche zusätzlich zeigt, schalten Sie einzeln im Menü <em>Ansicht ▸ Arbeitsfläche</em>: Platte, Raster, Lineale, Koordinatenkreuz und Hinweise.</p>
""",
    },
    {
        "slug": "ansicht-3d", "cat": C, "title": "Die 3D-Ansicht",
        "lead": "Die 3D-Ansicht zeigt das Werkstück, so wie es nach dem Fräsen aussehen wird.",
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
<p>Die Werkzeugwege selbst legen Sie über die Option „Werkzeugpfade im 3D einblenden“ als Linien über das Werkstück; mehr dazu unter [[vorschau-werkzeugwege]].</p>
""",
        "neu": True,
    },
    {
        "slug": "ebenenliste", "cat": C, "title": "Ebenen",
        "lead": "Ebenen gruppieren Formen: Sie lassen sich ein- und ausblenden, sperren und färben.",
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
        "lead": "Eine neue Ebene entsteht mit einem Klick auf das Plus in der Ebenenliste.",
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
        "lead": "Beim Löschen einer Ebene entscheiden Sie, was mit ihren Formen geschieht.",
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
        "lead": "CncCraft kann hell oder dunkel erscheinen oder dem Design Ihres Betriebssystems folgen.",
        "body": """
<p>Im Menü <em>Ansicht ▸ Design</em> wählen Sie zwischen <strong>System</strong>, <strong>Hell</strong> und <strong>Dunkel</strong>. Rechts oben in der Titelleiste wechselt ein Knopf (Mond bzw. Sonne) zwischen hell und dunkel. Die Wahl wird gespeichert und beim nächsten Start wieder verwendet.</p>
{{img:dunkel.png|Das dunkle Design}}
<p>Das Zeichenblatt bleibt in beiden Designs hell, damit die Farben der Formen und Werkzeugwege gleich aussehen. Texte, Eingabefelder und Knöpfe sind in beiden Designs gut lesbar – ein automatischer Test prüft das Kontrastverhältnis aller Oberflächen.</p>
""",
    },
    {
        "slug": "tastenkuerzel", "cat": C, "title": "Tastenkürzel",
        "lead": "Die wichtigsten Befehle haben Tasten. Die Liste „Tastenkürzel“ im Hilfe-Menü zeigt sie im Programm.",
        "body": """
<p>Auf dem Mac gilt <kbd>⌘</kbd> (Befehl), unter Windows und Linux <kbd>Strg</kbd>. <em>Hilfe ▸ Tastenkürzel</em> (<kbd>⌘/</kbd>) öffnet die Liste im Programm; dieselben Tasten stehen auch neben den Menüeinträgen.</p>
<h2>Werkzeuge</h2>
<table>
<tr><th>Taste</th><th>Wirkung</th></tr>
<tr><td><kbd>S</kbd></td><td>Auswählen</td></tr>
<tr><td><kbd>L</kbd></td><td>Linie zeichnen</td></tr>
<tr><td><kbd>R</kbd></td><td>Rechteck zeichnen</td></tr>
<tr><td><kbd>C</kbd></td><td>Kreis zeichnen</td></tr>
<tr><td><kbd>O</kbd></td><td>Vektoren versetzen</td></tr>
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
</table>
<div class="neu-box"><p>Die Einzeltasten für Werkzeuge (<kbd>S</kbd>, <kbd>L</kbd>, <kbd>R</kbd>, <kbd>C</kbd>, <kbd>O</kbd>, <kbd>D</kbd>, <kbd>H</kbd>) sind an Programme wie Fusion 360 angelehnt, damit das Zeichnen ohne Mausweg zur Werkzeugleiste auskommt.</p></div>
""",
        "neu": True,
    },
    {
        "slug": "dateien-speichern", "cat": C, "title": "Dateien speichern und öffnen", "neu": True,
        "lead": "Projekte werden als <code>.cnccraft</code>-Dateien gespeichert und enthalten Zeichnung, Ebenen, Variablen, Bedingungen und Werkzeugwege.",
        "body": """
<table>
<tr><th>Befehl</th><th>Wirkung</th></tr>
<tr><td><em>Datei ▸ Neu …</em></td><td>Beginnt einen neuen Job ([[neuer-job]]).</td></tr>
<tr><td><em>Datei ▸ Öffnen …</em></td><td>Lädt ein Projekt (<code>.cnccraft</code>).</td></tr>
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
<p>Das Projekt ist eine Textdatei im JSON-Format; ältere Dateien mit der früheren Endung <code>.funke</code> lassen sich weiterhin öffnen.</p>
""",
    },
]
