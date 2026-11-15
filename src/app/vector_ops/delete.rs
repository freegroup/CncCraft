//! Löschen: the segment clicked is removed. A closed shape becomes an open path, an open path
//! splits at the gap (middle → two paths, end → shorter path). A non-path shape is first
//! converted to curves.

use super::super::App;
use shape_edit::{Path, Pt, Shape};

impl App {
    /// Delete the segment nearest to `p`.
    pub(super) fn delete_segment_at(&mut self, p: Pt) {
        let Some(hit) = self.segment_at(p) else {
            self.status = t!("Hier ist kein Segment").into();
            return;
        };
        let item = &self.shapes[hit.shape];
        let contours = item.shape.contours();
        let Some(c) = contours.first() else {
            self.status = t!("Hier ist kein Segment").into();
            return;
        };
        let pts = c.pts.clone();
        let closed = c.closed;
        let n = pts.len();
        if n < 2 {
            return;
        }

        self.push_undo();

        if closed {
            if n <= 3 {
                self.shapes.remove(hit.shape);
                self.edit.selection.clear();
                self.edit.retain_existing(&self.shapes);
                self.status = t!("Segment gelöscht, Form entfernt").into();
                return;
            }
            let start = (hit.seg + 1) % n;
            let new_pts: Vec<Pt> = (0..n).map(|i| pts[(start + i) % n]).collect();
            self.shapes[hit.shape].shape = Shape::Path(Path::new(new_pts, false));
        } else {
            let segs = n - 1;
            if segs <= 1 {
                self.shapes.remove(hit.shape);
                self.edit.selection.clear();
                self.edit.retain_existing(&self.shapes);
                self.status = t!("Segment gelöscht").into();
                return;
            }
            if hit.seg == 0 {
                self.shapes[hit.shape].shape = Shape::Path(Path::new(pts[1..].to_vec(), false));
            } else if hit.seg == segs - 1 {
                self.shapes[hit.shape].shape = Shape::Path(Path::new(pts[..n - 1].to_vec(), false));
            } else {
                let left = pts[..=hit.seg].to_vec();
                let right = pts[hit.seg + 1..].to_vec();
                self.shapes[hit.shape].shape = Shape::Path(Path::new(left, false));
                let _id = self.add_shape(Path::new(right, false));
            }
        }

        if let Shape::Path(p) = &mut self.shapes[hit.shape].shape {
            p.corners.clear();
        }

        self.recompute_all();
        self.status = t!("Segment gelöscht").into();
    }
}
