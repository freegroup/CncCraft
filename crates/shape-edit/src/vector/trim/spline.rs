//! Trimming a Bezier spline: it keeps its **exact** curve. The spline is cut at every crossing and
//! falls apart into open sub-splines, each an exact de-Casteljau piece of the original cubics — the
//! pieces together trace the very same path, so a split never reshapes the curve (the hard case the
//! user warned about). Crossing parameters are found on the real control points, never on the
//! faceted preview.

use super::Cutters;
use crate::forms::{cubic, Spline, SplineNode};
use crate::Pt;

/// A spline cut at every crossing with the cutters, as open sub-splines. Nothing when nothing
/// crosses. This is what [`crate::Spline`]'s `Form::trim` calls.
pub fn split_spline_at_crossings(spline: &Spline, cutters: &Cutters) -> Vec<Spline> {
    let cuts = crossing_params(spline, cutters);
    if cuts.is_empty() {
        return vec![];
    }
    split_at(spline, &cuts)
}

/// Cut `spline` at the parameters in `cuts` — `(segment index, t in 0..1 along that segment)` —
/// into open sub-splines, without moving the curve: every piece is an exact de-Casteljau split of
/// the original cubics. A closed spline opens at the first cut and runs once round; an open one
/// keeps its own two ends. Fewer than one cut leaves it whole.
pub fn split_at(spline: &Spline, cuts: &[(usize, f64)]) -> Vec<Spline> {
    let nodes = &spline.nodes;
    let n = nodes.len();
    if n < 2 || cuts.is_empty() {
        return vec![spline.clone()];
    }
    let seg_count = if spline.closed { n } else { n - 1 };
    let mut sorted = cuts.to_vec();
    sorted.sort_by(|a, b| (a.0, a.1).partial_cmp(&(b.0, b.1)).unwrap());
    // the segments in traversal order: a ring opens at the first cut's segment
    let start = if spline.closed { sorted[0].0 } else { 0 };
    let order: Vec<usize> = (0..seg_count).map(|i| (start + i) % n).collect();
    // each `Piece` is a Bezier sub-cubic (p0, p1, p2, p3) plus whether it opens a NEW spline at its
    // start. A spline boundary is a cut point, so every piece after a cut (and the opened ring's
    // seam) starts fresh; a plain segment boundary does not.
    struct Piece {
        c: [Pt; 4],
        breaks_before: bool,
    }
    let mut pieces: Vec<Piece> = Vec::new();
    for (idx, &si) in order.iter().enumerate() {
        let a = &nodes[si];
        let b = &nodes[(si + 1) % n];
        let (mut c, mut from_t) = ([a.pos, a.cp_out, b.cp_in, b.pos], 0.0);
        let first_breaks = false; // a segment boundary is a normal node, not a cut
        if spline.closed && idx == 0 {
            let (_, _, mid, r1, r2, _) = split_bezier(c[0], c[1], c[2], c[3], sorted[0].1);
            c = [mid, r1, r2, c[3]];
            from_t = sorted[0].1;
        }
        let mut ts: Vec<f64> = sorted.iter().filter(|(cs, t)| *cs == si && *t > from_t + 1e-9).map(|(_, t)| *t).collect();
        ts.sort_by(|x, y| x.total_cmp(y));
        let mut running = c;
        let mut t0 = from_t;
        let mut breaks = first_breaks;
        for &t in &ts {
            let local = (t - t0) / (1.0 - t0);
            let (l1, l2, mid, r1, r2, p3) = split_bezier(running[0], running[1], running[2], running[3], local);
            pieces.push(Piece { c: [running[0], l1, l2, mid], breaks_before: breaks });
            running = [mid, r1, r2, p3];
            t0 = t;
            breaks = true; // every piece after a cut starts a fresh spline
        }
        pieces.push(Piece { c: running, breaks_before: breaks });
    }
    // turn the chain of sub-pieces into splines: consecutive pieces share a node (prev.p3 ==
    // next.p0); a piece that `breaks_before` begins a new spline at that shared point
    let mut splines: Vec<Vec<SplineNode>> = vec![vec![]];
    for (i, piece) in pieces.iter().enumerate() {
        let [p0, p1, p2, p3] = piece.c;
        if piece.breaks_before {
            splines.push(vec![]);
        }
        let cp_in = if i > 0 && !piece.breaks_before { pieces[i - 1].c[2] } else { p0 };
        splines.last_mut().unwrap().push(SplineNode { pos: p0, cp_in, cp_out: p1, smooth: false });
        let last = i + 1 == pieces.len();
        if last || pieces[i + 1].breaks_before {
            splines.last_mut().unwrap().push(SplineNode { pos: p3, cp_in: p2, cp_out: p3, smooth: false });
        }
    }
    splines.into_iter().filter(|nodes| nodes.len() >= 2).map(|nodes| Spline { nodes, closed: false }).collect()
}

/// The crossings with the cutters as `(segment index, t in 0..1)`, sorted: each cubic segment is
/// walked finely and a sign change in the side of a cutter line (or of a cutter circle) is bisected
/// to a precise parameter. An open spline's own two ends are not cuts.
fn crossing_params(spline: &Spline, cutters: &Cutters) -> Vec<(usize, f64)> {
    let nodes = &spline.nodes;
    let n = nodes.len();
    if n < 2 {
        return vec![];
    }
    let seg_count = if spline.closed { n } else { n - 1 };
    const STEPS: usize = 64;
    let mut out: Vec<(usize, f64)> = Vec::new();
    for si in 0..seg_count {
        let a = &nodes[si];
        let b = &nodes[(si + 1) % n];
        let c = [a.pos, a.cp_out, b.cp_in, b.pos];
        let point = |t: f64| cubic(c[0], c[1], c[2], c[3], t);
        for seg in cutters.outlines.iter().flat_map(|o| o.segments()) {
            refine_crossings(si, &point, |p| side_of_line(p, seg.0, seg.1), |t| on_segment(point(t), seg.0, seg.1), STEPS, &mut out);
        }
        for r in &cutters.rounds {
            refine_crossings(si, &point, |p| (p[0] - r.center[0]).hypot(p[1] - r.center[1]) - r.radius, |t| super::arc::arc_has_point(r, point(t)), STEPS, &mut out);
        }
    }
    out.sort_by(|x, y| (x.0, x.1).partial_cmp(&(y.0, y.1)).unwrap());
    out.dedup_by(|x, y| x.0 == y.0 && (x.1 - y.1).abs() < 1e-6);
    if !spline.closed {
        out.retain(|(si, t)| !(*si == 0 && *t < 1e-6) && !(*si == seg_count - 1 && *t > 1.0 - 1e-6));
    }
    out
}

fn lerp(a: Pt, b: Pt, t: f64) -> Pt {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

/// De-Casteljau split of the cubic `(p0, p1, p2, p3)` at `t`. Returns `(l1, l2, mid, r1, r2, p3)`:
/// the left half is the cubic `(p0, l1, l2, mid)`, the right half `(mid, r1, r2, p3)`. The two
/// halves together are the exact same curve, so a spline split this way never changes shape.
fn split_bezier(p0: Pt, p1: Pt, p2: Pt, p3: Pt, t: f64) -> (Pt, Pt, Pt, Pt, Pt, Pt) {
    let a = lerp(p0, p1, t);
    let b = lerp(p1, p2, t);
    let c = lerp(p2, p3, t);
    let d = lerp(a, b, t);
    let e = lerp(b, c, t);
    let mid = lerp(d, e, t);
    (a, d, mid, e, c, p3)
}

/// The signed side of the line through `a`–`b` a point is on (zero on the line).
fn side_of_line(p: Pt, a: Pt, b: Pt) -> f64 {
    (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])
}

/// Whether the foot of `p` on the line `a`–`b` lies within the segment (so the crossing is real,
/// not on the line's extension).
fn on_segment(p: Pt, a: Pt, b: Pt) -> bool {
    let d = [b[0] - a[0], b[1] - a[1]];
    let len2 = d[0] * d[0] + d[1] * d[1];
    if len2 < 1e-24 {
        return false;
    }
    let t = ((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / len2;
    (-1e-6..=1.0 + 1e-6).contains(&t)
}

/// Walk the cubic `point(t)` in `steps` steps; wherever `f` (a signed distance to a cutter) changes
/// sign and the crossing `within` the cutter holds, bisect to a precise `t` and record `(si, t)`.
fn refine_crossings(si: usize, point: &impl Fn(f64) -> Pt, f: impl Fn(Pt) -> f64, within: impl Fn(f64) -> bool, steps: usize, out: &mut Vec<(usize, f64)>) {
    let mut t0 = 0.0;
    let mut s0 = f(point(0.0));
    for i in 1..=steps {
        let t1 = i as f64 / steps as f64;
        let s1 = f(point(t1));
        if s0 == 0.0 {
            if within(t0) {
                out.push((si, t0));
            }
        } else if s0 * s1 < 0.0 {
            let (mut lo, mut hi, mut flo) = (t0, t1, s0);
            for _ in 0..40 {
                let mid = (lo + hi) / 2.0;
                let fm = f(point(mid));
                if flo * fm <= 0.0 {
                    hi = mid;
                } else {
                    lo = mid;
                    flo = fm;
                }
            }
            let t = (lo + hi) / 2.0;
            if within(t) {
                out.push((si, t));
            }
        }
        t0 = t1;
        s0 = s1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::dist;

    /// Sample a cubic Bezier chain the way the spline's `evaluate` would, to compare pieces.
    fn sample(nodes: &[SplineNode], steps: usize) -> Vec<Pt> {
        let mut out = vec![nodes[0].pos];
        for w in nodes.windows(2) {
            for i in 1..=steps {
                out.push(cubic(w[0].pos, w[0].cp_out, w[1].cp_in, w[1].pos, i as f64 / steps as f64));
            }
        }
        out
    }

    #[test]
    fn splitting_a_spline_keeps_the_exact_curve() {
        let s = Spline::from_points(&[[0.0, 0.0], [10.0, 12.0], [22.0, -4.0], [30.0, 6.0]], false);
        let pieces = split_at(&s, &[(0, 0.4), (1, 0.7), (1, 0.2)]);
        assert_eq!(pieces.len(), 4, "three cuts → four pieces");
        assert!(pieces.iter().all(|p| !p.closed));
        assert_eq!(pieces[0].nodes[0].pos, [0.0, 0.0]);
        assert_eq!(pieces.last().unwrap().nodes.last().unwrap().pos, [30.0, 6.0]);
        for w in pieces.windows(2) {
            assert!(dist(w[0].nodes.last().unwrap().pos, w[1].nodes[0].pos) < 1e-9, "pieces meet");
        }
        let whole = sample(&s.nodes, 400);
        let on_curve = |p: Pt| whole.iter().any(|q| dist(*q, p) < 0.05);
        for piece in &pieces {
            for p in sample(&piece.nodes, 40) {
                assert!(on_curve(p), "a split point drifted off the original curve: {p:?}");
            }
        }
    }

    #[test]
    fn splitting_a_closed_spline_opens_it_once_round() {
        let s = Spline::from_points(&[[0.0, 0.0], [20.0, 0.0], [20.0, 20.0], [0.0, 20.0]], true);
        let pieces = split_at(&s, &[(0, 0.5), (2, 0.5)]);
        assert_eq!(pieces.len(), 2, "a ring cut twice → two open pieces");
        assert!(pieces.iter().all(|p| !p.closed));
        let ring_nodes = { let mut v = s.nodes.clone(); v.push(s.nodes[0]); v };
        let whole = sample(&ring_nodes, 400);
        let on_curve = |p: Pt| whole.iter().any(|q| dist(*q, p) < 0.1);
        for piece in &pieces {
            for p in sample(&piece.nodes, 40) {
                assert!(on_curve(p), "off the original ring: {p:?}");
            }
        }
    }

    #[test]
    fn a_spline_crossed_by_a_line_is_cut_there() {
        // a hump that rises over y = 5 and comes back: the line y = 5 cuts it twice
        let s = Spline::from_points(&[[0.0, 0.0], [10.0, 12.0], [20.0, 0.0]], false);
        let line = crate::geometry::Contour::open(vec![[-5.0, 5.0], [25.0, 5.0]]);
        let pieces = split_spline_at_crossings(&s, &Cutters::outlines(vec![line]));
        assert_eq!(pieces.len(), 3, "below, over, below again");
        // every piece lies on the original curve
        let whole = sample(&s.nodes, 400);
        for piece in &pieces {
            for p in sample(&piece.nodes, 40) {
                assert!(whole.iter().any(|q| dist(*q, p) < 0.1), "off the curve: {p:?}");
            }
        }
    }
}
