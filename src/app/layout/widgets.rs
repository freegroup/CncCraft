//! Small pieces of interface that the layout modules share (not reusable outside layout).

use crate::theme::{hairline, ink, muted};
use egui::{Color32, FontId, Pos2, Stroke, Vec2};
use super::tool_column::GAP;

/// A pencil lying at 45°, tip at the lower left: a body with an eraser band and a pointed tip
/// (drawn, because the font has no pencil glyph).
pub(super) fn draw_pencil(painter: &egui::Painter, c: Pos2, tone: Color32) {
    let d = Vec2::new(0.7071, -0.7071);
    let n = Vec2::new(0.7071, 0.7071);
    let h = 2.8;
    let tip = c - d * 7.5;
    let join = c - d * 3.0;
    let end = c + d * 7.0;
    let stroke = Stroke::new(1.3_f32, tone);
    painter.add(egui::Shape::convex_polygon(vec![join + n * h, end + n * h, end - n * h, join - n * h], Color32::TRANSPARENT, stroke));
    painter.add(egui::Shape::convex_polygon(vec![tip, join + n * h, join - n * h], tone.gamma_multiply(0.45), stroke));
    let band = end - d * 2.4;
    painter.line_segment([band + n * h, band - n * h], stroke);
}

/// The box round what belongs together in the toolpath panel.
pub(super) fn group_container() -> egui::Frame {
    egui::Frame::none().fill(crate::theme::surface()).stroke(Stroke::new(1.0_f32, hairline())).rounding(6.0).inner_margin(egui::Margin::same(8.0))
}

/// The title of a group box, a line under it, and room at the right end for a control.
pub(super) fn group_title(ui: &mut egui::Ui, title: &str, right: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(title).size(14.5).strong().color(ink()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), right);
    });
    ui.add_space(2.0);
    let y = ui.cursor().top();
    let (l, r) = (ui.min_rect().left(), ui.min_rect().right());
    ui.painter().line_segment([Pos2::new(l, y), Pos2::new(r, y)], Stroke::new(1.0_f32, hairline()));
    ui.add_space(8.0);
}

pub(super) fn toolbar_group(ui: &mut egui::Ui, name: &str, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none().stroke(Stroke::new(1.0_f32, crate::theme::separator())).rounding(5.0).fill(crate::theme::surface()).inner_margin(egui::Margin { left: 7.0, right: 7.0, top: 3.0, bottom: 2.0 }).show(ui, |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            let row = ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = GAP;
                add(ui);
            });
            ui.allocate_ui_with_layout(Vec2::new(row.response.rect.width(), 12.0), egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(name).size(10.0).color(muted()));
            });
        });
    });
}
