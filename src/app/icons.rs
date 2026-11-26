//! All icons of the window in one consistent style, after the icon set the person picked:
//! duotone blue. Areas are filled with a soft sky-blue gradient (light on top, deeper below), the
//! outline is a mid blue of one width with rounded corners, secondary parts are white (or dashed
//! when they are only a guide), no other colours. The button around an icon is bare; it gets a
//! light frame under the pointer and a blue tint with a fine edge when chosen.

use super::{DrawTool, Icon};
use crate::theme::{icon_back, icon_bottom, icon_line, icon_top, ACCENT_DEEP};
use egui::epaint::{Mesh, Vertex, WHITE_UV};
use egui::{Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Vec2};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// Size of an icon button, and the width of every outline.
const BUTTON: f32 = 38.0;
const LINE: f32 = 1.8;

/// The colours an icon is drawn with.
#[derive(Clone, Copy)]
pub(crate) struct Colours {
    pub line: Color32,
    pub top: Color32,
    pub bottom: Color32,
    /// The second, plain fill (the white of a part in front).
    pub back: Color32,
}

impl Colours {
    pub fn normal() -> Colours {
        Colours { line: icon_line(), top: icon_top(), bottom: icon_bottom(), back: icon_back() }
    }
}

/// Triangulate a simple polygon (ear clipping), so concave outlines can be filled too.
fn triangulate(pts: &[Pos2]) -> Vec<[usize; 3]> {
    let n = pts.len();
    if n < 3 {
        return vec![];
    }
    let area: f32 = (0..n).map(|i| {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        a.x * b.y - b.x * a.y
    }).sum();
    let mut idx: Vec<usize> = if area >= 0.0 { (0..n).collect() } else { (0..n).rev().collect() };
    let cross = |a: Pos2, b: Pos2, c: Pos2| (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    let inside = |p: Pos2, a: Pos2, b: Pos2, c: Pos2| cross(a, b, p) >= 0.0 && cross(b, c, p) >= 0.0 && cross(c, a, p) >= 0.0;
    let mut out = vec![];
    let mut guard = 0;
    while idx.len() > 3 && guard < 10_000 {
        guard += 1;
        let m = idx.len();
        let mut clipped = false;
        for i in 0..m {
            let (ia, ib, ic) = (idx[(i + m - 1) % m], idx[i], idx[(i + 1) % m]);
            let (a, b, c) = (pts[ia], pts[ib], pts[ic]);
            if cross(a, b, c) <= 1e-6 {
                continue;
            }
            if idx.iter().any(|&k| k != ia && k != ib && k != ic && inside(pts[k], a, b, c)) {
                continue;
            }
            out.push([ia, ib, ic]);
            idx.remove(i);
            clipped = true;
            break;
        }
        if !clipped {
            // a degenerate outline: fan the rest
            for k in 1..idx.len() - 1 {
                out.push([idx[0], idx[k], idx[k + 1]]);
            }
            return out;
        }
    }
    if idx.len() == 3 {
        out.push([idx[0], idx[1], idx[2]]);
    }
    out
}

/// Fill a polygon with the vertical gradient (light at the top, deeper at the bottom).
pub(super) fn fill_gradient(p: &Painter, pts: &[Pos2], top: Color32, bottom: Color32) {
    let (lo, hi) = pts.iter().fold((f32::MAX, f32::MIN), |(l, h), q| (l.min(q.y), h.max(q.y)));
    let span = (hi - lo).max(0.001);
    let mut mesh = Mesh::default();
    for q in pts {
        let t = (q.y - lo) / span;
        let c = Color32::from_rgb(
            (top.r() as f32 + (bottom.r() as f32 - top.r() as f32) * t) as u8,
            (top.g() as f32 + (bottom.g() as f32 - top.g() as f32) * t) as u8,
            (top.b() as f32 + (bottom.b() as f32 - top.b() as f32) * t) as u8,
        );
        mesh.vertices.push(Vertex { pos: *q, uv: WHITE_UV, color: c });
    }
    for [a, b, c] in triangulate(pts) {
        mesh.indices.extend([a as u32, b as u32, c as u32]);
    }
    p.add(Shape::mesh(mesh));
}


/// The roles of the colours on an operation icon, all taken from the blue of the other icons so
/// that the slabs belong to the set: the top face is the usual gradient, the sides are deeper
/// blues, a cut is a dark blue.
struct Wood {
    top: (Color32, Color32),
    pit: (Color32, Color32),
    left: (Color32, Color32),
    right: (Color32, Color32),
    edge: Color32,
    groove: Color32,
    through: Color32,
    lip: Color32,
}

fn shade(c: Color32, f: f32) -> Color32 {
    Color32::from_rgb((c.r() as f32 * f).min(255.0) as u8, (c.g() as f32 * f).min(255.0) as u8, (c.b() as f32 * f).min(255.0) as u8)
}

/// A slab seen at an angle: `p(u, v)` puts a point of its top face (both from -1 to 1) on the screen.
struct Slab {
    c: Pos2,
    hw: f32,
    hh: f32,
}

impl Slab {
    fn p(&self, u: f32, v: f32) -> Pos2 {
        self.c + Vec2::new((u - v) * self.hw / 2.0, (u + v) * self.hh / 2.0)
    }
}

/// A rounded rectangle as a polygon.
fn rounded_rect(r: Rect, radius: f32) -> Vec<Pos2> {
    let radius = radius.min(r.width() / 2.0).min(r.height() / 2.0);
    if radius < 0.5 {
        return vec![r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom()];
    }
    let mut v = vec![];
    for (corner, start) in [(Pos2::new(r.right() - radius, r.top() + radius), -FRAC_PI_2), (Pos2::new(r.right() - radius, r.bottom() - radius), 0.0), (Pos2::new(r.left() + radius, r.bottom() - radius), FRAC_PI_2), (Pos2::new(r.left() + radius, r.top() + radius), PI)] {
        for k in 0..=5 {
            v.push(corner + Vec2::angled(start + FRAC_PI_2 * k as f32 / 5.0) * radius);
        }
    }
    v
}

/// An icon button with a [`super::Tip`] that shows the button's own icon on its left.
pub(super) fn icon_button_tip(ui: &mut egui::Ui, icon: Icon, selected: bool, tip: super::Tip) -> Response {
    use super::WithTip;
    icon_button_bare(ui, icon, selected).tip(&tip.icon(icon))
}

/// An icon button: returns the click response, with `tip` on hover.
pub(super) fn icon_button(ui: &mut egui::Ui, icon: Icon, selected: bool, tip: &str) -> Response {
    icon_button_bare(ui, icon, selected).on_hover_text(tip)
}

fn icon_button_bare(ui: &mut egui::Ui, icon: Icon, selected: bool) -> Response {
    icon_button_sized(ui, icon, selected, BUTTON)
}

pub(super) fn icon_button_sized(ui: &mut egui::Ui, icon: Icon, selected: bool, size: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let enabled = ui.is_enabled();
    let hovered = resp.hovered() && enabled;
    let bg = if selected {
        crate::theme::selected_fill()
    } else if hovered {
        crate::theme::hover()
    } else {
        crate::theme::panel()
    };
    if selected {
        ui.painter().rect(rect, 6.0, bg, Stroke::new(1.0_f32, ACCENT_DEEP));
    } else if hovered {
        ui.painter().rect_filled(rect, 6.0, bg);
    }
    let scale = size / BUTTON;
    Pen { p: ui.painter(), c: rect.center(), k: Colours::normal(), scale }.draw(icon);
    resp
}

/// Draws on a 24-point grid centred on `c`.
pub(super) struct Pen<'a> {
    pub p: &'a Painter,
    pub c: Pos2,
    pub k: Colours,
    /// 1.0 on a button; smaller when the icon sits in a list row.
    pub scale: f32,
}

impl Pen<'_> {
    fn at(&self, x: f32, y: f32) -> Pos2 {
        self.c + Vec2::new(x, y) * self.scale
    }

    /// A line width that shrinks with the icon, but never below a hairline.
    fn w(&self, width: f32) -> f32 {
        width * self.scale.max(0.7)
    }

    fn stroke(&self) -> Stroke {
        Stroke::new(LINE, self.k.line)
    }

    fn line(&self, pts: &[(f32, f32)]) {
        self.p.add(Shape::line(pts.iter().map(|&(x, y)| self.at(x, y)).collect(), self.stroke()));
    }

    /// A filled, outlined area: the gradient inside, the blue line round it.
    fn solid(&self, pts: Vec<Pos2>) {
        fill_gradient(self.p, &pts, self.k.top, self.k.bottom);
        self.p.add(Shape::closed_line(pts, self.stroke()));
    }

    /// A plain white area with an outline (a part in front, or a hollow one).
    fn hollow(&self, pts: Vec<Pos2>) {
        fill_gradient(self.p, &pts, self.k.back, self.k.back);
        self.p.add(Shape::closed_line(pts, self.stroke()));
    }

    fn outline(&self, pts: Vec<Pos2>) {
        self.p.add(Shape::closed_line(pts, self.stroke()));
    }

    fn dashed(&self, pts: &[Pos2]) {
        self.p.extend(Shape::dashed_line(pts, self.stroke(), 3.0, 2.4));
    }

    fn dashed_rect(&self, r: Rect) {
        self.dashed(&[r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()]);
    }

    fn rrect(&self, x: f32, y: f32, w: f32, h: f32, r: f32) -> Vec<Pos2> {
        rounded_rect(Rect::from_center_size(self.at(x, y), Vec2::new(w, h)), r)
    }

    fn node(&self, x: f32, y: f32) {
        let pts = rounded_rect(Rect::from_center_size(self.at(x, y), Vec2::splat(5.0)), 1.0);
        fill_gradient(self.p, &pts, self.k.back, self.k.back);
        self.p.add(Shape::closed_line(pts, Stroke::new(1.4_f32, self.k.line)));
    }

    /// A small filled disc.
    fn dot(&self, x: f32, y: f32) {
        let pts = self.disc(x, y, 2.3);
        self.solid(pts);
    }

    fn disc(&self, x: f32, y: f32, r: f32) -> Vec<Pos2> {
        (0..32).map(|i| self.at(x, y) + Vec2::angled(i as f32 / 32.0 * TAU) * r).collect()
    }

    fn polygon(&self, n: usize, r_out: f32, r_in: Option<f32>, cx: f32, cy: f32) -> Vec<Pos2> {
        let k = if r_in.is_some() { n * 2 } else { n };
        (0..k)
            .map(|i| {
                let a = -FRAC_PI_2 + i as f32 / k as f32 * TAU;
                let r = if i % 2 == 1 { r_in.unwrap_or(r_out) } else { r_out };
                self.at(cx + a.cos() * r, cy + a.sin() * r)
            })
            .collect()
    }

    fn ellipse(&self, cx: f32, cy: f32, rx: f32, ry: f32) -> Vec<Pos2> {
        (0..40).map(|i| i as f32 / 40.0 * TAU).map(|a| self.at(cx + a.cos() * rx, cy + a.sin() * ry)).collect()
    }

    fn arrow_head(&self, tip: Pos2, dir: Vec2) {
        let side = Vec2::new(-dir.y, dir.x) * 3.2;
        self.p.add(Shape::convex_polygon(vec![tip, tip - dir * 4.5 + side, tip - dir * 4.5 - side], self.k.line, Stroke::NONE));
    }

    /// An open, thin arrow head (two short strokes) — elegant rather than a filled wedge.
    fn thin_arrow(&self, tip: Pos2, dir: Vec2) {
        let n = Vec2::new(-dir.y, dir.x);
        let thin = Stroke::new(1.3_f32, self.k.line);
        self.p.line_segment([tip, tip - dir * 3.6 + n * 2.6], thin);
        self.p.line_segment([tip, tip - dir * 3.6 - n * 2.6], thin);
    }

    /// Draw the slab (top, left and right side) and return it, to draw the work on its top.
    fn slab(&self) -> Slab {
        let w = self.wood();
        let (hw, hh, t) = (15.0 * self.scale, 7.5 * self.scale, 4.5 * self.scale);
        let c = self.at(0.0, -2.5);
        let left = vec![c + Vec2::new(-hw, 0.0), c + Vec2::new(0.0, hh), c + Vec2::new(0.0, hh + t), c + Vec2::new(-hw, t)];
        let right = vec![c + Vec2::new(0.0, hh), c + Vec2::new(hw, 0.0), c + Vec2::new(hw, t), c + Vec2::new(0.0, hh + t)];
        let top = vec![c + Vec2::new(-hw, 0.0), c + Vec2::new(0.0, -hh), c + Vec2::new(hw, 0.0), c + Vec2::new(0.0, hh)];
        fill_gradient(self.p, &left, w.left.0, w.left.1);
        fill_gradient(self.p, &right, w.right.0, w.right.1);
        fill_gradient(self.p, &top, w.top.0, w.top.1);
        for face in [left, right, top] {
            self.p.add(Shape::closed_line(face, Stroke::new(self.w(1.0), w.edge)));
        }
        Slab { c, hw, hh }
    }

    fn wood(&self) -> Wood {
        let k = self.k;
        Wood {
            top: (k.top, k.bottom),
            pit: (shade(k.bottom, 0.78), shade(k.top, 0.92)),
            left: (shade(k.bottom, 1.0), shade(k.bottom, 0.8)),
            right: (shade(k.bottom, 0.76), shade(k.bottom, 0.58)),
            edge: shade(k.line, 0.85),
            groove: shade(k.line, 0.55),
            through: shade(k.line, 0.28),
            lip: Color32::from_rgb(0xea, 0xfa, 0xff),
        }
    }

    /// A corner of two edges with its arc: the edges end where the arc starts, the corner they
    /// left behind shows faintly, the arc stands out in red. Rounded off (the arc cuts the corner)
    /// or an ear (a circle round the corner point, open towards the edges).
    fn corner_icon(&self, ear: bool) {
        let v = (-9.0_f32, 0.0_f32);
        let (u1, u2) = ((0.9138_f32, -0.4061_f32), (0.9138_f32, 0.4061_f32));
        let along = |u: (f32, f32), t: f32| (v.0 + u.0 * t, v.1 + u.1 * t);
        let (reach, arc): (f32, Vec<(f32, f32)>) = if ear {
            // a circle about the corner: from one edge round the outside to the other
            let r = 6.0;
            let a0 = u2.1.atan2(u2.0);
            let pts = (0..=20).map(|i| a0 + i as f32 / 20.0 * (std::f32::consts::TAU - 2.0 * a0)).map(|a| (v.0 + r * a.cos(), v.1 + r * a.sin())).collect();
            (r, pts)
        } else {
            // a quarter-ish arc touching both edges, its centre inside the corner
            let (r, t) = (4.0, 9.0);
            let c = (v.0 + r / 0.4061, v.1);
            let start = along(u2, t);
            let a0 = (start.1 - c.1).atan2(start.0 - c.0);
            let sweep = std::f32::consts::TAU - 2.0 * a0;
            let pts = (0..=16).map(|i| a0 + i as f32 / 16.0 * sweep).map(|a| (c.0 + r * a.cos(), c.1 + r * a.sin())).collect();
            (t, pts)
        };
        let faint = Stroke::new(1.2_f32, self.k.line.gamma_multiply(0.35));
        let edge = Stroke::new(2.6_f32, self.k.line);
        let red = Stroke::new(2.6_f32, Color32::from_rgb(0xe0, 0x4b, 0x3a));
        for u in [u1, u2] {
            let (a, b, end) = (along(u, 0.0), along(u, reach), along(u, 20.0));
            self.p.line_segment([self.at(a.0, a.1), self.at(b.0, b.1)], faint);
            self.p.line_segment([self.at(b.0, b.1), self.at(end.0, end.1)], edge);
        }
        self.p.add(Shape::line(arc.iter().map(|(x, y)| self.at(*x, *y)).collect(), red));
    }

    pub(super) fn draw(&self, icon: Icon) {
        match icon {
            Icon::Draw(DrawTool::Select) => self.solid(vec![self.at(-5.0, -10.0), self.at(-5.0, 7.0), self.at(-0.8, 3.2), self.at(2.6, 10.0), self.at(5.2, 8.8), self.at(1.9, 2.0), self.at(7.0, 1.4)]),
            Icon::Draw(DrawTool::NodeEdit) => {
                self.line(&[(-9.0, 8.0), (-2.0, -7.0), (9.0, 1.0)]);
                for (x, y) in [(-9.0, 8.0), (-2.0, -7.0), (9.0, 1.0)] {
                    self.node(x, y);
                }
            }
            Icon::Draw(DrawTool::Polyline) => {
                let pts = [(-10.0, 9.0), (-4.0, -8.0), (3.0, 6.0), (10.0, -8.0)];
                self.line(&pts);
                for (x, y) in pts {
                    self.dot(x, y);
                }
            }
            Icon::Draw(DrawTool::Spline) => {
                let curve: Vec<(f32, f32)> = (0..=16)
                    .map(|i| {
                        let t = i as f32 / 16.0;
                        let u = 1.0 - t;
                        let (u2, t2) = (u * u, t * t);
                        let (u3, t3) = (u2 * u, t2 * t);
                        let x = u3 * (-10.0) + 3.0 * u2 * t * (-6.0) + 3.0 * u * t2 * 6.0 + t3 * 10.0;
                        let y = u3 * (-8.0) + 3.0 * u2 * t * 9.0 + 3.0 * u * t2 * (-9.0) + t3 * 8.0;
                        (x, y)
                    })
                    .collect();
                self.line(&curve);
                self.dot(-10.0, -8.0);
                self.dot(10.0, 8.0);
            }
            Icon::Draw(DrawTool::Rect) => self.solid(self.rrect(0.0, 0.0, 20.0, 15.0, 2.5)),
            Icon::Draw(DrawTool::Circle) => self.solid(self.disc(0.0, 0.0, 9.5)),
            Icon::Draw(DrawTool::Ellipse) => self.solid(self.ellipse(0.0, 0.0, 11.0, 7.0)),
            Icon::Draw(DrawTool::Polygon) => self.solid(self.polygon(6, 10.5, None, 0.0, 0.5)),
            Icon::Draw(DrawTool::Trim) => {
                let red = Stroke::new(2.2_f32, Color32::from_rgb(0xe0, 0x4b, 0x3a));
                let keep = Stroke::new(2.0_f32, self.k.line);
                // a horizontal line split by a vertical one: left and right pieces stay (blue),
                // the piece between the crossings is removed (red)
                self.p.line_segment([self.at(-11.0, 0.0), self.at(-2.0, 0.0)], keep);
                self.p.line_segment([self.at(-2.0, 0.0), self.at(2.0, 0.0)], red);
                self.p.line_segment([self.at(2.0, 0.0), self.at(11.0, 0.0)], keep);
                self.p.line_segment([self.at(0.0, -10.0), self.at(0.0, 10.0)], keep);
                self.dot(-2.0, 0.0);
                self.dot(2.0, 0.0);
            }
            Icon::Draw(DrawTool::Erase) => {
                let red = Stroke::new(2.2_f32, Color32::from_rgb(0xe0, 0x4b, 0x3a));
                let keep = Stroke::new(2.0_f32, self.k.line);
                // a line with a gap: the removed segment is red, the rest stays (blue)
                self.p.line_segment([self.at(-11.0, 0.0), self.at(-3.0, 0.0)], keep);
                self.p.line_segment([self.at(-3.0, 0.0), self.at(3.0, 0.0)], red);
                self.p.line_segment([self.at(3.0, 0.0), self.at(11.0, 0.0)], keep);
                self.p.line_segment([self.at(-3.0, -5.0), self.at(-3.0, 5.0)], keep);
                self.p.line_segment([self.at(3.0, -5.0), self.at(3.0, 5.0)], keep);
            }
            Icon::Draw(DrawTool::Connect) => {
                let edge = Stroke::new(2.0_f32, self.k.line);
                // two loose ends: existing segments, a new connecting segment (dashed)
                self.p.line_segment([self.at(-11.0, 7.0), self.at(-4.0, 0.0)], edge);
                self.p.line_segment([self.at(4.0, 0.0), self.at(11.0, -7.0)], edge);
                self.dashed(&[self.at(-4.0, 0.0), self.at(4.0, 0.0)]);
                self.dot(-4.0, 0.0);
                self.dot(4.0, 0.0);
            }
            Icon::Draw(DrawTool::FilletInner) | Icon::Draw(DrawTool::FilletOuter) => self.corner_icon(icon == Icon::Draw(DrawTool::FilletOuter)),
            Icon::Draw(DrawTool::Text) => {
                self.line(&[(-8.0, -8.0), (8.0, -8.0)]);
                self.line(&[(0.0, -8.0), (0.0, 9.0)]);
                self.line(&[(-3.0, 9.0), (3.0, 9.0)]);
            }
            Icon::Job => {
                // a board seen from above at an angle, with its thickness
                self.hollow(vec![self.at(-10.0, -1.0), self.at(-10.0, 5.0), self.at(3.0, 5.0), self.at(3.0, -1.0)]);
                self.hollow(vec![self.at(3.0, -1.0), self.at(3.0, 5.0), self.at(10.0, -2.0), self.at(10.0, -8.0)]);
                self.solid(vec![self.at(-10.0, -1.0), self.at(-3.0, -8.0), self.at(10.0, -8.0), self.at(3.0, -1.0)]);
            }
            Icon::MirrorH | Icon::MirrorV => {
                // a shape, the mirror line, and its reflection
                let flip = |x: f32, y: f32| if icon == Icon::MirrorH { (x, y) } else { (y, x) };
                let (a, b) = (flip(0.0, -11.0), flip(0.0, 11.0));
                self.dashed(&[self.at(a.0, a.1), self.at(b.0, b.1)]);
                let left = [flip(-3.0, -7.0), flip(-3.0, 7.0), flip(-11.0, 7.0)].map(|(x, y)| self.at(x, y));
                let right = [flip(3.0, -7.0), flip(3.0, 7.0), flip(11.0, 7.0)].map(|(x, y)| self.at(x, y));
                self.solid(left.to_vec());
                self.hollow(right.to_vec());
            }
            Icon::Center => {
                self.hollow(self.rrect(0.0, 0.0, 22.0, 18.0, 2.0));
                self.solid(self.rrect(0.0, 0.0, 8.0, 6.0, 1.0));
                for (from, to) in [((-11.0, 0.0), (-6.0, 0.0)), ((11.0, 0.0), (6.0, 0.0)), ((0.0, -9.0), (0.0, -5.0)), ((0.0, 9.0), (0.0, 5.0))] {
                    self.line(&[from, to]);
                }
            }
            Icon::SnapCenter => {
                // two slim concentric rings with a centre dot and short cross ticks — thin lines
                let thin = Stroke::new(1.3_f32, self.k.line);
                self.p.add(Shape::closed_line(self.disc(0.0, 0.0, 8.5), thin));
                self.p.add(Shape::closed_line(self.disc(0.0, 0.0, 3.0), thin));
                for (a, b) in [((-11.0, 0.0), (-8.0, 0.0)), ((8.0, 0.0), (11.0, 0.0)), ((0.0, -11.0), (0.0, -8.0)), ((0.0, 8.0), (0.0, 11.0))] {
                    self.p.line_segment([self.at(a.0, a.1), self.at(b.0, b.1)], thin);
                }
                self.dot(0.0, 0.0);
            }
            Icon::Offset => {
                self.solid(self.rrect(0.0, 0.0, 10.0, 8.0, 1.5));
                self.dashed_rect(Rect::from_center_size(self.c, Vec2::new(22.0, 20.0)));
            }
            Icon::Array => {
                // a 3×2 grid of small squares: the top-left one solid (the original), the rest hollow
                for (r, c) in (0..2).flat_map(|r| (0..3).map(move |c| (r, c))) {
                    let x = -8.0 + c as f32 * 8.0;
                    let y = -5.0 + r as f32 * 10.0;
                    let cell = Rect::from_center_size(self.at(x, y), Vec2::new(self.w(5.5), self.w(5.5)));
                    if r == 0 && c == 0 {
                        self.p.rect_filled(cell, 1.0, self.k.line);
                    } else {
                        self.p.rect_stroke(cell, 1.0, self.stroke());
                    }
                }
            }
            Icon::Curves => {
                let pts: Vec<Pos2> = (0..=24).map(|i| i as f32 / 24.0).map(|t| self.at(-10.0 + 20.0 * t, -(t * TAU).sin() * 7.0)).collect();
                self.p.add(Shape::line(pts, self.stroke()));
                self.node(-10.0, 0.0);
                self.node(10.0, 0.0);
            }
            Icon::Union | Icon::Subtract | Icon::Intersect => self.combine(icon),
            // the toolpath operations: a slab of wood seen at an angle, with what the cutter does to it
            Icon::Profile | Icon::Cutout => {
                let w = self.wood();
                let slab = self.slab();
                let inside = [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)];
                let ring: Vec<(f32, f32)> = if icon == Icon::Profile { vec![(-0.68, -0.68), (0.68, -0.68), (0.68, 0.68), (-0.68, 0.68)] } else { vec![(-0.74, -0.74), (0.74, -0.74), (0.74, 0.74), (-0.74, 0.74)] };
                let (groove, width) = if icon == Icon::Profile { (w.groove, 2.4) } else { (w.through, 3.2) };
                let mut pts: Vec<Pos2> = ring.iter().map(|&(u, v)| slab.p(u, v)).collect();
                pts.push(pts[0]);
                self.p.add(Shape::line(pts, Stroke::new(self.w(width), groove)));
                if icon == Icon::Cutout {
                    // tabs: the cut leaves two bridges
                    for (u, v) in [(0.0, -0.74), (0.0, 0.74)] {
                        let c = slab.p(u, v);
                        self.p.add(Shape::line(vec![c + Vec2::new(-1.6, -0.8) * self.scale, c + Vec2::new(1.6, 0.8) * self.scale], Stroke::new(self.w(3.0), w.top.0)));
                    }
                }
                // the part that stays, lifted a little with a light edge
                let part: Vec<Pos2> = inside.iter().map(|&(u, v)| slab.p(u, v)).collect();
                fill_gradient(self.p, &part, w.top.0, w.top.1);
                self.p.add(Shape::closed_line(part, Stroke::new(self.w(1.0), w.edge)));
            }
            Icon::Pocket => {
                let w = self.wood();
                let slab = self.slab();
                let pit: Vec<Pos2> = [(-0.62, -0.62), (0.62, -0.62), (0.62, 0.62), (-0.62, 0.62)].iter().map(|&(u, v)| slab.p(u, v)).collect();
                fill_gradient(self.p, &pit, w.pit.0, w.pit.1);
                // the walls: dark on the far sides, a light lip on the near ones
                let line = |a: (f32, f32), b: (f32, f32), wd: f32, c: Color32| self.p.line_segment([slab.p(a.0, a.1), slab.p(b.0, b.1)], Stroke::new(self.w(wd), c));
                line((-0.62, -0.62), (0.62, -0.62), 2.0, w.groove);
                line((-0.62, -0.62), (-0.62, 0.62), 2.0, w.groove);
                line((0.62, -0.62), (0.62, 0.62), 1.2, w.lip);
                line((-0.62, 0.62), (0.62, 0.62), 1.2, w.lip);
            }
            Icon::Drill => {
                let w = self.wood();
                let slab = self.slab();
                for (u, v) in [(-0.55, -0.42), (0.0, -0.42), (0.55, -0.42), (-0.55, 0.42), (0.0, 0.42), (0.55, 0.42)] {
                    let c = slab.p(u, v);
                    let hole: Vec<Pos2> = (0..20).map(|i| c + Vec2::angled(i as f32 / 20.0 * TAU) * Vec2::new(3.3, 2.0) * self.scale).collect();
                    fill_gradient(self.p, &hole, w.through, w.groove);
                    // a light rim on the near edge
                    let rim: Vec<Pos2> = (2..9).map(|i| c + Vec2::new(0.0, 0.3) * self.scale + Vec2::angled(i as f32 / 20.0 * TAU) * Vec2::new(2.8, 1.7) * self.scale).collect();
                    self.p.add(Shape::line(rim, Stroke::new(self.w(0.9), w.lip)));
                }
            }
            Icon::VCarve => {
                let w = self.wood();
                let slab = self.slab();
                // the engraved V: a dark groove with a light edge beside it
                let v = [self.at(-6.0, -6.5), self.at(0.0, 3.0), self.at(6.0, -6.5)];
                self.p.add(Shape::line(v.to_vec(), Stroke::new(self.w(3.6), w.groove)));
                self.p.add(Shape::line(v.iter().map(|q| *q + Vec2::new(0.0, 0.2) * self.scale).collect(), Stroke::new(self.w(1.4), w.through)));
                let _ = slab;
            }
            Icon::Transform => {
                self.dashed_rect(Rect::from_center_size(self.c, Vec2::splat(11.0)));
                for d in [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y] {
                    self.p.line_segment([self.c + d * 7.5, self.c + d * 11.5], self.stroke());
                    self.arrow_head(self.c + d * 12.5, d);
                }
            }
            Icon::Grid => {
                self.solid(self.rrect(0.0, 0.0, 20.0, 20.0, 2.5));
                for f in [-3.3_f32, 3.3] {
                    self.p.line_segment([self.at(f, -9.0), self.at(f, 9.0)], Stroke::new(1.3_f32, self.k.back));
                    self.p.line_segment([self.at(-9.0, f), self.at(9.0, f)], Stroke::new(1.3_f32, self.k.back));
                }
            }
            Icon::Snap => {
                // SnapToGeometry (bounding box): a slim dashed box with solid corner L-marks and a
                // snapping dot — elegant thin lines, not a heavy filled magnet
                let thin = Stroke::new(1.3_f32, self.k.line);
                let b = Rect::from_center_size(self.c, Vec2::splat(18.0));
                self.p.extend(Shape::dashed_line(&[b.left_top(), b.right_top(), b.right_bottom(), b.left_bottom(), b.left_top()], thin, 2.6, 2.2));
                let corner = 4.0;
                for (cx, cy, dx, dy) in [(-9.0, -9.0, 1.0, 1.0), (9.0, -9.0, -1.0, 1.0), (9.0, 9.0, -1.0, -1.0), (-9.0, 9.0, 1.0, -1.0)] {
                    self.p.line_segment([self.at(cx, cy), self.at(cx + dx * corner, cy)], thin);
                    self.p.line_segment([self.at(cx, cy), self.at(cx, cy + dy * corner)], thin);
                }
                self.dot(0.0, 0.0);
            }
            Icon::SnapGap => {
                // two slim outlined boxes with a measured gap (a thin double arrow) between them
                let thin = Stroke::new(1.3_f32, self.k.line);
                self.p.add(Shape::closed_line(self.rrect(-8.0, 0.0, 5.0, 14.0, 1.0), thin));
                self.p.add(Shape::closed_line(self.rrect(8.0, 0.0, 5.0, 14.0, 1.0), thin));
                self.p.line_segment([self.at(-4.5, 0.0), self.at(4.5, 0.0)], thin);
                self.thin_arrow(self.at(-4.5, 0.0), Vec2::new(-1.0, 0.0));
                self.thin_arrow(self.at(4.5, 0.0), Vec2::new(1.0, 0.0));
            }
            Icon::ZoomFit | Icon::ZoomSelection => {
                let b = Rect::from_center_size(self.at(-3.0, -3.0), Vec2::new(16.0, 14.0));
                if icon == Icon::ZoomFit {
                    self.solid(rounded_rect(b, 2.0));
                } else {
                    self.dashed_rect(b);
                }
                let lens = self.at(5.0, 5.0);
                self.p.line_segment([lens + Vec2::splat(3.4), lens + Vec2::splat(7.0)], Stroke::new(LINE + 0.8, self.k.line));
                let ring: Vec<Pos2> = (0..32).map(|i| lens + Vec2::angled(i as f32 / 32.0 * TAU) * 4.8).collect();
                self.hollow(ring);
            }
            Icon::Dimensions => {
                // a dimension line: two extension ticks, the dashed line between them, arrow heads
                self.line(&[(-10.0, -8.0), (-10.0, 8.0)]);
                self.line(&[(10.0, -8.0), (10.0, 8.0)]);
                self.dashed(&[self.at(-8.0, 0.0), self.at(8.0, 0.0)]);
                self.arrow_head(self.at(-10.0, 0.0), Vec2::new(-1.0, 0.0));
                self.arrow_head(self.at(10.0, 0.0), Vec2::new(1.0, 0.0));
            }
            Icon::Align => {
                // a level bar and an upright bar, each with a tick at its ends
                self.line(&[(-9.0, -5.0), (9.0, -5.0)]);
                self.line(&[(-9.0, -8.0), (-9.0, -2.0)]);
                self.line(&[(9.0, -8.0), (9.0, -2.0)]);
                self.line(&[(-6.0, 0.0), (-6.0, 10.0)]);
                self.line(&[(-9.0, 0.0), (-3.0, 0.0)]);
                self.line(&[(-9.0, 10.0), (-3.0, 10.0)]);
            }
            Icon::Distance => {
                // two points with a dimension line above them
                for x in [-8.0, 8.0] {
                    self.line(&[(x, 5.0), (x, -6.0)]);
                    self.solid(self.disc(x, 7.0, 2.8));
                }
                self.line(&[(-8.0, -4.0), (8.0, -4.0)]);
                self.arrow_head(self.at(-8.0, -4.0), Vec2::new(-1.0, 0.0));
                self.arrow_head(self.at(8.0, -4.0), Vec2::new(1.0, 0.0));
            }
            Icon::Origin => {
                // the axes seen from above with the zero point where they meet
                self.line(&[(-9.0, 9.0), (-9.0, -9.0)]);
                self.line(&[(-9.0, 9.0), (9.0, 9.0)]);
                self.arrow_head(self.at(-9.0, -10.0), Vec2::new(0.0, -1.0));
                self.arrow_head(self.at(10.0, 9.0), Vec2::new(1.0, 0.0));
                self.solid(self.disc(-9.0, 9.0, 3.2));
                self.dashed(&[self.at(-9.0, 9.0), self.at(5.0, -5.0)]);
            }
            Icon::Ruler => {
                // a ruler: a long bar with tick marks, like the edge of a measuring rule
                self.hollow(self.rrect(0.0, 0.0, 22.0, 9.0, 1.0));
                for (i, x) in [-8.0, -4.0, 0.0, 4.0, 8.0].into_iter().enumerate() {
                    let long = i % 2 == 0;
                    self.line(&[(x, -4.5), (x, if long { 0.5 } else { -1.5 })]);
                }
            }
            Icon::Group => {
                // two shapes held together by a dashed frame
                self.dashed_rect(Rect::from_center_size(self.c, Vec2::splat(22.0 * self.scale)));
                self.solid(self.rrect(-3.5, -3.5, 9.0, 9.0, 1.5));
                self.hollow(self.rrect(3.5, 3.5, 9.0, 9.0, 1.5));
            }
            Icon::Ungroup => {
                // the frame is gone, the two shapes lie apart
                self.solid(self.rrect(-6.0, -6.0, 9.0, 9.0, 1.5));
                self.hollow(self.rrect(6.0, 6.0, 9.0, 9.0, 1.5));
            }
            Icon::ImportDxf => {
                // a page with a folded corner, marked DXF
                self.line(&[(-9.0, 11.0), (-9.0, -11.0), (3.0, -11.0), (9.0, -5.0), (9.0, 11.0), (-9.0, 11.0)]);
                self.line(&[(3.0, -11.0), (3.0, -5.0), (9.0, -5.0)]);
                self.p.text(self.at(0.0, 3.5), egui::Align2::CENTER_CENTER, "DXF", egui::FontId::proportional(9.5 * self.scale.max(0.7)), self.k.line);
            }
            Icon::Variables => {
                // f(x): an italic f with a small (x) after it
                self.p.text(self.at(-5.0, 0.0), egui::Align2::CENTER_CENTER, "ƒ", egui::FontId::proportional(20.0), self.k.line);
                self.p.text(self.at(5.0, 1.0), egui::Align2::CENTER_CENTER, "(x)", egui::FontId::proportional(11.0), self.k.line);
            }
            Icon::InsertImage => {
                // a picture frame with a mountain and sun
                let r = Rect::from_center_size(self.at(0.0, 0.0), Vec2::new(20.0, 16.0));
                self.solid(rounded_rect(r, 2.0));
                // mountain
                self.p.add(Shape::line(vec![
                    Pos2::new(r.left() + 3.0, r.bottom() - 3.0),
                    Pos2::new(r.left() + 8.0, r.top() + 4.0),
                    Pos2::new(r.left() + 11.0, r.top() + 7.0),
                    Pos2::new(r.left() + 14.0, r.top() + 4.5),
                    Pos2::new(r.right() - 3.0, r.bottom() - 3.0),
                ], Stroke::new(LINE, self.k.back)));
                // sun
                self.p.circle_filled(Pos2::new(r.right() - 5.5, r.top() + 5.0), 2.5, self.k.back);
            }
            Icon::Calibrate => {
                // two crosshairs connected by a double-arrow with a ruler mark
                let a = self.at(-6.0, -4.0);
                let b = self.at(6.0, 4.0);
                let arm = 3.5_f32;
                // crosshair A
                self.p.line_segment([a - Vec2::new(arm, 0.0), a + Vec2::new(arm, 0.0)], Stroke::new(LINE, self.k.line));
                self.p.line_segment([a - Vec2::new(0.0, arm), a + Vec2::new(0.0, arm)], Stroke::new(LINE, self.k.line));
                self.p.circle_stroke(a, 2.5, Stroke::new(LINE, self.k.line));
                // crosshair B
                self.p.line_segment([b - Vec2::new(arm, 0.0), b + Vec2::new(arm, 0.0)], Stroke::new(LINE, self.k.line));
                self.p.line_segment([b - Vec2::new(0.0, arm), b + Vec2::new(0.0, arm)], Stroke::new(LINE, self.k.line));
                self.p.circle_stroke(b, 2.5, Stroke::new(LINE, self.k.line));
                // dashed line between
                let mid = a + (b - a) * 0.5;
                self.p.line_segment([a + Vec2::new(arm + 1.0, 0.0), mid - Vec2::new(1.0, 0.0)], Stroke::new(LINE * 0.8, self.k.line));
                self.p.line_segment([mid + Vec2::new(1.0, 0.0), b - Vec2::new(arm + 1.0, 0.0)], Stroke::new(LINE * 0.8, self.k.line));
                // dimension text
                self.p.text(self.at(0.0, -1.0), egui::Align2::CENTER_BOTTOM, "d", egui::FontId::proportional(9.0), self.k.line);
            }
            Icon::Slot => {
                // a groove milled along the slab surface (isometric, following the u-axis)
                let w = self.wood();
                let slab = self.slab();
                let half = 0.14_f32;
                let groove: Vec<Pos2> = vec![
                    slab.p(-0.9, -half),
                    slab.p(-0.9, half),
                    slab.p(0.9, half),
                    slab.p(0.9, -half),
                ];
                fill_gradient(self.p, &groove, w.pit.0, w.pit.1);
                // dark wall on the far side, light lip on the near side
                self.p.line_segment([slab.p(-0.9, -half), slab.p(0.9, -half)], Stroke::new(self.w(1.4), w.groove));
                self.p.line_segment([slab.p(-0.9, half), slab.p(0.9, half)], Stroke::new(self.w(1.0), w.lip));
            }
            Icon::SlotOnLine => {
                let line_s = Stroke::new(self.w(1.6), self.k.line);
                let groove_s = Stroke::new(self.w(0.8), self.k.line.gamma_multiply(0.4));
                self.p.line_segment([self.at(-10.0, 0.0), self.at(10.0, 0.0)], line_s);
                self.p.line_segment([self.at(-10.0, -3.5), self.at(10.0, -3.5)], groove_s);
                self.p.line_segment([self.at(-10.0, 3.5), self.at(10.0, 3.5)], groove_s);
                let cutter: Vec<Pos2> = (0..24).map(|i| self.at(0.0, 0.0) + Vec2::angled(i as f32 / 24.0 * TAU) * 3.5 * self.scale).collect();
                fill_gradient(self.p, &cutter, self.k.top, self.k.bottom);
                self.p.add(Shape::closed_line(cutter, Stroke::new(self.w(1.0), self.k.line)));
            }
            Icon::SlotLeft => {
                let line_s = Stroke::new(self.w(1.6), self.k.line);
                let groove_s = Stroke::new(self.w(0.8), self.k.line.gamma_multiply(0.4));
                self.p.line_segment([self.at(-10.0, 3.0), self.at(10.0, 3.0)], line_s);
                self.p.line_segment([self.at(-10.0, -0.5), self.at(10.0, -0.5)], groove_s);
                self.p.line_segment([self.at(-10.0, -7.5), self.at(10.0, -7.5)], groove_s);
                let cutter: Vec<Pos2> = (0..24).map(|i| self.at(0.0, -4.0) + Vec2::angled(i as f32 / 24.0 * TAU) * 3.5 * self.scale).collect();
                fill_gradient(self.p, &cutter, self.k.top, self.k.bottom);
                self.p.add(Shape::closed_line(cutter, Stroke::new(self.w(1.0), self.k.line)));
            }
            Icon::SlotRight => {
                let line_s = Stroke::new(self.w(1.6), self.k.line);
                let groove_s = Stroke::new(self.w(0.8), self.k.line.gamma_multiply(0.4));
                self.p.line_segment([self.at(-10.0, -3.0), self.at(10.0, -3.0)], line_s);
                self.p.line_segment([self.at(-10.0, 0.5), self.at(10.0, 0.5)], groove_s);
                self.p.line_segment([self.at(-10.0, 7.5), self.at(10.0, 7.5)], groove_s);
                let cutter: Vec<Pos2> = (0..24).map(|i| self.at(0.0, 4.0) + Vec2::angled(i as f32 / 24.0 * TAU) * 3.5 * self.scale).collect();
                fill_gradient(self.p, &cutter, self.k.top, self.k.bottom);
                self.p.add(Shape::closed_line(cutter, Stroke::new(self.w(1.0), self.k.line)));
            }
            Icon::SlotCapOuter => {
                let line_s = Stroke::new(self.w(1.6), self.k.line);
                let groove_s = Stroke::new(self.w(0.8), self.k.line.gamma_multiply(0.4));
                self.p.line_segment([self.at(-10.0, 0.0), self.at(4.0, 0.0)], line_s);
                self.dot(4.0, 0.0);
                self.p.line_segment([self.at(-10.0, -3.5), self.at(7.5, -3.5)], groove_s);
                self.p.line_segment([self.at(-10.0, 3.5), self.at(7.5, 3.5)], groove_s);
                let cap: Vec<Pos2> = (0..13).map(|i| {
                    let a = -FRAC_PI_2 + i as f32 / 12.0 * PI;
                    self.at(4.0 + a.cos() * 3.5, a.sin() * 3.5)
                }).collect();
                self.p.add(Shape::line(cap, groove_s));
                let cutter: Vec<Pos2> = (0..24).map(|i| self.at(4.0, 0.0) + Vec2::angled(i as f32 / 24.0 * TAU) * 3.5 * self.scale).collect();
                fill_gradient(self.p, &cutter, self.k.top, self.k.bottom);
                self.p.add(Shape::closed_line(cutter, Stroke::new(self.w(1.0), self.k.line)));
            }
            Icon::SlotCapInner => {
                let line_s = Stroke::new(self.w(1.6), self.k.line);
                let groove_s = Stroke::new(self.w(0.8), self.k.line.gamma_multiply(0.4));
                self.p.line_segment([self.at(-10.0, 0.0), self.at(4.0, 0.0)], line_s);
                self.dot(4.0, 0.0);
                self.p.line_segment([self.at(-10.0, -3.5), self.at(0.5, -3.5)], groove_s);
                self.p.line_segment([self.at(-10.0, 3.5), self.at(0.5, 3.5)], groove_s);
                let cap: Vec<Pos2> = (0..13).map(|i| {
                    let a = -FRAC_PI_2 + i as f32 / 12.0 * PI;
                    self.at(0.5 + a.cos() * 3.5, a.sin() * 3.5)
                }).collect();
                self.p.add(Shape::line(cap, groove_s));
                let cutter: Vec<Pos2> = (0..24).map(|i| self.at(0.5, 0.0) + Vec2::angled(i as f32 / 24.0 * TAU) * 3.5 * self.scale).collect();
                fill_gradient(self.p, &cutter, self.k.top, self.k.bottom);
                self.p.add(Shape::closed_line(cutter, Stroke::new(self.w(1.0), self.k.line)));
            }
            Icon::ClimbMilling => {
                // Climb: feed →, cutter on right edge, cleared step on right
                // Workpiece with step cut out on the right
                let block: Vec<Pos2> = vec![
                    self.at(-11.0, -1.0), self.at(2.0, -1.0),
                    self.at(2.0, -6.0), self.at(11.0, -6.0),
                    self.at(11.0, 8.0), self.at(-11.0, 8.0),
                ];
                fill_gradient(self.p, &block, self.k.bottom, shade(self.k.bottom, 0.7));
                self.p.add(Shape::closed_line(block, Stroke::new(self.w(0.8), self.k.line)));
                // Cutter circle at the step edge
                let cx = 2.0;
                let cy = -6.0;
                let r = 5.0;
                let rs = r * self.scale;
                let cutter: Vec<Pos2> = (0..24).map(|i| self.at(cx, cy) + Vec2::angled(i as f32 / 24.0 * TAU) * rs).collect();
                fill_gradient(self.p, &cutter, self.k.top, self.k.bottom);
                self.p.add(Shape::closed_line(cutter, Stroke::new(self.w(1.0), self.k.line)));
                // 4-flute cross
                let fl = 3.2;
                self.p.line_segment([self.at(cx - fl, cy), self.at(cx + fl, cy)], Stroke::new(self.w(0.8), self.k.line));
                self.p.line_segment([self.at(cx, cy - fl), self.at(cx, cy + fl)], Stroke::new(self.w(0.8), self.k.line));
                // Rotation arrow: clockwise (top-left → right → bottom)
                let ra = r + 2.0;
                let arc: Vec<Pos2> = (0..10).map(|i| {
                    let a = PI * 1.15 + i as f32 / 9.0 * PI * 1.2;
                    self.at(cx + a.cos() * ra, cy + a.sin() * ra)
                }).collect();
                let arc_tip = arc.last().copied();
                self.p.add(Shape::line(arc, Stroke::new(self.w(1.2), self.k.line)));
                if let Some(last) = arc_tip {
                    let a_end = PI * 1.15 + PI * 1.2;
                    self.arrow_head(last, Vec2::new(-a_end.sin(), a_end.cos()));
                }
                // Feed arrow: left → right, below block
                self.p.line_segment([self.at(-9.0, 11.0), self.at(7.0, 11.0)], Stroke::new(self.w(1.2), self.k.line));
                self.arrow_head(self.at(8.5, 11.0), Vec2::new(1.0, 0.0));
            }
            Icon::ConventionalMilling => {
                // Conventional: feed →, cutter on left edge, cleared step on left
                // Workpiece with step cut out on the left
                let block: Vec<Pos2> = vec![
                    self.at(-11.0, -6.0), self.at(-2.0, -6.0),
                    self.at(-2.0, -1.0), self.at(11.0, -1.0),
                    self.at(11.0, 8.0), self.at(-11.0, 8.0),
                ];
                fill_gradient(self.p, &block, self.k.bottom, shade(self.k.bottom, 0.7));
                self.p.add(Shape::closed_line(block, Stroke::new(self.w(0.8), self.k.line)));
                // Cutter circle at the step edge
                let cx = -2.0;
                let cy = -6.0;
                let r = 5.0;
                let rs = r * self.scale;
                let cutter: Vec<Pos2> = (0..24).map(|i| self.at(cx, cy) + Vec2::angled(i as f32 / 24.0 * TAU) * rs).collect();
                fill_gradient(self.p, &cutter, self.k.top, self.k.bottom);
                self.p.add(Shape::closed_line(cutter, Stroke::new(self.w(1.0), self.k.line)));
                // 4-flute cross
                let fl = 3.2;
                self.p.line_segment([self.at(cx - fl, cy), self.at(cx + fl, cy)], Stroke::new(self.w(0.8), self.k.line));
                self.p.line_segment([self.at(cx, cy - fl), self.at(cx, cy + fl)], Stroke::new(self.w(0.8), self.k.line));
                // Rotation arrow: clockwise, arc on right side (sweeps right→bottom→left)
                let ra = r + 2.0;
                let arc: Vec<Pos2> = (0..10).map(|i| {
                    let a = -PI * 0.15 - i as f32 / 9.0 * PI * 1.2;
                    self.at(cx + a.cos() * ra, cy + a.sin() * ra)
                }).collect();
                let arc_tip = arc.last().copied();
                self.p.add(Shape::line(arc, Stroke::new(self.w(1.2), self.k.line)));
                if let Some(last) = arc_tip {
                    let a_end = -PI * 0.15 - PI * 1.2;
                    self.arrow_head(last, Vec2::new(-a_end.sin(), a_end.cos()));
                }
                // Feed arrow: right → left, below block
                self.p.line_segment([self.at(9.0, 11.0), self.at(-7.0, 11.0)], Stroke::new(self.w(1.2), self.k.line));
                self.arrow_head(self.at(-8.5, 11.0), Vec2::new(-1.0, 0.0));
            }
            Icon::ClearOffset => {
                // Concentric shrinking rectangles (top-down view of offset clearing)
                let s = Stroke::new(self.w(1.0), self.k.line);
                let dim = Stroke::new(self.w(0.7), self.k.line.gamma_multiply(0.45));
                // outer boundary
                self.p.add(Shape::closed_line(vec![
                    self.at(-11.0, -9.0), self.at(11.0, -9.0),
                    self.at(11.0, 9.0), self.at(-11.0, 9.0),
                ], s));
                // inner rings
                for i in 1..=3 {
                    let m = i as f32 * 2.8;
                    self.p.add(Shape::closed_line(vec![
                        self.at(-11.0 + m, -9.0 + m), self.at(11.0 - m, -9.0 + m),
                        self.at(11.0 - m, 9.0 - m), self.at(-11.0 + m, 9.0 - m),
                    ], dim));
                }
            }
            Icon::ClearRaster => {
                // Parallel horizontal lines with turn-arounds (top-down raster pattern)
                let s = Stroke::new(self.w(1.0), self.k.line);
                let dim = Stroke::new(self.w(0.7), self.k.line.gamma_multiply(0.45));
                // outer boundary
                self.p.add(Shape::closed_line(vec![
                    self.at(-11.0, -9.0), self.at(11.0, -9.0),
                    self.at(11.0, 9.0), self.at(-11.0, 9.0),
                ], s));
                // horizontal raster lines
                let lines = 5;
                let top = -7.0_f32;
                let bot = 7.0_f32;
                let step = (bot - top) / (lines - 1) as f32;
                for i in 0..lines {
                    let y = top + i as f32 * step;
                    self.p.line_segment([self.at(-9.0, y), self.at(9.0, y)], dim);
                }
                // small U-turns connecting alternating ends
                for i in 0..lines - 1 {
                    let y0 = top + i as f32 * step;
                    let y1 = y0 + step;
                    let x = if i % 2 == 0 { 9.0 } else { -9.0 };
                    let arc: Vec<Pos2> = (0..6).map(|j| {
                        let a = if x > 0.0 { -FRAC_PI_2 + j as f32 / 5.0 * PI } else { FRAC_PI_2 - j as f32 / 5.0 * PI };
                        self.at(x + a.cos() * step / 2.0, (y0 + y1) / 2.0 + a.sin() * step / 2.0)
                    }).collect();
                    self.p.add(Shape::line(arc, dim));
                }
            }
            Icon::ClearSpiral => {
                // Continuous spiral from center outward (top-down view)
                let s = Stroke::new(self.w(1.0), self.k.line);
                let dim = Stroke::new(self.w(0.7), self.k.line.gamma_multiply(0.45));
                // outer boundary
                self.p.add(Shape::closed_line(vec![
                    self.at(-11.0, -9.0), self.at(11.0, -9.0),
                    self.at(11.0, 9.0), self.at(-11.0, 9.0),
                ], s));
                // rectangular spiral from inside out
                let steps = 3;
                let mut pts: Vec<Pos2> = vec![self.at(0.0, 0.0)];
                for i in 1..=steps {
                    let f = i as f32;
                    let mx = f * 3.2;
                    let my = f * 2.6;
                    // right, up, left, down — each ring goes one step further out
                    pts.push(self.at(mx, my - 2.6));
                    pts.push(self.at(mx, -my));
                    pts.push(self.at(-mx, -my));
                    pts.push(self.at(-mx, my));
                    pts.push(self.at(mx + 3.2, my));
                }
                self.p.add(Shape::line(pts, dim));
            }
        }
    }

    /// Two overlapping rounded squares; the gradient area is what the operation keeps.
    fn combine(&self, icon: Icon) {
        let a = Rect::from_center_size(self.at(-3.0, -3.0), Vec2::splat(14.0));
        let b = Rect::from_center_size(self.at(3.0, 3.0), Vec2::splat(14.0));
        let overlap = a.intersect(b);
        match icon {
            Icon::Union => {
                fill_gradient(self.p, &rounded_rect(a, 2.0), self.k.top, self.k.bottom);
                fill_gradient(self.p, &rounded_rect(b, 2.0), self.k.top, self.k.bottom);
                self.outline(rounded_rect(a, 2.0));
                self.outline(rounded_rect(b, 2.0));
            }
            Icon::Subtract => {
                self.solid(rounded_rect(a, 2.0));
                fill_gradient(self.p, &rounded_rect(overlap, 0.0), self.k.back, self.k.back);
                self.dashed(&[b.left_top(), b.right_top(), b.right_bottom(), b.left_bottom(), b.left_top()]);
            }
            _ => {
                self.hollow(rounded_rect(a, 2.0));
                self.hollow(rounded_rect(b, 2.0));
                self.solid(rounded_rect(overlap, 0.5));
            }
        }
    }
}

/// An eye (crossed out when hidden), in the icon colours; also used by the lists.
pub(crate) fn paint_eye(p: &Painter, c: Pos2, open: bool, k: Colours, hover: bool) {
    let lid: Vec<Pos2> = (0..=16)
        .map(|i| c + Vec2::angled(PI * (1.0 + i as f32 / 16.0)) * Vec2::new(8.5, 5.5))
        .chain((0..=16).map(|i| c + Vec2::angled(PI * (i as f32 / 16.0)) * Vec2::new(8.5, 5.5)))
        .collect();
    let line = if hover { k.bottom } else { k.line };
    fill_gradient(p, &lid, k.back, k.back);
    p.add(Shape::closed_line(lid, Stroke::new(1.6_f32, line)));
    let pupil: Vec<Pos2> = (0..24).map(|i| c + Vec2::angled(i as f32 / 24.0 * TAU) * 2.9).collect();
    fill_gradient(p, &pupil, k.top, k.bottom);
    p.add(Shape::closed_line(pupil, Stroke::new(1.2_f32, line)));
    if !open {
        p.line_segment([c + Vec2::new(-8.5, 6.5), c + Vec2::new(8.5, -6.5)], Stroke::new(1.8_f32, line));
    }
}

/// Two stacked sheets: the symbol of the layer list.
pub(super) fn paint_layers_symbol(p: &Painter, c: Pos2, k: Colours) {
    let sheet = |dy: f32| vec![c + Vec2::new(-8.0, dy + 3.5), c + Vec2::new(-4.5, dy - 3.5), c + Vec2::new(8.0, dy - 3.5), c + Vec2::new(4.5, dy + 3.5)];
    let (back, front) = (sheet(-3.5), sheet(2.5));
    fill_gradient(p, &back, k.top, k.bottom);
    p.add(Shape::closed_line(back, Stroke::new(LINE, k.line)));
    fill_gradient(p, &front, k.back, k.back);
    p.add(Shape::closed_line(front, Stroke::new(LINE, k.line)));
}

/// The small picture of what a toolpath does, for a row of the toolpath list: the same slab as the
/// operation buttons, smaller (`size` is its width in points).
pub(in crate::app) fn operation_icon(p: &Painter, c: Pos2, strategy: &crate::toolpath::Strategy, size: f32) {
    use crate::toolpath::Strategy;
    let icon = match strategy {
        Strategy::Profile { tabs: true, .. } => Icon::Cutout,
        Strategy::Profile { .. } => Icon::Profile,
        Strategy::Pocket { .. } => Icon::Pocket,
        Strategy::Drill => Icon::Drill,
        Strategy::VCarve { .. } => Icon::VCarve,
        Strategy::Slot { .. } => Icon::Slot,
    };
    Pen { p, c, k: Colours::normal(), scale: size / 30.0 }.draw(icon);
}
