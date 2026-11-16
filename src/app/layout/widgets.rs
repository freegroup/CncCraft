//! Small pieces of interface that several windows share.

use crate::theme::{hairline, ink, muted};
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use super::tool_column::GAP;

/// A small eye (crossed out when hidden) with an optional label, clickable.
pub(in crate::app) fn eye(ui: &mut egui::Ui, open: bool, label: &str) -> egui::Response {
    let text_w = if label.is_empty() { 0.0 } else { ui.fonts(|f| f.layout_no_wrap(label.to_owned(), FontId::proportional(13.0), ink()).size().x) + 8.0 };
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(22.0 + text_w, 20.0), Sense::click());
    let p = ui.painter();
    let c = Pos2::new(rect.left() + 11.0, rect.center().y);
    crate::app::icons::paint_eye(p, c, open, crate::app::icons::Colours::normal(), resp.hovered());
    if !label.is_empty() {
        p.text(Pos2::new(rect.left() + 26.0, rect.center().y), Align2::LEFT_CENTER, label, FontId::proportional(13.0), if resp.hovered() { crate::theme::icon_line() } else { ink() });
    }
    resp
}

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
            // the name under the buttons, as wide as they are, so it is centred and the box stays
            // only as wide as its content
            ui.allocate_ui_with_layout(Vec2::new(row.response.rect.width(), 12.0), egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(name).size(10.0).color(muted()));
            });
        });
    });
}

/// A ComboBox whose popup has a scrollable item list and a pinned "Verwalten…" button below it.
/// `items` draws the selectable rows; returns `true` when "Verwalten…" was clicked. `tip` shows
/// while the list is closed.
pub(in crate::app) fn combo_with_manage(
    ui: &mut egui::Ui,
    id: &str,
    width: f32,
    selected: &str,
    tip: Option<&crate::app::Tip>,
    items: impl FnOnce(&mut egui::Ui),
) -> bool {
    let widget_id = ui.make_persistent_id(id);
    let popup_id = widget_id.with("mgr_popup");
    let is_open = ui.memory(|m| m.is_popup_open(popup_id));

    let margin = ui.spacing().button_padding;
    let icon_size = Vec2::splat(ui.spacing().icon_width);
    let total_w = width.max(ui.spacing().combo_width);
    let galley = ui.painter().layout_no_wrap(
        selected.to_string(),
        egui::FontId::default(),
        ui.visuals().text_color(),
    );
    let desired = Vec2::new(total_w, galley.size().y.max(icon_size.y) + 2.0 * margin.y);
    let (rect, mut button) = ui.allocate_exact_size(desired, Sense::click());
    if let (Some(tip), false) = (tip, is_open) {
        use crate::app::WithTip;
        button = button.tip(tip);
    }

    if ui.is_rect_visible(rect) {
        let visuals = if is_open {
            &ui.visuals().widgets.open
        } else {
            ui.style().interact(&button)
        };
        ui.painter().rect(rect, visuals.rounding, visuals.bg_fill, visuals.bg_stroke);
        let inner = rect.shrink2(margin);
        let icon_rect = egui::Align2::RIGHT_CENTER.align_size_within_rect(icon_size, inner);
        let tri = Rect::from_center_size(icon_rect.center(), Vec2::new(icon_rect.width() * 0.7, icon_rect.height() * 0.45));
        ui.painter().add(egui::Shape::convex_polygon(
            vec![tri.left_top(), tri.right_top(), tri.center_bottom()],
            visuals.fg_stroke.color,
            Stroke::NONE,
        ));
        let text_rect = egui::Align2::LEFT_CENTER.align_size_within_rect(galley.size(), inner);
        ui.painter().galley(text_rect.min, galley, visuals.text_color());
    }

    if button.clicked() {
        ui.memory_mut(|m| m.toggle_popup(popup_id));
    }

    let mut manage = false;
    let above_or_below = if button.rect.center().y > ui.ctx().screen_rect().center().y {
        egui::AboveOrBelow::Above
    } else {
        egui::AboveOrBelow::Below
    };
    egui::popup::popup_above_or_below_widget(
        ui,
        popup_id,
        &button,
        above_or_below,
        egui::PopupCloseBehavior::CloseOnClick,
        |ui| {
            ui.set_min_width(width);
            let max_h = (ui.ctx().screen_rect().height() * 0.45).max(120.0);
            egui::ScrollArea::vertical().max_height(max_h).show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                items(ui);
            });
            ui.separator();
            if ui.button(t!("Verwalten\u{2026}")).clicked() {
                manage = true;
                ui.memory_mut(|m| m.close_popup());
            }
        },
    );
    manage
}
