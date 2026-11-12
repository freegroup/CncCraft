//! The board: the grey desk around it, its soft shadow, the white material and its edge.

use super::{Decoration, Layer, SheetView};
use egui::{Color32, Painter, Stroke, Vec2};

pub struct Board;

impl Decoration for Board {
    fn name(&self) -> &'static str {
        "Platte"
    }

    fn layer(&self) -> Layer {
        Layer::Under
    }

    fn paint(&self, view: &SheetView, painter: &Painter) {
        painter.rect_filled(view.canvas, 0.0, if crate::theme::is_dark() { Color32::from_rgb(0x2d, 0x2f, 0x35) } else { Color32::from_rgb(0xd9, 0xdc, 0xe0) });
        let board = view.board();
        painter.rect_filled(board.translate(Vec2::new(3.0, 3.0)), 0.0, Color32::from_black_alpha(30));
        painter.rect_filled(board, 0.0, Color32::WHITE);
        painter.rect_stroke(board, 0.0, Stroke::new(1.0_f32, Color32::from_gray(150)));
    }
}
