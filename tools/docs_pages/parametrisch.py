"""Kategorie „Parametrisch entwerfen“ – alles hier ist neu gegenüber der Vorlage Spark."""

C = "parametrisch"

PAGES = [
    {
        "slug": "parametrisch-uebersicht", "cat": C, "title": "Parametrisch entwerfen – Überblick",
        "lead": "Maße müssen nicht fest sein: Sie können sie benennen, berechnen und aneinander binden.",
        "neu": True,
        "body": """
<div class="neu-box"><p>Das Original Spark kennt weder Variablen noch Bedingungen. In CncCraft ändert eine geänderte Variable die Zeichnung – bis hin zum G-Code, denn auch die Werkzeugwege werden neu berechnet.</p></div>
<p>Es gibt vier Bausteine, die sich kombinieren lassen:</p>
<table>
<tr><th>Baustein</th><th>Wofür</th></tr>
<tr><td>[[variablen|Variablen]]</td><td>Benannte Werte, die in Formeln benutzt werden, zum Beispiel <code>rand = 12</code>.</td></tr>
<tr><td>[[formeln-in-feldern|Formeln in Maßfeldern]]</td><td>Jedes Maßfeld und jede Tiefe rechnet: <code>platte_b - 2*rand</code>.</td></tr>
<tr><td>[[abstand-bemassen|Abstände bemaßen]]</td><td>Der Abstand zwischen zwei Punkten, einem Punkt und einer Linie oder zur Plattenkante bleibt, wie er ist.</td></tr>
<tr><td>[[waagrecht-senkrecht|Waagrecht/Senkrecht]]</td><td>Eine Linie bleibt waagrecht oder senkrecht.</td></tr>
</table>
<p>Ein Beispiel: Sie zeichnen ein Schild und legen <code>rand</code> fest. Die Kontur bekommt die Breite <code>platte_b - 2*rand</code> und die Höhe <code>platte_l - 2*rand</code>. Ändern Sie später die Plattengröße im Job oder den Wert von <code>rand</code>, passt sich die Kontur an, und die Werkzeugwege folgen.</p>
{{img:variablen.png|Variablen und gebundene Maße im Zusammenspiel}}
<h2>Grenzen</h2>
<ul>
<li>CncCraft löst die Bedingungen direkt, ohne ein allgemeines Gleichungssystem: Eine Form, die eine Bedingung festhält, springt beim Ziehen zurück.</li>
<li>Verschieben mit der Maus, Knotenbearbeitung und Boolesche Operationen erzeugen neue Geometrie ohne Bindungen; die Statuszeile und die Bedienung weisen darauf hin.</li>
</ul>
""",
    },
    {
        "slug": "variablen", "cat": C, "title": "Variablen",
        "lead": "Variablen sind benannte Zahlen oder Formeln, die Sie in allen Maßfeldern verwenden.",
        "neu": True,
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
        "lead": "Jedes Zahlenfeld rechnet mit: Grundrechenarten, Klammern, Potenzen und Variablen.",
        "neu": True,
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
        "lead": "Eine Abstands-Bedingung hält zwei Dinge in einem festen Abstand – auch wenn Sie das eine verschieben oder Variablen ändern.",
        "neu": True,
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
        "lead": "Mit einem Klick bleibt eine Linie waagrecht oder senkrecht – auch wenn Sie an den Knoten ziehen.",
        "neu": True,
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
]
