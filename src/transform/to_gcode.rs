//! From the program's toolpaths to G-code: the machine file's envelope (units, job start and end)
//! and the hooks of a post-processor around and between the toolpaths' moves.

use super::frame;
use crate::gcode::PostProcessor;
use crate::toolpath::{Job, Toolpath};

fn expand(s: &str, ctx: &HookCtx) -> String {
    if s.is_empty() {
        return String::new();
    }
    let out = s
        .replace("{tool_number}", &ctx.tool_number.to_string())
        .replace("{rpm}", &ctx.rpm.to_string())
        .replace("{safe_z}", &ctx.fmt(ctx.safe_z))
        .replace("{home_z}", &ctx.fmt(ctx.home_z));
    let mut result = String::new();
    for line in out.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            result.push_str(&semicolon_comments(line.trim_start()));
            result.push('\n');
        }
    }
    result
}

/// A line with every comment written after a semicolon: a `(comment)` in the post-processor's own
/// text moves behind the code of its line (all comments in the files are `;` comments).
fn semicolon_comments(line: &str) -> String {
    let (code_part, tail) = match line.find(';') {
        Some(i) => (&line[..i], Some(line[i + 1..].trim())),
        None => (line, None),
    };
    let (mut code, mut notes) = (String::new(), Vec::new());
    let mut chars = code_part.chars();
    while let Some(c) = chars.next() {
        if c == '(' {
            let mut note = String::new();
            for d in chars.by_ref() {
                if d == ')' {
                    break;
                }
                note.push(d);
            }
            notes.push(note.trim().to_string());
        } else {
            code.push(c);
        }
    }
    if let Some(t) = tail {
        notes.push(t.to_string());
    }
    let code = code.trim_end();
    let notes: Vec<&str> = notes.iter().map(String::as_str).filter(|n| !n.is_empty()).collect();
    match (code.is_empty(), notes.is_empty()) {
        (_, true) => code.to_string(),
        (true, false) => format!("; {}", notes.join(" ")),
        (false, false) => format!("{code} ; {}", notes.join(" ")),
    }
}

struct HookCtx {
    tool_number: u32,
    rpm: u32,
    safe_z: f64,
    home_z: f64,
    k: f64,
}

impl HookCtx {
    fn fmt(&self, v: f64) -> String {
        format!("{:.3}", v * self.k)
    }
}

/// What the header of a file says besides the job and the toolpaths.
#[derive(Default, Clone)]
pub struct ExportMeta {
    /// Which toolpaths were chosen to be exported ("Sichtbare Werkzeugwege (4)").
    pub scope: Option<String>,
    /// This file is number `n` of `m` (one file for each tool).
    pub file: Option<(usize, usize)>,
}

#[cfg(test)]
pub fn write(job: &Job, paths: &[&Toolpath], pp: &PostProcessor) -> String {
    write_with_meta(job, paths, pp, &ExportMeta::default())
}

pub fn write_with_meta(job: &Job, paths: &[&Toolpath], pp: &PostProcessor, meta: &ExportMeta) -> String {
    build(job, paths, pp, meta).0
}

/// The G-code, and the line (counted from 0) where each toolpath's own part starts.
pub fn write_with_sections(job: &Job, paths: &[&Toolpath], pp: &PostProcessor) -> (String, Vec<usize>) {
    build(job, paths, pp, &ExportMeta::default())
}

const RULE_WIDTH: usize = 76;
const LABEL_WIDTH: usize = 20;

fn minutes_text(min: f64) -> String {
    let s = (min * 60.0).round() as i64;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

/// The comment block at the top of every file: where it comes from, the workpiece, the tools
/// with all their settings and the toolpaths. Every line is a `;` comment.
fn header(job: &Job, paths: &[&Toolpath], pp: &PostProcessor, meta: &ExportMeta) -> String {
    let rule = format!("; {}\n", "=".repeat(RULE_WIDTH));
    let thin = format!(";   {}\n", "-".repeat(RULE_WIDTH - 2));
    let row = |label: String, value: String| format!(";     {label:<LABEL_WIDTH$}{value}\n");
    let sub = |label: String, value: String| format!(";         {label:<LABEL_WIDTH$}{value}\n");
    let mm = |v: f64| format!("{v:.1} mm");
    let mut h = String::new();
    h.push_str(&rule);
    h.push_str(&format!(";   CncCraft {}  ·  G-Code\n", env!("CARGO_PKG_VERSION")));
    h.push_str(";   https://github.com/freegroup/CncCraft\n");
    h.push_str(&rule);

    h.push_str(";\n");
    h.push_str(&format!(";   {}\n", t!("DATEI")));
    h.push_str(&thin);
    h.push_str(&row(t!("Erstellt"), chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()));
    h.push_str(&row(t!("Postprozessor"), pp.label()));
    if let Some(scope) = &meta.scope {
        h.push_str(&row(t!("Export"), scope.clone()));
    }
    if let Some((n, m)) = meta.file {
        h.push_str(&row(t!("Datei"), t!("%{n} von %{m}", n = n, m = m)));
    }

    h.push_str(";\n");
    h.push_str(&format!(";   {}\n", t!("WERKSTÜCK")));
    h.push_str(&thin);
    h.push_str(&row(t!("Größe (X × Y × Z)"), format!("{:.1} × {:.1} × {:.1} mm", job.width, job.height, job.thickness)));
    h.push_str(&row(t!("Material"), crate::tools::material_label(&job.material)));
    h.push_str(&row(
        t!("Nullpunkt XY"),
        match job.datum {
            crate::toolpath::Datum::LowerLeft => t!("unten links"),
            crate::toolpath::Datum::Center => t!("Mitte"),
        },
    ));
    h.push_str(&row(
        t!("Nullpunkt Z"),
        match job.z_zero {
            crate::toolpath::ZZero::MaterialSurface => t!("Materialoberfläche"),
            crate::toolpath::ZZero::MachineBed => t!("Maschinentisch"),
        },
    ));
    h.push_str(&row(t!("Sichere Höhe"), mm(job.safe_z)));
    h.push_str(&row(t!("Referenzhöhe"), mm(job.home_z)));
    h.push_str(&row(t!("Einheiten"), if job.inches { t!("Zoll (G20)") } else { t!("Millimeter (G21)") }));
    if job.inches {
        h.push_str(&row(String::new(), t!("Maße in diesem Kopf in mm")));
    }

    // every tool once, in the order it is first used
    let mut tools: Vec<&crate::tools::Tool> = vec![];
    for tp in paths {
        if !tools.contains(&&tp.tool) {
            tools.push(&tp.tool);
        }
    }
    h.push_str(";\n");
    h.push_str(&format!(";   {}\n", t!("WERKZEUGE")));
    h.push_str(&thin);
    for (n, tool) in tools.iter().enumerate() {
        if n > 0 {
            h.push_str(";\n");
        }
        h.push_str(&format!(";     T{}  {}\n", tool.number, crate::tools::shown_name(&tool.name)));
        h.push_str(&sub(t!("Typ"), tool.kind.group()));
        h.push_str(&sub(t!("Durchmesser"), mm(tool.diameter)));
        if matches!(tool.kind, crate::tools::ToolKind::VBit | crate::tools::ToolKind::Drill) {
            h.push_str(&sub(t!("Spitzenwinkel"), format!("{:.0}°", tool.angle)));
        }
        h.push_str(&sub(t!("Drehzahl"), format!("{} {}", tool.rpm, t!("U/min"))));
        h.push_str(&sub(t!("Vorschub"), format!("{:.0} mm/min", tool.feed)));
        h.push_str(&sub(t!("Eintauchen"), format!("{:.0} mm/min", tool.plunge)));
        h.push_str(&sub(t!("Zustellung"), mm(tool.stepdown)));
        h.push_str(&sub(t!("Querzustellung"), mm(tool.stepover)));
    }

    h.push_str(";\n");
    h.push_str(&format!(";   {}\n", t!("WERKZEUGWEGE")));
    h.push_str(&thin);
    // a table: one line for each toolpath, the columns as wide as their longest entry
    let mut table: Vec<[String; 6]> = vec![[t!("Nr."), t!("Name"), t!("Strategie"), t!("Werkzeug"), t!("Tiefe (Start → Schnitt)"), t!("Zeit")]];
    let mut total = 0.0;
    for (n, tp) in paths.iter().enumerate() {
        let minutes = tp.stats().minutes;
        total += minutes;
        table.push([(n + 1).to_string(), tp.name.clone(), tp.strategy.label(), format!("T{}", tp.tool.number), format!("{:.1} → {:.1} mm", tp.start_depth, tp.cut_depth), minutes_text(minutes)]);
    }
    let widths: Vec<usize> = (0..6).map(|c| table.iter().map(|r| r[c].chars().count()).max().unwrap_or(0)).collect();
    for (i, r) in table.iter().enumerate() {
        let cells: Vec<String> = r.iter().zip(&widths).map(|(cell, w)| format!("{cell:<w$}")).collect();
        h.push_str(&format!(";     {}\n", cells.join("   ").trim_end()));
        if i == 0 {
            h.push_str(&format!(";     {}\n", widths.iter().map(|w| "-".repeat(*w)).collect::<Vec<_>>().join("   ")));
        }
    }
    h.push_str(";\n");
    h.push_str(&format!(";     {}  {}\n", t!("Gesamte Fräszeit:"), minutes_text(total)));
    h.push_str(&rule);
    h
}

fn build(job: &Job, paths: &[&Toolpath], pp: &PostProcessor, meta: &ExportMeta) -> (String, Vec<usize>) {
    let (k, [dx, dy, dz]) = frame(job);
    let f = |v: f64| format!("{:.3}", v * k);
    let mut sections = Vec::with_capacity(paths.len());
    let mut s = String::new();
    s.push_str(&header(job, paths, pp, meta));
    s.push_str("\n");
    s.push_str(if job.inches { "G20\n" } else { "G21\n" });
    s.push_str("G90 G17 G40 G49 G80 G94\n");

    if let Some(first) = paths.first() {
        let ctx = HookCtx { tool_number: first.tool.number, rpm: first.tool.rpm, safe_z: job.safe_z + dz, home_z: job.home_z + dz, k };
        let block = expand(&pp.job_start, &ctx);
        if !block.is_empty() {
            s.push_str(&t!("; -- Job Start --\n"));
            s.push_str(&block);
        }
    }

    let mut first = true;
    for tp in paths {
        let ctx = HookCtx { tool_number: tp.tool.number, rpm: tp.tool.rpm, safe_z: job.safe_z + dz, home_z: job.home_z + dz, k };

        if !first && pp.tool_change {
            let block = expand(&pp.on_tool_change, &ctx);
            if !block.is_empty() {
                s.push_str(&t!("; -- Werkzeugwechsel --\n"));
                s.push_str(&block);
            } else {
                s.push_str("M5\n");
                s.push_str(&format!("T{} M6\n", tp.tool.number));
            }
        } else if first && pp.tool_change {
            s.push_str(&format!("T{} M6\n", tp.tool.number));
        }

        sections.push(s.bytes().filter(|b| *b == b'\n').count());
        s.push_str(&format!("; {} – {}\n", tp.name, crate::tools::shown_name(&tp.tool.name)));
        s.push_str(&format!("S{} M3\n", tp.tool.rpm));

        let block = expand(&pp.before_path, &ctx);
        if !block.is_empty() {
            s.push_str(&block);
        }

        first = false;
        let mut last_f = -1.0;
        let mut prev = [f64::NAN; 3];
        for m in &tp.moves {
            let p = [m.to[0] + dx, m.to[1] + dy, m.to[2] + dz];
            let mut line = String::from(if m.rapid { "G0" } else { "G1" });
            for (i, a) in ["X", "Y", "Z"].iter().enumerate() {
                if prev[i].is_nan() || (prev[i] - p[i]).abs() > 1e-6 {
                    line.push_str(&format!(" {a}{}", f(p[i])));
                }
            }
            if line.len() == 2 {
                continue;
            }
            if !m.rapid {
                let plunge = (prev[0] - p[0]).abs() < 1e-6 && (prev[1] - p[1]).abs() < 1e-6;
                let rate = if plunge { tp.tool.plunge } else { tp.tool.feed };
                if (rate - last_f).abs() > 1e-6 {
                    line.push_str(&format!(" F{:.1}", rate * k));
                    last_f = rate;
                }
            }
            s.push_str(&line);
            s.push('\n');
            prev = p;
        }

        let block = expand(&pp.after_path, &ctx);
        if !block.is_empty() {
            s.push_str(&block);
        }
    }

    if let Some(last) = paths.last() {
        let ctx = HookCtx { tool_number: last.tool.number, rpm: last.tool.rpm, safe_z: job.safe_z + dz, home_z: job.home_z + dz, k };
        let block = expand(&pp.job_end, &ctx);
        if !block.is_empty() {
            s.push_str(&t!("; -- Job Ende --\n"));
            s.push_str(&block);
        }
    }

    s.push_str("M30\n");
    (s, sections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gcode::builtins;
    use crate::toolpath::{self, Move, Strategy};
    use crate::tools::{Tool, ToolKind};

    fn job() -> Job {
        Job::default()
    }

    fn tool() -> Tool {
        Tool { name: "Test".into(), kind: ToolKind::EndMill, diameter: 6.0, angle: 0.0, feed: 1000.0, plunge: 300.0, rpm: 18000, stepdown: 2.0, stepover: 2.4, number: 1 }
    }

    fn simple_toolpath() -> Toolpath {
        Toolpath {
            name: "Profil".into(),
            strategy: Strategy::Profile { side: toolpath::Side::Outside, climb: true, tabs: false, tab_count: 0 },
            tool: tool(),
            start_depth: 0.0,
            cut_depth: 3.0,
            start_expr: None,
            cut_expr: None,
            shape_ids: vec![],
            visible: true,
            moves: vec![
                Move { to: [0.0, 0.0, 10.0], rapid: true },
                Move { to: [0.0, 0.0, 5.0], rapid: true },
                Move { to: [50.0, 30.0, 5.0], rapid: true },
                Move { to: [50.0, 30.0, 0.5], rapid: true },
                Move { to: [50.0, 30.0, -2.0], rapid: false },
                Move { to: [80.0, 30.0, -2.0], rapid: false },
                Move { to: [80.0, 30.0, 5.0], rapid: true },
                Move { to: [0.0, 0.0, 5.0], rapid: true },
                Move { to: [0.0, 0.0, 10.0], rapid: true },
            ],
            warning: None,
        }
    }

    /// A profile toolpath around a 20 × 10 rectangle at `centre`, worked out for real.
    fn profile_around(id: u64, centre: [f64; 2], items: &mut Vec<shape_edit::Item>, job: &Job) -> Toolpath {
        items.push(shape_edit::Item::new(id, shape_edit::Rect::new(centre, 20.0, 10.0)));
        let mut tp = simple_toolpath();
        tp.shape_ids = vec![id];
        tp.compute(job, items);
        tp
    }


    #[test]
    fn toolpaths_never_visit_home_and_only_travel_at_safe_z() {
        let j = job();
        let mut items = vec![];
        let a = profile_around(1, [50.0, 50.0], &mut items, &j);
        let b = profile_around(2, [150.0, 100.0], &mut items, &j);
        for tp in [&a, &b] {
            assert!(!tp.moves.is_empty());
            assert!(tp.moves[0].rapid && (tp.moves[0].to[2] - j.safe_z).abs() < 1e-9, "starts by going to safe_z over the first cut: {:?}", tp.moves[0]);
            assert!(tp.moves.iter().all(|m| m.to[0].hypot(m.to[1]) > 1.0), "no move to 0/0");
            // sideways rapids happen at safe_z or above, never lower
            let mut prev = tp.moves[0].to;
            for m in &tp.moves {
                if m.rapid && (m.to[0] != prev[0] || m.to[1] != prev[1]) {
                    assert!(prev[2] >= j.safe_z - 1e-9 && m.to[2] >= j.safe_z - 1e-9, "sideways rapid below safe_z: {prev:?} -> {:?}", m.to);
                }
                prev = m.to;
            }
            let (last, before) = (tp.moves[tp.moves.len() - 1], tp.moves[tp.moves.len() - 2]);
            assert!(last.rapid && (last.to[2] - j.safe_z).abs() < 1e-9 && last.to[..2] == before.to[..2], "ends lifting straight up to safe_z: {last:?}");
        }
        // then the file: from one toolpath to the next, and at the end, the machine never goes to 0/0
        let code = write(&j, &[&a, &b], &builtins()[1]);
        assert!(!code.contains("X0.000 Y0.000"), "{code}");
        let lines: Vec<&str> = code.lines().collect();
        let travel = lines.iter().position(|l| l.starts_with("G0 X")).expect("a first travel move");
        assert!(lines[travel].contains(&format!("Z{:.3}", j.safe_z)), "home to the first toolpath runs at safe_z: {}", lines[travel]);
        let home = lines.iter().position(|l| l.starts_with(&format!("G0 Z{:.3}", j.home_z))).expect("the start hook goes to home height");
        assert!(home < travel, "home height first, then the travel to the first toolpath");
    }

    #[test]
    fn the_written_code_read_back_shows_home_and_the_travel() {
        use crate::transform::from_gcode::{trace_job, Kind};
        let mut j = job();
        let mut items = vec![];
        let a = profile_around(1, [50.0, 50.0], &mut items, &j);
        let b = profile_around(2, [150.0, 100.0], &mut items, &j);
        let (code, sections) = write_with_sections(&j, &[&a, &b], &builtins()[1]);
        assert_eq!(sections.len(), 2);
        assert!(code.lines().nth(sections[1]).is_some_and(|l| l.starts_with("; Profil")), "the section starts at the toolpath's own comment");

        let steps = trace_job(&code, &j);
        // the first move: from home (work origin, home height) at safe_z to the first cut
        let first = steps[0];
        assert_eq!((first.kind, first.from), (Kind::Rapid, [0.0, 0.0, j.home_z]));
        assert!((first.to[2] - j.safe_z).abs() < 1e-9 && first.to[0].hypot(first.to[1]) > 1.0, "{first:?}");
        // travelling sideways never happens below safe_z
        for s in steps.iter().filter(|s| s.kind == Kind::Rapid && (s.from[..2] != s.to[..2])) {
            assert!(s.from[2] >= j.safe_z - 1e-9 && s.to[2] >= j.safe_z - 1e-9, "{s:?}");
        }
        // between the two toolpaths the cutter goes from the end of the first to the start of the second
        let between = steps.iter().find(|s| s.line >= sections[1] && s.kind == Kind::Rapid && s.from[..2] != s.to[..2]).expect("a travel move to the second toolpath");
        assert!(between.from[0].hypot(between.from[1]) > 1.0, "not from 0/0: {between:?}");
        assert!((between.to[0] - 150.0).abs() < 25.0 && (between.to[1] - 100.0).abs() < 25.0, "{between:?}");

        // the datum in the middle and inches change the numbers in the file, not what is drawn
        j.datum = crate::toolpath::Datum::Center;
        j.inches = true;
        let (code, _) = write_with_sections(&j, &[&a, &b], &builtins()[1]);
        assert!(code.contains("G20"));
        let steps2 = trace_job(&code, &j);
        assert_eq!(steps2.len(), steps.len());
        assert!((steps2[0].from[0] - 150.0).abs() < 1e-9 && (steps2[0].from[1] - 100.0).abs() < 1e-9, "home is the middle of the plate: {:?}", steps2[0]);
        // three decimals of an inch are 0.025 mm, so the rounding may be off by half of that
        assert!((steps2[0].to[0] - first.to[0]).abs() < 0.02 && (steps2[0].to[1] - first.to[1]).abs() < 0.02, "the same first cut: {:?} and {:?}", steps2[0].to, first.to);
    }


    #[test]
    fn every_comment_is_a_semicolon_comment_and_the_header_tells_the_job() {
        rust_i18n::set_locale("de");
        let mut j = job();
        j.datum = crate::toolpath::Datum::Center;
        let tp = simple_toolpath();
        let mut pp = builtins()[0].clone();
        pp.before_path = "( Vor dem Weg )\nM8 (Kühlung)".into();
        let meta = ExportMeta { scope: Some("Sichtbare Werkzeugwege (1)".into()), file: Some((1, 2)) };
        let code = write_with_meta(&j, &[&tp], &pp, &meta);
        println!("{code}");
        assert!(code.lines().all(|l| !l.contains('(') || l.trim_start().starts_with(';') || l.contains(" ; ")), "a parenthesis comment is left:\n{code}");
        for needle in ["https://github.com/freegroup/CncCraft", env!("CARGO_PKG_VERSION"), "Erstellt", "300.0 × 200.0 × 18.0 mm", "Mitte", "T1", "Drehzahl", "18000", "1 von 2", "Sichtbare Werkzeugwege (1)"] {
            assert!(code.contains(needle), "header lacks {needle}:\n{code}");
        }
        assert!(code.contains("M8 ; Kühlung") && code.contains("\n; Vor dem Weg\n"), "{code}");
    }

    #[test]
    fn the_times_of_the_moves_add_up_to_the_milling_time() {
        let tp = simple_toolpath();
        let sum: f64 = (0..tp.moves.len()).map(|i| tp.move_seconds(i)).sum();
        assert!(sum > 0.0 && (sum - tp.stats().minutes * 60.0).abs() < 1e-6, "{sum} s against {} min", tp.stats().minutes);
    }

    #[test]
    fn hooks_are_injected() {
        let j = job();
        let tp = simple_toolpath();
        let mut pp = builtins()[0].clone();
        pp.job_start = "M8 (Kühlung an)".into();
        pp.job_end = "M9 (Kühlung aus)".into();
        let code = write(&j, &[&tp], &pp);
        assert!(code.contains("M8 ; Kühlung an"), "job_start hook missing");
        assert!(code.contains("M9 ; Kühlung aus"), "job_end hook missing");
    }

    #[test]
    fn placeholders_are_expanded() {
        let j = job();
        let tp = simple_toolpath();
        let mut pp = builtins()[0].clone();
        pp.before_path = "G0 Z{safe_z}\n( Werkzeug {tool_number}, {rpm} U/min )".into();
        let code = write(&j, &[&tp], &pp);
        assert!(code.contains("G0 Z5.000"), "safe_z placeholder not expanded");
        assert!(code.contains("Werkzeug 1, 18000 U/min"), "tool/rpm placeholder not expanded");
    }

    #[test]
    fn travel_uses_separate_z_and_xy() {
        let j = job();
        let tp = simple_toolpath();
        let code = write(&j, &[&tp], &builtins()[0]);
        assert!(code.contains("G0 Z5.000\n"), "must lower to safe_z first");
        assert!(code.contains("G0 X50.000 Y30.000\n"), "then travel XY at safe_z");
    }
}
