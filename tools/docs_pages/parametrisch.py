"""Kategorie „Parametrisch entwerfen“ – alles hier ist neu gegenüber der Vorlage Spark."""

C = "parametrisch"

PAGES = [
    {
        "slug": "parametrisch-uebersicht", "cat": C, "title": "Parametrisch entwerfen – Überblick",
        "title_en": "Parametric design – overview",
        "lead": "Parametrisches CAD für Einsteiger: Maße müssen nicht fest sein – Sie benennen, berechnen und verbinden sie, und die Zeichnung folgt.",
        "lead_en": "Parametric CAD for beginners: dimensions need not be fixed – you name, compute and connect them, and the drawing follows.",
        "neu": True,
        "body_en": """
<div class="neu-box"><p>The original Spark has neither variables nor constraints. In CncCraft a changed variable changes the drawing – all the way to the G-code, because the toolpaths recompute too.</p></div>
<p>Working parametrically means you record <em>what</em> a dimension should mean (“10 mm from the edge”, “half the board's width”) instead of just drawing it once. Change a value later and CncCraft recomputes everything. There are five building blocks that combine freely – none needs prior knowledge from professional CAD programs:</p>
<table>
<tr><th>Building block</th><th>What for</th></tr>
<tr><td>[[variablen|Variables]]</td><td>Named values used in formulas, for example <code>rand = 12</code>.</td></tr>
<tr><td>[[formeln-in-feldern|Formulas in dimension fields]]</td><td>Every dimension field and depth does maths: <code>platte_b - 2*rand</code>.</td></tr>
<tr><td>[[abstand-bemassen|Dimension distances]]</td><td>The distance between two points, a point and a line, or to the board edge stays as it is.</td></tr>
<tr><td>[[waagrecht-senkrecht|Level/upright]]</td><td>A line stays level or upright.</td></tr>
<tr><td>[[hilfslinien|Construction lines]]</td><td>Lines from the ruler that shapes snap to and hang on: move the line – or change its formula – and the shapes move along.</td></tr>
</table>
{{img:hilfslinien.png|Construction lines on the ruler: one follows a formula, the shapes hang on them}}
<p>An example: you draw a sign and set <code>rand</code>. The outline gets the width <code>platte_b - 2*rand</code> and the height <code>platte_l - 2*rand</code>. Change the board size in the job later, or the value of <code>rand</code>, and the outline adapts, and the toolpaths follow.</p>
{{img:variablen.png|Variables and bound dimensions working together}}
<h2>Limits</h2>
<ul>
<li>CncCraft solves the constraints directly, without a general equation system: a shape that holds a constraint springs back when dragged.</li>
<li>Moving with the mouse, node editing and boolean operations create new geometry without bindings; the status line and the behaviour point this out.</li>
<li>Every step can be [[rueckgaengig-wiederholen|undone]] – trying things out costs nothing.</li>
</ul>
""",
        "body": """
<div class="neu-box"><p>Das Original Spark kennt weder Variablen noch Bedingungen. In CncCraft ändert eine geänderte Variable die Zeichnung – bis hin zum G-Code, denn auch die Werkzeugwege werden neu berechnet.</p></div>
<p>Parametrisch zu arbeiten heißt: Sie halten fest, <em>was</em> ein Maß bedeuten soll („10 mm vom Rand“, „halb so breit wie die Platte“), statt es nur einmal hinzuzeichnen. Ändern Sie später einen Wert, rechnet CncCraft alles nach. Dafür gibt es fünf Bausteine, die sich kombinieren lassen – keiner verlangt Vorwissen aus Profi-CAD-Programmen:</p>
<table>
<tr><th>Baustein</th><th>Wofür</th></tr>
<tr><td>[[variablen|Variablen]]</td><td>Benannte Werte, die in Formeln benutzt werden, zum Beispiel <code>rand = 12</code>.</td></tr>
<tr><td>[[formeln-in-feldern|Formeln in Maßfeldern]]</td><td>Jedes Maßfeld und jede Tiefe rechnet: <code>platte_b - 2*rand</code>.</td></tr>
<tr><td>[[abstand-bemassen|Abstände bemaßen]]</td><td>Der Abstand zwischen zwei Punkten, einem Punkt und einer Linie oder zur Plattenkante bleibt, wie er ist.</td></tr>
<tr><td>[[waagrecht-senkrecht|Waagrecht/Senkrecht]]</td><td>Eine Linie bleibt waagrecht oder senkrecht.</td></tr>
<tr><td>[[hilfslinien|Hilfslinien]]</td><td>Linien aus dem Lineal, an denen Formen einrasten und hängen bleiben: Verschieben Sie die Linie – oder ändert sich ihre Formel –, wandern die Formen mit.</td></tr>
</table>
{{img:hilfslinien.png|Hilfslinien am Lineal: eine folgt einer Formel, die Formen hängen an ihnen}}
<p>Ein Beispiel: Sie zeichnen ein Schild und legen <code>rand</code> fest. Die Kontur bekommt die Breite <code>platte_b - 2*rand</code> und die Höhe <code>platte_l - 2*rand</code>. Ändern Sie später die Plattengröße im Job oder den Wert von <code>rand</code>, passt sich die Kontur an, und die Werkzeugwege folgen.</p>
{{img:variablen.png|Variablen und gebundene Maße im Zusammenspiel}}
<h2>Grenzen</h2>
<ul>
<li>CncCraft löst die Bedingungen direkt, ohne ein allgemeines Gleichungssystem: Eine Form, die eine Bedingung festhält, springt beim Ziehen zurück.</li>
<li>Verschieben mit der Maus, Knotenbearbeitung und Boolesche Operationen erzeugen neue Geometrie ohne Bindungen; die Statuszeile und die Bedienung weisen darauf hin.</li>
<li>Jeder Schritt lässt sich [[rueckgaengig-wiederholen|rückgängig machen]] – Ausprobieren kostet nichts.</li>
</ul>
""",
    },
    {
        "slug": "variablen", "cat": C, "title": "Variablen",
        "title_en": "Variables",
        "lead": "Variablen sind benannte Zahlen oder Formeln, die Sie in allen Maßfeldern verwenden.",
        "lead_en": "Variables are named numbers or formulas you can use in every dimension field.",
        "neu": True,
        "body_en": """
<p>The <strong>f(x)</strong> button next to “Set up job” in the toolbar opens the variables dialog.</p>
{{img:variablen.png|The variables dialog}}
<h2>Your own variables</h2>
<p>With <strong>New variable</strong> you add a row. Every variable has:</p>
<table>
<tr><th>Column</th><th>Meaning</th></tr>
<tr><td>Name</td><td>The name you use the variable by. It must not be empty, must not start with an underscore and must not occur twice.</td></tr>
<tr><td>Expression</td><td>A number or formula, also using other variables (<code>breite/2 - rand</code>).</td></tr>
<tr><td>Value</td><td>The computed result – or the error message if the formula does not work out.</td></tr>
<tr><td>Comment</td><td>A note for yourself.</td></tr>
</table>
<p>Change an expression and all dependent variables, shapes and toolpaths recompute. Variables may refer to other variables; a circular reference (<code>a = b + 1</code>, <code>b = a</code>) is reported as an error instead of looping forever. If a formula uses a name that does not exist, CncCraft reports “Unknown variable”.</p>
<h2>From the job</h2>
<p>The job always provides three values:</p>
<table>
<tr><th>Name</th><th>Meaning</th></tr>
<tr><td><code>platte_b</code></td><td>Board width (X)</td></tr>
<tr><td><code>platte_l</code></td><td>Board length (Y)</td></tr>
<tr><td><code>platte_d</code></td><td>Board thickness (Z)</td></tr>
</table>
<p>These names are taken; your own variables must not use them.</p>
<h2>Using variables</h2>
<p>Type the name into a dimension field, a constraint or a depth in a toolpath. As you type, a suggestion list of matching variables and their value appears under the field (<kbd>↑</kbd> <kbd>↓</kbd> to choose, <kbd>Tab</kbd> or <kbd>Enter</kbd> to accept). More under [[formeln-in-feldern]].</p>
<div class="tipp"><p>Names with spaces or special characters go in curly braces: <code>{Rand links}</code>.</p></div>
""",
        "body": """
<p>Der Knopf <strong>f(x)</strong> neben „Job einrichten“ in der Werkzeugleiste öffnet den Variablen-Dialog.</p>
{{img:variablen.png|Der Variablen-Dialog}}
<h2>Eigene Variablen</h2>
<p>Mit <strong>Neue Variable</strong> legen Sie eine Zeile an. Jede Variable hat:</p>
<table>
<tr><th>Spalte</th><th>Bedeutung</th></tr>
<tr><td>Name</td><td>Der Name, mit dem Sie die Variable verwenden. Er darf nicht leer sein, nicht mit einem Unterstrich beginnen und nicht doppelt vorkommen.</td></tr>
<tr><td>Ausdruck</td><td>Eine Zahl oder Formel, auch mit anderen Variablen (<code>breite/2 - rand</code>).</td></tr>
<tr><td>Wert</td><td>Das ausgerechnete Ergebnis – oder die Fehlermeldung, wenn die Formel nicht aufgeht.</td></tr>
<tr><td>Kommentar</td><td>Eine Notiz für Sie selbst.</td></tr>
</table>
<p>Ändern Sie einen Ausdruck, rechnen alle abhängigen Variablen, Formen und Werkzeugwege neu. Variablen dürfen sich auf andere Variablen beziehen; ein Kreisverweis (<code>a = b + 1</code>, <code>b = a</code>) wird als Fehler gemeldet statt endlos zu rechnen. Kommt in einer Formel ein Name vor, den es nicht gibt, meldet CncCraft „Unbekannte Variable“.</p>
<h2>Aus dem Job</h2>
<p>Drei Werte stellt der Job immer bereit:</p>
<table>
<tr><th>Name</th><th>Bedeutung</th></tr>
<tr><td><code>platte_b</code></td><td>Breite der Platte (X)</td></tr>
<tr><td><code>platte_l</code></td><td>Länge der Platte (Y)</td></tr>
<tr><td><code>platte_d</code></td><td>Dicke der Platte (Z)</td></tr>
</table>
<p>Diese Namen sind vergeben; eigene Variablen dürfen nicht so heißen.</p>
<h2>Variablen benutzen</h2>
<p>Tippen Sie den Namen in ein Maßfeld, ein Constraint oder eine Tiefe im Werkzeugweg ein. Während der Eingabe erscheint unter dem Feld eine Vorschlagsliste mit passenden Variablen und ihrem Wert (<kbd>↑</kbd> <kbd>↓</kbd> zum Wählen, <kbd>Tab</kbd> oder <kbd>Enter</kbd> zum Übernehmen). Mehr dazu unter [[formeln-in-feldern]].</p>
<div class="tipp"><p>Namen mit Leerzeichen oder Sonderzeichen schreiben Sie in geschweiften Klammern: <code>{Rand links}</code>.</p></div>
""",
    },
    {
        "slug": "formeln-in-feldern", "cat": C, "title": "Formeln in Maßfeldern",
        "title_en": "Formulas in dimension fields",
        "lead": "Jedes Zahlenfeld rechnet mit: Grundrechenarten, Klammern, Potenzen und Variablen.",
        "lead_en": "Every number field does the maths: arithmetic, brackets, powers and variables.",
        "neu": True,
        "body_en": """
<p>In every dimension field – width, height, radius, position, constraint values and the start and cut depths of toolpaths – you can type a formula instead of a number.</p>
<h2>What is allowed</h2>
<table>
<tr><th>Notation</th><th>Meaning</th></tr>
<tr><td><code>+</code> <code>-</code></td><td>Add, subtract</td></tr>
<tr><td><code>*</code> <code>×</code> <code>·</code></td><td>Multiply (the letter <code>x</code> is not a times sign, so it can be a name)</td></tr>
<tr><td><code>/</code> <code>÷</code></td><td>Divide</td></tr>
<tr><td><code>^</code></td><td>Power (<code>2^3</code> = 8; <code>2^3^2</code> is evaluated from the right)</td></tr>
<tr><td><code>( )</code></td><td>Brackets</td></tr>
<tr><td><code>12,5</code> or <code>12.5</code></td><td>Decimal (comma or point)</td></tr>
<tr><td><code>breite</code>, <code>{Rand links}</code></td><td>Variable ([[variablen]])</td></tr>
</table>
<p>Multiplication before addition applies as usual. A <kbd>=</kbd> at the end evaluates the formula at once and replaces it with the number.</p>
<h2>Bound dimensions</h2>
<p>If a dimension field holds a formula with variables, the dimension is <em>bound</em>: change the variable and the shape adapts. In that case:</p>
<ul>
<li>The lower-left corner of the shape stays fixed; the top or right edge moves.</li>
<li>The handles of a bound direction are locked (blue diamonds). Drag the free direction and only it changes.</li>
<li>Change the value by hand in the form, or move nodes, and the binding is released.</li>
</ul>
<p>Typing a width or height on a rectangle or ellipse is technically the same as a [[abstand-bemassen|distance constraint]] from the lower-left corner: the side is then fixed and drawn <strong>black</strong>; free sides are <strong>blue</strong>.</p>
<h2>Errors</h2>
<p>If a formula is invalid or uses an unknown variable, the field turns red and names the reason (“Unknown variable: …”). While the field is invalid, <kbd>Enter</kbd> does not close it – <kbd>Esc</kbd> discards the entry.</p>
{{img:tiefe.png|A depth with a formula: “platte_d/2” gives 9 mm at 18 mm thickness; a typo in the name is reported in red}}
<div class="tipp"><p>With the formula <code>platte_d</code> as the cut depth, a cut-out always cuts exactly through the material – even if you change the thickness in the job later.</p></div>
""",
        "body": """
<p>In allen Maßfeldern – Breite, Höhe, Radius, Position, Constraint-Werte sowie Start- und Schnitttiefe der Werkzeugwege – können Sie statt einer Zahl eine Formel eintippen.</p>
<h2>Was erlaubt ist</h2>
<table>
<tr><th>Schreibweise</th><th>Bedeutung</th></tr>
<tr><td><code>+</code> <code>-</code></td><td>Addieren, Subtrahieren</td></tr>
<tr><td><code>*</code> <code>×</code> <code>·</code></td><td>Malnehmen (der Buchstabe <code>x</code> ist kein Malzeichen, damit er als Name taugt)</td></tr>
<tr><td><code>/</code> <code>÷</code></td><td>Teilen</td></tr>
<tr><td><code>^</code></td><td>Potenz (<code>2^3</code> = 8; <code>2^3^2</code> wird von rechts gerechnet)</td></tr>
<tr><td><code>( )</code></td><td>Klammern</td></tr>
<tr><td><code>12,5</code> oder <code>12.5</code></td><td>Dezimalzahl (Komma oder Punkt)</td></tr>
<tr><td><code>breite</code>, <code>{Rand links}</code></td><td>Variable ([[variablen]])</td></tr>
</table>
<p>Punkt vor Strich gilt wie üblich. Ein <kbd>=</kbd> am Ende rechnet die Formel sofort aus und ersetzt sie durch die Zahl.</p>
<h2>Gebundene Maße</h2>
<p>Enthält ein Maßfeld eine Formel mit Variablen, ist das Maß <em>gebunden</em>: Ändert sich die Variable, passt sich die Form an. Dabei gilt:</p>
<ul>
<li>Die untere linke Ecke der Form bleibt fest; die obere beziehungsweise rechte Kante wandert.</li>
<li>Die Griffe einer gebundenen Richtung sind gesperrt (blaue Rauten). Ziehen Sie an der freien Richtung, ändert sich nur sie.</li>
<li>Ändern Sie den Wert dagegen von Hand im Formular oder verschieben Knoten, löst sich die Bindung.</li>
</ul>
<p>Tippen Sie bei einem Rechteck oder einer Ellipse eine Breite oder Höhe ein, ist das technisch dasselbe wie eine [[abstand-bemassen|Abstands-Bedingung]] von der linken unteren Ecke: Die Seite ist danach festgelegt und in der Zeichnung <strong>schwarz</strong>; freie Seiten sind <strong>blau</strong>.</p>
<h2>Fehler</h2>
<p>Ist eine Formel ungültig oder kennt sie eine Variable nicht, wird das Feld rot und nennt den Grund („Unbekannte Variable: …“). Solange das Feld ungültig ist, schließt <kbd>Enter</kbd> es nicht – <kbd>Esc</kbd> verwirft die Eingabe.</p>
{{img:tiefe.png|Eine Tiefe mit Formel: „platte_d/2“ ergibt 9 mm bei 18 mm Dicke; ein Tippfehler im Namen wird rot gemeldet}}
<div class="tipp"><p>Mit der Formel <code>platte_d</code> als Schnitttiefe fräst ein Ausschnitt immer genau durch das Material – auch wenn Sie die Dicke im Job später ändern.</p></div>
""",
    },
    {
        "slug": "abstand-bemassen", "cat": C, "title": "Abstände bemaßen",
        "title_en": "Dimension distances",
        "lead": "Eine Abstands-Bedingung hält zwei Dinge in einem festen Abstand – auch wenn Sie das eine verschieben oder Variablen ändern.",
        "lead_en": "A distance constraint keeps two things a fixed distance apart – even when you move one or change variables.",
        "neu": True,
        "body_en": """
<p>The <strong>Distance</strong> button in the “Constraints” group (key <kbd>D</kbd>) turns on the dimensioning mode.</p>
{{img:abstand.png|A distance being dimensioned}}
<h2>How to do it</h2>
<ol>
<li>Press <kbd>D</kbd> or click <strong>Distance</strong>.</li>
<li>Pick the first object: a corner, the centre or a node of a shape, a line – or a corner or edge of the <em>board</em>.</li>
<li>Pick the second object.</li>
<li>Move the pointer to where the dimension should sit and click. Beside the points it measures level or upright, in between straight.</li>
<li>Type the distance – a number, a formula or a variable – and press <kbd>Enter</kbd>.</li>
</ol>
<p><kbd>Esc</kbd> takes back the last pick; another <kbd>Esc</kbd> ends the mode.</p>
<h2>What can be measured</h2>
<ul>
<li><strong>Point and point:</strong> level, upright or straight distance.</li>
<li><strong>Point and line:</strong> the perpendicular distance.</li>
<li><strong>A single line:</strong> its length – then click in the drawing to place the dimension.</li>
<li><strong>Two lines</strong> cannot be dimensioned (“Two lines cannot be dimensioned: pick a point”).</li>
</ul>
<h2>Which one moves?</h2>
<p>The shape of the object picked <em>first</em> is the reference and stays put; the second shape moves until the distance is right. The board is always the reference. On one and the same shape only opposite corners are selectable (that is then width or height; the shape grows from the lower left).</p>
<h2>Editing and deleting dimensions</h2>
<ul>
<li><strong>Change the value:</strong> hold the value in the drawing (about half a second), then type.</li>
<li><strong>Move the dimension line:</strong> drag the dimension. The place is saved with it.</li>
<li><strong>Delete:</strong> right-click the value, or click it briefly and press <kbd>Del</kbd>.</li>
</ul>
<p>Bound dimensions appear blue, formulas as “ƒ <em>formula</em> = <em>value</em> mm”. The <strong>Show dimensions</strong> switch in the header shows or hides them; the mode always shows them. A shape that holds a dimension springs back to its place when dragged.</p>
<div class="neu-box"><p>Constraints are saved in the project and copied with the shape when all shapes involved are copied along. If a shape is deleted, its constraints fall away.</p></div>
""",
        "body": """
<p>Der Knopf <strong>Abstand</strong> in der Gruppe „Constraints“ (Taste <kbd>D</kbd>) schaltet den Bemaßen-Modus ein.</p>
{{img:abstand.png|Ein Abstand wird bemaßt}}
<h2>So gehen Sie vor</h2>
<ol>
<li>Drücken Sie <kbd>D</kbd> oder klicken Sie auf <strong>Abstand</strong>.</li>
<li>Wählen Sie das erste Objekt: eine Ecke, die Mitte oder einen Knoten einer Form, eine Linie – oder eine Ecke oder Kante der <em>Platte</em>.</li>
<li>Wählen Sie das zweite Objekt.</li>
<li>Bewegen Sie den Zeiger dorthin, wo das Maß stehen soll, und klicken Sie. Neben den Punkten wird waagrecht oder senkrecht gemessen, dazwischen gerade.</li>
<li>Tippen Sie den Abstand ein – eine Zahl, eine Formel oder eine Variable – und drücken Sie <kbd>Enter</kbd>.</li>
</ol>
<p>Mit <kbd>Esc</kbd> nehmen Sie die letzte Wahl zurück; ein weiteres <kbd>Esc</kbd> beendet den Modus.</p>
<h2>Was gemessen werden kann</h2>
<ul>
<li><strong>Punkt und Punkt:</strong> waagrechter, senkrechter oder gerader Abstand.</li>
<li><strong>Punkt und Linie:</strong> der Lotabstand.</li>
<li><strong>Eine einzelne Linie:</strong> ihre Länge – klicken Sie danach in die Zeichenfläche, um das Maß zu platzieren.</li>
<li><strong>Zwei Linien</strong> lassen sich nicht bemaßen („Zwei Linien lassen sich nicht bemaßen: einen Punkt wählen“).</li>
</ul>
<h2>Wer bewegt sich?</h2>
<p>Die Form des <em>zuerst</em> gewählten Objekts ist der Bezug und bleibt stehen; die Form des zweiten wird verschoben, bis der Abstand stimmt. Die Platte ist immer Bezug. Auf ein und derselben Form sind nur gegenüberliegende Ecken wählbar (das ist dann Breite oder Höhe; die Form wächst von links unten).</p>
<h2>Maße bearbeiten und löschen</h2>
<ul>
<li><strong>Wert ändern:</strong> den Wert in der Zeichnung gedrückt halten (etwa eine halbe Sekunde), dann tippen.</li>
<li><strong>Maßlinie verschieben:</strong> am Maß ziehen. Der Platz wird mitgespeichert.</li>
<li><strong>Löschen:</strong> Rechtsklick auf den Wert, oder ihn kurz anklicken und <kbd>Entf</kbd> drücken.</li>
</ul>
<p>Gebundene Maße erscheinen blau, bei Formeln als „ƒ <em>Formel</em> = <em>Wert</em> mm“. Mit dem Schalter <strong>Maße anzeigen</strong> in der Kopfleiste blenden Sie sie ein oder aus; der Modus zeigt sie immer. Eine Form, die ein Maß festhält, springt beim Ziehen zurück an ihren Platz.</p>
<div class="neu-box"><p>Bedingungen werden im Projekt gespeichert und mit der Form kopiert, wenn alle beteiligten Formen mitkopiert werden. Wird eine Form gelöscht, fallen ihre Bedingungen weg.</p></div>
""",
    },
    {
        "slug": "waagrecht-senkrecht", "cat": C, "title": "Waagrecht und senkrecht festlegen",
        "title_en": "Set level and upright",
        "lead": "Mit einem Klick bleibt eine Linie waagrecht oder senkrecht – auch wenn Sie an den Knoten ziehen.",
        "lead_en": "With one click a line stays level or upright – even when you drag the nodes.",
        "neu": True,
        "body_en": """
<p>The <strong>Level/upright</strong> button in the “Constraints” group (key <kbd>H</kbd>) turns on the mode.</p>
{{img:ausrichten.png|Level/upright in use}}
<h2>How to use it</h2>
<ul>
<li><strong>Click a line</strong> (a side of a shape or a segment of a polyline): CncCraft sets it level or upright, whichever the line is closer to. On a tie it becomes level.</li>
<li>Or click <strong>two points</strong> one after another, from two different shapes as well.</li>
</ul>
<p><kbd>H</kbd> or <kbd>Esc</kbd> ends the mode.</p>
<h2>What happens then</h2>
<ul>
<li>On a polyline the <em>second</em> node gives way: it is set to the row or column of the first.</li>
<li>Between two shapes the shape of the second point only moves sideways.</li>
<li>Sides of rectangles and ellipses already lie that way; two points of the same shape that do not already lie so are rejected (“only works on the nodes of a vector”).</li>
<li>The board is always the reference, never the second point.</li>
</ul>
<p>The constraint appears as a small blue symbol beside the line (red if it cannot be met). Click it and press <kbd>Del</kbd> – or right-click it – to delete it; the line then stays as it is.</p>
""",
        "body": """
<p>Der Knopf <strong>Waagrecht/Senkrecht</strong> in der Gruppe „Constraints“ (Taste <kbd>H</kbd>) schaltet den Modus ein.</p>
{{img:ausrichten.png|Waagrecht/Senkrecht im Einsatz}}
<h2>Bedienen</h2>
<ul>
<li><strong>Eine Linie anklicken</strong> (eine Seite einer Form oder ein Segment eines Linienzugs): CncCraft legt sie waagrecht oder senkrecht fest, je nachdem, was der Linie näher ist. Bei Gleichstand wird sie waagrecht.</li>
<li>Oder <strong>zwei Punkte</strong> nacheinander anklicken, auch von zwei verschiedenen Formen.</li>
</ul>
<p><kbd>H</kbd> oder <kbd>Esc</kbd> beendet den Modus.</p>
<h2>Was dann passiert</h2>
<ul>
<li>Bei einem Linienzug gibt der <em>zweite</em> Knoten nach: Er wird auf die Höhe oder Spalte des ersten gesetzt.</li>
<li>Zwischen zwei Formen wird die Form des zweiten Punktes nur quer verschoben.</li>
<li>Seiten von Rechtecken und Ellipsen liegen ohnehin so; zwei Punkte derselben Form, die nicht schon so liegen, werden abgelehnt („geht nur an den Knoten eines Vektors“).</li>
<li>Die Platte ist immer Bezug, nie der zweite Punkt.</li>
</ul>
<p>Die Bedingung erscheint als kleines blaues Symbol neben der Linie (rot, wenn sie sich nicht einhalten lässt). Klicken Sie es an und drücken Sie <kbd>Entf</kbd> – oder klicken Sie mit rechts darauf –, um sie zu löschen; die Linie bleibt dann, wie sie ist.</p>
""",
    },
    {
        "slug": "hilfslinien", "cat": C, "title": "Hilfslinien am Lineal",
        "title_en": "Construction lines on the ruler",
        "lead": "Rote Hilfslinien quer über das Werkstück, an denen Formen einrasten und hängen bleiben – mit Maß, Formel und Sperre.",
        "lead_en": "Red construction lines across the workpiece that shapes snap to and hang on – with a value, a formula and a lock.",
        "neu": True,
        "body_en": """
<div class="neu-box"><p>Spark has similar construction lines; in CncCraft they are also a <em>soft constraint</em>: shapes you drag onto a line stay attached and move along when the line moves – for example because a variable changes.</p></div>
{{img:hilfslinien.png|Two construction lines: a fixed one at 120 mm above, and one on the left that follows the formula <code>platte_l/2</code>}}
<h2>Turning the rulers on and off</h2>
<p>The ruler button in the header (“Decoration” group) shows the rulers along the top and left edge of the board. Construction lines exist only while the rulers are on.</p>
<h2>Adding a construction line</h2>
<ul>
<li>A <strong>click in the top ruler</strong> adds a vertical line there, a click in the left ruler a horizontal one.</li>
<li>You can also <strong>press and drag</strong> in the ruler: the new line follows the mouse until you let go.</li>
</ul>
<h2>Moving – only on the ruler</h2>
<p>Each line has a red <strong>handle</strong> in the ruler. Only there can it be dragged; over the handle the pointer shows a double arrow. In the drawing itself construction lines do not react – so they never get in the way of drawing. While dragging, the line always lands on <strong>whole millimetres</strong>.</p>
<h2>The exact distance</h2>
<p>Outside the ruler the line's distance from the left or bottom board edge is shown – in the same label as the values of [[abstand-bemassen|distances]]. Over the number the pointer shows a hand: <strong>long-press</strong> opens it for typing. As in any dimension field, decimals, [[variablen|variables]] and [[formeln-in-feldern|formulas]] are allowed; a formula is kept (“ƒ platte_l/2 = 100 mm”), and the line follows when the variable or the board size changes. Drag a formula line by its handle and it becomes a fixed number again.</p>
<h2>Making shapes snap</h2>
<p>Drag a shape so close to a construction line that its left, right, top or bottom edge or its centre almost touches the line, and the shape turns <strong>red</strong> and the line grows thicker: let go now and the shape snaps exactly into place.</p>
<ul>
<li>The snapped shape <strong>hangs on the line</strong>: move the line, change its number or its formula, and the shape moves along – and with it the toolpaths.</li>
<li>Drag the shape itself away again and it <strong>comes off</strong> the line. That is why it is a soft constraint.</li>
<li>Snapping happens only when <em>you</em> drag a shape onto a line. Push a line across a resting shape and it does not attach.</li>
</ul>
<h2>Selecting and deleting</h2>
<p>A click on the handle selects the line (it grows bolder), <kbd>Del</kbd> deletes it. The shapes that hung on it stay where they are.</p>
<h2>Locking construction lines</h2>
<p>In the corner where the rulers meet sits a <strong>lock</strong>. One click locks all construction lines; a short message confirms it. Locked lines become thin and semi-transparent, can no longer be added, moved, edited or deleted – but shapes still snap to them. Another click unlocks them.</p>
{{img:hilfslinien-gesperrt.png|Locked construction lines step into the background}}
<h2>Undo</h2>
<p>Adding, moving (a whole drag is one step), changing the number and deleting can be undone with <kbd>⌘Z</kbd>; attached shapes go back with them. Construction lines and the lock are saved with the project.</p>
""",
        "body": """
<div class="neu-box"><p>Hilfslinien kennt die Vorlage Spark ähnlich; in CncCraft sind sie zusätzlich ein <em>weicher Constraint</em>: Formen, die Sie an eine Linie ziehen, bleiben an ihr hängen und wandern mit, wenn die Linie sich bewegt – etwa weil sich eine Variable ändert.</p></div>
{{img:hilfslinien.png|Zwei Hilfslinien: oben eine feste bei 120 mm, links eine, die der Formel <code>platte_l/2</code> folgt}}
<h2>Lineale ein- und ausschalten</h2>
<p>Der Knopf mit dem Lineal in der Kopfleiste (Gruppe „Dekoration“) zeigt die Lineale am oberen und linken Plattenrand. Hilfslinien gibt es nur bei eingeschalteten Linealen.</p>
<h2>Hilfslinie anlegen</h2>
<ul>
<li>Ein <strong>Klick ins obere Lineal</strong> legt an dieser Stelle eine senkrechte Linie an, ein Klick ins linke Lineal eine waagrechte.</li>
<li>Sie können auch im Lineal <strong>drücken und ziehen</strong>: Die neue Linie folgt der Maus, bis Sie loslassen.</li>
</ul>
<h2>Verschieben – nur am Lineal</h2>
<p>Jede Linie hat im Lineal einen roten <strong>Griff</strong>. Nur dort lässt sie sich ziehen; über dem Griff zeigt der Zeiger einen Doppelpfeil. In der Zeichnung selbst reagieren Hilfslinien nicht – sie stören beim Zeichnen also nie. Beim Ziehen landet die Linie immer auf <strong>ganzen Millimetern</strong>.</p>
<h2>Der genaue Abstand</h2>
<p>Außerhalb des Lineals steht der Abstand der Linie vom linken bzw. unteren Plattenrand – im selben Schild wie die Werte der [[abstand-bemassen|Abstände]]. Über der Zahl zeigt der Zeiger eine Hand: <strong>Lange drücken</strong> öffnet sie zum Eintippen. Wie in jedem Maßfeld sind Kommazahlen, [[variablen|Variablen]] und [[formeln-in-feldern|Formeln]] erlaubt; eine Formel bleibt erhalten („ƒ platte_l/2 = 100 mm“), und die Linie folgt, wenn sich die Variable oder die Plattengröße ändert. Ziehen Sie eine Formel-Linie am Griff, wird sie wieder zu einer festen Zahl.</p>
<h2>Formen einrasten lassen</h2>
<p>Ziehen Sie eine Form so nahe an eine Hilfslinie, dass ihre linke, rechte, obere oder untere Kante oder ihre Mitte die Linie fast berührt, färbt sich die Form <strong>rot</strong> und die Linie wird dicker: Lassen Sie jetzt los, rastet die Form genau ein.</p>
<ul>
<li>Die eingerastete Form <strong>hängt an der Linie</strong>: Verschieben Sie die Linie, ändern Sie ihre Zahl oder ihre Formel, wandert die Form mit – und mit ihr die Werkzeugwege.</li>
<li>Ziehen Sie die Form selbst wieder weg, <strong>löst sie sich</strong> von der Linie. Darum ist es ein weicher Constraint.</li>
<li>Einrasten geschieht nur, wenn <em>Sie</em> eine Form an eine Linie ziehen. Schieben Sie eine Linie über eine ruhende Form, hängt diese nicht an.</li>
</ul>
<h2>Auswählen und löschen</h2>
<p>Ein Klick auf den Griff wählt die Linie aus (sie wird kräftiger), <kbd>Entf</kbd> löscht sie. Die Formen, die an ihr hingen, bleiben wo sie sind.</p>
<h2>Hilfslinien sperren</h2>
<p>In der Ecke, in der sich die Lineale treffen, sitzt ein <strong>Schloss</strong>. Ein Klick sperrt alle Hilfslinien; eine kurze Meldung bestätigt es. Gesperrte Linien werden dünn und halb durchsichtig, lassen sich nicht mehr anlegen, verschieben, bearbeiten oder löschen – Formen rasten aber weiter an ihnen ein. Ein weiterer Klick entsperrt sie.</p>
{{img:hilfslinien-gesperrt.png|Gesperrte Hilfslinien treten in den Hintergrund}}
<h2>Rückgängig</h2>
<p>Anlegen, Verschieben (ein ganzer Zug ist ein Schritt), Zahl ändern und Löschen lassen sich mit <kbd>⌘Z</kbd> rückgängig machen; angehängte Formen gehen dabei mit zurück. Hilfslinien und die Sperre werden mit dem Projekt gespeichert.</p>
""",
    },
]
