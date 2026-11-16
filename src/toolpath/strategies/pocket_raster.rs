//! Pocket cleared by back-and-forth lines at a chosen angle, then one round along the wall. The
//! cutter stays down from one line to the next where they neighbour inside the pocket, and goes
//! over to the wall round without lifting.

use super::pocket::Pocket;
use crate::geom::{dist, Pt, Ring};
use crate::toolpath::{raster_segments, ring_dir, CutContext, Out};

fn rot(p: Pt, s: f64, c: f64) -> Pt {
    [p[0] * c - p[1] * s, p[0] * s + p[1] * c]
}

pub(super) fn cut(out: &mut Out, cx: &CutContext, pocket: &Pocket, depths: &[f64], angle: f64) {
    let rings = pocket.inset.rings();
    let step = pocket.step;
    let (s, c) = angle.to_radians().sin_cos();
    // the lines are found with the pocket turned so they run along X, then turned back
    let scan_rings: Vec<Ring> = rings.iter().map(|rg| Ring { pts: rg.pts.iter().map(|p| rot(*p, -s, c)).collect(), hole: rg.hole }).collect();
    let segs: Vec<(Pt, Pt)> = raster_segments(&scan_rings, step).into_iter().map(|(a, b)| (rot(a, s, c), rot(b, s, c))).collect();
    let stay = |a, b| pocket.stay_down(a, b);
    for (i, &z) in depths.iter().enumerate() {
        out.set_top(cx.top_of(depths, i));
        let mut last: Option<Pt> = None;
        for &(a, b) in &segs {
            if last.is_some_and(|l| dist(l, a) < step * 1.6 && pocket.stay_down(l, a)) {
                out.feed([a[0], a[1], z]);
            } else {
                out.enter_along(a, b, z);
            }
            out.feed([b[0], b[1], z]);
            last = Some(b);
        }
        for ring in &rings {
            out.cut_ring_linked(&ring_dir(ring, pocket.climb), z, &stay);
        }
        out.retract();
    }
}
