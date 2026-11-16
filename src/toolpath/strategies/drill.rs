//! Drill tool: at the centre of each selected closed contour the drill goes down in steps and comes
//! right out of the hole after each one, so the chips are carried out. Never in one go.
//!
//! The rules are the original's peck routine (Spark `0x10124baec`, called per hole from
//! `0x10124b870`; see MECHANICS.md, Nachtrag "Bohren aus dem Original"):
//! - the depth is split into n equal steps: n = max(1, round(depth / peck)), one more when the rest
//!   depth − n·peck is over 1 % of the peck; each step is depth / n (never longer than asked for by
//!   more than the rounding allows, and no short last step);
//! - every step is one plunge at plunge feed from where the drill is down to the step's depth;
//! - after every step but the last it goes straight up at rapid to the retract height above the
//!   material ("Retract above model"), which is never higher than safe Z (`0x10124b7a4`);
//! - after the last step no extra lift: the hole's normal retract follows.

use crate::toolpath::{CutContext, Out, ToolpathKind, RETRACT_GAP};

/// A step longer than the peck by less than this share of it doesn't need another step (Spark: 0.01).
const PECK_TOLERANCE: f64 = 0.01;

/// The depths below `top` the drill steps to: `total` split into equal steps no longer than about `peck`.
pub fn peck_depths(top: f64, total: f64, peck: f64) -> Vec<f64> {
    if total <= 1e-9 {
        return vec![];
    }
    let peck = peck.max(1e-6);
    let mut n = ((total / peck + 0.5) as i64).max(1);
    if total - peck * n as f64 > peck * PECK_TOLERANCE {
        n += 1;
    }
    let step = total / n as f64;
    (1..=n).map(|i| top - step * i as f64).collect()
}

pub struct DrillTool;

impl ToolpathKind for DrillTool {
    fn label(&self) -> String {
        t!("Bohren")
    }

    fn cut(&self, out: &mut Out, cx: &CutContext) -> Option<String> {
        let top = -cx.start_depth;
        let steps = peck_depths(top, cx.cut_depth - cx.start_depth, cx.tool.stepdown);
        let retract_z = out.safe_z().min(RETRACT_GAP);
        for c in cx.contours.iter().filter(|c| c.closed) {
            let Some((lo, hi)) = crate::geom::bounds_of(std::slice::from_ref(c)) else { continue };
            let p = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
            out.go_to([p[0], p[1], top]);
            for (i, &z) in steps.iter().enumerate() {
                out.feed([p[0], p[1], z]);
                if i + 1 < steps.len() {
                    out.rapid([p[0], p[1], retract_z]);
                }
            }
            out.retract();
        }
        None
    }
}
