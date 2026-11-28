use super::{bounds_lines, flip_point, scale_point, DimLine, Flip, Form, Shape};
use crate::geometry::Contour;
use crate::Pt;
use serde::{Deserialize, Serialize};

/// A cubic-Bezier spline: each node has a position and two control handles (in/out), plus a
/// flag saying whether it is smooth (handles are collinear through the point) or a corner
/// (handles are independent, both collapsed onto the point).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spline {
    pub nodes: Vec<SplineNode>,
    pub closed: bool,
}

/// One node of a [`Spline`].
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SplineNode {
    /// The point on the curve.
    pub pos: Pt,
    /// Incoming control handle (world coordinates).
    pub cp_in: Pt,
    /// Outgoing control handle (world coordinates).
    pub cp_out: Pt,
    /// Smooth: handles mirror each other through `pos`.  Corner: handles sit on `pos`.
    pub smooth: bool,
}

impl SplineNode {
    /// A smooth node: control handles are placed at ±`arm` along the tangent direction.
    pub fn smooth(pos: Pt, cp_in: Pt, cp_out: Pt) -> Self {
        SplineNode { pos, cp_in, cp_out, smooth: true }
    }

    /// A corner node: both handles collapse onto the position.
    pub fn corner(pos: Pt) -> Self {
        SplineNode { pos, cp_in: pos, cp_out: pos, smooth: false }
    }
}

impl Spline {
    pub fn new(nodes: Vec<SplineNode>, closed: bool) -> Self {
        Spline { nodes, closed }
    }

    /// Build a spline from just positions, auto-computing smooth handles.
    pub fn from_points(pts: &[Pt], closed: bool) -> Self {
        let n = pts.len();
        if n < 2 {
            return Spline { nodes: pts.iter().map(|p| SplineNode::corner(*p)).collect(), closed };
        }
        let nodes: Vec<SplineNode> = (0..n)
            .map(|i| {
                let prev = if i == 0 {
                    if closed { pts[n - 1] } else { pts[0] }
                } else {
                    pts[i - 1]
                };
                let next = if i == n - 1 {
                    if closed { pts[0] } else { pts[n - 1] }
                } else {
                    pts[i + 1]
                };
                let p = pts[i];
                let dx = next[0] - prev[0];
                let dy = next[1] - prev[1];
                let len = (dx * dx + dy * dy).sqrt().max(1e-12);
                let d_in = crate::geometry::dist(prev, p);
                let d_out = crate::geometry::dist(p, next);
                let arm_in = d_in * SMOOTHING;
                let arm_out = d_out * SMOOTHING;
                let ux = dx / len;
                let uy = dy / len;
                SplineNode::smooth(
                    p,
                    [p[0] - ux * arm_in, p[1] - uy * arm_in],
                    [p[0] + ux * arm_out, p[1] + uy * arm_out],
                )
            })
            .collect();
        Spline { nodes, closed }
    }

    pub fn min_nodes(&self) -> usize {
        if self.closed { 3 } else { 2 }
    }

    /// Evaluate one cubic Bezier segment from `a` to `b` into `steps` line segments.
    fn eval_segment(a: &SplineNode, b: &SplineNode, pts: &mut Vec<Pt>) {
        let steps = STEPS_PER_SEGMENT;
        for i in 1..=steps {
            let t = i as f64 / steps as f64;
            pts.push(cubic(a.pos, a.cp_out, b.cp_in, b.pos, t));
        }
    }

    /// The polyline approximation of the whole spline.
    pub fn evaluate(&self) -> Vec<Pt> {
        let n = self.nodes.len();
        if n == 0 {
            return vec![];
        }
        if n == 1 {
            return vec![self.nodes[0].pos];
        }
        let mut pts = vec![self.nodes[0].pos];
        let segs = if self.closed { n } else { n - 1 };
        for i in 0..segs {
            let a = &self.nodes[i];
            let b = &self.nodes[(i + 1) % n];
            Self::eval_segment(a, b, &mut pts);
        }
        pts
    }

    fn transform_nodes(&self, f: impl Fn(Pt) -> Pt) -> Vec<SplineNode> {
        self.nodes
            .iter()
            .map(|n| SplineNode {
                pos: f(n.pos),
                cp_in: f(n.cp_in),
                cp_out: f(n.cp_out),
                smooth: n.smooth,
            })
            .collect()
    }

    /// Cut the spline at the parameters in `cuts` — `(segment index, t in 0..1 along that
    /// segment)` — into open sub-splines, **without moving the curve**: every piece is an exact
    /// de-Casteljau split of the original cubics, so the pieces together trace the very same path.
    /// A closed spline opens at the first cut and runs once round; an open one keeps its own two
    /// ends. Fewer than one cut leaves it whole (returns `self`).
    pub fn split_at(&self, cuts: &[(usize, f64)]) -> Vec<Spline> {
        let n = self.nodes.len();
        if n < 2 || cuts.is_empty() {
            return vec![self.clone()];
        }
        let seg_count = if self.closed { n } else { n - 1 };
        let mut sorted = cuts.to_vec();
        sorted.sort_by(|a, b| (a.0, a.1).partial_cmp(&(b.0, b.1)).unwrap());
        // the segments in traversal order: a ring opens at the first cut's segment
        let start = if self.closed { sorted[0].0 } else { 0 };
        let order: Vec<usize> = (0..seg_count).map(|i| (start + i) % n).collect();
        // one `Cut` is a Bezier sub-piece (p0, p1, p2, p3) plus whether a knot opens a NEW spline at
        // its start (true = this piece begins a fresh spline). Build the whole chain of sub-pieces
        // first; a spline boundary is a cut point, so the second piece onward of a cut segment, and
        // the seam of an opened ring, start fresh.
        struct Piece {
            c: [Pt; 4],
            breaks_before: bool,
        }
        let mut pieces: Vec<Piece> = Vec::new();
        for (idx, &si) in order.iter().enumerate() {
            let a = &self.nodes[si];
            let b = &self.nodes[(si + 1) % n];
            // the whole segment's cubic; a ring's opening segment starts at the seam cut
            let (mut c, mut from_t) = ([a.pos, a.cp_out, b.cp_in, b.pos], 0.0);
            // a segment boundary is a normal node, not a cut: it never breaks the spline
            let mut first_breaks = false;
            if self.closed && idx == 0 {
                let (_, _, mid, r1, r2, _) = split_bezier(c[0], c[1], c[2], c[3], sorted[0].1);
                c = [mid, r1, r2, c[3]];
                from_t = sorted[0].1;
                first_breaks = false; // the opened ring's seam is the chain start, not a break
            }
            // this segment's cuts strictly after the seam, in order
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
                // the previous spline closed on this shared point; open a new one at it
                splines.push(vec![]);
            }
            // the start node of this piece: its cp_out is p1; cp_in comes from the previous piece's p2
            let cp_in = if i > 0 && !piece.breaks_before { pieces[i - 1].c[2] } else { p0 };
            splines.last_mut().unwrap().push(SplineNode { pos: p0, cp_in, cp_out: p1, smooth: false });
            // if this is the last piece, or the next one breaks, close on p3
            let last = i + 1 == pieces.len();
            if last || pieces[i + 1].breaks_before {
                splines.last_mut().unwrap().push(SplineNode { pos: p3, cp_in: p2, cp_out: p3, smooth: false });
            }
        }
        splines.into_iter().filter(|nodes| nodes.len() >= 2).map(|nodes| Spline { nodes, closed: false }).collect()
    }

    /// The crossings with the cutters as `(segment index, t in 0..1)`, sorted: each cubic segment
    /// is walked finely and a sign change in the side of a cutter line (or of a cutter circle) is
    /// bisected to a precise parameter. An open spline's own two ends are not cuts.
    fn crossing_params(&self, cutters: &crate::vector::trim::Cutters) -> Vec<(usize, f64)> {
        let n = self.nodes.len();
        if n < 2 {
            return vec![];
        }
        let seg_count = if self.closed { n } else { n - 1 };
        const STEPS: usize = 64;
        let mut out: Vec<(usize, f64)> = Vec::new();
        for si in 0..seg_count {
            let a = &self.nodes[si];
            let b = &self.nodes[(si + 1) % n];
            let c = [a.pos, a.cp_out, b.cp_in, b.pos];
            let point = |t: f64| cubic(c[0], c[1], c[2], c[3], t);
            // each cutter edge, and each round cutter as (centre, radius): a crossing is where the
            // signed side of that edge/circle flips along the segment
            for seg in cutters.outlines.iter().flat_map(|o| o.segments()) {
                refine_crossings(si, &point, |p| side_of_line(p, seg.0, seg.1), |t| on_segment(point(t), seg.0, seg.1), STEPS, &mut out);
            }
            for r in &cutters.rounds {
                refine_crossings(si, &point, |p| (p[0] - r.center[0]).hypot(p[1] - r.center[1]) - r.radius, |t| crate::vector::trim::arc_has_point(r, point(t)), STEPS, &mut out);
            }
        }
        out.sort_by(|x, y| (x.0, x.1).partial_cmp(&(y.0, y.1)).unwrap());
        out.dedup_by(|x, y| x.0 == y.0 && (x.1 - y.1).abs() < 1e-6);
        // an open spline's own ends are no cuts
        if !self.closed {
            out.retain(|(si, t)| !(*si == 0 && *t < 1e-6) && !(*si == seg_count - 1 && *t > 1.0 - 1e-6));
        }
        out
    }
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
            // exactly on it at a step: take it if within
            if within(t0) {
                out.push((si, t0));
            }
        } else if s0 * s1 < 0.0 {
            // a sign change: bisect for a precise crossing parameter
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

const SMOOTHING: f64 = 0.3;
const STEPS_PER_SEGMENT: usize = 20;

fn cubic(p0: Pt, p1: Pt, p2: Pt, p3: Pt, t: f64) -> Pt {
    let u = 1.0 - t;
    let (u2, t2) = (u * u, t * t);
    let (u3, t3) = (u2 * u, t2 * t);
    [
        u3 * p0[0] + 3.0 * u2 * t * p1[0] + 3.0 * u * t2 * p2[0] + t3 * p3[0],
        u3 * p0[1] + 3.0 * u2 * t * p1[1] + 3.0 * u * t2 * p2[1] + t3 * p3[1],
    ]
}

fn lerp(a: Pt, b: Pt, t: f64) -> Pt {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

/// De-Casteljau split of the cubic `(p0, p1, p2, p3)` at `t`. Returns `(l1, l2, mid, r1, r2, _)`:
/// the left half is the cubic `(p0, l1, l2, mid)`, the right half `(mid, r1, r2, p3)`. The two
/// halves together are the exact same curve, so a spline split this way never changes shape. The
/// sixth value repeats `p3` for callers that destructure six fields.
fn split_bezier(p0: Pt, p1: Pt, p2: Pt, p3: Pt, t: f64) -> (Pt, Pt, Pt, Pt, Pt, Pt) {
    let a = lerp(p0, p1, t);
    let b = lerp(p1, p2, t);
    let c = lerp(p2, p3, t);
    let d = lerp(a, b, t);
    let e = lerp(b, c, t);
    let mid = lerp(d, e, t);
    (a, d, mid, e, c, p3)
}

impl Form for Spline {
    fn label(&self) -> &'static str {
        if self.closed {
            "Geschlossener Spline"
        } else {
            "Offener Spline"
        }
    }

    fn contours(&self) -> Vec<Contour> {
        vec![Contour { pts: self.evaluate(), closed: self.closed }]
    }

    fn decoration(&self) -> Vec<DimLine> {
        bounds_lines(&self.contours())
    }

    fn translated(&self, dx: f64, dy: f64) -> Self {
        Spline {
            nodes: self.transform_nodes(|p| [p[0] + dx, p[1] + dy]),
            closed: self.closed,
        }
    }

    fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Shape {
        Spline {
            nodes: self.transform_nodes(|p| scale_point(p, anchor, sx, sy)),
            closed: self.closed,
        }
        .into()
    }

    fn mirrored(&self, axis: Pt, flip: Flip) -> Shape {
        Spline {
            nodes: self.transform_nodes(|p| flip_point(p, axis, flip)),
            closed: self.closed,
        }
        .into()
    }

    /// Trimming a spline keeps its exact curve: it is cut at every crossing with the cutters and
    /// falls apart into sub-splines, each an exact de-Casteljau piece of the original (see
    /// [`Spline::split_at`]). The crossing parameters are found per cubic segment, then the split
    /// is done on the real control points — never on the faceted preview, so the shape can't drift.
    fn trim(&self, cutters: &crate::vector::trim::Cutters) -> Option<Vec<Shape>> {
        let cuts = self.crossing_params(cutters);
        if cuts.is_empty() {
            return Some(vec![]);
        }
        Some(self.split_at(&cuts).into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_points_builds_smooth_handles() {
        let pts = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]];
        let s = Spline::from_points(&pts, false);
        assert_eq!(s.nodes.len(), 3);
        assert!(s.nodes.iter().all(|n| n.smooth));
        let poly = s.evaluate();
        assert!(poly.len() > 3);
        assert_eq!(poly[0], [0.0, 0.0]);
    }

    #[test]
    fn corner_nodes_give_straight_segments() {
        let s = Spline::new(
            vec![SplineNode::corner([0.0, 0.0]), SplineNode::corner([10.0, 0.0])],
            false,
        );
        let pts = s.evaluate();
        for p in &pts {
            assert!((p[1]).abs() < 1e-12, "y should be zero: {p:?}");
        }
    }

    #[test]
    fn closed_spline_wraps() {
        let pts = vec![[0.0, 0.0], [10.0, 0.0], [5.0, 10.0]];
        let s = Spline::from_points(&pts, true);
        let poly = s.evaluate();
        assert!(poly.len() > pts.len());
        let last = *poly.last().unwrap();
        let first = poly[0];
        assert!((last[0] - first[0]).abs() < 1e-6 && (last[1] - first[1]).abs() < 1e-6);
    }

    /// Sample a cubic Bezier chain the way `evaluate` would, to compare a split against the whole.
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
        // an open S-curve, split inside its first and second segments
        let s = Spline::from_points(&[[0.0, 0.0], [10.0, 12.0], [22.0, -4.0], [30.0, 6.0]], false);
        let pieces = s.split_at(&[(0, 0.4), (1, 0.7), (1, 0.2)]);
        assert_eq!(pieces.len(), 4, "three cuts → four pieces");
        assert!(pieces.iter().all(|p| !p.closed));
        assert_eq!(pieces[0].nodes[0].pos, [0.0, 0.0]);
        assert_eq!(pieces.last().unwrap().nodes.last().unwrap().pos, [30.0, 6.0]);
        for w in pieces.windows(2) {
            assert!(crate::geometry::dist(w[0].nodes.last().unwrap().pos, w[1].nodes[0].pos) < 1e-9, "pieces meet");
        }
        // every sampled point of every piece lies on the original curve (the shape didn't change)
        let whole = sample(&s.nodes, 400);
        let on_curve = |p: Pt| whole.iter().any(|q| crate::geometry::dist(*q, p) < 0.05);
        for piece in &pieces {
            for p in sample(&piece.nodes, 40) {
                assert!(on_curve(p), "a split point drifted off the original curve: {p:?}");
            }
        }
    }

    #[test]
    fn splitting_a_closed_spline_opens_it_once_round() {
        let s = Spline::from_points(&[[0.0, 0.0], [20.0, 0.0], [20.0, 20.0], [0.0, 20.0]], true);
        let pieces = s.split_at(&[(0, 0.5), (2, 0.5)]);
        assert_eq!(pieces.len(), 2, "a ring cut twice → two open pieces");
        assert!(pieces.iter().all(|p| !p.closed));
        let ring_nodes = { let mut v = s.nodes.clone(); v.push(s.nodes[0]); v };
        let whole = sample(&ring_nodes, 400);
        let on_curve = |p: Pt| whole.iter().any(|q| crate::geometry::dist(*q, p) < 0.1);
        for piece in &pieces {
            for p in sample(&piece.nodes, 40) {
                assert!(on_curve(p), "off the original ring: {p:?}");
            }
        }
    }

    #[test]
    fn translation_preserves_handles() {
        let s = Spline::from_points(&[[0.0, 0.0], [10.0, 0.0]], false);
        let t = s.translated(5.0, 3.0);
        assert_eq!(t.nodes[0].pos, [5.0, 3.0]);
        assert_eq!(t.nodes[1].pos, [15.0, 3.0]);
        assert!(t.nodes[0].smooth);
    }
}
