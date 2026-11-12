use super::{box_lines, curves_scaled, ellipse_points, flip_point, is_axis_aligned, is_uniform, place, scale_point, DimLine, Draw, Flip, Form, Shape};
use crate::geometry::Contour;
use crate::Pt;
use serde::{Deserialize, Serialize};

/// An ellipse with radii along its own axes, rotated about its centre.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ellipse {
    pub center: Pt,
    pub rx: f64,
    pub ry: f64,
    /// Degrees, anticlockwise.
    pub rotation: f64,
}

impl Ellipse {
    pub fn new(center: Pt, rx: f64, ry: f64) -> Self {
        Ellipse { center, rx, ry, rotation: 0.0 }
    }

    /// The axis-aligned ellipse filling the box between two corners.
    pub fn from_corners(a: Pt, b: Pt) -> Self {
        Ellipse::new([(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0], (b[0] - a[0]).abs() / 2.0, (b[1] - a[1]).abs() / 2.0)
    }
}

impl Draw for Ellipse {
    /// Filling the box between two corners.
    fn from_drag(start: Pt, end: Pt) -> Self {
        Ellipse::from_corners(start, end)
    }
}

impl Form for Ellipse {
    fn label(&self) -> &'static str {
        "Ellipse"
    }

    fn contours(&self) -> Vec<Contour> {
        vec![Contour::closed(place(ellipse_points(self.rx, self.ry), self.center, self.rotation))]
    }

    fn decoration(&self) -> Vec<DimLine> {
        box_lines(self.center, self.rx * 2.0, self.ry * 2.0, self.rotation)
    }

    fn translated(&self, dx: f64, dy: f64) -> Self {
        Ellipse { center: [self.center[0] + dx, self.center[1] + dy], ..self.clone() }
    }

    fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Shape {
        if !(is_uniform(sx, sy) || is_axis_aligned(self.rotation)) {
            return curves_scaled(self.contours(), anchor, sx, sy);
        }
        Ellipse { center: scale_point(self.center, anchor, sx, sy), rx: self.rx * sx, ry: self.ry * sy, rotation: self.rotation }.into()
    }

    fn mirrored(&self, axis: Pt, flip: Flip) -> Shape {
        Ellipse { center: flip_point(self.center, axis, flip), rotation: -self.rotation, ..self.clone() }.into()
    }
}
