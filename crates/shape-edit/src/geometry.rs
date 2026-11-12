//! Small geometry helpers on points and polylines, shared by the shape classes, the regions
//! and the host's toolpath code.

use crate::Pt;
use serde::{Deserialize, Serialize};

/// One outline of a shape in world coordinates: a polyline, closed or open.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contour {
    pub pts: Vec<Pt>,
    pub closed: bool,
}

impl Contour {
    pub fn closed(pts: Vec<Pt>) -> Self {
        Contour { pts, closed: true }
    }
    pub fn open(pts: Vec<Pt>) -> Self {
        Contour { pts, closed: false }
    }
    /// The segments as index pairs (wrapping round for closed contours).
    pub fn segments(&self) -> impl Iterator<Item = (Pt, Pt)> + '_ {
        let n = self.pts.len();
        let m = if self.closed { n } else { n.saturating_sub(1) };
        (0..m).map(move |i| (self.pts[i], self.pts[(i + 1) % n]))
    }
}

pub fn dist(a: Pt, b: Pt) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// Distance from `p` to the segment `a`–`b`.
pub fn dist_to_segment(p: Pt, a: Pt, b: Pt) -> f64 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let l2 = d[0] * d[0] + d[1] * d[1];
    let t = if l2 < 1e-18 { 0.0 } else { (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / l2).clamp(0.0, 1.0) };
    dist(p, [a[0] + t * d[0], a[1] + t * d[1]])
}

/// Axis-aligned bounds of all points of the contours.
pub fn bounds_of(contours: &[Contour]) -> Option<(Pt, Pt)> {
    let mut pts = contours.iter().flat_map(|c| c.pts.iter());
    let first = *pts.next()?;
    Some(pts.fold((first, first), |(lo, hi), p| ([lo[0].min(p[0]), lo[1].min(p[1])], [hi[0].max(p[0]), hi[1].max(p[1])])))
}

/// Signed area of a closed ring: positive when anticlockwise.
pub fn signed_area(pts: &[Pt]) -> f64 {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>()
        / 2.0
}

/// Length along the polyline (back to the start when `closed`).
pub fn perimeter(pts: &[Pt], closed: bool) -> f64 {
    Contour { pts: pts.to_vec(), closed }.segments().map(|(a, b)| dist(a, b)).sum()
}

/// Whether `p` lies inside the closed ring (even-odd rule).
pub fn point_in_ring(p: Pt, ring: &[Pt]) -> bool {
    let mut inside = false;
    let n = ring.len();
    let mut j = n.wrapping_sub(1);
    for i in 0..n {
        let (a, b) = (ring[i], ring[j]);
        if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0] {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_perimeter_and_containment() {
        let sq = [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]];
        assert_eq!(signed_area(&sq), 4.0);
        assert_eq!(perimeter(&sq, true), 8.0);
        assert_eq!(perimeter(&sq, false), 6.0);
        assert!(point_in_ring([1.0, 1.0], &sq));
        assert!(!point_in_ring([3.0, 1.0], &sq));
        assert_eq!(dist_to_segment([1.0, 1.0], [0.0, 0.0], [2.0, 0.0]), 1.0);
        assert_eq!(bounds_of(&[Contour::closed(sq.to_vec())]), Some(([0.0, 0.0], [2.0, 2.0])));
    }
}
