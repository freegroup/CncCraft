//! Writing toolpaths as machine files (G-code) with configurable post-processors.

use crate::toolpath::{Datum, Job, Toolpath, ZZero};
use serde::{Deserialize, Serialize};

/// A complete post-processor definition: dialect settings plus injectable G-code hooks.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PostProcessor {
    pub name: String,
    #[serde(default)]
    pub inch: bool,
    pub ext: String,
    pub tool_change: bool,
    pub job_start: String,
    pub job_end: String,
    pub on_tool_change: String,
    pub before_path: String,
    pub after_path: String,
    pub builtin: bool,
}

impl PostProcessor {
    pub fn label(&self) -> String {
        format!("{} (*.{})", self.name, self.ext)
    }
}

pub fn builtins() -> Vec<PostProcessor> {
    vec![
        PostProcessor {
            name: "G-Code".into(),
            inch: false,
            ext: "nc".into(),
            tool_change: true,
            job_start: "G54              ; Werkstück-Koordinatensystem 1\nG0 Z{home_z}     ; Eilgang zur Referenzhöhe".into(),
            job_end: "G0 Z{safe_z}     ; Eilgang zur Sicherheitshöhe\nM5               ; Spindel AUS\nM9               ; Kühlmittel AUS".into(),
            on_tool_change: "G0 Z{safe_z}       ; Eilgang zur Sicherheitshöhe\nM5                 ; Spindel AUS\nT{tool_number} M6  ; Werkzeugwechsel\nM0                 ; Halt – weiter mit Cycle Start".into(),
            before_path: String::new(),
            after_path: "G0 Z{safe_z}     ; Eilgang zur Sicherheitshöhe".into(),
            builtin: true,
        },
        PostProcessor {
            name: "GRBL".into(),
            inch: false,
            ext: "gcode".into(),
            tool_change: false,
            job_start: "G54              ; Werkstück-Koordinatensystem 1\nG0 Z{home_z}     ; Eilgang zur Referenzhöhe".into(),
            job_end: "G0 Z{safe_z}     ; Eilgang zur Sicherheitshöhe\nM5               ; Spindel AUS".into(),
            on_tool_change: String::new(),
            before_path: String::new(),
            after_path: "G0 Z{safe_z}     ; Eilgang zur Sicherheitshöhe".into(),
            builtin: true,
        },
    ]
}

/// Backwards compatibility: deserialize the old Project format that stored `Post` enum + `PostHooks`.
pub fn migrate_old_format(hooks: &OldPostHooks) -> Vec<PostProcessor> {
    let mut list = builtins();
    let non_default = OldPostHooks::default();
    if hooks != &non_default {
        for pp in &mut list {
            pp.job_start = hooks.job_start.clone();
            pp.job_end = hooks.job_end.clone();
            pp.on_tool_change = hooks.tool_change.clone();
            pp.before_path = hooks.before_toolpath.clone();
            pp.after_path = hooks.after_toolpath.clone();
        }
    }
    list
}

/// The old PostHooks struct for backwards-compatible deserialization.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct OldPostHooks {
    #[serde(default)]
    pub job_start: String,
    #[serde(default)]
    pub job_end: String,
    #[serde(default)]
    pub tool_change: String,
    #[serde(default)]
    pub before_toolpath: String,
    #[serde(default)]
    pub after_toolpath: String,
}

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
            result.push_str(line.trim_start());
            result.push('\n');
        }
    }
    result
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

pub fn write(job: &Job, paths: &[&Toolpath], pp: &PostProcessor) -> String {
    let k = if job.inches { 1.0 / 25.4 } else { 1.0 };
    let (dx, dy) = match job.datum {
        Datum::LowerLeft => (0.0, 0.0),
        Datum::Center => (-job.width / 2.0, -job.height / 2.0),
    };
    let dz = match job.z_zero {
        ZZero::MaterialSurface => 0.0,
        ZZero::MachineBed => job.thickness,
    };
    let f = |v: f64| format!("{:.3}", v * k);
    let mut s = String::new();
    s.push_str(&format!("( CncCraft – {} Werkzeugweg(e) )\n", paths.len()));
    s.push_str(&format!(
        "( Material: {:.1} x {:.1} x {:.1} mm, Z-Null: {} )\n",
        job.width,
        job.height,
        job.thickness,
        if dz == 0.0 { "Materialoberfläche" } else { "Maschinentisch" }
    ));
    s.push_str(if job.inches { "G20\n" } else { "G21\n" });
    s.push_str("G90 G17 G40 G49 G80 G94\n");

    if let Some(first) = paths.first() {
        let ctx = HookCtx { tool_number: first.tool.number, rpm: first.tool.rpm, safe_z: job.safe_z + dz, home_z: job.home_z + dz, k };
        let block = expand(&pp.job_start, &ctx);
        if !block.is_empty() {
            s.push_str("( -- Job Start -- )\n");
            s.push_str(&block);
        }
    }

    let mut first = true;
    for tp in paths {
        let ctx = HookCtx { tool_number: tp.tool.number, rpm: tp.tool.rpm, safe_z: job.safe_z + dz, home_z: job.home_z + dz, k };

        if !first && pp.tool_change {
            let block = expand(&pp.on_tool_change, &ctx);
            if !block.is_empty() {
                s.push_str("( -- Werkzeugwechsel -- )\n");
                s.push_str(&block);
            } else {
                s.push_str("M5\n");
                s.push_str(&format!("T{} M6\n", tp.tool.number));
            }
        } else if first && pp.tool_change {
            s.push_str(&format!("T{} M6\n", tp.tool.number));
        }

        s.push_str(&format!("( {} – {} )\n", tp.name, tp.tool.name));
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
            s.push_str("( -- Job Ende -- )\n");
            s.push_str(&block);
        }
    }

    s.push_str("M30\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn only_grbl_has_no_tool_change() {
        let b = builtins();
        assert!(b[0].tool_change);
        assert!(!b[1].tool_change);
    }

    #[test]
    fn hooks_are_injected() {
        let j = job();
        let tp = simple_toolpath();
        let mut pp = builtins()[0].clone();
        pp.job_start = "M8 (Kühlung an)".into();
        pp.job_end = "M9 (Kühlung aus)".into();
        let code = write(&j, &[&tp], &pp);
        assert!(code.contains("M8 (Kühlung an)"), "job_start hook missing");
        assert!(code.contains("M9 (Kühlung aus)"), "job_end hook missing");
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
