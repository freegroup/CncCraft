//! Transformers between the program's own toolpaths and machine files. Each direction has its own
//! file: `to_gcode` writes toolpaths as G-code with the post-processor's hooks, `from_gcode` reads
//! G-code back into the machine's moves (to draw what the post-processor really wrote). Both rest
//! on the same relation between the job's coordinates and the machine's, [`frame`].

pub mod from_gcode;
pub mod to_gcode;

use crate::toolpath::{Datum, Job, ZZero};

/// How the G-code's coordinates relate to the job's: a job point `p` is written as `(p + offset) * k`
/// (`k` is 1 for millimetres, 1/25.4 for inches; the offset moves the zero point to the datum and
/// the height zero to the machine bed).
pub fn frame(job: &Job) -> (f64, [f64; 3]) {
    let k = if job.inches { 1.0 / 25.4 } else { 1.0 };
    let (dx, dy) = match job.datum {
        Datum::LowerLeft => (0.0, 0.0),
        Datum::Center => (-job.width / 2.0, -job.height / 2.0),
    };
    let dz = match job.z_zero {
        ZZero::MaterialSurface => 0.0,
        ZZero::MachineBed => job.thickness,
    };
    (k, [dx, dy, dz])
}
