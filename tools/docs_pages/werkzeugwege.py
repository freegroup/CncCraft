"""Kategorie „Werkzeugwege“."""

C = "werkzeugwege"

PAGES = [
    {
        "slug": "werkzeugwege-uebersicht", "cat": C, "title": "Werkzeugwege – Überblick",
        "title_en": "Toolpaths – overview",
        "lead": "Ein Werkzeugweg sagt der Fräse, welche Formen sie wie bearbeiten soll. CncCraft kennt sechs Arten.",
        "lead_en": "A toolpath tells the router which shapes to machine and how. CncCraft has six kinds.",
        "body_en": """
<p>On the right of the window, under “Board setup”, are six buttons for new toolpaths. The menu items under <em>Toolpaths</em> do the same.</p>
<table>
<tr><th>Toolpath</th><th>What for</th></tr>
<tr><td>[[profil-werkzeugweg|Profile]]</td><td>Runs along a contour – outside, inside or right on the line.</td></tr>
<tr><td>[[ausschnitt-werkzeugweg|Cut-out]]</td><td>Cuts a part out of the material, with tabs.</td></tr>
<tr><td>[[taschen-werkzeugweg|Pocket]]</td><td>Clears the area inside a contour down to a depth.</td></tr>
<tr><td>[[nut-werkzeugweg|Slot]]</td><td>Mills a slot the width of the cutter along a line – exactly as long as the line.</td></tr>
<tr><td>[[bohr-werkzeugweg|Drilling]]</td><td>Drills into the centre of circles and other contours.</td></tr>
<tr><td>[[vcarving-werkzeugweg|V-carving]]</td><td>Carves lettering and drawings with a V-bit; the depth follows the width.</td></tr>
</table>
<h2>How a toolpath is made</h2>
<ol>
<li>Select the vectors to be machined.</li>
<li>Click the button of the kind you want on the right. A form opens.</li>
<li>Pick the tool and enter the depths ([[werkzeugauswahl]]). If a value exceeds what the [[materialien|material]] allows, a warning appears under the field.</li>
<li>Click <strong>Apply</strong>. The toolpath appears in the list and as coloured lines in the drawing.</li>
</ol>
{{img:tiefe.png|The form for a new toolpath}}
<p>Afterwards you manage the toolpaths in the [[werkzeugwegliste|list]], check them in the [[vorschau-werkzeugwege|preview]] and output them with [[werkzeugwege-speichern|Save]].</p>
<p>Hover over a strategy or cut direction and the tooltip shows a small drawing: where the cutter runs, which way it turns and what is cleared.</p>
<div class="hinweis"><p>Toolpaths remember which shapes they belong to. Change a shape or a variable ([[variablen]]) and CncCraft recomputes the toolpaths.</p></div>
""",
        "body": """
<p>Rechts im Fenster, unter „Platten-Setup“, liegen sechs Knöpfe für neue Werkzeugwege. Die Menüpunkte unter <em>Werkzeugwege</em> tun dasselbe.</p>
<table>
<tr><th>Werkzeugweg</th><th>Wofür</th></tr>
<tr><td>[[profil-werkzeugweg|Profil]]</td><td>Fährt an einer Kontur entlang – außen, innen oder genau auf der Linie.</td></tr>
<tr><td>[[ausschnitt-werkzeugweg|Ausschnitt]]</td><td>Trennt ein Teil aus dem Material heraus, mit Haltestegen.</td></tr>
<tr><td>[[taschen-werkzeugweg|Tasche]]</td><td>Räumt die Fläche innerhalb einer Kontur bis zu einer Tiefe aus.</td></tr>
<tr><td>[[nut-werkzeugweg|Nut]]</td><td>Fräst eine Nut mit der Breite des Fräsers entlang einer Linie – genau so lang wie die Linie.</td></tr>
<tr><td>[[bohr-werkzeugweg|Bohren]]</td><td>Bohrt in die Mitte von Kreisen und anderen Konturen.</td></tr>
<tr><td>[[vcarving-werkzeugweg|V-Carving]]</td><td>Schnitzt Schrift und Zeichnungen mit einem V-Fräser; die Tiefe folgt der Breite.</td></tr>
</table>
<h2>So entsteht ein Werkzeugweg</h2>
<ol>
<li>Wählen Sie die Vektoren aus, die bearbeitet werden sollen.</li>
<li>Klicken Sie rechts auf den Knopf der gewünschten Art. Ein Formular öffnet sich.</li>
<li>Wählen Sie das Werkzeug und tragen Sie die Tiefen ein ([[werkzeugauswahl]]). Geht ein Wert über das, was das [[materialien|Material]] verträgt, steht eine Warnung unter dem Feld.</li>
<li>Klicken Sie auf <strong>Anwenden</strong>. Der Werkzeugweg erscheint in der Liste und als farbige Linien in der Zeichnung.</li>
</ol>
{{img:tiefe.png|Das Formular für einen neuen Werkzeugweg}}
<p>Danach verwalten Sie die Werkzeugwege in der [[werkzeugwegliste|Liste]], prüfen sie in der [[vorschau-werkzeugwege|Vorschau]] und geben sie mit [[werkzeugwege-speichern|Speichern]] aus.</p>
<p>Fährt die Maus über eine Strategie oder Laufrichtung, zeigt der Hilfetext eine kleine Zeichnung: wo der Fräser fährt, in welche Richtung er dreht und was ausgeräumt wird.</p>
<div class="hinweis"><p>Werkzeugwege merken sich, zu welchen Formen sie gehören. Ändern Sie eine Form oder eine Variable ([[variablen]]), berechnet CncCraft die Werkzeugwege neu.</p></div>
""",
    },
    {
        "slug": "werkzeugwegliste", "cat": C, "title": "Die Werkzeugwegliste", "neu": True,
        "title_en": "The toolpath list",
        "lead": "Die Liste zeigt alle Werkzeugwege mit Farbe, Art, Name und geschätzter Fräszeit.",
        "lead_en": "The list shows all toolpaths with colour, kind, name and estimated milling time.",
        "body_en": """
<p>Below the buttons for new toolpaths is the list.</p>
{{img:werkzeugliste.png|The toolpath list with the tool entries above the toolpaths}}
<table>
<tr><th>Element</th><th>Meaning</th></tr>
<tr><td>Handle on the left (⠿)</td><td>Dragging changes the order – that is how the file is written later too.</td></tr>
<tr><td>Eye</td><td>Shows or hides the toolpath in the drawing, preview and output.</td></tr>
<tr><td>Colour square and symbol</td><td>The kind of toolpath and its colour in the drawing – the colour of its shapes' layer. Give the layer another colour and the entry and the cutter paths follow.</td></tr>
<tr><td>Name</td><td>A click selects the toolpath: its shapes are selected in the drawing, and it becomes the “selected” one for preview and export. Double-click opens the form to change it.</td></tr>
<tr><td>Time</td><td>The estimated milling time in minutes and seconds.</td></tr>
</table>
<ul>
<li><strong>Hide all / Show all</strong> switches all toolpaths at once.</li>
<li><strong>Tool entries:</strong> above the first toolpath and wherever a different tool begins, a small entry “Tool: …” appears – from there this cutter mills until a new entry comes. If two toolpaths in a row use the same tool, the entry appears only above the first.</li>
<li>The <strong>↻</strong> button recomputes all toolpaths.</li>
<li>Hover over a row and small buttons appear on the right: <strong>✎</strong> edit, <strong>🗑</strong> delete and <strong>⏶</strong> and <strong>⏷</strong> to move up and down.</li>
<li>Below the list is the <strong>total milling time of the visible</strong> toolpaths.</li>
<li>Hover over an entry and a tooltip names the kind, start and cut depth and the time.</li>
</ul>
<p>If a toolpath cannot be computed – for example because the tool is too large for the shape – the list shows a warning triangle with the reason.</p>
""",
        "body": """
<p>Unter den Knöpfen für neue Werkzeugwege liegt die Liste.</p>
{{img:werkzeugliste.png|Die Werkzeugwegliste mit den Werkzeug-Einträgen über den Werkzeugwegen}}
<table>
<tr><th>Element</th><th>Bedeutung</th></tr>
<tr><td>Griff links (⠿)</td><td>Ziehen ändert die Reihenfolge – so wird auch die Datei später geschrieben.</td></tr>
<tr><td>Auge</td><td>Blendet den Werkzeugweg in Zeichnung, Vorschau und Ausgabe ein oder aus.</td></tr>
<tr><td>Farbquadrat und Symbol</td><td>Die Art des Werkzeugwegs und seine Farbe in der Zeichnung – die Farbe der Ebene seiner Formen. Bekommt die Ebene eine andere Farbe, folgen Eintrag und Fräsbahnen.</td></tr>
<tr><td>Name</td><td>Ein Klick wählt den Werkzeugweg aus: Seine Formen werden in der Zeichnung gewählt, und er ist der „ausgewählte“ für Vorschau und Export. Doppelklick öffnet das Formular zum Ändern.</td></tr>
<tr><td>Zeit</td><td>Die geschätzte Fräszeit in Minuten und Sekunden.</td></tr>
</table>
<ul>
<li><strong>Alle ausblenden / Alle einblenden</strong> schaltet alle Werkzeugwege zugleich.</li>
<li><strong>Werkzeug-Einträge:</strong> Über dem ersten Werkzeugweg und überall, wo ein anderes Werkzeug beginnt, steht ein kleiner Eintrag „Werkzeug: …“ – ab dort fräst dieser Fräser, bis ein neuer Eintrag kommt. Haben zwei Werkzeugwege hintereinander dasselbe Werkzeug, steht der Eintrag nur über dem ersten.</li>
<li>Der Knopf <strong>↻</strong> berechnet alle Werkzeugwege neu.</li>
<li>Fährt die Maus über eine Zeile, erscheinen rechts kleine Knöpfe: <strong>✎</strong> bearbeiten, <strong>🗑</strong> löschen sowie <strong>⏶</strong> und <strong>⏷</strong> zum Verschieben nach oben und unten.</li>
<li>Unter der Liste steht die <strong>Gesamtfräszeit der sichtbaren</strong> Werkzeugwege.</li>
<li>Fährt die Maus über einen Eintrag, nennt ein Hilfetext Art, Start- und Schnitttiefe sowie die Zeit.</li>
</ul>
<p>Kann ein Werkzeugweg nicht berechnet werden – zum Beispiel weil das Werkzeug für die Form zu groß ist –, zeigt die Liste ein Warndreieck mit dem Grund.</p>
""",
    },
    {
        "slug": "werkzeugauswahl", "cat": C, "title": "Werkzeuge und Schnittwerte",
        "title_en": "Tools and cutting values",
        "lead": "Jeder Werkzeugweg bekommt ein Werkzeug mit passenden Werten für Vorschub, Drehzahl und Zustellung.",
        "lead_en": "Every toolpath gets a tool with matching values for feed, speed and stepdown.",
        "body_en": """
<p>In a toolpath's form you pick from a built-in list in the <strong>Tool</strong> field. It holds, per [[materialien|material]], tools in several diameters with values that suit that material:</p>
<ul>
<li><strong>End mills</strong> for profiles, cut-outs and pockets,</li>
<li><strong>Ball-nose cutters</strong>,</li>
<li><strong>V-bits</strong> with a tip angle for V-carving,</li>
<li><strong>Drills</strong> for holes.</li>
</ul>
<p>Which tools appear depends on the job's material ([[neuer-job]]). When a tool is chosen, the form takes its values; you can change each one.</p>
<table>
<tr><th>Field</th><th>Meaning</th></tr>
<tr><td>Start depth</td><td>Where the cut begins, from the material surface (0 = surface).</td></tr>
<tr><td>Cut depth</td><td>How deep in total. For a cut-out the default is the material thickness.</td></tr>
<tr><td>Stepdown</td><td>How much deeper the cutter goes per pass. Depth and stepdown give the number of passes.</td></tr>
<tr><td>Stepover</td><td>The sideways distance between two passes for pockets and V-carving, in millimetres and as a percentage of the cutter diameter.</td></tr>
<tr><td>Spindle speed</td><td>Spindle revolutions per minute (rpm).</td></tr>
<tr><td>Feed rate</td><td>Cutting speed in mm/min.</td></tr>
<tr><td>Plunge</td><td>Speed when plunging into the material in mm/min.</td></tr>
<tr><td>Tool no.</td><td>The number that stands in the G-code file for the tool change.</td></tr>
</table>
<div class="neu-box"><p>Start and cut depth take formulas and variables, for example <code>platte_d/2</code> or <code>platte_d</code> ([[formeln-in-feldern]]).</p></div>
<h2>Warnings</h2>
<p>Every material has limits: the highest feed, the speed range and how deep the cutter may go per pass (for example two thirds of the diameter in hardwood). If a field exceeds a limit, a warning below it states what the material allows. Save anyway and CncCraft also writes the warning into the header of the G-code file.</p>
<div class="achtung"><p>The shipped cutting values are starting points. Always check feed, speed and stepdown for your router and material, and mill a test in scrap material first.</p></div>
""",
        "body": """
<p>Im Formular eines Werkzeugwegs wählen Sie im Feld <strong>Werkzeug</strong> aus einer eingebauten Liste. Sie enthält je [[materialien|Material]] Werkzeuge in mehreren Durchmessern, mit Werten, die zu diesem Material passen:</p>
<ul>
<li><strong>Schaftfräser</strong> für Profile, Ausschnitte und Taschen,</li>
<li><strong>Kugelfräser</strong>,</li>
<li><strong>V-Fräser</strong> mit Spitzenwinkel für das V-Carving,</li>
<li><strong>Bohrer</strong> für Bohrungen.</li>
</ul>
<p>Welche Werkzeuge erscheinen, hängt vom Material des Jobs ab ([[neuer-job]]). Wird ein Werkzeug gewählt, übernimmt das Formular dessen Werte; Sie können jeden einzeln ändern.</p>
<table>
<tr><th>Feld</th><th>Bedeutung</th></tr>
<tr><td>Starttiefe</td><td>Wo der Schnitt beginnt, von der Materialoberfläche aus (0 = Oberfläche).</td></tr>
<tr><td>Schnitttiefe</td><td>Wie tief insgesamt gefräst wird. Beim Ausschnitt ist die Voreinstellung die Dicke des Materials.</td></tr>
<tr><td>Zustellung</td><td>Wie viel die Fräse je Durchgang tiefer geht. Aus Tiefe und Zustellung ergibt sich die Zahl der Durchgänge.</td></tr>
<tr><td>Querzustellung</td><td>Der seitliche Abstand zwischen zwei Bahnen bei Taschen und V-Carving, in Millimetern und in Prozent des Fräserdurchmessers.</td></tr>
<tr><td>Drehzahl</td><td>Umdrehungen der Spindel pro Minute (U/min).</td></tr>
<tr><td>Vorschub</td><td>Geschwindigkeit beim Schneiden in mm/min.</td></tr>
<tr><td>Eintauchen</td><td>Geschwindigkeit beim Eintauchen in das Material in mm/min.</td></tr>
<tr><td>Werkzeug-Nr.</td><td>Die Nummer, die in der G-Code-Datei für den Werkzeugwechsel steht.</td></tr>
</table>
<div class="neu-box"><p>Start- und Schnitttiefe nehmen Formeln und Variablen an, zum Beispiel <code>platte_d/2</code> oder <code>platte_d</code> ([[formeln-in-feldern]]).</p></div>
<h2>Warnungen</h2>
<p>Jedes Material hat Grenzen: höchster Vorschub, Drehzahlbereich und wie tief der Fräser je Durchgang gehen darf (zum Beispiel zwei Drittel des Durchmessers bei Hartholz). Überschreitet ein Feld eine Grenze, steht darunter eine Warnung mit dem, was das Material verträgt. Speichern Sie trotzdem, schreibt CncCraft die Warnung auch in den Kopf der G-Code-Datei.</p>
<div class="achtung"><p>Die mitgelieferten Schnittwerte sind Ausgangswerte. Prüfen Sie Vorschub, Drehzahl und Zustellung immer für Ihre Fräse und Ihr Material und fräsen Sie zuerst eine Probe in Abfallmaterial.</p></div>
""",
    },
    {
        "slug": "profil-werkzeugweg", "cat": C, "title": "Profil-Werkzeugweg",
        "title_en": "Profile toolpath",
        "lead": "Das Profil fährt mit dem Fräser an der Kontur entlang – außen, innen oder auf der Linie.",
        "lead_en": "The profile runs the cutter along the contour – outside, inside or on the line.",
        "body_en": """
<p>Select the vectors and click the <strong>Profile toolpath</strong> button (<em>Toolpaths ▸ Profile</em>). In addition to the general fields ([[werkzeugauswahl]]) you set:</p>
<h2>Cut</h2>
<ul>
<li><strong>Outside:</strong> the cutter runs outside the contour, offset by the cutter radius – for the part that stays.</li>
<li><strong>Inside:</strong> the cutter runs inside – for a hole or recess.</li>
<li><strong>On the line:</strong> the centre of the cutter runs exactly on the contour.</li>
</ul>
<h2>Cut direction</h2>
<ul>
<li><strong>Climb:</strong> the cutter turns with the feed direction – usually the cleaner result.</li>
<li><strong>Conventional:</strong> against the feed direction.</li>
</ul>
<h2>Tabs</h2>
<p>With <strong>Tabs</strong>, a few bridges are left along the contour to hold the part in the board until everything is milled. You set the number next to it (default: 4). The tabs form only in the lowest passes.</p>
<p>Open lines are run back and forth, so the cutter does not fly back after every pass.</p>
<div class="hinweis"><p>If the cutter is too large for the shape, the list reports “Tool too large for this shape”.</p></div>
""",
        "body": """
<p>Wählen Sie die Vektoren und klicken Sie auf den Knopf <strong>Profil-Werkzeugweg</strong> (<em>Werkzeugwege ▸ Profil</em>). Zusätzlich zu den allgemeinen Feldern ([[werkzeugauswahl]]) stellen Sie ein:</p>
<h2>Fräsen</h2>
<ul>
<li><strong>Außen:</strong> der Fräser läuft außerhalb der Kontur, um den Fräserradius versetzt – für das Teil, das stehen bleibt.</li>
<li><strong>Innen:</strong> der Fräser läuft innerhalb – für ein Loch oder eine Aussparung.</li>
<li><strong>Auf der Linie:</strong> der Mittelpunkt des Fräsers läuft genau auf der Kontur.</li>
</ul>
<h2>Laufrichtung</h2>
<ul>
<li><strong>Gleichlauf:</strong> Fräser dreht mit der Vorschubrichtung – meist das sauberere Ergebnis.</li>
<li><strong>Gegenlauf:</strong> entgegen der Vorschubrichtung.</li>
</ul>
<h2>Haltestege</h2>
<p>Mit <strong>Haltestege</strong> bleiben an der Kontur einige Stege stehen, die das Teil in der Platte halten, bis alles gefräst ist. Die Zahl stellen Sie daneben ein (Voreinstellung: 4). Die Stege entstehen nur in den untersten Durchgängen.</p>
<p>Offene Linien werden hin und her gefahren, sodass der Fräser nicht nach jedem Durchgang zurückfliegt.</p>
<div class="hinweis"><p>Ist der Fräser zu groß für die Form, meldet die Liste „Werkzeug zu groß für diese Form“.</p></div>
""",
    },
    {
        "slug": "ausschnitt-werkzeugweg", "cat": C, "title": "Ausschnitt-Werkzeugweg",
        "title_en": "Cut-out toolpath",
        "lead": "Der Ausschnitt ist ein Profil, das ein Teil ganz herausschneidet und es mit Haltestegen festhält.",
        "lead_en": "The cut-out is a profile that cuts a part all the way out and holds it with tabs.",
        "body_en": """
<p>The cut-out (<em>Toolpaths ▸ Cut-out</em>, also “cut part free”) is a profile toolpath with preset values for the most common case: freeing a part from the board.</p>
<table>
<tr><th>Setting</th><th>Default for the cut-out</th></tr>
<tr><td>Cut</td><td>Outside</td></tr>
<tr><td>Tabs</td><td>on, 4</td></tr>
<tr><td>Cut depth</td><td>the material thickness</td></tr>
<tr><td>Tool</td><td>End mill, 6 mm</td></tr>
</table>
<p>Every field can be changed in the form – they are the same as for the [[profil-werkzeugweg|profile]]. After milling you cut the tabs with a knife or a saw and sand the spots.</p>
<div class="tipp"><p>Enter <code>platte_d</code> as the cut depth and the cut-out always goes through the whole material, even if you change the thickness later.</p></div>
""",
        "body": """
<p>Der Ausschnitt (<em>Werkzeugwege ▸ Ausschnitt</em>, auch „Teil heraustrennen“) ist ein Profil-Werkzeugweg mit vorbereiteten Einstellungen für den häufigsten Fall: ein Teil aus der Platte zu lösen.</p>
<table>
<tr><th>Einstellung</th><th>Voreinstellung beim Ausschnitt</th></tr>
<tr><td>Fräsen</td><td>Außen</td></tr>
<tr><td>Haltestege</td><td>an, 4 Stück</td></tr>
<tr><td>Schnitttiefe</td><td>die Dicke des Materials</td></tr>
<tr><td>Werkzeug</td><td>Schaftfräser, 6 mm</td></tr>
</table>
<p>Alle Felder lassen sich im Formular ändern – es sind dieselben wie beim [[profil-werkzeugweg|Profil]]. Nach dem Fräsen trennen Sie die Haltestege mit einem Messer oder einer Säge und schleifen die Stellen nach.</p>
<div class="tipp"><p>Tragen Sie als Schnitttiefe <code>platte_d</code> ein, schneidet der Ausschnitt immer durch das ganze Material, auch wenn Sie die Dicke später ändern.</p></div>
""",
    },
    {
        "slug": "taschen-werkzeugweg", "cat": C, "title": "Taschen-Werkzeugweg",
        "title_en": "Pocket toolpath",
        "lead": "Die Tasche räumt eine Fläche innerhalb einer geschlossenen Kontur bis zu einer Tiefe aus.",
        "lead_en": "The pocket clears an area inside a closed contour down to a depth.",
        "body_en": """
<p>Select closed vectors and click <strong>Pocket toolpath</strong>. Besides the general fields ([[werkzeugauswahl]]) you choose the <strong>clearing strategy</strong>:</p>
<table>
<tr><th>Strategy</th><th>How the cutter clears</th></tr>
<tr><td><strong>Offset</strong></td><td>In rings that follow the contour, from the inside out. Between neighbouring rings the cutter stays down.</td></tr>
<tr><td><strong>Raster</strong></td><td>Back and forth in straight parallel passes, then one lap along the wall. The <strong>raster angle</strong> (in degrees) sets the direction.</td></tr>
<tr><td><strong>Spiral</strong></td><td>One continuous spiral from the inside out: no fresh start, each lap flows smoothly into the next.</td></tr>
</table>
{{img:taschen-werkzeugweg.png|The pocket form with the three strategies}}
<p>As with the profile there is the cut direction <em>climb</em> or <em>conventional</em>.</p>
<p>The distance between neighbouring passes is the <strong>stepover</strong>; depth and <strong>stepdown</strong> set the number of levels cleared one after another.</p>
""",
        "body": """
<p>Wählen Sie geschlossene Vektoren und klicken Sie auf <strong>Taschen-Werkzeugweg</strong>. Neben den allgemeinen Feldern ([[werkzeugauswahl]]) wählen Sie die <strong>Ausräumstrategie</strong>:</p>
<table>
<tr><th>Strategie</th><th>So räumt der Fräser aus</th></tr>
<tr><td><strong>Versatz</strong></td><td>In Ringen, die der Kontur folgen, von innen nach außen. Zwischen benachbarten Ringen bleibt der Fräser unten.</td></tr>
<tr><td><strong>Raster</strong></td><td>Hin und her in geraden, parallelen Bahnen, danach eine Runde an der Wand entlang. Der <strong>Rasterwinkel</strong> (in Grad) legt die Richtung fest.</td></tr>
<tr><td><strong>Spirale</strong></td><td>Eine durchgehende Spirale von innen nach außen: kein neuer Ansatz, jede Runde geht stufenlos in die nächste über.</td></tr>
</table>
{{img:taschen-werkzeugweg.png|Das Formular der Tasche mit den drei Strategien}}
<p>Dazu kommt wie beim Profil die Laufrichtung <em>Gleichlauf</em> oder <em>Gegenlauf</em>.</p>
<p>Der Abstand benachbarter Bahnen ist die <strong>Querzustellung</strong>; Tiefe und <strong>Zustellung</strong> bestimmen die Zahl der Ebenen, die nacheinander ausgeräumt werden.</p>
""",
    },
    {
        "slug": "bohr-werkzeugweg", "cat": C, "title": "Bohr-Werkzeugweg",
        "title_en": "Drilling toolpath",
        "lead": "Der Bohr-Werkzeugweg bohrt in die Mitte jeder gewählten Kontur.",
        "lead_en": "The drilling toolpath drills into the centre of every selected contour.",
        "body_en": """
<p>Select the shapes – usually small circles – and click <strong>Drilling toolpath</strong>. The drill moves to the centre of every closed contour and drills down to the <strong>cut depth</strong>.</p>
<ul>
<li>The <strong>stepdown</strong> here is the depth per peck: after each peck the drill lifts to clear the chips (“pecking”).</li>
<li>The circle diameter does not set the hole – the chosen tool does. Draw the circles the size of the drill so the drawing is right.</li>
</ul>
<p>The defaults are a drill and 10 mm depth; change both in the form.</p>
""",
        "body": """
<p>Wählen Sie die Formen – meist kleine Kreise – und klicken Sie auf <strong>Bohr-Werkzeugweg</strong>. Der Bohrer fährt zur Mitte jeder geschlossenen Kontur und bohrt bis zur <strong>Schnitttiefe</strong>.</p>
<ul>
<li>Die <strong>Zustellung</strong> ist hier die Tiefe je Bohrabschnitt: Nach jedem Abschnitt hebt sich der Bohrer, um Späne zu lösen („Pecken“).</li>
<li>Der Kreisdurchmesser bestimmt nicht die Bohrung – maßgeblich ist das gewählte Werkzeug. Zeichnen Sie die Kreise deshalb so groß wie den Bohrer, damit die Zeichnung stimmt.</li>
</ul>
<p>Voreingestellt sind ein Bohrer und 10 mm Tiefe; beides ändern Sie im Formular.</p>
""",
    },
    {
        "slug": "vcarving-werkzeugweg", "cat": C, "title": "V-Carving-Werkzeugweg",
        "title_en": "V-carving toolpath",
        "lead": "Beim V-Carving schnitzt ein V-Fräser Schrift und Formen; je breiter die Linie, desto tiefer geht er.",
        "lead_en": "In V-carving a V-bit carves lettering and shapes; the wider the line, the deeper it goes.",
        "body_en": """
<p>Select closed vectors – a piece of text, for example – and click <strong>V-carving toolpath</strong>. The tool is a V-bit; its tip angle and diameter determine the result.</p>
<p>The depth at a point comes from its distance to the edge of the shape, divided by the tangent of half the tip angle. So in wide areas the cutter plunges deeper than in narrow ones.</p>
<table>
<tr><th>Field</th><th>Meaning</th></tr>
<tr><td>Cut depth</td><td>The greatest depth the path may reach at all.</td></tr>
<tr><td>Depth limit</td><td>Limits how deep the tip goes in wide areas (in mm). <strong>0</strong> means: limited only by the cutter diameter.</td></tr>
<tr><td>Stepover</td><td>Distance between passes.</td></tr>
</table>
{{img:oberflaeche.png|In the example the “CncCraft” lettering is a V-carving toolpath (orange)}}
<div class="tipp"><p>Check V-carvings in the 3D preview ([[vorschau-werkzeugwege]]): there you see how deep the tips go and whether fine letters come out cleanly.</p></div>
""",
        "body": """
<p>Wählen Sie geschlossene Vektoren – zum Beispiel einen Text – und klicken Sie auf <strong>V-Carving-Werkzeugweg</strong>. Das Werkzeug ist ein V-Fräser; seine Spitzenwinkel und sein Durchmesser bestimmen das Ergebnis.</p>
<p>Die Tiefe an einem Punkt ergibt sich aus seinem Abstand zum Rand der Form, geteilt durch den Tangens des halben Spitzenwinkels. In breiten Bereichen taucht der Fräser also tiefer ein als in schmalen.</p>
<table>
<tr><th>Feld</th><th>Bedeutung</th></tr>
<tr><td>Schnitttiefe</td><td>Die größte Tiefe, die der Weg überhaupt erreichen darf.</td></tr>
<tr><td>Tiefenbegrenzung</td><td>Begrenzt, wie tief die Spitze in breiten Bereichen geht (in mm). <strong>0</strong> bedeutet: nur durch den Durchmesser des Fräsers begrenzt.</td></tr>
<tr><td>Querzustellung</td><td>Abstand der Bahnen.</td></tr>
</table>
{{img:oberflaeche.png|Im Beispiel ist die Schrift „CncCraft“ ein V-Carving-Werkzeugweg (orange)}}
<div class="tipp"><p>Prüfen Sie V-Carvings in der 3D-Vorschau ([[vorschau-werkzeugwege]]): Dort sehen Sie, wie tief die Spitzen gehen und ob feine Buchstaben sauber werden.</p></div>
""",
    },
    {
        "slug": "vorschau-werkzeugwege", "cat": C, "title": "Vorschau der Werkzeugwege",
        "title_en": "Toolpath preview",
        "lead": "Die Vorschau spielt die Werkzeugwege am Werkstück ab und zeigt, was die Fräse abtragen wird.",
        "lead_en": "The preview plays the toolpaths on the workpiece and shows what the router will remove.",
        "body_en": """
<p>The <strong>Simulation &amp; Export</strong> area at the bottom right has two buttons: <em>Preview</em> and <em>Save</em>. Preview starts the simulation; CncCraft switches to the [[ansicht-3d|3D view]] for it.</p>
{{img:ansicht-3d.png|The 3D preview}}
<table>
<tr><th>Control</th><th>Effect</th></tr>
<tr><td><strong>▶ Play</strong></td><td>Runs the toolpaths one after another; the material is removed as it goes.</td></tr>
<tr><td><strong>⏭ At once</strong></td><td>Shows the end result straight away.</td></tr>
<tr><td><strong>↺</strong></td><td>Resets the preview to the untouched material.</td></tr>
<tr><td>All / Selected / Visible</td><td>Which toolpaths the preview shows: all, the one clicked in the list, or the visible ones.</td></tr>
<tr><td><strong>Speed</strong></td><td>How fast the animation runs.</td></tr>
<tr><td><strong>Detail</strong></td><td>How finely the material is drawn: “faster” on the left, “more precise” on the right. This only changes the display, not the toolpaths.</td></tr>
<tr><td>Show G-code motion in 3D</td><td>Overlays the toolpaths as coloured lines on the workpiece.</td></tr>
</table>
<div class="neu-box"><p>The <strong>Detail</strong> slider (0–100 %) lets you choose between a fast rough view and a fine one. In the 3D view the view cube and the origin axis cross also help you keep your bearings.</p></div>
<h2>Milling time</h2>
<p>The list gives an estimated milling time per toolpath and, below it, the total time of the visible toolpaths ([[werkzeugwegliste]]). The estimate uses the set feeds and a flat rapid speed; the real time depends on your machine.</p>
""",
        "body": """
<p>Der Bereich <strong>Simulation & Export</strong> rechts unten hat zwei Knöpfe: <em>Vorschau</em> und <em>Speichern</em>. Mit der Vorschau starten Sie die Simulation; CncCraft schaltet dazu in die [[ansicht-3d|3D-Ansicht]].</p>
{{img:ansicht-3d.png|Die 3D-Vorschau}}
<table>
<tr><th>Bedienelement</th><th>Wirkung</th></tr>
<tr><td><strong>▶ Abspielen</strong></td><td>Fährt die Werkzeugwege nacheinander ab; das Material wird dabei abgetragen.</td></tr>
<tr><td><strong>⏭ Sofort</strong></td><td>Zeigt gleich das Endergebnis.</td></tr>
<tr><td><strong>↺</strong></td><td>Setzt die Vorschau zurück auf das unbearbeitete Material.</td></tr>
<tr><td>Alle / Ausgewählter / Sichtbare</td><td>Welche Werkzeugwege die Vorschau zeigt: alle, der in der Liste angeklickte oder die sichtbaren.</td></tr>
<tr><td><strong>Tempo</strong></td><td>Wie schnell die Animation läuft.</td></tr>
<tr><td><strong>Darstellung</strong></td><td>Wie fein das Material gezeichnet wird: „schneller“ links, „genauer“ rechts. Das ändert nur die Anzeige, nicht die Werkzeugwege.</td></tr>
<tr><td>Werkzeugpfade im 3D einblenden</td><td>Legt die Werkzeugwege als farbige Linien über das Werkstück.</td></tr>
</table>
<div class="neu-box"><p>Der Regler <strong>Darstellung</strong> (0–100 %) lässt Sie zwischen einer schnellen Grobansicht und einer feinen Darstellung wählen. In der 3D-Ansicht helfen außerdem der Ansichtswürfel und das Achsenkreuz des Nullpunkts bei der Orientierung.</p></div>
<h2>Fräszeit</h2>
<p>Die Liste nennt je Werkzeugweg eine geschätzte Fräszeit und darunter die Gesamtzeit der sichtbaren Werkzeugwege ([[werkzeugwegliste]]). Die Schätzung rechnet mit den eingestellten Vorschüben und einer pauschalen Eilganggeschwindigkeit; die echte Zeit hängt von Ihrer Maschine ab.</p>
""",
    },
    {
        "slug": "werkzeugwege-speichern", "cat": C, "title": "Werkzeugwege speichern", "neu": True,
        "title_en": "Saving toolpaths",
        "lead": "Für die Fräse speichern Sie die Werkzeugwege als G-Code-Datei.",
        "lead_en": "For the router you save the toolpaths as a G-code file.",
        "body_en": """
<p>Click <em>Save</em> in the <strong>Simulation &amp; Export</strong> area at the bottom right – or choose <em>File ▸ Save toolpaths …</em>.</p>
<h2>What gets saved</h2>
<ul>
<li><strong>Selected toolpath:</strong> only the one clicked in the list.</li>
<li><strong>Visible toolpaths:</strong> all visible toolpaths in the order of the list.</li>
</ul>
<p>A line below states how many toolpaths are involved and – if several tools are needed one after another – how many tool changes that is.</p>
<h2>Several tools: one file or one per tool</h2>
<div class="neu-box"><p>If the toolpaths need more than one tool, CncCraft asks when saving:</p>
{{img:export-werkzeuge.png|The question when saving toolpaths with several tools}}
<ul>
<li><strong>One file per tool:</strong> for each run of toolpaths with the same tool a separate G-code file is made; all files go together into a ZIP. The files are named <code>&lt;name&gt;_01_&lt;tool&gt;.nc</code>, <code>&lt;name&gt;_02_&lt;tool&gt;.nc</code> and so on: first the general file name (the ZIP's name), then a running number in list order, then the tool (for example <code>sign_01_End-mill-6-mm.nc</code>). So the files line up in the right milling order.</li>
<li><strong>Everything in one file:</strong> as before, one file with all toolpaths; before each tool change the change command stands (<code>T… M6</code>, not for GRBL).</li>
</ul>
<p>With the <strong>GRBL</strong> output, “everything in one file” is not possible with several tools: GRBL has no tool change, so only one file per tool remains.</p>
<p>A tool change counts only where the tool changes from the toolpath before: two toolpaths in a row with the same cutter stay in one file. If the same cutter appears again later after a different one, that is a new section with its own file.</p></div>
<h2>Post-processor</h2>
<p>The <strong>Post-processor</strong> choice sets how the G-code is written:</p>
<table>
<tr><th>Entry</th><th>Extension</th><th>Unit</th></tr>
<tr><td>G-code (mm)</td><td><code>.nc</code></td><td>Millimetres</td></tr>
<tr><td>G-code (inch)</td><td><code>.tap</code></td><td>Inches</td></tr>
<tr><td>GRBL (mm)</td><td><code>.gcode</code></td><td>Millimetres, for GRBL controllers</td></tr>
</table>
<p>Then click <strong>Save toolpaths …</strong> and choose the destination and file name.</p>
<div class="achtung"><p>Check every new file in the [[vorschau-werkzeugwege|preview]] and run it first without material or in scrap. Make sure the origin and safe height ([[neuer-job]]) match your machine.</p></div>
""",
        "body": """
<p>Klicken Sie rechts unten im Bereich <strong>Simulation & Export</strong> auf den Knopf <em>Speichern</em> – oder wählen Sie <em>Datei ▸ Werkzeugwege speichern …</em>.</p>
<h2>Was gespeichert wird</h2>
<ul>
<li><strong>Ausgewählter Werkzeugweg:</strong> nur der in der Liste angeklickte.</li>
<li><strong>Sichtbare Werkzeugwege:</strong> alle sichtbaren Werkzeugwege in der Reihenfolge der Liste.</li>
</ul>
<p>Eine Zeile darunter nennt, wie viele Werkzeugwege betroffen sind und – wenn mehrere Werkzeuge nacheinander gebraucht werden – wie viele Werkzeugwechsel das sind.</p>
<h2>Mehrere Werkzeuge: eine Datei oder je Werkzeug eine</h2>
<div class="neu-box"><p>Brauchen die Werkzeugwege mehr als ein Werkzeug, fragt CncCraft beim Speichern nach:</p>
{{img:export-werkzeuge.png|Die Frage beim Speichern von Werkzeugwegen mit mehreren Werkzeugen}}
<ul>
<li><strong>Je Werkzeug eine Datei:</strong> Für jede Folge von Werkzeugwegen mit demselben Werkzeug entsteht eine eigene G-Code-Datei; alle Dateien kommen zusammen in eine ZIP-Datei. Die Dateien heißen <code>&lt;Name&gt;_01_&lt;Werkzeug&gt;.nc</code>, <code>&lt;Name&gt;_02_&lt;Werkzeug&gt;.nc</code> und so weiter: erst der allgemeine Dateiname (der Name der ZIP-Datei), dann eine laufende Nummer in der Reihenfolge der Liste, dann das Werkzeug (zum Beispiel <code>schild_01_Schaftfraeser-6-mm.nc</code>). So stehen die Dateien beim Fräsen in der richtigen Reihenfolge untereinander.</li>
<li><strong>Alles in eine Datei:</strong> wie bisher eine Datei mit allen Werkzeugwegen; vor jedem Werkzeugwechsel steht der Wechselbefehl (<code>T… M6</code>, bei GRBL nicht).</li>
</ul>
<p>Mit der Ausgabe <strong>GRBL</strong> ist „Alles in eine Datei“ bei mehreren Werkzeugen nicht möglich: GRBL kennt keinen Werkzeugwechsel, also bleibt nur je Werkzeug eine Datei.</p>
<p>Ein Werkzeugwechsel zählt nur dort, wo sich das Werkzeug gegenüber dem Werkzeugweg davor ändert: Zwei Werkzeugwege hintereinander mit demselben Fräser bleiben in einer Datei. Kommt derselbe Fräser nach einem anderen später noch einmal vor, ist das ein neuer Abschnitt mit eigener Datei.</p></div>
<h2>Postprozessor</h2>
<p>Die Auswahl <strong>Postprozessor</strong> bestimmt, wie der G-Code geschrieben wird:</p>
<table>
<tr><th>Eintrag</th><th>Endung</th><th>Einheit</th></tr>
<tr><td>G-Code (mm)</td><td><code>.nc</code></td><td>Millimeter</td></tr>
<tr><td>G-Code (Zoll)</td><td><code>.tap</code></td><td>Zoll</td></tr>
<tr><td>GRBL (mm)</td><td><code>.gcode</code></td><td>Millimeter, für GRBL-Steuerungen</td></tr>
</table>
<p>Klicken Sie anschließend auf <strong>Werkzeugpfade speichern …</strong> und wählen Sie Ziel und Dateinamen.</p>
<div class="achtung"><p>Prüfen Sie jede neue Datei in der [[vorschau-werkzeugwege|Vorschau]] und fahren Sie sie zuerst ohne Material oder mit Abfall ab. Achten Sie darauf, dass Nullpunkt und Sicherheitshöhe ([[neuer-job]]) zu Ihrer Maschine passen.</p></div>
""",
    },
    {
        "slug": "nut-werkzeugweg", "cat": C, "title": "Nut-Werkzeugweg",
        "title_en": "Slot toolpath",
        "lead": "Die Nut fräst entlang eines Vektors einen Kanal, so breit wie der Fräser.",
        "lead_en": "The slot mills a channel along a vector, as wide as the cutter.",
        "neu": True,
        "body_en": """
<div class="neu-box"><p>The slot toolpath does not exist in the Spark template.</p></div>
<p>Select one or more vectors – open or closed – and click <strong>Slot</strong>. The slot is exactly as wide as the cutter; so there is no stepover.</p>
{{img:nut-werkzeugweg.png|The slot toolpath form}}
<table>
<tr><th>Setting</th><th>Meaning</th></tr>
<tr><td><strong>Side</strong></td><td><em>On the line</em>: the centre of the slot lies on the vector. <em>Left</em> / <em>Right</em>: the slot lies beside the vector, one edge on the line.</td></tr>
<tr><td><strong>Cap</strong></td><td>For open lines: <em>Outer</em> – the cutter runs past the end point, the slot becomes longer than the vector by the cutter radius. <em>Inner</em> – the cutter stops half a diameter before the end point, the slot is exactly as long as the vector.</td></tr>
<tr><td><strong>Cut direction</strong></td><td><em>Climb</em> or <em>conventional</em>.</td></tr>
<tr><td><strong>Ramp angle</strong></td><td>The cutter plunges at an angle instead of straight down; 0° = straight.</td></tr>
</table>
<div class="tipp"><p>In the form, hover over an option: a picture in the tooltip shows what it does – for example where the cutter stops with <em>Inner</em> and <em>Outer</em>, or how the turning and feed relate with climb and conventional.</p></div>
""",
        "body": """
<div class="neu-box"><p>Den Nut-Werkzeugweg gibt es in der Vorlage Spark nicht.</p></div>
<p>Wählen Sie einen oder mehrere Vektoren – offen oder geschlossen – und klicken Sie auf <strong>Nut</strong>. Die Nut ist genau so breit wie der Fräser; eine Querzustellung gibt es darum nicht.</p>
{{img:nut-werkzeugweg.png|Das Formular des Nut-Werkzeugwegs}}
<table>
<tr><th>Einstellung</th><th>Bedeutung</th></tr>
<tr><td><strong>Seite</strong></td><td><em>Auf der Linie</em>: die Mitte der Nut liegt auf dem Vektor. <em>Links</em> / <em>Rechts</em>: die Nut liegt neben dem Vektor, eine Kante auf der Linie.</td></tr>
<tr><td><strong>Kappe</strong></td><td>Bei offenen Linien: <em>Außen</em> – der Fräser fährt über den Endpunkt hinaus, die Nut wird um den Werkzeugradius länger. <em>Innen</em> – der Fräser stoppt einen halben Durchmesser vor dem Endpunkt, die Nut wird genau so lang wie der Vektor.</td></tr>
<tr><td><strong>Fräsrichtung</strong></td><td><em>Gleichlauf</em> oder <em>Gegenlauf</em>.</td></tr>
<tr><td><strong>Rampenwinkel</strong></td><td>Der Fräser taucht schräg statt senkrecht ein; 0° = senkrecht.</td></tr>
</table>
<div class="tipp"><p>Fahren Sie im Formular mit der Maus über eine Option: Ein Bild im Hinweis zeigt, was sie bewirkt – zum Beispiel wo bei <em>Innen</em> und <em>Außen</em> der Fräser stoppt, oder wie bei Gleich- und Gegenlauf Drehrichtung und Vorschub zueinander stehen.</p></div>
""",
    },
    {
        "slug": "materialien", "cat": C, "title": "Materialien und Grenzwerte",
        "title_en": "Materials and limits",
        "lead": "Jedes Werkstück ist aus einem Material. Es kennt seine Grenzen, empfiehlt Fräser und Kühlung und bestimmt das Aussehen in 3D.",
        "lead_en": "Every workpiece is made of a material. It knows its limits, recommends cutters and cooling and sets the look in 3D.",
        "neu": True,
        "body_en": """
<div class="neu-box"><p>The Spark template knows materials only as names in the tool database. In CncCraft every material has limits and recommendations, and CncCraft warns when a toolpath exceeds them – for beginners the most important help against broken cutters.</p></div>
<h2>Choosing a material</h2>
<p>When you create a new job, the material is mandatory: only once one is chosen can the job be created ([[neuer-job]]). CncCraft suggests the one used last. Under the picker the <strong>Recommendations</strong> area unfolds the description, cutting values, suitable cutters and cooling.</p>
<p>Shipped are <strong>softwood, hardwood, plywood, MDF, acrylic, HDPE/plastic, rigid foam, aluminium, brass</strong> and <strong>copper</strong>.</p>
<h2>What a material knows</h2>
<table>
<tr><th>Value</th><th>Meaning</th></tr>
<tr><td>Max. feed</td><td>The cutter should not move through the material faster (mm/min).</td></tr>
<tr><td>Speed</td><td>The recommended spindle range (rpm).</td></tr>
<tr><td>Max. stepdown</td><td>How deep per pass at most, as a fraction of the cutter diameter. A rule of thumb for wood: at most two thirds of the diameter.</td></tr>
<tr><td>Cutter</td><td>Which cutters suit: number of flutes, material, spiral direction.</td></tr>
<tr><td>Cooling</td><td>Whether and with what to cool – for example isopropanol for aluminium.</td></tr>
<tr><td>Look</td><td>How the workpiece looks in the 3D preview.</td></tr>
</table>
<h2>Warnings</h2>
<ul>
<li>In the <strong>toolpath form</strong> a note appears right under stepdown, feed, plunge or speed when the value is outside the recommendation – for example 6 mm stepdown with a 5 mm cutter in hardwood.</li>
<li>In the <strong>header of the G-code file</strong> there is a section “WARNING: TOO FAST OR TOO DEEP FOR THE MATERIAL” with every affected toolpath ([[werkzeugwege-speichern]]). Slower and shallower than recommended is never a problem and is not reported.</li>
</ul>
<h2>Look in the 3D preview</h2>
<p>The [[vorschau-werkzeugwege|preview]] shows the workpiece in the look of its material: grain for wood, the layers at a plywood edge, brushed and shiny metal.</p>
{{img:material-hartholz.png|A hardwood workpiece in the 3D preview}}
{{img:material-aluminium.png|The same workpiece in aluminium}}
<h2>Your own materials</h2>
<p><strong>Manage …</strong> at the end of the material picker opens material management. With <strong>+</strong> you make a copy of the chosen material – with its tools, trimmed to the new material's limits. Name, description, limits, recommendations and look are editable. Shipped materials and the current job's material cannot be deleted.</p>
{{img:materialien.png|Material management}}
<div class="achtung"><p>The limits are cautious starting values for hobby routers. Your machine and your cutters may do more or less – mill a test in scrap material first.</p></div>
""",
        "body": """
<div class="neu-box"><p>Die Vorlage Spark kennt Materialien nur als Namen in der Werkzeugdatenbank. In CncCraft hat jedes Material Grenzwerte und Empfehlungen, und CncCraft warnt, wenn ein Werkzeugweg sie überschreitet – gerade für den Einstieg die wichtigste Hilfe gegen abgebrochene Fräser.</p></div>
<h2>Material wählen</h2>
<p>Beim Anlegen eines neuen Jobs ist das Material Pflicht: Erst wenn eines gewählt ist, lässt sich der Job erstellen ([[neuer-job]]). CncCraft schlägt dabei das zuletzt benutzte vor. Unter dem Auswahlfeld klappt der Bereich <strong>Empfehlungen</strong> Beschreibung, Schnittwerte, passende Fräser und Kühlung auf.</p>
<p>Mitgeliefert werden <strong>Weichholz, Hartholz, Sperrholz, MDF, Acryl, HDPE/Kunststoff, Hartschaum, Aluminium, Messing</strong> und <strong>Kupfer</strong>.</p>
<h2>Was ein Material weiß</h2>
<table>
<tr><th>Wert</th><th>Bedeutung</th></tr>
<tr><td>Max. Vorschub</td><td>Schneller sollte der Fräser nicht durch das Material fahren (mm/min).</td></tr>
<tr><td>Drehzahl</td><td>Der empfohlene Bereich der Spindel (U/min).</td></tr>
<tr><td>Max. Zustellung</td><td>Wie tief je Durchgang höchstens, als Anteil des Fräserdurchmessers. Eine Faustregel für Holz: höchstens zwei Drittel des Durchmessers.</td></tr>
<tr><td>Fräser</td><td>Welche Fräser passen: Schneidenzahl, Werkstoff, Spiralrichtung.</td></tr>
<tr><td>Kühlung</td><td>Ob und womit gekühlt wird – etwa Isopropanol bei Aluminium.</td></tr>
<tr><td>Aussehen</td><td>Wie das Werkstück in der 3D-Vorschau aussieht.</td></tr>
</table>
<h2>Warnungen</h2>
<ul>
<li>Im <strong>Werkzeugweg-Formular</strong> erscheint direkt unter Zustellung, Vorschub, Eintauchen oder Drehzahl ein Hinweis, wenn der Wert außerhalb der Empfehlung liegt – zum Beispiel 6 mm Zustellung mit einem 5-mm-Fräser in Hartholz.</li>
<li>Im <strong>Kopf der G-Code-Datei</strong> steht ein Abschnitt „ACHTUNG: ZU SCHNELL ODER ZU TIEF FÜR DAS MATERIAL“ mit jedem betroffenen Werkzeugweg ([[werkzeugwege-speichern]]). Langsamer und flacher als empfohlen ist nie ein Problem und wird nicht gemeldet.</li>
</ul>
<h2>Aussehen in der 3D-Vorschau</h2>
<p>Die [[vorschau-werkzeugwege|Vorschau]] zeigt das Werkstück im Aussehen seines Materials: Maserung bei Holz, die Lagen an der Sperrholzkante, gebürstetes und glänzendes Metall.</p>
{{img:material-hartholz.png|Ein Werkstück aus Hartholz in der 3D-Vorschau}}
{{img:material-aluminium.png|Dasselbe Werkstück aus Aluminium}}
<h2>Eigene Materialien</h2>
<p><strong>Verwalten …</strong> am Ende der Materialauswahl öffnet die Material-Verwaltung. Mit <strong>+</strong> legen Sie eine Kopie des gewählten Materials an – samt dessen Werkzeugen, die auf die Grenzen des neuen Materials gekürzt werden. Name, Beschreibung, Grenzwerte, Empfehlungen und Aussehen sind änderbar. Mitgelieferte Materialien und das Material des aktuellen Jobs lassen sich nicht löschen.</p>
{{img:materialien.png|Die Material-Verwaltung}}
<div class="achtung"><p>Die Grenzwerte sind vorsichtige Ausgangswerte für Hobbyfräsen. Ihre Maschine und Ihre Fräser können mehr oder weniger – fräsen Sie zuerst eine Probe in Abfallmaterial.</p></div>
""",
    },
]
