use super::{box_lines, curves_scaled, flip_point, is_axis_aligned, is_uniform, place, rotate, scale_point, DimLine, Draw, Flip, Form, Grip, GripKind, Shape};
use crate::geometry::Contour;
use crate::Pt;
use serde::{Deserialize, Serialize};

/// A rectangle, optionally with rounded corners, rotated about its centre.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub center: Pt,
    pub width: f64,
    pub height: f64,
    pub corner_radius: f64,
    /// Degrees, anticlockwise.
    pub rotation: f64,
}

impl Rect {
    pub fn new(center: Pt, width: f64, height: f64) -> Self {
        Rect { center, width, height, corner_radius: 0.0, rotation: 0.0 }
    }

    /// The axis-aligned rectangle spanning two corners.
    pub fn from_corners(a: Pt, b: Pt) -> Self {
        Rect::new([(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0], (b[0] - a[0]).abs(), (b[1] - a[1]).abs())
    }

    /// Half width, half height, and how far inside the corner the radius handle sits.
    fn grip_frame(&self) -> (f64, f64, f64) {
        let (hw, hh) = (self.width / 2.0, self.height / 2.0);
        (hw, hh, 0.12 * self.width.min(self.height))
    }

    fn local(&self) -> Vec<Pt> {
        let (hw, hh) = (self.width / 2.0, self.height / 2.0);
        let c = self.corner_radius.max(0.0).min(hw.min(hh));
        if c <= 1e-9 {
            return vec![[-hw, -hh], [hw, -hh], [hw, hh], [-hw, hh]];
        }
        // a quarter arc round each corner, anticlockwise from the bottom right
        [[hw - c, -hh + c, -90.0], [hw - c, hh - c, 0.0], [-hw + c, hh - c, 90.0], [-hw + c, -hh + c, 180.0]]
            .into_iter()
            .flat_map(|[x, y, a0]| {
                (0..=8).map(move |i| {
                    let a = (a0 + i as f64 * 90.0 / 8.0).to_radians();
                    [x + c * a.cos(), y + c * a.sin()]
                })
            })
            .collect()
    }
}

impl Draw for Rect {
    /// Corner to opposite corner.
    fn from_drag(start: Pt, end: Pt) -> Self {
        Rect::from_corners(start, end)
    }
}

impl Form for Rect {
    fn label(&self) -> &'static str {
        "Rechteck"
    }

    fn contours(&self) -> Vec<Contour> {
        vec![Contour::closed(place(self.local(), self.center, self.rotation))]
    }

    fn decoration(&self) -> Vec<DimLine> {
        box_lines(self.center, self.width, self.height, self.rotation)
    }

    /// A corner-radius handle just inside the top-right corner, as in the original.
    fn grips(&self) -> Vec<Grip> {
        let (hw, hh, k) = self.grip_frame();
        let c = self.corner_radius.max(0.0).min(hw.min(hh));
        let local = [self.center[0] + hw - c - k, self.center[1] + hh - c - k];
        vec![Grip { kind: GripKind::CornerRadius, at: rotate(local, self.center, self.rotation) }]
    }

    fn with_grip(&self, kind: GripKind, p: Pt) -> Option<Shape> {
        if kind != GripKind::CornerRadius {
            return None;
        }
        let (hw, hh, k) = self.grip_frame();
        let q = rotate(p, self.center, -self.rotation);
        let (dx, dy) = (q[0] - self.center[0], q[1] - self.center[1]);
        // the handle slides along the corner's diagonal; its distance in sets the radius
        let c = (((hw - dx) + (hh - dy)) / 2.0 - k).clamp(0.0, hw.min(hh));
        Some(Rect { corner_radius: c, ..self.clone() }.into())
    }

    fn translated(&self, dx: f64, dy: f64) -> Self {
        Rect { center: [self.center[0] + dx, self.center[1] + dy], ..self.clone() }
    }

    fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Shape {
        if !(is_uniform(sx, sy) || is_axis_aligned(self.rotation)) {
            return curves_scaled(self.contours(), anchor, sx, sy);
        }
        Rect {
            center: scale_point(self.center, anchor, sx, sy),
            width: self.width * sx,
            height: self.height * sy,
            corner_radius: self.corner_radius * sx.min(sy),
            rotation: self.rotation,
        }
        .into()
    }

    fn mirrored(&self, axis: Pt, flip: Flip) -> Shape {
        // symmetric about both of its own axes, so mirroring only reverses the rotation
        Rect { center: flip_point(self.center, axis, flip), rotation: -self.rotation, ..self.clone() }.into()
    }
}
