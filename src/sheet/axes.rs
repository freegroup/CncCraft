//! The axis cross: red X and green Y arrows with their letters, well outside the board's lower
//! left corner so they touch neither the ruler nor the work, and a small ring on the zero point.

use super::rulers::RULER;
use super::{Decoration, Layer, SheetView};
use crate::theme::INK;
use egui::{Align2, Color32, FontId, Painter, Pos2, Stroke, Vec2};

/// How far (screen px) the cross sits left of and below the board's corner: clear of the ruler.
const OFFSET: f32 = RULER + 20.0;
/// Length of an arrow in screen pixels.
const LEN: f32 = 46.0;

pub struct AxisCross;

impl Decoration for AxisCross {
    fn name(&self) -> &'static str {
        "Koordinatenkreuz"
    }

    fn layer(&self) -> Layer {
        Layer::Over
    }

    fn paint(&self, view: &SheetView, painter: &Painter) {
        let at = view.to_screen([0.0, 0.0]) + Vec2::new(-OFFSET, OFFSET);
        let axes = [(Vec2::new(1.0, 0.0), "X", Color32::from_rgb(0xd0, 0x2a, 0x2a)), (Vec2::new(0.0, -1.0), "Y", Color32::from_rgb(0x1f, 0x9a, 0x3c))];
        for (dir, name, colour) in axes {
            let tip = at + dir * LEN;
            let side = Vec2::new(-dir.y, dir.x);
            painter.line_segment([at, tip - dir * 8.0], Stroke::new(2.6_f32, colour));
            painter.add(egui::Shape::convex_polygon(vec![tip, tip - dir * 11.0 + side * 5.5, tip - dir * 11.0 - side * 5.5], colour, Stroke::new(1.0_f32, Color32::WHITE)));
            // the letter just past the tip, in a small round badge of the axis colour
            let badge: Pos2 = tip + dir * 12.0;
            painter.circle(badge, 9.0, colour, Stroke::new(1.5_f32, Color32::WHITE));
            painter.text(badge, Align2::CENTER_CENTER, name, FontId::proportional(12.5), Color32::WHITE);
        }
        painter.circle(at, 4.0, Color32::WHITE, Stroke::new(2.0_f32, INK));
        // the job's zero point on the board
        painter.circle(view.to_screen(view.datum), 3.5, Color32::WHITE, Stroke::new(1.6_f32, INK));
    }
}
