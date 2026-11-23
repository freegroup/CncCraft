//! The vector operations, each in its own file here. Those of their own group in the tool column
//! work by clicking in the drawing: [`trim`] (cut a vector where it crosses others), [`delete`]
//! (delete the vector clicked), [`connect`] (join two points with a line), [`fillet_outer`] (an
//! "ear" round a corner) and [`fillet_inner`] (round a corner off). The boolean operations work on
//! the selection: [`join`], [`subtract`] and [`intersect`]. What they share lives here: finding
//! the vector or corner under the pointer, what they draw over the drawing, the radius form of the
//! two corner tools, and putting a result in place of the selection.
//!
//! A rounded corner or an ear is a property of the corner (`shape_edit::Corner`): the corner point
//! stays, the outline round it is a true arc, and the radius can be changed or the rounding taken
//! away again at any time.

mod connect;
mod delete;
mod fillet_inner;
mod fillet_outer;
mod intersect;
mod join;
mod subtract;
mod trim;

use super::{App, DrawTool};
use crate::theme::{ink, muted, ACCENT, SELECT};
use egui::{Color32, Pos2, Response, Stroke};
use shape_edit::geometry::{dist, dist_to_segment};
use shape_edit::vector::fillet::corner_arc;
use shape_edit::{CornerKind, Path, Pt, Shape};

/// How close (screen pixels) the pointer must come to a vector or a corner to pick it.
const PICK_PX: f64 = 7.0;

/// What the vector operations remember between clicks.
#[derive(Default)]
pub(super) struct VectorOps {
    /// The first point of a connection, once it is clicked.
    pub(super) connect_from: Option<Pt>,
    /// The radius the corner tools use, as typed (a number, a formula or variables).
    pub(super) radius_text: String,
    /// The corner whose radius the form shows: the shape, its path, its node.
    pub(super) picked: Option<(u64, usize, usize)>,
    /// Whether the change of the picked corner's radius is already one undo step.
    picked_undo: bool,
}

impl VectorOps {
    pub(super) fn new() -> Self {
        VectorOps { radius_text: "3".into(), ..Default::default() }
    }
}

/// A corner under the pointer: which shape (index into the shapes), which of its paths, which
/// node, and where the corner point is.
#[derive(Clone, Copy, Debug)]
struct CornerHit {
    index: usize,
    path: usize,
    node: usize,
    at: Pt,
}

/// Which segment of which shape is nearest to the pointer.
pub(super) struct SegHit {
    pub shape: usize,
    pub seg: usize,
    dist: f64,
}

/// The paths of a shape whose corners can be changed, as they would be once it is a curve:
/// paths keep theirs, straight-edged rectangles and polygons become one.
fn corner_paths(shape: &Shape) -> Option<Vec<Path>> {
    match shape {
        Shape::Path(p) => Some(vec![p.clone()]),
        Shape::Compound(c) => Some(c.paths.clone()),
        Shape::Rect(r) if r.corner_radius <= 0.0 => match shape.to_curves() {
            Shape::Path(p) => Some(vec![p]),
            _ => None,
        },
        Shape::Polygon(_) => match shape.to_curves() {
            Shape::Path(p) => Some(vec![p]),
            _ => None,
        },
        _ => None,
    }
}

/// The nodes of a path that are corners (not the ends of an open one).
fn corner_nodes(p: &Path) -> impl Iterator<Item = usize> + '_ {
    let n = p.points.len();
    (0..n).filter(move |&i| p.closed || (i > 0 && i + 1 < n))
}

impl App {
    fn pick_tolerance(&self) -> f64 {
        PICK_PX / self.zoom as f64
    }

    /// The topmost vector under `p` (an index into the shapes).
    fn vector_at(&self, p: Pt) -> Option<usize> {
        use shape_edit::Editable;
        let tol = self.pick_tolerance();
        (0..self.shapes.len()).rev().find(|&i| self.shapes[i].hit(p, tol))
    }

    /// The nearest segment of any visible shape to `p`.
    pub(super) fn segment_at(&self, p: Pt) -> Option<SegHit> {
        let tol = self.pick_tolerance();
        let mut best: Option<SegHit> = None;
        for (si, item) in self.shapes.iter().enumerate().rev() {
            if item.hidden || item.frozen {
                continue;
            }
            for c in item.shape.contours() {
                let n = c.pts.len();
                let segs = if c.closed { n } else { n.saturating_sub(1) };
                for seg in 0..segs {
                    let (a, b) = (c.pts[seg], c.pts[(seg + 1) % n]);
                    let d = dist_to_segment(p, a, b);
                    if d <= tol && best.as_ref().map_or(true, |b| d < b.dist) {
                        best = Some(SegHit { shape: si, seg, dist: d });
                    }
                }
            }
        }
        best
    }

    /// The corner nearest to `p` within reach: on its point, or on the arc of a changed corner.
    fn corner_at(&self, p: Pt) -> Option<CornerHit> {
        let tol = self.pick_tolerance();
        let mut best: Option<(f64, CornerHit)> = None;
        for (index, item) in self.shapes.iter().enumerate().rev() {
            if item.hidden || item.frozen {
                continue;
            }
            let Some(paths) = corner_paths(&item.shape) else { continue };
            for (pi, path) in paths.iter().enumerate() {
                for node in corner_nodes(path) {
                    let at = path.points[node];
                    let mut d = dist(at, p);
                    // a changed corner is also picked on its arc
                    if let Some(c) = path.corner(node) {
                        let n = path.points.len();
                        if let Some(arc) = corner_arc(path.points[(node + n - 1) % n], at, path.points[(node + 1) % n], c.kind, c.radius) {
                            let pts = arc.points(0.1);
                            d = d.min(pts.windows(2).map(|w| dist_to_segment(p, w[0], w[1])).fold(f64::MAX, f64::min));
                        }
                    }
                    if d <= tol && best.as_ref().map_or(true, |(bd, _)| d < *bd) {
                        best = Some((d, CornerHit { index, path: pi, node, at }));
                    }
                }
            }
        }
        best.map(|(_, h)| h)
    }

    /// Put `result` (of a boolean operation on `count` selected shapes) in place of the selection,
    /// selected, as one undo step; `done` is what the status line then says.
    fn replace_selection(&mut self, count: usize, result: Option<Shape>, done: String) {
        if count < 2 {
            self.status = t!("Dafür mindestens zwei Formen auswählen").into();
            return;
        }
        let Some(result) = result else {
            self.status = t!("Das Ergebnis wäre leer").into();
            return;
        };
        self.push_undo();
        let sel = std::mem::take(&mut self.edit.selection);
        self.shapes.retain(|s| !sel.contains(&s.id));
        let id = self.add_shape(result);
        self.edit.selection = vec![id];
        self.edit.retain_existing(&self.shapes);
        self.status = done;
    }

    /// A click (or Esc) while a vector operation is the tool.
    pub(super) fn vector_op_input(&mut self, resp: &Response, mouse: Option<Pt>) {
        if resp.ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.vops.connect_from = None;
        }
        if !resp.clicked() {
            return;
        }
        let Some(m) = mouse else { return };
        match self.tool {
            DrawTool::Trim => self.trim_at(m),
            DrawTool::Erase => self.delete_segment_at(m),
            DrawTool::Connect => self.connect_click(m),
            DrawTool::FilletOuter => self.fillet_outer_click(m),
            DrawTool::FilletInner => self.fillet_inner_click(m),
            _ => {}
        }
    }

    /// Put the corner of `hit` to `kind` with the form's radius, or pick it when it already has
    /// that kind (its radius then shows in the form), or switch it to `kind`.
    fn corner_click(&mut self, p: Pt, kind: CornerKind) {
        let Some(hit) = self.corner_at(p) else {
            self.status = t!("Hier ist keine Ecke: eine Ecke eines Linienzugs, Rechtecks oder Polygons anklicken").into();
            return;
        };
        let id = self.shapes[hit.index].id;
        let paths = corner_paths(&self.shapes[hit.index].shape).unwrap_or_default();
        let existing = paths.get(hit.path).and_then(|p| p.corner(hit.node)).cloned();
        if let Some(c) = &existing {
            if c.kind == kind {
                self.vops.picked = Some((id, hit.path, hit.node));
                self.vops.picked_undo = false;
                self.vops.radius_text = c.expr.clone().unwrap_or_else(|| super::fields::fmt(c.radius));
                self.status = t!("Ecke gewählt: Radius links ändern oder die Rundung entfernen").into();
                return;
            }
        }
        let (radius, expr) = match self.radius_value() {
            Ok(r) => r,
            Err(e) => {
                self.status = e;
                return;
            }
        };
        let (radius, expr) = match existing {
            Some(c) => (c.radius, c.expr),
            None => (radius, expr),
        };
        self.push_undo();
        self.set_corner(hit.index, hit.path, hit.node, Some((kind, radius, expr)));
        self.vops.picked = Some((id, hit.path, hit.node));
        self.vops.picked_undo = true;
        self.recompute_all();
    }

    /// The form's radius: its value, and the formula when it is one.
    fn radius_value(&self) -> Result<(f64, Option<String>), String> {
        let text = self.vops.radius_text.trim().to_string();
        let value = shape_edit::calc::eval(&text, &self.vars_resolved).map_err(|e| crate::lang::calc_message(&e))?;
        if value <= 0.0 {
            return Err(t!("Der Radius muss größer als 0 sein").into());
        }
        let plain = !shape_edit::calc::is_formula(&text) && shape_edit::calc::names_in(&text).is_empty();
        Ok((value, (!plain).then_some(text)))
    }

    /// Change corner `node` of path `path` of shape `index` (a rectangle or polygon becomes a curve
    /// for it); `None` takes the change away.
    fn set_corner(&mut self, index: usize, path: usize, node: usize, corner: Option<(CornerKind, f64, Option<String>)>) {
        let item = &mut self.shapes[index];
        let Some(mut paths) = corner_paths(&item.shape) else { return };
        let Some(p) = paths.get_mut(path) else { return };
        p.set_corner(node, corner);
        let was_curve = matches!(item.shape, Shape::Path(_) | Shape::Compound(_));
        item.shape = if paths.len() == 1 && !matches!(item.shape, Shape::Compound(_)) { Shape::Path(paths.remove(0)) } else { Shape::Compound(shape_edit::Compound::new(paths)) };
        if !was_curve {
            // as a curve it has no width or height of its own any more
            item.bound.clear();
        }
    }

    /// The picked corner, if it is still there: its shape's index and the corner.
    fn picked_corner(&self) -> Option<(usize, usize, usize, shape_edit::Corner)> {
        let (id, path, node) = self.vops.picked?;
        let index = self.shapes.iter().position(|s| s.id == id)?;
        let c = corner_paths(&self.shapes[index].shape)?.get(path)?.corner(node)?.clone();
        Some((index, path, node, c))
    }

    /// The form of the two corner tools: the radius (new corners get it; a picked corner follows
    /// it at once), and taking the picked corner's rounding away.
    pub(super) fn corner_form(&mut self, ui: &mut egui::Ui) {
        let picked = self.picked_corner();
        ui.horizontal(|ui| {
            ui.label(t!("Radius"));
            let r = ui.add(egui::TextEdit::singleline(&mut self.vops.radius_text).desired_width(110.0).font(egui::TextStyle::Monospace));
            ui.label(egui::RichText::new("mm").color(muted()));
            if r.changed() {
                if let (Some((index, path, node, c)), Ok((radius, expr))) = (&picked, self.radius_value()) {
                    if !self.vops.picked_undo {
                        self.push_undo();
                        self.vops.picked_undo = true;
                    }
                    self.set_corner(*index, *path, *node, Some((c.kind, radius, expr)));
                    self.recompute_all();
                }
            }
        });
        match self.radius_value() {
            Ok((v, Some(_))) => {
                ui.label(egui::RichText::new(format!("= {} mm", super::fields::fmt(v))).monospace().color(muted()));
            }
            Ok(_) => {}
            Err(e) => {
                ui.label(egui::RichText::new(e).color(crate::theme::error()));
            }
        }
        ui.add_space(6.0);
        if let Some((index, path, node, _)) = picked {
            ui.label(egui::RichText::new(t!("Gewählte Ecke: der Radius folgt dem Feld.")).color(ink()));
            if ui.button(t!("Rundung entfernen")).clicked() {
                self.push_undo();
                self.set_corner(index, path, node, None);
                self.vops.picked = None;
                self.recompute_all();
            }
        } else {
            ui.label(egui::RichText::new(t!("Eine Ecke anklicken: sie bekommt diesen Radius. Eine schon gerundete Ecke anklicken: ihr Radius lässt sich hier ändern oder die Rundung entfernen.")).color(muted()));
        }
        ui.label(egui::RichText::new(t!("Der Eckpunkt bleibt erhalten; der Radius darf eine Formel oder Variable sein.")).small().color(muted()));
    }

    /// What the vector operations draw over the drawing: the vector or corner under the pointer,
    /// the connection being made, and the changed corners with their radii.
    pub(super) fn draw_vector_ops(&self, painter: &egui::Painter, mouse: Option<Pt>) {
        if !self.tool.is_vector_op() {
            return;
        }
        let screen = |p: Pt| self.to_screen(p);
        let outline = |index: usize, colour: Color32| {
            for c in self.shapes[index].shape.contours() {
                let mut pts: Vec<Pos2> = c.pts.iter().map(|p| screen(*p)).collect();
                if c.closed && !pts.is_empty() {
                    pts.push(pts[0]);
                }
                painter.add(egui::Shape::line(pts, Stroke::new(3.0_f32, colour)));
            }
        };
        match self.tool {
            DrawTool::Erase => {
                if let Some(hit) = mouse.and_then(|m| self.segment_at(m)) {
                    let contours = self.shapes[hit.shape].shape.contours();
                    if let Some(c) = contours.first() {
                        let n = c.pts.len();
                        let segs = if c.closed { n } else { n.saturating_sub(1) };
                        if hit.seg < segs {
                            let (a, b) = (c.pts[hit.seg], c.pts[(hit.seg + 1) % n]);
                            painter.add(egui::Shape::line(vec![screen(a), screen(b)], Stroke::new(4.0_f32, crate::theme::error())));
                        }
                    }
                }
            }
            DrawTool::Trim => {
                if let Some((i, m)) = mouse.and_then(|m| self.vector_at(m).map(|i| (i, m))) {
                    outline(i, ACCENT.gamma_multiply(0.8));
                    for p in self.trim_points(i, m) {
                        painter.circle(screen(p), 4.0, ACCENT, Stroke::new(1.0_f32, ink()));
                    }
                }
            }
            DrawTool::Connect => {
                if let Some(from) = self.vops.connect_from {
                    let a = screen(from);
                    painter.circle_filled(a, 4.0, SELECT);
                    if let Some(m) = mouse {
                        let b = screen(m);
                        painter.add(egui::Shape::dashed_line(&[a, b], Stroke::new(1.5_f32, SELECT), 6.0, 4.0));
                    }
                }
            }
            DrawTool::FilletOuter | DrawTool::FilletInner => {
                // corner under the pointer highlighted (the labels are drawn by draw_corners)
                if let Some(hit) = mouse.and_then(|m| self.corner_at(m)) {
                    painter.circle_stroke(screen(hit.at), 7.0, Stroke::new(2.0_f32, ACCENT));
                }
            }
            _ => {}
        }
    }

    /// Draw the fillet radius labels as constraint decorations in the drawing (always visible,
    /// not only when fillet tools are active). Each label has a thin leader line from the corner
    /// point to the label, and is clickable/draggable like other constraints.
    pub(super) fn draw_corners(&mut self, painter: &egui::Painter) {
        use crate::theme::{CONSTRAINT, SELECT};
        let selected = self.selected_constraint;
        let zoom = self.zoom as f64;
        let mut labels = Vec::new();
        for item in self.shapes.iter().filter(|s| !s.hidden) {
            let Some(paths) = corner_paths(&item.shape) else { continue };
            for (pi, path) in paths.iter().enumerate() {
                for c in &path.corners {
                    let at = path.points[c.node];
                    let at_screen = self.to_screen(at);
                    let chosen = matches!(selected, Some(super::distance::LabelTarget::Corner(id, p, n)) if id == item.id && p == pi && n == c.node);
                    let tone = if chosen { SELECT } else { CONSTRAINT };
                    let default_offset = [8.0 / zoom, 8.0 / zoom];
                    let off = c.label_offset.unwrap_or(default_offset);
                    let label_world = [at[0] + off[0], at[1] + off[1]];
                    let label_screen = self.to_screen(label_world);
                    let text = if let Some(expr) = &c.expr {
                        format!("R ƒ{expr}")
                    } else {
                        format!("R{}", super::fields::fmt(c.radius))
                    };
                    painter.line_segment([at_screen, label_screen], egui::Stroke::new(1.0_f32, tone.gamma_multiply(0.5)));
                    painter.circle_stroke(at_screen, 3.0, egui::Stroke::new(1.2_f32, tone));
                    let r = super::fields::value_label(painter, label_screen, text, tone);
                    labels.push((item.id, pi, c.node, r));
                }
            }
        }
        self.corner_labels = labels;
    }

    /// Open the inline edit field for a fillet radius.
    pub(super) fn open_corner_edit(&mut self, id: u64, pi: usize, ni: usize) {
        let Some(index) = self.shapes.iter().position(|s| s.id == id) else { return };
        let Some(paths) = corner_paths(&self.shapes[index].shape) else { return };
        let Some(c) = paths.get(pi).and_then(|p| p.corner(ni)) else { return };
        let text = c.expr.clone().unwrap_or_else(|| super::fields::fmt(c.radius));
        let rect = self.corner_labels.iter().find(|(i, p, n, _)| *i == id && *p == pi && *n == ni).map(|(_, _, _, r)| *r).unwrap_or(egui::Rect::NOTHING);
        self.corner_edit = Some(super::CornerEdit { id, path: pi, node: ni, text, rect, error: None, input: super::input::ConstraintInput::selecting(0) });
        self.status = t!("Radius ändern · Feld leeren und Enter drücken entfernt die Rundung · Esc bricht ab").into();
    }

    /// The inline edit field for a fillet radius: the shared number field (same as the shape
    /// sizes, the distance values and the construction lines). It takes a number, a variable or
    /// a formula; an empty field removes the rounding.
    pub(super) fn corner_edit_ui(&mut self, ctx: &egui::Context, _painter: &egui::Painter) {
        let id = egui::Id::new("corner_edit");
        let Some(mut ce) = self.corner_edit.take() else {
            // the field is gone: so is its keyboard focus (tool keys must not type into it)
            ctx.memory_mut(|m| {
                if m.has_focus(id) {
                    m.surrender_focus(id);
                }
            });
            return;
        };
        let tab = std::mem::take(&mut self.corner_tab);
        let names = self.var_names();
        let value = |n: &str| self.vars_resolved.value(n).ok();
        let lookup = self.vars_resolved.clone();
        let (outcome, rect) = super::input::edit_number_field(ctx, id, ce.rect.center(), &mut ce.text, &mut ce.input, &names, &value, &lookup, super::input::Accepts::PositiveOrEmpty, tab, ce.error.is_some());
        ce.rect = rect;
        match outcome {
            super::input::NumberOutcome::Open => self.corner_edit = Some(ce),
            super::input::NumberOutcome::Cancel => self.status = t!("Abgebrochen").into(),
            super::input::NumberOutcome::Commit => self.take_corner_radius(ce),
        }
    }

    /// Put the typed radius on its corner: nothing removes the rounding, a plain number is just
    /// the radius, a variable or formula is kept so the corner follows it.
    fn take_corner_radius(&mut self, mut ce: super::CornerEdit) {
        let text = ce.text.trim().trim_end_matches('=').trim().to_string();
        if text.is_empty() {
            self.push_undo();
            self.set_corner_by_id(ce.id, ce.path, ce.node, None);
            self.recompute_all();
            self.status = t!("Rundung entfernt").into();
            return;
        }
        let v = match shape_edit::calc::eval(&text, &self.vars_resolved) {
            Ok(v) => v,
            Err(e) => {
                ce.error = Some(crate::lang::calc_message(&e));
                self.corner_edit = Some(ce);
                return;
            }
        };
        let Some(index) = self.shapes.iter().position(|s| s.id == ce.id) else { return };
        let plain = !shape_edit::calc::is_formula(&text) && shape_edit::calc::names_in(&text).is_empty();
        let expr = (!plain).then_some(text.clone());
        let kind = corner_paths(&self.shapes[index].shape).and_then(|ps| ps.get(ce.path)?.corner(ce.node).map(|c| c.kind)).unwrap_or(shape_edit::CornerKind::Round);
        self.push_undo();
        self.set_corner(index, ce.path, ce.node, Some((kind, v, expr)));
        self.vops.radius_text = text;
        self.recompute_all();
        self.status = t!("Radius geändert").into();
    }

    /// Move a corner's label offset by a world-space delta.
    pub(super) fn nudge_corner_label(&mut self, id: u64, pi: usize, ni: usize, delta: [f64; 2]) {
        let Some(index) = self.shapes.iter().position(|s| s.id == id) else { return };
        let Some(mut paths) = corner_paths(&self.shapes[index].shape) else { return };
        let Some(path) = paths.get_mut(pi) else { return };
        let Some(c) = path.corners.iter_mut().find(|c| c.node == ni) else { return };
        let default = [8.0 / self.zoom as f64, 8.0 / self.zoom as f64];
        let off = c.label_offset.unwrap_or(default);
        c.label_offset = Some([off[0] + delta[0], off[1] + delta[1]]);
        let item = &mut self.shapes[index];
        item.shape = if paths.len() == 1 && !matches!(item.shape, Shape::Compound(_)) {
            Shape::Path(paths.remove(0))
        } else {
            Shape::Compound(shape_edit::Compound::new(paths))
        };
    }

    /// Set or remove a corner by shape id (used by the constraint system).
    pub(super) fn set_corner_by_id(&mut self, id: u64, pi: usize, ni: usize, corner: Option<(shape_edit::CornerKind, f64, Option<String>)>) {
        let Some(index) = self.shapes.iter().position(|s| s.id == id) else { return };
        self.set_corner(index, pi, ni, corner);
    }
}
