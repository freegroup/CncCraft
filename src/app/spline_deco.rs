//! Spline node decorations: the control-handle arms and circles drawn over a spline in
//! node-edit mode, **plus** the double-click interaction (toggle smooth/corner, insert node).
//! Separated from the main canvas code so the shape only decides *which* decoration is active,
//! and the decoration owns both the visuals and the input handling.

use super::{App, DrawTool};
use crate::theme::{ACCENT, SELECT};
use egui::{Color32, Painter, Rect, Stroke, Vec2};
use shape_edit::{Editable, Item, Pt, View};

impl App {
    /// Draw the node handles of every selected shape.  For a spline this means the control-
    /// handle arms and dots; for any other curve class it means the plain node squares.
    pub(super) fn draw_node_handles(&self, painter: &Painter, mouse: Option<Pt>, view: View) {
        if self.tool != DrawTool::NodeEdit {
            return;
        }
        for it in self.shapes.iter().filter(|s| self.edit.selection.contains(&s.id)) {
            let all_nodes = it.nodes().unwrap_or_default();
            if matches!(&it.shape, shape_edit::Shape::Spline(_)) {
                self.draw_spline_nodes(painter, it, &all_nodes);
            } else {
                self.draw_path_nodes(painter, it, &all_nodes);
            }
        }
        if mouse.is_some_and(|m| self.edit.node_under(&self.shapes, m, view).is_some()) {
            painter.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
    }

    /// Handle a double-click in node-edit mode.  Returns `true` if the event was consumed
    /// (either a spline node was toggled or a node was inserted on a segment).
    pub(super) fn node_double_click(&mut self, mouse: Pt, view: View) -> bool {
        if let Some((id, idx)) = self.edit.node_under(&self.shapes, mouse, view) {
            if let Some(item) = self.shapes.iter().find(|s| s.id == id) {
                if let Some(toggled) = item.toggle_spline_node(idx) {
                    let before = self.shapes.clone();
                    if let Some(slot) = self.shapes.iter_mut().find(|s| s.id == id) {
                        *slot = toggled;
                    }
                    self.push_undo_from(before);
                    self.status = "Knotentyp umgeschaltet".into();
                    self.recompute_all();
                    return true;
                }
            }
        }
        let before = self.shapes.clone();
        if self.edit.insert_node(&mut self.shapes, mouse, view) {
            self.push_undo_from(before);
            self.status = "Knoten eingefügt".into();
            self.recompute_all();
            return true;
        }
        false
    }

    fn draw_spline_nodes(&self, painter: &Painter, it: &Item, nodes: &[Pt]) {
        for (si, chunk) in nodes.chunks(3).enumerate() {
            if chunk.len() < 3 {
                continue;
            }
            let (cp_in, pos, cp_out) = (chunk[0], chunk[1], chunk[2]);

            let pos_active = self.edit.active_node == Some((it.id, si * 3 + 1));
            let in_active = self.edit.active_node == Some((it.id, si * 3));
            let out_active = self.edit.active_node == Some((it.id, si * 3 + 2));
            let any_active = pos_active || in_active || out_active;

            let arm_col = if any_active {
                ACCENT
            } else {
                Color32::from_rgba_unmultiplied(0x40, 0x80, 0xd0, 180)
            };

            let pos_s = self.to_screen(pos);
            let ci_s = self.to_screen(cp_in);
            let co_s = self.to_screen(cp_out);

            painter.line_segment([ci_s, pos_s], Stroke::new(1.0_f32, arm_col));
            painter.line_segment([pos_s, co_s], Stroke::new(1.0_f32, arm_col));

            painter.circle_filled(ci_s, if in_active { 5.0 } else { 3.5 }, if in_active { ACCENT } else { arm_col });
            painter.circle_filled(co_s, if out_active { 5.0 } else { 3.5 }, if out_active { ACCENT } else { arm_col });

            let sz = if pos_active { 9.0 } else { 7.0 };
            let fill = if pos_active { ACCENT } else { Color32::WHITE };
            painter.rect(
                Rect::from_center_size(pos_s, Vec2::splat(sz)),
                0.0,
                fill,
                Stroke::new(1.2_f32, SELECT),
            );
        }
    }

    fn draw_path_nodes(&self, painter: &Painter, it: &Item, nodes: &[Pt]) {
        for (i, n) in nodes.iter().enumerate() {
            let on = self.edit.active_node == Some((it.id, i));
            let sz = if on { 9.0 } else { 7.0 };
            let fill = if on { ACCENT } else { Color32::WHITE };
            painter.rect(
                Rect::from_center_size(self.to_screen(*n), Vec2::splat(sz)),
                0.0,
                fill,
                Stroke::new(1.2_f32, SELECT),
            );
        }
    }
}
