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
<p>CncCraft kennt fünf Arten von Einrastpunkten:</p>
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
<div class="neu-box"><p>Das Raster ist nur zu sehen, solange <strong>Am Raster einrasten</strong> an ist – was man sieht, rastet auch ein. Zusätzlich rasten verschobene Formen an [[hilfslinien|Hilfslinien]] ein: mit einer Kante oder ihrer Mitte, und die Linie färbt die Form rot, solange sie einrasten würde.</p></div>
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
    {
        "slug": "spline", "cat": C, "title": "Spline zeichnen",
        "lead": "Eine weiche Kurve durch die Punkte, die Sie anklicken.",
        "neu": True,
        "body": """
<p>Das Werkzeug <strong>Spline</strong> (Taste <kbd>P</kbd>) legt eine glatte Kurve durch jeden Punkt, den Sie anklicken. Ein Doppelklick oder <kbd>Enter</kbd> beendet sie; ein Klick auf den ersten Punkt schließt sie zu einer geschlossenen Form, die sich zum Beispiel als [[taschen-werkzeugweg|Tasche]] ausräumen lässt.</p>
<p>Die Punkte lassen sich danach mit der [[knotenbearbeitung]] verschieben; die Kurve bleibt dabei weich.</p>
""",
    },
    {
        "slug": "vektoren-trennen-verbinden", "cat": C, "title": "Vektoren trennen, löschen und verbinden",
        "lead": "Vektoren an Kreuzungen teilen, einzelne Stücke entfernen und offene Enden verbinden.",
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
        "lead": "Ecken mit einem Radius abrunden oder als „Ohr“ ausfräsen, damit Teile ineinander passen.",
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
        "lead": "Aus einer Form ein rechteckiges Muster aus verbundenen Kopien machen – die mit der Form lebt.",
        "neu": True,
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
        "lead": "Mehrere Formen zu einer Gruppe zusammenfassen, die sich wie eine Form auswählen lässt.",
        "body": """
<p><strong>Gruppieren</strong> (<kbd>⌘G</kbd>) fasst die Auswahl zusammen: Ein Klick auf irgendein Teil wählt danach die ganze Gruppe. Gruppen lassen sich selbst wieder gruppieren.</p>
<p><strong>Gruppe auflösen</strong> (<kbd>⇧⌘G</kbd>) löst die äußerste Gruppe der Auswahl auf, eine Stufe je Klick. Kopien einer Gruppe bilden eine eigene, neue Gruppe ([[kopieren-einfuegen]]).</p>
""",
    },
    {
        "slug": "dxf-importieren", "cat": C, "title": "DXF importieren",
        "lead": "Zeichnungen aus anderen Programmen als DXF-Datei übernehmen.",
        "body": """
<p>Das Werkzeug <strong>DXF importieren</strong> liest eine DXF-Datei (R12, R14 und neuer) und legt ihren Inhalt als <strong>eine Gruppe</strong> in die Zeichnung ([[gruppieren]]). Linien, Bögen, Kreise und Polylinien werden zu Vektoren, die Sie wie selbst gezeichnete bearbeiten und mit Werkzeugwegen versehen.</p>
<p>Ändert sich die Datei später im anderen Programm, lädt ein <strong>Rechtsklick auf die Form</strong> sie neu von der Platte.</p>
""",
    },
    {
        "slug": "bild-als-vorlage", "cat": C, "title": "Bild als Vorlage",
        "lead": "Ein Foto oder eine Skizze unter die Zeichnung legen, auf das richtige Maß bringen und nachzeichnen.",
        "neu": True,
        "body": """
<p><strong>Bild einfügen</strong> lädt ein Bild (PNG, JPEG) auf eine Bild-Ebene ([[ebene-neu]]). Es liegt durchscheinend unter der Zeichnung; die Deckkraft stellen Sie in der [[ebenenliste]] ein.</p>
<h2>Kalibrieren</h2>
<p>Damit das Bild maßstäblich ist, klicken Sie mit <strong>Bild kalibrieren</strong> zwei Punkte im Bild an, deren echten Abstand Sie kennen (etwa die Enden eines mitfotografierten Lineals), und tippen den Abstand ein. CncCraft skaliert das Bild so, dass es 1:1 zur Platte passt – danach lässt sich darauf maßgenau nachzeichnen.</p>
""",
    },
]
