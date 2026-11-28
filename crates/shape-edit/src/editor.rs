//! The gesture state machine: press, drag, release (plus click-free actions such as nudging
//! and node insertion). All positions are world coordinates.

use crate::geometry::dist;
use crate::handles::{anchor_for, handle_at, handle_point, selection_bounds, Handle};
use crate::snap::{MoveSnap, SnapKind, Snapping};
use crate::{Editable, Grip, GripKind, Pt, SelectionStyle};

/// Smallest scale factor a resize may reach, so a shape can't collapse or flip inside out.
const MIN_SCALE: f64 = 0.01;

/// Which kind of editing the pointer does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    /// Select, move and resize whole shapes.
    Select,
    /// Drag, insert and delete the nodes of one shape.
    Nodes,
}

/// How big things are on screen, in world units (the host divides pixels by its zoom).
#[derive(Clone, Copy, Debug)]
pub struct View {
    /// How close the pointer must be to pick a shape, handle or node.
    pub tolerance: f64,
    /// How far the selection box (and its handles) sits outside the selection bounds.
    pub handle_pad: f64,
}

/// The gesture in progress, with snapshots of what it changes.
#[derive(Clone, Debug)]
pub enum Gesture<S> {
    Move { start: Pt, orig: Vec<S> },
    /// `grip` is the box corner/edge the handle stands for, `start` where the pointer went down
    /// (a little outside it, on the drawn handle): only the movement since then counts.
    Resize { handle: Handle, anchor: Pt, grip: Pt, start: Pt, orig: Vec<S> },
    Node { index: usize, orig: S },
    /// Dragging one segment of an open vector parallel to itself: `index` is its place in
    /// [`Editable::segments`], `start` where the pointer went down. Both its nodes move by the
    /// pointer's movement across the segment; movement along it is ignored.
    Segment { index: usize, start: Pt, orig: S },
    Band { start: Pt, end: Pt, additive: bool },
    /// Turning the selection about `center`; `angle` is the current turn in degrees.
    Rotate { center: Pt, start: Pt, angle: f64, orig: Vec<S> },
    /// Dragging one of the shape's own grips; `offset` is grip minus press point (no jump).
    Grip { kind: GripKind, offset: Pt, orig: S },
}

/// What a press started. The host saves an undo step for the ones that may change shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Started {
    Move,
    Resize(Handle),
    Node,
    /// A segment of an open vector, to be dragged parallel to itself.
    Segment,
    Rotate,
    Grip(GripKind),
    Band,
    /// Only the selection changed (or nothing at all).
    Nothing,
}

impl Started {
    /// Whether this gesture may change the shapes (the host keeps an undo step for it).
    pub fn edits(self) -> bool {
        matches!(self, Started::Move | Started::Resize(_) | Started::Node | Started::Segment | Started::Rotate | Started::Grip(_))
    }
}

/// How a gesture ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Moved,
    Resized,
    NodeMoved,
    Rotated,
    /// One of a shape's own grips was dragged (e.g. a rectangle's corner radius).
    Reshaped,
    /// The pointer didn't move far enough to change a shape; only the selection may differ.
    Selected,
    Unchanged,
}

impl Outcome {
    pub fn changed_shapes(self) -> bool {
        matches!(self, Outcome::Moved | Outcome::Resized | Outcome::NodeMoved | Outcome::Rotated | Outcome::Reshaped)
    }
}

/// Selection plus the gesture in progress.
#[derive(Clone, Debug)]
pub struct Editor<S> {
    pub selection: Vec<u64>,
    /// In node mode, or on a selected open vector: the shape id and node index last picked.
    pub active_node: Option<(u64, usize)>,
    /// On a selected open vector: the shape id and segment (its place in [`Editable::segments`])
    /// last picked. A node and a segment are never picked at once.
    pub active_segment: Option<(u64, usize)>,
    /// Shapes on this layer are picked first, even if another shape is on top.
    pub priority_layer: u32,
    /// Snapping used while moving, resizing and dragging nodes.
    pub snapping: Snapping,
    /// What the pointer snapped to during the current gesture (for a marker on screen).
    pub last_snap: Option<(Pt, SnapKind)>,
    /// When a Move drag aligned the shape to another on one or both axes: the caught column (x) and
    /// row (y), each drawn as an alignment guide. Independent of `last_snap` (which marks a point).
    pub move_snap: Option<MoveSnap>,
    /// The corner handles turn the selection instead of resizing it. Clicking a selected shape
    /// again switches this on and off ("click again to transform", as in the original).
    pub rotate_mode: bool,
    /// Angle steps in degrees while the step key is held during a turn.
    pub angle_step: f64,
    gesture: Option<Gesture<S>>,
    /// The press landed on a shape that was already selected: a click without moving toggles
    /// the rotate mode.
    clicked_selected: bool,
    /// Snap targets of the shapes the gesture doesn't change, gathered when it starts.
    targets: Vec<(Pt, SnapKind)>,
    /// Bounding boxes of the shapes and groups the gesture doesn't change (for the gap policy).
    target_boxes: Vec<(Pt, Pt)>,
    moved: bool,
    toggle_on_release: Option<u64>,
    /// The press landed on one of several selected shapes: if it ends without moving, the click
    /// narrows the selection to that shape (and its group).
    narrow_on_release: Option<u64>,
    /// The press landed on a segment of a wholly selected open vector: the drag moves the whole
    /// selection, but if it ends without moving the click picks just that segment (shape, segment).
    pick_segment_on_release: Option<(u64, usize)>,
}

impl<S> Default for Editor<S> {
    fn default() -> Self {
        Editor {
            selection: vec![],
            active_node: None,
            active_segment: None,
            priority_layer: 0,
            snapping: Snapping::default(),
            last_snap: None,
            move_snap: None,
            rotate_mode: false,
            angle_step: 15.0,
            gesture: None,
            clicked_selected: false,
            targets: vec![],
            target_boxes: vec![],
            moved: false,
            toggle_on_release: None,
            narrow_on_release: None,
            pick_segment_on_release: None,
        }
    }
}

fn find<S: Editable>(shapes: &[S], id: u64) -> Option<&S> {
    shapes.iter().find(|s| s.id() == id)
}

fn put<S: Editable>(shapes: &mut [S], s: S) {
    if let Some(slot) = shapes.iter_mut().find(|x| x.id() == s.id()) {
        *slot = s;
    }
}

/// The snap anchors of a bounding box `lo..hi`: its four corners (Node), the middle of each edge
/// (MidPoint) and its centre (Centre). Used so a group snaps as one block.
fn bbox_snap_points(lo: Pt, hi: Pt) -> Vec<(Pt, SnapKind)> {
    let (mx, my) = ((lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0);
    vec![
        ([lo[0], lo[1]], SnapKind::Node),
        ([hi[0], lo[1]], SnapKind::Node),
        ([hi[0], hi[1]], SnapKind::Node),
        ([lo[0], hi[1]], SnapKind::Node),
        ([mx, lo[1]], SnapKind::MidPoint),
        ([mx, hi[1]], SnapKind::MidPoint),
        ([lo[0], my], SnapKind::MidPoint),
        ([hi[0], my], SnapKind::MidPoint),
        ([mx, my], SnapKind::Centre),
    ]
}

/// The segment of `s` nearest `p` within the pick tolerance (its place in [`Editable::segments`]).
fn segment_of<S: Editable>(s: &S, p: Pt, view: View) -> Option<usize> {
    s.segments()
        .into_iter()
        .enumerate()
        .map(|(k, (a, b))| (k, s.segment_distance(p, a, b)))
        .filter(|(_, d)| *d <= view.tolerance)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(k, _)| k)
}

/// The unit normal of the segment a→b (its direction turned a quarter anticlockwise), or `None`
/// for a segment of no length.
fn unit_normal(a: Pt, b: Pt) -> Option<Pt> {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = (dx * dx + dy * dy).sqrt();
    (len > 1e-12).then(|| [-dy / len, dx / len])
}

/// The directions any of the shapes locks against resizing (see [`Editable::locked_axes`]).
fn locked_of<S: Editable>(shapes: &[S]) -> (bool, bool) {
    shapes.iter().map(Editable::locked_axes).fold((false, false), |(x, y), (lx, ly)| (x || lx, y || ly))
}

/// How far `p` has moved `grip` away from `anchor`, as a positive scale factor.
fn ratio(p: f64, anchor: f64, grip: f64) -> f64 {
    let span = grip - anchor;
    if span.abs() < 1e-12 {
        return 1.0;
    }
    ((p - anchor) / span).max(MIN_SCALE)
}

impl<S: Editable> Editor<S> {
    pub fn gesture(&self) -> Option<&Gesture<S>> {
        self.gesture.as_ref()
    }

    pub fn is_busy(&self) -> bool {
        self.gesture.is_some()
    }

    /// Snap targets of every shape except those in `skip`: each shape's own geometry, plus — for a
    /// group — the corners, edge middles and centre of the whole group's bounding box, so a shape
    /// snaps to a group as one block, not only to its members.
    pub fn targets(shapes: &[S], skip: &[u64]) -> Vec<(Pt, SnapKind)> {
        let mut out: Vec<(Pt, SnapKind)> = shapes.iter().filter(|s| !skip.contains(&s.id())).flat_map(Editable::snap_points).collect();
        for (_, lo, hi) in Self::group_boxes(shapes, skip) {
            out.extend(bbox_snap_points(lo, hi));
        }
        out
    }

    /// Combined bounding box of each group among the non-skipped, visible shapes: `(group id, lo, hi)`.
    fn group_boxes(shapes: &[S], skip: &[u64]) -> Vec<(u64, Pt, Pt)> {
        let mut groups: Vec<(u64, Pt, Pt)> = vec![];
        for s in shapes.iter().filter(|s| !skip.contains(&s.id()) && !s.hidden()) {
            let (Some(g), Some((lo, hi))) = (s.group(), s.bounds()) else { continue };
            match groups.iter_mut().find(|(id, _, _)| *id == g) {
                Some((_, glo, ghi)) => {
                    *glo = [glo[0].min(lo[0]), glo[1].min(lo[1])];
                    *ghi = [ghi[0].max(hi[0]), ghi[1].max(hi[1])];
                }
                None => groups.push((g, lo, hi)),
            }
        }
        groups
    }

    /// The bounding box of every "block" except those in `skip`, for the gap (equal-spacing) policy:
    /// a whole group counts as one box, and every loose shape as its own box.
    pub fn target_boxes(shapes: &[S], skip: &[u64]) -> Vec<(Pt, Pt)> {
        let mut out: Vec<(Pt, Pt)> = Self::group_boxes(shapes, skip).into_iter().map(|(_, lo, hi)| (lo, hi)).collect();
        for s in shapes.iter().filter(|s| !skip.contains(&s.id()) && !s.hidden() && s.group().is_none()) {
            if let Some((lo, hi)) = s.bounds() {
                out.push((lo, hi));
            }
        }
        out
    }

    /// Snap targets for MOVING a shape: the bounding-box points (corners, edge middles, centre) of
    /// every block — each loose shape and each whole group. Bounding boxes, not raw facet nodes, so
    /// snapping lands edges exactly and is the same at every zoom (a facetted circle has dozens of
    /// nodes; snapping to one of them, not its extent, left a gap when zoomed out — the user's bug).
    ///
    /// Plus each shape's own centre where that is not the middle of its box (an arc's circle
    /// centre, which can sit outside the box): a real, zoom-stable anchor a move should catch.
    pub fn move_targets(shapes: &[S], skip: &[u64]) -> Vec<(Pt, SnapKind)> {
        let mut out: Vec<(Pt, SnapKind)> = Self::target_boxes(shapes, skip).into_iter().flat_map(|(lo, hi)| bbox_snap_points(lo, hi)).collect();
        out.extend(Self::offbox_centres(shapes, skip).into_iter().map(|c| (c, SnapKind::Centre)));
        out
    }

    /// The real centres of the non-skipped, visible shapes that do not sit at the middle of their
    /// bounding box (an arc's circle centre). These are extra move anchors on top of the box points.
    fn offbox_centres(shapes: &[S], skip: &[u64]) -> Vec<Pt> {
        shapes
            .iter()
            .filter(|s| !skip.contains(&s.id()) && !s.hidden())
            .filter_map(|s| match (s.centre_point(), s.bounds()) {
                (Some(c), Some((lo, hi))) => {
                    let box_mid = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
                    (dist(c, box_mid) > 1e-6).then_some(c)
                }
                _ => None,
            })
            .collect()
    }

    /// Where `p` snaps to among all shapes (for drawing tools, outside a gesture).
    pub fn snap_free(&self, shapes: &[S], p: Pt) -> (Pt, Option<SnapKind>) {
        self.snapping.snap(p, &Self::targets(shapes, &[]))
    }

    /// Snap a pointer position during a gesture, remembering what it snapped to.
    fn snapped(&mut self, p: Pt) -> Pt {
        let (q, kind) = self.snapping.snap(p, &self.targets);
        self.last_snap = kind.map(|k| (q, k));
        q
    }

    /// The rubber band being dragged, as (corner, corner).
    pub fn band(&self) -> Option<(Pt, Pt)> {
        match self.gesture {
            Some(Gesture::Band { start, end, .. }) => Some((start, end)),
            _ => None,
        }
    }

    /// The directions of the selection box the resize handles can't change because a shape has a
    /// size tied to a formula: (horizontal, vertical). For drawing those sides differently.
    pub fn locked_axes(&self, shapes: &[S]) -> (bool, bool) {
        locked_of(&self.selected(shapes))
    }

    /// How the selection shows: the style of the one selected shape, the box for anything else.
    pub fn style(&self, shapes: &[S]) -> SelectionStyle {
        match self.selection.as_slice() {
            [id] if !self.rotate_mode => find(shapes, *id).map_or(SelectionStyle::Box, Editable::selection_style),
            _ => SelectionStyle::Box,
        }
    }

    /// The resize handles the selection shows (and answers to): all eight round a box, the four
    /// on the sides of a ring, none on a line (which has its ends instead).
    pub fn visible_handles(&self, shapes: &[S]) -> Vec<Handle> {
        match self.style(shapes) {
            SelectionStyle::Box => Handle::ALL.to_vec(),
            SelectionStyle::Ring => vec![Handle::Top, Handle::Right, Handle::Bottom, Handle::Left],
            SelectionStyle::Polyline => vec![],
        }
    }

    /// The selected open vector, when the selection shows as one (see [`SelectionStyle::Polyline`]).
    fn polyline<'a>(&self, shapes: &'a [S]) -> Option<&'a S> {
        if self.style(shapes) != SelectionStyle::Polyline {
            return None;
        }
        self.selection.first().and_then(|id| find(shapes, *id))
    }

    /// The node handles of a selected open vector — one on every node: (shape, node index, place).
    pub fn node_handles(&self, shapes: &[S]) -> Vec<(u64, usize, Pt)> {
        let Some(s) = self.polyline(shapes) else { return vec![] };
        s.nodes().unwrap_or_default().into_iter().enumerate().map(|(i, at)| (s.id(), i, at)).collect()
    }

    /// The node handle of a selected open vector under `p`, if any.
    pub fn node_handle_under(&self, shapes: &[S], p: Pt, view: View) -> Option<(u64, usize)> {
        self.node_handles(shapes).into_iter().filter(|(_, _, at)| dist(*at, p) <= view.tolerance).min_by(|a, b| dist(a.2, p).total_cmp(&dist(b.2, p))).map(|(id, i, _)| (id, i))
    }

    /// The segment of a selected open vector under `p` (its place in [`Editable::segments`]), if
    /// any. Nodes come first: a press near a node is for the node, not for its segments.
    pub fn segment_under(&self, shapes: &[S], p: Pt, view: View) -> Option<(u64, usize)> {
        let s = self.polyline(shapes)?;
        segment_of(s, p, view).map(|k| (s.id(), k))
    }

    /// Whether the whole of the selected open vector is selected (by a box, a click on an
    /// unpicked shape, …), not one of its nodes or segments: a drag on it then moves everything.
    pub fn polyline_whole(&self, shapes: &[S]) -> bool {
        self.polyline(shapes).is_some() && self.active_node.is_none() && self.active_segment.is_none()
    }

    /// Segment `index` of `s` as its two node indices and its two ends now.
    pub fn segment_ends(s: &S, index: usize) -> Option<(usize, usize, Pt, Pt)> {
        let (a, b) = *s.segments().get(index)?;
        let nodes = s.nodes()?;
        Some((a, b, *nodes.get(a)?, *nodes.get(b)?))
    }

    /// `s` with segment `index` moved parallel to itself by `d` along its normal, keeping the
    /// angle of every segment: the segment's line is shifted first, then each of its ends is put
    /// where that line meets the (unchanged) line of the neighbouring segment — so the moved
    /// segment and its neighbours change length, never direction. An end with no neighbour (the
    /// start or end of an open vector) simply goes across with the line; a neighbour that runs
    /// parallel to the segment has no meeting point, and its shared end goes across too.
    /// `None` when the shift would turn the segment or a neighbour round (shrink it through zero
    /// length): no angle may flip, so such a shift is refused.
    pub fn moved_parallel(s: &S, index: usize, d: f64) -> Option<S> {
        let (ia, ib, a, b) = Self::segment_ends(s, index)?;
        let n = unit_normal(a, b)?;
        let nodes = s.nodes()?;
        let segments = s.segments();
        let u = [b[0] - a[0], b[1] - a[1]];
        // a point of the shifted line
        let on_line = [a[0] + n[0] * d, a[1] + n[1] * d];
        // where the shifted line meets the neighbour that shares `end` (its far node `far` stays):
        // the end moves along the neighbour's own line, so the neighbour keeps its direction
        let meet = |end: usize, end_at: Pt| -> Option<Pt> {
            let shifted = [end_at[0] + n[0] * d, end_at[1] + n[1] * d];
            let far = segments.iter().enumerate().filter(|(k, _)| *k != index).find_map(|(_, &(x, y))| match (x == end, y == end) {
                (true, false) => Some(y),
                (false, true) => Some(x),
                _ => None,
            });
            let Some(far) = far.and_then(|f| nodes.get(f).copied()) else { return Some(shifted) };
            let v = [end_at[0] - far[0], end_at[1] - far[1]];
            let cross = u[0] * v[1] - u[1] * v[0];
            if cross.abs() < 1e-12 * (u[0].hypot(u[1]) * v[0].hypot(v[1])).max(1e-300) {
                return Some(shifted);
            }
            // far + t·v lies on the shifted line: (far + t·v − on_line) · n = 0
            let t = ((on_line[0] - far[0]) * n[0] + (on_line[1] - far[1]) * n[1]) / (v[0] * n[0] + v[1] * n[1]);
            // t ≤ 0 would put the end at or behind the far node: the neighbour would turn round
            (t > 1e-9).then(|| [far[0] + v[0] * t, far[1] + v[1] * t])
        };
        let a2 = meet(ia, a)?;
        let b2 = meet(ib, b)?;
        // the moved segment itself must keep its sense too
        if (b2[0] - a2[0]) * u[0] + (b2[1] - a2[1]) * u[1] <= 1e-9 {
            return None;
        }
        Some(s.with_node(ia, a2).with_node(ib, b2))
    }

    /// The handle under `p` on the current selection, if any (for cursors and highlights).
    pub fn handle_under(&self, shapes: &[S], p: Pt, view: View) -> Option<Handle> {
        let (lo, hi) = selection_bounds(shapes, &self.selection)?;
        let visible = self.visible_handles(shapes);
        handle_at(lo, hi, view.handle_pad, p, view.tolerance).filter(|h| visible.contains(h))
    }

    /// The grips of the selected shape: shown (and draggable) only when exactly one shape is
    /// selected and the corners aren't turning it.
    pub fn grips(&self, shapes: &[S]) -> Vec<Grip> {
        match self.selection.as_slice() {
            [id] if !self.rotate_mode => find(shapes, *id).map(Editable::grips).unwrap_or_default(),
            _ => vec![],
        }
    }

    /// The grip under `p`, if any.
    pub fn grip_under(&self, shapes: &[S], p: Pt, view: View) -> Option<Grip> {
        self.grips(shapes).into_iter().filter(|g| dist(g.at, p) <= view.tolerance).min_by(|a, b| dist(a.at, p).total_cmp(&dist(b.at, p)))
    }

    /// Where the selection's centre mark (the move handle) sits. Normally the middle of the
    /// bounding box, but when exactly one shape is selected the shape decides (an arc puts it on
    /// the circle's centre, which can lie outside the box).
    pub fn centre_mark(&self, shapes: &[S]) -> Option<Pt> {
        if let [id] = self.selection.as_slice() {
            if let Some(c) = find(shapes, *id).and_then(Editable::centre_point) {
                return Some(c);
            }
        }
        selection_bounds(shapes, &self.selection).map(|(lo, hi)| [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0])
    }

    /// The topmost shape under `p`. Shapes on the priority layer are preferred.
    pub fn shape_under(&self, shapes: &[S], p: Pt, view: View) -> Option<u64> {
        let pl = self.priority_layer;
        if pl != 0 {
            if let Some(id) = shapes.iter().rev().filter(|s| s.layer() == pl).find(|s| s.hit(p, view.tolerance)).map(Editable::id) {
                return Some(id);
            }
        }
        shapes.iter().rev().find(|s| s.hit(p, view.tolerance)).map(Editable::id)
    }

    /// The node under `p` on a selected shape that has nodes.
    pub fn node_under(&self, shapes: &[S], p: Pt, view: View) -> Option<(u64, usize)> {
        self.selection.iter().filter_map(|id| find(shapes, *id)).find_map(|s| {
            let nodes = s.nodes()?;
            nodes
                .iter()
                .enumerate()
                .map(|(i, n)| (i, dist(*n, p)))
                .filter(|(_, d)| *d <= view.tolerance)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(i, _)| (s.id(), i))
        })
    }

    /// Pointer pressed at `p` (pass where the button went down, not where a drag was noticed).
    /// `additive` is Shift/Cmd: add to or toggle the selection instead of replacing it.
    pub fn press(&mut self, shapes: &[S], p: Pt, view: View, tool: Tool, additive: bool) -> Started {
        self.moved = false;
        self.toggle_on_release = None;
        self.narrow_on_release = None;
        self.pick_segment_on_release = None;
        self.gesture = None;
        self.last_snap = None;
        self.move_snap = None;
        self.clicked_selected = false;
        let started = match tool {
            Tool::Select => self.press_select(shapes, p, view, additive),
            Tool::Nodes => self.press_nodes(shapes, p, view, additive),
        };
        // snap onto everything the gesture doesn't change
        self.targets = match &self.gesture {
            // moving snaps the shape's bounding box onto other blocks' bounding boxes (zoom-stable,
            // edge-exact), not onto every facet node
            Some(Gesture::Move { .. }) => Self::move_targets(shapes, &self.selection),
            Some(Gesture::Resize { .. }) => Self::targets(shapes, &self.selection),
            Some(Gesture::Node { index, orig }) => {
                let own = orig.nodes().and_then(|n| n.get(*index).copied());
                Self::targets(shapes, &[]).into_iter().filter(|(q, _)| Some(*q) != own).collect()
            }
            // a segment snaps onto everything but its own two (moving) ends; the shape's other
            // nodes stay, so a segment can line up with another node of the same polyline
            Some(Gesture::Segment { index, orig, .. }) => {
                let own = Self::segment_ends(orig, *index).map(|(_, _, a, b)| [a, b]).unwrap_or_default();
                Self::targets(shapes, &[]).into_iter().filter(|(q, _)| !own.contains(q)).collect()
            }
            _ => vec![],
        };
        // bounding boxes of the resting shapes/groups, for the gap (equal-spacing) policy while moving
        self.target_boxes = match &self.gesture {
            Some(Gesture::Move { .. }) => Self::target_boxes(shapes, &self.selection),
            _ => vec![],
        };
        started
    }

    fn press_select(&mut self, shapes: &[S], p: Pt, view: View, additive: bool) -> Started {
        if !additive {
            // a shape's own grips sit inside its box, so they come before the box handles
            if let Some(g) = self.grip_under(shapes, p, view) {
                if let Some(orig) = find(shapes, self.selection[0]).cloned() {
                    self.gesture = Some(Gesture::Grip { kind: g.kind, offset: [g.at[0] - p[0], g.at[1] - p[1]], orig });
                    return Started::Grip(g.kind);
                }
            }
            // a selected open vector is edited part by part, as in Fusion 360: a node drags freely
            if let Some((id, index)) = self.node_handle_under(shapes, p, view) {
                if let Some(s) = find(shapes, id) {
                    self.active_node = Some((id, index));
                    self.active_segment = None;
                    self.gesture = Some(Gesture::Node { index, orig: s.clone() });
                    return Started::Node;
                }
            }
            // …and a segment follows the selection rule — except on the centre mark, which moves the
            // whole (it may sit on a segment, as on a straight line)
            let on_centre = self.centre_mark(shapes).is_some_and(|c| dist(c, p) <= view.tolerance);
            if let Some(started) = self.press_segment(shapes, p, view).filter(|_| !on_centre) {
                return started;
            }
            if let Some((lo, hi)) = selection_bounds(shapes, &self.selection) {
                if let Some(handle) = self.handle_under(shapes, p, view) {
                    let orig = self.selected(shapes);
                    if self.rotate_mode {
                        // corners turn the selection about its middle; edges do nothing here
                        let (ax, ay) = handle.axes();
                        if ax && ay {
                            let center = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
                            self.gesture = Some(Gesture::Rotate { center, start: p, angle: 0.0, orig });
                            return Started::Rotate;
                        }
                        return Started::Nothing;
                    }
                    // a handle that could only change locked directions does nothing at all
                    let (ax, ay) = handle.axes();
                    let (lx, ly) = locked_of(&orig);
                    if (!ax || lx) && (!ay || ly) {
                        return Started::Nothing;
                    }
                    let anchor = anchor_for(lo, hi, handle);
                    let grip = handle_point(lo, hi, 0.0, handle);
                    self.gesture = Some(Gesture::Resize { handle, anchor, grip, start: p, orig });
                    return Started::Resize(handle);
                }
            }
        }
        // the centre mark of the selection moves it, as the shape's own line does (on an open
        // vector it is the one way to move the whole, its segments being edited one by one)
        if !additive && !self.rotate_mode {
            if let Some(centre) = self.centre_mark(shapes) {
                if dist(centre, p) <= view.tolerance {
                    self.clicked_selected = true;
                    self.active_node = None;
                    self.active_segment = None;
                    self.gesture = Some(Gesture::Move { start: p, orig: self.selected(shapes) });
                    return Started::Move;
                }
            }
        }
        // anything else picks whole shapes: a node or segment picked before is let go
        self.active_node = None;
        self.active_segment = None;
        match self.shape_under(shapes, p, view) {
            Some(id) => {
                let selected = self.selection.contains(&id);
                self.clicked_selected = selected && !additive;
                if selected && !additive && self.selection.len() > 1 {
                    self.narrow_on_release = Some(id);
                }
                if !selected {
                    self.rotate_mode = false;
                }
                if additive && selected {
                    self.toggle_on_release = Some(id);
                } else if additive {
                    self.selection.push(id);
                } else if !selected {
                    self.selection = vec![id];
                }
                self.select_whole_groups(shapes);
                self.gesture = Some(Gesture::Move { start: p, orig: self.selected(shapes) });
                Started::Move
            }
            None => {
                if !additive {
                    self.selection.clear();
                    self.rotate_mode = false;
                }
                self.gesture = Some(Gesture::Band { start: p, end: p, additive });
                Started::Band
            }
        }
    }

    /// A press on a segment of an open vector, by the selection rule: a press on what is selected
    /// moves all of the selection; a press on a segment that is not part of the selection picks just
    /// that segment and moves it alone (parallel). So with the whole polyline selected (by a box,
    /// nothing of it picked) a drag on any of its segments moves everything that is selected, and a
    /// click without moving narrows the selection to that segment. With one segment picked, a drag
    /// on it moves it; a drag on another segment picks that one instead. Grouped shapes keep the
    /// group rule (the press picks the group). `None` when the press is not on such a segment.
    fn press_segment(&mut self, shapes: &[S], p: Pt, view: View) -> Option<Started> {
        if self.rotate_mode {
            return None;
        }
        let id = self.shape_under(shapes, p, view)?;
        let s = find(shapes, id).filter(|s| s.selection_style() == SelectionStyle::Polyline && s.group().is_none())?;
        let k = segment_of(s, p, view)?;
        let whole = self.selection.contains(&id) && self.active_segment.is_none() && self.active_node.is_none();
        if whole {
            // the press is on the selection: it all moves; a click alone picks the segment
            self.pick_segment_on_release = Some((id, k));
            self.gesture = Some(Gesture::Move { start: p, orig: self.selected(shapes) });
            return Some(Started::Move);
        }
        // the segment that is picked moves; any other segment becomes the selection and moves alone
        self.selection = vec![id];
        self.active_segment = Some((id, k));
        self.active_node = None;
        self.gesture = Some(Gesture::Segment { index: k, start: p, orig: s.clone() });
        Some(Started::Segment)
    }

    fn press_nodes(&mut self, shapes: &[S], p: Pt, view: View, additive: bool) -> Started {
        if let Some((id, index)) = self.node_under(shapes, p, view) {
            self.active_node = Some((id, index));
            if let Some(s) = find(shapes, id) {
                self.gesture = Some(Gesture::Node { index, orig: s.clone() });
                return Started::Node;
            }
        }
        self.active_node = None;
        match self.shape_under(shapes, p, view).filter(|id| find(shapes, *id).is_some_and(|s| s.nodes().is_some())) {
            Some(id) => self.selection = vec![id],
            None if !additive => self.selection.clear(),
            None => {}
        }
        Started::Nothing
    }

    /// A shape in a group brings the rest of its (outermost) group into the selection.
    fn select_whole_groups(&mut self, shapes: &[S]) {
        let groups: Vec<u64> = shapes.iter().filter(|s| self.selection.contains(&s.id())).filter_map(|s| s.group()).collect();
        if groups.is_empty() {
            return;
        }
        for s in shapes {
            if !s.hidden() && !s.frozen() && s.group().is_some_and(|g| groups.contains(&g)) && !self.selection.contains(&s.id()) {
                self.selection.push(s.id());
            }
        }
    }

    fn selected(&self, shapes: &[S]) -> Vec<S> {
        shapes.iter().filter(|s| self.selection.contains(&s.id())).cloned().collect()
    }

    /// Pointer moved to `p` with the button held. `uniform` keeps the aspect ratio on corners.
    pub fn drag(&mut self, shapes: &mut [S], mut p: Pt, uniform: bool) {
        // Resize and node drags move a single handle, so the pointer itself is what snaps. A Move
        // drag is different: the pointer sits somewhere inside the shape, far from any target, so
        // snapping the pointer catches nothing. There the shape's own geometry snaps (handled in the
        // Move branch below, like draw2d's SnapToGeometry), so the pointer must stay raw here.
        if matches!(self.gesture, Some(Gesture::Resize { .. } | Gesture::Node { .. })) {
            p = self.snapped(p);
        } else if matches!(self.gesture, Some(Gesture::Move { .. })) {
            self.last_snap = None;
            self.move_snap = None;
        }
        let step = self.angle_step;
        let Some(g) = &mut self.gesture else { return };
        match g {
            Gesture::Move { start, orig } => {
                let (mut dx, mut dy) = (p[0] - start[0], p[1] - start[1]);
                // the moved shapes' combined bounding box at the tentative position
                let source_box = orig.iter().filter_map(Editable::bounds).fold(None, |acc: Option<(Pt, Pt)>, (lo, hi)| {
                    let (lo, hi) = ([lo[0] + dx, lo[1] + dy], [hi[0] + dx, hi[1] + dy]);
                    Some(match acc {
                        Some((alo, ahi)) => ([alo[0].min(lo[0]), alo[1].min(lo[1])], [ahi[0].max(hi[0]), ahi[1].max(hi[1])]),
                        None => (lo, hi),
                    })
                });
                if let Some(source_box) = source_box {
                    // snap the moved box's bounding-box points (corners, edge middles, centre) onto the
                    // resting blocks' bounding boxes, each axis independently — bounding boxes, not
                    // facet nodes, so the result is edge-exact and the same at every zoom
                    let (slo, shi) = source_box;
                    let mut sources = bbox_snap_points(slo, shi);
                    // plus the moved shapes' own off-box centres (an arc's circle centre), moved by the
                    // tentative offset, so the centre itself can catch a guide
                    for c in Self::offbox_centres(orig, &[]) {
                        sources.push(([c[0] + dx, c[1] + dy], SnapKind::Centre));
                    }
                    if let Some(snap) = self.snapping.snap_move(&sources, source_box, &self.targets, &self.target_boxes) {
                        dx += snap.offset[0];
                        dy += snap.offset[1];
                        // a single marker for the corner that caught (x guide preferred, else y)
                        self.last_snap = match (snap.x_guide, snap.y_guide) {
                            (Some((x, k)), Some((y, _))) => Some(([x, y], k)),
                            (Some((x, k)), None) => sources.iter().map(|(s, _)| s[1]).next().map(|sy| ([x, sy + snap.offset[1]], k)),
                            (None, Some((y, k))) => sources.iter().map(|(s, _)| s[0]).next().map(|sx| ([sx + snap.offset[0], y], k)),
                            (None, None) => None,
                        };
                        self.move_snap = Some(snap);
                    }
                }
                self.moved |= dx != 0.0 || dy != 0.0;
                for o in orig.iter() {
                    put(shapes, o.translated(dx, dy));
                }
            }
            Gesture::Resize { handle, anchor, grip, start, orig } => {
                // only the directions the handle changes and no shape has locked
                let (hx, hy) = handle.axes();
                let (lx, ly) = locked_of(orig);
                let (ax, ay) = (hx && !lx, hy && !ly);
                // where the grip has been dragged to: no jump when the press was beside it
                let to = [grip[0] + p[0] - start[0], grip[1] + p[1] - start[1]];
                let mut fx = if ax { ratio(to[0], anchor[0], grip[0]) } else { 1.0 };
                let mut fy = if ay { ratio(to[1], anchor[1], grip[1]) } else { 1.0 };
                if uniform && ax && ay {
                    let f = fx.max(fy);
                    (fx, fy) = (f, f);
                } else if orig.len() == 1 && orig[0].keeps_proportions() && (ax || ay) {
                    // a circle stays a circle: a side handle grows both ways
                    let f = if ax { fx } else { fy };
                    (fx, fy) = (f, f);
                }
                self.moved |= fx != 1.0 || fy != 1.0;
                for o in orig.iter() {
                    put(shapes, o.scaled(*anchor, fx, fy));
                }
            }
            Gesture::Node { index, orig } => {
                self.moved = true;
                put(shapes, orig.with_node(*index, p));
            }
            Gesture::Segment { index, start, orig } => {
                let Some((_, _, a, b)) = Self::segment_ends(orig, *index) else { return };
                let Some(n) = unit_normal(a, b) else { return };
                // only the movement across the segment counts: it moves parallel, never along itself
                let d = (p[0] - start[0]) * n[0] + (p[1] - start[1]) * n[1];
                let (d, caught) = self.snapping.snap_across(a, n, d, &self.targets);
                self.last_snap = caught;
                // a shift that would turn the segment or a neighbour round keeps the last good place
                if let Some(moved) = Self::moved_parallel(orig, *index, d) {
                    self.moved |= d != 0.0;
                    put(shapes, moved);
                }
            }
            Gesture::Band { end, start, .. } => {
                *end = p;
                self.moved |= dist(*start, p) > 0.0;
            }
            Gesture::Rotate { center, start, angle, orig } => {
                let dir = |q: Pt| (q[1] - center[1]).atan2(q[0] - center[0]).to_degrees();
                // keep the turn between −180° and 180°
                let mut a = (dir(p) - dir(*start) + 180.0).rem_euclid(360.0) - 180.0;
                if uniform && step > 0.0 {
                    a = (a / step).round() * step;
                }
                *angle = a;
                self.moved |= a != 0.0;
                for o in orig.iter() {
                    put(shapes, o.rotated(*center, a));
                }
            }
            Gesture::Grip { kind, offset, orig } => {
                let to = [p[0] + offset[0], p[1] + offset[1]];
                let s = orig.with_grip(*kind, to);
                self.moved |= s.bounds() != orig.bounds() || s.grips() != orig.grips();
                put(shapes, s);
            }
        }
    }

    /// The turn in degrees while a rotation is being dragged (for an angle readout).
    pub fn rotation_angle(&self) -> Option<f64> {
        match self.gesture {
            Some(Gesture::Rotate { angle, .. }) => Some(angle),
            _ => None,
        }
    }

    /// Turn the selection by `deg` about the middle of its bounds (a typed angle).
    pub fn rotate_selection(&mut self, shapes: &mut [S], deg: f64) -> bool {
        let Some((lo, hi)) = selection_bounds(shapes, &self.selection) else { return false };
        if deg == 0.0 || !deg.is_finite() {
            return false;
        }
        let center = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
        let turned: Vec<S> = self.selected(shapes).into_iter().map(|s| s.rotated(center, deg)).collect();
        for s in turned {
            put(shapes, s);
        }
        true
    }

    /// Button released. Finishes the gesture and says what it did.
    pub fn release(&mut self, shapes: &[S]) -> Outcome {
        self.targets.clear();
        self.target_boxes.clear();
        self.last_snap = None;
        self.move_snap = None;
        let Some(g) = self.gesture.take() else { return Outcome::Unchanged };
        match g {
            Gesture::Move { .. } if self.moved => {
                self.pick_segment_on_release = None;
                Outcome::Moved
            }
            Gesture::Move { .. } => {
                // a click on a segment of the whole selected polyline picks just that segment
                if let Some((id, k)) = self.pick_segment_on_release.take() {
                    self.selection = vec![id];
                    self.active_segment = Some((id, k));
                    self.active_node = None;
                    return Outcome::Selected;
                }
                if let Some(id) = self.toggle_on_release.take() {
                    // a shift-click on a member takes the whole group out
                    let group = find(shapes, id).and_then(|s| s.group());
                    self.selection.retain(|x| *x != id && (group.is_none() || find(shapes, *x).and_then(|s| s.group()) != group));
                }
                // a click on one of several selected shapes picks that one (with its group) alone
                if let Some(id) = self.narrow_on_release.take() {
                    let kept = std::mem::replace(&mut self.selection, vec![id]);
                    self.select_whole_groups(shapes);
                    if self.selection.len() == kept.len() && kept.iter().all(|k| self.selection.contains(k)) {
                        self.selection = kept;
                    } else {
                        self.clicked_selected = false;
                        self.rotate_mode = false;
                    }
                }
                if self.clicked_selected {
                    self.rotate_mode = !self.rotate_mode;
                }
                Outcome::Selected
            }
            Gesture::Rotate { .. } if self.moved => Outcome::Rotated,
            Gesture::Rotate { .. } => Outcome::Unchanged,
            Gesture::Grip { .. } if self.moved => Outcome::Reshaped,
            Gesture::Grip { .. } => Outcome::Unchanged,
            Gesture::Resize { .. } if self.moved => Outcome::Resized,
            Gesture::Node { .. } | Gesture::Segment { .. } if self.moved => Outcome::NodeMoved,
            Gesture::Resize { .. } | Gesture::Node { .. } => Outcome::Unchanged,
            // a click on a segment without moving only picks it
            Gesture::Segment { .. } => Outcome::Selected,
            Gesture::Band { start, end, additive } => {
                if self.moved {
                    let lo = [start[0].min(end[0]), start[1].min(end[1])];
                    let hi = [start[0].max(end[0]), start[1].max(end[1])];
                    let inside = shapes
                        .iter()
                        .filter(|s| !s.hidden() && !s.frozen() && s.bounds().is_some_and(|(a, b)| a[0] >= lo[0] && a[1] >= lo[1] && b[0] <= hi[0] && b[1] <= hi[1]))
                        .map(Editable::id);
                    if !additive {
                        self.selection.clear();
                    }
                    for id in inside {
                        if !self.selection.contains(&id) {
                            self.selection.push(id);
                        }
                    }
                    self.select_whole_groups(shapes);
                }
                Outcome::Selected
            }
        }
    }

    /// Abort the gesture and put every shape back as it was when it started.
    pub fn cancel(&mut self, shapes: &mut [S]) {
        self.targets.clear();
        self.target_boxes.clear();
        self.last_snap = None;
        self.move_snap = None;
        match self.gesture.take() {
            Some(Gesture::Move { orig, .. }) | Some(Gesture::Resize { orig, .. }) | Some(Gesture::Rotate { orig, .. }) => {
                for o in orig {
                    put(shapes, o);
                }
            }
            Some(Gesture::Node { orig, .. }) | Some(Gesture::Segment { orig, .. }) | Some(Gesture::Grip { orig, .. }) => put(shapes, orig),
            _ => {}
        }
    }

    /// Move the active node (node mode, or a picked node of a selected open vector), a picked
    /// segment (across itself only, as when it is dragged) or else the selection by (dx, dy).
    /// Returns whether anything moved.
    pub fn nudge(&mut self, shapes: &mut [S], tool: Tool, dx: f64, dy: f64) -> bool {
        let part_picked = tool == Tool::Nodes || self.polyline(shapes).is_some();
        if part_picked {
            if let Some((id, i)) = self.active_node {
                let Some(s) = find(shapes, id).cloned() else { return false };
                let Some(n) = s.nodes().and_then(|n| n.get(i).copied()) else { return false };
                put(shapes, s.with_node(i, [n[0] + dx, n[1] + dy]));
                return true;
            }
        }
        if tool == Tool::Select && self.polyline(shapes).is_some() {
            if let Some((id, k)) = self.active_segment {
                let Some(s) = find(shapes, id).cloned() else { return false };
                let Some((_, _, a, b)) = Self::segment_ends(&s, k) else { return false };
                let Some(n) = unit_normal(a, b) else { return false };
                let d = dx * n[0] + dy * n[1];
                if d == 0.0 {
                    // an arrow along the segment would slide it sideways: a segment only moves across
                    return false;
                }
                let Some(moved) = Self::moved_parallel(&s, k, d) else { return false };
                put(shapes, moved);
                return true;
            }
        }
        let moved: Vec<S> = self.selected(shapes).into_iter().map(|s| s.translated(dx, dy)).collect();
        let any = !moved.is_empty();
        for s in moved {
            put(shapes, s);
        }
        any
    }

    /// Insert a node where `p` lies on a selected shape's outline (node mode double-click).
    pub fn insert_node(&mut self, shapes: &mut [S], p: Pt, view: View) -> bool {
        let mut best: Option<(f64, u64, usize)> = None;
        for s in self.selection.iter().filter_map(|id| find(shapes, *id)) {
            for (a, b) in s.segments() {
                let d = s.segment_distance(p, a, b);
                if d <= view.tolerance && best.is_none_or(|x| d < x.0) {
                    best = Some((d, s.id(), a));
                }
            }
        }
        let Some((_, id, after)) = best else { return false };
        let Some(s) = find(shapes, id).cloned() else { return false };
        put(shapes, s.with_inserted_node(after, p));
        self.active_node = Some((id, after + 1));
        true
    }

    /// Delete the active node. Returns whether it was removed.
    pub fn delete_active_node(&mut self, shapes: &mut [S]) -> bool {
        let Some((id, i)) = self.active_node else { return false };
        let Some(s) = find(shapes, id) else { return false };
        match s.without_node(i) {
            Some(new) => {
                put(shapes, new);
                self.active_node = None;
                true
            }
            None => false,
        }
    }

    /// Drop ids from the selection that no longer exist (after delete or undo).
    pub fn retain_existing(&mut self, shapes: &[S]) {
        self.selection.retain(|id| shapes.iter().any(|s| s.id() == *id && !s.hidden() && !s.frozen()));
        if let Some((id, i)) = self.active_node {
            let ok = find(shapes, id).and_then(|s| s.nodes()).is_some_and(|n| i < n.len());
            if !ok {
                self.active_node = None;
            }
        }
        if let Some((id, k)) = self.active_segment {
            let ok = self.selection.contains(&id) && find(shapes, id).is_some_and(|s| k < s.segments().len());
            if !ok {
                self.active_segment = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A polyline test shape: enough to exercise every gesture.
    #[derive(Clone, Debug, PartialEq)]
    struct Poly {
        id: u64,
        pts: Vec<Pt>,
        closed: bool,
    }

    impl Editable for Poly {
        fn id(&self) -> u64 {
            self.id
        }
        fn bounds(&self) -> Option<(Pt, Pt)> {
            let mut lo = [f64::MAX; 2];
            let mut hi = [f64::MIN; 2];
            for p in &self.pts {
                lo = [lo[0].min(p[0]), lo[1].min(p[1])];
                hi = [hi[0].max(p[0]), hi[1].max(p[1])];
            }
            Some((lo, hi))
        }
        // the test shape is picked anywhere in its box, which keeps the tests short
        fn hit(&self, p: Pt, tol: f64) -> bool {
            let (lo, hi) = self.bounds().unwrap();
            p[0] >= lo[0] - tol && p[0] <= hi[0] + tol && p[1] >= lo[1] - tol && p[1] <= hi[1] + tol
        }
        fn translated(&self, dx: f64, dy: f64) -> Self {
            Poly { pts: self.pts.iter().map(|p| [p[0] + dx, p[1] + dy]).collect(), ..self.clone() }
        }
        fn rotated(&self, about: Pt, deg: f64) -> Self {
            let (s, c) = deg.to_radians().sin_cos();
            let turn = |p: &Pt| {
                let (x, y) = (p[0] - about[0], p[1] - about[1]);
                [about[0] + x * c - y * s, about[1] + x * s + y * c]
            };
            Poly { pts: self.pts.iter().map(turn).collect(), ..self.clone() }
        }
        fn scaled(&self, a: Pt, sx: f64, sy: f64) -> Self {
            Poly { pts: self.pts.iter().map(|p| [a[0] + (p[0] - a[0]) * sx, a[1] + (p[1] - a[1]) * sy]).collect(), ..self.clone() }
        }
        fn nodes(&self) -> Option<Vec<Pt>> {
            Some(self.pts.clone())
        }
        fn snap_points(&self) -> Vec<(Pt, SnapKind)> {
            self.pts.iter().map(|p| (*p, SnapKind::Node)).collect()
        }
        fn nodes_closed(&self) -> bool {
            self.closed
        }
        fn with_node(&self, i: usize, p: Pt) -> Self {
            let mut s = self.clone();
            s.pts[i] = p;
            s
        }
        fn with_inserted_node(&self, after: usize, p: Pt) -> Self {
            let mut s = self.clone();
            s.pts.insert(after + 1, p);
            s
        }
        fn without_node(&self, i: usize) -> Option<Self> {
            (self.pts.len() > if self.closed { 3 } else { 2 }).then(|| {
                let mut s = self.clone();
                s.pts.remove(i);
                s
            })
        }
    }

    fn square(id: u64, x: f64, y: f64, size: f64) -> Poly {
        Poly { id, pts: vec![[x, y], [x + size, y], [x + size, y + size], [x, y + size]], closed: true }
    }

    const VIEW: View = View { tolerance: 1.0, handle_pad: 0.0 };

    fn bounds(s: &Poly) -> (Pt, Pt) {
        s.bounds().unwrap()
    }

    #[test]
    fn click_selects_and_drag_moves_from_press_point() {
        let mut shapes = vec![square(1, 0.0, 0.0, 10.0)];
        let mut ed = Editor::default();
        assert_eq!(ed.press(&shapes, [5.0, 5.0], VIEW, Tool::Select, false), Started::Move);
        assert_eq!(ed.selection, vec![1]);
        ed.drag(&mut shapes, [7.0, 8.0], false);
        ed.drag(&mut shapes, [15.0, 25.0], false);
        assert_eq!(ed.release(&shapes), Outcome::Moved);
        assert_eq!(bounds(&shapes[0]), ([10.0, 20.0], [20.0, 30.0]));
    }

    #[test]
    fn click_without_moving_only_selects() {
        let shapes = vec![square(1, 0.0, 0.0, 10.0)];
        let mut ed = Editor::default();
        ed.press(&shapes, [5.0, 5.0], VIEW, Tool::Select, false);
        assert_eq!(ed.release(&shapes), Outcome::Selected);
        assert_eq!(ed.selection, vec![1]);
    }

    #[test]
    fn every_handle_resizes_without_flipping() {
        // Each handle, dragged 10 units outward, grows the box by 10 on its side(s) and
        // keeps the opposite side fixed.
        let cases = [
            (Handle::Right, [20.0, 5.0], ([0.0, 0.0], [20.0, 10.0])),
            (Handle::Left, [-10.0, 5.0], ([-10.0, 0.0], [10.0, 10.0])),
            (Handle::Top, [5.0, 20.0], ([0.0, 0.0], [10.0, 20.0])),
            (Handle::Bottom, [5.0, -10.0], ([0.0, -10.0], [10.0, 10.0])),
            (Handle::TopRight, [20.0, 20.0], ([0.0, 0.0], [20.0, 20.0])),
            (Handle::TopLeft, [-10.0, 20.0], ([-10.0, 0.0], [10.0, 20.0])),
            (Handle::BottomLeft, [-10.0, -10.0], ([-10.0, -10.0], [10.0, 10.0])),
            (Handle::BottomRight, [20.0, -10.0], ([0.0, -10.0], [20.0, 10.0])),
        ];
        for (handle, to, expect) in cases {
            let mut shapes = vec![square(1, 0.0, 0.0, 10.0)];
            let mut ed = Editor::default();
            ed.selection = vec![1];
            let grip = handle_point([0.0, 0.0], [10.0, 10.0], 0.0, handle);
            assert_eq!(ed.press(&shapes, grip, VIEW, Tool::Select, false), Started::Resize(handle), "{handle:?}");
            ed.drag(&mut shapes, to, false);
            assert_eq!(ed.release(&shapes), Outcome::Resized, "{handle:?}");
            assert_eq!(bounds(&shapes[0]), expect, "{handle:?}");
        }
    }

    #[test]
    fn grabbing_a_padded_handle_does_not_jump() {
        // handles drawn 2 units outside the box; grabbing one and not moving changes nothing,
        // moving it by 5 grows the box by exactly 5
        let view = View { tolerance: 1.0, handle_pad: 2.0 };
        let mut shapes = vec![square(1, 0.0, 0.0, 10.0)];
        let mut ed = Editor::default();
        ed.selection = vec![1];
        assert_eq!(ed.press(&shapes, [-2.0, 5.0], view, Tool::Select, false), Started::Resize(Handle::Left));
        ed.drag(&mut shapes, [-2.0, 5.0], false);
        assert_eq!(bounds(&shapes[0]), ([0.0, 0.0], [10.0, 10.0]));
        ed.drag(&mut shapes, [-7.0, 5.0], false);
        assert_eq!(ed.release(&shapes), Outcome::Resized);
        assert_eq!(bounds(&shapes[0]), ([-5.0, 0.0], [10.0, 10.0]));
    }

    #[test]
    fn resize_never_turns_inside_out() {
        let mut shapes = vec![square(1, 0.0, 0.0, 10.0)];
        let mut ed = Editor::default();
        ed.selection = vec![1];
        ed.press(&shapes, [0.0, 5.0], VIEW, Tool::Select, false); // left handle
        ed.drag(&mut shapes, [50.0, 5.0], false); // far past the right edge
        let (lo, hi) = bounds(&shapes[0]);
        assert!(hi[0] - lo[0] > 0.0 && (hi[0] - 10.0).abs() < 1e-9);
    }

    #[test]
    fn uniform_corner_resize_keeps_aspect() {
        let mut shapes = vec![Poly { id: 1, pts: vec![[0.0, 0.0], [20.0, 0.0], [20.0, 10.0], [0.0, 10.0]], closed: true }];
        let mut ed = Editor::default();
        ed.selection = vec![1];
        ed.press(&shapes, [20.0, 10.0], VIEW, Tool::Select, false);
        ed.drag(&mut shapes, [40.0, 12.0], true);
        assert_eq!(bounds(&shapes[0]), ([0.0, 0.0], [40.0, 20.0]));
    }

    #[test]
    fn cancel_restores_shapes() {
        let mut shapes = vec![square(1, 0.0, 0.0, 10.0)];
        let before = shapes.clone();
        let mut ed = Editor::default();
        ed.press(&shapes, [5.0, 5.0], VIEW, Tool::Select, false);
        ed.drag(&mut shapes, [50.0, 50.0], false);
        ed.cancel(&mut shapes);
        assert_eq!(shapes, before);
        assert!(!ed.is_busy());
    }

    #[test]
    fn additive_click_toggles_and_band_selects() {
        let shapes = vec![square(1, 0.0, 0.0, 10.0), square(2, 20.0, 0.0, 10.0), square(3, 100.0, 100.0, 5.0)];
        let mut ed = Editor::default();
        ed.press(&shapes, [5.0, 5.0], VIEW, Tool::Select, false);
        ed.release(&shapes);
        ed.press(&shapes, [25.0, 5.0], VIEW, Tool::Select, true);
        ed.release(&shapes);
        assert_eq!(ed.selection, vec![1, 2]);
        ed.press(&shapes, [25.0, 5.0], VIEW, Tool::Select, true);
        ed.release(&shapes);
        assert_eq!(ed.selection, vec![1]);

        // rubber band from empty space around the first two squares
        let mut shapes = shapes;
        assert_eq!(ed.press(&shapes, [-5.0, 50.0], VIEW, Tool::Select, false), Started::Band);
        assert!(ed.selection.is_empty());
        ed.drag(&mut shapes, [40.0, -5.0], false);
        assert_eq!(ed.release(&shapes), Outcome::Selected);
        assert_eq!(ed.selection, vec![1, 2]);
    }

    #[test]
    fn nodes_drag_insert_delete_and_nudge() {
        let mut shapes = vec![Poly { id: 7, pts: vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]], closed: false }];
        let mut ed = Editor::default();
        // first click picks the shape, the next press grabs a node
        assert_eq!(ed.press(&shapes, [10.0, 5.0], VIEW, Tool::Nodes, false), Started::Nothing);
        assert_eq!(ed.selection, vec![7]);
        assert_eq!(ed.press(&shapes, [10.2, 0.1], VIEW, Tool::Nodes, false), Started::Node);
        ed.drag(&mut shapes, [12.0, -3.0], false);
        assert_eq!(ed.release(&shapes), Outcome::NodeMoved);
        assert_eq!(shapes[0].pts[1], [12.0, -3.0]);

        assert!(ed.nudge(&mut shapes, Tool::Nodes, 1.0, 0.0));
        assert_eq!(shapes[0].pts[1], [13.0, -3.0]);

        // insert on the first segment, then delete it again
        assert!(ed.insert_node(&mut shapes, [6.5, -1.5], VIEW));
        assert_eq!(shapes[0].pts.len(), 4);
        assert_eq!(ed.active_node, Some((7, 1)));
        assert!(ed.delete_active_node(&mut shapes));
        assert_eq!(shapes[0].pts.len(), 3);
    }

    #[test]
    fn moving_snaps_the_shape_onto_another_shape() {
        // drag the first square so one of its corners comes near the second square's corner: the
        // whole shape snaps by that corner (draw2d's SnapToGeometry snaps the figure, not the
        // pointer), landing the corners exactly on each other
        let mut shapes = vec![square(1, 0.0, 0.0, 10.0), square(2, 30.0, 0.0, 10.0)];
        let mut ed = Editor::default();
        ed.snapping = Snapping { radius: 1.5, ..Snapping::default() };
        ed.press(&shapes, [5.0, 5.0], VIEW, Tool::Select, false);
        // delta ~(+19.3, +0.4): the bottom-right corner lands near the second square's [30, 0]
        ed.drag(&mut shapes, [24.3, 5.4], false);
        assert_eq!(ed.last_snap, Some(([30.0, 0.0], SnapKind::Node)));
        ed.release(&shapes);
        assert_eq!(bounds(&shapes[0]), ([20.0, 0.0], [30.0, 10.0]), "corner landed exactly on the corner");
        assert_eq!(ed.last_snap, None);
    }

    #[test]
    fn moving_snaps_by_the_bounding_box_not_a_facet_node() {
        // a many-sided polygon (a facetted "circle") whose nodes are NOT on its bbox corners: the
        // rightmost extent is at x=10 but the nearest node to a target is inside. Snapping must use
        // the bounding box, so the shape's right edge lands exactly on the target's left edge — no
        // gap (the zoom-out bug: a facet node, not the extent, used to catch and leave a gap).
        let blob = Poly {
            id: 1,
            // extremes at x=0 and x=10, y=0 and y=10, but the side nodes are pulled inward
            pts: vec![[5.0, 0.0], [10.0, 5.0], [5.0, 10.0], [0.0, 5.0]],
            closed: true,
        };
        let target = square(2, 30.0, 0.0, 10.0); // its left edge is at x=30
        let mut shapes = vec![blob, target];
        let mut ed = Editor::default();
        ed.snapping = Snapping { radius: 1.5, ..Snapping::default() };
        ed.press(&shapes, [5.0, 5.0], VIEW, Tool::Select, false);
        // move right so the blob's bbox right edge (x=10 → ~29.4) comes near the target's left (30)
        ed.drag(&mut shapes, [24.4, 5.0], false);
        ed.release(&shapes);
        let (lo, hi) = bounds(&shapes[0]);
        assert!((hi[0] - 30.0).abs() < 1e-9, "the blob's right extent sits exactly on x=30, got {}", hi[0]);
        assert!((lo[0] - 20.0).abs() < 1e-9, "width is kept (20..30)");
    }

    #[test]
    fn a_moving_shape_never_snaps_to_itself() {
        let mut shapes = vec![square(1, 0.0, 0.0, 10.0)];
        let mut ed = Editor::default();
        ed.snapping = Snapping { radius: 5.0, ..Snapping::default() };
        ed.press(&shapes, [10.0, 5.0], VIEW, Tool::Select, false);
        ed.drag(&mut shapes, [12.0, 5.0], false);
        assert_eq!(ed.last_snap, None);
        assert_eq!(bounds(&shapes[0]).0, [2.0, 0.0]);
    }

    #[test]
    fn node_drag_snaps_to_grid() {
        let mut shapes = vec![Poly { id: 7, pts: vec![[0.0, 0.0], [10.0, 0.0]], closed: false }];
        let mut ed = Editor::default();
        ed.snapping = Snapping { policies: crate::snap::SnapPolicies { geometry: false, center: false, gap: false }, grid: Some(5.0), radius: 1.0 };
        ed.selection = vec![7];
        assert_eq!(ed.press(&shapes, [10.0, 0.0], VIEW, Tool::Nodes, false), Started::Node);
        ed.drag(&mut shapes, [14.7, 5.4], false);
        ed.release(&shapes);
        assert_eq!(shapes[0].pts[1], [15.0, 5.0]);
    }

    #[test]
    fn clicking_a_selected_shape_again_switches_to_rotating() {
        let mut shapes = vec![square(1, 0.0, 0.0, 10.0)];
        let mut ed = Editor::default();
        // click the left edge away from its handle
        ed.press(&shapes, [0.0, 2.0], VIEW, Tool::Select, false);
        ed.release(&shapes);
        assert!(!ed.rotate_mode, "first click only selects");
        ed.press(&shapes, [0.0, 2.0], VIEW, Tool::Select, false);
        ed.release(&shapes);
        assert!(ed.rotate_mode, "second click: rotate");

        // drag the top-right corner a quarter turn round the middle (5, 5)
        assert_eq!(ed.press(&shapes, [10.0, 10.0], VIEW, Tool::Select, false), Started::Rotate);
        ed.drag(&mut shapes, [0.0, 10.0], false);
        assert!((ed.rotation_angle().unwrap() - 90.0).abs() < 1e-9);
        assert_eq!(ed.release(&shapes), Outcome::Rotated);
        let p = &shapes[0].pts;
        assert!((p[0][0] - 10.0).abs() < 1e-9 && p[0][1].abs() < 1e-9, "corner (0,0) went to (10,0): {p:?}");

        // a third click switches back to resizing (the square now runs 0..10 again)
        ed.press(&shapes, [0.0, 2.0], VIEW, Tool::Select, false);
        ed.release(&shapes);
        assert!(!ed.rotate_mode);
    }

    #[test]
    fn rotation_steps_cancel_and_typed_angle() {
        let mut shapes = vec![square(1, 0.0, 0.0, 10.0)];
        let mut ed = Editor::default();
        ed.selection = vec![1];
        ed.rotate_mode = true;
        ed.press(&shapes, [10.0, 10.0], VIEW, Tool::Select, false);
        // from 45° to about 59°: with the step key held that is one whole 15° step
        let a = 59f64.to_radians();
        ed.drag(&mut shapes, [5.0 + 7.0 * a.cos(), 5.0 + 7.0 * a.sin()], true);
        assert_eq!(ed.rotation_angle(), Some(15.0));
        ed.cancel(&mut shapes);
        assert_eq!(shapes[0], square(1, 0.0, 0.0, 10.0), "cancel turns it back");

        assert!(ed.rotate_selection(&mut shapes, 90.0));
        let (lo, hi) = bounds(&shapes[0]);
        assert!(lo[0].abs() < 1e-9 && lo[1].abs() < 1e-9 && (hi[0] - 10.0).abs() < 1e-9, "a quarter turn of a square keeps its box");
        assert!(!ed.rotate_selection(&mut shapes, 0.0));
    }

    #[test]
    fn nudge_moves_selection() {
        let mut shapes = vec![square(1, 0.0, 0.0, 10.0), square(2, 50.0, 0.0, 10.0)];
        let mut ed = Editor::default();
        ed.selection = vec![2];
        assert!(ed.nudge(&mut shapes, Tool::Select, 0.0, 5.0));
        assert_eq!(bounds(&shapes[1]).0, [50.0, 5.0]);
        assert_eq!(bounds(&shapes[0]).0, [0.0, 0.0]);
    }
}
