"""Kategorie „Versionen“."""

C = "versionen"

PAGES = [
    {
        "slug": "version-0-1-0", "cat": C, "title": "Version 0.1.0 – Funktionsumfang",
        "title_en": "Version 0.1.0 – feature set",
        "lead": "Das kann die aktuelle Version, und was gegenüber der Vorlage Spark neu ist.",
        "lead_en": "What the current version can do, and what is new compared with the Spark template.",
        "body_en": """
<p>CncCraft is free: no licence, no subscription, no account. It is aimed at beginners who want to design parametrically without having to learn a big CAD program.</p>
<h2>What CncCraft 0.1.0 can do</h2>
<ul>
<li>Jobs with size, material and origin; save and open projects and reopen recent ones ([[neuer-job]], [[dateien-speichern]]).</li>
<li>Drawing: line, [[spline|spline]], rectangle, circle, ellipse, polygon, text ([[auswahlwerkzeug]]); [[dxf-importieren|import DXF]], [[bild-als-vorlage|images as a template]].</li>
<li>Editing: nodes, offset, [[vervielfaeltigen|pattern (array)]], mirror, centre, convert to curves, boolean operations, [[gruppieren|grouping]], [[vektoren-trennen-verbinden|trim, delete and join]], [[ecken-abrunden|rounding and ears]], copy and paste ([[knotenbearbeitung]], [[boolesche-operationen]]).</li>
<li>Layers with visibility, lock and colour ([[ebenenliste]]).</li>
<li>Toolpaths: profile, cut-out, pocket (offset, raster, spiral), slot, drilling, V-carving ([[werkzeugwege-uebersicht]]).</li>
<li>Preview in 2D and 3D and output as G-code (mm, inch, GRBL) ([[vorschau-werkzeugwege]], [[werkzeugwege-speichern]]).</li>
<li>Light and dark theme ([[einstellungen-design]]).</li>
</ul>
<h2>New compared with the Spark template</h2>
<ul>
<li>[[variablen|Variables]] and [[formeln-in-feldern|formulas]] in every dimension field and depth.</li>
<li>[[abstand-bemassen|Distance constraints]] and [[waagrecht-senkrecht|level/upright]] solved directly.</li>
<li>[[kopieren-einfuegen|Copy and paste]] including toolpaths and constraints.</li>
<li>[[ansicht-3d|View cube and origin axis cross]] in 3D; worksheet decorations you switch on one by one.</li>
<li>[[tastenkuerzel|Single keys for tools]] (S, L, P, R, C, O, T, D, H).</li>
<li>The preview's “Detail” slider.</li>
<li>[[werkzeugwegliste|Tool entries in the toolpath list]] and, when [[werkzeugwege-speichern|saving]], the choice between one file or one file per tool in a ZIP.</li>
<li>[[dateien-speichern|A warning about unsaved changes]] on closing and on replacing the drawing.</li>
<li>[[hilfslinien|Construction lines]] from the ruler: shapes snap and hang on them, distances take formulas and variables, whole millimetres while dragging, a lock against accidental moves.</li>
<li>A [[materialien|material library]] with ten materials, your own materials, recommendations, warnings for cuts that are too fast or too deep, and material surfaces in the 3D view.</li>
<li>The [[nut-werkzeugweg|slot toolpath]] and the spiral strategy for [[taschen-werkzeugweg|pockets]].</li>
<li>[[rueckgaengig-wiederholen|Undo]] for every action on layers, toolpaths and construction lines.</li>
<li>One shared input field for all values in the drawing (dimensions, distances, radii, construction lines): the same suggestions, formulas and error messages everywhere.</li>
</ul>
""",
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
