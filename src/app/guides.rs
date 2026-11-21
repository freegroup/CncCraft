//! Construction lines in the app: drawing them over the workpiece, the ruler interaction that
//! creates, moves, selects and deletes them, dragging their attached shapes along, and snapping a
//! shape onto a guide while the user drags the shape.
//!
//! Guides are only grabbed on the ruler, never in the drawing area. A shape attaches to a guide
//! only while the user drags that shape onto it (within the snap radius); the guide then owns the
//! attachment. Dragging the shape away releases it. Moving a guide never attaches a resting shape.

use super::App;
use crate::guides::{Guide, GuideAxis, Side};
use crate::sheet::rulers::RULER;
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke};
use shape_edit::Editable;

/// How close (screen pixels) the pointer must be to a guide on the ruler to grab it.
const GRAB_PX: f32 = 6.0;
/// How close (mm on screen, in pixels) a shape edge must come to a guide to snap while dragging.
const SNAP_PX: f32 = 8.0;
/// Red of the construction lines, like Sketch's ruler guides.
fn guide_colour() -> Color32 {
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
    fn rulers_shown(&mut self) -> bool {
        self.sheet.switches().any(|(name, shown)| name == "Lineale" && *shown)
    }

    /// Draw the guides across the whole board, and their distance labels outside it, on the ruler.
    /// Called from the canvas after the sheet's `Over` layer (so guides sit above the design but
    /// their handles live on the ruler).
    pub(super) fn paint_guides(&self, painter: &egui::Painter) {
        let board = Rect::from_two_pos(self.to_screen([0.0, 0.0]), self.to_screen([self.job.width, self.job.height]));
        for g in &self.guides {
            let selected = self.guide_selected == Some(g.id);
            let col = if selected { guide_colour() } else { guide_colour().gamma_multiply(0.85) };
            let stroke = Stroke::new(if selected { 1.6 } else { 1.0 }, col);
            match g.axis {
                GuideAxis::Vertical => {
                    let x = self.to_screen([g.offset, 0.0]).x;
                    painter.line_segment([Pos2::new(x, board.top()), Pos2::new(x, board.bottom())], stroke);
                    // the distance label on the top ruler, outside the drawing area
                    painter.text(Pos2::new(x, board.top() - RULER / 2.0), Align2::CENTER_CENTER, format!("{:.0}", g.offset), FontId::proportional(10.0), col);
                }
                GuideAxis::Horizontal => {
                    let y = self.to_screen([0.0, g.offset]).y;
                    painter.line_segment([Pos2::new(board.left(), y), Pos2::new(board.right(), y)], stroke);
                    painter.text(Pos2::new(board.left() - RULER / 2.0, y), Align2::CENTER_CENTER, format!("{:.0}", g.offset), FontId::proportional(10.0), col);
                }
            }
        }
    }

    /// Handle the ruler: a click on the top or left ruler adds a guide; a drag on a guide's handle
    /// moves it (and the shapes attached to it); a click selects it. Only runs when the rulers are
    /// shown. Returns whether the ruler took the interaction (so the canvas ignores it).
    pub(super) fn guide_ruler_interact(&mut self, resp: &egui::Response, ui: &egui::Ui) -> bool {
        if !self.rulers_shown() {
            return false;
        }
        let board = Rect::from_two_pos(self.to_screen([0.0, 0.0]), self.to_screen([self.job.width, self.job.height]));
        let top = Rect::from_min_max(Pos2::new(board.left(), board.top() - RULER), Pos2::new(board.right(), board.top()));
        let left = Rect::from_min_max(Pos2::new(board.left() - RULER, board.top()), Pos2::new(board.left(), board.bottom()));
        let Some(p) = resp.interact_pointer_pos().or_else(|| resp.hover_pos()) else { return false };
        let on_top = top.contains(p);
        let on_left = left.contains(p);

        // pointer in mm, clamped to the board
        let at_v = (((p.x - self.origin.x) / self.zoom) as f64).clamp(0.0, self.job.width);
        let at_h = (((self.origin.y - p.y) / self.zoom) as f64).clamp(0.0, self.job.height);

        // which existing guide's handle is under the pointer on the ruler?
        let near = self.guides.iter().find(|g| match g.axis {
            GuideAxis::Vertical => on_top && (self.to_screen([g.offset, 0.0]).x - p.x).abs() <= GRAB_PX,
            GuideAxis::Horizontal => on_left && (self.to_screen([0.0, g.offset]).y - p.y).abs() <= GRAB_PX,
        }).map(|g| g.id);

        if resp.drag_started_by(egui::PointerButton::Primary) {
            if let Some(id) = near {
                // grab an existing guide
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

        if let Some((id, _)) = self.guide_drag {
            if resp.dragged() {
                let axis = self.guides.iter().find(|g| g.id == id).map(|g| g.axis);
                if let Some(axis) = axis {
                    let offset = if axis == GuideAxis::Vertical { at_v } else { at_h };
                    self.move_guide(id, offset);
                }
                ui.ctx().set_cursor_icon(if matches!(axis, Some(GuideAxis::Vertical)) { egui::CursorIcon::ResizeHorizontal } else { egui::CursorIcon::ResizeVertical });
                return true;
            }
            if resp.drag_stopped() {
                self.guide_drag = None;
                return true;
            }
        }

        // a plain click on a ruler handle selects it; a click on empty ruler clears the selection
        if resp.clicked() && (on_top || on_left) {
            self.guide_selected = near;
            return true;
        }
        // a click anywhere else clears a guide selection (the drawing has no guide interaction)
        if resp.clicked() && self.guide_selected.is_some() {
            self.guide_selected = None;
        }
        self.guide_drag.is_some()
    }

    /// Add a guide and return its id.
    fn add_guide(&mut self, axis: GuideAxis, offset: f64) -> u64 {
        let id = self.next_guide_id;
        self.next_guide_id += 1;
        self.guides.push(Guide::new(id, axis, offset));
        id
    }

    /// Move a guide to a new offset and drag its attached shapes along by the same amount.
    fn move_guide(&mut self, id: u64, offset: f64) {
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
        if !attached.is_empty() {
            self.recompute_all();
        }
    }

    /// Delete the selected guide (its attachments go with it; the shapes stay put).
    pub(super) fn delete_selected_guide(&mut self) -> bool {
        let Some(id) = self.guide_selected else { return false };
        self.guides.retain(|g| g.id != id);
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
        let tol = SNAP_PX as f64 / self.zoom as f64;
        let mut best: [Option<(f64, f64, u64, Side)>; 2] = [None, None]; // per axis: (dist, snap_delta, guide, side)
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
        let mut changed = false;
        for g in &mut self.guides {
            if g.axis == axis {
                changed |= g.detach(shape);
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
