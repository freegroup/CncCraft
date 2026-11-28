use super::{flip_point, place, scale_point, segments_for, Dim, DimLine, Draw, Ellipse, Flip, Form, Shape};
use crate::geometry::Contour;
use crate::Pt;
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

/// A circular arc — a piece of a circle kept as a circle (centre, radius) with a start and an end
/// angle, not as a polyline. Trimming a circle makes one of these, so the radius and centre stay
/// editable and a constraint such as a tangent keeps something round to act on. Angles are in
/// radians, measured anticlockwise from the +x axis; the arc is always drawn the direct way from
/// `start` to `end` — anticlockwise when `end > start`, clockwise when `end < start` — so either
/// order is valid and a negative sweep draws correctly (the span is `|end - start|`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Arc {
    pub center: Pt,
    pub radius: f64,
    pub start: f64,
    pub end: f64,
}

impl Arc {
    pub fn new(center: Pt, radius: f64, start: f64, end: f64) -> Self {
        Arc { center, radius, start, end }
    }

    /// The point on the arc at angle `a` (world).
    pub fn at(&self, a: f64) -> Pt {
        [self.center[0] + self.radius * a.cos(), self.center[1] + self.radius * a.sin()]
    }

    /// The signed sweep from `start` to `end`: positive anticlockwise, negative clockwise. The arc
    /// is always drawn the direct way from `start` to `end`, so a negative `end` (an end angle below
    /// the start) simply sweeps the other way — no "start must be ≤ end" assumption.
    fn sweep(&self) -> f64 {
        self.end - self.start
    }

    /// The angular span (always positive).
    pub fn span(&self) -> f64 {
        self.sweep().abs()
    }

    /// Whether the arc goes all the way round (a full circle).
    fn full(&self) -> bool {
        self.span() >= TAU - 1e-9
    }

    /// The sampled points from start to end (world), fine enough for the radius. Sweeps the signed
    /// way (clockwise when `end < start`), so the drawn curve always matches `start`/`end`.
    fn sample(&self) -> Vec<Pt> {
        let sweep = self.sweep();
        let n = segments_for(self.radius).max(2);
        // at least enough segments that the arc is not visibly faceted, scaled by how much of the
        // circle it covers
        let steps = ((n as f64) * (sweep.abs() / TAU)).ceil().max(2.0) as usize;
        (0..=steps).map(|i| self.at(self.start + sweep * (i as f64 / steps as f64))).collect()
    }
}

impl Draw for Arc {
    /// Dragging makes a half-circle from the centre to the dragged point (a sensible default; the
    /// usual way an arc appears is by trimming a circle, not by dragging).
    fn from_drag(start: Pt, end: Pt) -> Self {
        let r = crate::geometry::dist(start, end);
        let a = (end[1] - start[1]).atan2(end[0] - start[0]);
        Arc::new(start, r, a, a + std::f64::consts::PI)
    }
}

impl Form for Arc {
    fn label(&self) -> &'static str {
        "Kreissegment"
    }

    fn contours(&self) -> Vec<Contour> {
        let pts = self.sample();
        if self.full() {
            vec![Contour::closed(pts)]
        } else {
            vec![Contour::open(pts)]
        }
    }

    /// The radius: one plain line from the centre to the middle of the arc, with the value on it
    /// (`outward` zero, so no offset or witness lines — like the polygon's radius).
    fn decoration(&self) -> Vec<DimLine> {
        let mid = self.at((self.start + self.end) / 2.0);
        vec![DimLine { dim: Dim::Radius, from: self.center, to: mid, outward: [0.0, 0.0] }]
    }

    fn translated(&self, dx: f64, dy: f64) -> Self {
        Arc { center: [self.center[0] + dx, self.center[1] + dy], ..*self }
    }

    fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Shape {
        let center = scale_point(self.center, anchor, sx, sy);
        if super::is_uniform(sx, sy) {
            Arc { center, radius: self.radius * sx, ..*self }.into()
        } else {
            // a non-uniform scale is no longer a circle arc: fall back to an ellipse of the bounds
            let pts = place(self.sample().iter().map(|p| [(p[0] - self.center[0]), (p[1] - self.center[1])]).collect::<Vec<_>>(), center, 0.0);
            let _ = pts;
            Ellipse { center, rx: self.radius * sx, ry: self.radius * sy, rotation: 0.0 }.into()
        }
    }

    fn mirrored(&self, axis: Pt, flip: Flip) -> Shape {
        // mirror the centre and reflect the two angles about the mirror axis' direction
        let center = flip_point(self.center, axis, flip);
        let reflect = |a: f64| match flip {
            Flip::Horizontal => std::f64::consts::PI - a,
            Flip::Vertical => -a,
        };
        let (s, e) = (reflect(self.start), reflect(self.end));
        // keep start <= end with a positive span
        let (start, end) = if s <= e { (s, e) } else { (e, s) };
        Arc { center, radius: self.radius, start, end }.into()
    }

    /// Trimming an arc keeps it round: it falls apart into arcs on the same circle, each one
    /// starting and ending exactly where it was cut (see [`crate::vector::trim::split_arc_at_crossings`]).
    fn trim(&self, cutters: &crate::vector::trim::Cutters) -> Option<Vec<Shape>> {
        Some(crate::vector::trim::split_arc_at_crossings(self, cutters).into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI};

    #[test]
    fn a_half_arc_is_open_and_spans_its_endpoints() {
        // right half of a circle at origin: start 0 → end PI, radius 5
        let a = Arc::new([0.0, 0.0], 5.0, 0.0, PI);
        let c = &a.contours()[0];
        assert!(!c.closed, "a segment is an open contour");
        let first = c.pts.first().unwrap();
        let last = c.pts.last().unwrap();
        assert!((first[0] - 5.0).abs() < 1e-6 && first[1].abs() < 1e-6, "starts at angle 0 (5,0)");
        assert!((last[0] + 5.0).abs() < 1e-6 && last[1].abs() < 1e-6, "ends at angle 180° (-5,0)");
    }

    #[test]
    fn a_full_span_closes_like_a_circle() {
        let a = Arc::new([1.0, 2.0], 3.0, 0.0, 2.0 * PI);
        assert!(a.contours()[0].closed, "a 360° arc is a closed circle");
    }

    #[test]
    fn moving_and_resizing_keep_it_an_arc() {
        let a = Arc::new([0.0, 0.0], 4.0, 0.0, FRAC_PI_2);
        assert_eq!(a.translated(10.0, 0.0).center, [10.0, 0.0]);
        // a uniform scale keeps the arc (radius doubles); a non-uniform one is no longer a circle arc
        assert!(matches!(a.scaled([0.0, 0.0], 2.0, 2.0), Shape::Arc(b) if (b.radius - 8.0).abs() < 1e-9));
        assert!(matches!(a.scaled([0.0, 0.0], 2.0, 1.0), Shape::Ellipse(_)));
    }

    #[test]
    fn a_negative_end_sweeps_clockwise_correctly() {
        // start 0°, end -90°: the arc should run from (r,0) down to (0,-r), not up to (0,+r)
        let a = Arc::new([0.0, 0.0], 5.0, 0.0, -FRAC_PI_2);
        assert!((a.span() - FRAC_PI_2).abs() < 1e-9, "span is the absolute sweep, 90°");
        let c = &a.contours()[0];
        let first = c.pts.first().unwrap();
        let last = c.pts.last().unwrap();
        assert!((first[0] - 5.0).abs() < 1e-6 && first[1].abs() < 1e-6, "starts at (5,0)");
        assert!(last[0].abs() < 1e-6 && (last[1] + 5.0).abs() < 1e-6, "ends at (0,-5), swept clockwise");
        // the radius decoration points to the true mid angle (-45°), down-right, matching the curve
        let dec = a.decoration()[0];
        assert!(dec.to[0] > 0.0 && dec.to[1] < 0.0, "radius line to the mid of the drawn arc: {:?}", dec.to);
    }
}
