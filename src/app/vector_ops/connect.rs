//! Verbinden (connect): two points clicked one after the other are joined by a straight line.
//! Where the first is the end of an open polyline, the line becomes part of it; otherwise where
//! the second is; between the two ends of one polyline it closes it; else it is a line of its own.
//! The points snap like when drawing (to nodes, centres, the grid).

use super::super::App;
use shape_edit::vector::connect::{end_at, extend};
use shape_edit::{Path, Pt, Shape};

impl App {
    /// The open polyline with an end at `p`: its index and which end.
    fn open_end_at(&self, p: Pt) -> Option<(usize, shape_edit::vector::connect::End)> {
        let tol = self.pick_tolerance() / 4.0;
        self.shapes.iter().enumerate().rev().filter(|(_, s)| !s.hidden && !s.frozen).find_map(|(i, s)| match &s.shape {
            Shape::Path(path) => end_at(path, p, tol).map(|e| (i, e)),
            _ => None,
        })
    }

    /// A click with the connect tool: the first point, then the second, which makes the line.
    pub(super) fn connect_click(&mut self, m: Pt) {
        let p = self.snap(m);
        let Some(from) = self.vops.connect_from else {
            self.vops.connect_from = Some(p);
            self.status = t!("Ersten Punkt gewählt – jetzt den zweiten anklicken (Esc bricht ab)").into();
            return;
        };
        if shape_edit::geometry::dist(from, p) < 1e-6 {
            return;
        }
        self.vops.connect_from = None;
        let tol = self.pick_tolerance() / 4.0;
        self.push_undo();
        match (self.open_end_at(from), self.open_end_at(p)) {
            (Some((i, end)), _) => {
                let Shape::Path(path) = &self.shapes[i].shape else { return };
                let grown = extend(path, end, p, tol);
                let closed = grown.closed;
                self.shapes[i].shape = Shape::Path(grown);
                self.status = if closed { t!("Linienzug geschlossen") } else { t!("Linienzug verlängert") }.into();
            }
            (None, Some((i, end))) => {
                let Shape::Path(path) = &self.shapes[i].shape else { return };
                self.shapes[i].shape = Shape::Path(extend(path, end, from, tol));
                self.status = t!("Linienzug verlängert").into();
            }
            (None, None) => {
                self.add_shape(Path::new(vec![from, p], false));
                self.status = t!("Linie erstellt").into();
            }
        }
        self.recompute_all();
    }
}
