//! Slot (trace) tool: cut along open or closed vectors, on the line or offset to one side by
//! the tool radius. For open vectors, the caps control whether the cutter extends past the
//! endpoints (Outer) or stops so its edge meets them (Inner).

use crate::geom::{dist, Pt};
use crate::toolpath::{ring_dir, CutContext, Out, Side, ToolpathKind};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Cap {
    #[default]
    Outer,
    Inner,
}

pub struct SlotTool {
    pub side: Side,
    pub climb: bool,
    pub caps: Cap,
}

impl ToolpathKind for SlotTool {
    fn label(&self) -> String {
        t!("Nut")
    }

    fn cut(&self, out: &mut Out, cx: &CutContext) -> Option<String> {
        let depths = cx.depths();
        if depths.is_empty() {
            return None;
        }
        let r = cx.radius();
        let off = match self.side {
            Side::Outside => r,
            Side::Inside => -r,
            Side::On => 0.0,
        };

        let closed = cx.closed();
        if off.abs() > 1e-9 && !closed.is_empty() {
            let region = crate::geom::Region::from_contours(&closed);
            let rings = region.offset(off).rings();
            for ring in &rings {
                let pts = ring_dir(ring, self.climb);
                for (i, &z) in depths.iter().enumerate() {
                    out.set_top(cx.top_of(&depths, i));
                    out.cut_ring(&pts, z, None);
                }
                out.retract();
            }
        } else {
            for c in cx.contours.iter().filter(|c| c.closed) {
                let pts = c.pts.clone();
                for (i, &z) in depths.iter().enumerate() {
                    out.set_top(cx.top_of(&depths, i));
                    out.cut_ring(&pts, z, None);
                }
                out.retract();
            }
        }

        for c in cx.contours.iter().filter(|c| !c.closed && c.pts.len() >= 2) {
            let pts = if off.abs() > 1e-9 {
                offset_open(&c.pts, off)
            } else {
                c.pts.clone()
            };
            let pts = self.apply_caps(&pts, r);
            for (i, &z) in depths.iter().enumerate() {
                out.set_top(cx.top_of(&depths, i));
                if i % 2 == 0 {
                    out.cut_open(&pts, z);
                } else {
                    let rev: Vec<Pt> = pts.iter().rev().copied().collect();
                    out.cut_open(&rev, z);
                }
            }
            out.retract();
        }
        None
    }
}

impl SlotTool {
    fn apply_caps(&self, pts: &[Pt], radius: f64) -> Vec<Pt> {
        if pts.len() < 2 || radius < 1e-9 {
            return pts.to_vec();
        }
        match self.caps {
            Cap::Outer => pts.to_vec(),
            Cap::Inner => trim_ends(pts, radius),
        }
    }
}

/// Trim `radius` from each end of the polyline so the tool edge, not its centre, meets the
/// original endpoint.
fn trim_ends(pts: &[Pt], radius: f64) -> Vec<Pt> {
    if pts.len() < 2 {
        return pts.to_vec();
    }
    let mut out = pts.to_vec();

    // trim front
    let mut trimmed = 0.0;
    while trimmed < radius && out.len() >= 2 {
        let seg = dist(out[0], out[1]);
        let remaining = radius - trimmed;
        if seg <= remaining + 1e-9 {
            trimmed += seg;
            out.remove(0);
        } else {
            let f = remaining / seg;
            out[0] = [
                out[0][0] + (out[1][0] - out[0][0]) * f,
                out[0][1] + (out[1][1] - out[0][1]) * f,
            ];
            break;
        }
    }

    // trim back
    let n = out.len();
    if n < 2 {
        return out;
    }
    trimmed = 0.0;
    while trimmed < radius && out.len() >= 2 {
        let last = out.len() - 1;
        let seg = dist(out[last - 1], out[last]);
        let remaining = radius - trimmed;
        if seg <= remaining + 1e-9 {
            trimmed += seg;
            out.pop();
        } else {
            let f = remaining / seg;
            let prev = out[out.len() - 2];
            let end = out.last_mut().unwrap();
            *end = [
                end[0] + (prev[0] - end[0]) * f,
                end[1] + (prev[1] - end[1]) * f,
            ];
            break;
        }
    }
    out
}

/// Offset an open polyline by `d` to the left (positive) or right (negative). Each segment is
/// shifted perpendicular to its direction; consecutive shifted segments are joined at their
/// intersection or connected with a bevel.
fn offset_open(pts: &[Pt], d: f64) -> Vec<Pt> {
    if pts.len() < 2 {
        return pts.to_vec();
    }
    let normals: Vec<[f64; 2]> = pts
        .windows(2)
        .map(|w| {
            let dx = w[1][0] - w[0][0];
            let dy = w[1][1] - w[0][1];
            let len = (dx * dx + dy * dy).sqrt().max(1e-12);
            [-dy / len * d, dx / len * d]
        })
        .collect();

    let shifted: Vec<([f64; 2], [f64; 2])> = pts
        .windows(2)
        .zip(&normals)
        .map(|(w, n)| {
            ([w[0][0] + n[0], w[0][1] + n[1]], [w[1][0] + n[0], w[1][1] + n[1]])
        })
        .collect();

    let mut out = Vec::with_capacity(pts.len());
    out.push(shifted[0].0);
    for i in 0..shifted.len() - 1 {
        let (_, b1) = shifted[i];
        let (a2, _) = shifted[i + 1];
        if let Some(p) = line_intersect(shifted[i].0, b1, a2, shifted[i + 1].1) {
            out.push(p);
        } else {
            out.push(b1);
        }
    }
    out.push(shifted.last().unwrap().1);
    out
}

fn line_intersect(a1: Pt, a2: Pt, b1: Pt, b2: Pt) -> Option<Pt> {
    let (d1x, d1y) = (a2[0] - a1[0], a2[1] - a1[1]);
    let (d2x, d2y) = (b2[0] - b1[0], b2[1] - b1[1]);
    let denom = d1x * d2y - d1y * d2x;
    if denom.abs() < 1e-12 {
        return None;
    }
    let t = ((b1[0] - a1[0]) * d2y - (b1[1] - a1[1]) * d2x) / denom;
    Some([a1[0] + d1x * t, a1[1] + d1y * t])
}
