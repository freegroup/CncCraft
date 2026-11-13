use egui::text::LayoutJob;
use egui::{Color32, FontId, TextFormat};
use std::sync::Arc;

const MONO: fn() -> FontId = || FontId::monospace(12.0);

fn cmd_color(dark: bool) -> Color32 {
    if dark { Color32::from_rgb(86, 180, 233) } else { Color32::from_rgb(0, 100, 180) }
}

fn num_color(dark: bool) -> Color32 {
    if dark { Color32::from_rgb(230, 159, 0) } else { Color32::from_rgb(160, 80, 0) }
}

fn comment_color(dark: bool) -> Color32 {
    if dark { Color32::from_rgb(120, 140, 120) } else { Color32::from_rgb(100, 120, 100) }
}

fn placeholder_color(dark: bool) -> Color32 {
    if dark { Color32::from_rgb(200, 130, 220) } else { Color32::from_rgb(140, 50, 160) }
}

fn text_color(dark: bool) -> Color32 {
    if dark { Color32::from_rgb(210, 210, 210) } else { Color32::from_rgb(30, 30, 30) }
}

pub fn layout_gcode(ui: &egui::Ui, text: &str, wrap_width: f32) -> Arc<egui::Galley> {
    let dark = ui.visuals().dark_mode;
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;

    for line in text.split_inclusive('\n') {
        layout_line(&mut job, line, dark);
    }
    if !text.ends_with('\n') && !text.is_empty() {
        // no trailing newline — already handled
    }
    ui.fonts(|f| f.layout_job(job))
}

fn layout_line(job: &mut LayoutJob, line: &str, dark: bool) {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'(' {
            if let Some(end) = line[i..].find(')') {
                let span = &line[i..i + end + 1];
                job.append(span, 0.0, TextFormat { font_id: MONO(), color: comment_color(dark), ..Default::default() });
                i += end + 1;
                continue;
            }
        }

        if bytes[i] == b';' {
            job.append(&line[i..], 0.0, TextFormat { font_id: MONO(), color: comment_color(dark), ..Default::default() });
            return;
        }

        if bytes[i] == b'{' {
            if let Some(end) = line[i..].find('}') {
                let span = &line[i..i + end + 1];
                job.append(span, 0.0, TextFormat { font_id: MONO(), color: placeholder_color(dark), ..Default::default() });
                i += end + 1;
                continue;
            }
        }

        let b = bytes[i].to_ascii_uppercase();
        if matches!(b, b'G' | b'M' | b'T' | b'S' | b'F' | b'N' | b'X' | b'Y' | b'Z') {
            let start = i;
            i += 1;
            while i < len && (bytes[i].is_ascii_digit() || bytes[i] == b'.' || bytes[i] == b'-') {
                i += 1;
            }
            if i > start + 1 {
                let letter = &line[start..start + 1];
                let number = &line[start + 1..i];
                let letter_color = match b {
                    b'G' | b'M' | b'T' => cmd_color(dark),
                    b'S' | b'F' | b'N' => num_color(dark),
                    _ => num_color(dark), // X Y Z
                };
                job.append(letter, 0.0, TextFormat { font_id: MONO(), color: letter_color, ..Default::default() });
                job.append(number, 0.0, TextFormat { font_id: MONO(), color: num_color(dark), ..Default::default() });
                continue;
            }
            i = start + 1;
            job.append(&line[start..i], 0.0, TextFormat { font_id: MONO(), color: text_color(dark), ..Default::default() });
            continue;
        }

        let start = i;
        while i < len
            && bytes[i] != b'('
            && bytes[i] != b';'
            && bytes[i] != b'{'
            && !matches!(bytes[i].to_ascii_uppercase(), b'G' | b'M' | b'T' | b'S' | b'F' | b'N' | b'X' | b'Y' | b'Z')
        {
            i += 1;
        }
        if i > start {
            job.append(&line[start..i], 0.0, TextFormat { font_id: MONO(), color: text_color(dark), ..Default::default() });
        }
    }
}

pub fn gcode_description(code: &str) -> Option<&'static str> {
    match code {
        // G-Codes: Bewegung
        "G0" | "G00" => Some("Eilgang (Rapid) – schnelle Positionierung ohne Schnitt"),
        "G1" | "G01" => Some("Linearinterpolation – geradlinige Fahrt mit Vorschub (Schnitt)"),
        "G2" | "G02" => Some("Kreisinterpolation im Uhrzeigersinn (CW)"),
        "G3" | "G03" => Some("Kreisinterpolation gegen den Uhrzeigersinn (CCW)"),
        "G4" | "G04" => Some("Verweilzeit (Dwell) – Pause für P Sekunden/Millisekunden"),
        "G5" | "G05" => Some("Kubische Spline-Interpolation (High-Speed-Modus)"),
        "G5.1" => Some("Quadratische B-Spline-Interpolation"),
        "G5.2" => Some("NURBS-Interpolation Start"),
        "G5.3" => Some("NURBS-Interpolation Ende"),

        // G-Codes: Ebenen
        "G17" => Some("XY-Ebene wählen (Standard für Fräsen)"),
        "G18" => Some("XZ-Ebene wählen"),
        "G19" => Some("YZ-Ebene wählen"),
        "G17.1" => Some("UV-Ebene wählen"),
        "G18.1" => Some("WU-Ebene wählen"),
        "G19.1" => Some("VW-Ebene wählen"),

        // G-Codes: Einheiten
        "G20" => Some("Einheit: Zoll (Inch)"),
        "G21" => Some("Einheit: Millimeter"),

        // G-Codes: Referenzfahrt
        "G28" => Some("Referenzfahrt – über Zwischenpunkt zur Maschinenreferenz"),
        "G28.1" => Some("Referenzpunkt speichern (aktuelle Position)"),
        "G30" => Some("Zweite Referenzposition anfahren"),
        "G30.1" => Some("Zweiten Referenzpunkt speichern"),

        // G-Codes: Werkzeugradiuskompensation
        "G40" => Some("Werkzeugradiuskompensation aufheben"),
        "G41" => Some("Werkzeugradiuskompensation links"),
        "G41.1" => Some("Werkzeugradiuskompensation links, dynamischer Radius"),
        "G42" => Some("Werkzeugradiuskompensation rechts"),
        "G42.1" => Some("Werkzeugradiuskompensation rechts, dynamischer Radius"),

        // G-Codes: Werkzeuglänge
        "G43" => Some("Werkzeuglängenkompensation positiv (H-Nummer)"),
        "G43.1" => Some("Werkzeuglängenkompensation dynamisch"),
        "G43.2" => Some("Werkzeuglängenkompensation mit zusätzlicher Achse"),
        "G49" => Some("Werkzeuglängenkompensation aufheben"),

        // G-Codes: Koordinatensysteme
        "G53" => Some("Maschinenkoordinaten (nicht Werkstück) – nur für diese Zeile"),
        "G54" => Some("Werkstück-Koordinatensystem 1 (Standard)"),
        "G55" => Some("Werkstück-Koordinatensystem 2"),
        "G56" => Some("Werkstück-Koordinatensystem 3"),
        "G57" => Some("Werkstück-Koordinatensystem 4"),
        "G58" => Some("Werkstück-Koordinatensystem 5"),
        "G59" => Some("Werkstück-Koordinatensystem 6"),
        "G59.1" => Some("Werkstück-Koordinatensystem 7"),
        "G59.2" => Some("Werkstück-Koordinatensystem 8"),
        "G59.3" => Some("Werkstück-Koordinatensystem 9"),

        // G-Codes: Bahnsteuerung
        "G61" => Some("Genauhalt-Modus (Exact Stop) – jede Ecke exakt anfahren"),
        "G61.1" => Some("Genauhalt-Modus für Einzelblock"),
        "G64" => Some("Bahnsteuerung (Continuous Mode) – Ecken werden verrundet"),

        // G-Codes: Koordinaten-Rotation / Skalierung
        "G68" => Some("Koordinatensystem-Rotation EIN"),
        "G69" => Some("Koordinatensystem-Rotation AUS"),

        // G-Codes: Festzyklen
        "G73" => Some("Hochgeschwindigkeits-Tieflochbohrzyklus (Chip Breaking)"),
        "G74" => Some("Linksgewinde-Bohrzyklus (Reverse Tap)"),
        "G76" => Some("Feinbohrzyklus (Fine Boring)"),
        "G80" => Some("Festzyklus aufheben"),
        "G81" => Some("Bohrzyklus (einfach, ohne Verweilzeit)"),
        "G82" => Some("Bohrzyklus mit Verweilzeit am Grund"),
        "G83" => Some("Tieflochbohrzyklus (Spanbrechen, Vollrückzug)"),
        "G84" => Some("Gewindebohrzyklus (Rechtsgewinde)"),
        "G85" => Some("Reibzyklus (Reaming, Rückzug mit Vorschub)"),
        "G86" => Some("Bohrzyklus (Spindel stoppt am Grund, Eilgang-Rückzug)"),
        "G87" => Some("Rückwärts-Bohrzyklus (Back Boring)"),
        "G88" => Some("Bohrzyklus mit Verweilzeit und manuellem Rückzug"),
        "G89" => Some("Reibzyklus mit Verweilzeit am Grund"),

        // G-Codes: Absolutmaß / Kettenmaß
        "G90" => Some("Absolutmaß – Koordinaten beziehen sich auf den Nullpunkt"),
        "G90.1" => Some("IJ-Absolutmaß für Kreisinterpolation"),
        "G91" => Some("Kettenmaß (Inkremental) – relativ zur aktuellen Position"),
        "G91.1" => Some("IJ-Inkremental für Kreisinterpolation (Standard)"),
        "G92" => Some("Koordinatensystem-Versatz setzen (Nullpunkt verschieben)"),
        "G92.1" => Some("Koordinatensystem-Versatz zurücksetzen"),
        "G92.2" => Some("Koordinatensystem-Versatz zurücksetzen (ohne Parameter ändern)"),
        "G92.3" => Some("Koordinatensystem-Versatz wiederherstellen"),
        "G93" => Some("Vorschub als Inverse Time (1/min)"),
        "G94" => Some("Vorschub in mm/min (oder Zoll/min)"),
        "G95" => Some("Vorschub in mm/Umdrehung"),
        "G96" => Some("Konstante Schnittgeschwindigkeit (CSS) EIN"),
        "G97" => Some("Konstante Schnittgeschwindigkeit AUS (Drehzahl direkt)"),
        "G98" => Some("Festzyklus: Rückzug zur Anfangshöhe"),
        "G99" => Some("Festzyklus: Rückzug zur R-Ebene"),

        // M-Codes: Programm
        "M0" | "M00" => Some("Programmhalt – Maschine stoppt, weiter mit Cycle Start"),
        "M1" | "M01" => Some("Optionaler Halt – stoppt nur bei aktivem Optional-Stop-Schalter"),
        "M2" | "M02" => Some("Programmende"),

        // M-Codes: Spindel
        "M3" | "M03" => Some("Spindel EIN – Rechtslauf (im Uhrzeigersinn, CW)"),
        "M4" | "M04" => Some("Spindel EIN – Linkslauf (gegen Uhrzeigersinn, CCW)"),
        "M5" | "M05" => Some("Spindel AUS"),

        // M-Codes: Werkzeug
        "M6" | "M06" => Some("Werkzeugwechsel – wechselt zum mit T gewählten Werkzeug"),

        // M-Codes: Kühlmittel
        "M7" | "M07" => Some("Kühlmittel EIN (Sprühnebel / Mist)"),
        "M8" | "M08" => Some("Kühlmittel EIN (Flut / Flood)"),
        "M9" | "M09" => Some("Kühlmittel AUS"),

        // M-Codes: Programm / Steuerung
        "M19" => Some("Spindel orientieren (Winkelposition anfahren)"),
        "M30" => Some("Programmende und Rückspulen zum Anfang"),
        "M47" => Some("Programm von Anfang wiederholen"),
        "M48" => Some("Override-Regler (Vorschub/Spindel) aktivieren"),
        "M49" => Some("Override-Regler (Vorschub/Spindel) deaktivieren"),
        "M98" => Some("Unterprogramm aufrufen"),
        "M99" => Some("Rücksprung aus Unterprogramm / Programmende mit Rückspulen"),

        // T-Codes
        t if t.starts_with('T') => Some("Werkzeug wählen (T-Nummer) – wird mit M6 gewechselt"),

        // S-Codes
        s if s.starts_with('S') => Some("Spindeldrehzahl setzen (U/min)"),

        // F-Codes
        f if f.starts_with('F') => Some("Vorschubgeschwindigkeit setzen"),

        // N-Codes
        n if n.starts_with('N') => Some("Satznummer (Zeilennummer zur Organisation)"),

        _ => None,
    }
}

pub fn gcode_tooltips(
    text: &str,
    response: &egui::Response,
    galley: &std::sync::Arc<egui::Galley>,
    text_draw_pos: egui::Pos2,
) {
    if text.is_empty() || !response.hovered() {
        return;
    }
    let Some(pointer) = response.ctx.input(|i| i.pointer.hover_pos()) else { return };
    let local = pointer - text_draw_pos.to_vec2();
    let cursor = galley.cursor_from_pos(local.to_vec2());
    let byte_idx = cursor.rcursor.column;
    let row = cursor.rcursor.row;

    let line = match text.lines().nth(row) {
        Some(l) => l,
        None => return,
    };

    if let Some(code) = code_at(line, byte_idx) {
        let upper = code.to_ascii_uppercase();
        if let Some(desc) = gcode_description(&upper) {
            response.clone().on_hover_ui(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&upper).strong().monospace());
                    ui.label("–");
                    ui.label(desc);
                });
            });
        }
    }
}

fn code_at(line: &str, col: usize) -> Option<String> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return None;
    }

    let mut i = 0;
    while i < len {
        let b = bytes[i].to_ascii_uppercase();
        if matches!(b, b'G' | b'M' | b'T' | b'S' | b'F') {
            let start = i;
            i += 1;
            while i < len && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            if i > start + 1 && col >= start && col < i {
                return Some(line[start..i].to_string());
            }
            continue;
        }
        i += 1;
    }
    None
}
