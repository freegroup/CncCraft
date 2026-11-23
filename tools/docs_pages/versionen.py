"""Kategorie „Versionen“."""

C = "versionen"

PAGES = [
    {
        "slug": "version-0-1-0", "cat": C, "title": "Version 0.1.0 – Funktionsumfang",
        "lead": "Das kann die aktuelle Version, und was gegenüber der Vorlage Spark neu ist.",
        "body": """
<p>CncCraft ist kostenlos: keine Lizenz, kein Abo, kein Konto. Es richtet sich an Einsteiger, die parametrisch entwerfen möchten, ohne ein großes CAD-Programm lernen zu müssen.</p>
<h2>Das kann CncCraft 0.1.0</h2>
<ul>
<li>Jobs mit Größe, Material und Nullpunkt; Projekte speichern, öffnen und zuletzt verwendete wieder öffnen ([[neuer-job]], [[dateien-speichern]]).</li>
<li>Zeichnen: Linie, [[spline|Spline]], Rechteck, Kreis, Ellipse, Polygon, Text ([[auswahlwerkzeug]]); [[dxf-importieren|DXF importieren]], [[bild-als-vorlage|Bilder als Vorlage]].</li>
<li>Bearbeiten: Knoten, Versatz, [[vervielfaeltigen|Vervielfältigen (Gitter)]], Spiegeln, Zentrieren, In Kurven umwandeln, Boolesche Operationen, [[gruppieren|Gruppieren]], [[vektoren-trennen-verbinden|Trennen, Löschen und Verbinden]], [[ecken-abrunden|Abrunden und Ohren]], Kopieren und Einfügen ([[knotenbearbeitung]], [[boolesche-operationen]]).</li>
<li>Ebenen mit Sichtbarkeit, Sperre und Farbe ([[ebenenliste]]).</li>
<li>Werkzeugwege: Profil, Ausschnitt, Tasche (Versatz, Raster, Spirale), Nut, Bohren, V-Carving ([[werkzeugwege-uebersicht]]).</li>
<li>Vorschau in 2D und 3D und Ausgabe als G-Code (mm, Zoll, GRBL) ([[vorschau-werkzeugwege]], [[werkzeugwege-speichern]]).</li>
<li>Helles und dunkles Design ([[einstellungen-design]]).</li>
</ul>
<h2>Neu gegenüber der Vorlage Spark</h2>
<ul>
<li>[[variablen|Variablen]] und [[formeln-in-feldern|Formeln]] in allen Maßfeldern und Tiefen.</li>
<li>[[abstand-bemassen|Abstands-Bedingungen]] und [[waagrecht-senkrecht|Waagrecht/Senkrecht]] mit direkter Lösung.</li>
<li>[[kopieren-einfuegen|Kopieren und Einfügen]] samt Werkzeugwegen und Bedingungen.</li>
<li>[[ansicht-3d|Ansichtswürfel und Nullpunkt-Achsenkreuz]] in 3D; einzeln schaltbare Dekorationen der Arbeitsfläche.</li>
<li>[[tastenkuerzel|Einzeltasten für Werkzeuge]] (S, L, P, R, C, O, T, D, H).</li>
<li>Der Regler „Darstellung“ der Vorschau.</li>
<li>[[werkzeugwegliste|Werkzeug-Einträge in der Werkzeugwegliste]] und beim [[werkzeugwege-speichern|Speichern]] die Wahl zwischen einer Datei oder je Werkzeug eine Datei in einer ZIP-Datei.</li>
<li>[[dateien-speichern|Warnung vor ungespeicherten Änderungen]] beim Schließen und beim Ersetzen der Zeichnung.</li>
<li>[[hilfslinien|Hilfslinien]] aus dem Lineal: Formen rasten ein und hängen daran, Abstände nehmen Formeln und Variablen an, ganze Millimeter beim Ziehen, Sperre gegen versehentliches Verschieben.</li>
<li>Eine [[materialien|Materialbibliothek]] mit zehn Materialien, eigenen Materialien, Empfehlungen, Warnungen bei zu schnellen oder zu tiefen Schnitten und Materialoberflächen in der 3D-Ansicht.</li>
<li>Der [[nut-werkzeugweg|Nut-Werkzeugweg]] und die Strategie Spirale für [[taschen-werkzeugweg|Taschen]].</li>
<li>[[rueckgaengig-wiederholen|Rückgängig]] für jede Aktion an Ebenen, Werkzeugwegen und Hilfslinien.</li>
<li>Ein gemeinsames Eingabefeld für alle Werte in der Zeichnung (Maße, Abstände, Radien, Hilfslinien): überall dieselben Vorschläge, Formeln und Fehlermeldungen.</li>
</ul>
""",
    },
]
