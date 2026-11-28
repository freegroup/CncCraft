//! Trennen (trim): the vector clicked is cut at every place where it crosses another vector. Each
//! shape class decides how it falls apart (`shape_edit::Form::trim`): a line/polyline into open
//! polylines, a circle or arc into arcs of the same circle, a spline into exact sub-splines. Only a
//! compound shape (holes) is split here, outline by outline, keeping the other outlines together.

use super::super::App;
use shape_edit::geometry::{dist_to_segment, Contour};
use shape_edit::vector::trim::{split_at_crossings, Cutters};
use shape_edit::{Arc, Compound, Item, Path, Pt, Shape};
use std::f64::consts::TAU;

impl App {
    /// What shape `index` may be cut by: the other shapes' outlines, the round ones (circles, arcs)
    /// kept apart as true arcs so the cuts on them are exact, plus this shape's own other outlines.
    fn trim_cutters(&self, index: usize, skip_own: Option<usize>) -> Cutters {
        let own = self.shapes[index].shape.contours();
        let mut cutters = Cutters::default();
        for s in self.shapes.iter().enumerate().filter(|(j, s)| *j != index && !s.hidden).map(|(_, s)| s) {
            match round_of(&s.shape) {
                Some(arc) => cutters.rounds.push(arc),
                None => cutters.outlines.extend(s.shape.contours()),
            }
        }
        if let Some(k) = skip_own {
            cutters.outlines.extend(own.iter().enumerate().filter(|(m, _)| *m != k).map(|(_, c)| c.clone()));
        }
        cutters
    }

    /// The outline of a compound shape `index` nearest to `p`, and its index.
    fn nearest_outline(&self, index: usize, p: Pt) -> Option<(Vec<Contour>, usize)> {
        let own = self.shapes[index].shape.contours();
        if own.is_empty() {
            return None;
        }
        let near = |c: &Contour| c.segments().map(|(a, b)| dist_to_segment(p, a, b)).fold(f64::MAX, f64::min);
        let k = (0..own.len()).min_by(|&a, &b| near(&own[a]).partial_cmp(&near(&own[b])).unwrap())?;
        Some((own, k))
    }

    /// Where the vector `index` nearest to `p` would be cut (shown before the click): the start of
    /// each piece the shape would fall into.
    pub(super) fn trim_points(&self, index: usize, p: Pt) -> Vec<Pt> {
        let cutters = self.trim_cutters(index, None);
        if let Some(pieces) = self.shapes[index].shape.trim(&cutters) {
            return pieces.iter().filter_map(|s| s.contours().into_iter().next()).filter_map(|c| c.pts.first().copied()).collect();
        }
        // a compound shape: only the outline nearest the pointer is cut
        let Some((own, k)) = self.nearest_outline(index, p) else { return vec![] };
        let cutters = self.trim_cutters(index, Some(k));
        split_at_crossings(&own[k], &cutters).into_iter().filter_map(|c| c.pts.first().copied()).collect()
    }

    /// Cut the vector under `p` at its crossings — each shape class into its own kind of pieces.
    pub(super) fn trim_at(&mut self, p: Pt) {
        let Some(index) = self.vector_at(p) else {
            self.status = t!("Hier ist kein Vektor").into();
            return;
        };
        // the shape decides how it falls apart; a compound shape is split outline by outline here,
        // its other outlines staying together under its id
        let cutters = self.trim_cutters(index, None);
        let (pieces, rest): (Vec<Shape>, Vec<Path>) = match self.shapes[index].shape.trim(&cutters) {
            Some(pieces) => (pieces, vec![]),
            None => {
                let Some((own, k)) = self.nearest_outline(index, p) else { return };
                let cutters = self.trim_cutters(index, Some(k));
                let pieces = split_at_crossings(&own[k], &cutters).into_iter().map(|c| Shape::Path(Path::new(c.pts, false))).collect();
                let rest = own.iter().enumerate().filter(|(m, _)| *m != k).map(|(_, c)| Path::new(c.pts.clone(), c.closed)).collect();
                (pieces, rest)
            }
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
        // the compound shape's other outlines stay together, under its id
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
