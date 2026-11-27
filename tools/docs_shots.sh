#!/bin/sh
# Nimmt die Abbildungen der Website (docs/assets/img) neu auf: jede ist ein Bild des
# Programmfensters (`cnccraft --screenshot`) mit dem Schalter, der den gezeigten Zustand
# herstellt, auf 1400 Punkte Breite gebracht.
#
# Aufruf aus dem Ordner app/:   sh tools/docs_shots.sh      (vorher: cargo build)
set -e
BIN=./target/debug/cnccraft
OUT=docs/assets/img
TMP=$(mktemp -d)
mkdir -p "$OUT"

shot() {
    name=$1; shift
    env "$@" "$BIN" --screenshot "$TMP/$name" >/dev/null 2>&1
    sips -Z 1400 "$TMP/$name" --out "$OUT/$name" >/dev/null
    echo "$OUT/$name"
}

# die Oberfläche mit Hilfslinien am Lineal (eine folgt einer Formel)
shot oberflaeche.png CNCCRAFT_SHOW_GUIDES=1
shot werkzeugliste.png CNCCRAFT_X=1
shot dunkel.png CNCCRAFT_DARK=1 CNCCRAFT_SHOW_GUIDES=1
shot ansicht-3d.png CNCCRAFT_SHOW_3D=1
shot ebenen.png CNCCRAFT_SHOW_LAYERS=1
shot variablen.png CNCCRAFT_SHOW_VARS=1
shot job-dialog.png CNCCRAFT_SHOW_DIALOG=1 CNCCRAFT_MATERIAL=Hardwood
shot tiefe.png CNCCRAFT_SHOW_DEPTH=1
shot abstand.png CNCCRAFT_SHOW_DISTANCE=1
shot ausrichten.png CNCCRAFT_SHOW_ALIGN=1
shot bedingung.png CNCCRAFT_SHOW_CONSTRAINT=1
shot ungespeichert.png CNCCRAFT_SHOW_UNSAVED=1
shot export-werkzeuge.png CNCCRAFT_SHOW_EXPORT=1
shot werkzeugleiste-eingeklappt.png CNCCRAFT_FOLD_TOOLS=1
# neu: Hilfslinien (eine mit offenem Formelfeld), Materialien, Nut
shot hilfslinien.png CNCCRAFT_SHOW_GUIDES=1 CNCCRAFT_GUIDE_EDIT=2
shot hilfslinien-gesperrt.png CNCCRAFT_SHOW_GUIDES=1 CNCCRAFT_GUIDES_LOCKED=1
shot materialien.png CNCCRAFT_SHOW_MATERIALS=1 CNCCRAFT_MATERIAL=Plywood
shot material-aluminium.png CNCCRAFT_SHOW_3D=1 CNCCRAFT_MATERIAL=Aluminium
shot material-hartholz.png CNCCRAFT_SHOW_3D=1 CNCCRAFT_MATERIAL=Hardwood
shot nut-werkzeugweg.png CNCCRAFT_SHOW_EDITOR=slot
shot taschen-werkzeugweg.png CNCCRAFT_SHOW_EDITOR=pocket
shot vervielfaeltigen.png CNCCRAFT_SHOW_ARRAY=1
# the cutter paths shown in the 2D view ("Show G-code motion") as coloured bands
shot gcode-fahrt-2d.png CNCCRAFT_SHOW_GCODE=1
rm -rf "$TMP"
