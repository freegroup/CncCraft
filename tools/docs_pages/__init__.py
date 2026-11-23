"""Die Kategorien und Seiten der Hilfe. Jede Seite: slug, title, cat, lead, body (HTML),
optional neu=True für Funktionen, die CncCraft zusätzlich zur Vorlage hat.

Nur Funktionen, die CncCraft wirklich hat, werden beschrieben; Funktionen der Vorlage, die hier
fehlen (z. B. Laser, Bildbearbeitung, Werkzeugdatenbank-Dialoge, Maschinenverwaltung), kommen
nicht vor.
"""
from . import erste_schritte, entwurf, parametrisch, werkzeugwege, versionen

CATEGORIES = [
    {"id": "erste-schritte", "title": "Erste Schritte", "symbol": "▶",
     "title_en": "Getting started",
     "text": "Programm kennenlernen, einen Job einrichten, sich in der Entwurfsansicht zurechtfinden.",
     "text_en": "Get to know the program, set up a job and find your way around the design view."},
    {"id": "entwurf", "title": "Entwurfswerkzeuge", "symbol": "✎",
     "title_en": "Design tools",
     "text": "Formen zeichnen, bearbeiten, ausrichten und mit Booleschen Operationen kombinieren.",
     "text_en": "Draw, edit and align shapes and combine them with boolean operations."},
    {"id": "parametrisch", "title": "Parametrisch entwerfen", "symbol": "ƒ",
     "title_en": "Parametric design",
     "text": "Variablen, Formeln und Bedingungen: Maße, die sich mit der Zeichnung ändern.",
     "text_en": "Variables, formulas and constraints: dimensions that change with the drawing."},
    {"id": "werkzeugwege", "title": "Werkzeugwege", "symbol": "⛭",
     "title_en": "Toolpaths",
     "text": "Profil, Ausschnitt, Tasche, Bohren, V-Carving – berechnen, ansehen, ausgeben.",
     "text_en": "Profile, cut-out, pocket, drilling, V-carving – compute, preview, output."},
    {"id": "versionen", "title": "Versionen", "symbol": "★",
     "title_en": "Versions",
     "text": "Der Funktionsumfang der aktuellen Version und was gegenüber dem Original neu ist.",
     "text_en": "What the current version can do and what is new compared with the original."},
]

PAGES = erste_schritte.PAGES + entwurf.PAGES + parametrisch.PAGES + werkzeugwege.PAGES + versionen.PAGES
