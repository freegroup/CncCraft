//! Snapping: pulling the pointer and the shapes being moved onto nearby geometry, onto a grid and
//! onto fixed angle steps. The workpiece itself is never a snap anchor — only the elements and
//! groups on it are.
//!
//! Moving a shape runs up to three independent policies, each in its own file and each toggled on
//! its own (or all together) from the toolbar:
//! - [`geometry`] — SnapToGeometry: the moved shape's bounding box lines up on another shape's /
//!   group's bounding box edges and corners (and on nodes/midpoints), x and y independently.
//! - [`center`] — SnapToCenter: the moved shape's centre lines up on another shape's centre.
//! - [`gap`] — SnapToGap: the moved shape takes the same gap to a neighbour as two other shapes
//!   have between them (equal spacing), with the measured distance shown in the guide.

use crate::geometry::dist;
use crate::Pt;

mod geometry;
mod center;
mod gap;

pub use gap::GapGuide;

/// What a point snapped to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapKind {
    /// A corner or node of a shape.
    Node,
    /// The middle of a shape.
    Centre,
    /// The middle of a straight segment.
    MidPoint,
    /// A grid intersection.
    Grid,
}

/// The largest angle step accepted. The original's settings-apply function (Spark
/// `sub_100785420`) resets the angle step unless it is between 0 and 90 degrees: it compares the
/// value against 0.0 and against the immediate `0x4056800000000000` = 90.0, and rejects anything
/// below 0 or above 90 (decompiled 2026-11-26).
pub const MAX_ANGLE_STEP: f64 = 90.0;

/// Which of the three move-snap policies are on. Each is an independent switch; any mix is valid,
/// and all three on at once is the usual "snap to everything".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapPolicies {
    /// SnapToGeometry (bounding box): edges, corners, nodes and midpoints line up.
    pub geometry: bool,
    /// SnapToCenter: centres line up.
    pub center: bool,
    /// SnapToGap: equal spacing to a neighbour.
    pub gap: bool,
}

impl Default for SnapPolicies {
    fn default() -> Self {
        SnapPolicies { geometry: true, center: true, gap: true }
    }
}

impl SnapPolicies {
    /// Whether any policy is on (so move-snapping does anything at all).
    pub fn any(self) -> bool {
        self.geometry || self.center || self.gap
    }
}

/// Snapping switches and distances. `radius` is in world units (the host converts its pixel setting
/// with the zoom, as the original converts its snap radius to whole pixels).
#[derive(Clone, Debug)]
pub struct Snapping {
    /// Which move-snap policies are on.
    pub policies: SnapPolicies,
    /// Grid spacing, when snapping to the grid.
    pub grid: Option<f64>,
    /// How close the pointer must be to snap. 0 switches snapping off.
    pub radius: f64,
}

impl Default for Snapping {
    fn default() -> Self {
        Snapping { policies: SnapPolicies::default(), grid: None, radius: 0.0 }
    }
}

/// One axis's pull: how far off it was (`gap`, for picking the closest), the line it caught
/// (`line`, an x for horizontal or a y for vertical) and what kind of point that was.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AxisPull {
    pub gap: f64,
    pub line: f64,
    pub kind: SnapKind,
}

impl AxisPull {
    /// Keep whichever of two candidates is closer (smaller gap).
    pub(crate) fn closer(a: Option<AxisPull>, b: Option<AxisPull>) -> Option<AxisPull> {
        match (a, b) {
            (Some(a), Some(b)) => Some(if a.gap <= b.gap { a } else { b }),
            (a, b) => a.or(b),
        }
    }
}

impl Snapping {
    /// The nearest snap point within the radius (for drawing tools, pointer-based): geometry first,
    /// then the grid. Honoured whenever the geometry OR centre policy is on.
    pub fn snap(&self, p: Pt, targets: &[(Pt, SnapKind)]) -> (Pt, Option<SnapKind>) {
        if self.radius <= 0.0 {
            return (p, None);
        }
        let want = |k: SnapKind| match k {
            SnapKind::Centre => self.policies.center,
            SnapKind::Node => self.policies.geometry,
            // segment midpoints are not owned by any policy, so they never snap
            SnapKind::MidPoint => false,
            SnapKind::Grid => true,
        };
        let points = targets.iter().copied().filter(|(_, k)| want(*k));
        if let Some((q, kind)) = points.map(|(q, k)| (q, k, dist(p, q))).filter(|(_, _, d)| *d <= self.radius).min_by(|a, b| a.2.total_cmp(&b.2)).map(|(q, k, _)| (q, k)) {
            return (q, Some(kind));
        }
        if let Some(step) = self.grid.filter(|s| *s > 0.0) {
            let g = [(p[0] / step).round() * step, (p[1] / step).round() * step];
            if dist(p, g) <= self.radius {
                return (g, Some(SnapKind::Grid));
            }
        }
        (p, None)
    }

    /// Snap a whole move, running each switched-on policy and keeping the closest pull on each axis.
    /// `sources` are the moved shapes' own snap points (already translated to the tentative
    /// position); `source_box` is their combined bounding box `(lo, hi)` at that position;
    /// `target_boxes` are the resting shapes'/groups' bounding boxes, for the gap policy. x and y
    /// snap independently so a shape far to the side still lines up on the same height.
    pub fn snap_move(&self, sources: &[(Pt, SnapKind)], source_box: (Pt, Pt), targets: &[(Pt, SnapKind)], target_boxes: &[(Pt, Pt)]) -> Option<MoveSnap> {
        if self.radius <= 0.0 || !self.policies.any() {
            return None;
        }
        let (mut px, mut py) = (None, None);
        if self.policies.geometry {
            let (x, y) = geometry::pull(sources, targets, self.radius);
            px = AxisPull::closer(px, x);
            py = AxisPull::closer(py, y);
        }
        if self.policies.center {
            let (x, y) = center::pull(sources, targets, self.radius);
            px = AxisPull::closer(px, x);
            py = AxisPull::closer(py, y);
        }
        let mut gap_guides: Vec<GapGuide> = vec![];
        if self.policies.gap {
            let out = gap::pull(source_box, target_boxes, self.radius);
            px = AxisPull::closer(px, out.x);
            py = AxisPull::closer(py, out.y);
            gap_guides = out.guides;
        }
        if px.is_none() && py.is_none() {
            return None;
        }
        // the offset that lands a source onto the caught column/row: pick the source nearest that
        // line, so the shape slides the least
        let offset_for = |line: f64, axis: usize| -> f64 {
            sources.iter().map(|(s, _)| line - s[axis]).min_by(|a, b| a.abs().total_cmp(&b.abs())).unwrap_or(0.0)
        };
        Some(MoveSnap {
            offset: [px.map_or(0.0, |p| offset_for(p.line, 0)), py.map_or(0.0, |p| offset_for(p.line, 1))],
            x_guide: px.map(|p| (p.line, p.kind)),
            y_guide: py.map(|p| (p.line, p.kind)),
            // the gap guides are only meaningful on the axis that actually took a gap pull
            gaps: gap_guides,
        })
    }
}

/// How a move snapped: an extra offset to apply, the caught column (x) and row (y) each for its
/// alignment guide, and any equal-gap guides to draw (with their measured distance). Any of these
/// may be empty/`None`.
#[derive(Clone, Debug, PartialEq)]
pub struct MoveSnap {
    pub offset: Pt,
    /// The x of the column the shape lined up on, and what kind of point it was.
    pub x_guide: Option<(f64, SnapKind)>,
    /// The y of the row the shape lined up on.
    pub y_guide: Option<(f64, SnapKind)>,
    /// Equal-spacing guides from the gap policy (arrow span + measured distance).
    pub gaps: Vec<GapGuide>,
}

/// `p` turned about `from` onto the nearest multiple of `step` degrees, keeping its distance.
/// Steps outside 0°..90° are ignored, as in the original (Spark `sub_100785420`).
pub fn snap_angle(from: Pt, p: Pt, step: f64) -> Pt {
    if !(step > 0.0 && step <= MAX_ANGLE_STEP) {
        return p;
    }
    let (dx, dy) = (p[0] - from[0], p[1] - from[1]);
    let len = (dx * dx + dy * dy).sqrt();
    let a = (dy.atan2(dx).to_degrees() / step).round() * step;
    let (s, c) = a.to_radians().sin_cos();
    [from[0] + len * c, from[1] + len * s]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_on(radius: f64) -> Snapping {
        Snapping { radius, ..Snapping::default() }
    }

    #[test]
    fn nearest_target_within_radius() {
        let s = all_on(2.0);
        let targets = [([10.0, 10.0], SnapKind::Node), ([11.5, 10.0], SnapKind::Centre)];
        assert_eq!(s.snap([11.2, 10.1], &targets), ([11.5, 10.0], Some(SnapKind::Centre)));
        assert_eq!(s.snap([20.0, 20.0], &targets), ([20.0, 20.0], None));
        let off = Snapping { radius: 0.0, ..s.clone() };
        assert_eq!(off.snap([10.1, 10.0], &targets).1, None, "radius 0 switches snapping off");
        let no_geometry = Snapping { policies: SnapPolicies { geometry: false, center: false, gap: false }, ..s };
        assert_eq!(no_geometry.snap([10.1, 10.0], &targets).1, None);
    }

    #[test]
    fn grid_snaps_when_policies_are_off() {
        let s = Snapping { policies: SnapPolicies { geometry: false, center: false, gap: false }, grid: Some(5.0), radius: 1.0 };
        assert_eq!(s.snap([14.6, 20.3], &[]), ([15.0, 20.0], Some(SnapKind::Grid)));
        assert_eq!(s.snap([12.5, 12.5], &[]).1, None, "too far from any grid point");
    }

    #[test]
    fn angle_steps_match_the_original_range() {
        // the original (Spark sub_100785420) accepts an angle step of 0 < s <= 90 and ignores the rest
        let p = snap_angle([0.0, 0.0], [10.0, 1.0], 15.0);
        assert!((p[0] - 10.0498756).abs() < 1e-6 && p[1].abs() < 1e-9, "flat onto 0°");
        let p = snap_angle([0.0, 0.0], [7.0, 7.4], 45.0);
        assert!((p[0] - p[1]).abs() < 1e-9, "diagonal onto 45°");
        assert_ne!(snap_angle([0.0, 0.0], [3.0, 4.0], 90.0), [3.0, 4.0], "90° is the top of the range and snaps");
        assert_ne!(snap_angle([0.0, 0.0], [3.0, 4.0], 89.0), [3.0, 4.0], "89° is inside the range and snaps");
        assert_eq!(snap_angle([0.0, 0.0], [3.0, 4.0], 91.0), [3.0, 4.0], "steps over 90° are refused");
        assert_eq!(snap_angle([0.0, 0.0], [3.0, 4.0], 0.0), [3.0, 4.0], "0 means no angle snap");
    }

    fn box_of(c: Pt, half: f64) -> (Pt, Pt) {
        ([c[0] - half, c[1] - half], [c[0] + half, c[1] + half])
    }

    #[test]
    fn snap_move_pulls_a_source_point_onto_the_nearest_target() {
        let s = all_on(2.0);
        // one moved corner sits (0.6, 0.3) away from a resting corner; both axes catch it
        let sources = [([9.4, 10.3], SnapKind::Node), ([20.0, 20.0], SnapKind::Centre)];
        let targets = [([10.0, 10.0], SnapKind::Node), ([50.0, 50.0], SnapKind::Centre)];
        let snap = s.snap_move(&sources, box_of([14.7, 15.15], 5.3), &targets, &[]).expect("should snap");
        assert_eq!(snap.x_guide, Some((10.0, SnapKind::Node)));
        assert_eq!(snap.y_guide, Some((10.0, SnapKind::Node)));
        assert!((snap.offset[0] - 0.6).abs() < 1e-9 && (snap.offset[1] + 0.3).abs() < 1e-9, "offset lands the corner exactly");
    }

    #[test]
    fn snap_move_aligns_on_one_axis_even_when_far_on_the_other() {
        // two shapes side by side at the same height but far apart horizontally: the y must still
        // snap (same height) while x does not catch at all — the bug the user reported
        let s = all_on(2.0);
        let sources = [([200.0, 10.4], SnapKind::Node)]; // far to the right, 0.4 off in height
        let targets = [([10.0, 10.0], SnapKind::Node)];
        let snap = s.snap_move(&sources, box_of([200.0, 10.4], 5.0), &targets, &[]).expect("should snap on height");
        assert_eq!(snap.x_guide, None, "x is nowhere near, so no vertical guide");
        assert_eq!(snap.y_guide, Some((10.0, SnapKind::Node)));
        assert!((snap.offset[0]).abs() < 1e-9 && (snap.offset[1] + 0.4).abs() < 1e-9, "only the height is corrected");
    }

    #[test]
    fn snap_move_is_off_without_policies_or_radius() {
        let sources = [([10.1, 10.0], SnapKind::Centre)];
        let targets = [([10.0, 10.0], SnapKind::Centre)];
        let bx = box_of([10.1, 10.0], 5.0);
        let none = SnapPolicies { geometry: false, center: false, gap: false };
        assert!(Snapping { policies: none, ..all_on(2.0) }.snap_move(&sources, bx, &targets, &[]).is_none());
        assert!(all_on(0.0).snap_move(&sources, bx, &targets, &[]).is_none());
    }

    #[test]
    fn only_center_policy_ignores_corner_pulls() {
        // a corner is 0.3 off a target node, but only the centre policy is on: the corner must not
        // pull; the centre (5 off) is out of radius, so nothing snaps
        let s = Snapping { policies: SnapPolicies { geometry: false, center: true, gap: false }, ..all_on(1.0) };
        let sources = [([9.7, 10.0], SnapKind::Node), ([15.0, 15.0], SnapKind::Centre)];
        let targets = [([10.0, 10.0], SnapKind::Node), ([20.0, 20.0], SnapKind::Centre)];
        assert!(s.snap_move(&sources, box_of([12.35, 12.5], 5.3), &targets, &[]).is_none());
    }
}
