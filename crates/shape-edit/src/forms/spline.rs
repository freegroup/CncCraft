use super::{bounds_lines, flip_point, scale_point, DimLine, Flip, Form, Shape};
use crate::geometry::Contour;
use crate::Pt;
use serde::{Deserialize, Serialize};

/// A cubic-Bezier spline: each node has a position and two control handles (in/out), plus a
/// flag saying whether it is smooth (handles are collinear through the point) or a corner
/// (handles are independent, both collapsed onto the point).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spline {
    pub nodes: Vec<SplineNode>,
    pub closed: bool,
}

/// One node of a [`Spline`].
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SplineNode {
    /// The point on the curve.
    pub pos: Pt,
    /// Incoming control handle (world coordinates).
    pub cp_in: Pt,
    /// Outgoing control handle (world coordinates).
    pub cp_out: Pt,
    /// Smooth: handles mirror each other through `pos`.  Corner: handles sit on `pos`.
    pub smooth: bool,
}

impl SplineNode {
    /// A smooth node: control handles are placed at ±`arm` along the tangent direction.
    pub fn smooth(pos: Pt, cp_in: Pt, cp_out: Pt) -> Self {
        SplineNode { pos, cp_in, cp_out, smooth: true }
    }

    /// A corner node: both handles collapse onto the position.
    pub fn corner(pos: Pt) -> Self {
        SplineNode { pos, cp_in: pos, cp_out: pos, smooth: false }
    }
}

impl Spline {
    pub fn new(nodes: Vec<SplineNode>, closed: bool) -> Self {
        Spline { nodes, closed }
    }

    /// Build a spline from just positions, auto-computing smooth handles.
    pub fn from_points(pts: &[Pt], closed: bool) -> Self {
        let n = pts.len();
        if n < 2 {
            return Spline { nodes: pts.iter().map(|p| SplineNode::corner(*p)).collect(), closed };
        }
        let nodes: Vec<SplineNode> = (0..n)
            .map(|i| {
                let prev = if i == 0 {
                    if closed { pts[n - 1] } else { pts[0] }
                } else {
                    pts[i - 1]
                };
                let next = if i == n - 1 {
                    if closed { pts[0] } else { pts[n - 1] }
                } else {
                    pts[i + 1]
                };
                let p = pts[i];
                let dx = next[0] - prev[0];
                let dy = next[1] - prev[1];
                let len = (dx * dx + dy * dy).sqrt().max(1e-12);
                let d_in = crate::geometry::dist(prev, p);
                let d_out = crate::geometry::dist(p, next);
                let arm_in = d_in * SMOOTHING;
                let arm_out = d_out * SMOOTHING;
                let ux = dx / len;
                let uy = dy / len;
                SplineNode::smooth(
                    p,
                    [p[0] - ux * arm_in, p[1] - uy * arm_in],
                    [p[0] + ux * arm_out, p[1] + uy * arm_out],
                )
            })
            .collect();
        Spline { nodes, closed }
    }

    pub fn min_nodes(&self) -> usize {
        if self.closed { 3 } else { 2 }
    }

    /// Evaluate one cubic Bezier segment from `a` to `b` into `steps` line segments.
    fn eval_segment(a: &SplineNode, b: &SplineNode, pts: &mut Vec<Pt>) {
        let steps = STEPS_PER_SEGMENT;
        for i in 1..=steps {
            let t = i as f64 / steps as f64;
            pts.push(cubic(a.pos, a.cp_out, b.cp_in, b.pos, t));
        }
    }

    /// The polyline approximation of the whole spline.
    pub fn evaluate(&self) -> Vec<Pt> {
        let n = self.nodes.len();
        if n == 0 {
            return vec![];
        }
        if n == 1 {
            return vec![self.nodes[0].pos];
        }
        let mut pts = vec![self.nodes[0].pos];
        let segs = if self.closed { n } else { n - 1 };
        for i in 0..segs {
            let a = &self.nodes[i];
            let b = &self.nodes[(i + 1) % n];
            Self::eval_segment(a, b, &mut pts);
        }
        pts
    }

    fn transform_nodes(&self, f: impl Fn(Pt) -> Pt) -> Vec<SplineNode> {
        self.nodes
            .iter()
            .map(|n| SplineNode {
                pos: f(n.pos),
                cp_in: f(n.cp_in),
                cp_out: f(n.cp_out),
                smooth: n.smooth,
            })
            .collect()
    }
}

const SMOOTHING: f64 = 0.3;
const STEPS_PER_SEGMENT: usize = 20;

fn cubic(p0: Pt, p1: Pt, p2: Pt, p3: Pt, t: f64) -> Pt {
    let u = 1.0 - t;
    let (u2, t2) = (u * u, t * t);
    let (u3, t3) = (u2 * u, t2 * t);
    [
        u3 * p0[0] + 3.0 * u2 * t * p1[0] + 3.0 * u * t2 * p2[0] + t3 * p3[0],
        u3 * p0[1] + 3.0 * u2 * t * p1[1] + 3.0 * u * t2 * p2[1] + t3 * p3[1],
    ]
}

impl Form for Spline {
    fn label(&self) -> &'static str {
        if self.closed {
            "Geschlossener Spline"
        } else {
            "Offener Spline"
        }
    }

    fn contours(&self) -> Vec<Contour> {
        vec![Contour { pts: self.evaluate(), closed: self.closed }]
    }

    fn decoration(&self) -> Vec<DimLine> {
        bounds_lines(&self.contours())
    }

    fn translated(&self, dx: f64, dy: f64) -> Self {
        Spline {
            nodes: self.transform_nodes(|p| [p[0] + dx, p[1] + dy]),
            closed: self.closed,
        }
    }

    fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Shape {
        Spline {
            nodes: self.transform_nodes(|p| scale_point(p, anchor, sx, sy)),
            closed: self.closed,
        }
        .into()
    }

    fn mirrored(&self, axis: Pt, flip: Flip) -> Shape {
        Spline {
            nodes: self.transform_nodes(|p| flip_point(p, axis, flip)),
            closed: self.closed,
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_points_builds_smooth_handles() {
        let pts = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]];
        let s = Spline::from_points(&pts, false);
        assert_eq!(s.nodes.len(), 3);
        assert!(s.nodes.iter().all(|n| n.smooth));
        let poly = s.evaluate();
        assert!(poly.len() > 3);
        assert_eq!(poly[0], [0.0, 0.0]);
    }

    #[test]
    fn corner_nodes_give_straight_segments() {
        let s = Spline::new(
            vec![SplineNode::corner([0.0, 0.0]), SplineNode::corner([10.0, 0.0])],
            false,
        );
        let pts = s.evaluate();
        for p in &pts {
            assert!((p[1]).abs() < 1e-12, "y should be zero: {p:?}");
        }
    }

    #[test]
    fn closed_spline_wraps() {
        let pts = vec![[0.0, 0.0], [10.0, 0.0], [5.0, 10.0]];
        let s = Spline::from_points(&pts, true);
        let poly = s.evaluate();
        assert!(poly.len() > pts.len());
        let last = *poly.last().unwrap();
        let first = poly[0];
        assert!((last[0] - first[0]).abs() < 1e-6 && (last[1] - first[1]).abs() < 1e-6);
    }

    #[test]
    fn translation_preserves_handles() {
        let s = Spline::from_points(&[[0.0, 0.0], [10.0, 0.0]], false);
        let t = s.translated(5.0, 3.0);
        assert_eq!(t.nodes[0].pos, [5.0, 3.0]);
        assert_eq!(t.nodes[1].pos, [15.0, 3.0]);
        assert!(t.nodes[0].smooth);
    }
}
