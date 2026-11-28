//! Trennen (trim): the vector clicked is cut at every place where it crosses another vector and
//! falls apart into separate open polylines. The other outlines of the same shape (the holes of a
//! compound shape) stay one shape. A circle or an arc stays round: it falls apart into arcs of the
//! same circle, each ending exactly at a crossing (see `split_arc_at_crossings`).

use super::super::App;
use shape_edit::geometry::{dist_to_segment, Contour};
use shape_edit::vector::trim::{crossings, split_arc_at_crossings, split_at_crossings, Cutters};
use shape_edit::{Arc, Compound, Item, Path, Pt, Shape};
use std::f64::consts::TAU;

impl App {
    /// The outline of shape `index` nearest to `p`, and what it may be cut by: the other shapes'
    /// outlines, the round ones (circles, arcs) as true arcs so the cuts on them are exact, plus the
    /// shape's own other outlines.
    fn trim_parts(&self, index: usize, p: Option<Pt>) -> Option<(Vec<Contour>, usize, Cutters)> {
        let own = self.shapes[index].shape.contours();
        if own.is_empty() {
            return None;
        }
        let near = |c: &Contour| c.segments().map(|(a, b)| dist_to_segment(p.unwrap_or(c.pts[0]), a, b)).fold(f64::MAX, f64::min);
        let k = (0..own.len()).min_by(|&a, &b| near(&own[a]).partial_cmp(&near(&own[b])).unwrap())?;
        let mut cutters = Cutters::default();
        for s in self.shapes.iter().enumerate().filter(|(j, s)| *j != index && !s.hidden).map(|(_, s)| s) {
            match round_of(&s.shape) {
                Some(arc) => cutters.rounds.push(arc),
                None => cutters.outlines.extend(s.shape.contours()),
            }
        }
        cutters.outlines.extend(own.iter().enumerate().filter(|(m, _)| *m != k).map(|(_, c)| c.clone()));
        Some((own, k, cutters))
    }

    /// Where the outline of vector `index` nearest to `p` would be cut (shown before the click).
    pub(super) fn trim_points(&self, index: usize, p: Pt) -> Vec<Pt> {
        let Some((own, k, cutters)) = self.trim_parts(index, Some(p)) else { return vec![] };
        if let Some(arc) = round_of(&self.shapes[index].shape) {
            // the exact crossings: where each arc piece will start
            return split_arc_at_crossings(&arc, &cutters).iter().map(|a| a.at(a.start)).collect();
        }
        crossings(&own[k], &cutters).into_iter().map(|s| point_along(&own[k], s)).collect()
    }

    /// Cut the vector under `p` at its crossings.
    pub(super) fn trim_at(&mut self, p: Pt) {
        let Some(index) = self.vector_at(p) else {
            self.status = t!("Hier ist kein Vektor").into();
            return;
        };
        let Some((own, k, cutters)) = self.trim_parts(index, Some(p)) else { return };
        // a circle or an arc falls apart into arcs; anything else into open polylines
        let pieces: Vec<Shape> = match round_of(&self.shapes[index].shape) {
            Some(arc) => split_arc_at_crossings(&arc, &cutters).into_iter().map(Shape::Arc).collect(),
            None => split_at_crossings(&own[k], &cutters).into_iter().map(|part| Shape::Path(Path::new(part.pts, false))).collect(),
        };
        if pieces.is_empty() {
            self.status = t!("Dieser Vektor kreuzt keinen anderen: nichts zu trennen").into();
            return;
        }
        self.push_undo();
        let original = self.shapes.remove(index);
        let make = |id: u64, shape: Shape| {
            let mut item = Item::new(id, shape);
            item.layer = original.layer;
            item.groups = original.groups.clone();
            item
        };
        let mut new_items = vec![];
        // the shape's other outlines stay together, under its id (a round shape has no others)
        let rest: Vec<Path> = own.iter().enumerate().filter(|(m, _)| *m != k).map(|(_, c)| Path::new(c.pts.clone(), c.closed)).collect();
        if !rest.is_empty() {
            let shape = if rest.len() == 1 { Shape::Path(rest.into_iter().next().unwrap()) } else { Shape::Compound(Compound::new(rest)) };
            new_items.push(make(original.id, shape));
        }
        let count = pieces.len();
        for piece in pieces {
            let id = if new_items.is_empty() { original.id } else { self.new_id() };
            new_items.push(make(id, piece));
        }
        // toolpaths that cut the vector cut all its pieces
        let ids: Vec<u64> = new_items.iter().map(|i| i.id).collect();
        for tp in &mut self.toolpaths {
            if tp.shape_ids.contains(&original.id) {
                tp.shape_ids.retain(|s| *s != original.id);
                tp.shape_ids.extend(&ids);
            }
        }
        for (n, item) in new_items.into_iter().enumerate() {
            self.shapes.insert(index + n, item);
        }
        self.edit.retain_existing(&self.shapes);
        self.status = t!("Vektor in %{n} Teile getrennt", n = count).into();
        self.recompute_all();
    }
}

/// A round shape as an arc — a circle is the arc once round from 0° — so trimming keeps it round
/// instead of turning it into a polyline, and cuts against it exactly; `None` for any other shape.
fn round_of(shape: &Shape) -> Option<Arc> {
    match shape {
        Shape::Circle(c) => Some(Arc::new(c.center, c.radius, 0.0, TAU)),
        Shape::Arc(a) => Some(*a),
        _ => None,
    }
}

/// The point at arc length `s` along `c`.
fn point_along(c: &Contour, s: f64) -> Pt {
    let mut left = s;
    for (a, b) in c.segments() {
        let l = shape_edit::geometry::dist(a, b);
        if left <= l {
            let f = if l > 0.0 { left / l } else { 0.0 };
            return [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f];
        }
        left -= l;
    }
    *c.pts.last().unwrap_or(&[0.0, 0.0])
}
