//! Trim: a vector is cut at every place where it crosses another one, and falls apart into
//! separate polylines — or, for a circle or an arc, into arcs of the same circle.

use crate::geometry::{dist, Contour};
use crate::{Arc, Pt};
use std::f64::consts::TAU;

/// Where segment `a0`–`a1` crosses segment `b0`–`b1`: the fraction along the first, if they cross.
fn crossing(a0: Pt, a1: Pt, b0: Pt, b1: Pt) -> Option<f64> {
    let (r, s) = ([a1[0] - a0[0], a1[1] - a0[1]], [b1[0] - b0[0], b1[1] - b0[1]]);
    let den = r[0] * s[1] - r[1] * s[0];
    if den.abs() < 1e-12 {
        return None;
    }
    let q = [b0[0] - a0[0], b0[1] - a0[1]];
    let t = (q[0] * s[1] - q[1] * s[0]) / den;
    let u = (q[0] * r[1] - q[1] * r[0]) / den;
    ((-1e-9..=1.0 + 1e-9).contains(&t) && (-1e-9..=1.0 + 1e-9).contains(&u)).then_some(t.clamp(0.0, 1.0))
}

/// The arc length at each point of a polyline (and at the closing point, for a closed one).
fn lengths(c: &Contour) -> Vec<f64> {
    let n = c.pts.len();
    let segs = if c.closed { n } else { n - 1 };
    let mut out = vec![0.0];
    for i in 0..segs {
        out.push(out[i] + dist(c.pts[i], c.pts[(i + 1) % n]));
    }
    out
}

/// What a trimmed vector is cut by: the outlines of the other shapes, and — kept apart — the round
/// ones (circles and arcs) as true circle arcs, so every crossing with them is exact, not a point
/// on a faceted approximation. A line trimmed at a circle then ends exactly where the circle's
/// trimmed arcs end.
#[derive(Clone, Debug, Default)]
pub struct Cutters {
    pub outlines: Vec<Contour>,
    pub rounds: Vec<Arc>,
}

impl Cutters {
    /// Only outlines (no round cutters).
    pub fn outlines(outlines: Vec<Contour>) -> Cutters {
        Cutters { outlines, rounds: vec![] }
    }
}

/// The positions (arc length along `c`) where it crosses any of the cutters, sorted, without
/// repeats, and without the very ends of an open polyline.
pub fn crossings(c: &Contour, cutters: &Cutters) -> Vec<f64> {
    let n = c.pts.len();
    if n < 2 {
        return vec![];
    }
    let len = lengths(c);
    let total = *len.last().unwrap();
    let mut out = vec![];
    for i in 0..len.len() - 1 {
        let (a0, a1) = (c.pts[i], c.pts[(i + 1) % n]);
        for o in &cutters.outlines {
            for (b0, b1) in o.segments() {
                if let Some(t) = crossing(a0, a1, b0, b1) {
                    out.push(len[i] + t * (len[i + 1] - len[i]));
                }
            }
        }
        // round cutters: exactly on the true circle, where it lies within the arc
        for r in &cutters.rounds {
            for (t, phi) in circle_segment_hits(r.center, r.radius, a0, a1) {
                if arc_contains(r, phi) {
                    out.push(len[i] + t * (len[i + 1] - len[i]));
                }
            }
        }
    }
    out.sort_by(|a, b| a.partial_cmp(b).unwrap());
    out.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    if c.closed {
        // the start and the end of a ring are the same place
        if out.len() > 1 && (out[0] + total - out[out.len() - 1]).abs() < 1e-6 {
            out.pop();
        }
    } else {
        out.retain(|s| *s > 1e-6 && *s < total - 1e-6);
    }
    out
}

/// The point at arc length `s`.
fn point_at(c: &Contour, len: &[f64], s: f64) -> Pt {
    let n = c.pts.len();
    let i = (0..len.len() - 1).find(|&k| s <= len[k + 1]).unwrap_or(len.len() - 2);
    let (a, b) = (c.pts[i], c.pts[(i + 1) % n]);
    let f = if len[i + 1] > len[i] { (s - len[i]) / (len[i + 1] - len[i]) } else { 0.0 };
    [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]
}

/// The stretch of `c` from arc length `from` to `to` (on a ring it may run over the start).
fn stretch(c: &Contour, len: &[f64], from: f64, to: f64) -> Vec<Pt> {
    let n = c.pts.len();
    let total = *len.last().unwrap();
    let wraps = c.closed && to <= from;
    let end = if wraps { to + total } else { to };
    let mut out = vec![point_at(c, len, from)];
    // every point strictly between the two ends, going once more round a ring where needed
    for lap in 0..if wraps { 2 } else { 1 } {
        for (k, l) in len.iter().enumerate() {
            let s = l + lap as f64 * total;
            if s > from + 1e-9 && s < end - 1e-9 {
                out.push(c.pts[k % n]);
            }
        }
    }
    out.push(point_at(c, len, to));
    out.dedup_by(|a, b| dist(*a, *b) < 1e-9);
    out
}

/// `c` cut at every crossing with the cutters, as open polylines in its own direction. Nothing
/// when it crosses nothing.
pub fn split_at_crossings(c: &Contour, cutters: &Cutters) -> Vec<Contour> {
    let cuts = crossings(c, cutters);
    if cuts.is_empty() {
        return vec![];
    }
    let len = lengths(c);
    let total = *len.last().unwrap();
    let mut bounds: Vec<(f64, f64)> = vec![];
    if c.closed {
        for k in 0..cuts.len() {
            bounds.push((cuts[k], cuts[(k + 1) % cuts.len()]));
        }
    } else {
        let mut at = 0.0;
        for &s in &cuts {
            bounds.push((at, s));
            at = s;
        }
        bounds.push((at, total));
    }
    bounds.into_iter().map(|(a, b)| Contour::open(stretch(c, &len, a, b))).filter(|p| p.pts.len() >= 2).collect()
}

/// Where the circle round `c` with radius `r` meets the segment `a`–`b`: for each meeting point the
/// fraction along the segment and the angle on the circle (radians, from +x anticlockwise). None,
/// one (touching) or two — exact, not from a faceted circle.
fn circle_segment_hits(c: Pt, r: f64, a: Pt, b: Pt) -> Vec<(f64, f64)> {
    let d = [b[0] - a[0], b[1] - a[1]];
    let f = [a[0] - c[0], a[1] - c[1]];
    let qa = d[0] * d[0] + d[1] * d[1];
    if qa < 1e-24 {
        return vec![];
    }
    let qb = 2.0 * (f[0] * d[0] + f[1] * d[1]);
    let qc = f[0] * f[0] + f[1] * f[1] - r * r;
    let disc = qb * qb - 4.0 * qa * qc;
    if disc < 0.0 {
        return vec![];
    }
    let root = disc.sqrt();
    let mut out = vec![];
    for t in [(-qb - root) / (2.0 * qa), (-qb + root) / (2.0 * qa)] {
        if (-1e-9..=1.0 + 1e-9).contains(&t) {
            let p = [a[0] + d[0] * t, a[1] + d[1] * t];
            out.push((t.clamp(0.0, 1.0), (p[1] - c[1]).atan2(p[0] - c[0])));
        }
    }
    out
}

/// Where two circles meet: for each meeting point the angle on the first circle and on the second.
fn circle_circle_angles(c1: Pt, r1: f64, c2: Pt, r2: f64) -> Vec<(f64, f64)> {
    let (dx, dy) = (c2[0] - c1[0], c2[1] - c1[1]);
    let d = dx.hypot(dy);
    // the same centre (no meeting, or the same circle), too far apart, or one inside the other
    if d < 1e-12 || d > r1 + r2 + 1e-12 || d < (r1 - r2).abs() - 1e-12 {
        return vec![];
    }
    let a = (r1 * r1 - r2 * r2 + d * d) / (2.0 * d);
    let h = (r1 * r1 - a * a).max(0.0).sqrt();
    let m = [c1[0] + a * dx / d, c1[1] + a * dy / d];
    let mut out = vec![];
    for sign in if h > 1e-12 { vec![1.0, -1.0] } else { vec![1.0] } {
        let p = [m[0] - sign * h * dy / d, m[1] + sign * h * dx / d];
        out.push(((p[1] - c1[1]).atan2(p[0] - c1[0]), (p[1] - c2[1]).atan2(p[0] - c2[0])));
    }
    out
}

/// Whether the angle `phi` lies on `arc` (its ends included): a full circle holds every angle.
fn arc_contains(arc: &Arc, phi: f64) -> bool {
    let sweep = arc.end - arc.start;
    if sweep.abs() >= TAU - 1e-9 {
        return true;
    }
    let rel = ((phi - arc.start) * sweep.signum()).rem_euclid(TAU);
    rel <= sweep.abs() + 1e-9 || rel >= TAU - 1e-9
}

/// The places (0..1 along the arc, from its start in its own direction) where `arc` crosses any of
/// the cutters, sorted and without repeats. A full circle has no ends, so every crossing counts;
/// an open arc's own ends are no cuts.
fn arc_crossings(arc: &Arc, cutters: &Cutters) -> Vec<f64> {
    let sweep = arc.end - arc.start;
    if sweep.abs() < 1e-12 {
        return vec![];
    }
    let full = sweep.abs() >= TAU - 1e-9;
    let on_lines = cutters.outlines.iter().flat_map(|o| o.segments().flat_map(|(a, b)| circle_segment_hits(arc.center, arc.radius, a, b)).map(|(_, phi)| phi).collect::<Vec<_>>());
    let on_rounds = cutters.rounds.iter().flat_map(|r| circle_circle_angles(arc.center, arc.radius, r.center, r.radius).into_iter().filter(|(_, phi2)| arc_contains(r, *phi2)).map(|(phi, _)| phi).collect::<Vec<_>>());
    let mut out: Vec<f64> = on_lines
        .chain(on_rounds)
        // how far round the arc, in its own direction, the crossing lies (0..1 of the sweep)
        .map(|phi| ((phi - arc.start) * sweep.signum()).rem_euclid(TAU) / sweep.abs())
        .filter(|t| if full { *t < 1.0 } else { *t > 1e-9 && *t < 1.0 - 1e-9 })
        .collect();
    out.sort_by(|a, b| a.total_cmp(b));
    // a crossing through a facet corner of another curve is found twice: once is enough
    out.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    if full && out.len() > 1 && (out[0] + 1.0 - out[out.len() - 1]).abs() < 1e-9 {
        out.pop();
    }
    out
}

/// A circle arc (or a whole circle, as the arc once round) cut at every crossing with the cutters,
/// as arcs on the same circle — each one starts and ends exactly at a crossing, so the pieces stay
/// round, keep their centre and radius, and meet without a gap. A circle crossed `n` times gives
/// `n` arcs (cut once, it opens into one arc right round from that point); an arc crossed `n`
/// times gives `n + 1`. Nothing when nothing crosses.
pub fn split_arc_at_crossings(arc: &Arc, cutters: &Cutters) -> Vec<Arc> {
    let cuts = arc_crossings(arc, cutters);
    if cuts.is_empty() {
        return vec![];
    }
    let sweep = arc.end - arc.start;
    let at = |t: f64| arc.start + sweep * t;
    let full = sweep.abs() >= TAU - 1e-9;
    let bounds: Vec<(f64, f64)> = if full {
        (0..cuts.len()).map(|k| (cuts[k], if k + 1 < cuts.len() { cuts[k + 1] } else { cuts[0] + 1.0 })).collect()
    } else {
        let mut edges = vec![0.0];
        edges.extend(&cuts);
        edges.push(1.0);
        edges.windows(2).map(|w| (w[0], w[1])).collect()
    };
    bounds.into_iter().map(|(a, b)| Arc { center: arc.center, radius: arc.radius, start: at(a), end: at(b) }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(a: Pt, b: Pt) -> Contour {
        Contour::open(vec![a, b])
    }

    #[test]
    fn an_open_line_falls_apart_at_each_crossing() {
        let c = Contour::open(vec![[0.0, 0.0], [10.0, 0.0], [20.0, 0.0]]);
        let others = [line([5.0, -5.0], [5.0, 5.0]), line([15.0, -5.0], [15.0, 5.0])];
        let parts = split_at_crossings(&c, &Cutters::outlines(others.to_vec()));
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].pts, vec![[0.0, 0.0], [5.0, 0.0]]);
        assert_eq!(parts[1].pts, vec![[5.0, 0.0], [10.0, 0.0], [15.0, 0.0]]);
        assert_eq!(parts[2].pts, vec![[15.0, 0.0], [20.0, 0.0]]);
        assert!(parts.iter().all(|p| !p.closed));
    }

    #[test]
    fn a_ring_cut_twice_becomes_two_open_pieces_over_its_start() {
        let ring = Contour::closed(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]);
        let parts = split_at_crossings(&ring, &Cutters::outlines(vec![line([5.0, -5.0], [5.0, 15.0])]));
        assert_eq!(parts.len(), 2);
        // one piece runs over the ring's first point
        assert!(parts.iter().any(|p| p.pts.contains(&[0.0, 0.0])));
        assert!(parts.iter().any(|p| p.pts.contains(&[10.0, 10.0])));
        let total: f64 = parts.iter().map(|p| p.segments().map(|(a, b)| dist(a, b)).sum::<f64>()).sum();
        assert!((total - 40.0).abs() < 1e-9, "nothing lost: {total}");
    }

    /// A circle as the arc once round.
    fn circle(c: Pt, r: f64) -> Arc {
        Arc::new(c, r, 0.0, TAU)
    }

    fn end_point(a: &Arc, angle: f64) -> Pt {
        [a.center[0] + a.radius * angle.cos(), a.center[1] + a.radius * angle.sin()]
    }

    #[test]
    fn a_circle_cut_by_a_line_becomes_two_arcs_meeting_exactly_on_the_line() {
        // the line x = 6 crosses the circle round the origin (r 10) at y = ±8
        let parts = split_arc_at_crossings(&circle([0.0, 0.0], 10.0), &Cutters::outlines(vec![line([6.0, -20.0], [6.0, 20.0])]));
        assert_eq!(parts.len(), 2, "two arcs, not polylines");
        for a in &parts {
            assert_eq!((a.center, a.radius), ([0.0, 0.0], 10.0), "same circle");
            for p in [end_point(a, a.start), end_point(a, a.end)] {
                assert!((p[0] - 6.0).abs() < 1e-12 && ((p[1].abs() - 8.0).abs() < 1e-12), "each end exactly on the line: {p:?}");
            }
        }
        let total: f64 = parts.iter().map(|a| (a.end - a.start).abs()).sum();
        assert!((total - TAU).abs() < 1e-12, "together exactly once round");
        // the small cap to the right spans 2·atan2(8, 6)
        let small = parts.iter().map(|a| (a.end - a.start).abs()).fold(f64::MAX, f64::min);
        assert!((small - 2.0 * 8f64.atan2(6.0)).abs() < 1e-12);
        // the pieces follow on: one's end is the next one's start
        assert!(((parts[0].end - parts[1].start).rem_euclid(TAU)).abs() < 1e-12);
    }

    #[test]
    fn as_many_arcs_as_the_circle_is_cut_and_an_arc_keeps_its_ends() {
        // a cross through the centre cuts the circle four times: four quarter arcs
        let cross = Cutters::outlines(vec![line([-20.0, 0.0], [20.0, 0.0]), line([0.0, -20.0], [0.0, 20.0])]);
        let quarters = split_arc_at_crossings(&circle([0.0, 0.0], 5.0), &cross);
        assert_eq!(quarters.len(), 4);
        assert!(quarters.iter().all(|a| ((a.end - a.start).abs() - TAU / 4.0).abs() < 1e-12));
        // cut once, a circle opens into one arc right round from that point
        let once = split_arc_at_crossings(&circle([0.0, 0.0], 5.0), &Cutters::outlines(vec![line([0.0, 0.0], [20.0, 0.0])]));
        assert_eq!(once.len(), 1);
        assert!(((once[0].end - once[0].start).abs() - TAU).abs() < 1e-12);
        // an upper half arc (0 → π, anticlockwise) cut by x = 0: two quarter arcs, its own ends kept
        let half = Arc::new([0.0, 0.0], 5.0, 0.0, std::f64::consts::PI);
        let pieces = split_arc_at_crossings(&half, &cross);
        assert_eq!(pieces.len(), 2, "the line along its ends (y = 0) is no cut");
        assert!((pieces[0].start - 0.0).abs() < 1e-12 && (pieces[1].end - std::f64::consts::PI).abs() < 1e-12);
        assert!((pieces[0].end - std::f64::consts::FRAC_PI_2).abs() < 1e-12 && (pieces[1].start - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
        // a clockwise arc keeps its sense
        let cw = Arc::new([0.0, 0.0], 5.0, 0.0, -std::f64::consts::PI);
        let pieces = split_arc_at_crossings(&cw, &cross);
        assert!(pieces.iter().all(|a| a.end < a.start), "clockwise pieces");
        // nothing crossing, nothing cut
        assert!(split_arc_at_crossings(&circle([50.0, 50.0], 5.0), &cross).is_empty());
    }

    #[test]
    fn a_line_cut_at_a_circle_ends_exactly_where_the_circles_arcs_end() {
        // the slot case: a line through a circle; both are trimmed against each other
        let ring = circle([0.0, 0.0], 10.0);
        let l = line([-3.0, -20.0], [9.0, 20.0]);
        let line_parts = split_at_crossings(&l, &Cutters { outlines: vec![], rounds: vec![ring] });
        assert_eq!(line_parts.len(), 3, "in, through and out of the circle");
        let arcs = split_arc_at_crossings(&ring, &Cutters::outlines(vec![l.clone()]));
        assert_eq!(arcs.len(), 2);
        // every inner line end lies exactly on the true circle, and matches an arc end
        let arc_ends: Vec<Pt> = arcs.iter().flat_map(|a| [a.at(a.start), a.at(a.end)]).collect();
        for p in [line_parts[0].pts.last().unwrap(), line_parts[2].pts.first().unwrap()] {
            assert!((dist(*p, [0.0, 0.0]) - 10.0).abs() < 1e-12, "on the circle, not on a facet: {p:?}");
            assert!(arc_ends.iter().any(|q| dist(*p, *q) < 1e-9), "the line ends where an arc ends: {p:?}");
        }
    }

    #[test]
    fn two_circles_cut_each_other_exactly() {
        // circles round (0,0) and (12,0), both r 10: they meet at x = 6, y = ±8
        let a = circle([0.0, 0.0], 10.0);
        let b = circle([12.0, 0.0], 10.0);
        let pieces = split_arc_at_crossings(&a, &Cutters { outlines: vec![], rounds: vec![b] });
        assert_eq!(pieces.len(), 2);
        for p in pieces.iter().flat_map(|q| [q.at(q.start), q.at(q.end)]) {
            assert!((p[0] - 6.0).abs() < 1e-12 && (p[1].abs() - 8.0).abs() < 1e-12, "exact meeting point: {p:?}");
        }
        // a round cutter that is only an arc cuts only where it really is
        let upper = Arc::new([12.0, 0.0], 10.0, 0.0, std::f64::consts::PI);
        assert_eq!(split_arc_at_crossings(&a, &Cutters { outlines: vec![], rounds: vec![upper] }).len(), 1, "only the upper meeting point cuts");
    }

    #[test]
    fn nothing_crossed_nothing_cut() {
        let c = Contour::open(vec![[0.0, 0.0], [10.0, 0.0]]);
        assert!(split_at_crossings(&c, &Cutters::outlines(vec![line([0.0, 5.0], [10.0, 5.0])])).is_empty());
        // touching at its own end is no cut
        assert!(split_at_crossings(&c, &Cutters::outlines(vec![line([10.0, -5.0], [10.0, 5.0])])).is_empty());
    }
}
