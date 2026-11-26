//! SnapToGap (equal spacing / in-between), after draw2d's `SnapToInBetweenEditPolicy`: while a shape
//! is dragged between others, it snaps so its gap to a neighbour equals another gap on the same
//! axis, and every matching gap is painted as a labelled arrow. Two cases, per axis:
//!
//! - **Centred between two neighbours**: one box on each side; snap so the gap to the left equals
//!   the gap to the right. Both of the moved box's gaps are painted.
//! - **Equal to an existing gap**: a pair of other boxes already has a gap between them; snap so the
//!   moved box repeats that gap against a neighbour. Both the moved box's new gap AND the reference
//!   gap between the other two boxes are painted, so it is clear they are equal.
//!
//! Only boxes that overlap the moved box on the OTHER axis count as neighbours (they are actually
//! side by side), as in a real layout.

use super::AxisPull;
use crate::Pt;

/// An equal-spacing guide to draw: a double arrow from `from` to `to` (world points) with the gap
/// distance `len` in millimetres, labelled at the arrow's middle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GapGuide {
    pub from: Pt,
    pub to: Pt,
    pub len: f64,
}

/// The gap pulls and the guides to draw for them.
pub(super) struct GapOut {
    pub x: Option<AxisPull>,
    pub y: Option<AxisPull>,
    pub guides: Vec<GapGuide>,
}

/// One axis of a box: its low edge, high edge, and the span on the other axis (lo, hi) used to tell
/// whether two boxes are side by side, and the centre of that cross span (where its gap arrows sit).
#[derive(Clone, Copy)]
struct Span {
    lo: f64,
    hi: f64,
    cross_lo: f64,
    cross_hi: f64,
}

impl Span {
    fn cross_mid(&self) -> f64 {
        (self.cross_lo + self.cross_hi) / 2.0
    }
}

fn spans(boxes: &[(Pt, Pt)], axis: usize) -> Vec<Span> {
    let other = 1 - axis;
    boxes.iter().map(|(lo, hi)| Span { lo: lo[axis], hi: hi[axis], cross_lo: lo[other], cross_hi: hi[other] }).collect()
}

fn overlaps(a_lo: f64, a_hi: f64, b_lo: f64, b_hi: f64) -> bool {
    a_lo < b_hi && b_lo < a_hi
}

/// A gap to paint, along the moving axis: from `a` to `b`, drawn at `cross` on the other axis.
struct Segment {
    a: f64,
    b: f64,
    cross: f64,
}

/// Snap `moved` so it is equally spaced from its neighbours. Returns the new LOW edge and the gap
/// segments to paint (the moved box's own gaps, plus any reference gap it matched).
fn axis_pull(moved_lo: f64, moved_hi: f64, m_cross_mid: f64, m_cross_lo: f64, m_cross_hi: f64, others: &[Span], radius: f64) -> Option<(f64, Vec<Segment>)> {
    // neighbours actually beside the moved box on the other axis
    let near: Vec<Span> = others.iter().copied().filter(|s| overlaps(m_cross_lo, m_cross_hi, s.cross_lo, s.cross_hi)).collect();
    let width = moved_hi - moved_lo;
    // candidate: (offset from current position, new low edge, the segments to paint)
    let mut best: Option<(f64, f64, Vec<Segment>)> = None;
    let mut consider = |new_lo: f64, segs: Vec<Segment>| {
        let off = (new_lo - moved_lo).abs();
        if off <= radius && best.as_ref().is_none_or(|b| off < b.0) {
            best = Some((off, new_lo, segs));
        }
    };
    // centred between a left and a right neighbour: paint the moved box's two equal gaps
    for l in &near {
        for r in &near {
            if l.hi > r.lo {
                continue; // l must be left of r
            }
            let free = r.lo - l.hi;
            let gap = (free - width) / 2.0;
            if gap > 0.0 {
                let new_lo = l.hi + gap;
                let new_hi = new_lo + width;
                consider(new_lo, vec![
                    Segment { a: l.hi, b: new_lo, cross: m_cross_mid },
                    Segment { a: new_hi, b: r.lo, cross: m_cross_mid },
                ]);
            }
        }
    }
    // equal to an existing gap between two other boxes (a left of b): repeat it against a neighbour,
    // and paint BOTH the moved box's new gap and the reference gap between a and b
    for a in &near {
        for b in &near {
            if a.hi > b.lo {
                continue;
            }
            let g = b.lo - a.hi; // the reference gap
            if g <= 0.0 {
                continue;
            }
            let reference = |cross: f64| Segment { a: a.hi, b: b.lo, cross };
            // the moved box just to the right of b, gap g
            consider(b.hi + g, vec![
                Segment { a: b.hi, b: b.hi + g, cross: m_cross_mid },
                reference(a.cross_mid()),
            ]);
            // or just to the left of a, gap g
            let new_lo = a.lo - g - width;
            consider(new_lo, vec![
                Segment { a: new_lo + width, b: a.lo, cross: m_cross_mid },
                reference(a.cross_mid()),
            ]);
        }
    }
    best.map(|(_, new_lo, segs)| (new_lo, segs))
}

pub(super) fn pull(source_box: (Pt, Pt), target_boxes: &[(Pt, Pt)], radius: f64) -> GapOut {
    let mut out = GapOut { x: None, y: None, guides: vec![] };
    if target_boxes.len() < 2 {
        return out; // equal spacing needs at least two other boxes
    }
    let (lo, hi) = source_box;
    for axis in [0usize, 1usize] {
        let other = 1 - axis;
        let sp = spans(target_boxes, axis);
        let m_cross_mid = (lo[other] + hi[other]) / 2.0;
        if let Some((new_lo, segs)) = axis_pull(lo[axis], hi[axis], m_cross_mid, lo[other], hi[other], &sp, radius) {
            out_set(&mut out, axis, AxisPull { gap: (new_lo - lo[axis]).abs(), line: new_lo, kind: super::SnapKind::MidPoint });
            for s in segs {
                push_gap(&mut out.guides, axis, s.a, s.b, s.cross);
            }
        }
    }
    out
}

fn out_set(out: &mut GapOut, axis: usize, pull: AxisPull) {
    if axis == 0 {
        out.x = Some(pull);
    } else {
        out.y = Some(pull);
    }
}

/// A gap arrow from `a` to `b` along `axis`, drawn at `cross` on the other axis.
fn push_gap(guides: &mut Vec<GapGuide>, axis: usize, a: f64, b: f64, cross: f64) {
    let len = (b - a).abs();
    if len <= 1e-9 {
        return;
    }
    let point = |v: f64| if axis == 0 { [v, cross] } else { [cross, v] };
    guides.push(GapGuide { from: point(a), to: point(b), len });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bx(lo: Pt, hi: Pt) -> (Pt, Pt) {
        (lo, hi)
    }

    #[test]
    fn centres_a_box_between_two_neighbours() {
        // two 10-wide boxes at x 0..10 and 90..100, same height band 0..10; a 10-wide moved box
        // dropped a little off centre should snap to x 45..55 (gap 35 on each side)
        let left = bx([0.0, 0.0], [10.0, 10.0]);
        let right = bx([90.0, 0.0], [100.0, 10.0]);
        let moved = bx([46.0, 0.0], [56.0, 10.0]);
        let out = pull(moved, &[left, right], 3.0);
        let x = out.x.expect("should snap on x");
        assert!((x.line - 45.0).abs() < 1e-9, "left edge snaps to 45, got {}", x.line);
        assert_eq!(out.guides.len(), 2, "a gap arrow on each side");
        assert!(out.guides.iter().all(|g| (g.len - 35.0).abs() < 1e-9), "both gaps are 35");
    }

    #[test]
    fn far_apart_on_cross_axis_does_not_count_as_neighbour() {
        // the two boxes are on a different height band, so the moved box is not between them
        let left = bx([0.0, 0.0], [10.0, 10.0]);
        let right = bx([90.0, 0.0], [100.0, 10.0]);
        let moved = bx([46.0, 50.0], [56.0, 60.0]);
        let out = pull(moved, &[left, right], 3.0);
        assert!(out.x.is_none(), "not side by side with either box");
    }

    #[test]
    fn repeats_an_existing_gap_and_paints_both() {
        // boxes a (0..10) and b (20..30) have a gap of 10; the moved box near x 40 should snap to
        // x 40..50 (gap 10 right of b) and paint two arrows: its own new gap and the reference gap
        let a = bx([0.0, 0.0], [10.0, 10.0]);
        let b = bx([20.0, 0.0], [30.0, 10.0]);
        let moved = bx([39.0, 0.0], [49.0, 10.0]);
        let out = pull(moved, &[a, b], 2.0);
        let x = out.x.expect("should snap on x");
        assert!((x.line - 40.0).abs() < 1e-9, "snaps to x=40 (gap 10 right of b), got {}", x.line);
        assert_eq!(out.guides.len(), 2, "the new gap and the reference gap");
        assert!(out.guides.iter().all(|g| (g.len - 10.0).abs() < 1e-9), "both gaps are 10");
    }
}

