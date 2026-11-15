use super::{bounds_lines, flip_point, scale_point, DimLine, Flip, Form, Shape};
use crate::geometry::Contour;
use crate::vector::{Corner, CornerKind};
use crate::Pt;
use serde::{Deserialize, Serialize};

/// A polyline of world points, open or closed. Its nodes can be edited one by one, and any
/// corner can be rounded off or given an "ear" (a true circular arc, see [`crate::vector::fillet`]).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Path {
    pub points: Vec<Pt>,
    pub closed: bool,
    /// The changed corners, by node. Old files have none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub corners: Vec<Corner>,
}

impl Path {
    pub fn new(points: Vec<Pt>, closed: bool) -> Self {
        Path { points, closed, corners: vec![] }
    }

    /// The same path with other points, its corners kept.
    pub fn with_points(&self, points: Vec<Pt>) -> Self {
        Path { points, closed: self.closed, corners: self.corners.clone() }
    }

    /// The outline as lines and true arcs.
    pub fn pieces(&self) -> Vec<crate::vector::Piece> {
        crate::vector::fillet::pieces(&self.points, self.closed, &self.corners)
    }

    /// The corner at node `node`, if it is changed.
    pub fn corner(&self, node: usize) -> Option<&Corner> {
        self.corners.iter().find(|c| c.node == node)
    }

    /// Change the corner at `node` (or take its change away with `None`).
    pub fn set_corner(&mut self, node: usize, corner: Option<(CornerKind, f64, Option<String>)>) {
        self.corners.retain(|c| c.node != node);
        if let Some((kind, radius, expr)) = corner {
            self.corners.push(Corner { node, kind, radius, expr, label_offset: None });
            self.corners.sort_by_key(|c| c.node);
        }
    }

    /// A node was put in after `after`: the corners behind it move on by one.
    pub fn node_inserted(&mut self, after: usize) {
        for c in &mut self.corners {
            if c.node > after {
                c.node += 1;
            }
        }
    }

    /// Node `index` was taken out: its corner goes, those behind it move back by one.
    pub fn node_removed(&mut self, index: usize) {
        self.corners.retain(|c| c.node != index);
        for c in &mut self.corners {
            if c.node > index {
                c.node -= 1;
            }
        }
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
        vec![Contour { pts: crate::vector::fillet::outline(&self.points, self.closed, &self.corners), closed: self.closed }]
    }

    fn decoration(&self) -> Vec<DimLine> {
        bounds_lines(&self.contours())
    }

    fn translated(&self, dx: f64, dy: f64) -> Self {
        self.with_points(self.points.iter().map(|p| [p[0] + dx, p[1] + dy]).collect())
    }

    fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Shape {
        self.with_points(self.points.iter().map(|p| scale_point(*p, anchor, sx, sy)).collect()).into()
    }

    fn mirrored(&self, axis: Pt, flip: Flip) -> Shape {
        self.with_points(self.points.iter().map(|p| flip_point(*p, axis, flip)).collect()).into()
    }
}
