//! Variables in the app: the dialog that lists and edits them (opened with the f(x) button next
//! to "Job einrichten"), working out their values, and keeping the sizes that are bound to them
//! up to date. The sums themselves are in `shape_edit::vars` and `shape_edit::calc`.
//!
//! A size typed as a formula or variable ("hoehe", "breite/2 - rand") is kept on its shape as
//! text ([`shape_edit::Item::bound`]); the shape's own number is derived from it. When a variable
//! (or the job's size) changes, every bound size is worked out again once and the shape rebuilt
//! from it, so no old value can survive. A bound size grows from the shape's lower-left corner
//! ([`shape_edit::Shape::with_dimension_anchored`]): change `hoehe` and the top moves.

use super::App;
use crate::theme::{ink, muted};
use super::input::ConstraintInput;
use egui::Color32;

/// The "Neue Variable" dialog while it is open.
pub(super) struct NewVariable {
    pub(super) name: String,
    pub(super) expr: String,
    pub(super) comment: String,
    /// The behaviour every number field shares: suggestions, the cursor (see `input.rs`). Field 0
    /// is the name, field 1 the expression.
    pub(super) input: ConstraintInput,
    /// The cursor was in the name last frame, and the expression gets it (all selected) when Tab
    /// leaves the name.
    pub(super) name_had_focus: bool,
}
use shape_edit::calc;
use shape_edit::{Dim, VarEvent};

/// Values from the job that formulas may use without defining them.
pub(in crate::app) const FROM_JOB: [(&str, crate::lang::Tr); 3] = [("platte_b", tr!("Breite der Platte (X)")), ("platte_l", tr!("Länge der Platte (Y)")), ("platte_d", tr!("Dicke der Platte (Z)"))];

impl App {
    /// Work the variables out again when they or the job's size changed, then rebuild the bound
    /// shapes and the toolpaths made from them.
    pub(super) fn refresh_vars(&mut self) {
        let job = [self.job.width, self.job.height, self.job.thickness];
        if !self.vars_dirty && job == self.vars_job && self.var_events.is_empty() {
            return;
        }
        self.vars_dirty = false;
        self.vars_job = job;
        let mut events = std::mem::take(&mut self.var_events);
        // 1. structure: users follow a rename, bindings to a deleted variable are released
        let mut users_changed = false;
        for event in &events {
            match event {
                VarEvent::Renamed { from, to } => {
                    self.vars.rename_users(from, to);
                    for tp in &mut self.toolpaths {
                        users_changed |= tp.rename_variable(from, to);
                    }
                    for c in &mut self.distances {
                        if calc::names_in(&c.expr).iter().any(|n| n == from) {
                            c.expr = calc::rename_in(&c.expr, from, to);
                            users_changed = true;
                        }
                    }
                    for item in &mut self.shapes {
                        users_changed |= item.rename_variable(from, to);
                    }
                }
                VarEvent::Removed(name) => {
                    for tp in &mut self.toolpaths {
                        tp.release_variable(name);
                    }
                    // a distance that used the variable is released: the shapes stay where they are
                    let before = self.distances.len();
                    self.distances.retain(|c| !calc::names_in(&c.expr).iter().any(|n| n == name));
                    users_changed |= self.distances.len() != before;
                    for item in &mut self.shapes {
                        users_changed |= item.release_variable(name);
                    }
                }
                VarEvent::Added(_) | VarEvent::ValueChanged(_) => {}
            }
        }
        // 2. values: work them out and tell what changed
        let before = std::mem::take(&mut self.vars_resolved);
        self.vars_resolved = self.vars.resolve(&[(FROM_JOB[0].0, job[0]), (FROM_JOB[1].0, job[1]), (FROM_JOB[2].0, job[2])]);
        events.extend(self.vars_resolved.changes_since(&before).into_iter().map(VarEvent::ValueChanged));
        // 3. a new value reaches the shapes bound to it and the toolpaths made from them
        let value_changed = events.iter().any(|e| matches!(e, VarEvent::ValueChanged(_)));
        let bound_changed = (value_changed || users_changed) && self.apply_bindings();
        // depths of toolpaths that are formulas ("platte_d/2") follow the variables and the job
        let mut depth_changed = false;
        for tp in &mut self.toolpaths {
            depth_changed |= tp.resolve_depths(&self.vars_resolved);
        }
        let needs_initial = self.toolpaths.iter().any(|tp| tp.moves.is_empty() && !tp.shape_ids.is_empty());
        if bound_changed || depth_changed || needs_initial {
            self.recompute_all();
        }
        self.var_log = events;
    }

    /// Every name a formula may use, sorted: the variables (also those with an error, so they can
    /// still be completed and fixed) and the job's values.
    pub(super) fn var_names(&self) -> Vec<String> {
        let mut names = self.vars_resolved.names();
        names.extend(self.vars.list.iter().map(|v| v.name.clone()));
        names.sort();
        names.dedup();
        names
    }

    /// Set every bound size from its formula. Returns whether any shape changed.
    fn apply_bindings(&mut self) -> bool {
        let mut changed = false;
        for item in &mut self.shapes {
            // corner radii typed as formulas follow the variables too
            changed |= item.resolve_corners(&self.vars_resolved);
            for (dim, expr) in item.bound.clone() {
                let Ok(value) = calc::eval(&expr, &self.vars_resolved) else { continue };
                let next = item.shape.with_dimension_anchored(dim, value);
                if next != item.shape {
                    item.shape = next;
                    changed = true;
                }
            }
        }
        changed
    }

    /// A size of shape `idx` changed by hand or by typing: set it, and bind it to `expr` when it
    /// was typed with variables (or forget an old binding when it was a plain number).
    pub(super) fn set_dimension(&mut self, idx: usize, dim: Dim, value: f64, expr: Option<String>) {
        let id = self.shapes[idx].id;
        self.shapes[idx].bound.retain(|(d, _)| *d != dim);
        let pair = shape_edit::constraints::length_pair(&self.shapes[idx], dim);
        match (expr, pair) {
            // a typed size on a shape with sizes along the axes: the same constraint as made with
            // the dimension mode (D); the size is set right away, the solver keeps it
            (Some(expr), Some(pair)) => {
                let item = &mut self.shapes[idx];
                item.shape = item.shape.with_dimension_anchored(dim, value);
                self.upsert_length_constraint(id, pair, expr);
            }
            (Some(expr), None) => {
                let item = &mut self.shapes[idx];
                item.shape = item.shape.with_dimension_anchored(dim, value);
                item.bound.push((dim, expr));
            }
            // a plain value without constraint: any constraint on this size goes
            (None, _) => {
                let item = &mut self.shapes[idx];
                item.shape = item.shape.with_dimension(dim, value);
                let item = self.shapes[idx].clone();
                self.distances.retain(|c| shape_edit::constraints::size_dim(&item, c) != Some(dim));
            }
        }
    }

    /// The selected shapes become plain numbers again (after a hand edit that a formula would
    /// otherwise undo).
    pub(super) fn drop_bindings_of_selection(&mut self) {
        for item in self.shapes.iter_mut().filter(|s| self.edit.selection.contains(&s.id)) {
            item.bound.clear();
        }
    }

    /// "Neue Variable": opens the dialog (name, expression, value, comment), as in Fusion 360's
    /// "Add User Parameter". The name starts as a free one (v1, v2 …) with all of it selected, so
    /// the right name can simply be typed.
    pub(super) fn open_new_variable(&mut self) {
        let mut n = self.vars.list.len() + 1;
        while self.vars.list.iter().any(|v| v.name == format!("v{n}")) || FROM_JOB.iter().any(|(j, _)| *j == format!("v{n}")) {
            n += 1;
        }
        self.new_var = Some(NewVariable { name: format!("v{n}"), expr: "10".into(), comment: String::new(), input: ConstraintInput::selecting(0), name_had_focus: false });
    }

    /// Put the variable the dialog describes into the document.
    pub(in crate::app) fn add_variable(&mut self, name: &str, expr: &str, comment: &str) {
        self.vars.set(name, expr);
        if let Some(v) = self.vars.list.iter_mut().find(|v| v.name == name) {
            v.comment = comment.to_string();
        }
        self.vars_committed.push(name.to_string());
        self.var_events.push(VarEvent::Added(name.to_string()));
        self.vars_dirty = true;
    }

}

pub(in crate::app) enum Glyph {
    Cross,
}

/// A small square button with a drawn cross (the font may lack symbol glyphs).
pub(in crate::app) fn glyph_button(ui: &mut egui::Ui, glyph: Glyph) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, 4.0, crate::theme::chip());
    }
    let colour = if resp.hovered() { crate::theme::error() } else { muted() };
    let c = rect.center();
    let stroke = egui::Stroke::new(1.6_f32, colour);
    match glyph {
        Glyph::Cross => {
            ui.painter().line_segment([c + egui::vec2(-4.5, -4.5), c + egui::vec2(4.5, 4.5)], stroke);
            ui.painter().line_segment([c + egui::vec2(-4.5, 4.5), c + egui::vec2(4.5, -4.5)], stroke);
        }
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A button with a drawn plus and a text.
pub(in crate::app) fn add_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(text.to_string(), egui::FontId::proportional(14.0), ink());
    let size = egui::vec2(34.0 + galley.size().x, 30.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let fill = if resp.hovered() { crate::theme::chip_hover() } else { crate::theme::chip() };
    ui.painter().rect(rect, 4.0, fill, egui::Stroke::new(1.0_f32, crate::theme::field_stroke()));
    let c = egui::pos2(rect.left() + 17.0, rect.center().y);
    let stroke = egui::Stroke::new(1.6_f32, ink());
    ui.painter().line_segment([c - egui::vec2(5.0, 0.0), c + egui::vec2(5.0, 0.0)], stroke);
    ui.painter().line_segment([c - egui::vec2(0.0, 5.0), c + egui::vec2(0.0, 5.0)], stroke);
    ui.painter().galley(egui::pos2(rect.left() + 29.0, rect.center().y - galley.size().y / 2.0), galley, ink());
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A result shown as a small grey, right-aligned chip of fixed width.
pub(in crate::app) fn value_chip(ui: &mut egui::Ui, text: &str, fill: Color32, colour: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(86.0, 28.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 4.0, fill);
    ui.painter().text(rect.right_center() - egui::vec2(8.0, 0.0), egui::Align2::RIGHT_CENTER, text, egui::FontId::monospace(13.0), colour);
}

/// What is wrong with a variable's name, in the original's words (VfUi catalogue): a short text
/// for the table and the full message for the tooltip. `dup` says another variable has the name.
pub(in crate::app) fn name_problem(name: &str, dup: bool) -> Option<(crate::lang::Tr, crate::lang::Tr)> {
    if name.is_empty() {
        Some((tr!("Name fehlt"), tr!("Leerer Dokumentvariablenname")))
    } else if name.starts_with('_') {
        Some((tr!("Name beginnt mit _"), tr!("Ein Variablenname darf nicht mit _ beginnen")))
    } else if !calc::is_valid_name(name) {
        Some((tr!("ungültiger Name"), tr!("Falscher Variablenname eingegeben. Ein Variablenname muss mit einem Buchstaben beginnen und dann Buchstaben, Zahlen oder Unterstriche besitzen.")))
    } else if dup || FROM_JOB.iter().any(|(n, _)| *n == name) {
        Some((tr!("Name vergeben"), tr!("Eine Variable mit diesem Namen ist schon angelegt!")))
    } else {
        None
    }
}

/// A value for display: up to three decimals, no trailing zeros.
pub(in crate::app) fn fmt(v: f64) -> String {
    let s = format!("{v:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::name_problem;
    use shape_edit::{Dim, Item, Rect, Variables};

    /// A fixed light grey or white in a fill is what made the dialog unreadable in the dark theme
    /// (light text on a light field): fills here come from `theme`, which has both themes.
    #[test]
    fn the_dialog_has_no_fixed_light_colours() {
        // (only the dialog's own code: the test module below names the patterns it looks for)
        let whole = include_str!("variables.rs");
        let source = &whole[..whole.find("#[cfg(test)]").unwrap_or(whole.len())];
        let mut bad = vec![];
        for (n, line) in source.lines().enumerate() {
            let mut rest = line;
            while let Some(at) = rest.find("Color32::from_rgb(0x") {
                let args = &rest[at + "Color32::from_rgb(".len()..];
                let end = args.find(')').unwrap_or(args.len());
                let parts: Vec<u32> = args[..end].split(',').filter_map(|p| u32::from_str_radix(p.trim().trim_start_matches("0x"), 16).ok()).collect();
                if parts.len() == 3 && parts.iter().all(|c| *c >= 0xc0) {
                    bad.push(t!("Zeile %{n}: %{text}", n = n + 1, text = line.trim()));
                }
                rest = &args[end..];
            }
            if line.contains("Color32::WHITE") || line.contains("from_gray(") {
                bad.push(t!("Zeile %{n}: %{text}", n = n + 1, text = line.trim()));
            }
        }
        assert!(bad.is_empty(), "feste helle Farben im Variablen-Dialog: {bad:?}");
    }

    #[test]
    fn name_rules_are_the_originals() {
        assert!(name_problem("breite", false).is_none());
        assert!(name_problem("a_1", false).is_none());
        assert_eq!(name_problem("", false).unwrap().1.key(), "Leerer Dokumentvariablenname");
        assert_eq!(name_problem("_x", false).unwrap().1.key(), "Ein Variablenname darf nicht mit _ beginnen");
        assert!(name_problem("2b", false).is_some() && name_problem("a b", false).is_some());
        assert!(name_problem("breite", true).is_some(), "a second variable with the name");
        assert!(name_problem("platte_b", false).is_some(), "taken by the job");
    }

    #[test]
    fn bindings_and_variables_survive_saving_and_old_files_still_load() {
        let mut item = Item::new(7, Rect::new([10.0, 10.0], 20.0, 10.0));
        item.bound.push((Dim::Height, "hoehe/2".into()));
        let mut vars = Variables::default();
        vars.set("hoehe", "40");

        let back: Item = serde_json::from_str(&serde_json::to_string(&item).unwrap()).unwrap();
        assert_eq!(back, item, "the formula is saved with the shape");
        let back: Variables = serde_json::from_str(&serde_json::to_string(&vars).unwrap()).unwrap();
        assert_eq!(back, vars);

        // a file from before variables: no "bound" on the shape
        let mut json: serde_json::Value = serde_json::to_value(&item).unwrap();
        json.as_object_mut().unwrap().remove("bound");
        let old: Item = serde_json::from_value(json).unwrap();
        assert!(old.bound.is_empty() && old.id == 7);
    }
}
