//! Construction lines in the app: drawing them over the workpiece, the ruler interaction that
//! creates, moves, selects and deletes them, dragging their attached shapes along, and snapping a
//! shape onto a guide while the user drags the shape.
//!
//! Guides are only grabbed on the ruler, never in the drawing area. A shape attaches to a guide
//! only while the user drags that shape onto it (within the snap radius); the guide then owns the
//! attachment. Dragging the shape away releases it. Moving a guide never attaches a resting shape.

use super::input::ConstraintInput;
use super::App;
use crate::guides::{Guide, GuideAxis, Side};
use crate::sheet::rulers::RULER;
use egui::{Color32, Pos2, Rect, Stroke};
use shape_edit::Editable;

/// Width (screen pixels) of a guide's handle tab on the ruler.
const HANDLE_W: f32 = 9.0;
/// How far (screen pixels) outside its tab a handle still reacts to the pointer.
const GRAB_PX: f32 = 1.0;
/// How thick (screen pixels) a guide's number label is across its ruler.
const LABEL_THICK: f32 = 20.0;
/// Space (screen pixels) between the ruler and a guide's number label.
const LABEL_GAP: f32 = 8.0;
/// How close (mm on screen, in pixels) a shape edge must come to a guide to snap while dragging.
const SNAP_PX: f32 = 8.0;
/// A guide's number while it is being typed.
pub(super) struct GuideEdit {
    pub(super) id: u64,
    pub(super) text: String,
    /// Where the number's label was: the field opens there.
    pub(super) rect: Rect,
    /// Suggestions and the cursor, shared with every number field (see `input.rs`).
    pub(super) input: ConstraintInput,
}

/// Red of the construction lines, like Sketch's ruler guides.
pub(super) fn guide_colour() -> Color32 {
    Color32::from_rgb(0xe0, 0x30, 0x30)
}

impl App {
    /// The X (vertical guide) or Y (horizontal) a shape part sits at, for a given guide.
    fn part_value(&self, shape: u64, axis: GuideAxis, side: Side) -> Option<f64> {
        let item = self.shapes.iter().find(|s| s.id == shape)?;
        let (lo, hi) = item.bounds()?;
        let k = if axis == GuideAxis::Vertical { 0 } else { 1 };
        Some(match side {
            Side::Low => lo[k],
            Side::High => hi[k],
            Side::Centre => (lo[k] + hi[k]) / 2.0,
        })
    }

    /// Whether the `Ruler` decoration is shown (guides only exist with it).
    pub(in crate::app) fn rulers_shown(&self) -> bool {
        self.sheet.is_shown("Lineale")
    }

    /// Show or hide the rulers (and, with them, the construction lines).
    pub(in crate::app) fn set_rulers_shown(&mut self, on: bool) {
        for (name, shown) in self.sheet.switches() {
            if name == "Lineale" {
                *shown = on;
            }
        }
        if !on {
            self.guide_selected = None;
            self.guide_drag = None;
        }
        self.status = if on { "Lineale werden gezeigt".into() } else { "Lineale ausgeblendet".into() };
    }

    /// The colour a guide is drawn in: its line, its handle on the ruler and its number all share
    /// it (selected guides in full red).
    fn guide_tone(&self, g: &Guide) -> Color32 {
        if self.guides_locked {
            // locked lines step back: half transparent
            guide_colour().gamma_multiply(0.4)
        } else if self.guide_selected == Some(g.id) {
            guide_colour()
        } else {
            guide_colour().gamma_multiply(0.85)
        }
    }

    /// The square where the two rulers meet: it holds the lock of the construction lines.
    fn ruler_corner(&self) -> Rect {
        let board = Rect::from_two_pos(self.to_screen([0.0, 0.0]), self.to_screen([self.job.width, self.job.height]));
        Rect::from_min_max(Pos2::new(board.left() - RULER, board.top() - RULER), Pos2::new(board.left(), board.top()))
    }

    /// Lock or open the construction lines. Locked lines can't be added, moved, edited or deleted
    /// (they still catch shapes); a toast says what changed.
    fn set_guides_locked(&mut self, ctx: &egui::Context, locked: bool) {
        self.guides_locked = locked;
        self.guide_selected = None;
        self.guide_edit = None;
        self.guide_drag = None;
        self.guide_press = None;
        let text = if locked {
            t!("Hilfslinien gesperrt: Sie können nicht mehr verschoben, bearbeitet oder gelöscht werden. Formen rasten weiter an ihnen ein.")
        } else {
            t!("Hilfslinien entsperrt: Sie können wieder verschoben, bearbeitet und gelöscht werden.")
        };
        self.status = text.clone();
        crate::widgets::toast::post(ctx, text);
    }

    /// Where a guide's number label sits: outside the ruler, a little apart from it.
    fn guide_label_pos(&self, g: &Guide) -> Pos2 {
        let board = Rect::from_two_pos(self.to_screen([0.0, 0.0]), self.to_screen([self.job.width, self.job.height]));
        match g.axis {
            // both sit outside their ruler, the same gap away, centred on the line; the label of a
            // horizontal guide stands upright (like the left ruler's numbers), so it needs no more
            // room beside the ruler than the other needs above it
            GuideAxis::Vertical => Pos2::new(self.to_screen([g.offset, 0.0]).x, board.top() - RULER - LABEL_GAP - LABEL_THICK / 2.0),
            GuideAxis::Horizontal => Pos2::new(board.left() - RULER - LABEL_GAP - LABEL_THICK / 2.0, self.to_screen([0.0, g.offset]).y),
        }
    }

    /// Draw the guides across the whole board, in the guide's colour (their handles and numbers
    /// come after the rulers, in `paint_guide_labels`).
    pub(super) fn paint_guides(&self, painter: &egui::Painter) {
        let board = Rect::from_two_pos(self.to_screen([0.0, 0.0]), self.to_screen([self.job.width, self.job.height]));
        // the guide a dragged shape would snap to is drawn thicker
        let target = self.guide_snap_preview();
        for g in &self.guides {
            let selected = self.guide_selected == Some(g.id);
            let tone = if target.contains(&g.id) { guide_colour() } else { self.guide_tone(g) };
            let width = if target.contains(&g.id) { 2.4_f32 } else if self.guides_locked { 0.7 } else if selected { 1.6 } else { 1.0 };
            let line = Stroke::new(width, tone);
            match g.axis {
                GuideAxis::Vertical => {
                    let x = self.to_screen([g.offset, 0.0]).x;
                    painter.line_segment([Pos2::new(x, board.top()), Pos2::new(x, board.bottom())], line);
                }
                GuideAxis::Horizontal => {
                    let y = self.to_screen([0.0, g.offset]).y;
                    painter.line_segment([Pos2::new(board.left(), y), Pos2::new(board.right(), y)], line);
                }
            }
        }
    }

    /// The guides' handles on the ruler and their numbers (the same label the constraints use),
    /// drawn after the rulers so a ruler never covers them. Remembers where the numbers are, for
    /// the long press that edits them.
    pub(super) fn paint_guide_labels(&mut self, painter: &egui::Painter) {
        for g in &self.guides {
            // the handle: a filled tab across the ruler in the line's colour, with grip lines
            let tone = self.guide_tone(g);
            let r = self.guide_handle_rect(g);
            painter.rect_filled(r, 2.0, tone);
            let grip = Stroke::new(1.0_f32, Color32::WHITE.gamma_multiply(0.85));
            let c = r.center();
            for k in [-2.0_f32, 2.0] {
                match g.axis {
                    GuideAxis::Vertical => painter.line_segment([Pos2::new(c.x + k * 0.6, c.y - 4.0), Pos2::new(c.x + k * 0.6, c.y + 4.0)], grip),
                    GuideAxis::Horizontal => painter.line_segment([Pos2::new(c.x - 4.0, c.y + k * 0.6), Pos2::new(c.x + 4.0, c.y + k * 0.6)], grip),
                };
            }
        }
        // the lock in the rulers' corner
        let corner = self.ruler_corner();
        paint_lock(painter, corner.center(), self.guides_locked, if self.guides_locked { guide_colour() } else { crate::theme::RULER_INK });
        let mut labels = Vec::with_capacity(self.guides.len());
        for g in &self.guides {
            if self.guide_edit.as_ref().is_some_and(|e| e.id == g.id) {
                continue;
            }
            // written like a distance constraint's value: a formula shows itself and its result
            let text = match &g.expr {
                Some(e) => format!("ƒ {e} = {} mm", super::fields::fmt(g.offset)),
                None => format!("{} mm", super::fields::fmt(g.offset)),
            };
            let (at, tone) = (self.guide_label_pos(g), self.guide_tone(g));
            let rect = match g.axis {
                GuideAxis::Vertical => super::fields::value_label(painter, at, text, tone),
                GuideAxis::Horizontal => upright_label(painter, at, text, tone),
            };
            labels.push((g.id, rect));
        }
        self.guide_labels = labels;
    }

    /// A guide's handle on the ruler: a tab across the ruler, centred on the line. It is what is
    /// drawn, hovered and grabbed.
    fn guide_handle_rect(&self, g: &Guide) -> Rect {
        let board = Rect::from_two_pos(self.to_screen([0.0, 0.0]), self.to_screen([self.job.width, self.job.height]));
        match g.axis {
            GuideAxis::Vertical => {
                let x = self.to_screen([g.offset, 0.0]).x;
                Rect::from_min_max(Pos2::new(x - HANDLE_W / 2.0, board.top() - RULER + 2.0), Pos2::new(x + HANDLE_W / 2.0, board.top() - 1.0))
            }
            GuideAxis::Horizontal => {
                let y = self.to_screen([0.0, g.offset]).y;
                Rect::from_min_max(Pos2::new(board.left() - RULER + 2.0, y - HANDLE_W / 2.0), Pos2::new(board.left() - 1.0, y + HANDLE_W / 2.0))
            }
        }
    }

    /// Every frame, over the ruler: the resize cursor over a handle, and a long press on a guide's
    /// number opens it for typing, exactly like the constraints' values (same hold time, same
    /// filling bar under the value).
    pub(super) fn guides_overlay(&mut self, ui: &mut egui::Ui) {
        if !self.rulers_shown() {
            self.guide_edit = None;
            self.guide_press = None;
            return;
        }
        // the lock in the corner: a click locks or opens the construction lines
        let corner = ui.interact(self.ruler_corner(), egui::Id::new("guides_lock"), egui::Sense::click());
        let tip = if self.guides_locked {
            super::Tip::new(t!("Hilfslinien gesperrt"), t!("Klicken, um die Hilfslinien wieder zu entsperren."))
        } else {
            super::Tip::new(t!("Hilfslinien sperren"), t!("Klicken, damit Hilfslinien nicht mehr verschoben, bearbeitet oder gelöscht werden können. Formen rasten weiter an ihnen ein."))
        };
        let corner = super::WithTip::tip(corner.on_hover_cursor(egui::CursorIcon::PointingHand), &tip);
        if corner.clicked() {
            let ctx = ui.ctx().clone();
            self.set_guides_locked(&ctx, !self.guides_locked);
        }
        let now = ui.input(|i| i.time);
        let pointer = ui.input(|i| i.pointer.hover_pos());
        if self.guides_locked {
            // locked: no editing; handles and numbers say so
            let over = pointer.is_some_and(|p| self.guides.iter().any(|g| self.guide_handle_rect(g).expand(GRAB_PX).contains(p)) || self.guide_labels.iter().any(|(_, r)| r.contains(p)));
            if over {
                ui.ctx().set_cursor_icon(egui::CursorIcon::NotAllowed);
            }
            self.guide_press = None;
            self.guide_edit = None;
            self.show_guide_edit(ui);
            return;
        }
        if self.guide_edit.is_none() {
            let over_handle = pointer.and_then(|p| self.guides.iter().find(|g| self.guide_handle_rect(g).expand(GRAB_PX).contains(p)).map(|g| g.axis));
            let over_label = pointer.is_some_and(|p| self.guide_labels.iter().any(|(_, r)| r.contains(p)));
            if let Some(axis) = over_handle {
                ui.ctx().set_cursor_icon(if axis == GuideAxis::Vertical { egui::CursorIcon::ResizeHorizontal } else { egui::CursorIcon::ResizeVertical });
            } else if over_label {
                // the number can be edited (long press)
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
        }
        // the long press belongs to the number, not the handle
        let label = pointer.and_then(|p| self.guide_labels.iter().find(|(_, r)| r.contains(p)).copied());
        let down = ui.input(|i| i.pointer.primary_down());
        if !down || self.guide_drag.is_some() || self.guide_edit.is_some() {
            self.guide_press = None;
        } else if let Some((id, rect)) = label {
            match self.guide_press {
                Some((pid, since)) if pid == id => {
                    if now - since >= super::distance::HOLD_SECS {
                        if let Some(g) = self.guides.iter().find(|g| g.id == id) {
                            let text = g.expr.clone().unwrap_or_else(|| super::fields::fmt(g.offset));
                            self.guide_edit = Some(GuideEdit { id, text, rect, input: ConstraintInput::selecting(0) });
                            self.guide_selected = Some(id);
                        }
                        self.guide_press = None;
                    } else {
                        // the same thin bar the constraints show: "keep holding"
                        let progress = ((now - since) / super::distance::HOLD_SECS).clamp(0.0, 1.0) as f32;
                        let painter = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("guide_hold_progress")));
                        painter.line_segment([rect.left_bottom() + egui::vec2(0.0, 3.0), rect.left_bottom() + egui::vec2(rect.width() * progress, 3.0)], Stroke::new(2.5_f32, guide_colour()));
                        ui.ctx().request_repaint_after(std::time::Duration::from_millis(30));
                    }
                }
                _ => {
                    self.guide_press = Some((id, now));
                    ui.ctx().request_repaint_after(std::time::Duration::from_millis(30));
                }
            }
        } else {
            self.guide_press = None;
        }
        self.show_guide_edit(ui);
    }

    /// The open guide-number field: runs the shared number field (same as the shape sizes, the
    /// distance values and the fillet radii), so it behaves the same everywhere. It takes a number,
    /// a variable or a formula ("rand", "platte_b/2"); a formula is kept and the line follows its
    /// variables.
    fn show_guide_edit(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let id = egui::Id::new("guide_value_field");
        let Some(mut e) = self.guide_edit.take() else {
            // the field is gone: so is its keyboard focus (tool keys must not type into it)
            ctx.memory_mut(|m| {
                if m.has_focus(id) {
                    m.surrender_focus(id);
                }
            });
            return;
        };
        if !self.guides.iter().any(|g| g.id == e.id) {
            return;
        }
        let names = self.var_names();
        let value = |n: &str| self.vars_resolved.value(n).ok();
        let tab = std::mem::take(&mut self.guide_tab);
        let (outcome, _) = super::input::edit_number_field(&ctx, id, e.rect.center(), &mut e.text, &mut e.input, &names, &value, &self.vars_resolved, false, tab, false);
        match outcome {
            super::input::NumberOutcome::Open => self.guide_edit = Some(e),
            super::input::NumberOutcome::Cancel => self.status = "Abgebrochen".into(),
            super::input::NumberOutcome::Commit => self.take_guide_value(&e),
        }
    }

    /// Put the typed value on its guide: a plain number is just a distance; a variable or formula
    /// is kept, and the line (with its attached shapes) moves to its value.
    fn take_guide_value(&mut self, e: &GuideEdit) {
        let text = e.text.trim().trim_end_matches('=').trim().to_string();
        let Ok(v) = shape_edit::calc::eval(&text, &self.vars_resolved) else { return };
        let expr = shape_edit::calc::is_formula(&text).then_some(text);
        self.move_guide_with(e.id, v, expr);
    }

    /// Lines given as formulas follow their variables (and the job's size): each is worked out
    /// again and moved there, its attached shapes with it. Returns whether any line moved.
    pub(super) fn resolve_guides(&mut self) -> bool {
        let wanted: Vec<(u64, f64)> = self
            .guides
            .iter()
            .filter_map(|g| g.expr.as_ref().and_then(|e| shape_edit::calc::eval(e, &self.vars_resolved).ok()).map(|v| (g.id, v)))
            .filter(|(id, v)| self.guides.iter().any(|g| g.id == *id && (g.offset - v).abs() > 1e-9))
            .collect();
        for (id, v) in &wanted {
            self.apply_guide_move(*id, *v);
        }
        !wanted.is_empty()
    }

    pub(super) fn guide_ruler_interact(&mut self, resp: &egui::Response) -> bool {
        if !self.rulers_shown() {
            return false;
        }
        let board = Rect::from_two_pos(self.to_screen([0.0, 0.0]), self.to_screen([self.job.width, self.job.height]));
        let top = Rect::from_min_max(Pos2::new(board.left(), board.top() - RULER), Pos2::new(board.right(), board.top()));
        let left = Rect::from_min_max(Pos2::new(board.left() - RULER, board.top()), Pos2::new(board.left(), board.bottom()));
        let Some(p) = resp.interact_pointer_pos().or_else(|| resp.hover_pos()) else { return false };
        // where the button first went down (a drag only reports once the pointer has moved a bit,
        // by then it may be off the thin handle); fall back to the current position
        let grab = resp.ctx.input(|i| i.pointer.press_origin()).unwrap_or(p);
        // a press on a guide's number belongs to its long press, never to the drawing
        if self.guide_labels.iter().any(|(_, r)| r.contains(grab)) {
            return true;
        }
        let on_top = top.contains(p) || top.contains(grab);
        let on_left = left.contains(p) || left.contains(grab);
        if self.guides_locked {
            // locked: the rulers add, move and pick no lines (the press is still theirs)
            self.guide_drag = None;
            return on_top || on_left;
        }

        // pointer in mm, on whole millimetres and on the board: a line placed or dragged by hand
        // always sits on a round value (a typed value may have decimals)
        let at_v = whole_mm((p.x - self.origin.x) / self.zoom, self.job.width);
        let at_h = whole_mm((self.origin.y - p.y) / self.zoom, self.job.height);

        // which existing guide's handle is under the press point on the ruler?
        let near = self.guides.iter().find(|g| match g.axis {
            GuideAxis::Vertical | GuideAxis::Horizontal => self.guide_handle_rect(g).expand(GRAB_PX).contains(grab),
        }).map(|g| g.id);

        if resp.drag_started_by(egui::PointerButton::Primary) {
            if let Some(id) = near {
                // grab an existing guide; the drag's final command makes it a plain number
                let off = self.guides.iter().find(|g| g.id == id).map(|g| g.offset).unwrap_or(0.0);
                self.guide_drag = Some((id, off));
                self.guide_selected = Some(id);
                return true;
            }
            if on_top || on_left {
                // a drag starting on empty ruler makes a new guide and grabs it at once
                let axis = if on_top { GuideAxis::Vertical } else { GuideAxis::Horizontal };
                let offset = if on_top { at_v } else { at_h };
                let id = self.add_guide(axis, offset);
                self.guide_drag = Some((id, offset));
                self.guide_selected = Some(id);
                return true;
            }
        }

        if let Some((id, start)) = self.guide_drag {
            if resp.dragged() {
                let axis = self.guides.iter().find(|g| g.id == id).map(|g| g.axis);
                if let Some(axis) = axis {
                    let offset = if axis == GuideAxis::Vertical { at_v } else { at_h };
                    // live, without an undo step; the whole drag becomes one command when it ends
                    self.apply_guide_move(id, offset);
                }
                resp.ctx.set_cursor_icon(if matches!(axis, Some(GuideAxis::Vertical)) { egui::CursorIcon::ResizeHorizontal } else { egui::CursorIcon::ResizeVertical });
                return true;
            }
            if resp.drag_stopped() {
                // one undo step for the whole drag: put the line back to where the drag began,
                // then let the command move it to where it was dropped (shapes follow both ways)
                let end = self.guides.iter().find(|g| g.id == id).map(|g| g.offset);
                self.guide_drag = None;
                if let Some(end) = end {
                    if (end - start).abs() > 1e-9 {
                        self.apply_guide_move(id, start);
                        self.move_guide(id, end);
                    }
                }
                return true;
            }
        }

        // a plain click on a ruler handle selects it; a click on the empty ruler adds a new guide
        // there (and selects it)
        if resp.clicked() && (on_top || on_left) {
            match near {
                Some(id) => self.guide_selected = Some(id),
                None => {
                    let axis = if on_top { GuideAxis::Vertical } else { GuideAxis::Horizontal };
                    let offset = if on_top { at_v } else { at_h };
                    let id = self.add_guide(axis, offset);
                    self.guide_selected = Some(id);
                }
            }
            return true;
        }
        // a click anywhere else clears a guide selection (the drawing has no guide interaction)
        if resp.clicked() && self.guide_selected.is_some() {
            self.guide_selected = None;
        }
        self.guide_drag.is_some()
    }

    /// Add a guide through the command stack (undoable) and return its id.
    fn add_guide(&mut self, axis: GuideAxis, offset: f64) -> u64 {
        let id = self.next_guide_id;
        self.next_guide_id += 1;
        self.execute_command(Box::new(crate::commandstack::GuideAddCommand::new(Guide::new(id, axis, offset))));
        id
    }

    /// Move a guide through the command stack (undoable): its attached shapes move with it, and a
    /// hand move drops any formula. Recomputes if shapes followed.
    fn move_guide(&mut self, id: u64, offset: f64) {
        self.move_guide_with(id, offset, None);
    }

    /// Like [`move_guide`], but also sets the line's formula (so typing a formula is one undo
    /// step); `None` makes it a plain number.
    fn move_guide_with(&mut self, id: u64, offset: f64, expr: Option<String>) {
        let Some(g) = self.guides.iter().find(|g| g.id == id) else { return };
        if (g.offset - offset).abs() < 1e-9 && g.expr == expr {
            return;
        }
        let (from, had_attached) = (g.offset, !g.attached.is_empty());
        self.execute_command(Box::new(crate::commandstack::GuideMoveCommand::new(id, from, offset).with_expr(expr)));
        if had_attached {
            self.recompute_all();
        }
    }

    /// Move a guide without an undo step: used when variables or the job size change, so the whole
    /// recompute is one action. Its attached shapes move with it.
    fn apply_guide_move(&mut self, id: u64, offset: f64) {
        let Some(g) = self.guides.iter().find(|g| g.id == id) else { return };
        let (axis, old, attached) = (g.axis, g.offset, g.attached.clone());
        let delta = offset - old;
        if delta == 0.0 {
            return;
        }
        let (dx, dy) = if axis == GuideAxis::Vertical { (delta, 0.0) } else { (0.0, delta) };
        for a in &attached {
            if let Some(item) = self.shapes.iter_mut().find(|s| s.id == a.shape) {
                *item = item.translated(dx, dy);
            }
        }
        if let Some(g) = self.guides.iter_mut().find(|g| g.id == id) {
            g.offset = offset;
        }
    }

    /// Delete the selected guide through the command stack (undoable); its shapes stay put.
    pub(super) fn delete_selected_guide(&mut self) -> bool {
        if self.guides_locked {
            return false;
        }
        let Some(id) = self.guide_selected else { return false };
        self.execute_command(Box::new(crate::commandstack::GuideRemoveCommand::new(id)));
        self.guide_selected = None;
        true
    }

    /// While the user drags `moved` shapes, snap each of their edges/centre that comes within the
    /// snap radius of a matching guide, and attach it to that guide. Dragging away releases it.
    /// Returns an extra offset to apply so the snapped edge sits exactly on the guide.
    pub(super) fn guide_snap_offset(&mut self, moved: &[u64]) -> [f64; 2] {
        if self.guides.is_empty() || moved.is_empty() {
            return [0.0, 0.0];
        }
        let best = self.guide_snap_candidates(moved);
        // apply: attach the snapped shapes to their guides, detach from others
        let mut delta = [0.0, 0.0];
        for (k, slot) in best.iter().enumerate() {
            if let Some((_, snap_delta, guide, side)) = *slot {
                delta[k] = snap_delta;
                for &shape in moved {
                    self.attach_shape(shape, guide, side);
                }
            } else {
                // no guide on this axis in reach: release the dragged shapes from axis-k guides
                let axis = if k == 0 { GuideAxis::Vertical } else { GuideAxis::Horizontal };
                for &shape in moved {
                    self.detach_shape_from_axis(shape, axis);
                }
            }
        }
        delta
    }

    /// For each axis (X, Y), the guide `moved` would snap to now: the closest edge or centre of any
    /// of the shapes within the snap radius of a guide, as (distance, the shift that puts it on the
    /// guide, the guide, which part). Only looks, changes nothing.
    fn guide_snap_candidates(&self, moved: &[u64]) -> [Option<(f64, f64, u64, Side)>; 2] {
        let tol = SNAP_PX as f64 / self.zoom as f64;
        let mut best: [Option<(f64, f64, u64, Side)>; 2] = [None, None];
        for &shape in moved {
            for g in &self.guides {
                let k = if g.axis == GuideAxis::Vertical { 0 } else { 1 };
                for side in [Side::Low, Side::Centre, Side::High] {
                    let Some(v) = self.part_value(shape, g.axis, side) else { continue };
                    let d = (g.offset - v).abs();
                    if d <= tol && best[k].is_none_or(|(bd, ..)| d < bd) {
                        best[k] = Some((d, g.offset - v, g.id, side));
                    }
                }
            }
        }
        best
    }

    /// While the user drags shapes: the guides they would snap to if released now (empty when
    /// none, or when nothing is being moved). Drives the visual feedback, attaches nothing.
    pub(super) fn guide_snap_preview(&self) -> Vec<u64> {
        if self.guides.is_empty() || !matches!(self.edit.gesture(), Some(shape_edit::Gesture::Move { .. })) {
            return vec![];
        }
        self.guide_snap_candidates(&self.edit.selection).iter().flatten().map(|(_, _, g, _)| *g).collect()
    }

    /// Attach `shape` to `guide` by `side`, removing it from any other guide of the same axis.
    fn attach_shape(&mut self, shape: u64, guide: u64, side: Side) {
        let axis = self.guides.iter().find(|g| g.id == guide).map(|g| g.axis);
        for g in &mut self.guides {
            if g.id == guide {
                g.attach(shape, side);
            } else if Some(g.axis) == axis {
                g.detach(shape);
            }
        }
    }

    /// Release `shape` from every guide of `axis`.
    fn detach_shape_from_axis(&mut self, shape: u64, axis: GuideAxis) {
        for g in &mut self.guides {
            if g.axis == axis {
                g.detach(shape);
            }
        }
    }

    /// Drop attachments to shapes that no longer exist (after a delete).
    pub(super) fn prune_guide_attachments(&mut self) {
        let ids: std::collections::HashSet<u64> = self.shapes.iter().map(|s| s.id).collect();
        for g in &mut self.guides {
            g.attached.retain(|a| ids.contains(&a.shape));
        }
    }

}

/// The constraints' value label (`fields::value_label`: same font, fill, frame and corners),
/// standing upright and reading upwards, centred on `at`; for numbers beside the left ruler.
fn upright_label(painter: &egui::Painter, at: Pos2, text: String, tone: Color32) -> Rect {
    let galley = painter.layout_no_wrap(text, egui::FontId::monospace(12.0), tone);
    let size = galley.size();
    // a quarter turn of the label's box: width and height swap
    let r = Rect::from_center_size(at, egui::vec2(size.y + 4.0, size.x + 10.0));
    painter.rect(r, 4.0, crate::theme::SHEET_LABEL, Stroke::new(1.0_f32, tone));
    let origin = Pos2::new(r.left() + 2.0, r.bottom() - 5.0);
    painter.add(egui::epaint::TextShape::new(origin, galley, tone).with_angle(-std::f32::consts::FRAC_PI_2));
    r
}

/// A small padlock centred on `c`: closed when `locked`, its shackle lifted open otherwise.
fn paint_lock(painter: &egui::Painter, c: Pos2, locked: bool, colour: Color32) {
    let body = Rect::from_min_max(Pos2::new(c.x - 4.5, c.y - 0.5), Pos2::new(c.x + 4.5, c.y + 5.5));
    painter.rect_filled(body, 1.5, colour);
    let lift = if locked { 0.0 } else { 3.0 };
    let top = c.y - 4.0 - lift;
    let mut shackle = vec![Pos2::new(c.x - 3.0, c.y - 0.5)];
    shackle.extend((0..=10).map(|i| {
        let a = std::f32::consts::PI * (1.0 + i as f32 / 10.0);
        Pos2::new(c.x + 3.0 * a.cos(), top + 3.0 * a.sin())
    }));
    // open: the right leg stops short of the body
    shackle.push(Pos2::new(c.x + 3.0, if locked { c.y - 0.5 } else { top + 1.5 }));
    painter.add(egui::Shape::line(shackle, Stroke::new(1.6_f32, colour)));
}

/// `mm` rounded to a whole millimetre and kept on the board (0 to `len`, whole millimetres only).
fn whole_mm(mm: f32, len: f64) -> f64 {
    (mm as f64).round().clamp(0.0, len.floor())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dragged_lines_land_on_whole_millimetres_on_the_board() {
        assert_eq!(whole_mm(65.45, 300.0), 65.0);
        assert_eq!(whole_mm(65.5, 300.0), 66.0);
        assert_eq!(whole_mm(-3.0, 300.0), 0.0);
        assert_eq!(whole_mm(300.4, 300.5), 300.0);
        assert_eq!(whole_mm(400.0, 300.5), 300.0);
    }
}
