//! Corners of a polyline turned into true circular arcs: rounded off ("inner fillet") or given an
//! "ear" (a dog-bone relief that reaches beyond the corner, so a round cutter can clear it).
//!
//! The arc is kept as an arc (centre, radius, start and end, direction); a polyline is only made
//! from it where points are needed (outlines for drawing and cutting). Exports write it as an arc.

use crate::geometry::dist;
use crate::Pt;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// How a corner is changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CornerKind {
    /// Rounded off: the arc touches both edges and cuts the corner.
    Round,
    /// An "ear": a circle round the corner point; the edges end where they meet it, and the
    /// outline runs round the outside of the circle from one edge to the other.
    Ear,
}

/// One changed corner of a path: which node, how, and how big. The corner point itself stays
/// where it is; the change only alters the outline drawn round it and can be taken away again.
/// The radius may be typed as a formula or with variables (`expr`); it is then worked out again
/// whenever they change.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Corner {
    pub node: usize,
    pub kind: CornerKind,
    pub radius: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expr: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_offset: Option<[f64; 2]>,
}

/// A circular arc from `from` to `to` about `center`; `sweep` is the angle covered (radians,
/// positive anticlockwise).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arc {
    pub from: Pt,
    pub to: Pt,
    pub center: Pt,
    pub radius: f64,
    pub sweep: f64,
}

impl Arc {
    /// The DXF "bulge" of the arc: tan(sweep/4), positive anticlockwise.
    pub fn bulge(&self) -> f64 {
        (self.sweep / 4.0).tan()
    }

    /// Points along the arc, start and end included, no further apart than `max_step` radians.
    pub fn points(&self, max_step: f64) -> Vec<Pt> {
        let n = ((self.sweep.abs() / max_step).ceil() as usize).clamp(1, 512);
        let a0 = (self.from[1] - self.center[1]).atan2(self.from[0] - self.center[0]);
        (0..=n)
            .map(|i| {
                if i == 0 {
                    return self.from;
                }
                if i == n {
                    return self.to;
                }
                let a = a0 + self.sweep * i as f64 / n as f64;
                [self.center[0] + self.radius * a.cos(), self.center[1] + self.radius * a.sin()]
            })
            .collect()
    }
}

fn unit(v: Pt) -> Option<Pt> {
    let l = (v[0] * v[0] + v[1] * v[1]).sqrt();
    (l > 1e-12).then(|| [v[0] / l, v[1] / l])
}

fn angle(c: Pt, p: Pt) -> f64 {
    (p[1] - c[1]).atan2(p[0] - c[0])
}

/// The anticlockwise angle from `a0` to `a1`, in [0, 2π).
fn ccw(a0: f64, a1: f64) -> f64 {
    (a1 - a0).rem_euclid(2.0 * PI)
}

/// The arc that replaces corner `v` between its neighbours `prev` and `next`, or `None` where
/// there is no corner (a straight run) or no room. The radius shrinks where the edges are too
/// short for it: an arc may take at most half of each edge, so neighbouring corners never overlap.
pub fn corner_arc(prev: Pt, v: Pt, next: Pt, kind: CornerKind, radius: f64) -> Option<Arc> {
    if radius <= 1e-9 {
        return None;
    }
    let a = unit([prev[0] - v[0], prev[1] - v[1]])?;
    let b = unit([next[0] - v[0], next[1] - v[1]])?;
    let cos_full = (a[0] * b[0] + a[1] * b[1]).clamp(-1.0, 1.0);
    let half = cos_full.acos() / 2.0;
    if half < 1e-6 || PI / 2.0 - half < 1e-6 {
        return None;
    }
    let d = unit([a[0] + b[0], a[1] + b[1]])?;
    let room = dist(prev, v).min(dist(v, next)) / 2.0;
    match kind {
        CornerKind::Round => {
            // the arc touches both edges at `reach` from the corner
            let r = radius.min(room * half.tan());
            let reach = r / half.tan();
            let from = [v[0] + a[0] * reach, v[1] + a[1] * reach];
            let to = [v[0] + b[0] * reach, v[1] + b[1] * reach];
            let k = r / half.sin();
            let center = [v[0] + d[0] * k, v[1] + d[1] * k];
            // the short way round, away from the corner point
            let s = ccw(angle(center, from), angle(center, to));
            let sweep = if s <= PI { s } else { s - 2.0 * PI };
            Some(Arc { from, to, center, radius: r, sweep })
        }
        CornerKind::Ear => {
            // a circle centred on the corner: the edges end where they meet it, and the outline
            // goes the long way round the outside of the corner
            let r = radius.min(room);
            let from = [v[0] + a[0] * r, v[1] + a[1] * r];
            let to = [v[0] + b[0] * r, v[1] + b[1] * r];
            let (a0, a1, inside) = (angle(v, from), angle(v, to), angle(v, [v[0] + d[0], v[1] + d[1]]));
            let s = ccw(a0, a1);
            let sweep = if ccw(a0, inside) < s { s - 2.0 * PI } else { s };
            Some(Arc { from, to, center: v, radius: r, sweep })
        }
    }
}

/// One piece of a path's outline: a straight run to `to`, or an arc.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Piece {
    Line { from: Pt, to: Pt },
    Arc(Arc),
}

/// A path's outline as lines and true arcs: the corners in `corners` are replaced by their arcs.
pub fn pieces(points: &[Pt], closed: bool, corners: &[Corner]) -> Vec<Piece> {
    let n = points.len();
    if n < 2 {
        return vec![];
    }
    // the arc at each node, where it has one
    let arcs: Vec<Option<Arc>> = (0..n)
        .map(|i| {
            let c = corners.iter().find(|c| c.node == i)?;
            if !closed && (i == 0 || i == n - 1) {
                return None;
            }
            corner_arc(points[(i + n - 1) % n], points[i], points[(i + 1) % n], c.kind, c.radius)
        })
        .collect();
    let start_of = |i: usize| arcs[i].map_or(points[i], |a| a.to);
    let end_at = |i: usize| arcs[i].map_or(points[i], |a| a.from);
    let mut out = vec![];
    let segs = if closed { n } else { n - 1 };
    for i in 0..segs {
        let j = (i + 1) % n;
        if let (true, Some(a)) = (closed || i > 0, arcs[i]) {
            out.push(Piece::Arc(a));
        }
        let (from, to) = (start_of(i), end_at(j));
        if dist(from, to) > 1e-9 {
            out.push(Piece::Line { from, to });
        }
    }
    out
}

/// A path's outline as points: the arcs made of short chords.
pub fn outline(points: &[Pt], closed: bool, corners: &[Corner]) -> Vec<Pt> {
    if corners.is_empty() {
        return points.to_vec();
    }
    let mut out: Vec<Pt> = vec![];
    for piece in pieces(points, closed, corners) {
        let pts = match piece {
            Piece::Line { from, to } => vec![from, to],
            Piece::Arc(a) => a.points(PI / 36.0),
        };
        for p in pts {
            if out.last().map_or(true, |l| dist(*l, p) > 1e-9) {
                out.push(p);
            }
        }
    }
    if closed && out.len() > 1 && dist(out[0], *out.last().unwrap()) < 1e-9 {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SQUARE: [Pt; 4] = [[0.0, 0.0], [20.0, 0.0], [20.0, 20.0], [0.0, 20.0]];

    fn corner(node: usize, kind: CornerKind, radius: f64) -> Corner {
        Corner { node, kind, radius, expr: None, label_offset: None }
    }

    #[test]
    fn a_rounded_corner_is_a_quarter_circle_touching_both_edges() {
        let a = corner_arc(SQUARE[0], SQUARE[1], SQUARE[2], CornerKind::Round, 3.0).unwrap();
        assert!((a.from[0] - 17.0).abs() < 1e-9 && a.from[1].abs() < 1e-9);
        assert!((a.to[0] - 20.0).abs() < 1e-9 && (a.to[1] - 3.0).abs() < 1e-9);
        assert!((a.center[0] - 17.0).abs() < 1e-9 && (a.center[1] - 3.0).abs() < 1e-9);
        assert!((a.sweep - PI / 2.0).abs() < 1e-9, "anticlockwise quarter: {}", a.sweep);
        for p in a.points(0.1) {
            assert!((dist(p, a.center) - 3.0).abs() < 1e-9);
        }
    }

    #[test]
    fn an_ear_is_a_circle_round_the_corner_point() {
        let a = corner_arc(SQUARE[0], SQUARE[1], SQUARE[2], CornerKind::Ear, 3.0).unwrap();
        assert_eq!(a.center, SQUARE[1], "centred on the corner");
        assert!((a.from[0] - 17.0).abs() < 1e-9 && a.from[1].abs() < 1e-9, "the edge ends on the circle");
        assert!((a.to[0] - 20.0).abs() < 1e-9 && (a.to[1] - 3.0).abs() < 1e-9);
        assert!((a.sweep.abs() - 1.5 * PI).abs() < 1e-9, "three quarters round the outside: {}", a.sweep);
        // round the outside: never into the square
        for p in a.points(0.05) {
            assert!(!(p[0] < 20.0 - 1e-6 && p[1] > 1e-6), "{p:?} lies inside the corner");
        }
    }

    #[test]
    fn a_radius_too_big_for_the_edges_shrinks() {
        let a = corner_arc(SQUARE[0], SQUARE[1], SQUARE[2], CornerKind::Round, 50.0).unwrap();
        assert!((a.radius - 10.0).abs() < 1e-9, "half the edge: {}", a.radius);
        assert!(corner_arc([0.0, 0.0], [10.0, 0.0], [20.0, 0.0], CornerKind::Round, 2.0).is_none(), "no corner on a straight run");
    }

    #[test]
    fn the_outline_replaces_only_the_changed_corners() {
        let pieces = pieces(&SQUARE, true, &[corner(1, CornerKind::Round, 2.0), corner(3, CornerKind::Ear, 2.0)]);
        assert_eq!(pieces.iter().filter(|p| matches!(p, Piece::Arc(_))).count(), 2);
        assert_eq!(pieces.iter().filter(|p| matches!(p, Piece::Line { .. })).count(), 4);
        let pts = outline(&SQUARE, true, &[corner(1, CornerKind::Round, 2.0)]);
        assert!(pts.contains(&[0.0, 0.0]) && pts.contains(&[20.0, 20.0]) && !pts.contains(&[20.0, 0.0]));
        // the ends of an open path have no corner
        assert_eq!(outline(&SQUARE, false, &[corner(0, CornerKind::Round, 2.0)]), SQUARE.to_vec());
    }
}
