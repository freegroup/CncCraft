//! Pocket cleared by one continuous spiral (like a "vase mode"): the cutter enters once in the
//! middle, goes round the innermost ring, then every turn moves over smoothly into the next ring
//! outwards, without a new start, and ends with a round along the wall.
//!
//! A turn blends the inner ring into the outer one: at a share t of the way round it is at
//! (1 - t) of the inner ring's point and t of the outer ring's point at the same share. Where the
//! pocket splits or has islands, those inner parts are cleared ring by ring first (as with offset
//! rings); a turn whose blend would leave the area the cutter may go is cut as a ring instead.

use super::pocket::Pocket;
use super::pocket_offset::cut_levels;
use crate::geom::{dist, perimeter, Pt};
use crate::toolpath::{ring_dir, start_nearest, CutContext, Out};

pub(super) fn cut(out: &mut Out, cx: &CutContext, pocket: &Pocket, depths: &[f64]) {
    let levels = pocket.offset_levels();
    // the outer levels that are one plain ring each make the spiral
    let run = levels.iter().take_while(|l| l.len() == 1 && !l[0].hole).count();
    let rings: Vec<Vec<Pt>> = levels[..run].iter().map(|l| ring_dir(&l[0], pocket.climb)).collect();
    let stay = |a, b| pocket.stay_down(a, b);
    for (i, &z) in depths.iter().enumerate() {
        out.set_top(cx.top_of(depths, i));
        cut_levels(out, pocket, &levels[run..], z);
        if let Some(inner) = rings.last() {
            out.cut_ring_linked(inner, z, &stay);
            for k in (1..rings.len()).rev() {
                let cur = out.cur();
                let from = start_nearest(&rings[k], [cur[0], cur[1]]);
                let to = start_nearest(&rings[k - 1], from[0]);
                let turn = blend(&from, &to, pocket.step);
                if out.is_down_at(z) && turn.iter().all(|p| pocket.may_go(*p)) {
                    out.feed_along(&turn, z);
                } else {
                    out.cut_ring_linked(&rings[k - 1], z, &stay);
                }
            }
            if rings.len() > 1 {
                // the turns only touch the wall at their end: one round along it to finish
                out.cut_ring_linked(&rings[0], z, &stay);
            }
        }
        out.retract();
    }
}

/// One turn from ring `a` over into ring `b` (both start where the turn starts and ends, and run
/// the same way round), as points no further apart than about half a stepover.
fn blend(a: &[Pt], b: &[Pt], step: f64) -> Vec<Pt> {
    let n = ((perimeter(b, true).max(perimeter(a, true)) / (step * 0.5).max(0.05)).ceil() as usize).clamp(32, 4000);
    let (ca, cb) = (cumulative(a), cumulative(b));
    (1..=n)
        .map(|k| {
            let t = k as f64 / n as f64;
            let (p, q) = (at(a, &ca, t), at(b, &cb, t));
            [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]
        })
        .collect()
}

/// The arc length of a closed ring at each of its points, and the whole length at the end.
fn cumulative(pts: &[Pt]) -> Vec<f64> {
    let n = pts.len();
    let mut cum = vec![0.0; n + 1];
    for i in 0..n {
        cum[i + 1] = cum[i] + dist(pts[i], pts[(i + 1) % n]);
    }
    cum
}

/// The point at share `t` (0..=1) of the way round a closed ring.
fn at(pts: &[Pt], cum: &[f64], t: f64) -> Pt {
    let n = pts.len();
    let s = cum[n] * t.clamp(0.0, 1.0);
    let i = cum[..n].partition_point(|&c| c <= s).saturating_sub(1).min(n - 1);
    let seg = (cum[i + 1] - cum[i]).max(1e-12);
    let f = ((s - cum[i]) / seg).clamp(0.0, 1.0);
    let (a, b) = (pts[i], pts[(i + 1) % n]);
    [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_turn_starts_on_the_inner_ring_and_ends_on_the_outer_one() {
        let inner = vec![[4.0, 4.0], [6.0, 4.0], [6.0, 6.0], [4.0, 6.0]];
        let outer = vec![[3.0, 3.0], [7.0, 3.0], [7.0, 7.0], [3.0, 7.0]];
        let turn = blend(&inner, &outer, 1.0);
        assert!(dist(turn[0], inner[0]) < 0.5, "starts next to the inner ring: {:?}", turn[0]);
        assert!(dist(*turn.last().unwrap(), outer[0]) < 1e-9, "ends on the outer ring's start");
        // it never steps further than the turn's spacing: no jumps
        assert!(turn.windows(2).all(|w| dist(w[0], w[1]) < 1.0));
    }
}
