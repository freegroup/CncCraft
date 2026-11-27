//! The pointer over the drawing: each drawing tool gets its own cursor, a fine crosshair with a
//! small badge of the tool (rectangle, circle, line, dimension …), drawn by the app. The system
//! pointer is hidden over the drawing while one of them is on. The select tool, handles, grabbing
//! values and everything outside the drawing keep the system's pointers.

use super::{App, DrawTool};
use crate::theme::{CONSTRAINT, INK};
use egui::{CursorIcon, Pos2, Rect, Stroke, Vec2};

/// What the badge next to the crosshair shows.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Badge {
    Dimension,
    Align,
    Rect,
    Circle,
    Ellipse,
    Polygon,
    Line,
    SplineBadge,
    Text,
    Node,
    Trim,
    Erase,
    Connect,
    Fillet,
}

impl App {
    /// The cursor of the tool in use, if it has one of its own.
    fn tool_badge(&self) -> Option<Badge> {
        if self.dim_mode.is_some() {
            return Some(Badge::Dimension);
        }
        if self.align_mode.is_some() {
            return Some(Badge::Align);
        }
        match self.tool {
            DrawTool::Select => None,
            DrawTool::NodeEdit => Some(Badge::Node),
            DrawTool::Polyline => Some(Badge::Line),
            DrawTool::Spline => Some(Badge::SplineBadge),
            DrawTool::Rect => Some(Badge::Rect),
            DrawTool::Circle => Some(Badge::Circle),
            DrawTool::Arc => Some(Badge::Circle),
            DrawTool::Ellipse => Some(Badge::Ellipse),
            DrawTool::Polygon => Some(Badge::Polygon),
            DrawTool::Text => Some(Badge::Text),
            DrawTool::Trim => Some(Badge::Trim),
            DrawTool::Erase => Some(Badge::Erase),
            DrawTool::Connect => Some(Badge::Connect),
            DrawTool::FilletOuter | DrawTool::FilletInner => Some(Badge::Fillet),
        }
    }

    /// Draw the tool's cursor at the pointer over the drawing `canvas`, and hide the system's.
    /// Where the app already asked for another pointer this frame (a handle, a value to grab) the
    /// system's pointer stays.
    pub(super) fn custom_cursor(&mut self, ctx: &egui::Context, painter: &egui::Painter, canvas: Rect, hovered: bool) {
        self.custom_cursor_badge = None;
        let Some(badge) = self.tool_badge() else { return };
        let Some(at) = ctx.input(|i| i.pointer.hover_pos()).filter(|p| hovered && canvas.contains(*p)) else { return };
        if ctx.output(|o| o.cursor_icon) != CursorIcon::Default {
            return;
        }
        ctx.set_cursor_icon(CursorIcon::None);
        self.custom_cursor_badge = Some(badge);
        let tone = if badge == Badge::Dimension { CONSTRAINT } else { INK };
        // every line gets a light edge, so it can be seen on dark toolpaths and on the board alike
        let halo = Stroke::new(3.2_f32, crate::theme::halo());
        let line = Stroke::new(1.3_f32, tone);
        let (gap, arm) = (3.0, 9.0);
        for dir in [Vec2::X, Vec2::Y, -Vec2::X, -Vec2::Y] {
            for stroke in [halo, line] {
                painter.line_segment([at + dir * gap, at + dir * arm], stroke);
            }
        }
        // the badge, down and to the right (the original puts the plus at the lower-left of a
        // 32 px cursor, hotspot near its centre; see Assets/cursors/draw*.svg)
        let c = at + Vec2::new(15.0, 15.0);
        let shapes = badge_shapes(badge, c);
        for s in &shapes {
            s.draw(painter, halo);
        }
        for s in &shapes {
            s.draw(painter, line);
        }
        if badge == Badge::Text {
            painter.text(c, egui::Align2::CENTER_CENTER, "T", egui::FontId::proportional(12.0), tone);
        }
    }
}

/// One stroke of a badge.
enum Part {
    Line(Pos2, Pos2),
    Poly(Vec<Pos2>, bool),
    Circle(Pos2, f32),
}

impl Part {
    fn draw(&self, painter: &egui::Painter, stroke: Stroke) {
        match self {
            Part::Line(a, b) => {
                painter.line_segment([*a, *b], stroke);
            }
            Part::Poly(points, closed) => {
                let mut p = points.clone();
                if *closed {
                    p.push(points[0]);
                }
                painter.add(egui::Shape::line(p, stroke));
            }
            Part::Circle(c, r) => {
                painter.circle_stroke(*c, *r, Stroke::new(stroke.width, stroke.color));
            }
        }
    }
}

/// The strokes of a badge centred on `c` (about 12 px big).
fn badge_shapes(badge: Badge, c: Pos2) -> Vec<Part> {
    let p = |x: f32, y: f32| c + Vec2::new(x, y);
    match badge {
        // two ticks with the dimension line between them
        Badge::Dimension => vec![Part::Line(p(-6.0, -5.0), p(-6.0, 5.0)), Part::Line(p(6.0, -5.0), p(6.0, 5.0)), Part::Line(p(-6.0, 0.0), p(6.0, 0.0)), Part::Line(p(-6.0, 0.0), p(-3.0, -2.0)), Part::Line(p(-6.0, 0.0), p(-3.0, 2.0)), Part::Line(p(6.0, 0.0), p(3.0, -2.0)), Part::Line(p(6.0, 0.0), p(3.0, 2.0))],
        // a level bar over an upright one
        Badge::Align => vec![Part::Line(p(-6.0, -3.0), p(6.0, -3.0)), Part::Line(p(-6.0, -5.5), p(-6.0, -0.5)), Part::Line(p(6.0, -5.5), p(6.0, -0.5)), Part::Line(p(-3.0, 2.0), p(-3.0, 8.0)), Part::Line(p(-5.5, 2.0), p(-0.5, 2.0)), Part::Line(p(-5.5, 8.0), p(-0.5, 8.0))],
        Badge::Rect => vec![Part::Poly(vec![p(-6.0, -4.5), p(6.0, -4.5), p(6.0, 4.5), p(-6.0, 4.5)], true)],
        Badge::Circle => vec![Part::Circle(c, 5.5)],
        Badge::Ellipse => vec![Part::Poly((0..24).map(|i| { let a = i as f32 / 24.0 * std::f32::consts::TAU; p(6.5 * a.cos(), 4.0 * a.sin()) }).collect(), true)],
        Badge::Polygon => vec![Part::Poly((0..6).map(|i| { let a = i as f32 / 6.0 * std::f32::consts::TAU; p(6.0 * a.cos(), 6.0 * a.sin()) }).collect(), true)],
        Badge::Line => vec![Part::Poly(vec![p(-6.0, 5.0), p(-2.0, -3.0), p(2.0, 3.0), p(6.0, -5.0)], false)],
        Badge::SplineBadge => vec![Part::Poly((0..=12).map(|i| {
            let t = i as f32 / 12.0;
            let u = 1.0 - t;
            let x = u * u * u * (-6.0) + 3.0 * u * u * t * (-3.0) + 3.0 * u * t * t * 3.0 + t * t * t * 6.0;
            let y = u * u * u * (-5.0) + 3.0 * u * u * t * 5.0 + 3.0 * u * t * t * (-5.0) + t * t * t * 5.0;
            p(x, y)
        }).collect(), false)],
        Badge::Text => vec![],
        // a small square: a node
        Badge::Node => vec![Part::Poly(vec![p(-3.5, -3.5), p(3.5, -3.5), p(3.5, 3.5), p(-3.5, 3.5)], true)],
        // a line broken where another crosses it
        Badge::Trim => vec![Part::Line(p(-6.0, 4.0), p(-1.5, 1.0)), Part::Line(p(1.5, -1.0), p(6.0, -4.0)), Part::Line(p(0.0, -6.0), p(0.0, 6.0))],
        // a cross
        Badge::Erase => vec![Part::Line(p(-4.5, -4.5), p(4.5, 4.5)), Part::Line(p(4.5, -4.5), p(-4.5, 4.5))],
        // two points and the line between them
        Badge::Connect => vec![Part::Circle(p(-5.0, 4.0), 1.8), Part::Circle(p(5.0, -4.0), 1.8), Part::Line(p(-3.5, 2.8), p(3.5, -2.8))],
        // a rounded corner
        Badge::Fillet => {
            let arc = (0..=6).map(|i| {
                let a = std::f32::consts::PI - i as f32 / 6.0 * std::f32::consts::FRAC_PI_2;
                p(1.0 + 6.0 * a.cos(), 6.0 * a.sin())
            });
            vec![Part::Poly(std::iter::once(p(-5.0, -6.0)).chain(arc).chain(std::iter::once(p(6.0, 6.0))).collect(), false)]
        }
    }
}
