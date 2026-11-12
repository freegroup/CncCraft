//! The shape classes. Each kind of shape is its own type implementing [`Form`]; [`Shape`]
//! holds any of them. A transform keeps a shape's class while the result is still that class
//! (a circle scaled evenly stays a circle) and changes it when it isn't (a circle stretched
//! sideways becomes an ellipse; a rotated rectangle stretched sideways becomes a curve).

mod circle;
mod compound;
mod ellipse;
mod path;
mod polygon;
mod rect;
mod text;

pub use circle::Circle;
pub use compound::Compound;
pub use ellipse::Ellipse;
pub use path::Path;
pub use polygon::Polygon;
pub use rect::Rect;
pub use text::Text;

use crate::geometry::{bounds_of, Contour};
use crate::Pt;
use serde::{Deserialize, Serialize};

/// Which way to mirror.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flip {
    /// Left ↔ right, about a vertical line.
    Horizontal,
    /// Top ↔ bottom, about a horizontal line.
    Vertical,
}

/// One dimension a shape shows while it is drawn and right after: the size `dim` measured from
/// `from` to `to` (world coordinates). `outward` points away from the shape, where the
/// dimension line and its field go; a zero vector means "on the measured line itself" (a
/// radius drawn from the centre).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DimLine {
    pub dim: Dim,
    pub from: Pt,
    pub to: Pt,
    pub outward: Pt,
}

impl DimLine {
    pub fn length(&self) -> f64 {
        crate::geometry::dist(self.from, self.to)
    }
}

/// What a shape adds to itself while it is selected (its "selected decoration"): a handle for a
/// size the selection box can't change, such as a rectangle's corner radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grip {
    pub kind: GripKind,
    pub at: Pt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GripKind {
    /// Rounds a rectangle's corners.
    CornerRadius,
}

/// Shapes a person draws by dragging. Each class decides how a drag from `start` to `end`
/// becomes that shape (corner to corner, centre to edge, …).
pub trait Draw: Form + Sized {
    fn from_drag(start: Pt, end: Pt) -> Self;
}

/// What every shape class can do.
pub trait Form {
    /// Name shown to the user.
    fn label(&self) -> &'static str;
    /// Outlines in world coordinates.
    fn contours(&self) -> Vec<Contour>;
    /// The dimensions this shape shows while it is created (its "creation decoration"), in the
    /// order their fields are visited with Tab.
    fn decoration(&self) -> Vec<DimLine>;
    /// Extra handles while selected (its "selected decoration"). None by default.
    fn grips(&self) -> Vec<Grip> {
        vec![]
    }
    /// The shape with grip `kind` dragged to `p`, if it has that grip.
    fn with_grip(&self, _kind: GripKind, _p: Pt) -> Option<Shape> {
        None
    }
    fn translated(&self, dx: f64, dy: f64) -> Self
    where
        Self: Sized;
    /// Scale by (sx, sy) about the fixed world point `anchor`.
    fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Shape;
    /// Mirror about the line through `axis`.
    fn mirrored(&self, axis: Pt, flip: Flip) -> Shape;
}

/// Any shape.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Shape {
    Rect(Rect),
    Circle(Circle),
    Ellipse(Ellipse),
    Polygon(Polygon),
    Path(Path),
    Compound(Compound),
    Text(Text),
}

/// Run `$body` with `$f` bound to the inner class, whatever it is.
macro_rules! each {
    ($shape:expr, $f:ident => $body:expr) => {
        match $shape {
            Shape::Rect($f) => $body,
            Shape::Circle($f) => $body,
            Shape::Ellipse($f) => $body,
            Shape::Polygon($f) => $body,
            Shape::Path($f) => $body,
            Shape::Compound($f) => $body,
            Shape::Text($f) => $body,
        }
    };
}

macro_rules! from_class {
    ($($class:ident),*) => {
        $(impl From<$class> for Shape {
            fn from(f: $class) -> Shape {
                Shape::$class(f)
            }
        })*
    };
}
from_class!(Rect, Circle, Ellipse, Polygon, Path, Compound, Text);

impl Shape {
    pub fn label(&self) -> &'static str {
        each!(self, f => f.label())
    }
    pub fn contours(&self) -> Vec<Contour> {
        each!(self, f => f.contours())
    }
    pub fn translated(&self, dx: f64, dy: f64) -> Shape {
        each!(self, f => f.translated(dx, dy).into())
    }
    pub fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Shape {
        each!(self, f => f.scaled(anchor, sx, sy))
    }
    pub fn mirrored(&self, axis: Pt, flip: Flip) -> Shape {
        each!(self, f => f.mirrored(axis, flip))
    }
    pub fn bounds(&self) -> Option<(Pt, Pt)> {
        bounds_of(&self.contours())
    }
    /// Middle of the bounds.
    pub fn center(&self) -> Pt {
        self.bounds().map(|(lo, hi)| [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0]).unwrap_or([0.0, 0.0])
    }
    pub fn is_closed(&self) -> bool {
        self.contours().iter().all(|c| c.closed)
    }
    /// The same outline as editable curves (a path, or a compound for several outlines).
    pub fn to_curves(&self) -> Shape {
        curves_from(self.contours())
    }

    /// Turned by `deg` (anticlockwise) about `about`. Classes with their own rotation keep their
    /// class: the centre moves round `about` and the rotation grows by `deg`.
    pub fn rotated(&self, about: Pt, deg: f64) -> Shape {
        let turn = |c: Pt| rotate(c, about, deg);
        let path = |p: &Path| Path { points: p.points.iter().map(|q| turn(*q)).collect(), closed: p.closed };
        match self {
            Shape::Rect(r) => Rect { center: turn(r.center), rotation: r.rotation + deg, ..r.clone() }.into(),
            Shape::Circle(c) => Circle { center: turn(c.center), ..*c }.into(),
            Shape::Ellipse(e) => Ellipse { center: turn(e.center), rotation: e.rotation + deg, ..e.clone() }.into(),
            Shape::Polygon(p) => Polygon { center: turn(p.center), rotation: p.rotation + deg, ..p.clone() }.into(),
            Shape::Text(t) => Text { center: turn(t.center), rotation: t.rotation + deg, ..t.clone() }.into(),
            Shape::Path(p) => path(p).into(),
            Shape::Compound(c) => Compound { paths: c.paths.iter().map(path).collect() }.into(),
        }
    }

    /// The dimension lines this shape shows while it is created.
    pub fn decoration(&self) -> Vec<DimLine> {
        each!(self, f => f.decoration())
    }

    /// The extra handles this shape shows while it is selected.
    pub fn grips(&self) -> Vec<Grip> {
        each!(self, f => f.grips())
    }

    /// The shape with grip `kind` dragged to `p` (unchanged if it has no such grip).
    pub fn with_grip(&self, kind: GripKind, p: Pt) -> Shape {
        each!(self, f => f.with_grip(kind, p)).unwrap_or_else(|| self.clone())
    }

    /// The sizes a person can type in for this shape, in the unit of [`Dim::unit`]: what its
    /// dimension lines measure.
    pub fn dimensions(&self) -> Vec<(Dim, f64)> {
        self.decoration().iter().map(|l| (l.dim, self.dimension_value(l))).collect()
    }

    /// The value one dimension line shows.
    pub fn dimension_value(&self, line: &DimLine) -> f64 {
        line.length()
    }

    /// Like [`Shape::with_dimension`], but the lower-left corner of the shape's bounds stays where
    /// it was, so a larger height grows the shape upwards and a larger width to the right instead
    /// of both sides moving. This is what a size bound to a variable uses: change the variable and
    /// the top moves, not the whole shape.
    pub fn with_dimension_anchored(&self, dim: Dim, value: f64) -> Shape {
        let grown = self.with_dimension(dim, value);
        match (self.bounds(), grown.bounds()) {
            (Some((before, _)), Some((after, _))) => grown.translated(before[0] - after[0], before[1] - after[1]),
            _ => grown,
        }
    }

    /// The shape with one size set to `value` (kept about its centre). Sizes that aren't
    /// positive, or don't belong to the shape, leave it unchanged.
    pub fn with_dimension(&self, dim: Dim, value: f64) -> Shape {
        if !(value > 0.0 && value.is_finite()) {
            return self.clone();
        }
        let mut s = self.clone();
        match (&mut s, dim) {
            (Shape::Rect(r), Dim::Width) => r.width = value,
            (Shape::Rect(r), Dim::Height) => r.height = value,
            // diameter × 0.5, as the polygon tool applies a typed diameter (0x10085765c)
            (Shape::Circle(c), Dim::Diameter) => c.radius = value * 0.5,
            (Shape::Ellipse(e), Dim::Width) => e.rx = value / 2.0,
            (Shape::Ellipse(e), Dim::Height) => e.ry = value / 2.0,
            (Shape::Polygon(p), Dim::Radius) => p.radius = value,
            (Shape::Text(t), Dim::TextHeight) => t.height = value,
            (Shape::Path(_) | Shape::Compound(_), Dim::Width | Dim::Height) => {
                let Some((lo, hi)) = self.bounds() else { return s };
                let c = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
                let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
                return match dim {
                    Dim::Width if w > 1e-9 => self.scaled(c, value / w, 1.0),
                    Dim::Height if h > 1e-9 => self.scaled(c, 1.0, value / h),
                    _ => s,
                };
            }
            _ => {}
        }
        s
    }
}

/// A size of a shape that can be typed in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dim {
    Width,
    Height,
    Diameter,
    Radius,
    TextHeight,
}

impl Dim {
    /// The unit a value of this dimension is typed in.
    pub fn unit(self) -> &'static str {
        "mm"
    }
}

// ---------- helpers for the classes ----------

/// Width along the bottom edge and height along the right edge of a box of size (w, h) around
/// `center`, turned by `deg`: the decoration of rectangles, ellipses and curves.
pub(crate) fn box_lines(center: Pt, w: f64, h: f64, deg: f64) -> Vec<DimLine> {
    let at = |x: f64, y: f64| rotate([center[0] + x, center[1] + y], center, deg);
    let dir = |x: f64, y: f64| rotate([x, y], [0.0, 0.0], deg);
    let (hw, hh) = (w / 2.0, h / 2.0);
    vec![
        DimLine { dim: Dim::Width, from: at(-hw, -hh), to: at(hw, -hh), outward: dir(0.0, -1.0) },
        DimLine { dim: Dim::Height, from: at(hw, -hh), to: at(hw, hh), outward: dir(1.0, 0.0) },
    ]
}

/// Width and height of the outline's bounds, for shapes without their own sizes.
pub(crate) fn bounds_lines(contours: &[Contour]) -> Vec<DimLine> {
    match bounds_of(contours) {
        Some((lo, hi)) => box_lines([(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0], hi[0] - lo[0], hi[1] - lo[1], 0.0),
        None => vec![],
    }
}

pub(crate) fn rotate(p: Pt, about: Pt, deg: f64) -> Pt {
    let (s, c) = deg.to_radians().sin_cos();
    let (x, y) = (p[0] - about[0], p[1] - about[1]);
    [about[0] + x * c - y * s, about[1] + x * s + y * c]
}

/// Local points (around the origin) rotated by `deg` and moved to `center`.
pub(crate) fn place(local: impl IntoIterator<Item = Pt>, center: Pt, deg: f64) -> Vec<Pt> {
    local.into_iter().map(|p| rotate([center[0] + p[0], center[1] + p[1]], center, deg)).collect()
}

pub(crate) fn scale_point(p: Pt, anchor: Pt, sx: f64, sy: f64) -> Pt {
    [anchor[0] + (p[0] - anchor[0]) * sx, anchor[1] + (p[1] - anchor[1]) * sy]
}

pub(crate) fn flip_point(p: Pt, axis: Pt, flip: Flip) -> Pt {
    match flip {
        Flip::Horizontal => [2.0 * axis[0] - p[0], p[1]],
        Flip::Vertical => [p[0], 2.0 * axis[1] - p[1]],
    }
}

pub(crate) fn is_uniform(sx: f64, sy: f64) -> bool {
    (sx - sy).abs() < 1e-9
}

/// Whether a shape symmetric about its own axes is rotated by a multiple of 180°, so its width
/// still runs along X.
pub(crate) fn is_axis_aligned(deg: f64) -> bool {
    let r = deg.rem_euclid(180.0);
    r < 1e-9 || r > 180.0 - 1e-9
}

/// How many segments approximate a curve of radius `r` (finer for bigger radii). Always a
/// multiple of four, so the sample points are symmetric about both axes and a mirrored curve
/// has exactly the same points as the mirrored outline.
pub(crate) fn segments_for(r: f64) -> usize {
    let n = ((r.abs() * 2.0).sqrt() * 8.0).clamp(24.0, 180.0) as usize;
    n.div_ceil(4) * 4
}

/// Points round an ellipse with radii (rx, ry), starting on +X.
pub(crate) fn ellipse_points(rx: f64, ry: f64) -> Vec<Pt> {
    let n = segments_for(rx.max(ry));
    (0..n)
        .map(|i| {
            let a = i as f64 / n as f64 * std::f64::consts::TAU;
            [rx * a.cos(), ry * a.sin()]
        })
        .collect()
}

/// Curves made from contours: one path, or a compound for several.
pub(crate) fn curves_from(mut contours: Vec<Contour>) -> Shape {
    if contours.len() == 1 {
        let c = contours.remove(0);
        Shape::Path(Path { points: c.pts, closed: c.closed })
    } else {
        Shape::Compound(Compound { paths: contours.into_iter().map(|c| Path { points: c.pts, closed: c.closed }).collect() })
    }
}

/// Bake contours into curves, scaled about `anchor`.
pub(crate) fn curves_scaled(contours: Vec<Contour>, anchor: Pt, sx: f64, sy: f64) -> Shape {
    curves_from(contours.into_iter().map(|c| Contour { pts: c.pts.iter().map(|p| scale_point(*p, anchor, sx, sy)).collect(), closed: c.closed }).collect())
}

/// Bake contours into curves, mirrored about `axis`.
pub(crate) fn curves_mirrored(contours: Vec<Contour>, axis: Pt, flip: Flip) -> Shape {
    curves_from(contours.into_iter().map(|c| Contour { pts: c.pts.iter().map(|p| flip_point(*p, axis, flip)).collect(), closed: c.closed }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Pt, b: Pt) -> bool {
        (a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6
    }

    #[test]
    fn rect_keeps_class_when_axis_aligned() {
        let r = Shape::from(Rect::new([10.0, 10.0], 20.0, 10.0));
        let s = r.scaled([0.0, 0.0], 2.0, 3.0);
        let Shape::Rect(s) = s else { panic!("still a rectangle") };
        assert!(close(s.center, [20.0, 30.0]));
        assert_eq!((s.width, s.height), (40.0, 30.0));
    }

    #[test]
    fn rotated_rect_stretched_becomes_curve() {
        let r = Shape::from(Rect { rotation: 30.0, ..Rect::new([0.0, 0.0], 20.0, 10.0) });
        assert!(matches!(r.scaled([0.0, 0.0], 2.0, 1.0), Shape::Path(_)));
        assert!(matches!(r.scaled([0.0, 0.0], 2.0, 2.0), Shape::Rect(_)));
    }

    #[test]
    fn circle_stretched_becomes_ellipse() {
        let c = Shape::from(Circle::new([0.0, 0.0], 5.0));
        assert!(matches!(c.scaled([0.0, 0.0], 2.0, 2.0), Shape::Circle(Circle { radius, .. }) if radius == 10.0));
        let Shape::Ellipse(e) = c.scaled([0.0, 0.0], 2.0, 1.0) else { panic!("ellipse") };
        assert_eq!((e.rx, e.ry), (10.0, 5.0));
    }

    #[test]
    fn mirroring_matches_point_mirroring() {
        // Every class mirrored as a class must have the same outline as its outline mirrored.
        let shapes: Vec<Shape> = vec![
            Rect { rotation: 25.0, corner_radius: 2.0, ..Rect::new([3.0, 4.0], 20.0, 10.0) }.into(),
            Ellipse { rotation: 40.0, ..Ellipse::new([1.0, 2.0], 9.0, 4.0) }.into(),
            Polygon { rotation: 10.0, ..Polygon::new([5.0, 5.0], 8.0, 5) }.into(),
        ];
        for s in shapes {
            for flip in [Flip::Horizontal, Flip::Vertical] {
                let axis = [7.0, -3.0];
                let class = s.mirrored(axis, flip).bounds().unwrap();
                let pts = curves_mirrored(s.contours(), axis, flip).bounds().unwrap();
                assert!(close(class.0, pts.0) && close(class.1, pts.1), "{} {flip:?}", s.label());
                assert_eq!(std::mem::discriminant(&s.mirrored(axis, flip)), std::mem::discriminant(&s), "{} keeps its class", s.label());
            }
        }
    }

    #[test]
    fn rotating_keeps_the_class_and_matches_rotating_the_outline() {
        let shapes: Vec<Shape> = vec![
            Rect::new([3.0, 4.0], 20.0, 10.0).into(),
            Ellipse::new([1.0, 2.0], 9.0, 4.0).into(),
            Polygon::new([5.0, 5.0], 8.0, 5).into(),
            Path::new(vec![[0.0, 0.0], [4.0, 1.0], [2.0, 5.0]], true).into(),
        ];
        let (about, deg) = ([7.0, -3.0], 33.0);
        // a circle has no rotation of its own: only its centre moves
        let c = Shape::from(Circle::new([2.0, 1.0], 3.0)).rotated(about, deg);
        assert!(matches!(c, Shape::Circle(Circle { center, radius }) if close(center, rotate([2.0, 1.0], about, deg)) && radius == 3.0));
        for s in shapes {
            let turned = s.rotated(about, deg);
            assert_eq!(std::mem::discriminant(&turned), std::mem::discriminant(&s), "{} keeps its class", s.label());
            let by_class: Vec<Pt> = turned.contours().into_iter().flat_map(|c| c.pts).collect();
            let by_points: Vec<Pt> = s.contours().into_iter().flat_map(|c| c.pts).map(|p| rotate(p, about, deg)).collect();
            assert_eq!(by_class.len(), by_points.len());
            for (a, b) in by_class.iter().zip(&by_points) {
                assert!(close(*a, *b), "{}: {a:?} vs {b:?}", s.label());
            }
        }
    }

    #[test]
    fn each_class_draws_itself_from_a_drag_and_measures_itself() {
        let (a, b) = ([10.0, 20.0], [40.0, 60.0]);
        let r = Rect::from_drag(a, b);
        assert_eq!((r.center, r.width, r.height), ([25.0, 40.0], 30.0, 40.0));
        assert_eq!(Circle::from_drag(a, b).radius, 50.0);
        assert_eq!(Ellipse::from_drag(a, b).rx, 15.0);
        assert_eq!(Polygon::from_drag(a, b).radius, 50.0);

        // every decoration measures the shape's own sizes
        let shapes: Vec<(Shape, Vec<(Dim, f64)>)> = vec![
            (Rect { rotation: 30.0, ..Rect::new([0.0, 0.0], 20.0, 10.0) }.into(), vec![(Dim::Width, 20.0), (Dim::Height, 10.0)]),
            (Circle::new([1.0, 1.0], 4.0).into(), vec![(Dim::Diameter, 8.0)]),
            (Ellipse { rotation: 70.0, ..Ellipse::new([0.0, 0.0], 6.0, 2.0) }.into(), vec![(Dim::Width, 12.0), (Dim::Height, 4.0)]),
            (Polygon { rotation: 12.0, ..Polygon::new([3.0, 3.0], 7.0, 6) }.into(), vec![(Dim::Radius, 7.0)]),
            (Text { rotation: 45.0, ..Text::new([0.0, 0.0], "Ab", 25.0) }.into(), vec![(Dim::TextHeight, 25.0)]),
        ];
        for (s, want) in shapes {
            let got = s.dimensions();
            assert_eq!(got.len(), want.len(), "{}", s.label());
            for ((d1, v1), (d2, v2)) in got.iter().zip(&want) {
                assert_eq!(d1, d2, "{}", s.label());
                assert!((v1 - v2).abs() < 1e-9, "{}: {v1} vs {v2}", s.label());
            }
        }
        // a rotated rectangle's width line runs along its turned bottom edge, outward downwards
        let line = Shape::from(Rect { rotation: 90.0, ..Rect::new([0.0, 0.0], 20.0, 10.0) }).decoration()[0];
        assert!(close(line.from, [5.0, -10.0]) && close(line.to, [5.0, 10.0]) && close(line.outward, [1.0, 0.0]));
    }

    #[test]
    fn anchored_resize_keeps_the_lower_left_corner() {
        // lower-left corner at 10/30; the height changes, the corner must not move
        let r: Shape = Rect::new([10.0 + 10.0, 30.0 + 5.0], 20.0, 10.0).into();
        let taller = r.with_dimension_anchored(Dim::Height, 40.0);
        let (lo, hi) = taller.bounds().unwrap();
        assert!(close(lo, [10.0, 30.0]), "lower-left stays: {lo:?}");
        assert!(close(hi, [30.0, 70.0]), "the top moves up: {hi:?}");
        // the plain version moves both edges about the centre
        let (lo2, _) = r.with_dimension(Dim::Height, 40.0).bounds().unwrap();
        assert!(!close(lo2, [10.0, 30.0]));
        // width grows to the right
        let (lo3, hi3) = r.with_dimension_anchored(Dim::Width, 50.0).bounds().unwrap();
        assert!(close(lo3, [10.0, 30.0]) && close(hi3, [60.0, 40.0]));
        // works for the other classes too
        let c: Shape = Circle::new([5.0, 5.0], 5.0).into();
        let (lo4, _) = c.with_dimension_anchored(Dim::Diameter, 30.0).bounds().unwrap();
        assert!(close(lo4, [0.0, 0.0]));
    }

    #[test]
    fn typed_dimensions() {
        let r: Shape = Rect::new([0.0, 0.0], 20.0, 10.0).into();
        assert_eq!(r.dimensions(), vec![(Dim::Width, 20.0), (Dim::Height, 10.0)]);
        assert!(matches!(r.with_dimension(Dim::Width, 50.0), Shape::Rect(Rect { width, height, .. }) if width == 50.0 && height == 10.0));
        assert_eq!(r.with_dimension(Dim::Width, -5.0), r, "sizes must be positive");
        assert_eq!(r.with_dimension(Dim::Diameter, 5.0), r, "a rectangle has no diameter");

        let c: Shape = Circle::new([0.0, 0.0], 5.0).into();
        assert_eq!(c.dimensions(), vec![(Dim::Diameter, 10.0)]);
        assert!(matches!(c.with_dimension(Dim::Diameter, 30.0), Shape::Circle(Circle { radius, .. }) if radius == 15.0));

        // curves are sized through their box, about its middle
        let p: Shape = Path::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 4.0]], false).into();
        let (lo, hi) = p.with_dimension(Dim::Width, 30.0).bounds().unwrap();
        assert!(close(lo, [-10.0, 0.0]) && close(hi, [20.0, 4.0]));
    }

    #[test]
    fn text_has_outlines_and_mirrors_to_curves() {
        let t = Shape::from(Text::new([0.0, 0.0], "Ab", 20.0));
        let (lo, hi) = t.bounds().unwrap();
        assert!((hi[1] - lo[1]) > 15.0 && (hi[0] - lo[0]) > 15.0);
        assert!(matches!(t.mirrored([0.0, 0.0], Flip::Horizontal), Shape::Compound(_)));
        assert!(matches!(t.scaled([0.0, 0.0], 2.0, 2.0), Shape::Text(Text { height, .. }) if height == 40.0));
    }
}
