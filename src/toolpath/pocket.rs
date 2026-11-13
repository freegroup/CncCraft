//! Pocket tool: clear the inside of a region, either as concentric offset rings or as a raster
//! fill (at a chosen angle) plus a finishing boundary pass.

use super::{raster_segments, ring_dir, CutContext, Out, ToolpathKind};
use crate::geom::{dist, Pt, Region, Ring};

pub struct PocketTool {
    pub raster: bool,
    pub climb: bool,
    /// Raster fill direction in degrees (0 = along X).
    pub raster_angle: f64,
}

fn rot(p: Pt, s: f64, c: f64) -> Pt {
    [p[0] * c - p[1] * s, p[0] * s + p[1] * c]
}

impl ToolpathKind for PocketTool {
    fn label(&self) -> String {
        t!("Tasche")
    }

    fn cut(&self, out: &mut Out, cx: &CutContext) -> Option<String> {
        let mut warning = None;
        let region = Region::from_contours(cx.contours);
        let r = cx.radius();
        let inset = region.offset(-r);
        let step = cx.tool.stepover.max(0.05);
        if inset.is_empty() && !cx.contours.is_empty() {
            warning = Some(t!("Werkzeug zu groß für diese Tasche").into());
        }
        for &z in &cx.depths() {
            if self.raster {
                let rings = inset.rings();
                // scan in a frame rotated by raster_angle, then rotate the segments back
                let (s, c) = self.raster_angle.to_radians().sin_cos();
                let scan_rings: Vec<Ring> = rings.iter().map(|rg| Ring { pts: rg.pts.iter().map(|p| rot(*p, -s, c)).collect(), hole: rg.hole }).collect();
                let segs = raster_segments(&scan_rings, step);
                let mut last: Option<Pt> = None;
                for (a, b) in segs {
                    let (a, b) = (rot(a, s, c), rot(b, s, c));
                    let mid = [(last.unwrap_or(a)[0] + a[0]) / 2.0, (last.unwrap_or(a)[1] + a[1]) / 2.0];
                    // stay down when the next row starts close by and still inside the pocket
                    if last.is_some_and(|l| dist(l, a) < step * 1.6 && inset.contains(mid)) {
                        out.feed([a[0], a[1], z]);
                    } else {
                        out.go_to([a[0], a[1], z]);
                    }
                    out.feed([b[0], b[1], z]);
                    last = Some(b);
                }
                for ring in &rings {
                    out.cut_ring(&ring_dir(ring, self.climb), z, None);
                }
            } else {
                // concentric offsets, cut inside-out so the boundary is the finishing pass
                let mut levels = vec![];
                let mut k = 0;
                loop {
                    let rings = region.offset(-r - step * k as f64).rings();
                    if rings.is_empty() || k > 2000 {
                        break;
                    }
                    levels.push(rings);
                    k += 1;
                }
                for rings in levels.iter().rev() {
                    for ring in rings {
                        out.cut_ring(&ring_dir(ring, self.climb), z, None);
                    }
                }
            }
            out.retract();
        }
        warning
    }
}
