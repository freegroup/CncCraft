//! The "Punkt auf Linie" tool (Fusion's **Coincident** for a point on a line): pick a point (a
//! corner, a centre, a vector node) and a line (a side of a shape, a segment of a vector), in any
//! order; CncCraft keeps the point sitting on that line. It shows a small symbol — a dot on a short
//! bar — beside the line, joined to it by a thin lead, the same look as the level/upright symbol.
//! Click the symbol to choose it, Entf (or right-click) deletes it. The maths is in
//! `shape_edit::constraints` ([`OnLine`]); the solver moves the point's shape onto the line.

use super::{App, DrawTool};
use crate::theme::{CONSTRAINT, SELECT};
use egui::{Color32, Rect, Stroke, Vec2};
use shape_edit::constraints::{self, Anchor};
use shape_edit::{Constrains, ConstraintView, OnLine};

/// The mode: the first thing picked so far (a point or a line), what is under the pointer, and the
/// tool to fall back to when another is chosen.
pub(super) struct OnLineMode {
    first: Option<Anchor>,
    hover: Option<Anchor>,
    tool: DrawTool,
}

impl OnLineMode {
    pub(super) fn new(tool: DrawTool) -> Self {
        OnLineMode { first: None, hover: None, tool }
    }
}

const PICK_PX: f64 = 10.0;
/// How far the symbol sits from the line (screen px) and how big it is.
const LEAD: f32 = 20.0;
const SYMBOL: f32 = 18.0;

impl App {
    /// Into the point-on-line mode, or out of it.
    pub(super) fn toggle_on_line_mode(&mut self) {
        if self.on_line_mode.is_some() {
            self.on_line_mode = None;
            self.status = "Punkt auf Linie beendet".into();
            return;
        }
        self.dims = None;
        self.dim_mode = None;
        self.align_mode = None;
        self.distance_edit = None;
        self.edit.rotate_mode = false;
        self.poly.clear();
        self.drawing = None;
        self.mode = super::ViewMode::TwoD;
        self.tool = DrawTool::Select;
        self.tool_form = None;
        self.on_line_mode = Some(OnLineMode::new(self.tool));
        self.status = "Punkt auf Linie: einen Punkt und eine Linie wählen (beliebige Reihenfolge) · Esc beendet".into();
    }

    /// Esc: take back the first pick, then leave the mode. Returns whether it did anything.
    pub(super) fn cancel_on_line_mode(&mut self) -> bool {
        match &mut self.on_line_mode {
            Some(m) if m.first.is_some() => {
                m.first = None;
                self.status = "Punkt auf Linie: einen Punkt und eine Linie wählen".into();
                true
            }
            Some(_) => {
                self.on_line_mode = None;
                self.status = "Punkt auf Linie beendet".into();
                true
            }
            None => false,
        }
    }

    /// The mode's clicks on the drawing: the first click picks a point or a line, the second the
    /// other kind; together they make the constraint.
    pub(super) fn on_line_input(&mut self, resp: &egui::Response, mouse: Option<shape_edit::Pt>) {
        let Some(mode) = &self.on_line_mode else { return };
        // another tool was chosen: the mode is over
        if self.tool != mode.tool {
            self.on_line_mode = None;
            return;
        }
        let Some(m) = mouse else { return };
        let mut items = self.shapes.clone();
        items.push(constraints::plate_item(self.job.width, self.job.height));
        let hover = constraints::pick(&items, m, PICK_PX / self.zoom as f64);
        let first = {
            let mode = self.on_line_mode.as_mut().expect("checked above");
            mode.hover = hover;
            mode.first
        };
        if !resp.clicked() {
            return;
        }
        let Some(h) = hover else { return };
        let made = match (first, h) {
            // nothing picked yet: remember this point or line, wait for the other kind
            (None, _) => {
                self.on_line_mode.as_mut().expect("checked above").first = Some(h);
                self.status = match h {
                    Anchor::Point(..) => "Jetzt die Linie wählen".into(),
                    Anchor::Edge(..) => "Jetzt den Punkt wählen".into(),
                };
                return;
            }
            // the second pick must be the other kind; same kind twice is not a point-on-line
            (Some(f), _) if f == h => return,
            (Some(f), _) => OnLine::between(f, h),
        };
        self.on_line_mode.as_mut().expect("checked above").first = None;
        let Some(ol) = made else {
            self.status = "Punkt auf Linie braucht einen Punkt und eine Linie, nicht zweimal dasselbe".into();
            return;
        };
        match ol.check(&items) {
            Ok(()) => {
                // the same pair twice is one constraint
                if !self.on_lines.iter().any(|o| o.point == ol.point && o.line == ol.line) {
                    self.on_lines.push(ol.clone());
                }
                self.apply_constraints();
                self.status = "Punkt auf Linie festgelegt (Symbol anklicken und Entf löscht es)".into();
            }
            Err(why) => self.status = why,
        }
    }

    /// The symbols of the point-on-line constraints, each beside its line and joined to it by a thin
    /// lead; also the pick marks of the mode.
    pub(super) fn draw_on_lines(&mut self, painter: &egui::Painter) {
        self.on_line_labels.clear();
        let mode_on = self.on_line_mode.is_some();
        if self.on_lines.is_empty() && !mode_on {
            return;
        }
        let mut items = self.shapes.clone();
        items.push(constraints::plate_item(self.job.width, self.job.height));
        let mut labels = Vec::new();
        for (i, ol) in self.on_lines.iter().enumerate() {
            // each constraint decides how much of it shows; while the mode is active all in full
            let view = if mode_on { ConstraintView::Full } else { ol.view(self.show_dims, &self.edit.selection) };
            if view == ConstraintView::Hidden {
                continue;
            }
            let slim = view == ConstraintView::Slim;
            let Anchor::Point(ip, pk) = ol.point else { continue };
            let Anchor::Edge(il, ek) = ol.line else { continue };
            let Some(p) = items.iter().find(|s| s.id == ip).and_then(|s| constraints::point_of(s, pk)) else { continue };
            let Some((e0, e1)) = items.iter().find(|s| s.id == il).and_then(|s| constraints::edge_of(s, ek)) else { continue };
            let off = ol.error(&items).is_some_and(|e| e.abs() > 1e-6);
            let chosen = matches!(self.selected_constraint, Some(super::distance::LabelTarget::OnLine(j)) if j == i);
            let tone = if chosen { SELECT } else if off { crate::theme::error() } else { CONSTRAINT };
            let tone = if slim { tone.gamma_multiply(0.5) } else { tone };
            // the symbol hangs beside the point, out along the line's normal
            let sp = self.to_screen(p);
            let (sa, sb) = (self.to_screen(e0), self.to_screen(e1));
            let along = if (sb - sa).length() > 0.5 { (sb - sa).normalized() } else { Vec2::X };
            let normal = Vec2::new(-along.y, along.x);
            let at = sp + normal * LEAD;
            painter.line_segment([sp, at - normal * (SYMBOL / 2.0)], Stroke::new(1.0_f32, tone.gamma_multiply(0.8)));
            let rect = Rect::from_center_size(at, Vec2::splat(SYMBOL));
            painter.rect(rect, 4.0, if slim { Color32::TRANSPARENT } else { crate::theme::SHEET_LABEL }, Stroke::new(if chosen { 1.8_f32 } else { 1.2_f32 }, tone));
            // the glyph: a short bar with a dot sitting on it (a point on a line)
            let c = rect.center();
            let k = SYMBOL * 0.3;
            let stroke = Stroke::new(1.6_f32, tone);
            painter.line_segment([c + Vec2::new(-k, k * 0.6), c + Vec2::new(k, k * 0.6)], stroke);
            painter.circle_filled(c + Vec2::new(0.0, -k * 0.2), 2.4, tone);
            if !slim {
                labels.push((i, rect));
            }
        }
        self.on_line_labels = labels;
        // the thing already picked and the one under the pointer
        if let Some(mode) = &self.on_line_mode {
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
