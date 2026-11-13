"""Kategorie „Versionen“."""

C = "versionen"

PAGES = [
    {
        "slug": "version-0-1-0", "cat": C, "title": "Version 0.1.0 – Funktionsumfang",
        "lead": "Das kann die aktuelle Version, und was gegenüber der Vorlage Spark neu ist.",
        "body": """
<h2>Das kann CncCraft 0.1.0</h2>
<ul>
<li>Jobs mit Größe, Material und Nullpunkt; Projekte speichern und öffnen ([[neuer-job]], [[dateien-speichern]]).</li>
<li>Zeichnen: Linie, Rechteck, Kreis, Ellipse, Polygon, Text ([[auswahlwerkzeug]]).</li>
<li>Bearbeiten: Knoten, Versatz, Spiegeln, Zentrieren, In Kurven umwandeln, Boolesche Operationen, Kopieren und Einfügen ([[knotenbearbeitung]], [[boolesche-operationen]]).</li>
<li>Ebenen mit Sichtbarkeit, Sperre und Farbe ([[ebenenliste]]).</li>
<li>Werkzeugwege: Profil, Ausschnitt, Tasche, Bohren, V-Carving ([[werkzeugwege-uebersicht]]).</li>
<li>Vorschau in 2D und 3D und Ausgabe als G-Code (mm, Zoll, GRBL) ([[vorschau-werkzeugwege]], [[werkzeugwege-speichern]]).</li>
<li>Helles und dunkles Design ([[einstellungen-design]]).</li>
</ul>
<h2>Neu gegenüber der Vorlage Spark</h2>
<ul>
<li>[[variablen|Variablen]] und [[formeln-in-feldern|Formeln]] in allen Maßfeldern und Tiefen.</li>
<li>[[abstand-bemassen|Abstands-Bedingungen]] und [[waagrecht-senkrecht|Waagrecht/Senkrecht]] mit direkter Lösung.</li>
<li>[[kopieren-einfuegen|Kopieren und Einfügen]] samt Werkzeugwegen und Bedingungen.</li>
<li>[[ansicht-3d|Ansichtswürfel und Nullpunkt-Achsenkreuz]] in 3D; einzeln schaltbare Dekorationen der Arbeitsfläche.</li>
<li>[[tastenkuerzel|Einzeltasten für Werkzeuge]] (S, L, R, C, O, D, H).</li>
<li>Der Regler „Darstellung“ der Vorschau.</li>
<li>[[werkzeugwegliste|Werkzeug-Einträge in der Werkzeugwegliste]] und beim [[werkzeugwege-speichern|Speichern]] die Wahl zwischen einer Datei oder je Werkzeug eine Datei in einer ZIP-Datei.</li>
<li>[[dateien-speichern|Warnung vor ungespeicherten Änderungen]] beim Schließen und beim Ersetzen der Zeichnung.</li>
</ul>
""",
    },
]
