"""Kategorie „Entwurfswerkzeuge“."""

C = "entwurf"

PAGES = [
    {
        "slug": "auswahlwerkzeug", "cat": C, "title": "Auswählen und verschieben",
        "lead": "Mit dem Auswahlwerkzeug klicken Sie Formen an, verschieben sie und ändern ihre Größe.",
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
        "lead": "Das Formular „Objekte transformieren“ legt Lage und Größe einer Form durch Zahlen fest.",
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
        "lead": "Mit dem Linienwerkzeug entstehen offene oder geschlossene Linienzüge aus Strecken.",
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
        "lead": "Rechtecke ziehen Sie von einer Ecke zur gegenüberliegenden auf; abgerundete Ecken sind ein Maßfeld.",
        "body": """
<p>Wählen Sie das Rechteck (Taste <kbd>R</kbd>), drücken Sie an einer Ecke und ziehen Sie zur gegenüberliegenden. Beim Loslassen ist das Rechteck fertig und ausgewählt.</p>
<p>Unter [[eigenschaften-masse]] stellen Sie Mitte, <strong>Breite</strong> und <strong>Höhe</strong> genau ein; der <strong>Eckradius</strong> rundet die Ecken ab. Der Eckradius wirkt auch auf die Werkzeugwege.</p>
""",
    },
    {
        "slug": "kreis", "cat": C, "title": "Kreis zeichnen",
        "lead": "Kreise zeichnen Sie vom Mittelpunkt aus; der Radius ist ein Maßfeld.",
        "body": """
<p>Wählen Sie den Kreis (Taste <kbd>C</kbd>), drücken Sie auf den Mittelpunkt und ziehen Sie bis zum gewünschten Radius. Mitte und Radius lassen sich danach unter [[eigenschaften-masse]] genau eintippen.</p>
<p>Kreise sind die Grundlage für Bohrungen: Der [[bohr-werkzeugweg|Bohr-Werkzeugweg]] bohrt in der Mitte jeder gewählten Kontur.</p>
""",
    },
    {
        "slug": "ellipse", "cat": C, "title": "Ellipse zeichnen",
        "lead": "Eine Ellipse hat zwei Maße: Breite und Höhe.",
        "body": """
<p>Wählen Sie die Ellipse und ziehen Sie den Begrenzungsrahmen auf, in den sie passen soll. Breite und Höhe stellen Sie danach unter [[eigenschaften-masse]] genauer ein. Eine Ellipse mit gleicher Breite und Höhe ist ein Kreis.</p>
""",
    },
    {
        "slug": "polygon", "cat": C, "title": "Polygon zeichnen",
        "lead": "Ein Polygon ist ein regelmäßiges Vieleck mit einstellbarer Eckenzahl.",
        "body": """
<p>Wählen Sie das Polygon, drücken Sie auf den Mittelpunkt und ziehen Sie bis zur gewünschten Größe. Es hat mindestens drei Ecken; die Zahl der <strong>Ecken</strong> und den <strong>Radius</strong> ändern Sie unter [[eigenschaften-masse]]. Das Beispiel enthält ein Sechseck.</p>
""",
    },
    {
        "slug": "text-erstellen", "cat": C, "title": "Text erstellen",
        "lead": "Text wird als Vektorform eingefügt und kann danach gefräst oder in Kurven umgewandelt werden.",
        "body": """
<p>Wählen Sie das Textwerkzeug und klicken Sie in die Zeichnung. Ein Beispieltext wird eingefügt; seinen Inhalt ändern Sie im Feld <strong>Text</strong> unter [[eigenschaften-masse]], die Größe mit dem Feld <strong>Höhe</strong>.</p>
<p>Schrift eignet sich besonders für das [[vcarving-werkzeugweg|V-Carving]]: Der V-Fräser fährt die Buchstaben je nach Breite unterschiedlich tief ab. Wollen Sie die einzelnen Buchstabenumrisse bearbeiten, wandeln Sie den Text zuerst [[in-kurven-umwandeln|in Kurven um]].</p>
""",
    },
    {
        "slug": "knotenbearbeitung", "cat": C, "title": "Knoten bearbeiten",
        "lead": "Im Knotenmodus ziehen Sie die Eckpunkte einer Form einzeln, fügen welche ein oder löschen sie.",
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
        "lead": "Der Versatz erzeugt eine Kontur im festen Abstand – größer oder kleiner als das Original.",
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
        "lead": "Spiegeln kehrt die gewählten Formen um ihre Mitte waagrecht oder senkrecht um.",
        "body": """
<p>Wählen Sie die Formen und klicken Sie in der Gruppe „Ausrichten und Spiegeln“ auf <strong>Waagrecht spiegeln</strong> oder <strong>Senkrecht spiegeln</strong>. Die Formen werden um die Mitte der Auswahl umgeklappt; bei mehreren Formen bleibt ihre Lage zueinander erhalten.</p>
""",
    },
    {
        "slug": "zentrieren", "cat": C, "title": "Auf dem Material zentrieren",
        "lead": "Zentrieren schiebt die Auswahl in die Mitte des Werkstücks.",
        "body": """
<p>Wählen Sie die Formen und klicken Sie auf <strong>Auf dem Material zentrieren</strong> (Gruppe „Ausrichten und Spiegeln“). Die Mitte der gesamten Auswahl wird auf die Mitte der Platte gelegt; ihre Größe ändert sich nicht.</p>
""",
    },
    {
        "slug": "in-kurven-umwandeln", "cat": C, "title": "In Kurven umwandeln",
        "lead": "Das Umwandeln macht aus Grundformen und Text freie Kurven, deren Knoten Sie bearbeiten können.",
        "body": """
<p>Wählen Sie Formen oder Text und klicken Sie auf <strong>In Kurven umwandeln</strong> (Gruppe „Objekte bearbeiten“). Aus einem Kreis wird eine Kurve aus Knoten, aus Text die Umrisse der Buchstaben. Danach lässt sich alles im Modus [[knotenbearbeitung|Knoten bearbeiten]] formen.</p>
<div class="hinweis"><p>Umgewandelte Formen haben keine Maßfelder wie „Radius“ oder „Text“ mehr, und Variablen an ihnen werden gelöst. Wer die Form noch parametrisch ändern will, wandelt sie deshalb erst am Ende um.</p></div>
""",
    },
    {
        "slug": "boolesche-operationen", "cat": C, "title": "Boolesche Operationen",
        "lead": "Vereinen, Abziehen und Schnittmenge setzen aus mehreren geschlossenen Formen eine neue zusammen.",
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
        "lead": "Formen lassen sich kopieren und einfügen – samt ihrer Werkzeugwege und der Bedingungen zwischen ihnen.",
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
        "lead": "Beim Zeichnen und Verschieben rastet der Zeiger an Hilfspunkten ein, damit alles genau sitzt.",
        "body": """
<p>CncCraft kennt vier Arten von Einrastpunkten:</p>
<ul>
<li><strong>Knoten</strong> – Eckpunkte bestehender Formen,</li>
<li><strong>Mitte</strong> – Mittelpunkte,</li>
<li><strong>Halbe Strecke</strong> – die Mitte einer Kante,</li>
<li><strong>Job-Punkt</strong> – die Ecken und die Mitte des Werkstücks,</li>
<li><strong>Raster</strong> – die Kreuzungen des Rasters.</li>
</ul>
<p>Ein kleiner Hinweis neben dem Zeiger nennt, woran er gerade einrastet.</p>
<h2>Einstellen</h2>
<p>Die Schalter <strong>Am Raster einrasten</strong> und <strong>An Geometrie einrasten</strong> in der Kopfleiste (Gruppe „Dekoration“) schalten die Arten ein und aus. Im Menü <em>Ansicht ▸ Einrasten</em> gibt es zusätzlich:</p>
<ul>
<li><em>An Geometrie (Knoten, Mitten)</em> und <em>An Job-Ecken und -Mitte</em> getrennt,</li>
<li><strong>Radius</strong> – wie nah in Bildschirmpunkten der Zeiger sein muss,</li>
<li><strong>Am Raster</strong> mit dem Rasterabstand in Millimetern.</li>
</ul>
<p>Mit gedrückter <kbd>⇧</kbd>-Taste beim Ziehen rastet nichts ein.</p>
""",
    },
    {
        "slug": "eigenschaften-masse", "cat": C, "title": "Maße und Eigenschaften",
        "lead": "Jede Form hat Maßfelder, in die Sie Zahlen, Formeln oder Variablen eintippen.",
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
]
