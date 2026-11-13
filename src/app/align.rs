//! The "Waagrecht/Senkrecht" tool (H): one tool for both. Pick a line (a side of a shape, a
//! segment of a vector) or two points; CncCraft keeps it level or upright, whichever fits it better,
//! and shows a small symbol at the line connected to it by a thin line, the same look as a length
//! constraint. Click the symbol to choose it, Entf deletes it (right-click too). The sums are in
//! `shape_edit::constraints` ([`Align`]).

use super::{App, DrawTool};
use crate::theme::{CONSTRAINT, SELECT};
use egui::{Color32, Rect, Stroke, Vec2};
use shape_edit::constraints::{self, Anchor};
use shape_edit::{Align, Constrains, ConstraintView, Orient};

/// The mode: the first point picked so far, the tool to go back to when another is chosen.
pub(super) struct AlignMode {
    first: Option<Anchor>,
    hover: Option<Anchor>,
    tool: DrawTool,
}

impl AlignMode {
    pub(super) fn new(tool: DrawTool) -> Self {
        AlignMode { first: None, hover: None, tool }
    }
}

const PICK_PX: f64 = 10.0;
/// How far the symbol sits from the line (screen px) and how big it is.
const LEAD: f32 = 20.0;
const SYMBOL: f32 = 18.0;

impl App {
    /// H: into the level/upright mode, or out of it.
    pub(super) fn toggle_align_mode(&mut self) {
        if self.align_mode.is_some() {
            self.align_mode = None;
            self.status = "Waagrecht/Senkrecht beendet".into();
            return;
        }
        self.dims = None;
        self.dim_mode = None;
        self.distance_edit = None;
        self.edit.rotate_mode = false;
        self.poly.clear();
        self.drawing = None;
        self.mode = super::ViewMode::TwoD;
        self.tool = DrawTool::Select;
        self.tool_form = None;
        self.align_mode = Some(AlignMode::new(self.tool));
        self.status = "Waagrecht/Senkrecht: eine Linie wählen, oder zwei Punkte · H oder Esc beendet".into();
    }

    /// Esc: take back the first point, then leave the mode. Returns whether it did anything.
    pub(super) fn cancel_align_mode(&mut self) -> bool {
        match &mut self.align_mode {
            Some(m) if m.first.is_some() => {
                m.first = None;
                self.status = "Waagrecht/Senkrecht: eine Linie wählen, oder zwei Punkte".into();
                true
            }
            Some(_) => {
                self.align_mode = None;
                self.status = "Waagrecht/Senkrecht beendet".into();
                true
            }
            None => false,
        }
    }

    /// The mode's clicks on the drawing: a click on a line makes it level or upright at once, two
    /// clicks on points do the same for the line between them.
    pub(super) fn align_input(&mut self, resp: &egui::Response, mouse: Option<shape_edit::Pt>) {
        let Some(mode) = &self.align_mode else { return };
        // another tool was chosen: the mode is over
        if self.tool != mode.tool {
            self.align_mode = None;
            return;
        }
        let Some(m) = mouse else { return };
        let mut items = self.shapes.clone();
        items.push(constraints::plate_item(self.job.width, self.job.height));
        let hover = constraints::pick(&items, m, PICK_PX / self.zoom as f64);
        let first = {
            let mode = self.align_mode.as_mut().expect("checked above");
            mode.hover = hover;
            mode.first
        };
        if !resp.clicked() {
            return;
        }
        let Some(h) = hover else { return };
        let made = match (first, h) {
            // a line: between its two ends
            (None, Anchor::Edge(..)) => Align::along(&items, h),
            (None, Anchor::Point(..)) => {
                self.align_mode.as_mut().expect("checked above").first = Some(h);
                self.status = "Zweiten Punkt wählen".into();
                return;
            }
            (Some(f), Anchor::Point(..)) if f != h => {
                // the plate never moves: it is the reference
                let (a, b) = if h.shape() == constraints::PLATE && f.shape() != constraints::PLATE { (h, f) } else { (f, h) };
                Align::between(&items, a, b)
            }
            (Some(_), Anchor::Point(..)) => return,
            (Some(_), Anchor::Edge(..)) => {
                self.status = "Zwei Punkte wählen (oder ESC und eine Linie)".into();
                return;
            }
        };
        self.align_mode.as_mut().expect("checked above").first = None;
        let Some(al) = made else {
            self.status = "Diese Linie lässt sich nicht festlegen".into();
            return;
        };
        match al.check(&items) {
            Ok(()) => {
                // the same line twice is one constraint
                if !self.aligns.iter().any(|o| o.a == al.a && o.b == al.b || o.a == al.b && o.b == al.a) {
                    self.aligns.push(al.clone());
                }
                self.apply_constraints();
                self.status = format!("{} festgelegt (Symbol anklicken und Entf löscht es)", if al.orient == Orient::Horizontal { "Waagrecht" } else { "Senkrecht" });
            }
            Err(why) => self.status = why,
        }
    }

    /// The symbols of the level/upright constraints, each at its line and joined to it by a thin
    /// line; also the pick marks of the mode.
    pub(super) fn draw_aligns(&mut self, painter: &egui::Painter) {
        self.align_labels.clear();
        let mode_on = self.align_mode.is_some();
        if self.aligns.is_empty() && !mode_on {
            return;
        }
        let mut items = self.shapes.clone();
        items.push(constraints::plate_item(self.job.width, self.job.height));
        let mut labels = Vec::new();
        for (i, al) in self.aligns.iter().enumerate() {
            // each constraint decides how much of it shows; while the mode is active all in full
            let view = if mode_on { ConstraintView::Full } else { al.view(self.show_dims, &self.edit.selection) };
            if view == ConstraintView::Hidden {
                continue;
            }
            let slim = view == ConstraintView::Slim;
            let Some((pa, pb)) = al.ends(&items) else { continue };
            let off = al.error(&items).is_some_and(|e| e > 1e-6);
            let chosen = matches!(self.selected_constraint, Some(super::distance::LabelTarget::Align(j)) if j == i);
            let tone = if chosen { SELECT } else if off { Color32::from_rgb(0xc0, 0x30, 0x30) } else { CONSTRAINT };
            let tone = if slim { tone.gamma_multiply(0.5) } else { tone };
            let (sa, sb) = (self.to_screen(pa), self.to_screen(pb));
            let mid = sa + (sb - sa) / 2.0;
            // the symbol sits to the side of the line, away from the shape's middle when it can tell
            let along = if (sb - sa).length() > 0.5 { (sb - sa).normalized() } else { Vec2::X };
            let mut normal = Vec2::new(-along.y, along.x);
            let centre = items.iter().find(|s| s.id == al.a.shape()).and_then(|s| shape_edit::Editable::bounds(s)).map(|(lo, hi)| self.to_screen([(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0]));
            if let Some(c) = centre {
                if (mid - c).dot(normal) < 0.0 {
                    normal = -normal;
                }
            }
            let at = mid + normal * LEAD;
            painter.line_segment([mid + normal * 2.0, at - normal * (SYMBOL / 2.0)], Stroke::new(1.0_f32, tone.gamma_multiply(0.8)));
            let rect = Rect::from_center_size(at, Vec2::splat(SYMBOL));
            painter.rect(rect, 4.0, if slim { Color32::TRANSPARENT } else { crate::theme::SHEET_LABEL }, Stroke::new(if chosen { 1.8_f32 } else { 1.2_f32 }, tone));
            let (c, k) = (rect.center(), SYMBOL * 0.28);
            let stroke = Stroke::new(1.6_f32, tone);
            match al.orient {
                // level: a bar with a tick at each end
                Orient::Horizontal => {
                    painter.line_segment([c + Vec2::new(-k, 0.0), c + Vec2::new(k, 0.0)], stroke);
                    painter.line_segment([c + Vec2::new(-k, -2.5), c + Vec2::new(-k, 2.5)], stroke);
                    painter.line_segment([c + Vec2::new(k, -2.5), c + Vec2::new(k, 2.5)], stroke);
                }
                Orient::Vertical => {
                    painter.line_segment([c + Vec2::new(0.0, -k), c + Vec2::new(0.0, k)], stroke);
                    painter.line_segment([c + Vec2::new(-2.5, -k), c + Vec2::new(2.5, -k)], stroke);
                    painter.line_segment([c + Vec2::new(-2.5, k), c + Vec2::new(2.5, k)], stroke);
                }
            }
            if !slim {
                labels.push((i, rect));
            }
        }
        self.align_labels = labels;
        // the point already picked and the one under the pointer
        if let Some(mode) = &self.align_mode {
            let mark = |a: Anchor, strong: bool| match a {
                Anchor::Point(id, k) => {
                    if let Some(p) = items.iter().find(|s| s.id == id).and_then(|s| constraints::point_of(s, k)) {
                        let c = self.to_screen(p);
                        if strong {
                            painter.circle_filled(c, 5.0, CONSTRAINT);
                        } else {
                            painter.circle_stroke(c, 6.5, Stroke::new(2.0_f32, CONSTRAINT));
                        }
                    }
                }
                Anchor::Edge(id, k) => {
                    if let Some((a, b)) = items.iter().find(|s| s.id == id).and_then(|s| constraints::edge_of(s, k)) {
                        painter.line_segment([self.to_screen(a), self.to_screen(b)], Stroke::new(2.4_f32, CONSTRAINT));
                    }
                }
            };
            if let Some(f) = mode.first {
                mark(f, true);
            }
            if let Some(h) = mode.hover {
                mark(h, false);
            }
        }
    }
}
