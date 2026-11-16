//! Pocket cleared by concentric offset rings, from the middle out to the wall. Between
//! neighbouring rings the cutter stays down and feeds straight over to the next one; it only
//! lifts (and ramps in again) where a ring is not next to the last one, and for a new depth.

use super::pocket::Pocket;
use crate::geom::Ring;
use crate::toolpath::{ring_dir, CutContext, Out};

pub(super) fn cut(out: &mut Out, cx: &CutContext, pocket: &Pocket, depths: &[f64]) {
    let levels = pocket.offset_levels();
    for (i, &z) in depths.iter().enumerate() {
        out.set_top(cx.top_of(depths, i));
        cut_levels(out, pocket, &levels, z);
        out.retract();
    }
}

/// Cut `levels` (outermost first) at depth `z`, innermost first, each ring joined to the one
/// before without lifting where they neighbour.
pub(super) fn cut_levels(out: &mut Out, pocket: &Pocket, levels: &[Vec<Ring>], z: f64) {
    let stay = |a, b| pocket.stay_down(a, b);
    for rings in levels.iter().rev() {
        for ring in rings {
            out.cut_ring_linked(&ring_dir(ring, pocket.climb), z, &stay);
        }
    }
}
