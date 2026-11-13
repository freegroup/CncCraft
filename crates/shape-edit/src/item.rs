//! [`Item`]: a shape with an id, ready for the [`crate::Editor`].

use crate::forms::{Compound, Dim, Grip, GripKind, Path, Shape};
use crate::snap::SnapKind;
use crate::geometry::{dist_to_segment, point_in_ring};
use crate::{Editable, Pt};
use serde::{Deserialize, Serialize};

/// Where a shape was imported from, so it can be read from disk again.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Source {
    /// The file's path as it was when the shape came in.
    pub file: String,
    /// The format's short name ("DXF").
    pub format: String,
    /// Which of the file's shapes this is (in the order the importer made them), so a reload
    /// knows what to read back into it. A copy keeps the number of its original.
    pub index: usize,
    /// How far the import moved the drawing (mm), so a reload puts it in the same place.
    pub offset: Pt,
}

/// A shape in a drawing, with the id the selection refers to.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: u64,
    pub shape: Shape,
    /// The file the shape was imported from, if it was. Any shape can have one. Old project files
    /// have none.
    #[serde(default)]
    pub source: Option<Source>,
    /// The groups the shape is in, innermost first: a group inside a group lists both. Picking any
    /// member picks the whole outermost group; ungrouping takes the shape out of its outermost
    /// group only. Old project files have none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<u64>,
    /// Sizes typed as a formula or variable (e.g. `Höhe = "hoehe"`): the expression is the source
    /// of truth and the shape's number is derived from it whenever the variables change. Old
    /// project files have none.
    #[serde(default)]
    pub bound: Vec<(Dim, String)>,
    /// Sides (across x, across y) whose size a distance constraint on the shape holds: set by the
    /// app before the editor looks at the shape, never saved. Their handles do nothing.
    #[serde(skip)]
    pub locked: (bool, bool),
    /// The layer the shape is on (ids start at 1). Old project files have all shapes on layer 1.
    #[serde(default = "first_layer")]
    pub layer: u32,
    /// The shape's layer is switched off: set by the app before the editor looks at the shape,
    /// never saved. A hidden shape can't be picked, snapped to or selected.
    #[serde(skip)]
    pub hidden: bool,
    /// The shape's layer is locked: it stays visible but nothing can be done with it (no picking,
    /// selecting, snapping). Set by the app like `hidden`, never saved.
    #[serde(skip)]
    pub frozen: bool,
}

fn first_layer() -> u32 {
    1
}

/// Whether the items with these ids can be grouped: they are at least two things, where a whole
/// group counts as one thing (a single group, however many shapes it holds, is nothing to group).
pub fn can_group(items: &[Item], ids: &[u64]) -> bool {
    let mut units: Vec<(bool, u64)> = Vec::new();
    for item in items.iter().filter(|i| ids.contains(&i.id)) {
        let unit = match item.groups.last() {
            Some(group) => (true, *group),
            None => (false, item.id),
        };
        if !units.contains(&unit) {
            units.push(unit);
        }
    }
    units.len() >= 2
}

/// Put the items with these ids (and the rest of their outermost groups) into a new group
/// `group_id`. Groups inside groups work: a group that is grouped becomes part of the new one.
/// Returns whether anything was grouped (see [`can_group`]).
pub fn group_items(items: &mut [Item], ids: &[u64], group_id: u64) -> bool {
    if !can_group(items, ids) {
        return false;
    }
    let tops: Vec<u64> = items.iter().filter(|i| ids.contains(&i.id)).filter_map(|i| i.groups.last().copied()).collect();
    let members: Vec<usize> = (0..items.len()).filter(|&k| ids.contains(&items[k].id) || items[k].groups.last().is_some_and(|g| tops.contains(g))).collect();
    for k in members {
        items[k].groups.push(group_id);
    }
    true
}

/// Take the items with these ids (and the rest of their outermost groups) out of their outermost
/// group: the group is dissolved, a group inside it stays. Returns whether there was a group.
pub fn ungroup_items(items: &mut [Item], ids: &[u64]) -> bool {
    let tops: Vec<u64> = items.iter().filter(|i| ids.contains(&i.id)).filter_map(|i| i.groups.last().copied()).collect();
    for item in items.iter_mut() {
        if item.groups.last().is_some_and(|g| tops.contains(g)) {
            item.groups.pop();
        }
    }
    !tops.is_empty()
}

impl Item {
    /// Whether one of this shape's bound sizes uses the variable.
    pub fn uses_variable(&self, name: &str) -> bool {
        self.bound.iter().any(|(_, e)| crate::calc::names_in(e).iter().any(|n| n == name))
    }

    /// A variable was renamed: bound sizes use the new name. Returns whether anything changed.
    pub fn rename_variable(&mut self, from: &str, to: &str) -> bool {
        let used = self.uses_variable(from);
        if used {
            for (_, e) in &mut self.bound {
                *e = crate::calc::rename_in(e, from, to);
            }
        }
        used
    }

    /// A variable was deleted: sizes bound to it become plain numbers (they keep their value).
    /// Returns whether anything changed.
    pub fn release_variable(&mut self, name: &str) -> bool {
        let before = self.bound.len();
        self.bound.retain(|(_, e)| !crate::calc::names_in(e).iter().any(|n| n == name));
        self.bound.len() != before
    }

    pub fn new(id: u64, shape: impl Into<Shape>) -> Self {
        Item { id, shape: shape.into(), source: None, groups: vec![], bound: vec![], locked: (false, false), layer: 1, hidden: false, frozen: false }
    }
}

/// How far the shape is turned (degrees), for the classes that can be.
fn rotation_of(shape: &Shape) -> f64 {
    match shape {
        Shape::Rect(s) => s.rotation,
        Shape::Ellipse(s) => s.rotation,
        Shape::Polygon(s) => s.rotation,
        Shape::Text(s) => s.rotation,
        Shape::Circle(_) | Shape::Path(_) | Shape::Compound(_) => 0.0,
    }
}

/// The paths whose nodes can be edited, if the shape has any.
fn paths(shape: &Shape) -> Option<Vec<&Path>> {
    match shape {
        Shape::Path(p) => Some(vec![p]),
        Shape::Compound(c) => Some(c.paths.iter().collect()),
        _ => None,
    }
}

/// Rebuild the shape after changing its paths.
fn with_paths(shape: &Shape, f: impl FnOnce(&mut Vec<Path>) -> bool) -> Option<Shape> {
    let mut list: Vec<Path> = paths(shape)?.into_iter().cloned().collect();
    if !f(&mut list) {
        return None;
    }
    Some(match shape {
        Shape::Path(_) => Shape::Path(list.remove(0)),
        _ => Shape::Compound(Compound::new(list)),
    })
}

impl Editable for Item {
    fn id(&self) -> u64 {
        self.id
    }

    fn bounds(&self) -> Option<(Pt, Pt)> {
        self.shape.bounds()
    }

    fn hidden(&self) -> bool {
        self.hidden
    }

    fn group(&self) -> Option<u64> {
        self.groups.last().copied()
    }

    fn frozen(&self) -> bool {
        self.frozen
    }

    fn hit(&self, p: Pt, tol: f64) -> bool {
        if self.hidden || self.frozen {
            return false;
        }
        let contours = self.shape.contours();
        if contours.iter().any(|c| c.segments().any(|(a, b)| dist_to_segment(p, a, b) <= tol)) {
            return true;
        }
        // the centre mark of a rectangle, circle, ellipse or polygon picks it too (and moves it)
        let centre = match &self.shape {
            Shape::Rect(r) => Some(r.center),
            Shape::Circle(c) => Some(c.center),
            Shape::Ellipse(e) => Some(e.center),
            Shape::Polygon(g) => Some(g.center),
            _ => None,
        };
        if centre.is_some_and(|c| crate::geometry::dist(c, p) <= tol) {
            return true;
        }
        // like the original, a vector is picked on its line, so a selection box can start
        // inside a big outline; only text is picked anywhere in its box (letters are thin)
        matches!(self.shape, Shape::Text(_)) && self.bounds().is_some_and(|(lo, hi)| p[0] >= lo[0] && p[0] <= hi[0] && p[1] >= lo[1] && p[1] <= hi[1])
    }

    /// Whether `p` lies inside the shape's filled area (even-odd), e.g. for "click inside" tools.
    fn contains(&self, p: Pt) -> bool {
        self.shape.contours().iter().filter(|c| c.closed).filter(|c| point_in_ring(p, &c.pts)).count() % 2 == 1
    }

    fn translated(&self, dx: f64, dy: f64) -> Self {
        Item { shape: self.shape.translated(dx, dy), ..self.clone() }
    }

    fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Self {
        Item { shape: self.shape.scaled(anchor, sx, sy), ..self.clone() }
    }

    fn rotated(&self, about: Pt, deg: f64) -> Self {
        Item { shape: self.shape.rotated(about, deg), ..self.clone() }
    }

    /// Handles can't change a size that is tied to a formula. The sizes are the shape's own
    /// width and height, so a shape turned a quarter swaps the two; one turned any other way can't
    /// say which box side is which, so every side counts as locked.
    fn locked_axes(&self) -> (bool, bool) {
        if self.bound.is_empty() {
            return self.locked;
        }
        let (mut width, mut height) = (false, false);
        for (dim, _) in &self.bound {
            match dim {
                Dim::Width => width = true,
                Dim::Height => height = true,
                // one number sizes the whole shape
                Dim::Diameter | Dim::Radius | Dim::TextHeight => (width, height) = (true, true),
            }
        }
        let turn = rotation_of(&self.shape).rem_euclid(180.0);
        let (x, y) = if turn < 1e-6 || 180.0 - turn < 1e-6 {
            (width, height)
        } else if (turn - 90.0).abs() < 1e-6 {
            (height, width)
        } else {
            (width || height, width || height)
        };
        (x || self.locked.0, y || self.locked.1)
    }

    fn grips(&self) -> Vec<Grip> {
        self.shape.grips()
    }

    fn with_grip(&self, kind: GripKind, p: Pt) -> Self {
        Item { shape: self.shape.with_grip(kind, p), ..self.clone() }
    }

    fn snap_points(&self) -> Vec<(Pt, SnapKind)> {
        let mut out = vec![];
        if self.hidden || self.frozen {
            return out;
        }
        let centre = match &self.shape {
            Shape::Rect(r) => Some(r.center),
            Shape::Circle(c) => Some(c.center),
            Shape::Ellipse(e) => Some(e.center),
            Shape::Polygon(p) => Some(p.center),
            Shape::Text(t) => Some(t.center),
            Shape::Path(_) | Shape::Compound(_) => None,
        };
        out.extend(centre.map(|c| (c, SnapKind::Centre)));
        // corners and edge middles of shapes made of straight segments
        let straight = match &self.shape {
            Shape::Rect(r) => r.corner_radius <= 0.0,
            Shape::Polygon(_) | Shape::Path(_) | Shape::Compound(_) => true,
            _ => false,
        };
        if straight {
            for c in self.shape.contours() {
                out.extend(c.pts.iter().map(|p| (*p, SnapKind::Node)));
                out.extend(c.segments().map(|(a, b)| ([(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0], SnapKind::MidPoint)));
            }
        }
        out
    }

    fn nodes(&self) -> Option<Vec<Pt>> {
        Some(paths(&self.shape)?.into_iter().flat_map(|p| p.points.iter().copied()).collect())
    }

    fn keeps_proportions(&self) -> bool {
        matches!(self.shape, Shape::Circle(_))
    }

    /// A circle and an ellipse show a ring with side handles, a straight line (an open vector of
    /// two nodes) shows its two ends; everything else the box.
    fn selection_style(&self) -> crate::SelectionStyle {
        match &self.shape {
            Shape::Circle(_) | Shape::Ellipse(_) => crate::SelectionStyle::Ring,
            Shape::Path(p) if !p.closed && p.points.len() == 2 => crate::SelectionStyle::Ends,
            _ => crate::SelectionStyle::Box,
        }
    }

    fn segments(&self) -> Vec<(usize, usize)> {
        let Some(paths) = paths(&self.shape) else { return vec![] };
        let mut start = 0;
        let mut out = Vec::new();
        for p in paths {
            let n = p.points.len();
            let m = if p.closed { n } else { n.saturating_sub(1) };
            out.extend((0..m).map(|i| (start + i, start + (i + 1) % n)));
            start += n;
        }
        out
    }

    fn with_node(&self, index: usize, p: Pt) -> Self {
        let shape = with_paths(&self.shape, |list| {
            let Some((pi, ni)) = Compound::new(list.clone()).locate(index) else { return false };
            list[pi].points[ni] = p;
            true
        });
        Item { shape: shape.unwrap_or_else(|| self.shape.clone()), ..self.clone() }
    }

    fn with_inserted_node(&self, after: usize, p: Pt) -> Self {
        let shape = with_paths(&self.shape, |list| {
            let Some((pi, ni)) = Compound::new(list.clone()).locate(after) else { return false };
            list[pi].points.insert(ni + 1, p);
            true
        });
        Item { shape: shape.unwrap_or_else(|| self.shape.clone()), ..self.clone() }
    }

    fn without_node(&self, index: usize) -> Option<Self> {
        let shape = with_paths(&self.shape, |list| {
            let Some((pi, ni)) = Compound::new(list.clone()).locate(index) else { return false };
            if list[pi].points.len() <= list[pi].min_nodes() {
                return false;
            }
            list[pi].points.remove(ni);
            true
        })?;
        Some(Item { shape, ..self.clone() })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bound_sizes_follow_renames_and_are_released_on_delete() {
        let mut item = Item::new(1, Rect::new([0.0, 0.0], 20.0, 10.0));
        item.bound.push((Dim::Width, "breite - 2*rand".into()));
        item.bound.push((Dim::Height, "hoehe".into()));
        assert!(item.rename_variable("rand", "rahmen"));
        assert_eq!(item.bound[0].1, "breite - 2*rahmen");
        assert!(!item.rename_variable("nix", "x"));
        assert!(item.release_variable("hoehe"));
        assert_eq!(item.bound.len(), 1, "only the size using the deleted variable is released");
    }

    use super::*;
    use crate::forms::{Circle, Rect, Text};
    use crate::{Editor, Started, Tool, View};

    const VIEW: View = View { tolerance: 1.0, handle_pad: 0.0 };

    #[test]
    fn hit_outline_inside_and_holes() {
        let ring = Item::new(
            1,
            Compound::new(vec![
                Path::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]], true),
                Path::new(vec![[3.0, 3.0], [7.0, 3.0], [7.0, 7.0], [3.0, 7.0]], true),
            ]),
        );
        assert!(!ring.hit([1.5, 1.5], 0.1), "inside is not the line");
        assert!(ring.contains([1.5, 1.5]), "but it is inside");
        assert!(!ring.contains([5.0, 5.0]), "inside the hole");
        assert!(ring.hit([5.0, 3.05], 0.1), "on the hole's edge");
        assert!(ring.hit([10.05, 5.0], 0.1), "on the outer edge");
        assert!(Item::new(2, Text::new([0.0, 0.0], "I I", 10.0)).hit([0.0, 0.0], 0.1), "text box");
    }

    #[test]
    fn snap_points_per_class() {
        let kinds = |i: &Item, k: SnapKind| i.snap_points().iter().filter(|(_, kk)| *kk == k).count();
        let rect = Item::new(1, Rect::new([5.0, 5.0], 10.0, 4.0));
        assert_eq!((kinds(&rect, SnapKind::Centre), kinds(&rect, SnapKind::Node), kinds(&rect, SnapKind::MidPoint)), (1, 4, 4));
        let circle = Item::new(2, Circle::new([1.0, 2.0], 3.0));
        assert_eq!(circle.snap_points(), vec![([1.0, 2.0], SnapKind::Centre)]);
        let open = Item::new(3, Path::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 2.0]], false));
        assert_eq!((kinds(&open, SnapKind::Node), kinds(&open, SnapKind::MidPoint), kinds(&open, SnapKind::Centre)), (3, 2, 0));
    }

    fn dots(n: u64) -> Vec<Item> {
        (1..=n).map(|id| Item::new(id, Circle::new([id as f64 * 10.0, 0.0], 2.0))).collect()
    }

    #[test]
    fn a_click_on_a_member_picks_the_whole_group() {
        let mut items = dots(4);
        assert!(group_items(&mut items, &[1, 2], 100));
        let mut ed = Editor::default();
        assert!(matches!(ed.press(&items, [20.0, 2.0], VIEW, Tool::Select, false), Started::Move));
        ed.release(&items);
        let mut picked = ed.selection.clone();
        picked.sort();
        assert_eq!(picked, vec![1, 2], "one member brings the other");

        // a shape of its own is picked alone, and a band picks a group when it takes one member
        assert!(matches!(ed.press(&items, [30.0, 2.0], VIEW, Tool::Select, false), Started::Move));
        ed.release(&items);
        assert_eq!(ed.selection, vec![3]);
        ed.press(&items, [5.0, 20.0], VIEW, Tool::Select, false);
        ed.drag(&mut items, [15.0, -5.0], false);
        ed.release(&items);
        let mut picked = ed.selection.clone();
        picked.sort();
        assert_eq!(picked, vec![1, 2]);

        // shift-click on a member of the selected group takes the whole group out again
        assert!(matches!(ed.press(&items, [10.0, 2.0], VIEW, Tool::Select, true), Started::Move));
        ed.release(&items);
        assert!(ed.selection.is_empty());
    }

    #[test]
    fn a_click_on_one_of_several_selected_shapes_picks_that_one_alone() {
        let mut items = dots(3);
        let mut ed = Editor::default();
        ed.selection = vec![1, 2, 3];
        ed.press(&items, [10.0, 2.0], VIEW, Tool::Select, false);
        ed.release(&items);
        assert_eq!(ed.selection, vec![1], "a click without moving narrows the selection");

        // dragging one of several selected shapes still moves them all and keeps the selection
        ed.selection = vec![1, 2, 3];
        ed.press(&items, [10.0, 2.0], VIEW, Tool::Select, false);
        ed.drag(&mut items, [15.0, 2.0], false);
        ed.release(&items);
        assert_eq!(ed.selection, vec![1, 2, 3]);

        // in a group the click narrows to the group
        let mut items = dots(3);
        group_items(&mut items, &[1, 2], 100);
        ed.selection = vec![1, 2, 3];
        ed.press(&items, [10.0, 2.0], VIEW, Tool::Select, false);
        ed.release(&items);
        ed.selection.sort();
        assert_eq!(ed.selection, vec![1, 2]);

        // a shift-click still adds and removes
        ed.press(&items, [30.0, 2.0], VIEW, Tool::Select, true);
        ed.release(&items);
        ed.selection.sort();
        assert_eq!(ed.selection, vec![1, 2, 3]);
    }

    #[test]
    fn groups_nest_and_ungroup_one_level_at_a_time() {
        let mut items = dots(4);
        assert!(group_items(&mut items, &[1, 2], 100));
        // the group and a loose shape make a bigger group
        assert!(group_items(&mut items, &[1, 3], 200));
        assert_eq!(items[0].groups, vec![100, 200]);
        assert_eq!(items[1].groups, vec![100, 200], "the rest of the group came along");
        assert_eq!(items[2].groups, vec![200]);
        assert!(items[3].groups.is_empty());
        // one group already: nothing to do
        assert!(!group_items(&mut items, &[1, 2, 3], 300));
        assert!(!group_items(&mut items, &[4], 300), "one shape alone");

        assert!(ungroup_items(&mut items, &[3]));
        assert_eq!((items[0].groups.clone(), items[1].groups.clone(), items[2].groups.clone()), (vec![100], vec![100], vec![]));
        assert!(ungroup_items(&mut items, &[1]));
        assert!(items.iter().all(|i| i.groups.is_empty()));
        assert!(!ungroup_items(&mut items, &[1, 2, 3]), "nothing left to ungroup");
    }

    #[test]
    fn grouping_needs_two_things_and_ungrouping_takes_every_selected_group_apart() {
        let mut items = dots(5);
        assert!(!can_group(&items, &[1]), "one shape");
        assert!(can_group(&items, &[1, 2]));
        group_items(&mut items, &[1, 2], 100);
        group_items(&mut items, &[3, 4], 200);
        assert!(!can_group(&items, &[1, 2]), "a whole group is one thing");
        assert!(!can_group(&items, &[1, 2, 99]), "an id that is not there counts for nothing");
        assert!(can_group(&items, &[1, 2, 5]), "a group and a shape");
        assert!(can_group(&items, &[1, 3]), "two groups");

        // two groups selected: both are dissolved, the shape outside any group is left as it is
        assert!(ungroup_items(&mut items, &[1, 2, 3, 4, 5]));
        assert!(items.iter().all(|i| i.groups.is_empty()));
    }

    #[test]
    fn compound_nodes_span_paths() {
        let mut items = vec![Item::new(
            1,
            Compound::new(vec![Path::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]], true), Path::new(vec![[5.0, 5.0], [6.0, 5.0]], false)]),
        )];
        assert_eq!(items[0].nodes().unwrap().len(), 5);
        assert_eq!(items[0].segments(), vec![(0, 1), (1, 2), (2, 0), (3, 4)]);
        let moved = items[0].with_node(4, [9.0, 9.0]);
        assert_eq!(moved.nodes().unwrap()[4], [9.0, 9.0]);
        // the triangle can't lose a corner, the open path can't lose its last two
        assert!(items[0].without_node(0).is_none());
        assert!(items[0].without_node(3).is_none());

        let mut ed = Editor::default();
        ed.selection = vec![1];
        assert!(ed.insert_node(&mut items, [5.5, 5.0], VIEW));
        assert_eq!(items[0].nodes().unwrap().len(), 6);
        assert!(ed.delete_active_node(&mut items));
        assert_eq!(items[0].nodes().unwrap().len(), 5);
    }

    #[test]
    fn selected_decoration_grips_per_class() {
        use crate::forms::{Circle, GripKind};
        use crate::Outcome;
        let mut items = vec![Item::new(1, Rect::new([0.0, 0.0], 20.0, 10.0)), Item::new(2, Circle::new([50.0, 0.0], 10.0))];
        let mut ed = Editor::default();
        assert!(ed.grips(&items).is_empty(), "nothing selected, no grips");

        // the rectangle's corner-radius grip: drag it 3 mm further in along the diagonal
        ed.selection = vec![1];
        let g = ed.grips(&items)[0];
        assert_eq!(g.kind, GripKind::CornerRadius);
        assert_eq!(ed.press(&items, g.at, VIEW, Tool::Select, false), Started::Grip(GripKind::CornerRadius));
        ed.drag(&mut items, [g.at[0] - 3.0, g.at[1] - 3.0], false);
        assert_eq!(ed.release(&items), Outcome::Reshaped);
        let Shape::Rect(r) = &items[0].shape else { panic!("still a rectangle") };
        assert!((r.corner_radius - 3.0).abs() < 1e-9, "radius {}", r.corner_radius);
        assert_eq!((r.width, r.height), (20.0, 10.0), "size untouched");
        // and the grip moved with the radius
        assert!((ed.grips(&items)[0].at[0] - (g.at[0] - 3.0)).abs() < 1e-9);

        // a circle has no extra handle
        ed.selection = vec![2];
        assert!(ed.grips(&items).is_empty());

        // two selected shapes, or the rotate mode: no grips
        ed.selection = vec![1, 2];
        assert!(ed.grips(&items).is_empty());
        ed.selection = vec![2];
        ed.rotate_mode = true;
        assert!(ed.grips(&items).is_empty());
    }

    #[test]
    fn a_size_tied_to_a_formula_cannot_be_dragged() {
        // width is bound: the side handles do nothing, the top/bottom ones still work, and a
        // corner only moves the free direction
        let mut rect = Item::new(1, Rect::new([5.0, 5.0], 10.0, 10.0));
        rect.bound.push((Dim::Width, "breite".into()));
        assert_eq!(rect.locked_axes(), (true, false));
        let mut items = vec![rect];
        let mut ed = Editor::default();
        ed.selection = vec![1];
        assert_eq!(ed.locked_axes(&items), (true, false));

        assert_eq!(ed.press(&items, [0.0, 5.0], VIEW, Tool::Select, false), Started::Nothing, "left handle is inert");
        assert_eq!(ed.press(&items, [10.0, 5.0], VIEW, Tool::Select, false), Started::Nothing, "right handle too");

        assert!(matches!(ed.press(&items, [5.0, 10.0], VIEW, Tool::Select, false), Started::Resize(_)), "top handle works");
        ed.drag(&mut items, [5.0, 14.0], false);
        ed.release(&items);
        let Shape::Rect(r) = &items[0].shape else { panic!("still a rectangle") };
        assert_eq!((r.width, r.height), (10.0, 14.0));

        // the top-right corner: the height follows the pointer, the width stays
        assert!(matches!(ed.press(&items, [10.0, 14.0], VIEW, Tool::Select, false), Started::Resize(_)));
        ed.drag(&mut items, [20.0, 19.0], false);
        ed.release(&items);
        let Shape::Rect(r) = &items[0].shape else { panic!("still a rectangle") };
        assert_eq!(r.width, 10.0, "width is locked");
        assert!(r.height > 14.0, "height still moves: {}", r.height);

        // one number that sizes the whole shape locks both directions
        let mut circle = Item::new(2, Circle::new([0.0, 0.0], 3.0));
        circle.bound.push((Dim::Diameter, "d".into()));
        assert_eq!(circle.locked_axes(), (true, true));
        // a quarter turn swaps the sides: "width" now runs up the page
        let mut turned = Item::new(3, Rect { rotation: 90.0, ..Rect::new([0.0, 0.0], 10.0, 4.0) });
        turned.bound.push((Dim::Width, "b".into()));
        assert_eq!(turned.locked_axes(), (false, true));
        // free shapes are not locked
        assert_eq!(Item::new(4, Rect::new([0.0, 0.0], 1.0, 1.0)).locked_axes(), (false, false));
    }


    #[test]
    fn editor_resizes_real_shapes_from_the_left() {
        let mut items = vec![Item::new(1, Rect::new([5.0, 5.0], 10.0, 10.0)), Item::new(2, Circle::new([50.0, 50.0], 2.0))];
        let mut ed = Editor::default();
        ed.selection = vec![1];
        assert!(matches!(ed.press(&items, [0.0, 5.0], VIEW, Tool::Select, false), Started::Resize(_)));
        ed.drag(&mut items, [-10.0, 5.0], false);
        ed.release(&items);
        let Shape::Rect(r) = &items[0].shape else { panic!("still a rectangle") };
        assert_eq!((r.width, r.height, r.center), (20.0, 10.0, [0.0, 5.0]));
        assert_eq!(items[1].shape, Shape::Circle(Circle::new([50.0, 50.0], 2.0)), "unselected shape untouched");
    }

    #[test]
    fn each_class_chooses_how_its_selection_shows() {
        use crate::forms::{Circle, Ellipse, Polygon};
        use crate::{handle_point, Handle, Outcome, SelectionStyle};
        let items = vec![
            Item::new(1, Rect::new([0.0, 0.0], 20.0, 10.0)),
            Item::new(2, Circle::new([50.0, 0.0], 8.0)),
            Item::new(3, Ellipse::new([100.0, 0.0], 9.0, 4.0)),
            Item::new(4, Path::new(vec![[0.0, 40.0], [30.0, 50.0]], false)),
            Item::new(5, Path::new(vec![[0.0, 80.0], [30.0, 80.0], [30.0, 100.0]], false)),
            Item::new(6, Polygon::new([100.0, 80.0], 8.0, 6)),
        ];
        let style = |ids: &[u64]| {
            let mut ed = Editor::default();
            ed.selection = ids.to_vec();
            ed.style(&items)
        };
        assert_eq!(style(&[1]), SelectionStyle::Box);
        assert_eq!(style(&[2]), SelectionStyle::Ring);
        assert_eq!(style(&[3]), SelectionStyle::Ring);
        assert_eq!(style(&[4]), SelectionStyle::Ends, "a straight line");
        assert_eq!(style(&[5]), SelectionStyle::Box, "a vector of three nodes");
        assert_eq!(style(&[6]), SelectionStyle::Box);
        assert_eq!(style(&[2, 4]), SelectionStyle::Box, "several shapes: the box");

        // a ring answers only on its four sides, a line has no box handles but its two ends
        let mut ed = Editor::default();
        ed.selection = vec![2];
        assert_eq!(ed.visible_handles(&items).len(), 4);
        let corner = handle_point([42.0, -8.0], [58.0, 8.0], VIEW.handle_pad, Handle::TopRight);
        assert!(ed.handle_under(&items, corner, VIEW).is_none(), "the corner of a circle's box does nothing");
        let side = handle_point([42.0, -8.0], [58.0, 8.0], VIEW.handle_pad, Handle::Right);
        assert_eq!(ed.handle_under(&items, side, VIEW), Some(Handle::Right));

        let mut items = items;
        ed.selection = vec![4];
        assert!(ed.visible_handles(&items).is_empty());
        assert_eq!(ed.end_handles(&items).len(), 2);
        // pressing on the end starts moving that end; the other stays
        assert_eq!(ed.press(&items, [30.0, 50.0], VIEW, Tool::Select, false), Started::Node);
        ed.drag(&mut items, [34.0, 56.0], false);
        assert_eq!(ed.release(&items), Outcome::NodeMoved);
        assert_eq!(items[3].nodes().unwrap(), vec![[0.0, 40.0], [34.0, 56.0]]);
        // a circle stays a circle when a side handle is dragged
        let mut circles = vec![Item::new(1, Circle::new([0.0, 0.0], 8.0))];
        let mut ed2 = Editor::default();
        ed2.selection = vec![1];
        assert_eq!(ed2.press(&circles, [8.0 + VIEW.handle_pad, 0.0], VIEW, Tool::Select, false), Started::Resize(Handle::Right));
        ed2.drag(&mut circles, [8.0 + VIEW.handle_pad + 4.0, 0.0], false);
        ed2.release(&circles);
        let Shape::Circle(c) = &circles[0].shape else { panic!("still a circle, not an ellipse") };
        assert!((c.radius - 10.0).abs() < 1e-6, "radius {}", c.radius);
        // the middle of the line moves the whole line
        assert_eq!(ed.press(&items, [17.0, 48.0], VIEW, Tool::Select, false), Started::Move);
    }
}
