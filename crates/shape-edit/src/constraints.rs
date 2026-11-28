//! Distance constraints between two things on the sheet, as set with the dimension mode ("D"):
//! point to point (measured horizontally, vertically or straight), point to line, and line to line
//! (two parallel lines, measured as their perpendicular gap — the constraint drops when they stop
//! being parallel).
//!
//! A constraint is not stored in a shape but in the document's list and points at its two ends
//! by shape id and index ([`Anchor`]). The ends are picked on the shapes' **corners and centre**
//! (box corners 0–3 lower-left, lower-right, upper-right, upper-left; 4 the centre; 5.. the
//! nodes of a vector) and on their **lines** (the four box sides 0–3 — bottom, right, top, left —
//! and, for a vector, each of its segments from 4 on).
//!
//! Solving is direct, not an equation system: the shape of the *second* end is the one that
//! moves (translated) until the distance holds; the first end is the reference. Two ends on the
//! same shape can only be opposite corners, which is the shape's own width or height. This is
//! step 1 of app/docs/PARAMETRIK.md "Stufe 2"; a real solver would come with later kinds.

use crate::{Editable, Item, Pt};
use serde::{Deserialize, Serialize};

/// The id the job's material plate has among the items a constraint looks at. The plate takes
/// part like a rectangle (its corners, centre and four sides can be picked) but never moves.
pub const PLATE: u64 = u64::MAX;

/// The material plate (0/0 to `width`/`height`) as an item, to be added to the shapes when
/// constraints are picked, measured or solved.
pub fn plate_item(width: f64, height: f64) -> Item {
    Item::new(PLATE, crate::Rect::new([width / 2.0, height / 2.0], width, height))
}

/// How much of a constraint is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstraintView {
    /// Nothing.
    Hidden,
    /// Only the line, faint, without the value.
    Slim,
    /// The whole constraint with its value.
    Full,
}

/// What every kind of constraint tells: the shapes it hangs on. From that, and from whether the
/// switch for showing all constraints is on, the constraint decides by itself how much of it is
/// drawn: on a selected shape always everything; on the others nothing with the switch off and
/// the faint line with it on.
pub trait Constrains {
    fn shapes(&self) -> Vec<u64>;

    fn view(&self, show_all: bool, selection: &[u64]) -> ConstraintView {
        if self.shapes().iter().any(|s| selection.contains(s)) {
            ConstraintView::Full
        } else if show_all {
            ConstraintView::Slim
        } else {
            ConstraintView::Hidden
        }
    }
}

impl Constrains for Distance {
    fn shapes(&self) -> Vec<u64> {
        vec![self.from.shape(), self.to.shape()]
    }
}

impl Constrains for Align {
    fn shapes(&self) -> Vec<u64> {
        vec![self.a.shape(), self.b.shape()]
    }
}

impl Constrains for OnLine {
    fn shapes(&self) -> Vec<u64> {
        vec![self.point.shape(), self.line.shape()]
    }
}

/// A size of one shape that was set to a number or a formula (a dimension tied to a variable).
pub struct SizeConstraint {
    pub shape: u64,
}

impl Constrains for SizeConstraint {
    fn shapes(&self) -> Vec<u64> {
        vec![self.shape]
    }
}

/// How a point-to-point distance is measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    /// Straight, along the line between the points.
    Aligned,
    Horizontal,
    Vertical,
}

/// One end of a distance: a point or a line on a shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Anchor {
    Point(u64, usize),
    Edge(u64, usize),
}

impl Anchor {
    pub fn shape(&self) -> u64 {
        match *self {
            Anchor::Point(id, _) | Anchor::Edge(id, _) => id,
        }
    }
}

/// A distance that must hold: `from` is the reference, the shape of `to` moves to keep it.
/// The value is text (a number, a formula or variables) and is worked out by the caller.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Distance {
    pub from: Anchor,
    pub to: Anchor,
    pub axis: Axis,
    pub expr: String,
    /// Where the dimension line was put when it was made (world coordinates). Only this first
    /// place is absolute; once the constraint has been measured it is kept as an `offset` from
    /// what it measures, so the dimension follows the shapes (see [`Distance::place_world`]).
    pub place: Pt,
    /// The dimension line's place relative to the middle of the two measured ends. `None` for a
    /// constraint that hasn't been measured yet (and in files from before this was kept so).
    #[serde(default)]
    pub offset: Option<Pt>,
}

/// A distance worked out on the current shapes.
#[derive(Clone, Copy, Debug)]
pub struct Measured {
    /// Where the two ends are (for a line: the foot of the perpendicular from the other end).
    pub a: Pt,
    pub b: Pt,
    /// The direction the distance is measured in (unit), and the signed distance along it.
    pub dir: Pt,
    pub d: f64,
}

/// The point `k` of an item (see the module text), if it has one.
pub fn point_of(item: &Item, k: usize) -> Option<Pt> {
    let (lo, hi) = item.bounds()?;
    match k {
        0 => Some(lo),
        1 => Some([hi[0], lo[1]]),
        2 => Some(hi),
        3 => Some([lo[0], hi[1]]),
        4 => Some([(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0]),
        _ => item.nodes()?.get(k - 5).copied(),
    }
}

/// How many points an item offers.
pub fn point_count(item: &Item) -> usize {
    if item.bounds().is_none() {
        return 0;
    }
    5 + item.nodes().map_or(0, |n| n.len())
}

/// The line `k` of an item as two points: the four box sides, then a vector's segments.
pub fn edge_of(item: &Item, k: usize) -> Option<(Pt, Pt)> {
    if k < 4 {
        let corners = [point_of(item, 0)?, point_of(item, 1)?, point_of(item, 2)?, point_of(item, 3)?];
        return Some((corners[k], corners[(k + 1) % 4]));
    }
    let nodes = item.nodes()?;
    let i = k - 4;
    let closed = item.nodes_closed();
    let n = nodes.len();
    if i + 1 < n {
        Some((nodes[i], nodes[i + 1]))
    } else if i + 1 == n && closed && n > 2 {
        Some((nodes[i], nodes[0]))
    } else {
        None
    }
}

/// How many lines an item offers.
pub fn edge_count(item: &Item) -> usize {
    if item.bounds().is_none() {
        return 0;
    }
    4 + item.nodes().map_or(0, |n| if item.nodes_closed() && n.len() > 2 { n.len() } else { n.len().saturating_sub(1) })
}

/// A line's own length as a distance between its two end points (on the same shape): the box
/// sides are the shape's width and height, a vector's segment goes from node to node.
pub fn length_of(items: &[Item], edge: Anchor, place: Pt) -> Option<Distance> {
    let Anchor::Edge(id, k) = edge else { return None };
    let it = find(items, id)?;
    let (p0, p1, axis) = if k < 4 {
        (k, (k + 1) % 4, if k % 2 == 0 { Axis::Horizontal } else { Axis::Vertical })
    } else {
        let n = it.nodes()?.len();
        let i = k - 4;
        (5 + i, 5 + (i + 1) % n, Axis::Aligned)
    };
    Some(Distance { from: Anchor::Point(id, p0), to: Anchor::Point(id, p1), axis, expr: String::new(), place, offset: None })
}

/// The point or line under `p` (world coordinates, `tol` world units): the nearest point first,
/// else the nearest line.
pub fn pick(items: &[Item], p: Pt, tol: f64) -> Option<Anchor> {
    let mut best: Option<(f64, Anchor)> = None;
    // shapes on a hidden or locked layer take no part
    let items: Vec<&Item> = items.iter().filter(|i| !i.hidden && !i.frozen).collect();
    for it in items.iter().copied() {
        for k in 0..point_count(it) {
            if let Some(q) = point_of(it, k) {
                let d = dist(p, q);
                if d <= tol && best.map_or(true, |(b, _)| d < b) {
                    best = Some((d, Anchor::Point(it.id, k)));
                }
            }
        }
    }
    if best.is_some() {
        return best.map(|(_, a)| a);
    }
    for it in items.iter().copied() {
        for k in 0..edge_count(it) {
            if let Some((a, b)) = edge_of(it, k) {
                let d = crate::geometry::dist_to_segment(p, a, b);
                if d <= tol && best.map_or(true, |(bd, _)| d < bd) {
                    best = Some((d, Anchor::Edge(it.id, k)));
                }
            }
        }
    }
    best.map(|(_, a)| a)
}

fn dist(a: Pt, b: Pt) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

fn unit(v: Pt) -> Option<Pt> {
    let l = (v[0] * v[0] + v[1] * v[1]).sqrt();
    (l > 1e-12).then(|| [v[0] / l, v[1] / l])
}

fn dot(a: Pt, b: Pt) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

/// The axis a point-to-point distance would get when the dimension is placed at `at`: placed
/// beside the two points (outside the box they span, along one side) it is measured across that
/// way, placed between them or diagonally it is straight. Horizontal when `at` is above or below
/// the span of the points' x, vertical when it is to the left or right of their y.
pub fn axis_for_placement(a: Pt, b: Pt, at: Pt) -> Axis {
    let (x0, x1) = (a[0].min(b[0]), a[0].max(b[0]));
    let (y0, y1) = (a[1].min(b[1]), a[1].max(b[1]));
    let in_x = at[0] >= x0 && at[0] <= x1;
    let in_y = at[1] >= y0 && at[1] <= y1;
    match (in_x, in_y) {
        (true, false) => Axis::Horizontal,
        (false, true) => Axis::Vertical,
        _ => Axis::Aligned,
    }
}

fn find(items: &[Item], id: u64) -> Option<&Item> {
    items.iter().find(|i| i.id == id)
}

impl Distance {
    /// The middle of the two measured ends: what the dimension's place is relative to.
    fn middle(m: &Measured) -> Pt {
        [(m.a[0] + m.b[0]) / 2.0, (m.a[1] + m.b[1]) / 2.0]
    }

    /// Where the dimension line is now, in world coordinates: the offset from the middle of what it
    /// measures (so it moves with the shapes), or, before it was measured, the place it was put.
    pub fn place_world(&self, m: &Measured) -> Pt {
        match self.offset {
            Some(o) => {
                let mid = Self::middle(m);
                [mid[0] + o[0], mid[1] + o[1]]
            }
            None => self.place,
        }
    }

    /// Put the dimension line at `p` (world coordinates), kept relative to what it measures.
    pub fn set_place_world(&mut self, m: &Measured, p: Pt) {
        let mid = Self::middle(m);
        self.place = p;
        self.offset = Some([p[0] - mid[0], p[1] - mid[1]]);
    }

    /// Both ends exist and the pair makes sense, or why it doesn't (for a message).
    pub fn check(&self, items: &[Item]) -> Result<(), String> {
        let (fa, fb) = (find(items, self.from.shape()), find(items, self.to.shape()));
        let (Some(_), Some(_)) = (fa, fb) else { return Err("Eine der Formen gibt es nicht mehr".into()) };
        if self.from == self.to {
            return Err("Zwei verschiedene Punkte wählen".into());
        }
        if self.to.shape() == PLATE {
            return Err("Die Platte bleibt, wo sie ist: zuerst die Form, dann die Platte wählen".into());
        }
        if self.from.shape() == PLATE && self.to.shape() == PLATE {
            return Err("Beide Enden liegen auf der Platte".into());
        }
        if let (Anchor::Edge(_, k), Anchor::Edge(_, l)) = (self.from, self.to) {
            // two lines may be dimensioned only while they are parallel: the gap between them is
            // the perpendicular distance. Non-parallel lines have no single distance.
            let (ia, ib) = (find(items, self.from.shape()), find(items, self.to.shape()));
            let edges = ia.and_then(|a| edge_of(a, k)).zip(ib.and_then(|b| edge_of(b, l)));
            match edges {
                Some(((a0, a1), (b0, b1))) if edges_parallel(a0, a1, b0, b1) => {}
                Some(_) => return Err("Die beiden Linien sind nicht parallel: nur parallele Linien lassen sich so bemaßen".into()),
                None => return Err("Eine der Linien gibt es nicht mehr".into()),
            }
        }
        if self.from.shape() == self.to.shape() {
            // the corners and sides of a rectangle or ellipse are those of its box: when it is
            // turned another way than by quarters they say nothing about its own size
            if let Some(it) = find(items, self.from.shape()) {
                if matches!(it.shape, crate::Shape::Rect(_) | crate::Shape::Ellipse(_)) && along_axes(it).is_none() {
                    return Err("Die Form ist schräg gedreht: ihre Maße gelten erst wieder, wenn sie gerade steht".into());
                }
            }
            // two things on one shape: the shape itself is scaled until the distance holds
            let m = self.measure(items).ok_or("Eine der Formen gibt es nicht mehr")?;
            if m.d.abs() < 1e-9 {
                return Err("Diese beiden Punkte haben in dieser Richtung keinen Abstand: so lässt sich die Form nicht skalieren".into());
            }
            if self.scale_factors(&m).is_none() {
                return Err("Auf derselben Form geht das nur an waagrechten oder senkrechten Linien".into());
            }
        }
        Ok(())
    }

    /// How a shape holding both ends is stretched to change the distance: `(1, 1)` for a straight
    /// point-to-point distance (scaled evenly), `(1, 0)` / `(0, 1)` when only x or only y changes,
    /// `None` when the direction is neither an axis nor a straight point-to-point direction.
    fn scale_factors(&self, m: &Measured) -> Option<(f64, f64)> {
        let aligned = matches!((self.from, self.to), (Anchor::Point(..), Anchor::Point(..))) && self.axis == Axis::Aligned;
        if aligned {
            return Some((1.0, 1.0));
        }
        if m.dir[0].abs() > 0.999_999 {
            Some((1.0, 0.0))
        } else if m.dir[1].abs() > 0.999_999 {
            Some((0.0, 1.0))
        } else {
            None
        }
    }

    /// For a distance between two nodes of the same vector (indices 5 and up): the second node's
    /// index in the node list, where it is, the measuring direction and the signed distance now.
    /// `None` for anything else (box corners, two shapes, a shape without nodes).
    fn node_end(&self, items: &[Item]) -> Option<(usize, Pt, Pt, f64)> {
        let (Anchor::Point(ia, i), Anchor::Point(ib, j)) = (self.from, self.to) else { return None };
        if ia != ib || i < 5 || j < 5 {
            return None;
        }
        find(items, ia)?.nodes()?;
        let m = self.measure(items)?;
        Some((j - 5, m.b, m.dir, m.d))
    }

    /// The distance as it is now, or `None` when an end is missing.
    pub fn measure(&self, items: &[Item]) -> Option<Measured> {
        let ia = find(items, self.from.shape())?;
        let ib = find(items, self.to.shape())?;
        match (self.from, self.to) {
            (Anchor::Point(_, i), Anchor::Point(_, j)) => {
                let (a, b) = (point_of(ia, i)?, point_of(ib, j)?);
                let v = [b[0] - a[0], b[1] - a[1]];
                let (dir, d) = match self.axis {
                    Axis::Horizontal => ([1.0, 0.0], v[0]),
                    Axis::Vertical => ([0.0, 1.0], v[1]),
                    Axis::Aligned => match unit(v) {
                        Some(u) => (u, dot(v, u)),
                        None => ([1.0, 0.0], 0.0),
                    },
                };
                Some(Measured { a, b, dir, d })
            }
            (Anchor::Point(_, i), Anchor::Edge(_, k)) => {
                let p = point_of(ia, i)?;
                let (e0, e1) = edge_of(ib, k)?;
                let n = normal(e0, e1)?;
                let d = dot([e0[0] - p[0], e0[1] - p[1]], n);
                Some(Measured { a: p, b: [p[0] + n[0] * d, p[1] + n[1] * d], dir: n, d })
            }
            (Anchor::Edge(_, k), Anchor::Point(_, j)) => {
                let q = point_of(ib, j)?;
                let (e0, e1) = edge_of(ia, k)?;
                let n = normal(e0, e1)?;
                let d = dot([q[0] - e0[0], q[1] - e0[1]], n);
                Some(Measured { a: [q[0] - n[0] * d, q[1] - n[1] * d], b: q, dir: n, d })
            }
            (Anchor::Edge(_, k), Anchor::Edge(_, l)) => {
                // two parallel lines: the perpendicular gap between them. Measured only while the
                // lines are parallel; once they are not, there is no measurement (and the host drops
                // the constraint). `a` is the foot on the first line below the second line's start.
                let (a0, a1) = edge_of(ia, k)?;
                let (b0, b1) = edge_of(ib, l)?;
                if !edges_parallel(a0, a1, b0, b1) {
                    return None;
                }
                let n = normal(a0, a1)?;
                let d = dot([b0[0] - a0[0], b0[1] - a0[1]], n);
                Some(Measured { a: [b0[0] - n[0] * d, b0[1] - n[1] * d], b: b0, dir: n, d })
            }
        }
    }

    /// Make the distance `value` (above zero): the shape of the second end moves along the
    /// measuring direction, or, on one shape, its width or height is set. Returns whether
    /// anything changed.
    pub fn apply(&self, items: &mut [Item], value: f64) -> bool {
        if self.check(items).is_err() || !(value > 0.0) {
            return false;
        }
        if self.from.shape() == self.to.shape() {
            // two nodes of one vector (a segment's length, or any two of its points): only the
            // second node moves, along the measuring direction — the rest of the vector stays. (Scaling
            // the whole vector instead made two lengths on one polyline fight each other: each fix
            // rescaled everything about another point, so the shape crept away frame after frame.)
            if let Some((index, b, dir, d)) = self.node_end(items) {
                let target = if d < 0.0 { -value } else { value };
                let shift = target - d;
                if shift.abs() < 1e-9 {
                    return false;
                }
                let Some(item) = items.iter_mut().find(|it| it.id == self.to.shape()) else { return false };
                *item = item.with_node(index, [b[0] + dir[0] * shift, b[1] + dir[1] * shift]);
                return true;
            }
            // anything else on one shape (its box corners: width, height) scales the shape about the
            // first end (it stays), along the direction of the distance
            let Some(m) = self.measure(items) else { return false };
            let Some((ax, ay)) = self.scale_factors(&m) else { return false };
            let s = value / m.d.abs();
            if (s - 1.0).abs() < 1e-9 {
                return false;
            }
            let (sx, sy) = if ax == 1.0 && ay == 1.0 { (s, s) } else { (1.0 + (s - 1.0) * ax, 1.0 + (s - 1.0) * ay) };
            let Some(item) = items.iter_mut().find(|it| it.id == self.from.shape()) else { return false };
            *item = item.scaled(m.a, sx, sy);
            return true;
        }
        let Some(m) = self.measure(items) else { return false };
        // keep the side the second end is on
        let target = if m.d < 0.0 { -value } else { value };
        let shift = target - m.d;
        if shift.abs() < 1e-9 {
            return false;
        }
        let Some(item) = items.iter_mut().find(|it| it.id == self.to.shape()) else { return false };
        *item = item.translated(m.dir[0] * shift, m.dir[1] * shift);
        true
    }
}

/// The unit normal of a segment (its direction turned a quarter, counter-clockwise).
fn normal(a: Pt, b: Pt) -> Option<Pt> {
    let u = unit([b[0] - a[0], b[1] - a[1]])?;
    Some([-u[1], u[0]])
}

/// How far from parallel two lines may be and still count as parallel: the cross product of their
/// unit directions (|sin θ|) must stay under this. ~0.6° — tight enough that a visibly tilted line
/// breaks it, loose enough to survive floating-point wobble.
pub const PARALLEL_TOL: f64 = 0.01;

/// Whether the segment a0→a1 is parallel to b0→b1 (within [`PARALLEL_TOL`]).
pub fn edges_parallel(a0: Pt, a1: Pt, b0: Pt, b1: Pt) -> bool {
    match (unit([a1[0] - a0[0], a1[1] - a0[1]]), unit([b1[0] - b0[0], b1[1] - b0[1]])) {
        (Some(u), Some(v)) => (u[0] * v[1] - u[1] * v[0]).abs() <= PARALLEL_TOL,
        _ => false,
    }
}

/// Whether a shape's own width and height lie along the sheet's axes (a rectangle or ellipse turned
/// by a multiple of 90°): `Some(true)` when it is turned a quarter (its width then runs up).
fn along_axes(item: &Item) -> Option<bool> {
    let rotation = match &item.shape {
        crate::Shape::Rect(r) => r.rotation,
        crate::Shape::Ellipse(e) => e.rotation,
        _ => return None,
    };
    let q = rotation / 90.0;
    ((q - q.round()).abs() < 1e-9).then(|| (q.round() as i64).rem_euclid(2) == 1)
}

/// How to constrain one of a shape's own sizes (its width or height) as a distance on the shape:
/// the two corners and the way of measuring. A typed size is exactly this: a shorthand for the same
/// constraint made with the dimension mode. `None` for shapes whose sizes aren't along the sheet's
/// axes (they keep the older size binding, see [`Item::bound`]).
pub fn length_pair(item: &Item, dim: crate::Dim) -> Option<(usize, usize, Axis)> {
    use crate::Dim;
    let quarter = along_axes(item)?;
    match (dim, quarter) {
        (Dim::Width, false) | (Dim::Height, true) => Some((0, 1, Axis::Horizontal)),
        (Dim::Height, false) | (Dim::Width, true) => Some((0, 3, Axis::Vertical)),
        _ => None,
    }
}

/// Which of the shape's own sizes a distance on the shape sets, if it sets one: two corners on
/// opposite sides, measured across.
pub fn size_dim(item: &Item, c: &Distance) -> Option<crate::Dim> {
    use crate::Dim;
    if c.from.shape() != item.id || c.to.shape() != item.id {
        return None;
    }
    let quarter = along_axes(item)?;
    let (Anchor::Point(_, i), Anchor::Point(_, j)) = (c.from, c.to) else { return None };
    if i > 3 || j > 3 {
        return None;
    }
    let right = |k: usize| matches!(k, 1 | 2);
    let top = |k: usize| matches!(k, 2 | 3);
    match c.axis {
        Axis::Horizontal if right(i) != right(j) => Some(if quarter { Dim::Height } else { Dim::Width }),
        Axis::Vertical if top(i) != top(j) => Some(if quarter { Dim::Width } else { Dim::Height }),
        _ => None,
    }
}

/// Which sides' sizes the distances on a shape hold: `(across x, across y)` on the sheet, for the
/// editor to leave those handles alone.
pub fn locked_sides(item: &Item, list: &[Distance]) -> (bool, bool) {
    use crate::Dim;
    let quarter = along_axes(item).unwrap_or(false);
    let mut locked = (false, false);
    for c in list {
        match size_dim(item, c) {
            Some(Dim::Width) if !quarter => locked.0 = true,
            Some(Dim::Height) if !quarter => locked.1 = true,
            Some(Dim::Width) | Some(Dim::Height) => match c.axis {
                Axis::Horizontal => locked.0 = true,
                _ => locked.1 = true,
            },
            _ => {}
        }
    }
    locked
}

/// The lines of an item whose length is fixed by the person: a size tied to a variable
/// ([`Item::bound`]) and distances on the shape itself (its own width or height, a line's length).
/// For a rectangle (turned by a multiple of 90°) these are its box sides, for a vector the
/// segments between nodes. Returned as world-coordinate segments, without repeats.
pub fn fixed_edges(item: &Item, list: &[Distance]) -> Vec<(Pt, Pt)> {
    use crate::{Dim, Shape};
    let mut sides = [false; 4]; // bottom, right, top, left
    let mut segments: Vec<usize> = Vec::new();
    let turned = match &item.shape {
        Shape::Rect(r) if (r.rotation / 90.0).fract().abs() < 1e-9 => Some(((r.rotation / 90.0).round() as i64).rem_euclid(2) == 1),
        _ => None,
    };
    if let Some(quarter) = turned {
        for (dim, _) in &item.bound {
            // a turned quarter swaps the shape's width and height on the sheet
            match (dim, quarter) {
                (Dim::Width, false) | (Dim::Height, true) => {
                    sides[0] = true;
                    sides[2] = true;
                }
                (Dim::Height, false) | (Dim::Width, true) => {
                    sides[1] = true;
                    sides[3] = true;
                }
                _ => {}
            }
        }
    }
    for c in list.iter().filter(|c| c.from.shape() == item.id && c.to.shape() == item.id) {
        let (Anchor::Point(_, i), Anchor::Point(_, j)) = (c.from, c.to) else { continue };
        if i >= 5 && j >= 5 {
            // two nodes next to each other: the segment between them
            let n = item.nodes().map_or(0, |n| n.len());
            let (lo, hi) = (i.min(j) - 5, i.max(j) - 5);
            if hi == lo + 1 {
                segments.push(lo);
            } else if n > 2 && lo == 0 && hi == n - 1 {
                segments.push(n - 1);
            }
        } else if turned.is_some() && i < 5 && j < 5 {
            match c.axis {
                Axis::Horizontal => {
                    sides[0] = true;
                    sides[2] = true;
                }
                Axis::Vertical => {
                    sides[1] = true;
                    sides[3] = true;
                }
                // straight between two corners on one side: just that side's direction
                Axis::Aligned => {
                    let corner_side = |a: usize, b: usize| (a < 4 && b < 4 && (a + 1) % 4 == b).then_some(a).or_else(|| (a < 4 && b < 4 && (b + 1) % 4 == a).then_some(b));
                    if let Some(k) = corner_side(i, j) {
                        sides[k] = true;
                        sides[(k + 2) % 4] = true;
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    for (k, on) in sides.iter().enumerate() {
        if *on {
            out.extend(edge_of(item, k));
        }
    }
    segments.sort_unstable();
    segments.dedup();
    for i in segments {
        out.extend(edge_of(item, 4 + i));
    }
    out
}

/// Which way a line is kept: level, or upright.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orient {
    Horizontal,
    Vertical,
}

impl Orient {
    /// The way of the better fit: level when the two ends are further apart sideways than up.
    pub fn best_for(a: Pt, b: Pt) -> Orient {
        if (b[1] - a[1]).abs() <= (b[0] - a[0]).abs() {
            Orient::Horizontal
        } else {
            Orient::Vertical
        }
    }

    /// The direction in which the two ends must not differ: up for a level line, sideways for an
    /// upright one.
    fn across(self) -> Pt {
        match self {
            Orient::Horizontal => [0.0, 1.0],
            Orient::Vertical => [1.0, 0.0],
        }
    }
}

/// "These two points lie level" or "upright": the line between them (a side of a shape, a segment
/// of a vector, or any two points) keeps that way. The first point is the reference; between two
/// shapes the shape of the second point moves (translated, solved together with the distances),
/// on a vector the second node moves.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Align {
    pub a: Anchor,
    pub b: Anchor,
    pub orient: Orient,
}

impl Align {
    /// A constraint for the line between two points, kept the way that fits better.
    pub fn between(items: &[Item], a: Anchor, b: Anchor) -> Option<Align> {
        let (Anchor::Point(ia, i), Anchor::Point(ib, j)) = (a, b) else { return None };
        let (pa, pb) = (point_of(find(items, ia)?, i)?, point_of(find(items, ib)?, j)?);
        Some(Align { a, b, orient: Orient::best_for(pa, pb) })
    }

    /// A constraint for a line (a side or a segment): between its two end points.
    pub fn along(items: &[Item], edge: Anchor) -> Option<Align> {
        let probe = length_of(items, edge, [0.0, 0.0])?;
        Align::between(items, probe.from, probe.to)
    }

    /// Where the two points are now.
    pub fn ends(&self, items: &[Item]) -> Option<(Pt, Pt)> {
        let (Anchor::Point(ia, i), Anchor::Point(ib, j)) = (self.a, self.b) else { return None };
        Some((point_of(find(items, ia)?, i)?, point_of(find(items, ib)?, j)?))
    }

    /// How far the points are from lying the right way (0 when they do), in mm.
    pub fn error(&self, items: &[Item]) -> Option<f64> {
        let (pa, pb) = self.ends(items)?;
        let w = self.orient.across();
        Some(((pb[0] - pa[0]) * w[0] + (pb[1] - pa[1]) * w[1]).abs())
    }

    /// Both points exist and the pair makes sense, or why it doesn't (for a message).
    pub fn check(&self, items: &[Item]) -> Result<(), String> {
        if self.ends(items).is_none() {
            return Err("Eine der Formen gibt es nicht mehr".into());
        }
        if self.a == self.b {
            return Err("Zwei verschiedene Punkte wählen".into());
        }
        if self.b.shape() == PLATE {
            return Err("Die Platte bleibt, wo sie ist: zuerst die Form, dann die Platte wählen".into());
        }
        if self.a.shape() == self.b.shape() {
            // on one shape only the nodes of a vector can move by themselves; anything else must
            // already lie that way (the sides of a rectangle always do)
            let (Anchor::Point(_, i), Anchor::Point(_, j)) = (self.a, self.b) else { return Err("Nur Punkte".into()) };
            let nodes = i >= 5 && j >= 5 && find(items, self.a.shape()).is_some_and(|it| it.nodes().is_some());
            if !nodes && self.error(items).is_some_and(|e| e > 1e-6) {
                return Err("Auf derselben Form geht das nur an den Knoten eines Vektors; diese Linie liegt nicht so".into());
            }
        }
        Ok(())
    }

    /// On one vector: the second node moves until the points lie the right way. Returns whether
    /// anything changed. (Between two shapes the solver does it, see [`solve_all_with`].)
    pub fn apply_nodes(&self, items: &mut [Item]) -> bool {
        if self.a.shape() != self.b.shape() || self.check(items).is_err() {
            return false;
        }
        let (Anchor::Point(id, i), Anchor::Point(_, j)) = (self.a, self.b) else { return false };
        if i < 5 || j < 5 {
            return false;
        }
        let Some((pa, pb)) = self.ends(items) else { return false };
        let w = self.orient.across();
        let off = (pb[0] - pa[0]) * w[0] + (pb[1] - pa[1]) * w[1];
        if off.abs() < 1e-9 {
            return false;
        }
        let moved = [pb[0] - w[0] * off, pb[1] - w[1] * off];
        let Some(item) = items.iter_mut().find(|it| it.id == id) else { return false };
        *item = item.with_node(j - 5, moved);
        true
    }
}

/// "This point lies on this line" — the Fusion "Coincident" case of a point on a line/edge. The
/// point's perpendicular distance to the (infinite) line through the edge is kept at zero. The
/// line is the reference; between two shapes the shape of the point moves (translated, solved
/// together with the distances and aligns), on a vector the point's own node moves.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OnLine {
    pub point: Anchor,
    pub line: Anchor,
}

impl OnLine {
    /// A point-on-line constraint from a picked point and a picked line (any order from the UI is
    /// sorted here: the [`Anchor::Point`] is the point, the [`Anchor::Edge`] is the line).
    pub fn between(a: Anchor, b: Anchor) -> Option<OnLine> {
        match (a, b) {
            (Anchor::Point(..), Anchor::Edge(..)) => Some(OnLine { point: a, line: b }),
            (Anchor::Edge(..), Anchor::Point(..)) => Some(OnLine { point: b, line: a }),
            _ => None,
        }
    }

    /// The point and the line's two end points, as they are now.
    fn parts(&self, items: &[Item]) -> Option<(Pt, Pt, Pt)> {
        let Anchor::Point(ip, i) = self.point else { return None };
        let Anchor::Edge(il, k) = self.line else { return None };
        let p = point_of(find(items, ip)?, i)?;
        let (e0, e1) = edge_of(find(items, il)?, k)?;
        Some((p, e0, e1))
    }

    /// The point's signed perpendicular distance to the line (0 when it lies on it), in mm.
    pub fn error(&self, items: &[Item]) -> Option<f64> {
        let (p, e0, e1) = self.parts(items)?;
        let n = normal(e0, e1)?;
        Some(dot([p[0] - e0[0], p[1] - e0[1]], n))
    }

    /// Both ends exist and the pair makes sense (a point and a line), or why it doesn't.
    pub fn check(&self, items: &[Item]) -> Result<(), String> {
        if !matches!((self.point, self.line), (Anchor::Point(..), Anchor::Edge(..))) {
            return Err("Punkt auf Linie braucht einen Punkt und eine Linie".into());
        }
        if self.parts(items).is_none() {
            return Err("Punkt oder Linie gibt es nicht mehr".into());
        }
        if self.point.shape() == PLATE {
            return Err("Die Platte bleibt, wo sie ist: zuerst den Punkt der Form, dann die Linie wählen".into());
        }
        if self.point.shape() == self.line.shape() {
            // on one shape only a vector's node can move to reach its own line; a box corner sits on
            // its own side already, anything else can't move by itself
            let Anchor::Point(_, i) = self.point else { return Err("Nur ein Punkt".into()) };
            let node = i >= 5 && find(items, self.point.shape()).is_some_and(|it| it.nodes().is_some());
            if !node && self.error(items).is_some_and(|e| e.abs() > 1e-6) {
                return Err("Auf derselben Form geht das nur an den Knoten eines Vektors".into());
            }
        }
        Ok(())
    }

    /// Whether the point end is a vector/polygon node (index 5 and up), not a box corner or centre.
    /// Such a point is moved by itself onto the line, so reshaping the other shape leaves the rest
    /// of the polygon alone — the one node stays on the line (the user's bug: a polygon's start node
    /// on a line must stay there when the polygon is changed).
    pub fn point_is_node(&self, items: &[Item]) -> bool {
        matches!(self.point, Anchor::Point(_, i) if i >= 5) && find(items, self.point.shape()).is_some_and(|it| it.nodes().is_some())
    }

    /// Move just the point's node perpendicularly onto its line, whether the line is on the same
    /// shape (a vector onto its own side) or another shape. Only when the point is a node (see
    /// [`Self::point_is_node`]); between two shapes with a plain corner/centre point the translation
    /// solver moves the whole shape instead. Returns whether anything changed.
    pub fn apply_nodes(&self, items: &mut [Item]) -> bool {
        if !self.point_is_node(items) || self.check(items).is_err() {
            return false;
        }
        let Anchor::Point(id, i) = self.point else { return false };
        let Some((p, e0, e1)) = self.parts(items) else { return false };
        let Some(n) = normal(e0, e1) else { return false };
        let off = dot([p[0] - e0[0], p[1] - e0[1]], n);
        if off.abs() < 1e-9 {
            return false;
        }
        let moved = [p[0] - n[0] * off, p[1] - n[1] * off];
        let Some(item) = items.iter_mut().find(|it| it.id == id) else { return false };
        *item = item.with_node(i - 5, moved);
        true
    }
}

/// The most rounds the constraints on single shapes (sizes, segment lengths, level/upright nodes,
/// node points on lines) are applied in one solve. Constraints that agree settle in a few rounds
/// (each round fixes what the round before disturbed further down a chain); still changing after
/// this many means they contradict each other, and the solve leaves those shapes as they were.
pub const SAME_SHAPE_ROUNDS: usize = 32;

/// How far a distance is from its value (absolute, in mm), or `None` when an end is missing.
/// Zero (within a hair) means it holds; anything else is a constraint the others prevent.
pub fn shortfall(c: &Distance, items: &[Item], value: f64) -> Option<f64> {
    Some((c.measure(items)?.d.abs() - value).abs())
}

/// Keep every constraint. Distances on one shape (its own width, height or a line's length) set
/// that shape's size first, each on its own. Then all distances **between** shapes are solved
/// together: the shapes that are the second end of a distance are moved (translated) by the
/// smallest change that makes every distance hold, so independent constraints, several on one
/// shape and chains work out at once. A shape that only serves as a reference stays; in a chain the
/// shape further down gives way before the one it hangs from. Constraints that contradict each
/// other can't all hold: the compromise is kept and [`shortfall`] says which are off.
/// `value` turns an expression into a number (`None` leaves that constraint alone). Returns
/// whether any shape changed.
pub fn solve_all(list: &[Distance], items: &mut [Item], value: &dyn Fn(&str) -> Option<f64>) -> bool {
    solve_all_with(list, &[], items, value)
}

/// [`solve_all`] together with the level/upright constraints: on a vector the second node of such
/// a line is moved, between two shapes it is one more equation for the shapes' shifts (the two
/// points must not differ across the line's way), solved together with the distances.
pub fn solve_all_with(list: &[Distance], aligns: &[Align], items: &mut [Item], value: &dyn Fn(&str) -> Option<f64>) -> bool {
    solve_all_full(list, aligns, &[], items, value)
}

/// [`solve_all_with`] together with the point-on-line (Coincident) constraints: on a vector the
/// point's own node moves onto its line; between two shapes it is one more equation (the point's
/// perpendicular distance to the line must be zero), solved together with the distances and aligns.
pub fn solve_all_full(list: &[Distance], aligns: &[Align], on_lines: &[OnLine], items: &mut [Item], value: &dyn Fn(&str) -> Option<f64>) -> bool {
    let mut changed = false;
    // 1. a shape's own size, a vector's segment lengths, its nodes that must lie level or upright,
    // and node points that must sit on a line. Each of these moves something the others may look
    // at (one node is the end of two segments), so they are applied round after round until a
    // round changes nothing. That is bounded: constraints that contradict each other would go on
    // forever, so after `SAME_SHAPE_ROUNDS` the shapes are put back as they were and the
    // contradiction shows (its constraints are drawn red), instead of the shapes drifting away.
    let before: Vec<Item> = items.to_vec();
    let mut settled = false;
    let mut moved_any = false;
    for _ in 0..SAME_SHAPE_ROUNDS {
        let mut round = false;
        for c in list.iter().filter(|c| c.from.shape() == c.to.shape()) {
            if let Some(v) = value(&c.expr) {
                round |= c.apply(items, v);
            }
        }
        for al in aligns.iter().filter(|a| a.a.shape() == a.b.shape()) {
            round |= al.apply_nodes(items);
        }
        // a point-on-line whose point is a vector/polygon node moves just that node onto the line,
        // whether the line is on the same shape or another; the rest of the shape is left alone
        let node_on_lines: Vec<&OnLine> = on_lines.iter().filter(|o| o.point_is_node(items)).collect();
        for ol in node_on_lines {
            round |= ol.apply_nodes(items);
        }
        moved_any |= round;
        if !round {
            settled = true;
            break;
        }
    }
    if settled {
        changed |= moved_any;
    } else {
        items.clone_from_slice(&before);
    }
    // 2. distances between shapes, as equations in the shifts of the moving shapes
    struct Row {
        from: Option<usize>,
        to: usize,
        /// The vector from the first to the second end now, the measuring direction (fixed unless
        /// the distance is a straight one), the signed distance, the wanted one.
        v0: Pt,
        dir: Pt,
        d0: f64,
        target: f64,
        straight: bool,
    }
    let mut movers: Vec<u64> = Vec::new();
    let mut usable: Vec<(&Distance, f64)> = Vec::new();
    for c in list.iter().filter(|c| c.from.shape() != c.to.shape()) {
        let Some(v) = value(&c.expr) else { continue };
        if !(v > 0.0) || c.check(items).is_err() {
            continue;
        }
        if !movers.contains(&c.to.shape()) {
            movers.push(c.to.shape());
        }
        usable.push((c, v));
    }
    // level/upright lines between two shapes: the shape of the second point moves
    let mut cross_aligns: Vec<&Align> = Vec::new();
    for al in aligns.iter().filter(|a| a.a.shape() != a.b.shape()) {
        if al.check(items).is_err() {
            continue;
        }
        if !movers.contains(&al.b.shape()) {
            movers.push(al.b.shape());
        }
        cross_aligns.push(al);
    }
    // point-on-line between two shapes whose point is a plain corner/centre: the whole point shape
    // moves onto the line. (A node point is handled above by moving just that node.)
    let mut cross_on_lines: Vec<&OnLine> = Vec::new();
    for ol in on_lines.iter().filter(|o| o.point.shape() != o.line.shape() && !o.point_is_node(items)) {
        if ol.check(items).is_err() {
            continue;
        }
        if !movers.contains(&ol.point.shape()) {
            movers.push(ol.point.shape());
        }
        cross_on_lines.push(ol);
    }
    if usable.is_empty() && cross_aligns.is_empty() && cross_on_lines.is_empty() {
        return changed;
    }
    // how far a mover hangs from a fixed shape: the deeper one is the easier to move
    let mut depth = vec![1usize; movers.len()];
    for _ in 0..movers.len() {
        for (c, _) in &usable {
            let Some(from) = movers.iter().position(|m| *m == c.from.shape()) else { continue };
            let to = movers.iter().position(|m| *m == c.to.shape()).expect("every second end is a mover");
            depth[to] = depth[to].max(depth[from] + 1).min(movers.len() + 1);
        }
    }
    for _ in 0..movers.len() {
        for al in &cross_aligns {
            let Some(from) = movers.iter().position(|m| *m == al.a.shape()) else { continue };
            let to = movers.iter().position(|m| *m == al.b.shape()).expect("every second end is a mover");
            depth[to] = depth[to].max(depth[from] + 1).min(movers.len() + 1);
        }
        for ol in &cross_on_lines {
            let Some(from) = movers.iter().position(|m| *m == ol.line.shape()) else { continue };
            let to = movers.iter().position(|m| *m == ol.point.shape()).expect("the point's shape is a mover");
            depth[to] = depth[to].max(depth[from] + 1).min(movers.len() + 1);
        }
    }
    let deepest = *depth.iter().max().unwrap_or(&1);
    let weight: Vec<f64> = depth.iter().map(|d| 100f64.powi((deepest - d) as i32)).collect();
    let mut rows: Vec<Row> = Vec::new();
    for (c, v) in &usable {
        let Some(m) = c.measure(items) else { continue };
        let straight = c.axis == Axis::Aligned && matches!((c.from, c.to), (Anchor::Point(..), Anchor::Point(..)));
        rows.push(Row {
            from: movers.iter().position(|x| *x == c.from.shape()),
            to: movers.iter().position(|x| *x == c.to.shape()).expect("every second end is a mover"),
            v0: [m.b[0] - m.a[0], m.b[1] - m.a[1]],
            dir: m.dir,
            d0: m.d,
            target: if m.d < 0.0 && !straight { -v } else { *v },
            straight,
        });
    }
    for al in &cross_aligns {
        let Some((pa, pb)) = al.ends(items) else { continue };
        let w = al.orient.across();
        let v0 = [pb[0] - pa[0], pb[1] - pa[1]];
        rows.push(Row {
            from: movers.iter().position(|x| *x == al.a.shape()),
            to: movers.iter().position(|x| *x == al.b.shape()).expect("every second end is a mover"),
            v0,
            dir: w,
            d0: v0[0] * w[0] + v0[1] * w[1],
            target: 0.0,
            straight: false,
        });
    }
    for ol in &cross_on_lines {
        // the point's perpendicular distance to its line must be zero; the point's shape moves,
        // the line's shape is the reference. `dir` is the line's normal (fixed), `target` 0.
        let Some((p, e0, e1)) = ol.parts(items) else { continue };
        let Some(nrm) = normal(e0, e1) else { continue };
        let v0 = [p[0] - e0[0], p[1] - e0[1]];
        rows.push(Row {
            from: movers.iter().position(|x| *x == ol.line.shape()),
            to: movers.iter().position(|x| *x == ol.point.shape()).expect("the point's shape is a mover"),
            v0,
            dir: nrm,
            d0: dot(v0, nrm),
            target: 0.0,
            straight: false,
        });
    }
    let n = movers.len();
    let mut shift = vec![[0.0f64; 2]; n];
    for _ in 0..10 {
        // residuals and the rows of the Jacobian at the present shifts
        let mut jac: Vec<Vec<f64>> = Vec::new();
        let mut res: Vec<f64> = Vec::new();
        for r in &rows {
            let mut dv = shift[r.to];
            if let Some(f) = r.from {
                dv = [dv[0] - shift[f][0], dv[1] - shift[f][1]];
            }
            let (dir, d) = if r.straight {
                let v = [r.v0[0] + dv[0], r.v0[1] + dv[1]];
                let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
                if len > 1e-12 { ([v[0] / len, v[1] / len], len) } else { ([1.0, 0.0], 0.0) }
            } else {
                (r.dir, r.d0 + r.dir[0] * dv[0] + r.dir[1] * dv[1])
            };
            let mut row = vec![0.0; 2 * n];
            row[2 * r.to] += dir[0];
            row[2 * r.to + 1] += dir[1];
            if let Some(f) = r.from {
                row[2 * f] -= dir[0];
                row[2 * f + 1] -= dir[1];
            }
            jac.push(row);
            res.push(r.target - d);
        }
        if res.iter().all(|e| e.abs() < 1e-10) {
            break;
        }
        // the smallest weighted change that cancels the residuals: delta = W^-1 J^T mu with
        // (J W^-1 J^T + eps) mu = res
        let m = rows.len();
        let mut a = vec![vec![0.0; m + 1]; m];
        for i in 0..m {
            for j in 0..m {
                a[i][j] = (0..2 * n).map(|k| jac[i][k] * jac[j][k] / weight[k / 2]).sum();
            }
            a[i][i] += 1e-9;
            a[i][m] = res[i];
        }
        let Some(mu) = gauss(&mut a) else { break };
        for k in 0..n {
            for axis in 0..2 {
                let dx: f64 = (0..m).map(|i| jac[i][2 * k + axis] * mu[i]).sum::<f64>() / weight[k];
                shift[k][axis] += dx;
            }
        }
    }
    for (k, id) in movers.iter().enumerate() {
        if shift[k][0].abs() < 1e-9 && shift[k][1].abs() < 1e-9 {
            continue;
        }
        if let Some(item) = items.iter_mut().find(|it| it.id == *id) {
            *item = item.translated(shift[k][0], shift[k][1]);
            changed = true;
        }
    }
    changed
}

/// Solve a small linear system given as an augmented matrix (Gauss with row swaps). `None` when
/// it is singular.
fn gauss(a: &mut [Vec<f64>]) -> Option<Vec<f64>> {
    let m = a.len();
    for col in 0..m {
        let pivot = (col..m).max_by(|x, y| a[*x][col].abs().total_cmp(&a[*y][col].abs()))?;
        if a[pivot][col].abs() < 1e-14 {
            return None;
        }
        a.swap(col, pivot);
        for row in col + 1..m {
            let f = a[row][col] / a[col][col];
            for k in col..=m {
                a[row][k] -= f * a[col][k];
            }
        }
    }
    let mut x = vec![0.0; m];
    for i in (0..m).rev() {
        let s: f64 = (i + 1..m).map(|k| a[i][k] * x[k]).sum();
        x[i] = (a[i][m] - s) / a[i][i];
    }
    Some(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Path, Rect};

    fn item(id: u64, cx: f64, cy: f64, w: f64, h: f64) -> Item {
        Item::new(id, Rect::new([cx, cy], w, h))
    }

    #[test]
    fn a_constraint_decides_how_much_of_it_shows() {
        let c = d(Anchor::Point(1, 0), Anchor::Point(2, 0), Axis::Horizontal);
        assert_eq!(c.view(false, &[]), ConstraintView::Hidden);
        assert_eq!(c.view(true, &[]), ConstraintView::Slim);
        assert_eq!(c.view(true, &[3]), ConstraintView::Slim);
        assert_eq!(c.view(false, &[2]), ConstraintView::Full);
        assert_eq!(c.view(true, &[1]), ConstraintView::Full);
        assert_eq!(SizeConstraint { shape: 5 }.view(false, &[5]), ConstraintView::Full);
    }

    fn d(from: Anchor, to: Anchor, axis: Axis) -> Distance {
        Distance { from, to, axis, expr: "0".into(), place: [0.0, 0.0], offset: None }
    }

    #[test]
    fn horizontal_distance_moves_the_second_shape_only() {
        // A: x 0..10; B: x 14..24 (lower-left 14,0)
        let mut items = vec![item(1, 5.0, 5.0, 10.0, 10.0), item(2, 19.0, 5.0, 10.0, 10.0)];
        let c = d(Anchor::Point(1, 1), Anchor::Point(2, 0), Axis::Horizontal); // A lower-right → B lower-left
        assert!((c.measure(&items).unwrap().d - 4.0).abs() < 1e-9);
        assert!(c.apply(&mut items, 25.0));
        assert_eq!(point_of(&items[1], 0), Some([35.0, 0.0]), "B moved right, y kept");
        assert_eq!(point_of(&items[0], 0), Some([0.0, 0.0]), "A is the reference");
        assert!(!c.apply(&mut items, 25.0), "already holds");
    }

    #[test]
    fn the_side_is_kept_and_vertical_leaves_x_alone() {
        // B left of A: signed distance negative stays negative
        let mut items = vec![item(1, 50.0, 5.0, 10.0, 10.0), item(2, 20.0, 9.0, 10.0, 10.0)];
        let c = d(Anchor::Point(1, 0), Anchor::Point(2, 1), Axis::Horizontal);
        c.apply(&mut items, 8.0);
        let m = c.measure(&items).unwrap();
        assert!((m.d + 8.0).abs() < 1e-9, "still to the left, now 8 away");
        let v = d(Anchor::Point(1, 0), Anchor::Point(2, 0), Axis::Vertical);
        let x_before = point_of(&items[1], 0).unwrap()[0];
        v.apply(&mut items, 30.0);
        assert_eq!(point_of(&items[1], 0).unwrap()[0], x_before);
        assert!((v.measure(&items).unwrap().d.abs() - 30.0).abs() < 1e-9);
    }

    #[test]
    fn straight_distance_keeps_the_direction() {
        let mut items = vec![item(1, 0.0, 0.0, 2.0, 2.0), item(2, 13.0, 4.0, 2.0, 2.0)];
        let c = d(Anchor::Point(1, 4), Anchor::Point(2, 4), Axis::Aligned);
        assert!((c.measure(&items).unwrap().d - (13f64 * 13.0 + 16.0).sqrt()).abs() < 1e-9);
        c.apply(&mut items, 50.0);
        let m = c.measure(&items).unwrap();
        assert!((m.d - 50.0).abs() < 1e-9);
        // the direction is still 13 : 4
        let (p, q) = (point_of(&items[0], 4).unwrap(), point_of(&items[1], 4).unwrap());
        assert!(((q[1] - p[1]) / (q[0] - p[0]) - 4.0 / 13.0).abs() < 1e-9);
    }

    #[test]
    fn point_to_line_is_the_perpendicular_distance() {
        // the line: A's top side (y = 10); the point: B's lower-left corner
        let mut items = vec![item(1, 5.0, 5.0, 10.0, 10.0), item(2, 30.0, 25.0, 10.0, 10.0)];
        let c = d(Anchor::Edge(1, 2), Anchor::Point(2, 0), Axis::Aligned);
        // top side runs right→left (corner 2 → 3), its normal points down; B is above: the sign says so
        let m = c.measure(&items).unwrap();
        assert!((m.d.abs() - 10.0).abs() < 1e-9, "B's lower edge is at y 20, the line at y 10");
        c.apply(&mut items, 4.0);
        assert!((point_of(&items[1], 0).unwrap()[1] - 14.0).abs() < 1e-9);
        assert_eq!(point_of(&items[1], 0).unwrap()[0], 25.0, "moved along the normal only");
    }

    #[test]
    fn parallel_lines_measure_their_gap_and_break_when_turned() {
        // two open polylines, each one horizontal segment, 20 apart; edge index 4 is that segment
        let line = |id: u64, y: f64| Item::new(id, crate::Path::new(vec![[0.0, y], [10.0, y]], false));
        let mut items = vec![line(1, 0.0), line(2, 20.0)];
        let c = d(Anchor::Edge(1, 4), Anchor::Edge(2, 4), Axis::Aligned);
        assert!(c.check(&items).is_ok(), "parallel lines may be dimensioned");
        let m = c.measure(&items).expect("parallel: has a gap");
        assert!((m.d.abs() - 20.0).abs() < 1e-9, "the lines are 20 apart, got {}", m.d);
        // setting the gap to 30 moves the second line along the normal
        assert!(c.apply(&mut items, 30.0));
        assert!((c.measure(&items).unwrap().d.abs() - 30.0).abs() < 1e-9);
        // turn the second line: no longer parallel → no measurement and the check refuses it
        let turned = items[1].rotated([5.0, 30.0], 20.0);
        let items2 = vec![items[0].clone(), turned];
        assert!(c.measure(&items2).is_none(), "not parallel → unmeasurable (host drops it)");
        assert!(c.check(&items2).is_err(), "not parallel → refused when newly set");
    }

    #[test]
    fn on_one_shape_the_shape_is_scaled_about_the_first_point() {
        let mut items = vec![item(1, 5.0, 5.0, 10.0, 6.0)]; // x 0..10, y 2..8
        // width: lower-left to lower-right
        let w = d(Anchor::Point(1, 0), Anchor::Point(1, 1), Axis::Horizontal);
        assert!(w.check(&items).is_ok());
        assert!(w.apply(&mut items, 40.0));
        let (lo, hi) = items[0].bounds().unwrap();
        assert!((hi[0] - lo[0] - 40.0).abs() < 1e-9 && lo[0].abs() < 1e-9, "grows from the first point");
        assert!((hi[1] - lo[1] - 6.0).abs() < 1e-9, "height untouched");
        // from the right corner the right side stays and the left one moves
        let back = d(Anchor::Point(1, 1), Anchor::Point(1, 0), Axis::Horizontal);
        assert!(back.apply(&mut items, 20.0));
        let (lo, hi) = items[0].bounds().unwrap();
        assert!((hi[0] - 40.0).abs() < 1e-9 && (hi[0] - lo[0] - 20.0).abs() < 1e-9);
        // centre to corner, vertical: half the height
        let half = d(Anchor::Point(1, 4), Anchor::Point(1, 2), Axis::Vertical);
        assert!(half.apply(&mut items, 6.0));
        let (lo, hi) = items[0].bounds().unwrap();
        assert!((hi[1] - lo[1] - 12.0).abs() < 1e-9, "the half is 6, so the whole is 12");
        // straight between opposite corners keeps the proportions
        let diag = d(Anchor::Point(1, 0), Anchor::Point(1, 2), Axis::Aligned);
        let before = items[0].bounds().unwrap();
        let ratio = (before.1[0] - before.0[0]) / (before.1[1] - before.0[1]);
        assert!(diag.apply(&mut items, 100.0));
        let (lo, hi) = items[0].bounds().unwrap();
        assert!(((hi[0] - lo[0]) / (hi[1] - lo[1]) - ratio).abs() < 1e-9);
        assert!((dist([lo[0], lo[1]], [hi[0], hi[1]]) - 100.0).abs() < 1e-9);
        // a point and a side of the same shape: the centre to the bottom side
        let side = d(Anchor::Point(1, 4), Anchor::Edge(1, 0), Axis::Aligned);
        assert!(side.check(&items).is_ok());
        // no distance in that direction (two corners on one vertical side, measured horizontally) is refused
        let flat = d(Anchor::Point(1, 0), Anchor::Point(1, 3), Axis::Horizontal);
        assert!(flat.check(&items).is_err());
        let same = d(Anchor::Point(1, 0), Anchor::Point(1, 0), Axis::Aligned);
        assert!(same.check(&items).is_err());
    }

    #[test]
    fn solve_all_follows_a_chain_and_a_missing_shape_is_refused() {
        let mut items = vec![item(1, 5.0, 5.0, 10.0, 10.0), item(2, 40.0, 5.0, 10.0, 10.0), item(3, 90.0, 5.0, 10.0, 10.0)];
        // list order is reversed on purpose: the second pass settles it
        let c2 = Distance { expr: "10".into(), ..d(Anchor::Point(2, 1), Anchor::Point(3, 0), Axis::Horizontal) };
        let c1 = Distance { expr: "10".into(), ..d(Anchor::Point(1, 1), Anchor::Point(2, 0), Axis::Horizontal) };
        assert!(solve_all(&[c2.clone(), c1.clone()], &mut items, &|e| e.parse().ok()));
        assert!((point_of(&items[1], 0).unwrap()[0] - 20.0).abs() < 1e-9);
        assert!((point_of(&items[2], 0).unwrap()[0] - 40.0).abs() < 1e-9);
        let gone = d(Anchor::Point(1, 0), Anchor::Point(9, 0), Axis::Aligned);
        assert!(gone.check(&items).is_err());
    }

    #[test]
    fn picking_prefers_points_over_lines() {
        let items = vec![item(1, 5.0, 5.0, 10.0, 10.0)];
        assert_eq!(pick(&items, [10.2, 0.3], 1.0), Some(Anchor::Point(1, 1)));
        assert_eq!(pick(&items, [5.0, 0.4], 1.0), Some(Anchor::Edge(1, 0)));
        assert_eq!(pick(&items, [5.0, 5.0], 1.0), Some(Anchor::Point(1, 4)), "the centre");
        assert_eq!(pick(&items, [50.0, 50.0], 1.0), None);
        // a vector offers its nodes and segments
        let path = Item::new(2, Path::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]], false));
        assert!(point_count(&path) >= 8 && edge_count(&path) == 6);
    }

    #[test]
    fn the_plate_is_a_fixed_reference() {
        // plate 300 x 200; a rectangle 20 x 10 with its lower-left corner at 12/7
        let mut items = vec![item(1, 22.0, 12.0, 20.0, 10.0), plate_item(300.0, 200.0)];
        // distance from the plate's left side to the rectangle's lower-left corner, measured straight
        let c = d(Anchor::Edge(PLATE, 3), Anchor::Point(1, 0), Axis::Aligned);
        assert!(c.check(&items).is_ok());
        assert!((c.measure(&items).unwrap().d.abs() - 12.0).abs() < 1e-9);
        assert!(c.apply(&mut items, 30.0));
        assert!((point_of(&items[0], 0).unwrap()[0] - 30.0).abs() < 1e-9);
        assert_eq!(point_of(&items[1], 0), Some([0.0, 0.0]), "the plate stays");
        // the plate can't be the moving end
        let back = d(Anchor::Point(1, 0), Anchor::Edge(PLATE, 3), Axis::Aligned);
        assert!(back.check(&items).is_err());
        // a corner of the plate as the reference of a vertical distance
        let v = d(Anchor::Point(PLATE, 0), Anchor::Point(1, 0), Axis::Vertical);
        v.apply(&mut items, 25.0);
        assert!((point_of(&items[0], 0).unwrap()[1] - 25.0).abs() < 1e-9);
        assert!(pick(&items, [0.2, 100.0], 1.0) == Some(Anchor::Edge(PLATE, 3)), "the plate's side can be picked");
    }

    #[test]
    fn a_lines_length_is_the_distance_between_its_ends() {
        let mut items = vec![item(1, 5.0, 5.0, 10.0, 6.0)];
        // the bottom side is the width, the right side the height
        let bottom = length_of(&items, Anchor::Edge(1, 0), [0.0, 0.0]).unwrap();
        assert!((bottom.measure(&items).unwrap().d.abs() - 10.0).abs() < 1e-9);
        assert!(bottom.apply(&mut items, 25.0));
        let right = length_of(&items, Anchor::Edge(1, 1), [0.0, 0.0]).unwrap();
        assert!((right.measure(&items).unwrap().d.abs() - 6.0).abs() < 1e-9);
        // the top side runs right to left: the sign doesn't matter
        let top = length_of(&items, Anchor::Edge(1, 2), [0.0, 0.0]).unwrap();
        assert!(top.check(&items).is_ok() && top.apply(&mut items, 30.0));
        let (lo, hi) = items[0].bounds().unwrap();
        assert!((hi[0] - lo[0] - 30.0).abs() < 1e-9);
        // a vector's segment
        let mut path = vec![Item::new(2, Path::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]], false))];
        let seg = length_of(&path, Anchor::Edge(2, 5), [0.0, 0.0]).unwrap();
        assert!((seg.measure(&path).unwrap().d.abs() - 10.0).abs() < 1e-9);
        assert!(seg.check(&path).is_ok() && seg.apply(&mut path, 20.0));
        assert!(length_of(&items, Anchor::Point(1, 0), [0.0, 0.0]).is_none());
    }

    #[test]
    fn several_distances_on_one_shape_hold_together() {
        // B is held from A horizontally and from the plate vertically, and from another shape straight
        let mut items = vec![item(1, 5.0, 5.0, 10.0, 10.0), item(2, 60.0, 40.0, 10.0, 10.0), item(3, 120.0, 20.0, 10.0, 10.0), plate_item(300.0, 200.0)];
        let h = Distance { expr: "25".into(), ..d(Anchor::Point(1, 1), Anchor::Point(2, 0), Axis::Horizontal) };
        let v = Distance { expr: "30".into(), ..d(Anchor::Edge(PLATE, 0), Anchor::Point(2, 0), Axis::Aligned) };
        let diag = Distance { expr: "50".into(), ..d(Anchor::Point(3, 4), Anchor::Point(2, 4), Axis::Aligned) };
        let value = |e: &str| e.parse::<f64>().ok();
        solve_all(&[h.clone(), v.clone()], &mut items, &value);
        assert!(shortfall(&h, &items, 25.0).unwrap() < 1e-9 && shortfall(&v, &items, 30.0).unwrap() < 1e-9, "two axes at once");
        assert_eq!(point_of(&items[0], 0), Some([0.0, 0.0]), "the reference stayed");
        // a third, straight one to a shape that stays: the same shape has to satisfy all three, which
        // fix two numbers: this one conflicts (B's place is already given), and says so
        solve_all(&[h.clone(), v.clone(), diag.clone()], &mut items, &value);
        let off: f64 = [(&h, 25.0), (&v, 30.0), (&diag, 50.0)].iter().map(|(c, x)| shortfall(c, &items, *x).unwrap()).sum();
        assert!(off > 1e-3, "contradicting distances can't all hold");
    }

    #[test]
    fn a_chain_moves_the_far_end_and_keeps_the_middle_still() {
        // A -> B -> C, each 20 apart horizontally; only C should move when B--C changes
        let mut items = vec![item(1, 5.0, 5.0, 10.0, 10.0), item(2, 35.0, 5.0, 10.0, 10.0), item(3, 70.0, 5.0, 10.0, 10.0)];
        let ab = Distance { expr: "20".into(), ..d(Anchor::Point(1, 1), Anchor::Point(2, 0), Axis::Horizontal) };
        let bc = Distance { expr: "20".into(), ..d(Anchor::Point(2, 1), Anchor::Point(3, 0), Axis::Horizontal) };
        let value = |e: &str| e.parse::<f64>().ok();
        solve_all(&[ab.clone(), bc.clone()], &mut items, &value);
        let b_x = point_of(&items[1], 0).unwrap()[0];
        assert!((b_x - 30.0).abs() < 1e-9 && (point_of(&items[2], 0).unwrap()[0] - 60.0).abs() < 1e-9);
        // change B--C: B stays put, C follows
        let bc = Distance { expr: "40".into(), ..bc };
        solve_all(&[ab, bc], &mut items, &value);
        assert!((point_of(&items[1], 0).unwrap()[0] - b_x).abs() < 1e-9, "B stays");
        assert!((point_of(&items[2], 0).unwrap()[0] - 80.0).abs() < 1e-9);
    }

    #[test]
    fn fixed_edges_are_the_sides_and_segments_with_a_set_length() {
        let mut r = item(1, 5.0, 5.0, 10.0, 6.0);
        assert!(fixed_edges(&r, &[]).is_empty(), "nothing set, nothing fixed");
        // a width tied to a variable: bottom and top
        r.bound.push((crate::Dim::Width, "breite".into()));
        let e = fixed_edges(&r, &[]);
        assert_eq!(e.len(), 2);
        assert!(e.iter().all(|(a, b)| (a[1] - b[1]).abs() < 1e-9), "horizontal sides");
        // plus a height set by a distance on the shape (left to upper-left, vertical)
        let h = Distance { expr: "6".into(), ..d(Anchor::Point(1, 0), Anchor::Point(1, 3), Axis::Vertical) };
        assert_eq!(fixed_edges(&r, &[h]).len(), 4);
        // a constraint between two shapes fixes no side
        let cross = Distance { expr: "5".into(), ..d(Anchor::Point(9, 0), Anchor::Point(1, 3), Axis::Vertical) };
        assert_eq!(fixed_edges(&item(1, 5.0, 5.0, 10.0, 6.0), &[cross]).len(), 0);
        // a vector's segment with a length constraint
        let path = Item::new(2, Path::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]], false));
        let seg = length_of(&[path.clone()], Anchor::Edge(2, 5), [0.0, 0.0]).unwrap();
        let fixed = fixed_edges(&path, &[seg]);
        assert_eq!(fixed, vec![([10.0, 0.0], [10.0, 10.0])]);
    }

    #[test]
    fn a_typed_size_is_a_distance_on_the_shape() {
        use crate::Dim;
        let r = item(1, 5.0, 5.0, 10.0, 6.0);
        assert_eq!(length_pair(&r, Dim::Width), Some((0, 1, Axis::Horizontal)));
        assert_eq!(length_pair(&r, Dim::Height), Some((0, 3, Axis::Vertical)));
        let w = Distance { expr: "40".into(), ..d(Anchor::Point(1, 0), Anchor::Point(1, 1), Axis::Horizontal) };
        assert_eq!(size_dim(&r, &w), Some(Dim::Width));
        // the same constraint from the other end, and one made with the dimension mode, count alike
        let back = d(Anchor::Point(1, 2), Anchor::Point(1, 3), Axis::Horizontal);
        assert_eq!(size_dim(&r, &back), Some(Dim::Width));
        assert_eq!(size_dim(&r, &d(Anchor::Point(1, 0), Anchor::Point(1, 4), Axis::Horizontal)), None, "the centre is not a side");
        // turned a quarter, the shape's width runs up the sheet
        let mut t = item(2, 5.0, 5.0, 10.0, 6.0);
        if let crate::Shape::Rect(rr) = &mut t.shape {
            rr.rotation = 90.0;
        }
        assert_eq!(length_pair(&t, Dim::Width), Some((0, 3, Axis::Vertical)));
        // other shapes keep the older binding
        let path = Item::new(3, Path::new(vec![[0.0, 0.0], [10.0, 0.0]], false));
        assert_eq!(length_pair(&path, Dim::Width), None);
    }

    #[test]
    fn the_dimension_line_follows_the_shapes() {
        let mut items = vec![item(1, 5.0, 5.0, 10.0, 10.0), item(2, 40.0, 5.0, 10.0, 10.0)];
        let mut c = d(Anchor::Point(1, 1), Anchor::Point(2, 0), Axis::Horizontal);
        // put 12 above the line between the two ends, as a click there would
        let m = c.measure(&items).unwrap();
        assert!(c.place_world(&m) == [0.0, 0.0], "before it is kept relative, the place put at creation counts");
        let mid_y = (m.a[1] + m.b[1]) / 2.0;
        c.set_place_world(&m, [20.0, mid_y + 12.0]);
        let before = c.place_world(&c.measure(&items).unwrap());
        // the second shape moves 30 right and 7 up: the dimension line goes along
        items[1] = items[1].translated(30.0, 7.0);
        let m2 = c.measure(&items).unwrap();
        let after = c.place_world(&m2);
        let mid_after = ((m2.a[0] + m2.b[0]) / 2.0, (m2.a[1] + m2.b[1]) / 2.0);
        assert!((after[1] - (mid_after.1 + 12.0)).abs() < 1e-9, "still 12 above the middle: {after:?}");
        assert!((after[0] - mid_after.0 - (20.0 - (m.a[0] + m.b[0]) / 2.0)).abs() < 1e-9, "same sideways offset");
        assert!(after != before, "it moved with the shape");
    }

    #[test]
    fn a_line_picks_level_or_upright_by_its_angle() {
        let items = vec![item(1, 5.0, 5.0, 10.0, 6.0)];
        // the box sides: bottom and top are level, left and right upright
        for (edge, want) in [(0, Orient::Horizontal), (1, Orient::Vertical), (2, Orient::Horizontal), (3, Orient::Vertical)] {
            let al = Align::along(&items, Anchor::Edge(1, edge)).unwrap();
            assert_eq!(al.orient, want, "side {edge}");
            assert!(al.check(&items).is_ok() && al.error(&items).unwrap() < 1e-9, "a rectangle's sides always fit");
        }
        // a slanted line: closer to level, so level; steep, so upright
        assert_eq!(Orient::best_for([0.0, 0.0], [10.0, 3.0]), Orient::Horizontal);
        assert_eq!(Orient::best_for([0.0, 0.0], [3.0, 10.0]), Orient::Vertical);
        assert_eq!(Orient::best_for([0.0, 0.0], [5.0, 5.0]), Orient::Horizontal, "a tie is level");
    }

    #[test]
    fn a_vector_segment_is_made_level_by_moving_its_second_node() {
        let mut items = vec![Item::new(1, Path::new(vec![[0.0, 0.0], [10.0, 2.0], [10.0, 10.0]], false))];
        // the first segment (nodes 0 and 1) leans a little: level
        let al = Align::along(&items, Anchor::Edge(1, 4)).unwrap();
        assert_eq!(al.orient, Orient::Horizontal);
        assert!((al.error(&items).unwrap() - 2.0).abs() < 1e-9);
        assert!(al.apply_nodes(&mut items));
        assert_eq!(items[0].nodes().unwrap(), vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]], "the second node gave way, the first stayed");
        assert!(!al.apply_nodes(&mut items), "holds now");
        // the next segment (nodes 1 and 2) is upright already; kept so when the first node moves
        let up = Align::along(&items, Anchor::Edge(1, 5)).unwrap();
        assert_eq!(up.orient, Orient::Vertical);
        assert!(up.error(&items).unwrap() < 1e-9);
        // two points of one shape that aren't nodes can't be forced
        let rect = vec![item(2, 5.0, 5.0, 10.0, 6.0)];
        let diag = Align { a: Anchor::Point(2, 0), b: Anchor::Point(2, 2), orient: Orient::Horizontal };
        assert!(diag.check(&rect).is_err());
    }

    #[test]
    fn level_between_two_shapes_moves_the_second_one_and_works_with_distances() {
        let mut items = vec![item(1, 5.0, 5.0, 10.0, 10.0), item(2, 40.0, 17.0, 10.0, 10.0), plate_item(300.0, 200.0)];
        // B's lower-left corner level with A's lower-left corner (y of both equal)
        let al = Align { a: Anchor::Point(1, 0), b: Anchor::Point(2, 0), orient: Orient::Horizontal };
        let value = |e: &str| e.parse::<f64>().ok();
        assert!(solve_all_with(&[], &[al.clone()], &mut items, &value));
        assert!(al.error(&items).unwrap() < 1e-9);
        assert_eq!(point_of(&items[0], 0), Some([0.0, 0.0]), "A is the reference");
        assert!((point_of(&items[1], 0).unwrap()[0] - 35.0).abs() < 1e-9, "only the sideways place was left alone");
        // together with a horizontal distance: both hold at once
        let dist = Distance { expr: "20".into(), ..d(Anchor::Point(1, 1), Anchor::Point(2, 0), Axis::Horizontal) };
        items[1] = items[1].translated(0.0, 9.0);
        solve_all_with(&[dist.clone()], &[al.clone()], &mut items, &value);
        assert!(al.error(&items).unwrap() < 1e-9 && shortfall(&dist, &items, 20.0).unwrap() < 1e-9);
        // the plate can't be the one that moves
        let bad = Align { a: Anchor::Point(1, 0), b: Anchor::Point(PLATE, 0), orient: Orient::Horizontal };
        assert!(bad.check(&items).is_err());
    }

    #[test]
    fn point_on_line_between_two_shapes_moves_the_point_shape_onto_the_line() {
        // B's lower-left corner must lie on A's right side (a vertical line at A's right edge).
        // A: box [0..10]×[0..10] → right side is x = 10. B starts to the right of it and above.
        let mut items = vec![item(1, 5.0, 5.0, 10.0, 10.0), item(2, 40.0, 20.0, 8.0, 8.0), plate_item(300.0, 200.0)];
        let value = |e: &str| e.parse::<f64>().ok();
        // A's right side is edge 1 (bottom 0, right 1, top 2, left 3); B's lower-left is point 0
        let ol = OnLine { point: Anchor::Point(2, 0), line: Anchor::Edge(1, 1) };
        assert!(ol.error(&items).unwrap().abs() > 1.0, "B is off the line to start with");
        assert!(solve_all_full(&[], &[], &[ol.clone()], &mut items, &value));
        assert!(ol.error(&items).unwrap().abs() < 1e-9, "B's corner now lies on A's right side");
        // only the perpendicular (x) place moved; B kept its along-line (y) place
        assert!((point_of(&items[1], 0).unwrap()[0] - 10.0).abs() < 1e-9, "snapped onto x = 10");
        assert!((point_of(&items[1], 0).unwrap()[1] - 16.0).abs() < 1e-9, "the along-line (y) place stayed");
        // the line's shape (A) is the reference and does not move
        assert_eq!(point_of(&items[0], 0), Some([0.0, 0.0]), "A stayed put");
        // the plate can't be the mover; Coincident needs a point and a line, not two points
        assert!(OnLine { point: Anchor::Point(PLATE, 0), line: Anchor::Edge(1, 1) }.check(&items).is_err());
        assert!(OnLine::between(Anchor::Point(1, 0), Anchor::Point(2, 0)).is_none());
        assert!(OnLine::between(Anchor::Edge(1, 1), Anchor::Point(2, 0)).is_some());
    }

    #[test]
    fn a_polygon_node_stays_on_a_line_when_the_polygon_is_reshaped() {
        // an open path P whose first node must lie on A's right side (x = 10); A is fixed.
        // Reshaping P (moving its other nodes) must not take the first node off the line.
        let a = item(1, 5.0, 5.0, 10.0, 10.0); // box 0..10 × 0..10, right side x = 10
        let p = Item::new(2, crate::Path::new(vec![[25.0, 20.0], [40.0, 30.0], [50.0, 5.0]], false));
        let mut items = vec![a, p, plate_item(300.0, 200.0)];
        let value = |e: &str| e.parse::<f64>().ok();
        // P's first node is point index 5; A's right side is edge 1
        let ol = OnLine { point: Anchor::Point(2, 5), line: Anchor::Edge(1, 1) };
        assert!(ol.point_is_node(&items), "the path's first node is a node point");
        assert!(solve_all_full(&[], &[], &[ol.clone()], &mut items, &value));
        assert!((items[1].nodes().unwrap()[0][0] - 10.0).abs() < 1e-9, "node 0 snapped to x = 10");
        assert!((items[1].nodes().unwrap()[0][1] - 20.0).abs() < 1e-9, "its y stayed (only the node moved)");
        // the rest of the path and A are untouched by the node move
        assert_eq!(items[1].nodes().unwrap()[1], [40.0, 30.0], "the other nodes did not move");
        assert_eq!(point_of(&items[0], 0), Some([0.0, 0.0]), "A did not move");
        // now reshape P: drag node 1 somewhere else, then re-solve — node 0 is still on the line
        items[1] = items[1].with_node(1, [70.0, 60.0]);
        solve_all_full(&[], &[], &[ol.clone()], &mut items, &value);
        assert!((items[1].nodes().unwrap()[0][0] - 10.0).abs() < 1e-9, "node 0 still on x = 10 after reshape");
    }

    #[test]
    fn two_segment_lengths_on_one_polyline_settle_and_stay_put() {
        // the user's Z: top (0,40)→(60,42), diagonal down to (5,1), bottom to (70,0)
        let z = Item::new(1, crate::Path::new(vec![[0.0, 40.0], [60.0, 42.0], [5.0, 1.0], [70.0, 0.0]], false));
        let mut items = vec![z, plate_item(300.0, 200.0)];
        let value = |e: &str| e.parse::<f64>().ok();
        let top = Distance { expr: "60".into(), ..length_of(&items, Anchor::Edge(1, 4), [0.0, 0.0]).unwrap() };
        let diagonal = Distance { expr: "80".into(), ..length_of(&items, Anchor::Edge(1, 5), [0.0, 0.0]).unwrap() };
        let list = [top.clone(), diagonal.clone()];
        assert!(solve_all(&list, &mut items, &value));
        assert!(shortfall(&top, &items, 60.0).unwrap() < 1e-9, "the top is 60");
        assert!(shortfall(&diagonal, &items, 80.0).unwrap() < 1e-9, "the diagonal is 80");
        // only the segments' end nodes moved: the first node and the last stay where they were
        let nodes = items[0].nodes().unwrap();
        assert_eq!(nodes[0], [0.0, 40.0], "the start of the Z stays");
        assert_eq!(nodes[3], [70.0, 0.0], "the end of the Z stays");
        // solving again (as every frame does) changes nothing: no drift
        let settled = items.clone();
        for _ in 0..100 {
            assert!(!solve_all(&list, &mut items, &value), "a settled solve is a no-op");
        }
        assert_eq!(items, settled, "the Z does not wander");
    }

    #[test]
    fn contradicting_lengths_leave_the_shape_alone_instead_of_drifting() {
        // one segment asked to be both 30 and 40 long: no answer exists
        let p = Item::new(1, crate::Path::new(vec![[0.0, 0.0], [35.0, 0.0], [35.0, 20.0]], false));
        let mut items = vec![p, plate_item(300.0, 200.0)];
        let value = |e: &str| e.parse::<f64>().ok();
        let a = Distance { expr: "30".into(), ..length_of(&items, Anchor::Edge(1, 4), [0.0, 0.0]).unwrap() };
        let b = Distance { expr: "40".into(), ..length_of(&items, Anchor::Edge(1, 4), [0.0, 0.0]).unwrap() };
        let before = items.clone();
        for _ in 0..100 {
            assert!(!solve_all(&[a.clone(), b.clone()], &mut items, &value), "the bounded solve gives up and changes nothing");
        }
        assert_eq!(items, before, "the shape stays where it was, however often it is solved");
        assert!(shortfall(&a, &items, 30.0).unwrap() > 1.0, "the contradiction shows (drawn red)");
    }

    #[test]
    fn placement_chooses_the_axis() {
        let (a, b) = ([0.0, 0.0], [10.0, 6.0]);
        assert_eq!(axis_for_placement(a, b, [5.0, 12.0]), Axis::Horizontal, "above, between the x of the points");
        assert_eq!(axis_for_placement(a, b, [14.0, 3.0]), Axis::Vertical);
        assert_eq!(axis_for_placement(a, b, [14.0, 12.0]), Axis::Aligned);
    }
}
