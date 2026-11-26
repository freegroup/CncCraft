"""Kategorie „Entwurfswerkzeuge“."""

C = "entwurf"

PAGES = [
    {
        "slug": "auswahlwerkzeug", "cat": C, "title": "Auswählen und verschieben",
        "title_en": "Select and move",
        "lead": "Mit dem Auswahlwerkzeug klicken Sie Formen an, verschieben sie und ändern ihre Größe.",
        "lead_en": "With the select tool you click shapes, move them and change their size.",
        "body_en": """
<p>The select tool (arrow, key <kbd>S</kbd>) is CncCraft's normal mode.</p>
<h2>Selecting</h2>
<ul>
<li><strong>Click</strong> a shape to select it. A click on empty space clears the selection.</li>
<li><strong>Drag a box</strong> to select all shapes inside it.</li>
<li><kbd>⇧</kbd> + click adds shapes to the selection or takes them out again. <kbd>⌘A</kbd> selects everything.</li>
</ul>
<p>The selection appears with a box, handles at the corners and a centre mark. The status line at the bottom shows the width and height of the selection.</p>
<h2>Moving and scaling</h2>
<ul>
<li><strong>Move:</strong> drag the shape – or the centre mark. With <kbd>⌥</kbd> or <kbd>⌃</kbd> while dragging a copy is made; with <kbd>⇧</kbd> nothing snaps.</li>
<li><strong>Arrow keys</strong> move the selection in small steps, with <kbd>⇧</kbd> ten times.</li>
<li><strong>Resize:</strong> drag a handle. <kbd>⇧</kbd> keeps the aspect ratio.</li>
</ul>
<h2>Typing dimensions</h2>
<p><kbd>Enter</kbd> with exactly one shape selected opens the dimension fields right at the shape (<em>Edit ▸ Edit dimensions</em>). With <kbd>↑</kbd> <kbd>↓</kbd> or <kbd>Tab</kbd> you switch fields. See [[eigenschaften-masse]].</p>
<h2>Help while moving</h2>
<p>While dragging, the selection snaps to nodes, centres, midpoints, job corners and the grid – more under [[einrasten]].</p>
""",
        "body": """
<p>Das Auswahlwerkzeug (Pfeil, Taste <kbd>S</kbd>) ist der Normalmodus von CncCraft.</p>
<h2>Auswählen</h2>
<ul>
<li><strong>Klick</strong> auf eine Form wählt sie. Ein Klick ins Leere hebt die Auswahl auf.</li>
<li><strong>Rahmen ziehen</strong> wählt alle Formen, die im Rahmen liegen.</li>
<li><kbd>⇧</kbd> + Klick fügt Formen der Auswahl hinzu oder nimmt sie wieder heraus. <kbd>⌘A</kbd> wählt alles.</li>
</ul>
<p>Die Auswahl erscheint mit einem Rahmen, Griffen an den Ecken und einer Mittenmarke. Unten in der Statuszeile stehen Breite und Höhe der Auswahl.</p>
<h2>Verschieben und skalieren</h2>
<ul>
<li><strong>Verschieben:</strong> die Form ziehen – oder die Mittenmarke. Mit <kbd>⌥</kbd> oder <kbd>⌃</kbd> beim Ziehen entsteht eine Kopie; mit <kbd>⇧</kbd> rastet nichts ein.</li>
<li><strong>Pfeiltasten</strong> verschieben die Auswahl in kleinen Schritten, mit <kbd>⇧</kbd> zehnfach.</li>
<li><strong>Größe ändern:</strong> an einem Griff ziehen. <kbd>⇧</kbd> hält das Seitenverhältnis.</li>
</ul>
<h2>Maße eintippen</h2>
<p><kbd>Enter</kbd> bei genau einer gewählten Form öffnet die Maßfelder direkt an der Form (<em>Bearbeiten ▸ Maße bearbeiten</em>). Mit <kbd>↑</kbd> <kbd>↓</kbd> oder <kbd>Tab</kbd> wechseln Sie die Felder. Siehe [[eigenschaften-masse]].</p>
<h2>Hilfen beim Verschieben</h2>
<p>Beim Ziehen rastet die Auswahl an Knoten, Mitten, Halbierungspunkten, Jobecken und Raster ein – mehr unter [[einrasten]].</p>
""",
    },
    {
        "slug": "transformieren", "cat": C, "title": "Objekte transformieren",
        "title_en": "Transform objects",
        "lead": "Das Formular „Objekte transformieren“ legt Lage und Größe einer Form durch Zahlen fest.",
        "lead_en": "The “Transform objects” form sets a shape's position and size by numbers.",
        "body_en": """
<p>The “Transform objects” button in the toolbar (<em>Mode</em> group) opens a form for the selected shape to the left of the drawing. It is the same block of fields as under [[eigenschaften-masse]]:</p>
<ul>
<li><strong>Centre X / Centre Y</strong> – where the shape's centre lies (from the job's origin).</li>
<li><strong>Width / Height</strong> for rectangle and ellipse, <strong>Radius</strong> for circle and polygon, <strong>Corners</strong> for the polygon, <strong>Corner radius</strong> for the rectangle.</li>
<li><strong>Text</strong> and <strong>Height</strong> for text.</li>
</ul>
<p>Every field takes numbers, formulas and variables ([[formeln-in-feldern]]). Change a value and the shape follows at once; a click on the cross closes the form.</p>
<p>If several shapes are selected, the form shows only their count – to move and scale them together use the [[auswahlwerkzeug|select tool]].</p>
""",
        "body": """
<p>Der Knopf „Objekte transformieren“ in der Werkzeugleiste (Gruppe <em>Modus</em>) öffnet links neben der Zeichnung ein Formular für die gewählte Form. Es ist dasselbe Feldblock wie unter [[eigenschaften-masse]]:</p>
<ul>
<li><strong>Mitte X / Mitte Y</strong> – wo der Mittelpunkt der Form liegt (vom Nullpunkt des Jobs aus).</li>
<li><strong>Breite / Höhe</strong> bei Rechteck und Ellipse, <strong>Radius</strong> bei Kreis und Polygon, <strong>Ecken</strong> beim Polygon, <strong>Eckradius</strong> beim Rechteck.</li>
<li><strong>Text</strong> und <strong>Höhe</strong> bei Text.</li>
</ul>
<p>Jedes Feld nimmt Zahlen, Formeln und Variablen an ([[formeln-in-feldern]]). Ändern Sie einen Wert, folgt die Form sofort; ein Klick auf das Kreuz schließt das Formular.</p>
<p>Sind mehrere Formen gewählt, zeigt das Formular nur deren Anzahl – zum gemeinsamen Verschieben und Skalieren benutzen Sie dann das [[auswahlwerkzeug]].</p>
""",
    },
    {
        "slug": "linie-polylinie", "cat": C, "title": "Linien und Polylinien zeichnen",
        "title_en": "Draw lines and polylines",
        "lead": "Mit dem Linienwerkzeug entstehen offene oder geschlossene Linienzüge aus Strecken.",
        "lead_en": "The line tool makes open or closed polylines from straight segments.",
        "body_en": """
<p>Choose the line tool (key <kbd>L</kbd>) and click the corner points into the drawing one after another. Between clicks a preview shows where the next segment leads. Snapping ([[einrasten]]) helps you hit points exactly.</p>
<table>
<tr><th>Key</th><th>Action</th></tr>
<tr><td><kbd>Enter</kbd> or double-click</td><td>finish the polyline (open)</td></tr>
<tr><td><kbd>Tab</kbd></td><td>close the polyline: the last point joins the first</td></tr>
<tr><td><kbd>Space</kbd></td><td>finish and start a new polyline at once</td></tr>
<tr><td><kbd>Esc</kbd></td><td>cancel</td></tr>
</table>
<p>Only closed polylines can be cut out or cleared. Open lines suit “on the line” profiles and V-carving.</p>
<div class="tipp"><p>A single segment can later be set to a fixed length with <em>Distance</em> ([[abstand-bemassen]]) or made level or upright with <em>Level/upright</em> ([[waagrecht-senkrecht]]).</p></div>
""",
        "body": """
<p>Wählen Sie das Linienwerkzeug (Taste <kbd>L</kbd>) und klicken Sie nacheinander die Eckpunkte in die Zeichnung. Zwischen den Klicks zeigt eine Vorschau, wohin die nächste Strecke führt. Einrasthilfen ([[einrasten]]) helfen, Punkte genau zu treffen.</p>
<table>
<tr><th>Taste</th><th>Wirkung</th></tr>
<tr><td><kbd>Enter</kbd> oder Doppelklick</td><td>Linienzug beenden (offen)</td></tr>
<tr><td><kbd>Tab</kbd></td><td>Linienzug schließen: der letzte Punkt wird mit dem ersten verbunden</td></tr>
<tr><td><kbd>Leertaste</kbd></td><td>beenden und sofort einen neuen Linienzug beginnen</td></tr>
<tr><td><kbd>Esc</kbd></td><td>abbrechen</td></tr>
</table>
<p>Nur geschlossene Linienzüge lassen sich ausfräsen oder ausräumen. Offene Linien eignen sich für Profile „auf der Linie“ und zum V-Carving.</p>
<div class="tipp"><p>Eine einzelne Strecke lässt sich später mit <em>Abstand</em> auf eine feste Länge ([[abstand-bemassen]]) oder mit <em>Waagrecht/Senkrecht</em> ([[waagrecht-senkrecht]]) festlegen.</p></div>
""",
    },
    {
        "slug": "rechteck", "cat": C, "title": "Rechteck zeichnen",
        "title_en": "Draw a rectangle",
        "lead": "Rechtecke ziehen Sie von einer Ecke zur gegenüberliegenden auf; abgerundete Ecken sind ein Maßfeld.",
        "lead_en": "Draw rectangles from one corner to the opposite one; rounded corners are a dimension field.",
        "body_en": """
<p>Choose the rectangle (key <kbd>R</kbd>), press at a corner and drag to the opposite one. On release the rectangle is finished and selected.</p>
<p>Under [[eigenschaften-masse]] you set the centre, <strong>width</strong> and <strong>height</strong> exactly; the <strong>corner radius</strong> rounds the corners. The corner radius also affects the toolpaths.</p>
""",
        "body": """
<p>Wählen Sie das Rechteck (Taste <kbd>R</kbd>), drücken Sie an einer Ecke und ziehen Sie zur gegenüberliegenden. Beim Loslassen ist das Rechteck fertig und ausgewählt.</p>
<p>Unter [[eigenschaften-masse]] stellen Sie Mitte, <strong>Breite</strong> und <strong>Höhe</strong> genau ein; der <strong>Eckradius</strong> rundet die Ecken ab. Der Eckradius wirkt auch auf die Werkzeugwege.</p>
""",
    },
    {
        "slug": "kreis", "cat": C, "title": "Kreis zeichnen",
        "title_en": "Draw a circle",
        "lead": "Kreise zeichnen Sie vom Mittelpunkt aus; der Radius ist ein Maßfeld.",
        "lead_en": "Draw circles from the centre; the radius is a dimension field.",
        "body_en": """
<p>Choose the circle (key <kbd>C</kbd>), press on the centre and drag to the radius you want. The centre and radius can then be typed exactly under [[eigenschaften-masse]].</p>
<p>Circles are the basis for holes: the [[bohr-werkzeugweg|drilling toolpath]] drills in the centre of every selected contour.</p>
""",
        "body": """
<p>Wählen Sie den Kreis (Taste <kbd>C</kbd>), drücken Sie auf den Mittelpunkt und ziehen Sie bis zum gewünschten Radius. Mitte und Radius lassen sich danach unter [[eigenschaften-masse]] genau eintippen.</p>
<p>Kreise sind die Grundlage für Bohrungen: Der [[bohr-werkzeugweg|Bohr-Werkzeugweg]] bohrt in der Mitte jeder gewählten Kontur.</p>
""",
    },
    {
        "slug": "ellipse", "cat": C, "title": "Ellipse zeichnen",
        "title_en": "Draw an ellipse",
        "lead": "Eine Ellipse hat zwei Maße: Breite und Höhe.",
        "lead_en": "An ellipse has two dimensions: width and height.",
        "body_en": """
<p>Choose the ellipse and drag the bounding box it should fit into. Width and height can then be set more precisely under [[eigenschaften-masse]]. An ellipse with equal width and height is a circle.</p>
""",
        "body": """
<p>Wählen Sie die Ellipse und ziehen Sie den Begrenzungsrahmen auf, in den sie passen soll. Breite und Höhe stellen Sie danach unter [[eigenschaften-masse]] genauer ein. Eine Ellipse mit gleicher Breite und Höhe ist ein Kreis.</p>
""",
    },
    {
        "slug": "polygon", "cat": C, "title": "Polygon zeichnen",
        "title_en": "Draw a polygon",
        "lead": "Ein Polygon ist ein regelmäßiges Vieleck mit einstellbarer Eckenzahl.",
        "lead_en": "A polygon is a regular n-gon with an adjustable number of corners.",
        "body_en": """
<p>Choose the polygon, press on the centre and drag to the size you want. It has at least three corners; change the number of <strong>corners</strong> and the <strong>radius</strong> under [[eigenschaften-masse]]. The example contains a hexagon.</p>
""",
        "body": """
<p>Wählen Sie das Polygon, drücken Sie auf den Mittelpunkt und ziehen Sie bis zur gewünschten Größe. Es hat mindestens drei Ecken; die Zahl der <strong>Ecken</strong> und den <strong>Radius</strong> ändern Sie unter [[eigenschaften-masse]]. Das Beispiel enthält ein Sechseck.</p>
""",
    },
    {
        "slug": "text-erstellen", "cat": C, "title": "Text erstellen",
        "title_en": "Create text",
        "lead": "Text wird als Vektorform eingefügt und kann danach gefräst oder in Kurven umgewandelt werden.",
        "lead_en": "Text is inserted as a vector shape and can then be milled or converted to curves.",
        "body_en": """
<p>Choose the text tool and click in the drawing. A sample text is inserted; change its content in the <strong>Text</strong> field under [[eigenschaften-masse]], and the size with the <strong>Height</strong> field.</p>
<p>Lettering suits [[vcarving-werkzeugweg|V-carving]] especially: the V-bit runs the letters at different depths by width. To edit the individual letter outlines, first convert the text [[in-kurven-umwandeln|to curves]].</p>
""",
        "body": """
<p>Wählen Sie das Textwerkzeug und klicken Sie in die Zeichnung. Ein Beispieltext wird eingefügt; seinen Inhalt ändern Sie im Feld <strong>Text</strong> unter [[eigenschaften-masse]], die Größe mit dem Feld <strong>Höhe</strong>.</p>
<p>Schrift eignet sich besonders für das [[vcarving-werkzeugweg|V-Carving]]: Der V-Fräser fährt die Buchstaben je nach Breite unterschiedlich tief ab. Wollen Sie die einzelnen Buchstabenumrisse bearbeiten, wandeln Sie den Text zuerst [[in-kurven-umwandeln|in Kurven um]].</p>
""",
    },
    {
        "slug": "knotenbearbeitung", "cat": C, "title": "Knoten bearbeiten",
        "title_en": "Node editing",
        "lead": "Im Knotenmodus ziehen Sie die Eckpunkte einer Form einzeln, fügen welche ein oder löschen sie.",
        "lead_en": "In node mode you drag a shape's corner points one by one, insert them or delete them.",
        "body_en": """
<p>Select a shape and press <kbd>N</kbd> (or choose the “Node editing” tool). The shape now shows its nodes.</p>
<ul>
<li><strong>Drag a node:</strong> with the mouse. While dragging the node snaps to help points; <kbd>⇧</kbd> turns that off.</li>
<li><strong>Insert a node:</strong> double-click the line.</li>
<li><strong>Delete a node:</strong> select the node and press <kbd>Del</kbd>.</li>
<li><strong>Move with the arrow keys:</strong> selected nodes in small steps (<kbd>⇧</kbd>: ten times).</li>
</ul>
<p><kbd>N</kbd> or <kbd>Esc</kbd> ends node mode. With no shape selected the status line reports “Select a vector first, then press ‘N’”.</p>
<div class="hinweis"><p>Circles, rectangles and other basic shapes become free curves when node-edited. Dimensions that were tied to variables fall away – the shape then has no reference sizes any more.</p></div>
""",
        "body": """
<p>Wählen Sie eine Form und drücken Sie <kbd>N</kbd> (oder wählen Sie das Werkzeug „Knoten bearbeiten“). Die Form zeigt jetzt ihre Knoten.</p>
<ul>
<li><strong>Knoten ziehen:</strong> mit der Maus. Beim Ziehen rastet der Knoten an Hilfspunkten ein; <kbd>⇧</kbd> schaltet das ab.</li>
<li><strong>Knoten einfügen:</strong> Doppelklick auf die Linie.</li>
<li><strong>Knoten löschen:</strong> Knoten wählen und <kbd>Entf</kbd>.</li>
<li><strong>Verschieben mit den Pfeiltasten:</strong> gewählte Knoten in kleinen Schritten (<kbd>⇧</kbd>: zehnfach).</li>
</ul>
<p><kbd>N</kbd> oder <kbd>Esc</kbd> beendet den Knotenmodus. Ohne gewählte Form meldet die Statuszeile „Erst einen Vektor auswählen, dann ‚N‘ drücken“.</p>
<div class="hinweis"><p>Kreise, Rechtecke und andere Grundformen werden beim Knotenbearbeiten zu freien Kurven. Maße, die mit Variablen verbunden waren, fallen dabei weg – die Form hat danach keine Bezugsgrößen mehr.</p></div>
""",
    },
    {
        "slug": "versatz", "cat": C, "title": "Versatz",
        "title_en": "Offset",
        "lead": "Der Versatz erzeugt eine Kontur im festen Abstand – größer oder kleiner als das Original.",
        "lead_en": "Offset creates a contour at a fixed distance – larger or smaller than the original.",
        "body_en": """
<p>Select one or more shapes and open the “Offset vectors” tool (key <kbd>O</kbd>). In the form on the left you set:</p>
<ul>
<li><strong>Distance</strong> in millimetres,</li>
<li><strong>Outside</strong> (the new contour is larger) or <strong>Inside</strong> (smaller).</li>
</ul>
<p>The button in the form creates the new contour as an extra shape and selects it; the original stays. If the offset gives no shape – for example because the contour vanishes entirely inward – the status line reports it (“Offset gives no shape”).</p>
<div class="tipp"><p>A typical use: a border at a fixed distance from the board, or a contour that should be larger by the cutter radius.</p></div>
""",
        "body": """
<p>Wählen Sie eine oder mehrere Formen und öffnen Sie das Werkzeug „Vektoren versetzen“ (Taste <kbd>O</kbd>). Im Formular links stellen Sie ein:</p>
<ul>
<li><strong>Abstand</strong> in Millimetern,</li>
<li><strong>Außen</strong> (die neue Kontur ist größer) oder <strong>Innen</strong> (kleiner).</li>
</ul>
<p>Der Knopf im Formular erzeugt die neue Kontur als zusätzliche Form und wählt sie aus; das Original bleibt erhalten. Wenn der Versatz keine Form ergibt – zum Beispiel weil die Kontur nach innen ganz verschwindet –, meldet das die Statuszeile („Versatz ergibt keine Form“).</p>
<div class="tipp"><p>Typische Anwendung: ein Rahmen im festen Abstand zur Platte, oder eine Kontur, die um den Fräserradius größer sein soll.</p></div>
""",
    },
    {
        "slug": "spiegeln", "cat": C, "title": "Spiegeln",
        "title_en": "Mirror",
        "lead": "Spiegeln kehrt die gewählten Formen um ihre Mitte waagrecht oder senkrecht um.",
        "lead_en": "Mirror flips the selected shapes about their centre, level or upright.",
        "body_en": """
<p>Select the shapes and in the “Align and mirror” group click <strong>Mirror horizontally</strong> or <strong>Mirror vertically</strong>. The shapes are flipped about the centre of the selection; with several shapes their arrangement to one another is kept.</p>
""",
        "body": """
<p>Wählen Sie die Formen und klicken Sie in der Gruppe „Ausrichten und Spiegeln“ auf <strong>Waagrecht spiegeln</strong> oder <strong>Senkrecht spiegeln</strong>. Die Formen werden um die Mitte der Auswahl umgeklappt; bei mehreren Formen bleibt ihre Lage zueinander erhalten.</p>
""",
    },
    {
        "slug": "zentrieren", "cat": C, "title": "Auf dem Material zentrieren",
        "title_en": "Centre on the material",
        "lead": "Zentrieren schiebt die Auswahl in die Mitte des Werkstücks.",
        "lead_en": "Centring moves the selection to the middle of the workpiece.",
        "body_en": """
<p>Select the shapes and click <strong>Centre on the material</strong> (“Align and mirror” group). The centre of the whole selection is placed on the centre of the board; its size does not change.</p>
""",
        "body": """
<p>Wählen Sie die Formen und klicken Sie auf <strong>Auf dem Material zentrieren</strong> (Gruppe „Ausrichten und Spiegeln“). Die Mitte der gesamten Auswahl wird auf die Mitte der Platte gelegt; ihre Größe ändert sich nicht.</p>
""",
    },
    {
        "slug": "in-kurven-umwandeln", "cat": C, "title": "In Kurven umwandeln",
        "title_en": "Convert to curves",
        "lead": "Das Umwandeln macht aus Grundformen und Text freie Kurven, deren Knoten Sie bearbeiten können.",
        "lead_en": "Converting turns basic shapes and text into free curves whose nodes you can edit.",
        "body_en": """
<p>Select shapes or text and click <strong>Convert to curves</strong> (“Edit objects” group). A circle becomes a curve of nodes, text becomes the outlines of the letters. After that everything can be shaped in [[knotenbearbeitung|node editing]] mode.</p>
<div class="hinweis"><p>Converted shapes no longer have dimension fields like “radius” or “text”, and variables on them are released. So if you still want to change the shape parametrically, convert it last.</p></div>
""",
        "body": """
<p>Wählen Sie Formen oder Text und klicken Sie auf <strong>In Kurven umwandeln</strong> (Gruppe „Objekte bearbeiten“). Aus einem Kreis wird eine Kurve aus Knoten, aus Text die Umrisse der Buchstaben. Danach lässt sich alles im Modus [[knotenbearbeitung|Knoten bearbeiten]] formen.</p>
<div class="hinweis"><p>Umgewandelte Formen haben keine Maßfelder wie „Radius“ oder „Text“ mehr, und Variablen an ihnen werden gelöst. Wer die Form noch parametrisch ändern will, wandelt sie deshalb erst am Ende um.</p></div>
""",
    },
    {
        "slug": "boolesche-operationen", "cat": C, "title": "Boolesche Operationen",
        "title_en": "Boolean operations",
        "lead": "Vereinen, Abziehen und Schnittmenge setzen aus mehreren geschlossenen Formen eine neue zusammen.",
        "lead_en": "Union, subtract and intersect build a new shape from several closed shapes.",
        "body_en": """
<p>Select at least two closed shapes and in the “Boolean operations” group click one of the three buttons:</p>
<table>
<tr><th>Operation</th><th>Result</th></tr>
<tr><td><strong>Union</strong></td><td>One shape covering all selected areas.</td></tr>
<tr><td><strong>Subtract</strong></td><td>The shape selected first, with all others subtracted from it.</td></tr>
<tr><td><strong>Intersect</strong></td><td>Only the area all selected shapes have in common.</td></tr>
</table>
<p>The original shapes are replaced by the result. With <kbd>⌘Z</kbd> you undo the step.</p>
<div class="tipp"><p>For “Subtract” the order matters: select the base shape first (the board, for example), then the cut-outs with <kbd>⇧</kbd> + click.</p></div>
""",
        "body": """
<p>Wählen Sie mindestens zwei geschlossene Formen und klicken Sie in der Gruppe „Boolesche Operationen“ auf eine der drei Schaltflächen:</p>
<table>
<tr><th>Operation</th><th>Ergebnis</th></tr>
<tr><td><strong>Vereinen</strong></td><td>Eine Form, die alle gewählten Flächen umfasst.</td></tr>
<tr><td><strong>Abziehen</strong></td><td>Die zuerst gewählte Form, von der alle anderen abgezogen werden.</td></tr>
<tr><td><strong>Schnittmenge</strong></td><td>Nur der Bereich, den alle gewählten Formen gemeinsam haben.</td></tr>
</table>
<p>Die ursprünglichen Formen werden durch das Ergebnis ersetzt. Mit <kbd>⌘Z</kbd> machen Sie den Schritt rückgängig.</p>
<div class="tipp"><p>Bei „Abziehen“ kommt es auf die Reihenfolge an: Wählen Sie zuerst die Grundform (zum Beispiel die Platte), dann mit <kbd>⇧</kbd> + Klick die Ausschnitte.</p></div>
""",
    },
    {
        "slug": "kopieren-einfuegen", "cat": C, "title": "Kopieren, Einfügen und Duplizieren",
        "title_en": "Copy, paste and duplicate",
        "lead": "Formen lassen sich kopieren und einfügen – samt ihrer Werkzeugwege und der Bedingungen zwischen ihnen.",
        "lead_en": "Shapes can be copied and pasted – with their toolpaths and the constraints between them.",
        "body_en": """
<table>
<tr><th>Command</th><th>Key</th><th>Effect</th></tr>
<tr><td>Copy</td><td><kbd>⌘C</kbd></td><td>puts the selection on the clipboard</td></tr>
<tr><td>Cut</td><td><kbd>⌘X</kbd></td><td>copies and deletes</td></tr>
<tr><td>Paste</td><td><kbd>⌘V</kbd></td><td>pastes the copy</td></tr>
<tr><td>Duplicate</td><td><kbd>⌘D</kbd></td><td>copies and pastes in one step</td></tr>
<tr><td>Delete</td><td><kbd>Del</kbd></td><td>removes the selection</td></tr>
</table>
<p>All commands are also in the <em>Edit</em> menu. On Windows and Linux <kbd>Ctrl</kbd> applies instead of <kbd>⌘</kbd>.</p>
<div class="neu-box"><p>When copying, CncCraft takes along what hangs on the shapes:</p>
<ul>
<li><strong>Toolpaths:</strong> if a shape belongs to a toolpath, a matching toolpath is made for the copy when pasting.</li>
<li><strong>Constraints:</strong> distance and level/upright constraints are copied only if all shapes they connect are part of the copy.</li>
</ul></div>
<p>Hold <kbd>⌥</kbd> or <kbd>⌃</kbd> while moving and a copy is made at the destination.</p>
""",
        "body": """
<table>
<tr><th>Befehl</th><th>Taste</th><th>Wirkung</th></tr>
<tr><td>Kopieren</td><td><kbd>⌘C</kbd></td><td>legt die Auswahl in die Zwischenablage</td></tr>
<tr><td>Ausschneiden</td><td><kbd>⌘X</kbd></td><td>kopiert und löscht</td></tr>
<tr><td>Einfügen</td><td><kbd>⌘V</kbd></td><td>fügt die Kopie ein</td></tr>
<tr><td>Duplizieren</td><td><kbd>⌘D</kbd></td><td>kopiert und fügt in einem Schritt ein</td></tr>
<tr><td>Löschen</td><td><kbd>Entf</kbd></td><td>entfernt die Auswahl</td></tr>
</table>
<p>Alle Befehle stehen auch im Menü <em>Bearbeiten</em>. Auf Windows und Linux gilt <kbd>Strg</kbd> statt <kbd>⌘</kbd>.</p>
<div class="neu-box"><p>CncCraft nimmt beim Kopieren mit, was an den Formen hängt:</p>
<ul>
<li><strong>Werkzeugwege:</strong> Gehört eine Form zu einem Werkzeugweg, entsteht beim Einfügen ein entsprechender Werkzeugweg für die Kopie.</li>
<li><strong>Bedingungen:</strong> Abstands- und Waagrecht/Senkrecht-Bedingungen werden nur mitkopiert, wenn alle Formen, die sie verbinden, Teil der Kopie sind.</li>
</ul></div>
<p>Wenn Sie beim Verschieben <kbd>⌥</kbd> oder <kbd>⌃</kbd> halten, entsteht eine Kopie am Zielort.</p>
""",
        "neu": True,
    },
    {
        "slug": "einrasten", "cat": C, "title": "Einrasten",
        "title_en": "Snapping",
        "lead": "Beim Zeichnen und Verschieben rastet der Zeiger an Hilfspunkten ein, damit alles genau sitzt.",
        "lead_en": "While drawing and moving, the pointer snaps to help points so everything sits exactly.",
        "body_en": """
<p>While moving a shape, CncCraft lines it up on the other shapes and groups with three snap
methods. Each has its own button in the header, in the <strong>Snapping</strong> group, and you
turn them on and off on their own or all together:</p>
<ul>
<li><strong>Snap to geometry (bounding box)</strong> – the moved shape's bounding box lines up with
the edges, corners and nodes of other shapes and whole groups. X and Y line up on their own, so a
shape far to the side can still sit at the same height as another. A dashed line through the caught
edge shows the alignment.</li>
<li><strong>Snap to centre</strong> – the centre of the shape lines up on the centre of another
shape or group.</li>
<li><strong>Snap to equal gap</strong> – the shape takes the same gap to a neighbour as another pair
of shapes has between them (equal spacing). The matching distance is drawn as a labelled arrow, both
for the gap you have chosen and for the reference gap it matched.</li>
</ul>
<p>Only the elements and groups on the sheet are snap anchors – the workpiece itself is not.</p>
<h2>While drawing</h2>
<p>With a drawing tool the pointer snaps to the <strong>corners and centres</strong> of existing
shapes and, separately, to the <strong>grid</strong>. A small marker and a hint beside the pointer
name what it caught.</p>
<h2>Settings</h2>
<p>The three snap buttons sit in the <strong>Snapping</strong> group; <strong>Snap to grid</strong>
is in the <strong>Decoration</strong> group. Hold <kbd>⇧</kbd> while dragging and nothing snaps. The
grid snaps with its spacing in millimetres and the angle step while drawing a line goes from 0° up
to 90°, exactly as in the original program.</p>
<div class="pilot-box"><p><strong>Pilot feature.</strong> The three snap methods and their guide
lines are new in CncCraft and still being refined; please report anything that looks off.</p></div>
<div class="neu-box"><p>The grid is only visible while <strong>Snap to grid</strong> is on – what you
see is what snaps. In addition, moved shapes snap to [[hilfslinien|construction lines]]: with an edge
or their centre, and the line colours the shape red while it would snap.</p></div>
""",
        "body": """
<p>Beim Verschieben einer Form richtet CncCraft sie mit drei Einrast-Methoden an den anderen Formen
und Gruppen aus. Jede hat einen eigenen Schalter in der Kopfleiste, in der Gruppe
<strong>Einrasten</strong>, und lässt sich einzeln oder gemeinsam ein- und ausschalten:</p>
<ul>
<li><strong>An Geometrie einrasten (Bounding-Box)</strong> – die Bounding-Box der Form richtet sich
an Kanten, Ecken und Knoten anderer Formen und ganzer Gruppen aus. X und Y rasten getrennt ein, so
kann eine weit entfernte Form trotzdem auf gleicher Höhe wie eine andere sitzen. Eine gestrichelte
Linie durch die eingerastete Kante zeigt die Ausrichtung.</li>
<li><strong>An Mitte einrasten</strong> – die Mitte der Form rastet auf die Mitte einer anderen Form
oder Gruppe.</li>
<li><strong>An gleichen Abstand einrasten</strong> – die Form nimmt denselben Abstand zu einem
Nachbarn an, den zwei andere Formen zueinander haben (gleicher Abstand). Der passende Abstand wird
als beschrifteter Pfeil gezeichnet – sowohl der gewählte Abstand als auch der Referenzabstand, an
den er angeglichen wurde.</li>
</ul>
<p>Einrastanker sind nur die Elemente und Gruppen auf dem Blatt – das Werkstück selbst nicht.</p>
<h2>Beim Zeichnen</h2>
<p>Mit einem Zeichenwerkzeug rastet der Zeiger an den <strong>Ecken und Mitten</strong> bestehender
Formen und, getrennt davon, am <strong>Raster</strong> ein. Eine kleine Markierung und ein Hinweis
neben dem Zeiger nennen, woran er gerade einrastet.</p>
<h2>Einstellen</h2>
<p>Die drei Einrast-Schalter liegen in der Gruppe <strong>Einrasten</strong>; <strong>Am Raster
einrasten</strong> liegt in der Gruppe <strong>Dekoration</strong>. Mit gedrückter <kbd>⇧</kbd>-Taste
beim Ziehen rastet nichts ein. Das Raster rastet mit seinem Abstand in Millimetern, und der
Winkelschritt beim Zeichnen einer Linie reicht von 0° bis 90° – genau wie im Originalprogramm.</p>
<div class="pilot-box"><p><strong>Pilot-Funktion.</strong> Die drei Einrast-Methoden und ihre
Hilfslinien sind neu in CncCraft und werden noch verfeinert; bitte melden, wenn etwas unstimmig
wirkt.</p></div>
<div class="neu-box"><p>Das Raster ist nur zu sehen, solange <strong>Am Raster einrasten</strong> an
ist – was man sieht, rastet auch ein. Zusätzlich rasten verschobene Formen an
[[hilfslinien|Hilfslinien]] ein: mit einer Kante oder ihrer Mitte, und die Linie färbt die Form rot,
solange sie einrasten würde.</p></div>
""",
    },
    {
        "slug": "eigenschaften-masse", "cat": C, "title": "Maße und Eigenschaften",
        "title_en": "Dimensions and properties",
        "lead": "Jede Form hat Maßfelder, in die Sie Zahlen, Formeln oder Variablen eintippen.",
        "lead_en": "Every shape has dimension fields where you type numbers, formulas or variables.",
        "body_en": """
<p>The fields appear in the [[transformieren|Transform objects]] form and – with a shape selected and <kbd>Enter</kbd> pressed – right at the shape.</p>
<table>
<tr><th>Shape</th><th>Fields</th></tr>
<tr><td>Rectangle</td><td>Centre X, Centre Y, Width, Height, Corner radius</td></tr>
<tr><td>Circle</td><td>Centre X, Centre Y, Radius</td></tr>
<tr><td>Ellipse</td><td>Centre X, Centre Y, Width, Height</td></tr>
<tr><td>Polygon</td><td>Centre X, Centre Y, Radius, Corners</td></tr>
<tr><td>Text</td><td>Centre X, Centre Y, Text, Height</td></tr>
<tr><td>Curve (polyline)</td><td>Number of points, closed or open, contours</td></tr>
</table>
<h2>Entry</h2>
<ul>
<li>The decimal mark is the comma or the point.</li>
<li><kbd>Tab</kbd>, <kbd>↑</kbd> and <kbd>↓</kbd> switch between fields; <kbd>Enter</kbd> accepts, <kbd>Esc</kbd> cancels.</li>
<li>The fields do maths: <code>300/2 - 12</code> or <code>breite/2</code> is a valid entry ([[formeln-in-feldern]]).</li>
<li>Values must be greater than zero where a dimension requires it (a radius, for example).</li>
</ul>
""",
        "body": """
<p>Die Felder erscheinen im Formular [[transformieren|Objekte transformieren]] und – bei gewählter Form und gedrückter <kbd>Enter</kbd>-Taste – direkt an der Form.</p>
<table>
<tr><th>Form</th><th>Felder</th></tr>
<tr><td>Rechteck</td><td>Mitte X, Mitte Y, Breite, Höhe, Eckradius</td></tr>
<tr><td>Kreis</td><td>Mitte X, Mitte Y, Radius</td></tr>
<tr><td>Ellipse</td><td>Mitte X, Mitte Y, Breite, Höhe</td></tr>
<tr><td>Polygon</td><td>Mitte X, Mitte Y, Radius, Ecken</td></tr>
<tr><td>Text</td><td>Mitte X, Mitte Y, Text, Höhe</td></tr>
<tr><td>Kurve (Polylinie)</td><td>Punktzahl, geschlossen oder offen, Konturen</td></tr>
</table>
<h2>Eingabe</h2>
<ul>
<li>Als Dezimalzeichen gilt das Komma oder der Punkt.</li>
<li><kbd>Tab</kbd>, <kbd>↑</kbd> und <kbd>↓</kbd> wechseln zwischen den Feldern; <kbd>Enter</kbd> übernimmt, <kbd>Esc</kbd> bricht ab.</li>
<li>Die Felder rechnen: <code>300/2 - 12</code> oder <code>breite/2</code> ist eine gültige Eingabe ([[formeln-in-feldern]]).</li>
<li>Werte müssen größer als null sein, wo ein Maß das verlangt (zum Beispiel ein Radius).</li>
</ul>
""",
    },
    {
        "slug": "spline", "cat": C, "title": "Spline zeichnen",
        "title_en": "Draw a spline",
        "lead": "Eine weiche Kurve durch die Punkte, die Sie anklicken.",
        "lead_en": "A smooth curve through the points you click.",
        "neu": True,
        "body_en": """
<p>The <strong>Spline</strong> tool (key <kbd>P</kbd>) lays a smooth curve through every point you click. A double-click or <kbd>Enter</kbd> finishes it; a click on the first point closes it into a closed shape that can be cleared as a [[taschen-werkzeugweg|pocket]], for example.</p>
<p>The points can then be moved with [[knotenbearbeitung|node editing]]; the curve stays smooth.</p>
""",
        "body": """
<p>Das Werkzeug <strong>Spline</strong> (Taste <kbd>P</kbd>) legt eine glatte Kurve durch jeden Punkt, den Sie anklicken. Ein Doppelklick oder <kbd>Enter</kbd> beendet sie; ein Klick auf den ersten Punkt schließt sie zu einer geschlossenen Form, die sich zum Beispiel als [[taschen-werkzeugweg|Tasche]] ausräumen lässt.</p>
<p>Die Punkte lassen sich danach mit der [[knotenbearbeitung]] verschieben; die Kurve bleibt dabei weich.</p>
""",
    },
    {
        "slug": "vektoren-trennen-verbinden", "cat": C, "title": "Vektoren trennen, löschen und verbinden",
        "title_en": "Trim, delete and join vectors",
        "lead": "Vektoren an Kreuzungen teilen, einzelne Stücke entfernen und offene Enden verbinden.",
        "lead_en": "Split vectors at crossings, remove individual pieces and join open ends.",
        "body_en": """
<table>
<tr><th>Tool</th><th>How it works</th></tr>
<tr><td><strong>Trim</strong> (<kbd>T</kbd>)</td><td>Click a vector: it is split at every crossing with other vectors.</td></tr>
<tr><td><strong>Delete vector</strong></td><td>Click a vector or a trimmed piece: it is removed. So you can take away overhanging ends after trimming.</td></tr>
<tr><td><strong>Join</strong></td><td>Click two points one after another: they are joined with a line. Click both ends of a polyline and it closes.</td></tr>
</table>
<div class="tipp"><p>You need closed vectors for [[taschen-werkzeugweg|pockets]] and [[ausschnitt-werkzeugweg|cut-outs]] with an inside and outside; open lines suit [[profil-werkzeugweg|profiles]] and [[nut-werkzeugweg|slots]].</p></div>
""",
        "body": """
<table>
<tr><th>Werkzeug</th><th>So geht es</th></tr>
<tr><td><strong>Trennen</strong> (<kbd>T</kbd>)</td><td>Einen Vektor anklicken: Er wird an jeder Kreuzung mit anderen Vektoren geteilt.</td></tr>
<tr><td><strong>Vektor löschen</strong></td><td>Einen Vektor oder ein getrenntes Stück anklicken: Es wird entfernt. So lassen sich überstehende Enden nach dem Trennen wegnehmen.</td></tr>
<tr><td><strong>Verbinden</strong></td><td>Zwei Punkte nacheinander anklicken: Sie werden mit einer Linie verbunden. Klicken Sie beide Enden eines Linienzugs an, wird er geschlossen.</td></tr>
</table>
<div class="tipp"><p>Geschlossene Vektoren brauchen Sie für [[taschen-werkzeugweg|Taschen]] und [[ausschnitt-werkzeugweg|Ausschnitte]] mit Innen- und Außenseite; offene Linien eignen sich für [[profil-werkzeugweg|Profile]] und [[nut-werkzeugweg|Nuten]].</p></div>
""",
    },
    {
        "slug": "ecken-abrunden", "cat": C, "title": "Ecken abrunden und Ohren",
        "title_en": "Round corners and ears",
        "lead": "Ecken mit einem Radius abrunden oder als „Ohr“ ausfräsen, damit Teile ineinander passen.",
        "lead_en": "Round corners with a radius or cut an “ear” so parts fit into one another.",
        "body_en": """
<p>A round cutter cannot mill a sharp inside corner. Two tools handle this; you enter the radius in the form on the left.</p>
<table>
<tr><th>Tool</th><th>Effect</th></tr>
<tr><td><strong>Round</strong></td><td>Click a corner: it is rounded with the radius.</td></tr>
<tr><td><strong>Ear</strong></td><td>Click a corner: an arc is added around the corner point (dog-bone) so the cutter reaches fully into the corner and a square counterpart fits in.</td></tr>
</table>
<h2>Changing the radius later</h2>
<p>At every rounded corner a label shows the radius, for example <code>R5</code>. <strong>Long-press</strong> opens it for typing – like any dimension field with [[formeln-in-feldern|formulas]] and [[variablen|variables]] (it then shows <code>R ƒfraeser/2</code>, for example). Clear the field and press <kbd>Enter</kbd> and the rounding disappears again.</p>
""",
        "body": """
<p>Ein runder Fräser kann keine scharfe Innenecke fräsen. Dafür gibt es zwei Werkzeuge; den Radius tragen Sie im Formular links ein.</p>
<table>
<tr><th>Werkzeug</th><th>Wirkung</th></tr>
<tr><td><strong>Abrunden</strong></td><td>Eine Ecke anklicken: Sie wird mit dem Radius gerundet.</td></tr>
<tr><td><strong>Ohr</strong></td><td>Eine Ecke anklicken: Um den Eckpunkt kommt ein Kreisbogen (Dog-Bone), damit der Fräser ganz in die Ecke reicht und ein eckiges Gegenstück hineinpasst.</td></tr>
</table>
<h2>Radius später ändern</h2>
<p>An jeder gerundeten Ecke steht ein Schild mit dem Radius, zum Beispiel <code>R5</code>. <strong>Lange drücken</strong> öffnet es zum Eintippen – wie jedes Maßfeld mit [[formeln-in-feldern|Formeln]] und [[variablen|Variablen]] (dann steht dort zum Beispiel <code>R ƒfraeser/2</code>). Leeren Sie das Feld und drücken Sie <kbd>Enter</kbd>, verschwindet die Rundung wieder.</p>
""",
    },
    {
        "slug": "vervielfaeltigen", "cat": C, "title": "Vervielfältigen (Muster)",
        "title_en": "Pattern (array)",
        "lead": "Aus einer Form ein rechteckiges Muster aus verbundenen Kopien machen – die mit der Form lebt.",
        "lead_en": "Turn one shape into a rectangular pattern of linked copies that stays alive with the shape.",
        "neu": True,
        "body_en": """
<p>The <strong>Pattern</strong> tool (between Offset and the vector operations) turns <em>one</em> selected shape into a rectangular <strong>pattern</strong>. Unlike a plain copy, the pattern stays live: the copies (the “children”) hang on the source shape and follow it.</p>
<p>Select a shape, open the tool and fill in the window:</p>
<table>
<tr><th>Field</th><th>Meaning</th></tr>
<tr><td>Axis 1 (X) · Quantity</td><td>How many shapes side by side (the source counted in).</td></tr>
<tr><td>Axis 1 (X) · Distance</td><td>Centre-to-centre distance across the width.</td></tr>
<tr><td>Axis 2 (Y) · Quantity</td><td>How many above one another.</td></tr>
<tr><td>Axis 2 (Y) · Distance</td><td>Centre-to-centre distance over the height.</td></tr>
</table>
{{img:vervielfaeltigen.png|The “Rectangular pattern” window with the preview “6 copies (3 × 2)”}}
<h2>What makes a pattern special</h2>
<ul>
<li>The source gets a small <strong>pattern marker</strong> (four dots). A <strong>double-click</strong> on it reopens the window for changes.</li>
<li>The children are drawn semi-transparent (60 %), as shadows of the source.</li>
<li>The children are firmly tied to the source: they can only be selected together, not one by one.</li>
<li><strong>Move</strong> the source and the whole pattern moves. <strong>Delete</strong> the source and the children go too.</li>
<li>A [[werkzeugwege-uebersicht|toolpath]] or any other tool always applies to the source and acts on every child. Children have no toolpaths of their own.</li>
</ul>
<h2>Dissolving a pattern</h2>
<p>With <em>Ungroup</em> (<kbd>⇧⌘G</kbd>) you dissolve the pattern: from then on each copy stands on its own, gets the source's toolpaths copied onto it and can be edited individually.</p>
<div class="neu-box"><p>Quantity and distances take numbers, [[variablen|variables]] and [[formeln-in-feldern|formulas]]. For example <code>platte_b / 50</code> columns fill the board whatever its width, and when a variable changes the pattern rebuilds – all in the spirit of [[parametrisch-uebersicht|parametric]] work. Every step can be [[rueckgaengig-wiederholen|undone]].</p></div>
""",
        "body": """
<p>Das Werkzeug <strong>Vervielfältigen</strong> (zwischen Versatz und den Vektoroperationen) macht aus <em>einer</em> gewählten Form ein rechteckiges <strong>Muster</strong>. Anders als beim bloßen Kopieren bleibt das Muster lebendig: Die Kopien (die „Kinder“) hängen an der Urform und folgen ihr.</p>
<p>Wählen Sie eine Form, öffnen Sie das Werkzeug und tragen Sie im Fenster ein:</p>
<table>
<tr><th>Feld</th><th>Bedeutung</th></tr>
<tr><td>Achse 1 (X) · Anzahl</td><td>Wie viele Formen nebeneinander (die Urform mitgezählt).</td></tr>
<tr><td>Achse 1 (X) · Abstand</td><td>Abstand von Mitte zu Mitte in der Breite.</td></tr>
<tr><td>Achse 2 (Y) · Anzahl</td><td>Wie viele übereinander.</td></tr>
<tr><td>Achse 2 (Y) · Abstand</td><td>Abstand von Mitte zu Mitte in der Höhe.</td></tr>
</table>
{{img:vervielfaeltigen.png|Das Fenster „Rechteckiges Muster“ mit der Vorschau „6 Kopien (3 × 2)“}}
<h2>Was ein Muster besonders macht</h2>
<ul>
<li>Die Urform bekommt ein kleines <strong>Muster-Zeichen</strong> (vier Punkte). Ein <strong>Doppelklick</strong> darauf öffnet das Fenster wieder zum Ändern.</li>
<li>Die Kinder werden halb durchscheinend (60 %) gezeichnet, als Schatten der Urform.</li>
<li>Die Kinder sind fest mit der Urform verbunden: Sie lassen sich nur zusammen auswählen, nicht einzeln.</li>
<li><strong>Bewegen</strong> Sie die Urform, bewegt sich das ganze Muster. <strong>Löschen</strong> Sie die Urform, verschwinden auch die Kinder.</li>
<li>Ein [[werkzeugwege-uebersicht|Werkzeugweg]] oder ein anderes Werkzeug gilt immer der Urform und wirkt auf alle Kinder. Kinder haben keine eigenen Werkzeugwege.</li>
</ul>
<h2>Muster auflösen</h2>
<p>Mit <em>Gruppe auflösen</em> (<kbd>⇧⌘G</kbd>) lösen Sie das Muster: Ab dann steht jede Kopie für sich, bekommt die Werkzeugwege der Urform als eigene Kopie und lässt sich einzeln bearbeiten.</p>
<div class="neu-box"><p>Anzahl und Abstände nehmen Zahlen, [[variablen|Variablen]] und [[formeln-in-feldern|Formeln]]. So füllt zum Beispiel <code>platte_b / 50</code> Spalten die Platte, egal wie breit sie ist, und ändert sich eine Variable, baut sich das Muster neu – ganz im Sinne des [[parametrisch-uebersicht|parametrischen]] Arbeitens. Jeder Schritt lässt sich [[rueckgaengig-wiederholen|rückgängig]] machen.</p></div>
""",
    },
    {
        "slug": "gruppieren", "cat": C, "title": "Gruppieren",
        "title_en": "Grouping",
        "lead": "Mehrere Formen zu einer Gruppe zusammenfassen, die sich wie eine Form auswählen lässt.",
        "lead_en": "Combine several shapes into a group that selects like a single shape.",
        "body_en": """
<p><strong>Group</strong> (<kbd>⌘G</kbd>) combines the selection: a click on any part then selects the whole group. Groups can themselves be grouped.</p>
<p><strong>Ungroup</strong> (<kbd>⇧⌘G</kbd>) dissolves the outermost group of the selection, one level per click. Copies of a group form their own new group ([[kopieren-einfuegen]]).</p>
""",
        "body": """
<p><strong>Gruppieren</strong> (<kbd>⌘G</kbd>) fasst die Auswahl zusammen: Ein Klick auf irgendein Teil wählt danach die ganze Gruppe. Gruppen lassen sich selbst wieder gruppieren.</p>
<p><strong>Gruppe auflösen</strong> (<kbd>⇧⌘G</kbd>) löst die äußerste Gruppe der Auswahl auf, eine Stufe je Klick. Kopien einer Gruppe bilden eine eigene, neue Gruppe ([[kopieren-einfuegen]]).</p>
""",
    },
    {
        "slug": "dxf-importieren", "cat": C, "title": "DXF importieren",
        "title_en": "Import DXF",
        "lead": "Zeichnungen aus anderen Programmen als DXF-Datei übernehmen.",
        "lead_en": "Bring in drawings from other programs as a DXF file.",
        "body_en": """
<p>The <strong>Import DXF</strong> tool reads a DXF file (R12, R14 and newer) and places its contents as <strong>one group</strong> in the drawing ([[gruppieren]]). Lines, arcs, circles and polylines become vectors you edit and give toolpaths like ones you drew yourself.</p>
<p>If the file changes later in the other program, a <strong>right-click on the shape</strong> reloads it from disk.</p>
""",
        "body": """
<p>Das Werkzeug <strong>DXF importieren</strong> liest eine DXF-Datei (R12, R14 und neuer) und legt ihren Inhalt als <strong>eine Gruppe</strong> in die Zeichnung ([[gruppieren]]). Linien, Bögen, Kreise und Polylinien werden zu Vektoren, die Sie wie selbst gezeichnete bearbeiten und mit Werkzeugwegen versehen.</p>
<p>Ändert sich die Datei später im anderen Programm, lädt ein <strong>Rechtsklick auf die Form</strong> sie neu von der Platte.</p>
""",
    },
    {
        "slug": "bild-als-vorlage", "cat": C, "title": "Bild als Vorlage",
        "title_en": "Image as a template",
        "lead": "Ein Foto oder eine Skizze unter die Zeichnung legen, auf das richtige Maß bringen und nachzeichnen.",
        "lead_en": "Place a photo or sketch under the drawing, bring it to the right scale and trace it.",
        "neu": True,
        "body_en": """
<p><strong>Insert image</strong> loads an image (PNG, JPEG) onto an image layer ([[ebene-neu]]). It sits translucent under the drawing; set the opacity in the [[ebenenliste|layer list]].</p>
<h2>Calibrating</h2>
<p>To make the image to scale, click two points in the image with <strong>Calibrate image</strong> whose real distance you know (such as the ends of a ruler in the photo), and type the distance. CncCraft scales the image so it matches the board 1:1 – after that you can trace on it to exact size.</p>
""",
        "body": """
<p><strong>Bild einfügen</strong> lädt ein Bild (PNG, JPEG) auf eine Bild-Ebene ([[ebene-neu]]). Es liegt durchscheinend unter der Zeichnung; die Deckkraft stellen Sie in der [[ebenenliste]] ein.</p>
<h2>Kalibrieren</h2>
<p>Damit das Bild maßstäblich ist, klicken Sie mit <strong>Bild kalibrieren</strong> zwei Punkte im Bild an, deren echten Abstand Sie kennen (etwa die Enden eines mitfotografierten Lineals), und tippen den Abstand ein. CncCraft skaliert das Bild so, dass es 1:1 zur Platte passt – danach lässt sich darauf maßgenau nachzeichnen.</p>
""",
    },
]
