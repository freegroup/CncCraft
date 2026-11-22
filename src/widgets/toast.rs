//! A toast: a short message that appears at the bottom of the window, stays a moment and fades
//! out on its own. Anything can post one (`post`); the app shows the current one once a frame
//! (`show`). A new message replaces the one showing.

use egui::{Align2, Color32, Context, Id, RichText, Vec2};

/// How long a toast stays, seconds (the last part fades out).
const STAYS: f64 = 2.8;
const FADE: f64 = 0.5;

fn key() -> Id {
    Id::new("cnccraft_toast")
}

/// Show `text` as a toast now.
pub fn post(ctx: &Context, text: impl Into<String>) {
    let now = ctx.input(|i| i.time);
    ctx.data_mut(|d| d.insert_temp(key(), (text.into(), now)));
    ctx.request_repaint();
}

/// Draw the toast that is showing, if any (call once a frame).
pub fn show(ctx: &Context) {
    let Some((text, since)) = ctx.data(|d| d.get_temp::<(String, f64)>(key())) else { return };
    let age = ctx.input(|i| i.time) - since;
    if age > STAYS {
        ctx.data_mut(|d| d.remove::<(String, f64)>(key()));
        return;
    }
    let alpha = ((STAYS - age) / FADE).clamp(0.0, 1.0) as f32;
    // on the dark theme the toast takes the app's yellow accent (dark text on it), on the light
    // theme a dark box with white text: either way it stands out from the window around it
    let (fill, edge, ink) = if ctx.style().visuals.dark_mode {
        (crate::theme::ACCENT, crate::theme::ACCENT_DEEP, crate::theme::INK)
    } else {
        (Color32::from_rgb(0x2b, 0x2d, 0x31), Color32::from_rgb(0x2b, 0x2d, 0x31), Color32::WHITE)
    };
    egui::Area::new(key().with("area"))
        .anchor(Align2::CENTER_BOTTOM, Vec2::new(0.0, -56.0))
        .order(egui::Order::Tooltip)
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(fill.gamma_multiply(0.97 * alpha))
                .stroke(egui::Stroke::new(1.0_f32, edge.gamma_multiply(alpha)))
                .rounding(8.0)
                .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(text).color(ink.gamma_multiply(alpha)).size(13.5));
                });
        });
    ctx.request_repaint_after(std::time::Duration::from_millis(30));
}
