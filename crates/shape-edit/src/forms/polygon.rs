use super::{curves_scaled, flip_point, is_uniform, place, rotate, scale_point, Dim, DimLine, Draw, Flip, Form, Shape};
use crate::geometry::{dist, Contour};
use crate::Pt;
use serde::{Deserialize, Serialize};

/// A regular polygon. With no rotation its first corner points straight up.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Polygon {
    pub center: Pt,
    /// Distance from the centre to each corner.
    pub radius: f64,
    pub sides: u32,
    /// Degrees, anticlockwise.
    pub rotation: f64,
}

impl Polygon {
    pub fn new(center: Pt, radius: f64, sides: u32) -> Self {
        Polygon { center, radius, sides, rotation: 0.0 }
    }
}

/// Mirroring a figure whose corners sit at 90° + rotation + k·step about a vertical line gives
/// rotation −r; about a horizontal line it gives 180° − r.
pub(super) fn mirrored_rotation(rotation: f64, flip: Flip) -> f64 {
    match flip {
        Flip::Horizontal => -rotation,
        Flip::Vertical => 180.0 - rotation,
    }
}

/// Corners of a newly drawn polygon (estimated: the original's default isn't in its data).
pub const DEFAULT_SIDES: u32 = 6;

impl Draw for Polygon {
    /// Centre to a corner.
    fn from_drag(start: Pt, end: Pt) -> Self {
        Polygon::new(start, dist(start, end), DEFAULT_SIDES)
    }
}

impl Form for Polygon {
    fn label(&self) -> &'static str {
        "Polygon"
    }

    fn contours(&self) -> Vec<Contour> {
        let n = self.sides.max(3) as usize;
        let local = (0..n).map(|i| {
            let a = std::f64::consts::FRAC_PI_2 + i as f64 / n as f64 * std::f64::consts::TAU;
            [self.radius * a.cos(), self.radius * a.sin()]
        });
        vec![Contour::closed(place(local, self.center, self.rotation))]
    }

    /// The radius, from the centre to the first corner.
    fn decoration(&self) -> Vec<DimLine> {
        let corner = rotate([self.center[0], self.center[1] + self.radius], self.center, self.rotation);
        vec![DimLine { dim: Dim::Radius, from: self.center, to: corner, outward: [0.0, 0.0] }]
    }

    fn translated(&self, dx: f64, dy: f64) -> Self {
        Polygon { center: [self.center[0] + dx, self.center[1] + dy], ..self.clone() }
    }

    fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Shape {
        if !is_uniform(sx, sy) {
            return curves_scaled(self.contours(), anchor, sx, sy);
        }
        Polygon { center: scale_point(self.center, anchor, sx, sy), radius: self.radius * sx, ..self.clone() }.into()
    }

    fn mirrored(&self, axis: Pt, flip: Flip) -> Shape {
        Polygon { center: flip_point(self.center, axis, flip), rotation: mirrored_rotation(self.rotation, flip), ..self.clone() }.into()
    }
}
