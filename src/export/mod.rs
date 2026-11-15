//! Exporting the drawing to files for other programs. Every format has its own file in this folder
//! with its settings and a `write` function; `FORMATS` lists them, so a new format is one new file
//! and one line there. Plain logic without a window, so it can be tested.

pub mod dxf;
pub mod svg;

use shape_edit::Shape;

/// One shape to export, with where it sits in the drawing.
pub struct Piece {
    pub shape: Shape,
    /// The name of its layer.
    pub layer: String,
    /// The groups it is in, outermost first.
    pub groups: Vec<u64>,
}

/// What an export writes: the shapes (in millimetres, y up) in drawing order, layer by layer.
pub struct Drawing {
    pub pieces: Vec<Piece>,
    /// The layers' names, in their order.
    pub layers: Vec<String>,
    /// Width and height of the plate (mm).
    pub plate: [f64; 2],
}

impl Drawing {
    /// The box round all shapes, if there are any.
    pub fn bounds(&self) -> Option<([f64; 2], [f64; 2])> {
        self.pieces.iter().filter_map(|p| p.shape.bounds()).reduce(|(a, b), (c, d)| ([a[0].min(c[0]), a[1].min(c[1])], [b[0].max(d[0]), b[1].max(d[1])]))
    }
}

/// What a shape becomes in a file: a true circle, or one line through points per outline. A
/// segment of a line may be a true arc: `bulges[i]` belongs to the segment from point `i` to the
/// next (0 straight, else tan(sweep/4), positive anticlockwise, as DXF has it); empty when every
/// segment is straight.
pub enum Outline {
    Circle { center: [f64; 2], radius: f64 },
    Line { pts: Vec<[f64; 2]>, closed: bool, bulges: Vec<f64> },
}

/// A path's outline with its rounded corners and ears as true arcs.
fn path_outline(path: &shape_edit::Path) -> Option<Outline> {
    use shape_edit::vector::Piece;
    let pieces = path.pieces();
    let first = match pieces.first()? {
        Piece::Line { from, .. } => *from,
        Piece::Arc(a) => a.from,
    };
    let (mut pts, mut bulges) = (vec![first], vec![]);
    for piece in &pieces {
        let (to, bulge) = match piece {
            Piece::Line { to, .. } => (*to, 0.0),
            Piece::Arc(a) => (a.to, a.bulge()),
        };
        pts.push(to);
        bulges.push(bulge);
    }
    if path.closed && pts.len() > 2 && shape_edit::geometry::dist(pts[0], *pts.last().unwrap()) < 1e-9 {
        pts.pop();
    }
    Some(Outline::Line { pts, closed: path.closed, bulges })
}

/// The outlines of a shape; a circle always stays a circle, rounded corners stay arcs.
pub fn outlines(shape: &Shape) -> Vec<Outline> {
    match shape {
        Shape::Circle(c) => return vec![Outline::Circle { center: c.center, radius: c.radius }],
        Shape::Path(p) if !p.corners.is_empty() => return path_outline(p).into_iter().collect(),
        Shape::Compound(c) if c.paths.iter().any(|p| !p.corners.is_empty()) => return c.paths.iter().filter_map(path_outline).collect(),
        _ => {}
    }
    shape.contours().into_iter().filter(|c| c.pts.len() > 1).map(|c| Outline::Line { pts: c.pts, closed: c.closed, bulges: vec![] }).collect()
}

/// The unit the file's numbers are in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Mm,
    Inch,
}

impl Unit {
    pub const ALL: [Unit; 2] = [Unit::Mm, Unit::Inch];

    pub fn name(self) -> &'static str {
        match self {
            Unit::Mm => "Millimeter",
            Unit::Inch => "Zoll",
        }
    }

    /// Millimetres to this unit.
    pub fn scale(self) -> f64 {
        match self {
            Unit::Mm => 1.0,
            Unit::Inch => 1.0 / 25.4,
        }
    }
}

/// The settings of every format, kept while the program runs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Settings {
    pub dxf: dxf::Options,
    pub svg: svg::Options,
}

/// A file format the drawing can be exported to.
pub struct Format {
    /// Short name, shown in the list.
    pub name: &'static str,
    /// What the list and the file window call it.
    pub description: &'static str,
    pub extension: &'static str,
    /// The file's text for a drawing with these settings.
    pub write: fn(&Drawing, &Settings) -> String,
    /// The format's name with the details chosen ("DXF R14"), for the status line.
    pub title: fn(&Settings) -> String,
}

pub const FORMATS: &[Format] = &[
    Format { name: "DXF", description: "DXF-Zeichnung", extension: "dxf", write: |d, s| dxf::write(d, &s.dxf), title: |s| format!("DXF {}", s.dxf.version.name()) },
    Format { name: "SVG", description: "SVG-Vektorgrafik", extension: "svg", write: |d, s| svg::write(d, &s.svg), title: |_| "SVG".into() },
];
