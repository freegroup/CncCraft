//! The eight resize handles around a selection and the point each one scales about.

use crate::geometry::dist;
use crate::{Editable, Pt};

/// A resize handle on the selection box. Names follow what the user sees: T is the top edge
/// (largest Y, because world Y points up).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

impl Handle {
    pub const ALL: [Handle; 8] = [
        Handle::TopLeft,
        Handle::Top,
        Handle::TopRight,
        Handle::Right,
        Handle::BottomRight,
        Handle::Bottom,
        Handle::BottomLeft,
        Handle::Left,
    ];

    /// Which axes this handle resizes: (horizontal, vertical).
    pub fn axes(self) -> (bool, bool) {
        match self {
            Handle::Left | Handle::Right => (true, false),
            Handle::Top | Handle::Bottom => (false, true),
            _ => (true, true),
        }
    }

    /// Position on the unit box: x and y each 0 (low), 0.5 (middle) or 1 (high).
    fn unit(self) -> (f64, f64) {
        match self {
            Handle::TopLeft => (0.0, 1.0),
            Handle::Top => (0.5, 1.0),
            Handle::TopRight => (1.0, 1.0),
            Handle::Right => (1.0, 0.5),
            Handle::BottomRight => (1.0, 0.0),
            Handle::Bottom => (0.5, 0.0),
            Handle::BottomLeft => (0.0, 0.0),
            Handle::Left => (0.0, 0.5),
        }
    }
}

/// Bounds of the shapes whose ids are in `selection`.
pub fn selection_bounds<S: Editable>(shapes: &[S], selection: &[u64]) -> Option<(Pt, Pt)> {
    let mut acc: Option<(Pt, Pt)> = None;
    for (lo, hi) in shapes.iter().filter(|s| selection.contains(&s.id())).filter_map(Editable::bounds) {
        acc = Some(match acc {
            None => (lo, hi),
            Some((a, b)) => ([a[0].min(lo[0]), a[1].min(lo[1])], [b[0].max(hi[0]), b[1].max(hi[1])]),
        });
    }
    acc
}

/// World position of a handle on the box `lo`–`hi` grown by `pad` on every side.
pub fn handle_point(lo: Pt, hi: Pt, pad: f64, h: Handle) -> Pt {
    let (ux, uy) = h.unit();
    let (l, t) = ([lo[0] - pad, lo[1] - pad], [hi[0] + pad, hi[1] + pad]);
    [l[0] + (t[0] - l[0]) * ux, l[1] + (t[1] - l[1]) * uy]
}

/// The point that stays put while `h` is dragged: the handle on the opposite side.
pub fn anchor_for(lo: Pt, hi: Pt, h: Handle) -> Pt {
    let (ux, uy) = h.unit();
    [lo[0] + (hi[0] - lo[0]) * (1.0 - ux), lo[1] + (hi[1] - lo[1]) * (1.0 - uy)]
}

/// The handle within `tol` of `p`, nearest first.
pub fn handle_at(lo: Pt, hi: Pt, pad: f64, p: Pt, tol: f64) -> Option<Handle> {
    Handle::ALL
        .into_iter()
        .map(|h| (h, dist(handle_point(lo, hi, pad, h), p)))
        .filter(|(_, d)| *d <= tol)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(h, _)| h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors_are_opposite() {
        let (lo, hi) = ([0.0, 0.0], [10.0, 20.0]);
        assert_eq!(anchor_for(lo, hi, Handle::TopLeft), [10.0, 0.0]);
        assert_eq!(anchor_for(lo, hi, Handle::BottomRight), [0.0, 20.0]);
        assert_eq!(anchor_for(lo, hi, Handle::Left), [10.0, 10.0]);
        assert_eq!(anchor_for(lo, hi, Handle::Top), [5.0, 0.0]);
    }

    #[test]
    fn finds_nearest_handle() {
        let (lo, hi) = ([0.0, 0.0], [10.0, 10.0]);
        assert_eq!(handle_at(lo, hi, 0.0, [10.4, 9.8], 1.0), Some(Handle::TopRight));
        assert_eq!(handle_at(lo, hi, 0.0, [5.0, 5.0], 1.0), None);
        assert_eq!(handle_at(lo, hi, 1.0, [-1.0, 5.0], 0.5), Some(Handle::Left));
    }
}
