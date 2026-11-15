//! Trim: a vector is cut at every place where it crosses another one, and falls apart into
//! separate polylines.

use crate::geometry::{dist, Contour};
use crate::Pt;

/// Where segment `a0`–`a1` crosses segment `b0`–`b1`: the fraction along the first, if they cross.
fn crossing(a0: Pt, a1: Pt, b0: Pt, b1: Pt) -> Option<f64> {
    let (r, s) = ([a1[0] - a0[0], a1[1] - a0[1]], [b1[0] - b0[0], b1[1] - b0[1]]);
    let den = r[0] * s[1] - r[1] * s[0];
    if den.abs() < 1e-12 {
        return None;
    }
    let q = [b0[0] - a0[0], b0[1] - a0[1]];
    let t = (q[0] * s[1] - q[1] * s[0]) / den;
    let u = (q[0] * r[1] - q[1] * r[0]) / den;
    ((-1e-9..=1.0 + 1e-9).contains(&t) && (-1e-9..=1.0 + 1e-9).contains(&u)).then_some(t.clamp(0.0, 1.0))
}

/// The arc length at each point of a polyline (and at the closing point, for a closed one).
fn lengths(c: &Contour) -> Vec<f64> {
    let n = c.pts.len();
    let segs = if c.closed { n } else { n - 1 };
    let mut out = vec![0.0];
    for i in 0..segs {
        out.push(out[i] + dist(c.pts[i], c.pts[(i + 1) % n]));
    }
    out
}

/// The positions (arc length along `c`) where it crosses any of `others`, sorted, without
/// repeats, and without the very ends of an open polyline.
pub fn crossings(c: &Contour, others: &[Contour]) -> Vec<f64> {
    let n = c.pts.len();
    if n < 2 {
        return vec![];
    }
    let len = lengths(c);
    let total = *len.last().unwrap();
    let mut out = vec![];
    for i in 0..len.len() - 1 {
        let (a0, a1) = (c.pts[i], c.pts[(i + 1) % n]);
        for o in others {
            for (b0, b1) in o.segments() {
                if let Some(t) = crossing(a0, a1, b0, b1) {
                    out.push(len[i] + t * (len[i + 1] - len[i]));
                }
            }
        }
    }
    out.sort_by(|a, b| a.partial_cmp(b).unwrap());
    out.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    if c.closed {
        // the start and the end of a ring are the same place
        if out.len() > 1 && (out[0] + total - out[out.len() - 1]).abs() < 1e-6 {
            out.pop();
        }
    } else {
        out.retain(|s| *s > 1e-6 && *s < total - 1e-6);
    }
    out
}

/// The point at arc length `s`.
fn point_at(c: &Contour, len: &[f64], s: f64) -> Pt {
    let n = c.pts.len();
    let i = (0..len.len() - 1).find(|&k| s <= len[k + 1]).unwrap_or(len.len() - 2);
    let (a, b) = (c.pts[i], c.pts[(i + 1) % n]);
    let f = if len[i + 1] > len[i] { (s - len[i]) / (len[i + 1] - len[i]) } else { 0.0 };
    [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]
}

/// The stretch of `c` from arc length `from` to `to` (on a ring it may run over the start).
fn stretch(c: &Contour, len: &[f64], from: f64, to: f64) -> Vec<Pt> {
    let n = c.pts.len();
    let total = *len.last().unwrap();
    let wraps = c.closed && to <= from;
    let end = if wraps { to + total } else { to };
    let mut out = vec![point_at(c, len, from)];
    // every point strictly between the two ends, going once more round a ring where needed
    for lap in 0..if wraps { 2 } else { 1 } {
        for (k, l) in len.iter().enumerate() {
            let s = l + lap as f64 * total;
            if s > from + 1e-9 && s < end - 1e-9 {
                out.push(c.pts[k % n]);
            }
        }
    }
    out.push(point_at(c, len, to));
    out.dedup_by(|a, b| dist(*a, *b) < 1e-9);
    out
}

/// `c` cut at every crossing with `others`, as open polylines in its own direction. Nothing when
/// it crosses nothing.
pub fn split_at_crossings(c: &Contour, others: &[Contour]) -> Vec<Contour> {
    let cuts = crossings(c, others);
    if cuts.is_empty() {
        return vec![];
    }
    let len = lengths(c);
    let total = *len.last().unwrap();
    let mut bounds: Vec<(f64, f64)> = vec![];
    if c.closed {
        for k in 0..cuts.len() {
            bounds.push((cuts[k], cuts[(k + 1) % cuts.len()]));
        }
    } else {
        let mut at = 0.0;
        for &s in &cuts {
            bounds.push((at, s));
            at = s;
        }
        bounds.push((at, total));
    }
    bounds.into_iter().map(|(a, b)| Contour::open(stretch(c, &len, a, b))).filter(|p| p.pts.len() >= 2).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(a: Pt, b: Pt) -> Contour {
        Contour::open(vec![a, b])
    }

    #[test]
    fn an_open_line_falls_apart_at_each_crossing() {
        let c = Contour::open(vec![[0.0, 0.0], [10.0, 0.0], [20.0, 0.0]]);
        let others = [line([5.0, -5.0], [5.0, 5.0]), line([15.0, -5.0], [15.0, 5.0])];
        let parts = split_at_crossings(&c, &others);
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].pts, vec![[0.0, 0.0], [5.0, 0.0]]);
        assert_eq!(parts[1].pts, vec![[5.0, 0.0], [10.0, 0.0], [15.0, 0.0]]);
        assert_eq!(parts[2].pts, vec![[15.0, 0.0], [20.0, 0.0]]);
        assert!(parts.iter().all(|p| !p.closed));
    }

    #[test]
    fn a_ring_cut_twice_becomes_two_open_pieces_over_its_start() {
        let ring = Contour::closed(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]);
        let parts = split_at_crossings(&ring, &[line([5.0, -5.0], [5.0, 15.0])]);
        assert_eq!(parts.len(), 2);
        // one piece runs over the ring's first point
        assert!(parts.iter().any(|p| p.pts.contains(&[0.0, 0.0])));
        assert!(parts.iter().any(|p| p.pts.contains(&[10.0, 10.0])));
        let total: f64 = parts.iter().map(|p| p.segments().map(|(a, b)| dist(a, b)).sum::<f64>()).sum();
        assert!((total - 40.0).abs() < 1e-9, "nothing lost: {total}");
    }

    #[test]
    fn nothing_crossed_nothing_cut() {
        let c = Contour::open(vec![[0.0, 0.0], [10.0, 0.0]]);
        assert!(split_at_crossings(&c, &[line([0.0, 5.0], [10.0, 5.0])]).is_empty());
        // touching at its own end is no cut
        assert!(split_at_crossings(&c, &[line([10.0, -5.0], [10.0, 5.0])]).is_empty());
    }
}
