"""Kategorie „Werkzeugwege“."""

C = "werkzeugwege"

PAGES = [
    {
        "slug": "werkzeugwege-uebersicht", "cat": C, "title": "Werkzeugwege – Überblick",
        "lead": "Ein Werkzeugweg sagt der Fräse, welche Formen sie wie bearbeiten soll. CncCraft kennt sechs Arten.",
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
        "lead": "Die Liste zeigt alle Werkzeugwege mit Farbe, Art, Name und geschätzter Fräszeit.",
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
        "lead": "Jeder Werkzeugweg bekommt ein Werkzeug mit passenden Werten für Vorschub, Drehzahl und Zustellung.",
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
        "lead": "Das Profil fährt mit dem Fräser an der Kontur entlang – außen, innen oder auf der Linie.",
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
        "lead": "Der Ausschnitt ist ein Profil, das ein Teil ganz herausschneidet und es mit Haltestegen festhält.",
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
        "lead": "Die Tasche räumt eine Fläche innerhalb einer geschlossenen Kontur bis zu einer Tiefe aus.",
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
        "lead": "Der Bohr-Werkzeugweg bohrt in die Mitte jeder gewählten Kontur.",
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
        "lead": "Beim V-Carving schnitzt ein V-Fräser Schrift und Formen; je breiter die Linie, desto tiefer geht er.",
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
        "lead": "Die Vorschau spielt die Werkzeugwege am Werkstück ab und zeigt, was die Fräse abtragen wird.",
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
        "lead": "Für die Fräse speichern Sie die Werkzeugwege als G-Code-Datei.",
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
        "lead": "Die Nut fräst entlang eines Vektors einen Kanal, so breit wie der Fräser.",
        "neu": True,
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
        "lead": "Jedes Werkstück ist aus einem Material. Es kennt seine Grenzen, empfiehlt Fräser und Kühlung und bestimmt das Aussehen in 3D.",
        "neu": True,
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
