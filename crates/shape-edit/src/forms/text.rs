use super::{curves_mirrored, curves_scaled, is_uniform, place, rotate, scale_point, Dim, DimLine, Flip, Form, Shape};
use crate::geometry::{bounds_of, dist, Contour};
use crate::Pt;
use serde::{Deserialize, Serialize};

/// A line (or lines) of text drawn as letter outlines, centred on `center`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Text {
    pub center: Pt,
    pub text: String,
    /// Height of a capital letter.
    pub height: f64,
    /// Degrees, anticlockwise.
    pub rotation: f64,
}

impl Text {
    pub fn new(center: Pt, text: impl Into<String>, height: f64) -> Self {
        Text { center, text: text.into(), height, rotation: 0.0 }
    }
}

impl Form for Text {
    fn label(&self) -> &'static str {
        "Text"
    }

    fn contours(&self) -> Vec<Contour> {
        outlines(&self.text, self.height)
            .into_iter()
            .map(|c| Contour { pts: place(c.pts, self.center, self.rotation), closed: c.closed })
            .collect()
    }

    /// The capital height, measured up the left side of the writing.
    fn decoration(&self) -> Vec<DimLine> {
        let left = bounds_of(&outlines(&self.text, self.height)).map_or(-self.height / 2.0, |(lo, _)| lo[0]);
        let at = |y: f64| rotate([self.center[0] + left, self.center[1] + y], self.center, self.rotation);
        let outward = rotate([-1.0, 0.0], [0.0, 0.0], self.rotation);
        vec![DimLine { dim: Dim::TextHeight, from: at(-self.height / 2.0), to: at(self.height / 2.0), outward }]
    }

    fn translated(&self, dx: f64, dy: f64) -> Self {
        Text { center: [self.center[0] + dx, self.center[1] + dy], ..self.clone() }
    }

    fn scaled(&self, anchor: Pt, sx: f64, sy: f64) -> Shape {
        if !is_uniform(sx, sy) {
            return curves_scaled(self.contours(), anchor, sx, sy);
        }
        Text { center: scale_point(self.center, anchor, sx, sy), height: self.height * sx, ..self.clone() }.into()
    }

    fn mirrored(&self, axis: Pt, flip: Flip) -> Shape {
        // mirrored writing isn't text any more
        curves_mirrored(self.contours(), axis, flip)
    }
}

/// Collects glyph outlines from the font, flattening curves into short segments.
struct Outliner {
    contours: Vec<Contour>,
    cur: Vec<Pt>,
    scale: f64,
    dx: f64,
}

impl Outliner {
    fn p(&self, x: f32, y: f32) -> Pt {
        [self.dx + x as f64 * self.scale, y as f64 * self.scale]
    }
}

impl ttf_parser::OutlineBuilder for Outliner {
    fn move_to(&mut self, x: f32, y: f32) {
        if self.cur.len() > 2 {
            let c = std::mem::take(&mut self.cur);
            self.contours.push(Contour::closed(c));
        }
        self.cur.clear();
        let p = self.p(x, y);
        self.cur.push(p);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.cur.push(p);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let p0 = *self.cur.last().unwrap_or(&[0.0, 0.0]);
        let (p1, p2) = (self.p(x1, y1), self.p(x, y));
        for i in 1..=6 {
            let t = i as f64 / 6.0;
            let u = 1.0 - t;
            self.cur.push([u * u * p0[0] + 2.0 * u * t * p1[0] + t * t * p2[0], u * u * p0[1] + 2.0 * u * t * p1[1] + t * t * p2[1]]);
        }
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let p0 = *self.cur.last().unwrap_or(&[0.0, 0.0]);
        let (p1, p2, p3) = (self.p(x1, y1), self.p(x2, y2), self.p(x, y));
        for i in 1..=8 {
            let t = i as f64 / 8.0;
            let u = 1.0 - t;
            let f = |a: f64, b: f64, c: f64, d: f64| u * u * u * a + 3.0 * u * u * t * b + 3.0 * u * t * t * c + t * t * t * d;
            self.cur.push([f(p0[0], p1[0], p2[0], p3[0]), f(p0[1], p1[1], p2[1], p3[1])]);
        }
    }
    fn close(&mut self) {
        let mut c = std::mem::take(&mut self.cur);
        if c.len() > 1 && dist(c[0], *c.last().unwrap()) < 1e-6 {
            c.pop();
        }
        if c.len() > 2 {
            self.contours.push(Contour::closed(c));
        }
    }
}

/// Outlines of `text` in a sans font, capital height = `height`, centred on (0, 0).
fn outlines(text: &str, height: f64) -> Vec<Contour> {
    let Ok(face) = ttf_parser::Face::parse(epaint_default_fonts::UBUNTU_LIGHT, 0) else {
        return vec![];
    };
    let cap = face.capital_height().unwrap_or((face.units_per_em() as f32 * 0.7) as i16) as f64;
    let scale = height / cap.max(1.0);
    let line_h = face.height() as f64 * scale;
    let mut o = Outliner { contours: vec![], cur: vec![], scale, dx: 0.0 };
    let mut lines: Vec<(f64, Vec<Contour>)> = Vec::new();
    for line in text.split('\n') {
        o.dx = 0.0;
        for ch in line.chars() {
            match face.glyph_index(ch) {
                Some(g) => {
                    face.outline_glyph(g, &mut o);
                    o.dx += face.glyph_hor_advance(g).unwrap_or(0) as f64 * scale;
                }
                None => o.dx += height * 0.5,
            }
        }
        lines.push((o.dx, std::mem::take(&mut o.contours)));
    }
    let total_h = height + (lines.len() as f64 - 1.0) * line_h;
    lines
        .into_iter()
        .enumerate()
        .flat_map(|(i, (width, cs))| {
            let y0 = total_h / 2.0 - height - i as f64 * line_h;
            cs.into_iter().map(move |c| Contour { pts: c.pts.iter().map(|p| [p[0] - width / 2.0, p[1] + y0]).collect(), closed: c.closed })
        })
        .collect()
}
