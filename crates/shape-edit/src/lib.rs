//! 2D shapes for a design program, independent of any UI toolkit:
//!
//! - [`forms`]: one class per kind of shape ([`Rect`], [`Circle`], [`Ellipse`], [`Polygon`],
//!   [`Path`], [`Compound`], [`Text`]) and the [`Shape`] that holds any of them;
//! - [`region`] and [`ops`]: union, difference, intersection, expand / shrink and mirroring;
//! - [`snap`]: snapping to nodes, centres, midpoints, job points, a grid and angle steps;
//! - [`Editor`]: select, move, resize and node-edit gestures, fed with pointer events in world
//!   coordinates (Y up) and working on anything that implements [`Editable`] (such as [`Item`]).
//!
//! The editor rewrites shapes from snapshots taken when a gesture starts, so a gesture never
//! accumulates rounding error and can always be cancelled.
//!
//! ```
//! use shape_edit::{Editor, Item, Rect, Shape, Tool, View};
//! let mut items = vec![Item::new(1, Rect::new([5.0, 5.0], 10.0, 10.0))];
//! let mut ed = Editor::default();
//! let view = View { tolerance: 1.0, handle_pad: 0.0 };
//! // shapes are picked on their line: grab the left edge and drag it 10 to the right
//! ed.press(&items, [0.0, 5.0], view, Tool::Select, false);
//! ed.drag(&mut items, [10.0, 5.0], false);
//! ed.release(&items);
//! assert_eq!(items[0].shape, Shape::Rect(Rect::new([15.0, 5.0], 10.0, 10.0)));
//! ```

mod editor;
pub mod calc;
pub mod constraints;
pub mod forms;
pub mod geometry;
mod handles;
mod item;
pub mod ops;
pub mod region;
pub mod snap;
pub mod vars;

pub use constraints::{Align, Anchor, Axis, Constrains, ConstraintView, Distance, Orient, SizeConstraint};
pub use editor::{Editor, Gesture, Outcome, Started, Tool, View};
pub use forms::{Circle, Compound, Dim, DimLine, Draw, Ellipse, Flip, Form, Grip, GripKind, Path, Polygon, Rect, Shape, Spline, SplineNode, Text};
pub use geometry::Contour;
pub use handles::{anchor_for, handle_at, handle_point, selection_bounds, Handle};
pub use item::{can_group, group_items, ungroup_items, Item, Source};
pub use region::{Region, Ring};
pub use snap::{snap_angle, SnapKind, Snapping};
pub use vars::{Resolved, VarEvent, Variable, Variables};

/// How a selected shape shows that it is selected, chosen by the shape's own class (its
/// "selection feedback").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionStyle {
    /// A box round the shape with eight handles, the outline in marching dashes (rectangles, text,
    /// vectors, several shapes together).
    Box,
    /// A ring-like shape: a solid outline, its centre, and four handles on the sides (circles and
    /// ellipses); no box.
    Ring,
    /// A single straight line: a solid line with a handle at its start and at its end; no box.
    Ends,
}

/// A point in world coordinates (millimetres, X right, Y up).
pub type Pt = [f64; 2];

/// What the editor needs to know about a shape. Transform methods return a new shape so the
/// editor can always rebuild from the snapshot taken when a gesture began.
pub trait Editable: Clone {
    fn id(&self) -> u64;
    /// Axis-aligned bounds in world coordinates.
    fn bounds(&self) -> Option<(Pt, Pt)>;
    /// Whether a point within `tol` (world units) picks this shape.
    fn hit(&self, p: Pt, tol: f64) -> bool;
    /// Whether `p` lies inside the shape's area (not used for picking).
    fn contains(&self, _p: Pt) -> bool {
        false
    }
    /// The shape's layer is switched off: it is neither picked nor snapped to nor selected.
    fn hidden(&self) -> bool {
        false
    }
    /// The shape's layer is locked: it is drawn, but nothing can be done with it.
    fn frozen(&self) -> bool {
        false
    }
    /// The outermost group the shape is in: picking one member of a group picks all of them.
    fn group(&self) -> Option<u64> {
        None
    }
    fn translated(&self, dx: f64, dy: f64) -> Self;
    /// Scale by (sx, sy) about the fixed world point `anchor`. Factors are always positive.
    fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Self;
    /// Turn by `deg` degrees (anticlockwise) about `about`.
    fn rotated(&self, _about: Pt, _deg: f64) -> Self {
        self.clone()
    }

    /// The directions of the selection box this shape does not let the resize handles change:
    /// (horizontal, vertical). A size tied to a formula is a constraint, so it can't be dragged.
    fn locked_axes(&self) -> (bool, bool) {
        (false, false)
    }

    /// The shape's own extra handles while it is selected alone (its "selected decoration").
    fn grips(&self) -> Vec<Grip> {
        vec![]
    }

    /// The shape with grip `kind` dragged to `p`.
    fn with_grip(&self, _kind: GripKind, _p: Pt) -> Self {
        self.clone()
    }

    /// Points other shapes and the pointer can snap to (nodes, centre, midpoints).
    fn snap_points(&self) -> Vec<(Pt, SnapKind)> {
        vec![]
    }

    /// Whether this shape can only be resized evenly (a circle stays a circle: even a side handle
    /// grows it in both directions).
    fn keeps_proportions(&self) -> bool {
        false
    }

    /// How this shape looks when it is the one selected shape.
    fn selection_style(&self) -> SelectionStyle {
        SelectionStyle::Box
    }

    /// The editable nodes in world coordinates, or `None` when the shape has no nodes.
    fn nodes(&self) -> Option<Vec<Pt>> {
        None
    }
    /// Whether the node list closes back on itself (used by the default [`Self::segments`]).
    fn nodes_closed(&self) -> bool {
        false
    }
    /// The outline segments as pairs of node indices; a new node can be inserted on any of them.
    fn segments(&self) -> Vec<(usize, usize)> {
        let n = self.nodes().map_or(0, |n| n.len());
        let m = if self.nodes_closed() { n } else { n.saturating_sub(1) };
        (0..m).map(|i| (i, (i + 1) % n)).collect()
    }
    /// Distance from `p` to segment `(a, b)`.  Default: straight-line distance.  Splines
    /// override this to walk the Bezier tessellation.
    fn segment_distance(&self, p: Pt, seg_a: usize, seg_b: usize) -> f64 {
        let Some(nodes) = self.nodes() else { return f64::MAX };
        if seg_a >= nodes.len() || seg_b >= nodes.len() { return f64::MAX; }
        geometry::dist_to_segment(p, nodes[seg_a], nodes[seg_b])
    }
    /// The shape with node `index` moved to world point `p`.
    fn with_node(&self, _index: usize, _p: Pt) -> Self {
        self.clone()
    }
    /// The shape with a new node at world point `p`, inserted after node `after`.
    fn with_inserted_node(&self, _after: usize, _p: Pt) -> Self {
        self.clone()
    }
    /// The shape without node `index`, or `None` when it would become degenerate.
    fn without_node(&self, _index: usize) -> Option<Self> {
        None
    }
}
