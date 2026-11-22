//! A light grid over the board: a line about every 30 screen pixels on a round step, every
//! fifth one a little darker.

use super::{Decoration, Layer, SheetView};
use egui::{Color32, Painter, Pos2, Stroke};

pub struct Grid;

impl Decoration for Grid {
    fn name(&self) -> &'static str {
        "Raster"
    }

    fn layer(&self) -> Layer {
        Layer::Under
    }

    fn paint(&self, view: &SheetView, painter: &Painter) {
        let board = view.board();
        // the board edge is always drawn; the grid lines only when snapping to the grid is on
        if !view.snap_grid {
            painter.rect_stroke(board, 0.0, Stroke::new(1.0_f32, Color32::from_gray(150)));
            return;
        }
        let step = SheetView::nice_step(30.0 / view.zoom as f64);
        let stroke = |k: usize| Stroke::new(1.0_f32, if k % 5 == 0 { Color32::from_gray(222) } else { Color32::from_gray(240) });
        for k in 0..=((view.width / step + 1e-6).floor() as usize) {
            let x = view.to_screen([k as f64 * step, 0.0]).x;
            painter.line_segment([Pos2::new(x, board.top()), Pos2::new(x, board.bottom())], stroke(k));
        }
        for k in 0..=((view.height / step + 1e-6).floor() as usize) {
            let y = view.to_screen([0.0, k as f64 * step]).y;
            painter.line_segment([Pos2::new(board.left(), y), Pos2::new(board.right(), y)], stroke(k));
        }
        // the board's edge over the grid lines
        painter.rect_stroke(board, 0.0, Stroke::new(1.0_f32, Color32::from_gray(150)));
    }
}
