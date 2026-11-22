//! Rulers along the board's top and left edges (not the window's), with numbered marks and
//! small ones between, as the original draws them; the pointer shows as a mark on each.

use super::{Decoration, Layer, SheetView};
use crate::theme::{RULER_FILL, RULER_INK};
use egui::{Align2, FontId, Painter, Pos2, Rect, Stroke};

/// Thickness of a ruler in screen pixels.
pub const RULER: f32 = 20.0;

pub struct Rulers;

impl Decoration for Rulers {
    fn name(&self) -> &'static str {
        "Lineale"
    }

    fn layer(&self) -> Layer {
        Layer::Over
    }

    fn paint(&self, view: &SheetView, painter: &Painter) {
        let board = view.board();
        let top = Rect::from_min_max(Pos2::new(board.left(), board.top() - RULER), Pos2::new(board.right(), board.top()));
        let left = Rect::from_min_max(Pos2::new(board.left() - RULER, board.top()), Pos2::new(board.left(), board.bottom()));
        // the corner where they meet belongs to the rulers too (it holds the guides' lock)
        let corner = Rect::from_min_max(Pos2::new(board.left() - RULER, board.top() - RULER), board.left_top());
        for r in [top, left, corner] {
            painter.rect_filled(r, 0.0, RULER_FILL);
        }
        let line = Stroke::new(1.0_f32, RULER_INK);
        painter.line_segment([top.left_bottom(), top.right_bottom()], line);
        painter.line_segment([left.right_top(), left.right_bottom()], line);
        let step = SheetView::nice_step(70.0 / view.zoom as f64);
        let minor = step / 5.0;
        let ticks = |len: f64| (0..=((len / minor).floor() as i64)).map(move |k| (k, k as f64 * minor));
        for (k, x) in ticks(view.width) {
            let sx = view.to_screen([x, 0.0]).x;
            let major = k % 5 == 0;
            painter.line_segment([Pos2::new(sx, top.bottom() - if major { 8.0 } else { 3.0 }), Pos2::new(sx, top.bottom())], line);
            if major {
                painter.text(Pos2::new(sx, top.top() + 1.0), Align2::CENTER_TOP, format!("{x:.0}"), FontId::proportional(10.0), RULER_INK);
            }
        }
        for (k, y) in ticks(view.height) {
            let sy = view.to_screen([0.0, y]).y;
            let major = k % 5 == 0;
            painter.line_segment([Pos2::new(left.right() - if major { 8.0 } else { 3.0 }, sy), Pos2::new(left.right(), sy)], line);
            if major {
                // numbers read upwards along the left ruler, as in the original
                let galley = painter.layout_no_wrap(format!("{y:.0}"), FontId::proportional(10.0), RULER_INK);
                let at = Pos2::new(left.left() + 1.0, sy + galley.size().x / 2.0);
                painter.add(egui::epaint::TextShape::new(at, galley, RULER_INK).with_angle(-std::f32::consts::FRAC_PI_2));
            }
        }
        if let Some(m) = view.pointer {
            let p = view.to_screen(m);
            let mark = Stroke::new(1.5_f32, RULER_INK);
            if p.x >= top.left() && p.x <= top.right() {
                painter.line_segment([Pos2::new(p.x, top.top() + 4.0), Pos2::new(p.x, top.bottom())], mark);
            }
            if p.y >= left.top() && p.y <= left.bottom() {
                painter.line_segment([Pos2::new(left.left() + 4.0, p.y), Pos2::new(left.right(), p.y)], mark);
            }
        }
    }
}
