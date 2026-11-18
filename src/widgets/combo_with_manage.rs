use egui::{Rect, Sense, Stroke, Vec2};
use crate::app::tooltip::{Tip, WithTip};

/// A ComboBox whose popup has a scrollable item list and a pinned "Verwalten…" button below it.
/// `items` draws the selectable rows; returns `true` when "Verwalten…" was clicked. `tip` shows
/// while the list is closed.
pub fn combo_with_manage(
    ui: &mut egui::Ui,
    id: &str,
    width: f32,
    selected: &str,
    tip: Option<&Tip>,
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
