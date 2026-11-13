//! The hint at the bottom right of the drawing: a button asking for help on the tool in use.

use super::{Decoration, Layer, SheetAction, SheetView};
use crate::theme::{hairline, ink};
use egui::{Align2, Sense, Stroke, Vec2};

/// Below this drawing width there is no room for the hints.
const MIN_WIDTH: f32 = 200.0;

// The original's help bar (uvs::OnScreenHelpBar, an overlay of its view): placed with the vector
// (18, 10) from the view's corner (0x100207d54), laid out with 14, 10, 44 and 18 (0x10020815c),
// read here as text size, padding, bar height and side padding.
const EDGE_RIGHT: f32 = 18.0;
const EDGE_BOTTOM: f32 = 10.0;
const TEXT_SIZE: f32 = 14.0;
const PAD_Y: f32 = 10.0;
const BAR_HEIGHT: f32 = 44.0;
const PAD_X: f32 = 18.0;
/// Space between two bars.
const GAP: f32 = 6.0;

#[derive(Default)]
pub struct Hints;

/// A rounded white bar floating over the drawing, as tall as the original's: one row of the
/// usual control height, with the rest of the bar height split above and below it (at least
/// the original's padding).
fn pill(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    let row = ui.spacing().interact_size.y;
    let pad_y = ((BAR_HEIGHT - row) / 2.0).max(PAD_Y);
    egui::Frame::none().fill(crate::theme::surface()).stroke(Stroke::new(1.0_f32, hairline())).rounding(6.0).inner_margin(egui::Margin::symmetric(PAD_X, pad_y)).show(ui, |ui| {
        ui.horizontal(add);
    });
}

fn text(s: impl Into<String>) -> egui::RichText {
    egui::RichText::new(s).size(TEXT_SIZE).color(ink())
}

impl Decoration for Hints {
    fn name(&self) -> &'static str {
        "Hinweise"
    }

    fn layer(&self) -> Layer {
        Layer::Over
    }

    fn interact(&mut self, view: &SheetView, ctx: &egui::Context) -> Option<SheetAction> {
        let r = view.canvas;
        if r.width() < MIN_WIDTH {
            return None;
        }
        // each bar floats on its own, stacked upwards from the corner, so only the bars
        // themselves catch the pointer (one area round all of them would block the drawing)
        let mut bottom = r.right_bottom() - Vec2::new(EDGE_RIGHT, EDGE_BOTTOM);
        let mut bar = |key: &str, add: &mut dyn FnMut(&mut egui::Ui)| {
            let shown = egui::Area::new(egui::Id::new(("sheet_hint", key))).order(egui::Order::Foreground).pivot(Align2::RIGHT_BOTTOM).fixed_pos(bottom).show(ctx, |ui| pill(ui, |ui| add(ui)));
            bottom.y = shown.response.rect.top() - GAP;
        };
        let mut action = None;
        bar("help", &mut |ui| {
            let label = egui::Label::new(text("Hilfe zum ausgewählten Werkzeug anzeigen  ?")).sense(Sense::click());
            if ui.add(label).clicked() {
                action = Some(SheetAction::ShowToolHelp);
            }
        });
        action
    }
}
