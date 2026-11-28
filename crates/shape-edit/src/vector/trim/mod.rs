//! Trim: a vector is cut where it crosses others. Each shape family has its own split, isolated in
//! its own file — [`polyline`] for open/closed contours (lines, polylines, rectangles, polygons…),
//! [`arc`] for circles and arcs (kept round), [`spline`] for Bezier splines (split exactly, no
//! reshaping). This file holds only what they share: the [`Cutters`] a shape is cut by, and the
//! low-level crossing maths (segment↔segment, segment↔circle, circle↔circle, a point on an arc).
//!
//! Each shape's [`crate::Form::trim`] calls its own `split_*` here; this module itself has no
//! per-shape knowledge, so a new shape adds a file next to these, not a branch inside one.

mod arc;
mod polyline;
mod spline;

pub use arc::{arc_has_point, split_arc_at_crossings};
pub use polyline::{crossings, split_at_crossings};
pub use spline::split_spline_at_crossings;

use crate::Arc;
use crate::Pt;

/// What a trimmed vector is cut by: the outlines of the other shapes, and — kept apart — the round
/// ones (circles and arcs) as true circle arcs, so every crossing with them is exact, not a point
/// on a faceted approximation. A line trimmed at a circle then ends exactly where the circle's
/// trimmed arcs end.
#[derive(Clone, Debug, Default)]
pub struct Cutters {
    pub outlines: Vec<crate::geometry::Contour>,
    pub rounds: Vec<Arc>,
}

impl Cutters {
    /// Only outlines (no round cutters).
    pub fn outlines(outlines: Vec<crate::geometry::Contour>) -> Cutters {
        Cutters { outlines, rounds: vec![] }
    }
}

/// Where segment `a0`–`a1` crosses segment `b0`–`b1`: the fraction along the first, if they cross.
pub(crate) fn segment_crossing(a0: Pt, a1: Pt, b0: Pt, b1: Pt) -> Option<f64> {
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

/// Where the circle round `c` with radius `r` meets the segment `a`–`b`: for each meeting point the
/// fraction along the segment and the angle on the circle (radians, from +x anticlockwise). None,
/// one (touching) or two — exact, not from a faceted circle.
pub(crate) fn circle_segment_hits(c: Pt, r: f64, a: Pt, b: Pt) -> Vec<(f64, f64)> {
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
pub(crate) fn circle_circle_angles(c1: Pt, r1: f64, c2: Pt, r2: f64) -> Vec<(f64, f64)> {
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
