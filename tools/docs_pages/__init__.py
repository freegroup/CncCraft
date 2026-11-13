"""Die Kategorien und Seiten der Hilfe. Jede Seite: slug, title, cat, lead, body (HTML),
optional neu=True für Funktionen, die CncCraft zusätzlich zur Vorlage hat.

Nur Funktionen, die CncCraft wirklich hat, werden beschrieben; Funktionen der Vorlage, die hier
fehlen (z. B. Laser, Bildbearbeitung, Werkzeugdatenbank-Dialoge, Maschinenverwaltung), kommen
nicht vor.
"""
from . import erste_schritte, entwurf, parametrisch, werkzeugwege, versionen

CATEGORIES = [
    {"id": "erste-schritte", "title": "Erste Schritte", "symbol": "▶",
     "text": "Programm kennenlernen, einen Job einrichten, sich in der Entwurfsansicht zurechtfinden."},
    {"id": "entwurf", "title": "Entwurfswerkzeuge", "symbol": "✎",
     "text": "Formen zeichnen, bearbeiten, ausrichten und mit Booleschen Operationen kombinieren."},
    {"id": "parametrisch", "title": "Parametrisch entwerfen", "symbol": "ƒ",
     "text": "Variablen, Formeln und Bedingungen: Maße, die sich mit der Zeichnung ändern."},
    {"id": "werkzeugwege", "title": "Werkzeugwege", "symbol": "⛭",
     "text": "Profil, Ausschnitt, Tasche, Bohren, V-Carving – berechnen, ansehen, ausgeben."},
    {"id": "versionen", "title": "Versionen", "symbol": "★",
     "text": "Der Funktionsumfang der aktuellen Version und was gegenüber dem Original neu ist."},
]

PAGES = erste_schritte.PAGES + entwurf.PAGES + parametrisch.PAGES + werkzeugwege.PAGES + versionen.PAGES
