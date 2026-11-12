//! Values typed on the canvas, as in the original: the dimension fields of a shape being drawn
//! (width, height, diameter, …) and the angle field of the rotate mode. Which dimensions a shape
//! shows, and where, is decided by the shape class itself ([`shape_edit::Form::decoration`]);
//! this module only draws them. Every field accepts a formula; finishing it with "=" puts the
//! result in the field.

use super::input::{ConstraintInput, Keys};
use super::{App, DrawTool};
use crate::theme::{CONSTRAINT, INK, MUTED, SELECT};
use egui::{Align2, Color32, FontId, Id, Pos2, Rect, Stroke, Vec2};
use shape_edit::{calc, selection_bounds, vars, Dim, DimLine, Shape};

/// How far a dimension line sits outside the shape, and its field beyond that (screen px).
const LINE_GAP: f32 = 16.0;
const FIELD_GAP: f32 = 18.0;
/// A radius field sits this far beyond the radius's end, along the radius: the polygon tool
/// moves it by 30 along the unit direction (0x10084ad78; the star tool did the same, 0x100856efc).
const RADIUS_FIELD_GAP: f32 = 30.0;

/// The dimension fields of one shape.
pub(super) struct DimEdit {
    id: u64,
    fields: Vec<(Dim, String)>,
    /// Fields the person typed in. A typed size is a length constraint of that edge; a value taken
    /// over from drawing is just a value.
    touched: Vec<bool>,
    /// The behaviour every number field shares: suggestions, the cursor (see `input.rs`).
    input: ConstraintInput,
    /// Where the fields were drawn last frame, so a click beside them can be told apart.
    rects: Vec<Rect>,
    /// Tab presses taken from the input this frame (see `take_tabs`).
    tabs: i32,
    /// The field that has (or is getting) the cursor.
    current: usize,
    /// Taking the values was refused because a field can't be used yet.
    blocked: bool,
    /// A right-click menu of a field is open (a click in it is not a click in the drawing).
    menu_open: bool,
}

fn label(d: Dim) -> &'static str {
    match d {
        Dim::Width => "Breite",
        Dim::Height => "Höhe",
        Dim::Diameter => "Durchmesser",
        Dim::Radius => "Radius",
        Dim::TextHeight => "Schrifthöhe",
    }
}

/// A value as shown in a field: up to three decimals, without trailing zeros.
pub(super) fn fmt(v: f64) -> String {
    let s = format!("{v:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Select all of a field's text, so typing replaces the value (as in the original). Returns
/// false while the field has not been drawn yet (it has no state to select in).
fn select_all(ctx: &egui::Context, id: Id, len: usize) -> bool {
    let Some(mut state) = egui::TextEdit::load_state(ctx, id) else { return false };
    let range = egui::text_selection::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(len));
    state.cursor.set_char_range(Some(range));
    state.store(ctx, id);
    true
}

/// Give a field the cursor with all its text selected, effective from the next frame (before
/// any further typing). Returns false if the field has not been drawn yet.
pub(super) fn enter_field(ctx: &egui::Context, id: Id, len: usize) -> bool {
    ctx.memory_mut(|m| m.request_focus(id));
    ctx.request_repaint();
    select_all(ctx, id, len)
}

/// Put the cursor at the end of a field's text, e.g. after a suggestion was taken. Returns false
/// while the field has not been drawn yet.
pub(super) fn focus_end(ctx: &egui::Context, id: Id, len: usize) -> bool {
    ctx.memory_mut(|m| m.request_focus(id));
    ctx.request_repaint();
    let Some(mut state) = egui::TextEdit::load_state(ctx, id) else { return false };
    state.cursor.set_char_range(Some(egui::text_selection::CCursorRange::one(egui::text::CCursor::new(len))));
    state.store(ctx, id);
    true
}

/// `name` put into a field's text: it replaces the unfinished name at the end, or a plain number,
/// or follows a formula's last operator.
fn insert_name(text: &str, name: &str) -> String {
    if vars::completion_token(text).is_some() || text.trim().is_empty() || !calc::is_formula(text) {
        if vars::completion_token(text).is_some() {
            return vars::apply_completion(text, name);
        }
        return name.to_string();
    }
    format!("{text}{name}")
}

/// A dimension's value in a small rounded label centred on `at`, in the line's colour.
pub(super) fn value_label(painter: &egui::Painter, at: Pos2, text: String, tone: Color32) -> Rect {
    let galley = painter.layout_no_wrap(text, FontId::monospace(12.0), tone);
    let r = egui::Rect::from_center_size(at, galley.size() + Vec2::new(10.0, 4.0));
    painter.rect(r, 4.0, crate::theme::surface(), Stroke::new(1.0_f32, if tone == INK { crate::theme::outline() } else { tone }));
    painter.galley(r.center() - galley.size() / 2.0, galley, tone);
    r
}

/// A small text field centred on `pos`; `invalid` draws its frame red.
pub(super) fn field(ctx: &egui::Context, key: Id, pos: Pos2, text: &mut String, invalid: bool) -> egui::Response {
    egui::Area::new(key.with("area"))
        .fixed_pos(pos - Vec2::new(38.0, 11.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::none().fill(crate::theme::surface()).stroke(Stroke::new(if invalid { 1.6 } else { 1.0_f32 }, if invalid { Color32::from_rgb(0xc0, 0x30, 0x30) } else { crate::theme::outline() })).rounding(5.0).inner_margin(egui::Margin::symmetric(4.0, 1.0)).show(ui, |ui| {
                // Tab must not wander off to other widgets: the fields handle it themselves
                ui.add(egui::TextEdit::singleline(text).id(key).desired_width(68.0).font(egui::TextStyle::Monospace).frame(false).lock_focus(true))
            })
        })
        .inner
        .inner
}

impl App {
    /// Draw one dimension line (offset outside the shape, with end ticks) and return where its
    /// value or field goes.
    fn draw_dim_line(&self, painter: &egui::Painter, line: &DimLine, tone: Color32) -> Pos2 {
        let (a, b) = (self.to_screen(line.from), self.to_screen(line.to));
        let along = (b - a).normalized();
        let across = Vec2::new(-along.y, along.x);
        // which side is "outside" on screen (Y points down there)
        let out = if line.outward == [0.0, 0.0] {
            Vec2::ZERO
        } else {
            let o = self.to_screen([line.from[0] + line.outward[0], line.from[1] + line.outward[1]]) - a;
            if o.length() > 0.0 {
                o.normalized()
            } else {
                Vec2::ZERO
            }
        };
        let shift = out * LINE_GAP;
        let (a2, b2) = (a + shift, b + shift);
        // a size tied to a variable is drawn in the constraint colour, the others in ink
        let stroke = Stroke::new(if tone == INK { 1.0_f32 } else { 1.4 }, tone);
        let ext = if tone == INK { MUTED } else { tone.gamma_multiply(0.7) };
        if out != Vec2::ZERO {
            // extension lines from the shape to the dimension line
            painter.line_segment([a + out * 3.0, a2 + out * 4.0], Stroke::new(0.8_f32, ext));
            painter.line_segment([b + out * 3.0, b2 + out * 4.0], Stroke::new(0.8_f32, ext));
        }
        painter.extend(egui::Shape::dashed_line(&[a2, b2], stroke, 5.0, 3.0));
        for p in [a2, b2] {
            painter.line_segment([p - across * 5.0, p + across * 5.0], stroke);
        }
        let mid = a2 + (b2 - a2) / 2.0;
        // a radius: the value goes beyond its outer end, along it
        if out == Vec2::ZERO {
            b2 + along * RADIUS_FIELD_GAP
        } else {
            mid + out * FIELD_GAP
        }
    }

    /// While a shape is being dragged out: its dimension lines with their current values.
    pub(super) fn decoration_preview(&self, painter: &egui::Painter, shape: &Shape) {
        for line in shape.decoration() {
            let at = self.draw_dim_line(painter, &line, INK);
            value_label(painter, at, format!("{} {}", fmt(shape.dimension_value(&line)), line.dim.unit()), INK);
        }
    }

    /// With "Maße anzeigen" on (the switch beside grid and magnet): the dimension lines of the
    /// selected shapes, and of every shape for the sizes that are tied to a variable — those in
    /// blue with their formula, since they are constraints.
    pub(super) fn draw_dimensions(&mut self, painter: &egui::Painter) {
        self.dim_labels.clear();
        // with "Maße anzeigen" off, the constraints (sizes that were set) of the chosen shapes still
        // show with their dimension; the free sizes only show when the switch is on
        let all = self.show_dims;
        if !all && self.edit.selection.is_empty() {
            return;
        }
        let mut labels = Vec::new();
        for it in &self.shapes {
            let selected = self.edit.selection.contains(&it.id);
            // the open size fields draw this shape's lines themselves
            if (!selected && it.bound.is_empty()) || (!all && !selected) || self.dims.as_ref().is_some_and(|d| d.id == it.id) {
                continue;
            }
            for line in it.shape.decoration() {
                // a size that is a distance constraint is drawn with the constraints
                if self.distances.iter().any(|c| shape_edit::constraints::size_dim(it, c) == Some(line.dim)) {
                    continue;
                }
                let bound = it.bound.iter().find(|(d, _)| *d == line.dim).map(|(_, expr)| expr);
                if bound.is_none() && (!selected || !all) {
                    continue;
                }
                let value = format!("{} {}", fmt(it.shape.dimension_value(&line)), line.dim.unit());
                match bound {
                    Some(expr) => {
                        // a chosen constraint is drawn in the selection colour
                        let chosen = matches!(self.selected_constraint, Some(super::distance::LabelTarget::ShapeDim(id, dim)) if id == it.id && dim == line.dim);
                        let tone = if chosen { SELECT } else { CONSTRAINT };
                        let at = self.draw_dim_line(painter, &line, tone);
                        // a plain number typed in is a constraint too, shown as just the value
                        let text = if calc::is_formula(expr) { format!("ƒ {expr} = {value}") } else { value.clone() };
                        labels.push((it.id, line.dim, value_label(painter, at, text, tone)));
                    }
                    None => {
                        let at = self.draw_dim_line(painter, &line, INK);
                        labels.push((it.id, line.dim, value_label(painter, at, value, INK)));
                    }
                }
            }
        }
        self.dim_labels = labels;
    }

    /// Open the size fields of a shape with the cursor in one size (the shape is selected first
    /// when it isn't). Used when a shown dimension value is held on.
    pub(super) fn open_dim_label(&mut self, id: u64, dim: Dim) {
        self.edit.selection = vec![id];
        self.open_dims(id);
        let at = self.shapes.iter().find(|s| s.id == id).and_then(|s| s.shape.dimensions().iter().position(|(d, _)| *d == dim)).unwrap_or(0);
        if let Some(d) = &mut self.dims {
            d.input.select = Some(at);
            d.current = at;
        }
    }

    /// Show the dimension fields for a shape (just drawn, or chosen with Enter). A size bound to a
    /// formula shows that formula, not just its number.
    pub(super) fn open_dims(&mut self, id: u64) {
        let Some(it) = self.shapes.iter().find(|s| s.id == id) else { return };
        let fields: Vec<(Dim, String)> = it
            .shape
            .dimensions()
            .into_iter()
            .map(|(d, v)| {
                // a size that is a constraint shows its text (a number, a formula, variables)
                let constraint = self.distances.iter().find(|c| shape_edit::constraints::size_dim(it, c) == Some(d)).map(|c| c.expr.clone());
                (d, constraint.or_else(|| it.bound.iter().find(|(b, _)| *b == d).map(|(_, expr)| expr.clone())).unwrap_or_else(|| fmt(v)))
            })
            .collect();
        let touched = vec![false; fields.len()];
        self.dims = Some(DimEdit { id, fields, touched, input: ConstraintInput::selecting(0), rects: vec![], tabs: 0, current: 0, blocked: false, menu_open: false });
        self.status = "Wert eingeben · Formel eingeben und „=“ drücken · TAB: zum nächsten Feld · ENTER zum Anwenden (oder Klick in die Arbeitsfläche) · ESC zum Abbrechen".into();
    }

    /// What is typed in the open size fields, for the self-test's messages.
    pub(super) fn dims_texts(&self) -> Vec<String> {
        self.dims.as_ref().map(|d| d.fields.iter().map(|(_, t)| t.clone()).collect()).unwrap_or_default()
    }

    /// Open the size fields of the one selected shape (Enter, or Bearbeiten → Maße).
    pub(super) fn open_selected_dims(&mut self) {
        if let [id] = self.edit.selection[..] {
            self.open_dims(id);
        }
    }

    /// While size fields are open, Tab belongs to them: take it out of the input before egui's
    /// own focus moving sees it (that would only respect a text field one frame after it got the
    /// cursor), and count it for [`Self::dims_ui`]: +1 forward, −1 back with Shift.
    pub(super) fn take_tabs(&mut self, raw: &mut egui::RawInput) {
        // the value field of a distance constraint: Tab takes a suggestion (noted for `distance_edit_ui`)
        if self.distance_edit.is_some() {
            let mut pressed = false;
            raw.events.retain(|e| match e {
                egui::Event::Key { key: egui::Key::Tab, pressed: p, .. } => {
                    pressed |= *p;
                    false
                }
                _ => true,
            });
            self.distance_tab |= pressed;
        }
        let Some(d) = &mut self.dims else { return };
        raw.events.retain(|e| match e {
            egui::Event::Key { key: egui::Key::Tab, pressed, modifiers, .. } => {
                if *pressed {
                    d.tabs += if modifiers.shift { -1 } else { 1 };
                }
                false
            }
            _ => true,
        });
    }

    /// The number a field's text stands for, or why it can't be used yet: an unknown variable, a
    /// broken formula, or a size that must be above zero.
    fn field_value(&self, _dim: Dim, text: &str) -> Result<f64, String> {
        let text = text.trim().trim_end_matches('=');
        let v = calc::eval(text, &self.vars_resolved).map_err(|e| e.message())?;
        if v <= 0.0 {
            return Err("muss größer als 0 sein".into());
        }
        Ok(v)
    }

    /// The first field that can't be used yet, with the reason.
    fn first_invalid(&self, d: &DimEdit) -> Option<(usize, String)> {
        d.fields.iter().enumerate().find_map(|(i, (dim, text))| self.field_value(*dim, text).err().map(|e| (i, e)))
    }

    /// Apply every field's value (formulas included) to the shape; a field with variables stays a
    /// formula on the shape, a plain number replaces any old formula.
    fn apply_dims(&mut self, d: &DimEdit) {
        let Some(idx) = self.shapes.iter().position(|s| s.id == d.id) else { return };
        let values: Vec<(Dim, f64, Option<String>)> = d
            .fields
            .iter()
            .enumerate()
            .filter_map(|(i, (dim, text))| {
                let expr = text.trim().trim_end_matches('=').trim().to_string();
                let with_names = !calc::names_in(&expr).is_empty();
                // a field left as it was (a value from drawing, or an existing constraint) stays
                if !with_names && !d.touched.get(i).copied().unwrap_or(false) {
                    return None;
                }
                let v = self.field_value(*dim, text).ok()?;
                // typed in: the size is a constraint, its text the expression
                Some((*dim, v, Some(expr)))
            })
            .collect();
        for (dim, v, expr) in values {
            self.set_dimension(idx, dim, v, expr);
        }
    }

    /// A press in the drawing beside the fields works like Enter: the values are taken and the
    /// fields close, unless one of them can't be used yet — then they stay open and say which.
    /// Returns whether the press was used up, so it does nothing else.
    pub(super) fn accept_dims_on_press(&mut self, ctx: &egui::Context, canvas: Rect) -> bool {
        let Some(d) = &self.dims else { return false };
        let at = ctx.input(|i| if i.pointer.primary_pressed() { i.pointer.press_origin() } else { None });
        let Some(p) = at else { return false };
        if d.menu_open || !canvas.contains(p) || d.rects.iter().any(|r| r.expand(6.0).contains(p)) || d.input.popup.iter().any(|(_, r, _)| r.contains(p)) {
            return false;
        }
        if let Some((i, why)) = self.first_invalid(d) {
            self.dims_block(i, &why);
            return true;
        }
        let d = self.dims.take().expect("checked above");
        self.apply_dims(&d);
        self.status = "Maße übernommen".into();
        true
    }

    /// Keep the fields open because field `i` isn't usable: mark it and put the cursor there.
    fn dims_block(&mut self, i: usize, why: &str) {
        if let Some(d) = &mut self.dims {
            d.blocked = true;
            d.input.select = Some(i);
        }
        self.status = format!("Eingabe nicht übernommen: {why}");
    }

    /// Dimension lines and fields of the shape being sized.
    pub(super) fn dims_ui(&mut self, ctx: &egui::Context, painter: &egui::Painter) {
        let Some(mut d) = self.dims.take() else { return };
        let Some(idx) = self.shapes.iter().position(|s| s.id == d.id) else { return };
        // the fields belong to the selected shape; selecting something else closes them
        if !self.edit.selection.contains(&d.id) || self.edit.is_busy() {
            return;
        }
        let lines = self.shapes[idx].shape.decoration();
        let (enter, esc) = ctx.input(|i| (i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Escape)));
        // a suggestion clicked in last frame's list: take it into its field
        if let Some((fi, name)) = d.input.click(ctx) {
            let t = &mut d.fields[fi].1;
            *t = vars::apply_completion(t, &name);
            d.touched[fi] = true;
            d.current = fi;
        }
        // Tab / ⇧Tab go to the next / previous field of this shape, round and round; ↑ ↓ do too
        // ("Use UP, DOWN keys to switch between properties") — unless a suggestion list is open:
        // then ↑ ↓ choose in it and Tab or Enter take the chosen one
        let mut arrows = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown) as i32 - i.key_pressed(egui::Key::ArrowUp) as i32);
        let mut tabs = std::mem::take(&mut d.tabs);
        let names = self.var_names();
        let mut focused = None;
        let (mut close, mut commit) = (false, false);
        let (mut menu_now, mut open_variables) = (false, false);
        let mut changed_by_menu: Option<usize> = None;
        let mut changed: Vec<(Dim, f64, Option<String>)> = Vec::new();
        d.rects.clear();
        for (i, (dim, text)) in d.fields.iter_mut().enumerate() {
            let Some(line) = lines.iter().find(|l| l.dim == *dim) else { continue };
            let tone = if self.shapes[idx].bound.iter().any(|(b, _)| b == dim) { CONSTRAINT } else { INK };
            let pos = self.draw_dim_line(painter, line, tone);
            if !matches!(dim, Dim::Width | Dim::Height) {
                painter.text(pos - Vec2::new(44.0, 0.0), Align2::RIGHT_CENTER, label(*dim), FontId::proportional(11.0), MUTED);
            }
            let key = Id::new(("dim-field", d.id, i));
            // unusable text is marked red once the cursor has left it, or when taking it was refused
            let invalid = self.field_value(*dim, text);
            let red = invalid.is_err() && (d.blocked || !ctx.memory(|m| m.has_focus(key)));
            let resp = field(ctx, key, pos, text, red);
            // egui's own focus: `resp.has_focus()` also wants the OS window to be in front, which
            // typing does not need (and a window just started, or behind another, would fail)
            let has_focus = ctx.memory(|m| m.has_focus(key));
            d.rects.push(resp.rect);
            // right-click: the variables to insert, as in the original's calc fields ("Neue
            // Dokumentvariable einfügen" / "Keine Dokumentvariablen")
            if resp
                .context_menu(|ui| {
                    if names.is_empty() {
                        ui.add_enabled(false, egui::Button::new("Keine Variablen"));
                    }
                    for n in &names {
                        if ui.button(n).clicked() {
                            *text = insert_name(text, n);
                            changed_by_menu = Some(i);
                            d.touched[i] = true;
                            ui.close_menu();
                        }
                    }
                    ui.separator();
                    if ui.button("Neue Variable einfügen…").clicked() {
                        open_variables = true;
                        ui.close_menu();
                    }
                })
                .is_some()
            {
                menu_now = true;
            }
            if let (true, Err(why)) = (red, &invalid) {
                painter.text(resp.rect.left_bottom() + Vec2::new(2.0, 3.0), Align2::LEFT_TOP, why, FontId::proportional(10.5), Color32::from_rgb(0xc0, 0x30, 0x30));
            }
            // the first field of a new shape gets the cursor as soon as it exists, and after taking
            // a suggestion it goes back into the field, behind the name
            d.input.cursor(ctx, i, key, text);
            if resp.changed() {
                d.touched[i] = true;
                // like the original's fields, typed or pasted text never keeps Tab or line
                // breaks (0x100245c20 drops characters 9, 10 and 13)
                text.retain(|c| !matches!(c, '\t' | '\n' | '\r'));
                let expr = text.trim().trim_end_matches('=').trim().to_string();
                let finished = text.trim_end().ends_with('=');
                if !calc::names_in(&expr).is_empty() {
                    // with variables: the formula stays on the shape and follows the variables
                    if let Ok(v) = calc::eval(&expr, &self.vars_resolved) {
                        if finished {
                            *text = expr.clone();
                        }
                        changed.push((*dim, v, Some(expr)));
                    }
                } else if finished {
                    // a finished formula: show its result
                    if let Some(v) = calc::evaluate(text) {
                        *text = fmt(v);
                        changed.push((*dim, v, Some(text.trim().to_string())));
                    }
                } else if !calc::is_formula(text) {
                    // plain numbers apply while typing, and are a length constraint (the number is
                    // the expression)
                    if let Some(v) = calc::evaluate(text) {
                        changed.push((*dim, v, Some(text.trim().to_string())));
                    }
                }
            }
            // variable names that continue what is being typed (the same for every number field)
            let mut keys = Keys { tab: tabs > 0, arrows, enter };
            let value = |n: &str| self.vars_resolved.value(n).ok();
            let suggested = d.input.suggest(ctx, i, key.with("list"), resp.rect, text, &names, &value, &mut keys, has_focus || (resp.lost_focus() && enter), 16.0);
            arrows = keys.arrows;
            if !keys.tab {
                tabs = 0;
            }
            if let Some(area) = suggested.area {
                d.rects.push(area);
            }
            let taken = suggested.taken;
            if resp.lost_focus() && enter && !taken {
                commit = true;
            }
            if has_focus && esc {
                close = true;
            }
            if has_focus {
                focused = Some(i);
            }
        }
        d.menu_open = menu_now;
        if open_variables {
            self.vars_open = true;
        }
        if let Some(i) = changed_by_menu {
            d.input.refocus = Some(i);
        }
        if let Some(i) = focused {
            d.current = i;
        }
        let step = tabs + arrows;
        if step != 0 && !d.fields.is_empty() {
            // from the field that has (or is getting) the cursor; a single field keeps it and
            // selects its text again
            let n = d.fields.len() as i32;
            let next = (d.current as i32 + step).rem_euclid(n) as usize;
            d.current = next;
            // every field was drawn this frame, so its text can be selected right away
            if !enter_field(ctx, Id::new(("dim-field", d.id, next)), d.fields[next].1.chars().count()) {
                d.input.select = Some(next);
            }
        }
        for (dim, v, expr) in changed {
            self.set_dimension(idx, dim, v, expr);
        }
        if commit {
            // Enter: take the values, but only when every field is usable — otherwise stay open
            match self.first_invalid(&d) {
                Some((i, why)) => {
                    d.blocked = true;
                    d.input.select = Some(i);
                    self.status = format!("Eingabe nicht übernommen: {why}");
                }
                None => {
                    self.apply_dims(&d);
                    close = true;
                }
            }
        }
        if close {
            self.status = "Maße übernommen".into();
        } else {
            if d.blocked && self.first_invalid(&d).is_none() {
                d.blocked = false;
            }
            self.dims = Some(d);
        }
    }

    /// The angle field of the rotate mode: shows the turn while dragging, takes a typed angle.
    pub(super) fn angle_ui(&mut self, ctx: &egui::Context, painter: &egui::Painter) {
        if self.tool != DrawTool::Select || !self.edit.rotate_mode {
            return;
        }
        let Some((lo, hi)) = selection_bounds(&self.shapes, &self.edit.selection) else { return };
        let pad = self.view().handle_pad;
        let top = self.to_screen([(lo[0] + hi[0]) / 2.0, hi[1] + pad]);
        let pos = top - Vec2::new(0.0, 40.0);
        // a little curved arrow under the field, as in the original
        let arc: Vec<Pos2> = (0..=12).map(|i| top - Vec2::new(0.0, 4.0) + Vec2::angled(std::f32::consts::PI * (1.15 + 0.7 * i as f32 / 12.0)) * 10.0).collect();
        painter.add(egui::Shape::line(arc, Stroke::new(1.6_f32, INK)));
        let key = Id::new("angle-field");
        if let Some(a) = self.edit.rotation_angle() {
            let mut shown = format!("{} °", fmt(a));
            field(ctx, key.with("live"), pos, &mut shown, false);
            return;
        }
        let mut text = std::mem::take(&mut self.angle_text);
        let resp = field(ctx, key, pos, &mut text, false);
        if resp.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(deg) = calc::evaluate(&text).filter(|d| *d != 0.0) {
                self.push_undo();
                self.edit.rotate_selection(&mut self.shapes, deg);
                self.status = format!("Um {} ° gedreht", fmt(deg));
            }
            text = "0".into();
        } else if text.trim_end().ends_with('=') {
            if let Some(v) = calc::evaluate(&text) {
                text = fmt(v);
            }
        }
        self.angle_text = text;
    }
}
