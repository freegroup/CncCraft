use super::{bounds_lines, flip_point, scale_point, DimLine, Flip, Form, Shape};
use crate::geometry::Contour;
use crate::Pt;
use serde::{Deserialize, Serialize};

/// A polyline of world points, open or closed. Its nodes can be edited one by one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Path {
    pub points: Vec<Pt>,
    pub closed: bool,
}

impl Path {
    pub fn new(points: Vec<Pt>, closed: bool) -> Self {
        Path { points, closed }
    }

    /// Fewest nodes this path may keep.
    pub fn min_nodes(&self) -> usize {
        if self.closed {
            3
        } else {
            2
        }
    }
}

impl Form for Path {
    fn label(&self) -> &'static str {
        if self.closed {
            "Geschlossene Kurve"
        } else {
            "Offene Kurve"
        }
    }

    fn contours(&self) -> Vec<Contour> {
        vec![Contour { pts: self.points.clone(), closed: self.closed }]
    }

    fn decoration(&self) -> Vec<DimLine> {
        bounds_lines(&self.contours())
    }

    fn translated(&self, dx: f64, dy: f64) -> Self {
        Path { points: self.points.iter().map(|p| [p[0] + dx, p[1] + dy]).collect(), closed: self.closed }
    }

    fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Shape {
        Path { points: self.points.iter().map(|p| scale_point(*p, anchor, sx, sy)).collect(), closed: self.closed }.into()
    }

    fn mirrored(&self, axis: Pt, flip: Flip) -> Shape {
        Path { points: self.points.iter().map(|p| flip_point(*p, axis, flip)).collect(), closed: self.closed }.into()
    }
}
