//! The selection decoration of an open vector (a line or a polyline), as in Fusion 360: instead of
//! a box, a round handle sits on every node and each segment can be picked on its own. A picked or
//! hovered segment is drawn thicker; the cursor tells what a drag would do — a node moves freely,
//! a segment only across itself (parallel), the centre mark moves the whole. The shape class only
//! chooses this style ([`shape_edit::SelectionStyle::Polyline`]); the editor owns the gestures and
//! this file owns the look and the hints, like a draw2d figure's own selection feedback.

use super::{App, DrawTool, ShapeEditor};
use crate::theme::{ACCENT, SELECT};
use egui::{Color32, CursorIcon, Painter, Stroke};
use shape_edit::{Gesture, Item, Pt, SelectionStyle, View};

impl App {
    pub(super) fn draw_polyline_selection(&self, painter: &Painter, ctx: &egui::Context, mouse: Option<Pt>, view: View) {
        let Some(it) = self.edit.selection.first().and_then(|id| self.shapes.iter().find(|s| s.id == *id)) else { return };
        let gesture = self.edit.gesture();
        let dragging_node = matches!(gesture, Some(Gesture::Node { .. }));
        let dragging_segment = matches!(gesture, Some(Gesture::Segment { .. }));
        // the pointer's target, in the order a press would take it: node, centre mark, segment
        let node_hover = mouse.and_then(|m| self.edit.node_handle_under(&self.shapes, m, view));
        let on_centre = mouse.zip(self.edit.centre_mark(&self.shapes)).is_some_and(|(m, c)| (m[0] - c[0]).hypot(m[1] - c[1]) <= view.tolerance);
        // with the whole polyline selected a drag on a segment moves it all (the selection rule):
        // then a segment under the pointer is no single target and isn't singled out
        let whole = self.edit.polyline_whole(&self.shapes);
        let segment_hover = mouse.filter(|_| node_hover.is_none() && !on_centre && !whole).and_then(|m| self.edit.segment_under(&self.shapes, m, view));

        // the picked segment and the one under the pointer change colour only, never width: they
        // are drawn over the solid outline at its own width
        let draw_segment = |k: usize, colour: Color32| {
            if let Some((_, _, a, b)) = ShapeEditor::<Item>::segment_ends(it, k) {
                painter.line_segment([self.to_screen(a), self.to_screen(b)], Stroke::new(super::SELECTED_OUTLINE_WIDTH, colour));
            }
        };
        if let Some((_, k)) = segment_hover.filter(|s| Some(*s) != self.edit.active_segment) {
            draw_segment(k, ACCENT.gamma_multiply(0.55));
        }
        if let Some(k) = self.edit.active_segment.filter(|(id, _)| *id == it.id).map(|(_, k)| k) {
            draw_segment(k, ACCENT);
        }

        // a handle on every node: the picked one filled, the one under the pointer bigger
        for (id, index, at) in self.edit.node_handles(&self.shapes) {
            let picked = self.edit.active_node == Some((id, index));
            let hot = node_hover == Some((id, index)) || (dragging_node && picked);
            let fill = if picked || hot { ACCENT } else { Color32::WHITE };
            painter.circle(self.to_screen(at), if hot { 6.5 } else { 5.0 }, fill, Stroke::new(1.5_f32, SELECT));
        }

        // the cursor: free move on a node, the move cursor over a wholly selected polyline, a
        // two-way arrow across the segment on a single segment (the centre mark already shows the
        // move cursor for the whole)
        let on_outline = mouse.is_some_and(|m| self.edit.shape_under(&self.shapes, m, view) == Some(it.id));
        if node_hover.is_some() || dragging_node {
            ctx.set_cursor_icon(CursorIcon::Crosshair);
        } else if whole && on_outline {
            ctx.set_cursor_icon(CursorIcon::Move);
        } else if let Some(k) = segment_hover.map(|(_, k)| k).or(if dragging_segment { self.edit.active_segment.map(|(_, k)| k) } else { None }) {
            if let Some((_, _, a, b)) = ShapeEditor::<Item>::segment_ends(it, k) {
                ctx.set_cursor_icon(across_cursor(a, b));
            }
        }
    }

    /// After a click with the select tool: what was picked on an open vector, and what a drag of it
    /// does (the editing isn't visible otherwise until one tries).
    pub(super) fn polyline_status(&mut self) {
        if self.tool != DrawTool::Select || self.edit.style(&self.shapes) != SelectionStyle::Polyline {
            return;
        }
        self.status = if self.edit.active_node.is_some() {
            "Punkt gewählt: ziehen bewegt ihn frei · Pfeiltasten schieben · Entf löscht ihn".into()
        } else if self.edit.active_segment.is_some() {
            "Linie gewählt: ziehen verschiebt sie parallel · eine andere Linie ziehen wählt und verschiebt nur diese · Mittelpunkt bewegt alles".into()
        } else {
            "Linienzug ganz gewählt: ziehen bewegt alles · eine Linie anklicken wählt nur sie · Punkte ziehen frei".into()
        };
    }
}

/// The resize cursor whose arrows point across the segment a→b (the way it can be dragged), to the
/// nearest of the four directions egui offers. The view shows world Y up, so a direction keeps its
/// look on screen: the normal of a segment rising to the right points up-left.
fn across_cursor(a: Pt, b: Pt) -> CursorIcon {
    // the normal's direction, folded onto 0°..180° (a two-way arrow has no sense of direction)
    let angle = (b[1] - a[1]).atan2(b[0] - a[0]).to_degrees() + 90.0;
    let folded = angle.rem_euclid(180.0);
    match folded {
        f if !(22.5..157.5).contains(&f) => CursorIcon::ResizeHorizontal,
        f if f < 67.5 => CursorIcon::ResizeNeSw,
        f if f < 112.5 => CursorIcon::ResizeVertical,
        _ => CursorIcon::ResizeNwSe,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cursor_points_across_the_segment() {
        assert_eq!(across_cursor([0.0, 0.0], [10.0, 0.0]), CursorIcon::ResizeVertical, "a level segment moves up and down");
        assert_eq!(across_cursor([0.0, 0.0], [0.0, 10.0]), CursorIcon::ResizeHorizontal, "an upright one sideways");
        // rising to the right: its normal points up-left / down-right
        assert_eq!(across_cursor([0.0, 0.0], [10.0, 10.0]), CursorIcon::ResizeNwSe);
        assert_eq!(across_cursor([0.0, 0.0], [10.0, -10.0]), CursorIcon::ResizeNeSw);
    }
}
