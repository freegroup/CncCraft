//! Operations on whole shapes: combine (union), subtract, intersect, expand / shrink and
//! mirror a group. Each returns new shapes and leaves its inputs alone.

use crate::forms::{Flip, Shape};
use crate::region::Region;
use crate::Pt;

/// Everything covered by any of the shapes, as one shape.
pub fn union(shapes: &[Shape]) -> Option<Shape> {
    Region::of_all(shapes).to_shape()
}

/// `base` with every one of `cutters` cut away.
pub fn subtract(base: &Shape, cutters: &[Shape]) -> Option<Shape> {
    Region::of(base).difference(&Region::of_all(cutters)).to_shape()
}

/// Only what all the shapes cover.
pub fn intersect(shapes: &[Shape]) -> Option<Shape> {
    let mut it = shapes.iter();
    let first = Region::of(it.next()?);
    it.fold(first, |acc, s| acc.intersection(&Region::of(s))).to_shape()
}

/// The outline grown by `distance` (shrunk when it is negative).
pub fn offset(shape: &Shape, distance: f64) -> Option<Shape> {
    Region::of(shape).offset(distance).to_shape()
}

/// Grow by `distance`.
pub fn expand(shape: &Shape, distance: f64) -> Option<Shape> {
    offset(shape, distance.abs())
}

/// Shrink by `distance`.
pub fn shrink(shape: &Shape, distance: f64) -> Option<Shape> {
    offset(shape, -distance.abs())
}

/// Centre of the bounds of several shapes.
pub fn group_center(shapes: &[Shape]) -> Option<Pt> {
    let (lo, hi) = shapes.iter().filter_map(Shape::bounds).reduce(|(a, b), (c, d)| ([a[0].min(c[0]), a[1].min(c[1])], [b[0].max(d[0]), b[1].max(d[1])]))?;
    Some([(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0])
}

/// Mirror the shapes as a group, about the centre of their bounds.
pub fn mirror(shapes: &[Shape], flip: Flip) -> Vec<Shape> {
    let Some(axis) = group_center(shapes) else { return vec![] };
    shapes.iter().map(|s| s.mirrored(axis, flip)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::{Circle, Rect};

    fn area(s: &Shape) -> f64 {
        Region::of(s).area()
    }

    #[test]
    fn combine_subtract_intersect() {
        let a: Shape = Rect::new([5.0, 5.0], 10.0, 10.0).into();
        let b: Shape = Rect::new([10.0, 5.0], 10.0, 10.0).into();
        assert!((area(&union(&[a.clone(), b.clone()]).unwrap()) - 150.0).abs() < 1e-6);
        assert!((area(&subtract(&a, &[b.clone()]).unwrap()) - 50.0).abs() < 1e-6);
        assert!((area(&intersect(&[a.clone(), b.clone()]).unwrap()) - 50.0).abs() < 1e-6);
        // shapes that don't touch have nothing in common
        let far: Shape = Circle::new([100.0, 100.0], 3.0).into();
        assert!(intersect(&[a.clone(), far]).is_none());
        // cutting everything away leaves nothing
        assert!(subtract(&a, &[a.clone()]).is_none());
    }

    #[test]
    fn expand_shrink() {
        let a: Shape = Rect::new([0.0, 0.0], 10.0, 10.0).into();
        let (lo, hi) = shrink(&a, 1.0).unwrap().bounds().unwrap();
        assert!((hi[0] - lo[0] - 8.0).abs() < 1e-6);
        let (lo, hi) = expand(&a, 1.0).unwrap().bounds().unwrap();
        assert!((hi[0] - lo[0] - 12.0).abs() < 1e-3);
        assert!(shrink(&a, 6.0).is_none());
    }

    #[test]
    fn mirror_group_keeps_its_bounds() {
        // rect spans x −2..2, circle 9..11: the group's box is −2..11 with its middle at 4.5
        let shapes: Vec<Shape> = vec![Rect::new([0.0, 0.0], 4.0, 2.0).into(), Circle::new([10.0, 0.0], 1.0).into()];
        let m = mirror(&shapes, Flip::Horizontal);
        // the two swap sides within the same overall box
        assert!((m[0].center()[0] - 9.0).abs() < 1e-9);
        assert!((m[1].center()[0] - -1.0).abs() < 1e-9);
        let span = |v: &[Shape]| v.iter().filter_map(Shape::bounds).fold((f64::MAX, f64::MIN), |(lo, hi), (a, b)| (lo.min(a[0]), hi.max(b[0])));
        let (a, b) = (span(&shapes), span(&m));
        assert!((a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9);
    }
}
