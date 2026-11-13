//! Drill tool: plunge at the centre of each selected closed contour, pecking in steps and
//! retracting between pecks to clear chips.

use super::{CutContext, Out, ToolpathKind, RETRACT_GAP};

pub struct DrillTool;

impl ToolpathKind for DrillTool {
    fn label(&self) -> String {
        t!("Bohren")
    }

    fn cut(&self, out: &mut Out, cx: &CutContext) -> Option<String> {
        let peck = cx.tool.stepdown.max(0.1);
        for c in cx.contours.iter().filter(|c| c.closed) {
            let Some((lo, hi)) = crate::geom::bounds_of(std::slice::from_ref(c)) else { continue };
            let p = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
            let mut z = -cx.start_depth;
            out.go_to([p[0], p[1], z]);
            while z > -cx.cut_depth + 1e-9 {
                z = (z - peck).max(-cx.cut_depth);
                out.feed([p[0], p[1], z]);
                if z > -cx.cut_depth + 1e-9 {
                    out.rapid([p[0], p[1], RETRACT_GAP]);
                    out.rapid([p[0], p[1], z + 0.2]);
                }
            }
            out.retract();
        }
        None
    }
}
