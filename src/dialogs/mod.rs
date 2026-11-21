//! Every dialog of the program, each in its own file here, and what they all share so they look
//! alike: the window itself (no system title bar, the same frame and margins), the header at its
//! top (the program's icon at the top left, the title beside it, left-aligned, the close button
//! at the right, a line below), the titled groups of fields and the
//! row of buttons that ends a dialog.
//!
//! The folder belongs to the `app` module (included from there), so the dialogs reach the
//! program's state like the rest of the interface.

mod about;
mod export;
mod export_toolpaths;
mod import_missing;
mod job;
mod layer;
mod materials;
mod new_variable;
mod postprocessor;
pub(in crate::app) mod shortcuts;
mod tools;
mod toolpath;
mod unsaved_changes;
mod variables;

pub(in crate::app) use export::ExportScope;

use crate::theme::{hairline, ink, muted, panel, surface};
use egui::{Id, Stroke, Vec2};
use std::hash::Hash;

/// The size of the icon in the header (points).
const ICON: f32 = 22.0;

/// A dialog window with the shared look; the caller adds its place (anchor, position), size and
/// order, and starts its contents with [`header`].
pub(in crate::app) fn window(id: impl Hash, ctx: &egui::Context) -> egui::Window<'static> {
    egui::Window::new("")
        .id(Id::new(id))
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .frame(egui::Frame::window(&ctx.style()).fill(panel()).inner_margin(egui::Margin::symmetric(18.0, 14.0)))
}

/// The header every dialog starts with, always with its close button; returns whether that was
/// clicked (the dialog then does what its cancel or close button does).
pub(in crate::app) fn header(ui: &mut egui::Ui, title: &str) -> bool {
    let mut close = false;
    let icon = app_icon(ui.ctx());
    ui.horizontal(|ui| {
        ui.add(egui::Image::new((icon.id(), Vec2::splat(ICON))));
        ui.add_space(4.0);
        ui.label(egui::RichText::new(title).size(15.0).strong().color(ink()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if close_button(ui).on_hover_text(t!("Schließen")).clicked() {
                close = true;
            }
        });
    });
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);
    close
}

/// Like [`header`], but with the toolpath's own operation icon at the left instead of the app icon,
/// so the editor shows which kind of toolpath it edits (Profil, Tasche, Nut …).
pub(in crate::app) fn header_with_strategy(ui: &mut egui::Ui, title: &str, strategy: &crate::toolpath::Strategy) -> bool {
    let mut close = false;
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(ICON), egui::Sense::hover());
        crate::app::icons::operation_icon(ui.painter(), rect.center(), strategy, ICON);
        ui.add_space(4.0);
        ui.label(egui::RichText::new(title).size(15.0).strong().color(ink()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if close_button(ui).on_hover_text(t!("Schließen")).clicked() {
                close = true;
            }
        });
    });
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);
    close
}

/// The close button of the header: a drawn cross (the font has no "✕"), darker under the pointer.
pub(in crate::app) fn close_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(ICON), egui::Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, 4.0, ui.visuals().widgets.hovered.weak_bg_fill);
    }
    let colour = if response.hovered() { ink() } else { muted() };
    let r = rect.shrink(6.0);
    let stroke = Stroke::new(1.6_f32, colour);
    ui.painter().line_segment([r.left_top(), r.right_bottom()], stroke);
    ui.painter().line_segment([r.right_top(), r.left_bottom()], stroke);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A titled group of fields in a dialog: the title, then a white panel with a hairline border.
pub(in crate::app) fn dialog_group(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.add_space(4.0);
    ui.label(egui::RichText::new(title).strong().size(13.0).color(ink()));
    ui.add_space(3.0);
    egui::Frame::none().fill(surface()).stroke(Stroke::new(1.0_f32, hairline())).rounding(5.0).inner_margin(egui::Margin::symmetric(12.0, 10.0)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        add(ui);
    });
    ui.add_space(6.0);
}

/// The button row that ends every dialog: a line above it and the buttons at the right end, the
/// main one outermost (so it is added first).
pub(in crate::app) fn dialog_buttons(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(2.0);
    ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), 30.0), egui::Layout::right_to_left(egui::Align::Center), add);
}

/// The program's icon as a texture, made once and kept for every later dialog.
fn app_icon(ctx: &egui::Context) -> egui::TextureHandle {
    let id = Id::new("dialog_header_app_icon");
    if let Some(t) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return t;
    }
    const SIZE: u32 = 64;
    let image = egui::ColorImage::from_rgba_unmultiplied([SIZE as usize; 2], &crate::appicon::rgba(SIZE));
    let t = ctx.load_texture("dialog_header_app_icon", image, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, t.clone()));
    t
}
