//! Filled areas made from closed outlines, with boolean operations (union, difference,
//! intersection, xor) and offsetting (expand / shrink).

use crate::forms::{Compound, Path, Shape};
use crate::geometry::{point_in_ring, Contour};
use crate::Pt;
use geo::{Area, BooleanOps, Buffer, Contains, Coord, LineString, MultiPolygon, Polygon, Simplify};

/// One boundary of a region: an outer edge or the edge of a hole.
#[derive(Clone, Debug, PartialEq)]
pub struct Ring {
    pub pts: Vec<Pt>,
    pub hole: bool,
}

/// A filled area. Outlines nested inside each other alternate between solid and hole (even-odd),
/// so letters like "O" keep their hole.
#[derive(Clone, Debug)]
pub struct Region(MultiPolygon<f64>);

impl Default for Region {
    fn default() -> Self {
        Region::empty()
    }
}

fn line(pts: &[Pt]) -> LineString<f64> {
    let mut v: Vec<Coord<f64>> = pts.iter().map(|p| Coord { x: p[0], y: p[1] }).collect();
    if let Some(f) = v.first().copied() {
        v.push(f);
    }
    LineString::new(v)
}

fn points(ls: &LineString<f64>) -> Vec<Pt> {
    let mut v: Vec<Pt> = ls.0.iter().map(|c| [c.x, c.y]).collect();
    if v.len() > 1 && v.first() == v.last() {
        v.pop();
    }
    v
}

impl Region {
    pub fn empty() -> Self {
        Region(MultiPolygon::new(vec![]))
    }

    /// The area enclosed by the closed contours (open ones are ignored).
    pub fn from_contours(contours: &[Contour]) -> Self {
        let rings: Vec<&Vec<Pt>> = contours.iter().filter(|c| c.closed && c.pts.len() >= 3).map(|c| &c.pts).collect();
        // how many other rings surround each ring: even = solid, odd = hole
        let depth: Vec<usize> = rings.iter().enumerate().map(|(i, r)| rings.iter().enumerate().filter(|(j, o)| *j != i && point_in_ring(r[0], o)).count()).collect();
        let mut polys: Vec<(usize, Polygon<f64>)> = rings.iter().enumerate().filter(|(i, _)| depth[*i] % 2 == 0).map(|(i, r)| (i, Polygon::new(line(r), vec![]))).collect();
        for (i, r) in rings.iter().enumerate().filter(|(i, _)| depth[*i] % 2 == 1) {
            // the hole belongs to the innermost solid ring around it
            let owner = polys.iter().rposition(|(oi, _)| depth[*oi] + 1 == depth[i] && point_in_ring(r[0], rings[*oi]));
            if let Some(k) = owner {
                polys[k].1.interiors_push(line(r));
            }
        }
        polys.into_iter().fold(Region::empty(), |acc, (_, p)| acc.union(&Region(MultiPolygon::new(vec![p]))))
    }

    pub fn of(shape: &Shape) -> Self {
        Region::from_contours(&shape.contours())
    }

    /// The area a round cutter of radius `r` sweeps while its centre travels along `polylines`:
    /// each polyline is buffered by `r` (a capsule with round caps and joins) and the results are
    /// unioned into one real area. Drawing this once avoids the stacked-transparency artefacts of
    /// painting many overlapping strokes. Empty when `r <= 0` or there are no segments.
    pub fn swept(polylines: &[Vec<Pt>], r: f64) -> Self {
        if r <= 0.0 {
            return Region::empty();
        }
        let lines: Vec<LineString<f64>> = polylines
            .iter()
            .filter(|p| p.len() >= 2)
            .map(|p| LineString::new(p.iter().map(|q| Coord { x: q[0], y: q[1] }).collect()))
            .collect();
        if lines.is_empty() {
            return Region::empty();
        }
        // buffer every stroke, then union them all into one area
        Region(geo::MultiLineString::new(lines).buffer(r)).simplify_in_place(r * 0.02)
    }

    fn simplify_in_place(self, eps: f64) -> Self {
        Region(self.0.simplify(eps.max(1e-6)))
    }

    /// Everything covered by any of the shapes.
    pub fn of_all<'a>(shapes: impl IntoIterator<Item = &'a Shape>) -> Self {
        shapes.into_iter().fold(Region::empty(), |acc, s| acc.union(&Region::of(s)))
    }

    pub fn union(&self, other: &Region) -> Region {
        if self.is_empty() {
            return other.clone();
        }
        Region(self.0.union(&other.0))
    }

    pub fn difference(&self, other: &Region) -> Region {
        Region(self.0.difference(&other.0))
    }

    pub fn intersection(&self, other: &Region) -> Region {
        Region(self.0.intersection(&other.0))
    }

    pub fn xor(&self, other: &Region) -> Region {
        Region(self.0.xor(&other.0))
    }

    /// Grow the area by `d` (shrink when `d` is negative). Corners round off when growing.
    pub fn offset(&self, d: f64) -> Region {
        if d.abs() < 1e-12 {
            return self.clone();
        }
        Region(self.0.buffer(d).simplify(d.abs() * 0.01))
    }

    pub fn is_empty(&self) -> bool {
        self.0 .0.is_empty() || self.area() < 1e-12
    }

    pub fn area(&self) -> f64 {
        self.0.unsigned_area()
    }

    pub fn contains(&self, p: Pt) -> bool {
        self.0.contains(&geo::Point::new(p[0], p[1]))
    }

    /// Outer edges and hole edges.
    pub fn rings(&self) -> Vec<Ring> {
        let mut out = Vec::new();
        for poly in &self.0 .0 {
            out.push(Ring { pts: points(poly.exterior()), hole: false });
            out.extend(poly.interiors().iter().map(|i| Ring { pts: points(i), hole: true }));
        }
        out.retain(|r| r.pts.len() >= 3);
        out
    }

    /// The region's edges as a shape: one closed path, or a compound for several edges.
    pub fn to_shape(&self) -> Option<Shape> {
        let mut paths: Vec<Path> = self.rings().into_iter().map(|r| Path::new(r.pts, true)).collect();
        match paths.len() {
            0 => None,
            1 => Some(Shape::Path(paths.remove(0))),
            _ => Some(Shape::Compound(Compound::new(paths))),
        }
    }

    /// The region as a list of filled triangles (earcut, holes handled), for drawing it as one
    /// solid translucent area instead of many overlapping strokes.
    pub fn triangles(&self) -> Vec<[Pt; 3]> {
        use geo::TriangulateEarcut;
        let mut out = Vec::new();
        for poly in &self.0 .0 {
            for t in poly.earcut_triangles_iter() {
                out.push([[t.v1().x, t.v1().y], [t.v2().x, t.v2().y], [t.v3().x, t.v3().y]]);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::{Circle, Rect, Text};

    fn square(x: f64, y: f64, size: f64) -> Shape {
        Rect::new([x + size / 2.0, y + size / 2.0], size, size).into()
    }

    fn near(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn boolean_operations_on_overlapping_squares() {
        let a = Region::of(&square(0.0, 0.0, 10.0));
        let b = Region::of(&square(5.0, 0.0, 10.0));
        assert!(near(a.union(&b).area(), 150.0, 1e-6));
        assert!(near(a.intersection(&b).area(), 50.0, 1e-6));
        assert!(near(a.difference(&b).area(), 50.0, 1e-6));
        assert!(near(a.xor(&b).area(), 100.0, 1e-6));
        assert!(a.difference(&a).is_empty());
    }

    #[test]
    fn separate_shapes_stay_separate_and_holes_stay_holes() {
        let two = Region::of_all(&[square(0.0, 0.0, 10.0), square(20.0, 0.0, 10.0)]);
        assert_eq!(two.rings().len(), 2);
        assert!(matches!(two.to_shape(), Some(Shape::Compound(_))));

        // a square with a square hole cut out of it
        let ring = Region::of(&square(0.0, 0.0, 10.0)).difference(&Region::of(&square(3.0, 3.0, 4.0)));
        assert!(near(ring.area(), 84.0, 1e-6));
        assert_eq!(ring.rings().iter().filter(|r| r.hole).count(), 1);
        assert!(!ring.contains([5.0, 5.0]));
        assert!(ring.contains([1.0, 1.0]));
        // rebuilt from its own edges it is the same area again (even-odd nesting)
        assert!(near(Region::of(&ring.to_shape().unwrap()).area(), 84.0, 1e-6));
    }

    #[test]
    fn expand_and_shrink() {
        let sq = Region::of(&square(0.0, 0.0, 10.0));
        let shrunk = sq.offset(-2.0);
        assert!(near(shrunk.area(), 36.0, 1e-6));
        // growing a square by 1 adds four 10×1 strips and four quarter circles of radius 1
        let grown = sq.offset(1.0);
        assert!(near(grown.area(), 100.0 + 40.0 + std::f64::consts::PI, 0.1));
        assert!(sq.offset(-6.0).is_empty());
    }

    #[test]
    fn circle_area_and_letter_holes() {
        let c = Region::of(&Circle::new([0.0, 0.0], 10.0).into());
        assert!(near(c.area(), std::f64::consts::PI * 100.0, 3.0));
        let o = Region::of(&Text::new([0.0, 0.0], "O", 30.0).into());
        assert!(o.rings().iter().any(|r| r.hole), "the O keeps its hole");
    }

    #[test]
    fn swept_area_is_a_capsule_unioned_once() {
        // a 10 mm straight pass, cutter radius 1: a 10×2 rectangle plus two half-discs of r=1.
        // Two passes over the same line must give the SAME area (union, not double-painted).
        let one = Region::swept(&[vec![[0.0, 0.0], [10.0, 0.0]]], 1.0);
        let two = Region::swept(&[vec![[0.0, 0.0], [10.0, 0.0]], vec![[0.0, 0.0], [10.0, 0.0]]], 1.0);
        let expected = 10.0 * 2.0 + std::f64::consts::PI; // body + two caps
        assert!(near(one.area(), expected, 0.3), "one pass ≈ {expected}, got {}", one.area());
        assert!(near(two.area(), one.area(), 0.3), "a repeated pass does not grow the area");
        assert!(Region::swept(&[vec![[0.0, 0.0], [10.0, 0.0]]], 0.0).is_empty(), "no radius, no band");
    }
}
