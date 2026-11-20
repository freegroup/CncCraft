use super::{box_lines, flip_point, scale_point, DimLine, Flip, Form, Shape};
use crate::geometry::Contour;
use crate::Pt;
use serde::{Deserialize, Serialize};

/// A reference image placed on the canvas. It behaves like a rectangle that always keeps its
/// aspect ratio when resized. The actual pixel data lives in the layer's `ImageData`; this shape
/// only carries the geometry so it participates in selection, move and resize.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RefImage {
    pub center: Pt,
    pub width: f64,
    pub height: f64,
    /// The image layer this reference image belongs to (for texture lookup at render time).
    #[serde(default)]
    pub layer_id: u32,
}

impl RefImage {
    pub fn new(center: Pt, width: f64, height: f64, layer_id: u32) -> Self {
        RefImage { center, width, height, layer_id }
    }
}

impl Form for RefImage {
    fn label(&self) -> &'static str {
        "Bild"
    }

    fn contours(&self) -> Vec<Contour> {
        let (hw, hh) = (self.width / 2.0, self.height / 2.0);
        let c = self.center;
        vec![Contour::closed(vec![
            [c[0] - hw, c[1] - hh],
            [c[0] + hw, c[1] - hh],
            [c[0] + hw, c[1] + hh],
            [c[0] - hw, c[1] + hh],
        ])]
    }

    fn decoration(&self) -> Vec<DimLine> {
        box_lines(self.center, self.width, self.height, 0.0)
    }

    fn translated(&self, dx: f64, dy: f64) -> Self {
        RefImage { center: [self.center[0] + dx, self.center[1] + dy], ..self.clone() }
    }

    fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Shape {
        RefImage {
            center: scale_point(self.center, anchor, sx, sy),
            width: self.width * sx,
            height: self.height * sy,
            layer_id: self.layer_id,
        }
        .into()
    }

    fn mirrored(&self, axis: Pt, flip: Flip) -> Shape {
        RefImage { center: flip_point(self.center, axis, flip), ..self.clone() }.into()
    }
}
