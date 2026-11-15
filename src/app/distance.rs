//! Distance constraints in the app: the dimension mode ("D", or the button in the toolbar), the
//! drawing of the constraints with their values, and typing or changing a value. The sums are in
//! `shape_edit::constraints`.
//!
//! Using it: press D (or the button), click a point or a line (corners and centre of a shape, the
//! nodes and segments of a vector), then a second point or line. With two points the dimension
//! follows the pointer — beside the points it measures across that way (horizontal or vertical),
//! between or diagonal it measures straight — and a click places it; then the value is typed (a
//! number, a formula or variables). The shape of the second pick is the one that moves. Shown
//! whenever "Maße anzeigen" is on or the mode is active; click a value to change it, right-click
//! it to delete the constraint.

use super::fields::{field, fmt, value_label};
use super::input::{ConstraintInput, Keys};
use super::{App, DrawTool};
use crate::theme::{CONSTRAINT, MUTED, SELECT};
use egui::{Color32, Id, Pos2, Rect, Stroke, Vec2};
use shape_edit::constraints::{self, axis_for_placement, Anchor, Axis, Distance, Measured};
use shape_edit::{calc, vars, Constrains, ConstraintView, Item, Pt};

/// The dimension mode: the picks so far, and the tool to go back to when another is chosen.
pub(super) struct DimMode {
    first: Option<Anchor>,
    second: Option<Anchor>,
    tool: DrawTool,
    hover: Option<Anchor>,
}

/// A value being typed for a new or existing constraint.
pub(super) struct DistanceEdit {
    /// The constraint changed (`None`: a new one).
    index: Option<usize>,
    draft: Distance,
    text: String,
    /// Where the field was drawn, so a click beside it can be told apart.
    rect: Rect,
    error: Option<String>,
    /// The behaviour every number field shares: suggestions, the cursor (see `input.rs`).
    input: ConstraintInput,
}

impl DistanceEdit {
    fn new(index: Option<usize>, draft: Distance, text: String) -> Self {
        DistanceEdit { index, draft, text, rect: Rect::NOTHING, error: None, input: ConstraintInput::selecting(0) }
    }
}

impl DimMode {
    pub(super) fn new(tool: DrawTool) -> Self {
        DimMode { first: None, second: None, tool, hover: None }
    }
}

const PICK_PX: f64 = 10.0;
/// How long a press on a dimension value must stay still before its value can be changed, and how
/// far it may wander (screen px) before it counts as a drag (estimated; Fusion-like).
const HOLD_SECS: f64 = 0.45;
const MOVE_PX: f32 = 8.0;

/// What a press on a drawn value is on.
#[derive(Clone, Copy)]
pub(super) enum LabelTarget {
    Distance(usize),
    ShapeDim(u64, shape_edit::Dim),
    /// The symbol of a level/upright constraint.
    Align(usize),
    /// A fillet radius label: shape id, path index, node index.
    Corner(u64, usize, usize),
}

/// A press on a drawn dimension value that is going on.
pub(super) struct LabelPress {
    target: LabelTarget,
    start: Pos2,
    last: Pos2,
    t0: f64,
    moved: bool,
    opened: bool,
    rect: Rect,
}

/// How close the pointer must be to a dimension line (px) to take hold of it.
const LINE_GRAB_PX: f32 = 6.0;

fn distance_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_sq();
    let t = if len2 > 0.0 { ((p - a).dot(ab) / len2).clamp(0.0, 1.0) } else { 0.0 };
    p.distance(a + ab * t)
}

impl App {
    /// The shapes and, last, the material plate: what constraints look at.
    fn with_plate(&self) -> Vec<Item> {
        let mut items = self.shapes.clone();
        items.push(constraints::plate_item(self.job.width, self.job.height));
        items
    }
}

impl App {
    /// D: into the dimension mode, or out of it.
    pub(super) fn toggle_dim_mode(&mut self) {
        if self.dim_mode.is_some() {
            self.dim_mode = None;
            self.distance_edit = None;
            self.status = "Bemaßen beendet".into();
            return;
        }
        self.dims = None;
        self.align_mode = None;
        self.edit.rotate_mode = false;
        self.poly.clear();
        self.drawing = None;
        self.mode = super::ViewMode::TwoD;
        // the mode works on its own: the tool column goes back to selecting
        self.tool = DrawTool::Select;
        self.tool_form = None;
        self.dim_mode = Some(DimMode { first: None, second: None, tool: self.tool, hover: None });
        self.status = "Bemaßen: einen Punkt oder eine Linie wählen · D oder Esc beendet".into();
    }

    /// Esc: take back the picks, then leave the mode. Returns whether it did anything.
    pub(super) fn cancel_dim_mode(&mut self) -> bool {
        if self.distance_edit.take().is_some() {
            return true;
        }
        match &mut self.dim_mode {
            Some(m) if m.first.is_some() || m.second.is_some() => {
                m.first = None;
                m.second = None;
                self.status = "Bemaßen: einen Punkt oder eine Linie wählen".into();
                true
            }
            Some(_) => {
                self.dim_mode = None;
                self.status = "Bemaßen beendet".into();
                true
            }
            None => false,
        }
    }

    /// Keep every distance: forget those whose shapes are gone, then move shapes until they hold.
    /// Toolpaths are made again once no drag is going on.
    pub(super) fn apply_constraints(&mut self) {
        // the sides a shape's size constraints hold, for the editor (their handles do nothing)
        let list = &self.distances;
        for it in &mut self.shapes {
            it.locked = shape_edit::constraints::locked_sides(it, list);
        }
        if self.distances.is_empty() && self.aligns.is_empty() {
            return;
        }
        // level/upright constraints whose shapes are gone are forgotten
        {
            let shapes = &self.shapes;
            let there = |id: u64| id == constraints::PLATE || shapes.iter().any(|s| s.id == id);
            self.aligns.retain(|al| there(al.a.shape()) && there(al.b.shape()));
        }
        let before = self.distances.len();
        let shapes = &self.shapes;
        let there = |id: u64| id == constraints::PLATE || shapes.iter().any(|s| s.id == id);
        self.distances.retain(|c| there(c.from.shape()) && there(c.to.shape()));
        if self.distances.len() != before {
            self.status = "Abstand entfernt: eine der Formen gibt es nicht mehr".into();
        }
        let vars = &self.vars_resolved;
        let value = |e: &str| calc::eval(e, vars).ok().filter(|v| *v > 0.0);
        // the plate takes part for a moment, as the last item
        self.shapes.push(constraints::plate_item(self.job.width, self.job.height));
        let changed = constraints::solve_all_with(&self.distances, &self.aligns, &mut self.shapes, &value);
        self.shapes.pop();
        if changed {
            self.distances_pending = true;
        }
        // distances that can't hold together are drawn red; say so once when it starts
        let items = self.with_plate();
        let off = self.distances.iter().filter(|c| calc::eval(&c.expr, &self.vars_resolved).ok().is_some_and(|w| constraints::shortfall(c, &items, w).is_some_and(|s| s > 1e-6 * w.abs().max(1.0)))).count();
        if off > 0 && off != self.distance_conflicts {
            self.status = format!("{off} Abstand/Abstände widersprechen sich und lassen sich nicht alle einhalten (rot markiert): einen davon ändern oder löschen");
        }
        self.distance_conflicts = off;
        // a dimension line is kept relative to what it measures from the moment it is in the list,
        // so it goes along when the shapes move
        if self.distances.iter().any(|c| c.offset.is_none()) {
            let items = self.with_plate();
            for c in &mut self.distances {
                if c.offset.is_none() {
                    if let Some(m) = c.measure(&items) {
                        let p = c.place;
                        c.set_place_world(&m, p);
                    }
                }
            }
        }
        if self.distances_pending && !self.edit.is_busy() {
            self.distances_pending = false;
            self.recompute_all();
        }
    }

    /// The dimension mode's clicks on the drawing.
    pub(super) fn dimension_input(&mut self, resp: &egui::Response, mouse: Option<Pt>) {
        let Some(mode) = &self.dim_mode else { return };
        // another tool was chosen: the mode is over
        if self.tool != mode.tool {
            self.dim_mode = None;
            self.distance_edit = None;
            return;
        }
        if self.distance_edit.is_some() {
            return;
        }
        let Some(m) = mouse else { return };
        let items = self.with_plate();
        let hover = constraints::pick(&items, m, PICK_PX / self.zoom as f64);
        let clicked = resp.clicked();
        let (first, second) = {
            let mode = self.dim_mode.as_mut().expect("checked above");
            mode.hover = hover;
            (mode.first, mode.second)
        };
        if !clicked {
            return;
        }
        match (first, second) {
            (None, _) => {
                if let Some(h) = hover {
                    self.dim_mode.as_mut().expect("checked above").first = Some(h);
                    self.status = if matches!(h, Anchor::Edge(..)) {
                        "Linie gewählt: in die Zeichenfläche klicken bemaßt ihre Länge · oder einen Punkt wählen".into()
                    } else {
                        "Zweiten Punkt oder eine Linie wählen".into()
                    };
                }
            }
            (Some(f), None) => {
                let Some(mut h) = hover else {
                    // a line alone: its length, placed where the click is
                    if let Some(draft) = constraints::length_of(&items, f, m) {
                        self.begin_distance_edit(&items, draft);
                    }
                    return;
                };
                let mut f = f;
                // the plate never moves: it is always the reference
                if h.shape() == constraints::PLATE && f.shape() != constraints::PLATE {
                    std::mem::swap(&mut f, &mut h);
                    self.dim_mode.as_mut().expect("checked above").first = Some(f);
                }
                if matches!((f, h), (Anchor::Edge(..), Anchor::Edge(..))) {
                    self.status = "Zwei Linien lassen sich nicht bemaßen: einen Punkt wählen".into();
                    return;
                }
                if f == h {
                    return;
                }
                let ok = Distance { from: f, to: h, axis: Axis::Aligned, expr: String::new(), place: m, offset: None }.check(&items);
                match ok {
                    Ok(()) => {
                        self.dim_mode.as_mut().expect("checked above").second = Some(h);
                        self.status = "Maß platzieren: Zeiger bewegen (neben den Punkten waagrecht oder senkrecht), klicken".into();
                    }
                    Err(why) => self.status = why,
                }
            }
            (Some(f), Some(t)) => {
                let draft = self.draft_at(&items, f, t, m);
                self.begin_distance_edit(&items, draft);
            }
        }
    }

    /// A placed dimension: its value is typed next (starting from the current one). A pair that
    /// can't be dimensioned is refused with the reason, and the picks are dropped.
    fn begin_distance_edit(&mut self, items: &[Item], draft: Distance) {
        let ok = draft.check(items);
        let mode = self.dim_mode.as_mut().expect("in the dimension mode");
        mode.first = None;
        mode.second = None;
        match ok {
            Ok(()) => {
                let value = draft.measure(items).map_or(0.0, |mm| mm.d.abs());
                self.distance_edit = Some(DistanceEdit::new(None, draft, fmt(value)));
                self.status = "Abstand eingeben (Zahl, Formel oder Variable) · Enter übernimmt · Esc bricht ab".into();
            }
            Err(why) => self.status = why,
        }
    }

    /// The distance between two picks as it would be placed at `at`.
    fn draft_at(&self, items: &[Item], from: Anchor, to: Anchor, at: Pt) -> Distance {
        let mut draft = Distance { from, to, axis: Axis::Aligned, expr: String::new(), place: at, offset: None };
        if matches!((from, to), (Anchor::Point(..), Anchor::Point(..))) {
            if let Some(m) = draft.measure(items) {
                draft.axis = axis_for_placement(m.a, m.b, at);
            }
        }
        draft
    }

    /// The typed value of a distance, as the person leaves the field: a number or a formula that
    /// works out above zero. Returns the reason when it can't be used yet.
    fn take_distance(&mut self) -> Result<(), String> {
        let Some(e) = &self.distance_edit else { return Ok(()) };
        let text = e.text.trim().trim_end_matches('=').trim().to_string();
        let v = calc::eval(&text, &self.vars_resolved).map_err(|er| crate::lang::calc_message(&er))?;
        if v <= 0.0 {
            return Err(t!("muss größer als 0 sein").into());
        }
        let e = self.distance_edit.take().expect("checked above");
        let mut c = e.draft;
        c.expr = text;
        match e.index {
            Some(i) if i < self.distances.len() => self.distances[i] = c,
            _ => {
                // the same two ends measured the same way are one constraint (two values would
                // fight each other): the new value replaces the old
                let same = |o: &Distance| o.axis == c.axis && ((o.from == c.from && o.to == c.to) || (o.from == c.to && o.to == c.from));
                match self.distances.iter().position(same) {
                    Some(i) => self.distances[i] = Distance { from: self.distances[i].from, to: self.distances[i].to, ..c },
                    None => self.distances.push(c),
                }
            }
        }
        self.apply_constraints();
        self.status = "Abstand gesetzt (Wert anklicken zum Ändern, Rechtsklick löscht)".into();
        Ok(())
    }

    /// A click in the drawing beside the value field takes the value, like Enter (the field stays
    /// open when it can't be used yet). Returns whether the press was used up.
    pub(super) fn accept_distance_on_press(&mut self, ctx: &egui::Context, canvas: Rect) -> bool {
        let Some(e) = &self.distance_edit else { return false };
        let at = ctx.input(|i| if i.pointer.primary_pressed() { i.pointer.press_origin() } else { None });
        let Some(p) = at else { return false };
        if !canvas.contains(p) || e.rect.expand(6.0).contains(p) {
            return false;
        }
        if let Err(why) = self.take_distance() {
            if let Some(e) = &mut self.distance_edit {
                e.error = Some(why.clone());
                e.input.select = Some(0);
            }
            self.status = format!("Eingabe nicht übernommen: {why}");
        }
        true
    }

    /// The drawn dimension values, as in Fusion: press on one and **drag** to move the dimension
    /// (its line follows the pointer), or press and **hold still** for a moment to change the
    /// value. A quick click does nothing. A right-click on a constraint's value deletes it.
    /// Called every frame; returns whether a new press on a value was taken (and so does nothing
    /// else in the drawing).
    pub(super) fn label_press_step(&mut self, ctx: &egui::Context) -> bool {
        let (hover, down, pressed_at, right, now) = ctx.input(|i| (i.pointer.hover_pos(), i.pointer.primary_down(), if i.pointer.primary_pressed() { i.pointer.press_origin() } else { None }, i.pointer.button_pressed(egui::PointerButton::Secondary), i.time));
        // a press going on: it moves the dimension, or after a still moment opens its value
        if let Some(mut lp) = self.label_press.take() {
            if !down {
                // let go after holding still (a frame may have been missed): open it now
                if !lp.moved && !lp.opened && now - lp.t0 >= HOLD_SECS {
                    self.open_label_target(lp.target);
                } else if !lp.moved && !lp.opened {
                    // a quick click chooses the constraint (Entf deletes it); a size that is no
                    // constraint has nothing to choose
                    let is_constraint = match lp.target {
                        LabelTarget::Distance(i) => i < self.distances.len(),
                        LabelTarget::Align(i) => i < self.aligns.len(),
                        LabelTarget::ShapeDim(id, dim) => self.shapes.iter().any(|s| s.id == id && s.bound.iter().any(|(d, _)| *d == dim)),
                        LabelTarget::Corner(..) => true,
                    };
                    if is_constraint {
                        self.selected_constraint = Some(lp.target);
                        self.status = "Constraint gewählt: Entf löscht es (bei einer Kante bleibt ihr jetziger Wert als normaler Wert)".into();
                    }
                }
                return false;
            }
            let p = hover.unwrap_or(lp.last);
            if !lp.moved && p.distance(lp.start) > MOVE_PX {
                lp.moved = true;
            }
            if lp.moved {
                if let LabelTarget::Distance(i) = lp.target {
                    let (now_mm, then_mm) = (self.to_mm(p), self.to_mm(lp.last));
                    let delta = [now_mm[0] - then_mm[0], now_mm[1] - then_mm[1]];
                    let items = self.with_plate();
                    if let Some(c) = self.distances.get_mut(i) {
                        if let Some(m) = c.measure(&items) {
                            let now = c.place_world(&m);
                            c.set_place_world(&m, [now[0] + delta[0], now[1] + delta[1]]);
                        }
                    }
                    ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
                    self.status = "Maß verschieben".into();
                } else if let LabelTarget::ShapeDim(id, dim) = lp.target {
                    // the dimension line of a size: pulled out of the shape or pushed in
                    self.nudge_dim_line(id, dim, p - lp.last);
                    ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
                } else if let LabelTarget::Corner(id, pi, ni) = lp.target {
                    let (now_mm, then_mm) = (self.to_mm(p), self.to_mm(lp.last));
                    let delta = [now_mm[0] - then_mm[0], now_mm[1] - then_mm[1]];
                    self.nudge_corner_label(id, pi, ni, delta);
                    ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
                }
            } else if !lp.opened && now - lp.t0 >= HOLD_SECS {
                lp.opened = true;
                self.open_label_target(lp.target);
            } else if !lp.opened {
                // a thin bar under the value fills while it is held: "keep holding"
                let progress = ((now - lp.t0) / HOLD_SECS).clamp(0.0, 1.0) as f32;
                let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, Id::new("hold_progress")));
                let r = lp.rect;
                painter.line_segment([r.left_bottom() + Vec2::new(0.0, 3.0), r.left_bottom() + Vec2::new(r.width() * progress, 3.0)], Stroke::new(2.5_f32, CONSTRAINT));
                ctx.request_repaint_after(std::time::Duration::from_millis(30));
            }
            lp.last = p;
            self.label_press = Some(lp);
            return false;
        }
        if self.distance_edit.is_some() || self.corner_edit.is_some() {
            return false;
        }
        // a value is grabbed on its label, a dimension drawn without its value on its line
        // (only with the select tool and no constraint mode: otherwise the press is for drawing or picking)
        let grab_lines = self.tool == DrawTool::Select && self.dim_mode.is_none() && self.align_mode.is_none();
        let over_distance = |p: Pos2| {
            self.distance_labels.iter().find(|(_, r)| r.contains(p)).map(|(i, _)| *i).or_else(|| {
                if !grab_lines {
                    return None;
                }
                self.distance_lines.iter().find(|(_, [a, b])| distance_to_segment(p, *a, *b) <= LINE_GRAB_PX).map(|(i, _)| *i)
            })
        };
        let over_dim = |p: Pos2| {
            self.dim_labels.iter().find(|(_, _, r)| r.contains(p)).map(|(id, dim, _)| (*id, *dim)).or_else(|| {
                if !grab_lines {
                    return None;
                }
                self.dim_lines.iter().find(|(_, _, [a, b])| distance_to_segment(p, *a, *b) <= LINE_GRAB_PX).map(|(id, dim, _)| (*id, *dim))
            })
        };
        let align_labels = &self.align_labels;
        let over_align = |p: Pos2| align_labels.iter().find(|(_, r)| r.contains(p)).map(|(i, _)| *i);
        let corner_labels = &self.corner_labels;
        let fillet_active = matches!(self.tool, DrawTool::FilletInner | DrawTool::FilletOuter);
        let over_corner = |p: Pos2| {
            if fillet_active { return None; }
            corner_labels.iter().find(|(_, _, _, r)| r.contains(p)).map(|(id, pi, ni, _)| (*id, *pi, *ni))
        };
        if hover.and_then(over_distance).is_some() || hover.and_then(over_align).is_some() || hover.and_then(over_corner).is_some() || (self.dims.is_none() && hover.and_then(over_dim).is_some()) {
            ctx.set_cursor_icon(egui::CursorIcon::Grab);
        }
        // right-click on a symbol deletes the constraint
        if let (true, Some(i)) = (right, hover.and_then(over_align)) {
            if i < self.aligns.len() {
                self.aligns.remove(i);
                self.selected_constraint = None;
                self.status = "Waagrecht/Senkrecht gelöscht".into();
            }
            return true;
        }
        if let (true, Some(i)) = (right, hover.and_then(over_distance)) {
            if i < self.distances.len() {
                self.distances.remove(i);
                self.status = "Abstand gelöscht".into();
            }
            return true;
        }
        if let (true, Some((id, pi, ni))) = (right, hover.and_then(over_corner)) {
            self.push_undo();
            self.set_corner_by_id(id, pi, ni, None);
            self.recompute_all();
            self.status = t!("Rundung entfernt").into();
            return true;
        }
        let Some(p) = pressed_at else { return false };
        if over_distance(p).is_none() && over_dim(p).is_none() && over_align(p).is_none() && over_corner(p).is_none() {
            self.selected_constraint = None;
        }
        let target = match (over_distance(p), over_align(p), over_corner(p), if self.dims.is_none() { over_dim(p) } else { None }) {
            (Some(i), _, _, _) => LabelTarget::Distance(i),
            (None, Some(i), _, _) => LabelTarget::Align(i),
            (None, None, Some((id, pi, ni)), _) => LabelTarget::Corner(id, pi, ni),
            (None, None, None, Some((id, dim))) => LabelTarget::ShapeDim(id, dim),
            _ => return false,
        };
        let rect = match target {
            LabelTarget::Distance(i) => self.distance_labels.iter().find(|(n, _)| *n == i).map(|(_, r)| *r),
            LabelTarget::ShapeDim(id, dim) => self.dim_labels.iter().find(|(n, d, _)| *n == id && *d == dim).map(|(_, _, r)| *r),
            LabelTarget::Align(i) => self.align_labels.iter().find(|(n, _)| *n == i).map(|(_, r)| *r),
            LabelTarget::Corner(id, pi, ni) => self.corner_labels.iter().find(|(i, p, n, _)| *i == id && *p == pi && *n == ni).map(|(_, _, _, r)| *r),
        }
        .unwrap_or(Rect::from_center_size(p, Vec2::splat(20.0)));
        self.label_press = Some(LabelPress { target, start: p, last: p, t0: now, moved: false, opened: false, rect });
        self.status = "Gedrückt halten zum Ändern des Werts · ziehen verschiebt das Maß".into();
        ctx.request_repaint_after(std::time::Duration::from_millis(60));
        true
    }

    /// A size typed in is the constraint "these two corners are this far apart": set it, or change
    /// the value of the one already there (the dimension line stays where it was put).
    pub(super) fn upsert_length_constraint(&mut self, id: u64, pair: (usize, usize, Axis), expr: String) {
        let same = |c: &Distance| {
            c.from.shape() == id && c.to.shape() == id && c.axis == pair.2 && matches!((c.from, c.to), (Anchor::Point(_, a), Anchor::Point(_, b)) if (a == pair.0 && b == pair.1) || (a == pair.1 && b == pair.0))
        };
        if let Some(c) = self.distances.iter_mut().find(|c| same(c)) {
            c.expr = expr;
            return;
        }
        let Some((lo, hi)) = self.shapes.iter().find(|s| s.id == id).and_then(|s| s.shape.bounds()) else { return };
        // the dimension line goes a little outside the shape: under it for a width, left of it for a height
        let gap = 16.0 / self.zoom as f64;
        let place = if pair.2 == Axis::Horizontal { [(lo[0] + hi[0]) / 2.0, lo[1] - gap] } else { [lo[0] - gap, (lo[1] + hi[1]) / 2.0] };
        self.distances.push(Distance { from: Anchor::Point(id, pair.0), to: Anchor::Point(id, pair.1), axis: pair.2, expr, place, offset: None });
    }

    /// Entf with a constraint chosen: it goes. A distance just leaves the list; a length set on an
    /// edge becomes a normal value (the size stays as it is, the shape can be resized again).
    /// Returns whether there was one.
    pub(super) fn delete_selected_constraint(&mut self) -> bool {
        let Some(target) = self.selected_constraint.take() else { return false };
        match target {
            LabelTarget::Distance(i) if i < self.distances.len() => {
                self.distances.remove(i);
                self.status = "Abstand gelöscht".into();
            }
            LabelTarget::ShapeDim(id, dim) => {
                if let Some(it) = self.shapes.iter_mut().find(|s| s.id == id) {
                    it.bound.retain(|(d, _)| *d != dim);
                }
                self.status = "Längen-Constraint gelöscht: der Wert bleibt, wie er ist".into();
            }
            LabelTarget::Align(i) if i < self.aligns.len() => {
                self.aligns.remove(i);
                self.status = "Waagrecht/Senkrecht gelöscht: die Linie bleibt, wie sie ist".into();
            }
            LabelTarget::Corner(id, pi, ni) => {
                self.push_undo();
                self.set_corner_by_id(id, pi, ni, None);
                self.recompute_all();
                self.status = t!("Rundung entfernt").into();
            }
            LabelTarget::Distance(_) | LabelTarget::Align(_) => return false,
        }
        true
    }

    /// Open the value field for a distance that isn't in the list yet (what a click placing it does).
    pub(super) fn open_distance_edit(&mut self, draft: Distance, text: &str) {
        self.distance_edit = Some(DistanceEdit::new(None, draft, text.to_string()));
    }

    /// Open the value a held press was on, for typing.
    fn open_label_target(&mut self, target: LabelTarget) {
        match target {
            LabelTarget::Distance(i) => {
                if let Some(c) = self.distances.get(i).cloned() {
                    self.distance_edit = Some(DistanceEdit::new(Some(i), c.clone(), c.expr));
                    self.status = "Abstand ändern · Enter übernimmt · Esc bricht ab".into();
                }
            }
            LabelTarget::ShapeDim(id, dim) => self.open_dim_label(id, dim),
            LabelTarget::Corner(id, pi, ni) => self.open_corner_edit(id, pi, ni),
            // a symbol has no value to change
            LabelTarget::Align(_) => {}
        }
    }

    /// One distance drawn: extension lines from its two ends to the dimension line, the line with
    /// end ticks, and the value in a label on it. Returns the label's place (none without a value)
    /// and the dimension line's two ends on the screen.
    fn draw_distance(&self, items: &[Item], painter: &egui::Painter, c: &Distance, text: Option<String>, tone: Color32) -> Option<(Option<Rect>, [Pos2; 2])> {
        let m: Measured = c.measure(items)?;
        let n = [-m.dir[1], m.dir[0]];
        let place = c.place_world(&m);
        let off = |p: Pt| {
            let t = (place[0] - p[0]) * n[0] + (place[1] - p[1]) * n[1];
            [p[0] + n[0] * t, p[1] + n[1] * t]
        };
        let (qa, qb) = (off(m.a), off(m.b));
        let (sa, sb) = (self.to_screen(m.a), self.to_screen(m.b));
        let (ta, tb) = (self.to_screen(qa), self.to_screen(qb));
        let ext = Stroke::new(0.8_f32, tone.gamma_multiply(0.7));
        let line = Stroke::new(1.4_f32, tone);
        for (s, t) in [(sa, ta), (sb, tb)] {
            let v = t - s;
            if v.length() > 4.0 {
                painter.line_segment([s + v.normalized() * 3.0, t + v.normalized() * 4.0], ext);
            }
        }
        painter.extend(egui::Shape::dashed_line(&[ta, tb], line, 5.0, 3.0));
        let across = {
            let v = tb - ta;
            if v.length() > 0.5 {
                let u = v.normalized();
                Vec2::new(-u.y, u.x)
            } else {
                Vec2::new(0.0, 1.0)
            }
        };
        for p in [ta, tb] {
            painter.line_segment([p - across * 5.0, p + across * 5.0], line);
        }
        let mid = ta + (tb - ta) / 2.0;
        // without a value (the faint view) there is no label: the line itself is what is grabbed
        Some((text.map(|text| value_label(painter, mid, text, tone)), [ta, tb]))
    }

    /// The constraints (each decides by itself how much of it shows, see `Constrains::view`; while
    /// the mode is active all of them show in full), the pick marks of the mode and the preview of
    /// the dimension being placed.
    pub(super) fn draw_distances(&mut self, painter: &egui::Painter, mouse: Option<Pos2>) {
        self.distance_labels.clear();
        self.distance_lines.clear();
        let mode_on = self.dim_mode.is_some();
        if self.distances.is_empty() && !mode_on {
            return;
        }
        let items = self.with_plate();
        {
            let mut labels = Vec::new();
            let mut lines = Vec::new();
            for (i, c) in self.distances.iter().enumerate() {
                let view = if mode_on { ConstraintView::Full } else { c.view(self.show_dims, &self.edit.selection) };
                if view == ConstraintView::Hidden {
                    continue;
                }
                // while its value is being typed the field stands in for the label
                if self.distance_edit.as_ref().is_some_and(|e| e.index == Some(i)) {
                    continue;
                }
                // a constraint that can't apply now (a turned shape) is not drawn
                if c.check(&items).is_err() {
                    continue;
                }
                let Some(m) = c.measure(&items) else { continue };
                let slim = view == ConstraintView::Slim;
                let v = fmt(m.d.abs());
                // a distance the others prevent is red, with the wanted and the real value
                let wanted = calc::eval(&c.expr, &self.vars_resolved).ok();
                let off = wanted.is_some_and(|w| (m.d.abs() - w).abs() > 1e-6 * w.abs().max(1.0));
                let text = match (off, wanted) {
                    (true, Some(w)) => format!("{} ≠ {v} mm", fmt(w)),
                    _ if calc::is_formula(&c.expr) => format!("ƒ {} = {v} mm", c.expr),
                    _ => format!("{v} mm"),
                };
                let chosen = matches!(self.selected_constraint, Some(LabelTarget::Distance(j)) if j == i);
                let tone = if chosen { SELECT } else if off { Color32::from_rgb(0xc0, 0x30, 0x30) } else { CONSTRAINT };
                let tone = if slim { tone.gamma_multiply(0.5) } else { tone };
                if let Some((label, line)) = self.draw_distance(&items, painter, c, (!slim).then_some(text), tone) {
                    if let Some(r) = label {
                        labels.push((i, r));
                    }
                    lines.push((i, line));
                }
            }
            self.distance_labels = labels;
            self.distance_lines = lines;
        }
        let Some(mode) = &self.dim_mode else { return };
        // every point that can be picked, small
        for it in &items {
            for k in 0..constraints::point_count(it) {
                if let Some(p) = constraints::point_of(it, k) {
                    painter.circle_stroke(self.to_screen(p), 3.0, Stroke::new(1.0_f32, CONSTRAINT.gamma_multiply(0.45)));
                }
            }
        }
        let mark = |a: Anchor, strong: bool| match a {
            Anchor::Point(id, k) => {
                if let Some(p) = items.iter().find(|s| s.id == id).and_then(|s| constraints::point_of(s, k)) {
                    let c = self.to_screen(p);
                    if strong {
                        painter.circle_filled(c, 5.0, CONSTRAINT);
                    } else {
                        painter.circle_stroke(c, 6.5, Stroke::new(2.0_f32, CONSTRAINT));
                    }
                }
            }
            Anchor::Edge(id, k) => {
                if let Some((a, b)) = items.iter().find(|s| s.id == id).and_then(|s| constraints::edge_of(s, k)) {
                    painter.line_segment([self.to_screen(a), self.to_screen(b)], Stroke::new(if strong { 3.0_f32 } else { 2.4_f32 }, CONSTRAINT));
                }
            }
        };
        for a in [mode.first, mode.second].into_iter().flatten() {
            mark(a, true);
        }
        if self.distance_edit.is_none() {
            if let Some(h) = mode.hover {
                mark(h, false);
            }
        }
        // a lone line picked: its length follows the pointer
        if let (Some(f @ Anchor::Edge(..)), None, None, Some(p)) = (mode.first, mode.second, mode.hover, mouse) {
            if let Some(draft) = constraints::length_of(&items, f, self.to_mm(p)) {
                if let Some(m) = draft.measure(&items) {
                    self.draw_distance(&items, painter, &draft, Some(format!("{} mm", fmt(m.d.abs()))), CONSTRAINT.gamma_multiply(0.8));
                }
            }
        }
        // the dimension following the pointer, once both ends are picked
        if let (Some(f), Some(t), Some(p)) = (mode.first, mode.second, mouse) {
            let draft = self.draft_at(&items, f, t, self.to_mm(p));
            if let Some(m) = draft.measure(&items) {
                self.draw_distance(&items, painter, &draft, Some(format!("{} mm", fmt(m.d.abs()))), CONSTRAINT.gamma_multiply(0.8));
            }
        }
    }

    /// The field for a distance's value, on the dimension line.
    pub(super) fn distance_edit_ui(&mut self, ctx: &egui::Context, painter: &egui::Painter) {
        let Some(mut e) = self.distance_edit.take() else {
            // the value field is gone: its keyboard focus must go too, or tool keys (S, D, …) would
            // be taken as typing for a field nobody sees
            let id = Id::new("distance_value_field");
            ctx.memory_mut(|m| {
                if m.has_focus(id) {
                    m.surrender_focus(id);
                }
            });
            return;
        };
        // the dimension itself, drawn through the draft, so it can be seen while typing
        let items = self.with_plate();
        let measured = e.draft.measure(&items);
        if let Some(m) = measured {
            let preview = Distance { expr: e.text.clone(), ..e.draft.clone() };
            if let Some((Some(r), _)) = self.draw_distance(&items, painter, &preview, Some(String::new()), CONSTRAINT) {
                // the label is replaced by the field: wipe it with the sheet's colour
                painter.rect_filled(r.expand(1.0), 4.0, crate::theme::SHEET_LABEL);
            }
            let _ = m;
        }
        let at = {
            let m = measured;
            match m {
                Some(m) => {
                    let n = [-m.dir[1], m.dir[0]];
                    let place = e.draft.place_world(&m);
                    let off = |p: Pt| {
                        let t = (place[0] - p[0]) * n[0] + (place[1] - p[1]) * n[1];
                        [p[0] + n[0] * t, p[1] + n[1] * t]
                    };
                    let (a, b) = (self.to_screen(off(m.a)), self.to_screen(off(m.b)));
                    a + (b - a) / 2.0
                }
                None => self.to_screen(e.draft.place),
            }
        };
        let id = Id::new("distance_value_field");
        let invalid = e.error.is_some() || self.field_problem(&e.text).is_some();
        let resp = field(ctx, id, at, &mut e.text, invalid);
        e.rect = resp.rect;
        // a click on one of last frame's suggestions takes that name
        if let Some((_, name)) = e.input.click(ctx) {
            e.text = vars::apply_completion(&e.text, &name);
        }
        e.input.cursor(ctx, 0, id, &e.text);
        if resp.changed() {
            e.error = None;
        }
        let (enter, esc) = ctx.input(|i| (i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Escape)));
        // suggestions (the same for every number field): Tab is taken out of the input by
        // `take_tabs` (the field would type it); Enter takes the cursor out of a one-line field in
        // the same frame, so that counts as having it
        let tab = std::mem::take(&mut self.distance_tab);
        let mut keys = Keys::read(ctx, tab);
        let focused = ctx.memory(|m| m.has_focus(id)) || (resp.lost_focus() && enter);
        let names = self.var_names();
        let value = |n: &str| self.vars_resolved.value(n).ok();
        let suggested = e.input.suggest(ctx, 0, id, resp.rect, &mut e.text, &names, &value, &mut keys, focused, 4.0);
        let (listing, taken) = (suggested.listing, suggested.taken);
        if let Some(why) = self.field_problem(&e.text) {
            // the reason, under the field
            painter.text(resp.rect.center_bottom() + Vec2::new(0.0, 6.0), egui::Align2::CENTER_TOP, why, egui::FontId::proportional(11.5), Color32::from_rgb(0xc0, 0x30, 0x30));
        }
        self.distance_edit = Some(e);
        if esc && listing {
            // the first Esc closes the list (by leaving the field), the next one cancels
            ctx.memory_mut(|m| m.surrender_focus(id));
        } else if esc {
            self.distance_edit = None;
            self.status = "Abgebrochen".into();
        } else if enter && !taken {
            if let Err(why) = self.take_distance() {
                if let Some(e) = &mut self.distance_edit {
                    e.error = Some(why.clone());
                    e.input.select = Some(0);
                }
                self.status = format!("Eingabe nicht übernommen: {why}");
            }
        }
        let _ = MUTED;
    }

    /// Why typed text can't be a distance yet (shown under the field).
    fn field_problem(&self, text: &str) -> Option<String> {
        super::input::problem(text, &self.vars_resolved, true)
    }
}
