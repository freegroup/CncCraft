//! Trimming a circle or an arc: it stays round. The piece between each pair of crossings is a new
//! [`Arc`] on the same circle — same centre and radius, its ends exactly where it was cut, meeting
//! the neighbours with no gap. A circle is handled as the arc once round from 0° (see
//! [`crate::Circle::trim`]); an arc keeps its own two ends.

use super::{circle_circle_angles, circle_segment_hits, Cutters};
use crate::{Arc, Pt};
use std::f64::consts::TAU;

/// Whether the angle `phi` lies on `arc` (its ends included): a full circle holds every angle.
pub(super) fn arc_contains(arc: &Arc, phi: f64) -> bool {
    let sweep = arc.end - arc.start;
    if sweep.abs() >= TAU - 1e-9 {
        return true;
    }
    let rel = ((phi - arc.start) * sweep.signum()).rem_euclid(TAU);
    rel <= sweep.abs() + 1e-9 || rel >= TAU - 1e-9
}

/// Whether the world point `p` (taken to be on the arc's circle) lies within the arc's sweep — the
/// angle test of [`arc_contains`], for callers that have a point rather than an angle (the spline
/// trim refines against a cutter arc this way).
pub fn arc_has_point(arc: &Arc, p: Pt) -> bool {
    arc_contains(arc, (p[1] - arc.center[1]).atan2(p[0] - arc.center[0]))
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
    use crate::geometry::{dist, Contour};

    fn line(a: Pt, b: Pt) -> Contour {
        Contour::open(vec![a, b])
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
        let small = parts.iter().map(|a| (a.end - a.start).abs()).fold(f64::MAX, f64::min);
        assert!((small - 2.0 * 8f64.atan2(6.0)).abs() < 1e-12);
        assert!(((parts[0].end - parts[1].start).rem_euclid(TAU)).abs() < 1e-12);
    }

    #[test]
    fn as_many_arcs_as_the_circle_is_cut_and_an_arc_keeps_its_ends() {
        let cross = Cutters::outlines(vec![line([-20.0, 0.0], [20.0, 0.0]), line([0.0, -20.0], [0.0, 20.0])]);
        let quarters = split_arc_at_crossings(&circle([0.0, 0.0], 5.0), &cross);
        assert_eq!(quarters.len(), 4);
        assert!(quarters.iter().all(|a| ((a.end - a.start).abs() - TAU / 4.0).abs() < 1e-12));
        let once = split_arc_at_crossings(&circle([0.0, 0.0], 5.0), &Cutters::outlines(vec![line([0.0, 0.0], [20.0, 0.0])]));
        assert_eq!(once.len(), 1);
        assert!(((once[0].end - once[0].start).abs() - TAU).abs() < 1e-12);
        let half = Arc::new([0.0, 0.0], 5.0, 0.0, std::f64::consts::PI);
        let pieces = split_arc_at_crossings(&half, &cross);
        assert_eq!(pieces.len(), 2, "the line along its ends (y = 0) is no cut");
        assert!((pieces[0].start - 0.0).abs() < 1e-12 && (pieces[1].end - std::f64::consts::PI).abs() < 1e-12);
        assert!((pieces[0].end - std::f64::consts::FRAC_PI_2).abs() < 1e-12 && (pieces[1].start - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
        let cw = Arc::new([0.0, 0.0], 5.0, 0.0, -std::f64::consts::PI);
        let pieces = split_arc_at_crossings(&cw, &cross);
        assert!(pieces.iter().all(|a| a.end < a.start), "clockwise pieces");
        assert!(split_arc_at_crossings(&circle([50.0, 50.0], 5.0), &cross).is_empty());
    }

    #[test]
    fn a_line_cut_at_a_circle_ends_exactly_where_the_circles_arcs_end() {
        let ring = circle([0.0, 0.0], 10.0);
        let l = line([-3.0, -20.0], [9.0, 20.0]);
        let line_parts = super::super::split_at_crossings(&l, &Cutters { outlines: vec![], rounds: vec![ring] });
        assert_eq!(line_parts.len(), 3, "in, through and out of the circle");
        let arcs = split_arc_at_crossings(&ring, &Cutters::outlines(vec![l.clone()]));
        assert_eq!(arcs.len(), 2);
        let arc_ends: Vec<Pt> = arcs.iter().flat_map(|a| [a.at(a.start), a.at(a.end)]).collect();
        for p in [line_parts[0].pts.last().unwrap(), line_parts[2].pts.first().unwrap()] {
            assert!((dist(*p, [0.0, 0.0]) - 10.0).abs() < 1e-12, "on the circle, not on a facet: {p:?}");
            assert!(arc_ends.iter().any(|q| dist(*p, *q) < 1e-9), "the line ends where an arc ends: {p:?}");
        }
    }

    #[test]
    fn two_circles_cut_each_other_exactly() {
        let a = circle([0.0, 0.0], 10.0);
        let b = circle([12.0, 0.0], 10.0);
        let pieces = split_arc_at_crossings(&a, &Cutters { outlines: vec![], rounds: vec![b] });
        assert_eq!(pieces.len(), 2);
        for p in pieces.iter().flat_map(|q| [q.at(q.start), q.at(q.end)]) {
            assert!((p[0] - 6.0).abs() < 1e-12 && (p[1].abs() - 8.0).abs() < 1e-12, "exact meeting point: {p:?}");
        }
        let upper = Arc::new([12.0, 0.0], 10.0, 0.0, std::f64::consts::PI);
        assert_eq!(split_arc_at_crossings(&a, &Cutters { outlines: vec![], rounds: vec![upper] }).len(), 1, "only the upper meeting point cuts");
    }
}
