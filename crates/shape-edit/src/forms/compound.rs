use super::{bounds_lines, DimLine, Flip, Form, Path, Shape};
use crate::geometry::Contour;
use crate::Pt;
use serde::{Deserialize, Serialize};

/// Several paths edited as one shape: letters turned into curves, or the result of combining
/// shapes (an outline with holes, or several separate pieces).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Compound {
    pub paths: Vec<Path>,
}

impl Compound {
    pub fn new(paths: Vec<Path>) -> Self {
        Compound { paths }
    }

    /// Path index and node index within it, for a node index counted across all paths.
    pub fn locate(&self, mut node: usize) -> Option<(usize, usize)> {
        for (i, p) in self.paths.iter().enumerate() {
            if node < p.points.len() {
                return Some((i, node));
            }
            node -= p.points.len();
        }
        None
    }

    fn each(&self, f: impl Fn(&Path) -> Shape) -> Shape {
        Compound { paths: self.paths.iter().map(|p| match f(p) { Shape::Path(p) => p, _ => unreachable!("a path stays a path") }).collect() }.into()
    }
}

impl Form for Compound {
    fn label(&self) -> &'static str {
        "Kurven"
    }

    fn contours(&self) -> Vec<Contour> {
        self.paths.iter().flat_map(Form::contours).collect()
    }

    fn decoration(&self) -> Vec<DimLine> {
        bounds_lines(&self.contours())
    }

    fn translated(&self, dx: f64, dy: f64) -> Self {
        Compound { paths: self.paths.iter().map(|p| p.translated(dx, dy)).collect() }
    }

    fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Shape {
        self.each(|p| p.scaled(anchor, sx, sy))
    }

    fn mirrored(&self, axis: Pt, flip: Flip) -> Shape {
        self.each(|p| p.mirrored(axis, flip))
    }
}
