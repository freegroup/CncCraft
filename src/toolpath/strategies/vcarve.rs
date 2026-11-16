//! V-carve tool: with a V-bit, the depth at a point is its distance from the shape edge divided
//! by tan(half the tip angle). Approximated by cutting inward-offset rings, each at its depth,
//! down to the cutter's reach or an optional flat-depth limit.

use crate::toolpath::{ring_dir, CutContext, Out, ToolpathKind};
use crate::geom::{Region, Ring};

pub struct VCarveTool {
    /// 0 = limited only by the cutter diameter.
    pub max_depth: f64,
}

impl ToolpathKind for VCarveTool {
    fn label(&self) -> String {
        t!("V-Carving")
    }

    fn cut(&self, out: &mut Out, cx: &CutContext) -> Option<String> {
        let region = Region::from_contours(cx.contours);
        let half = (cx.tool.angle.max(10.0) / 2.0).to_radians();
        let reach = cx.radius();
        let limit_off = reach.min(if self.max_depth > 0.0 { self.max_depth * half.tan() } else { f64::MAX });
        let step = cx.tool.stepover.max(0.1);
        let mut d = step;
        let mut levels: Vec<(f64, Vec<Ring>)> = vec![];
        while d <= limit_off + 1e-9 {
            let rings = region.offset(-d).rings();
            if rings.is_empty() {
                break;
            }
            // depth = horizontal distance from the edge / tan(half angle)
            levels.push((-(d / half.tan()), rings));
            d += step;
        }
        let floor = -cx.cut_depth.max(0.01);
        for (z, rings) in levels {
            for ring in &rings {
                out.cut_ring(&ring_dir(ring, true), z.max(floor).min(0.0), None);
            }
            out.retract();
        }
        None
    }
}
