//! A small picture of a material's surface (its look from above), for lists and dialogs. Each look
//! is baked once and kept for later frames.

use egui::{Response, Sense, Stroke, Vec2};

/// Pixels of a baked swatch: enough for the largest one shown.
const SIZE: usize = 128;

/// The surface of the look `look` (a shipped material's key) in a `size` square with rounded corners.
pub fn material_swatch(ui: &mut egui::Ui, look: &str, size: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    if ui.is_rect_visible(rect) {
        let texture = swatch_texture(ui.ctx(), look);
        let rounding = (size * 0.15).min(5.0);
        let mut shape = egui::epaint::RectShape::filled(rect, rounding, egui::Color32::WHITE);
        shape.fill_texture_id = texture.id();
        shape.uv = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));
        ui.painter().add(shape);
        ui.painter().rect_stroke(rect, rounding, Stroke::new(1.0_f32, crate::theme::hairline()));
    }
    response
}

fn swatch_texture(ctx: &egui::Context, look: &str) -> egui::TextureHandle {
    let id = egui::Id::new(("material_swatch", look));
    if let Some(t) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return t;
    }
    let picture = crate::materials::look(look).top;
    let pixels = crate::materials::bake(picture, SIZE * 4);
    // a quarter of the tile, so the grain or brushing is visible at this size
    let mut part = Vec::with_capacity(SIZE * SIZE * 4);
    for row in pixels.chunks(SIZE * 4 * 4).take(SIZE) {
        part.extend_from_slice(&row[..SIZE * 4]);
    }
    let image = egui::ColorImage::from_rgba_unmultiplied([SIZE, SIZE], &part);
    let t = ctx.load_texture(format!("material_swatch_{look}"), image, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, t.clone()));
    t
}
