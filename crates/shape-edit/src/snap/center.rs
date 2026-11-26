//! SnapToCenter: the moved shape's centre lines up on another shape's or group's centre. Like
//! draw2d's `SnapToCenterEditPolicy`, which snaps the dragged figure's bounding-box centre onto the
//! centres of the other figures, x and y independently.

use super::{AxisPull, SnapKind};
use crate::Pt;

/// The closest x pull and y pull of a source CENTRE onto a target CENTRE, within `radius`.
pub(super) fn pull(sources: &[(Pt, SnapKind)], targets: &[(Pt, SnapKind)], radius: f64) -> (Option<AxisPull>, Option<AxisPull>) {
    let mut x: Option<AxisPull> = None;
    let mut y: Option<AxisPull> = None;
    for (s, sk) in sources {
        if *sk != SnapKind::Centre {
            continue;
        }
        for (t, tk) in targets {
            if *tk != SnapKind::Centre {
                continue;
            }
            let gx = (s[0] - t[0]).abs();
            if gx <= radius && x.is_none_or(|b| gx < b.gap) {
                x = Some(AxisPull { gap: gx, line: t[0], kind: SnapKind::Centre });
            }
            let gy = (s[1] - t[1]).abs();
            if gy <= radius && y.is_none_or(|b| gy < b.gap) {
                y = Some(AxisPull { gap: gy, line: t[1], kind: SnapKind::Centre });
            }
        }
    }
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centre_snaps_to_centre_only() {
        let sources = [([9.6, 9.6], SnapKind::Centre), ([0.0, 0.0], SnapKind::Node)];
        let targets = [([10.0, 10.0], SnapKind::Centre), ([9.5, 9.5], SnapKind::Node)];
        let (x, y) = pull(&sources, &targets, 1.0);
        assert_eq!((x.map(|p| p.line), y.map(|p| p.line)), (Some(10.0), Some(10.0)));
        // a node source must not catch the centre target
        let only_node = [([9.6, 9.6], SnapKind::Node)];
        assert_eq!(pull(&only_node, &targets, 1.0), (None, None));
    }
}
