//! SnapToGeometry (bounding box): the moved shape's own snap points line up on the resting shapes'
//! and groups' bounding-box geometry — corners and nodes. Each axis is corrected on its own, like
//! draw2d's `getCorrectionFor(cols,…)` / `getCorrectionFor(rows,…)`, so a shape far to the side can
//! still line up on the same height as another.
//!
//! Segment MIDPOINTS are deliberately NOT part of this policy: snapping to the middle of an edge is
//! its own thing (the user turns "snap to geometry" on and does not expect midpoints to be caught).

use super::{AxisPull, SnapKind};
use crate::Pt;

/// Whether a target point counts for this policy: corners and nodes only, never segment midpoints.
fn counts(kind: SnapKind) -> bool {
    matches!(kind, SnapKind::Node)
}

/// The closest x pull and y pull of any source point onto any geometry target, within `radius`.
pub(super) fn pull(sources: &[(Pt, SnapKind)], targets: &[(Pt, SnapKind)], radius: f64) -> (Option<AxisPull>, Option<AxisPull>) {
    let mut x: Option<AxisPull> = None;
    let mut y: Option<AxisPull> = None;
    for (s, _) in sources {
        for (t, kind) in targets {
            if !counts(*kind) {
                continue;
            }
            let gx = (s[0] - t[0]).abs();
            if gx <= radius && x.is_none_or(|b| gx < b.gap) {
                x = Some(AxisPull { gap: gx, line: t[0], kind: *kind });
            }
            let gy = (s[1] - t[1]).abs();
            if gy <= radius && y.is_none_or(|b| gy < b.gap) {
                y = Some(AxisPull { gap: gy, line: t[1], kind: *kind });
            }
        }
    }
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_corner_pulls_each_axis_but_a_centre_target_is_ignored() {
        let sources = [([9.7, 20.4], SnapKind::Node)];
        let targets = [([10.0, 20.0], SnapKind::Node), ([9.8, 20.1], SnapKind::Centre)];
        let (x, y) = pull(&sources, &targets, 1.0);
        assert_eq!(x.map(|p| p.line), Some(10.0));
        assert_eq!(y.map(|p| p.line), Some(20.0), "the centre target, though closer, does not count here");
    }

    #[test]
    fn a_segment_midpoint_target_is_ignored() {
        // a midpoint sits right under a source, but geometry must not catch it (the user turned
        // midpoint snapping off; only corners/nodes pull)
        let sources = [([10.0, 10.0], SnapKind::Node)];
        let targets = [([10.0, 10.0], SnapKind::MidPoint)];
        assert_eq!(pull(&sources, &targets, 1.0), (None, None));
    }
}
