//! Writing SVG drawings in millimetres or inches (1 user unit = 1 mm or 1 inch). Layers become
//! `<g>` elements that Inkscape shows as layers, groups become nested `<g>` elements inside them,
//! circles stay circles.

use super::{outlines, Drawing, Outline, Piece, Unit};
use std::collections::HashSet;
use std::fmt::Write;

/// Which area the picture covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Area {
    /// The whole plate, so the shapes keep their place on it.
    Plate,
    /// Just the shapes.
    Shapes,
}

impl Area {
    pub const ALL: [Area; 2] = [Area::Plate, Area::Shapes];

    pub fn name(self) -> &'static str {
        match self {
            Area::Plate => "Werkstück",
            Area::Shapes => "Außenrahmen der Formen",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Options {
    pub unit: Unit,
    pub area: Area,
    /// Line width (mm).
    pub stroke: f64,
    /// Draw the plate's edge as a rectangle.
    pub frame: bool,
    /// Each layer is a `<g>` (an Inkscape layer).
    pub layers: bool,
    /// Each group of the drawing is a `<g>`.
    pub groups: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options { unit: Unit::Mm, area: Area::Plate, stroke: 0.2, frame: false, layers: true, groups: true }
    }
}

fn num(v: f64) -> String {
    let s = format!("{:.4}", v);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".into()
    } else {
        s.to_string()
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// An id XML accepts, unique in the file.
fn unique_id(base: &str, used: &mut HashSet<String>) -> String {
    let mut id: String = base.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect();
    if !id.starts_with(|c: char| c.is_alphabetic() || c == '_') {
        id.insert(0, '_');
    }
    let (stem, mut n) = (id.clone(), 2);
    while !used.insert(id.clone()) {
        id = format!("{stem}-{n}");
        n += 1;
    }
    id
}

struct Writer<'a> {
    out: String,
    o: &'a Options,
    /// Drawing y (up) to picture y (down): `top - y`; and the left edge.
    top: f64,
    left: f64,
    /// Millimetres to the file's unit.
    k: f64,
    used: HashSet<String>,
    /// Groups numbered in the order they appear in the file.
    group_numbers: Vec<u64>,
}

impl Writer<'_> {
    fn pt(&self, p: [f64; 2]) -> String {
        format!("{},{}", num((p[0] - self.left) * self.k), num((self.top - p[1]) * self.k))
    }

    /// An SVG arc from `a` to `b` for a segment with this DXF bulge. The picture's y points down,
    /// so an anticlockwise arc (positive bulge) is drawn with the sweep flag set.
    fn arc(&self, a: [f64; 2], b: [f64; 2], bulge: f64) -> String {
        let sweep = 4.0 * bulge.atan();
        let chord = shape_edit::geometry::dist(a, b);
        let r = chord / (2.0 * (sweep.abs() / 2.0).sin());
        let large = u8::from(sweep.abs() > std::f64::consts::PI);
        let flag = u8::from(sweep > 0.0);
        format!(" A{} {} 0 {large} {flag} {}", num(r * self.k), num(r * self.k), self.pt(b))
    }

    fn shape(&mut self, piece: &Piece, indent: usize) {
        for outline in outlines(&piece.shape) {
            let pad = "  ".repeat(indent);
            match outline {
                Outline::Circle { center, radius } => {
                    let _ = writeln!(self.out, "{pad}<circle cx=\"{}\" cy=\"{}\" r=\"{}\"/>", num((center[0] - self.left) * self.k), num((self.top - center[1]) * self.k), num(radius * self.k));
                }
                Outline::Line { pts, closed, bulges } => {
                    let mut d = format!("M{}", self.pt(pts[0]));
                    let n = pts.len();
                    let segs = if closed { n } else { n - 1 };
                    for i in 0..segs {
                        let (a, b) = (pts[i], pts[(i + 1) % n]);
                        let bulge = bulges.get(i).copied().unwrap_or(0.0);
                        if bulge.abs() > 1e-12 {
                            d.push_str(&self.arc(a, b, bulge));
                        } else if i + 1 < n {
                            d.push_str(" L");
                            d.push_str(&self.pt(b));
                        }
                    }
                    if closed {
                        d.push_str(" Z");
                    }
                    let _ = writeln!(self.out, "{pad}<path d=\"{d}\"/>");
                }
            }
        }
    }

    /// The pieces in their order; those sharing a group at `depth` (counted from the outermost)
    /// go together into one `<g>`, where the first of them stands.
    fn pieces(&mut self, pieces: &[&Piece], depth: usize, indent: usize) {
        let mut done: HashSet<u64> = HashSet::new();
        for piece in pieces {
            let group = if self.o.groups { piece.groups.get(depth).copied() } else { None };
            let Some(group) = group else {
                self.shape(piece, indent);
                continue;
            };
            if !done.insert(group) {
                continue;
            }
            let members: Vec<&Piece> = pieces.iter().copied().filter(|p| p.groups.get(depth) == Some(&group)).collect();
            let number = match self.group_numbers.iter().position(|&g| g == group) {
                Some(i) => i + 1,
                None => {
                    self.group_numbers.push(group);
                    self.group_numbers.len()
                }
            };
            let id = unique_id(&format!("gruppe-{number}"), &mut self.used);
            let pad = "  ".repeat(indent);
            let _ = writeln!(self.out, "{pad}<g id=\"{id}\">");
            self.pieces(&members, depth + 1, indent + 1);
            let _ = writeln!(self.out, "{pad}</g>");
        }
    }
}

pub fn write(drawing: &Drawing, o: &Options) -> String {
    let (lo, hi) = match (o.area, drawing.bounds()) {
        (Area::Shapes, Some((lo, hi))) => {
            let m = o.stroke / 2.0;
            ([lo[0] - m, lo[1] - m], [hi[0] + m, hi[1] + m])
        }
        _ => ([0.0, 0.0], drawing.plate),
    };
    let k = o.unit.scale();
    let suffix = if o.unit == Unit::Inch { "in" } else { "mm" };
    let (w, h) = (((hi[0] - lo[0]) * k).max(0.001), ((hi[1] - lo[1]) * k).max(0.001));
    let mut wr = Writer { out: String::new(), o, top: hi[1], left: lo[0], k, used: HashSet::new(), group_numbers: vec![] };
    let _ = writeln!(wr.out, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    let _ = writeln!(
        wr.out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:inkscape=\"http://www.inkscape.org/namespaces/inkscape\" width=\"{}{suffix}\" height=\"{}{suffix}\" viewBox=\"0 0 {} {}\">",
        num(w),
        num(h),
        num(w),
        num(h)
    );
    let _ = writeln!(wr.out, "  <g fill=\"none\" stroke=\"#000\" stroke-width=\"{}\" stroke-linejoin=\"round\" stroke-linecap=\"round\">", num(o.stroke * k));
    if o.frame {
        let (a, b) = (wr.pt([0.0, drawing.plate[1]]), [num(drawing.plate[0] * k), num(drawing.plate[1] * k)]);
        let (x, y) = a.split_once(',').unwrap_or(("0", "0"));
        let id = unique_id("werkstueck", &mut wr.used);
        let _ = writeln!(wr.out, "    <rect id=\"{id}\" x=\"{x}\" y=\"{y}\" width=\"{}\" height=\"{}\" stroke=\"#999\"/>", b[0], b[1]);
    }
    if o.layers {
        for layer in &drawing.layers {
            let pieces: Vec<&Piece> = drawing.pieces.iter().filter(|p| &p.layer == layer).collect();
            if pieces.is_empty() {
                continue;
            }
            let id = unique_id(layer, &mut wr.used);
            let _ = writeln!(wr.out, "    <g id=\"{id}\" inkscape:groupmode=\"layer\" inkscape:label=\"{}\">", escape(layer));
            wr.pieces(&pieces, 0, 3);
            let _ = writeln!(wr.out, "    </g>");
        }
    } else {
        let pieces: Vec<&Piece> = drawing.pieces.iter().collect();
        wr.pieces(&pieces, 0, 2);
    }
    let _ = writeln!(wr.out, "  </g>");
    let _ = writeln!(wr.out, "</svg>");
    wr.out
}

#[cfg(test)]
mod tests {
    use super::*;
    use shape_edit::{Circle, Rect, Shape};

    fn piece(shape: Shape, layer: &str, groups: Vec<u64>) -> Piece {
        Piece { shape, layer: layer.into(), groups }
    }

    fn drawing() -> Drawing {
        Drawing {
            pieces: vec![
                piece(Shape::Rect(Rect::new([50.0, 40.0], 60.0, 30.0)), "Ebene 1", vec![900, 7]),
                piece(Shape::Circle(Circle::new([120.0, 80.0], 10.0)), "Ebene 1", vec![]),
                piece(Shape::Circle(Circle::new([20.0, 20.0], 5.0)), "Ebene 1", vec![900, 7]),
                piece(Shape::Circle(Circle::new([30.0, 20.0], 5.0)), "Ebene 1", vec![900]),
                piece(Shape::Circle(Circle::new([10.0, 90.0], 2.0)), "Löcher & Co", vec![]),
            ],
            layers: vec!["Ebene 1".into(), "Löcher & Co".into()],
            plate: [200.0, 100.0],
        }
    }

    #[test]
    fn layers_and_nested_groups_become_g_elements() {
        let svg = write(&drawing(), &Options::default());
        assert!(svg.contains("inkscape:label=\"Ebene 1\"") && svg.contains("inkscape:label=\"Löcher &amp; Co\""));
        // the outer group holds the inner group (rectangle and first small circle) and the second small circle
        let outer = svg.find("<g id=\"gruppe-1\">").expect("outer group");
        let inner = svg.find("<g id=\"gruppe-2\">").expect("inner group");
        assert!(outer < inner);
        let inner_body = &svg[inner..svg[inner..].find("</g>").unwrap() + inner];
        assert_eq!(inner_body.matches("<path").count() + inner_body.matches("<circle").count(), 2);
        assert_eq!(svg.matches("<g id=\"gruppe-").count(), 2, "every group once, though its members are apart in the drawing");
        // the loose circle stays outside the groups
        assert!(svg.contains("<circle cx=\"120\" cy=\"20\" r=\"10\"/>"));
    }

    #[test]
    fn without_groups_and_layers_the_shapes_are_flat() {
        let svg = write(&drawing(), &Options { groups: false, layers: false, ..Options::default() });
        assert!(!svg.contains("gruppe-") && !svg.contains("inkscape:label"));
        assert_eq!(svg.matches("<circle").count(), 4);
    }

    #[test]
    fn inches_scale_the_picture_and_circles_stay_circles() {
        let svg = write(&drawing(), &Options { unit: Unit::Inch, ..Options::default() });
        assert!(svg.contains("viewBox=\"0 0 7.874 3.937\"") && svg.contains("width=\"7.874in\""), "{svg}");
        assert!(svg.contains("<circle cx=\"4.7244\" cy=\"0.7874\" r=\"0.3937\"/>"), "{svg}");
    }

    #[test]
    fn rounded_corners_are_svg_arcs() {
        let mut square = shape_edit::Path::new(vec![[0.0, 0.0], [40.0, 0.0], [40.0, 30.0], [0.0, 30.0]], true);
        square.set_corner(1, Some((shape_edit::CornerKind::Round, 5.0, None)));
        let d = Drawing { pieces: vec![piece(Shape::Path(square), "Ebene 1", vec![])], layers: vec!["Ebene 1".into()], plate: [100.0, 100.0] };
        let svg = write(&d, &Options::default());
        // a quarter circle of radius 5, anticlockwise in the drawing: clockwise in the picture
        assert!(svg.contains(" A5 5 0 0 1 "), "{svg}");
    }

    #[test]
    fn the_picture_is_in_millimetres_with_y_down() {
        let svg = write(&drawing(), &Options::default());
        assert!(svg.contains("width=\"200mm\" height=\"100mm\" viewBox=\"0 0 200 100\""));
        let tight = write(&drawing(), &Options { area: Area::Shapes, stroke: 0.0, ..Options::default() });
        // shapes reach from x 8 to 130 and y 15 to 92
        assert!(tight.contains("width=\"122mm\" height=\"77mm\""), "{tight}");
    }
}
