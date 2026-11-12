//! Snapping: pulling the pointer onto nearby geometry (shape nodes, centres, segment
//! midpoints), onto extra points such as the job's corners and centre, onto a grid, and onto
//! fixed angle steps. Mirrors the snapping settings of the original (grid with spacing, geometry,
//! job centre and corners, angle step, snap radius).

use crate::geometry::dist;
use crate::Pt;

/// What a point snapped to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapKind {
    /// A corner or node of a shape.
    Node,
    /// The middle of a shape.
    Centre,
    /// The middle of a straight segment.
    MidPoint,
    /// A point the host added, such as the job's corners and centre.
    Job,
    /// A grid intersection.
    Grid,
}

/// The largest angle step accepted (original: 0x100785420 resets steps outside 0°..90°).
pub const MAX_ANGLE_STEP: f64 = 90.0;

/// Snapping switches and distances. `radius` is in world units (the host converts its pixel
/// setting with the zoom, as the original converts its snap radius to whole pixels).
#[derive(Clone, Debug)]
pub struct Snapping {
    /// Snap to shape nodes, centres and midpoints.
    pub geometry: bool,
    /// Grid spacing, when snapping to the grid.
    pub grid: Option<f64>,
    /// Extra points to snap to (job corners and centre).
    pub extra: Vec<Pt>,
    /// How close the pointer must be to snap. 0 switches snapping off.
    pub radius: f64,
}

impl Default for Snapping {
    fn default() -> Self {
        Snapping { geometry: true, grid: None, extra: vec![], radius: 0.0 }
    }
}

impl Snapping {
    /// The nearest snap point within the radius: geometry and extra points first, then the grid.
    pub fn snap(&self, p: Pt, targets: &[(Pt, SnapKind)]) -> (Pt, Option<SnapKind>) {
        if self.radius <= 0.0 {
            return (p, None);
        }
        let points = targets.iter().filter(|_| self.geometry).copied().chain(self.extra.iter().map(|q| (*q, SnapKind::Job)));
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
}

/// `p` turned about `from` onto the nearest multiple of `step` degrees, keeping its distance.
/// Steps outside 0°..90° are ignored, as in the original.
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

    #[test]
    fn nearest_target_within_radius() {
        let s = Snapping { radius: 2.0, ..Snapping::default() };
        let targets = [([10.0, 10.0], SnapKind::Node), ([11.5, 10.0], SnapKind::Centre)];
        assert_eq!(s.snap([11.2, 10.1], &targets), ([11.5, 10.0], Some(SnapKind::Centre)));
        assert_eq!(s.snap([20.0, 20.0], &targets), ([20.0, 20.0], None));
        let off = Snapping { radius: 0.0, ..s.clone() };
        assert_eq!(off.snap([10.1, 10.0], &targets).1, None, "radius 0 switches snapping off");
        let no_geometry = Snapping { geometry: false, ..s };
        assert_eq!(no_geometry.snap([10.1, 10.0], &targets).1, None);
    }

    #[test]
    fn job_points_and_grid() {
        let s = Snapping { geometry: false, grid: Some(5.0), extra: vec![[0.0, 0.0]], radius: 1.0 };
        assert_eq!(s.snap([0.4, -0.3], &[]), ([0.0, 0.0], Some(SnapKind::Job)));
        assert_eq!(s.snap([14.6, 20.3], &[]), ([15.0, 20.0], Some(SnapKind::Grid)));
        assert_eq!(s.snap([12.5, 12.5], &[]).1, None, "too far from any grid point");
    }

    #[test]
    fn angle_steps() {
        let p = snap_angle([0.0, 0.0], [10.0, 1.0], 15.0);
        assert!((p[0] - 10.0498756).abs() < 1e-6 && p[1].abs() < 1e-9, "flat");
        let p = snap_angle([0.0, 0.0], [7.0, 7.4], 45.0);
        assert!((p[0] - p[1]).abs() < 1e-9, "diagonal");
        assert_eq!(snap_angle([0.0, 0.0], [3.0, 4.0], 120.0), [3.0, 4.0], "steps over 90° are refused");
        assert_eq!(snap_angle([0.0, 0.0], [3.0, 4.0], 0.0), [3.0, 4.0]);
    }
}
