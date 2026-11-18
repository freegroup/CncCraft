use egui::{Align2, FontId, Pos2, Sense, Vec2};
use crate::theme::ink;

/// A small eye (crossed out when hidden) with an optional label, clickable.
pub fn eye(ui: &mut egui::Ui, open: bool, label: &str) -> egui::Response {
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
