//! Writing toolpaths as machine files (G-code).

use crate::toolpath::{Datum, Job, Toolpath, ZZero};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Post {
    GcodeMm,
    GcodeInch,
    Grbl,
}

impl Post {
    pub const ALL: [Post; 3] = [Post::GcodeMm, Post::GcodeInch, Post::Grbl];
    pub fn label(&self) -> &'static str {
        match self {
            Post::GcodeMm => "G-Code (mm) (*.nc)",
            Post::GcodeInch => "G-Code (Zoll) (*.tap)",
            Post::Grbl => "GRBL (mm) (*.gcode)",
        }
    }
    /// Whether the dialect can change tools inside one file. The original refuses to write several
    /// tools into one file when it cannot ("... the selected Post-Processor does not support
    /// toolchanging", McPP catalogue); GRBL has no tool changer.
    pub fn supports_tool_change(&self) -> bool {
        *self != Post::Grbl
    }

    pub fn ext(&self) -> &'static str {
        match self {
            Post::GcodeMm => "nc",
            Post::GcodeInch => "tap",
            Post::Grbl => "gcode",
        }
    }
}

pub fn write(job: &Job, paths: &[&Toolpath], post: Post) -> String {
    let inch = post == Post::GcodeInch;
    let k = if inch { 1.0 / 25.4 } else { 1.0 };
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
    s.push_str(if inch { "G20\n" } else { "G21\n" });
    // safe start state: absolute, XY plane, cancel cutter comp and canned cycles, feed per minute
    s.push_str("G90 G17 G40 G49 G80 G94\n");
    let mut first = true;
    for tp in paths {
        s.push_str(&format!("( {} – {} )\n", tp.name, tp.tool.name));
        if !first {
            s.push_str("M5\n");
        }
        if post.supports_tool_change() {
            s.push_str(&format!("T{} M6\n", tp.tool.number));
        }
        s.push_str(&format!("S{} M3\n", tp.tool.rpm));
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
    }
    s.push_str("M5\n");
    s.push_str(&format!("G0 Z{}\n", f(job.home_z + dz)));
    s.push_str(&format!("G0 X{} Y{}\n", f(dx), f(dy)));
    s.push_str("M30\n");
    s
}

#[cfg(test)]
mod tests {
    use super::Post;

    #[test]
    fn only_grbl_has_no_tool_change() {
        assert!(Post::GcodeMm.supports_tool_change() && Post::GcodeInch.supports_tool_change());
        assert!(!Post::Grbl.supports_tool_change());
    }
}
