use super::{ellipse_points, flip_point, is_uniform, place, scale_point, Dim, DimLine, Draw, Ellipse, Flip, Form, Shape};
use crate::geometry::Contour;
use crate::Pt;
use serde::{Deserialize, Serialize};

/// A circle.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    pub center: Pt,
    pub radius: f64,
}

impl Circle {
    pub fn new(center: Pt, radius: f64) -> Self {
        Circle { center, radius }
    }
}

impl Draw for Circle {
    /// Centre to a point on the edge.
    fn from_drag(start: Pt, end: Pt) -> Self {
        Circle::new(start, crate::geometry::dist(start, end))
    }
}

impl Form for Circle {
    fn label(&self) -> &'static str {
        "Kreis"
    }

    fn contours(&self) -> Vec<Contour> {
        vec![Contour::closed(place(ellipse_points(self.radius, self.radius), self.center, 0.0))]
    }

    /// The diameter, measured along the bottom of the circle.
    fn decoration(&self) -> Vec<DimLine> {
        let [x, y] = self.center;
        let r = self.radius;
        vec![DimLine { dim: Dim::Diameter, from: [x - r, y - r], to: [x + r, y - r], outward: [0.0, -1.0] }]
    }

    fn translated(&self, dx: f64, dy: f64) -> Self {
        Circle { center: [self.center[0] + dx, self.center[1] + dy], ..*self }
    }

    fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Shape {
        let center = scale_point(self.center, anchor, sx, sy);
        if is_uniform(sx, sy) {
            Circle { center, radius: self.radius * sx }.into()
        } else {
            Ellipse { center, rx: self.radius * sx, ry: self.radius * sy, rotation: 0.0 }.into()
        }
    }

    fn mirrored(&self, axis: Pt, flip: Flip) -> Shape {
        Circle { center: flip_point(self.center, axis, flip), ..*self }.into()
    }

    /// Trimming a circle keeps it round: it becomes arcs on the same circle, each cut exactly where
    /// it crosses. A circle is the arc once round from 0° (see [`crate::Arc`]).
    fn trim(&self, cutters: &crate::vector::trim::Cutters) -> Option<Vec<Shape>> {
        let ring = crate::Arc::new(self.center, self.radius, 0.0, std::f64::consts::TAU);
        Some(crate::vector::trim::split_arc_at_crossings(&ring, cutters).into_iter().map(Into::into).collect())
    }
}

impl Copy for Circle {}
