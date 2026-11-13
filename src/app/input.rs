//! `ConstraintInput`: the behaviour every field has that takes a number, a formula or a variable
//! — the size fields of a shape, the value of a distance constraint, the expression of a new
//! variable. The original has one field class for all of them too (`uvs::NumberEditField` and
//! its variants, see MECHANICS.md Nachtrag 39).
//!
//! The text field itself is drawn by the place that owns it (each puts it somewhere else); this
//! component does the rest, the same everywhere:
//! * **suggestions**: the variables (and the job's values) that continue the name being typed, in
//!   a list under the field with their values; ↑ ↓ choose, Tab or Enter or a click take one;
//! * **cursor**: all of a field's text selected when it is entered, and the cursor back behind a
//!   name that was taken;
//! * **errors**: a field that can't be worked out yet says why, under the field.
//!
//! One `ConstraintInput` serves a group of fields (field numbers 0, 1, 2 …) of which one has the
//! cursor at a time.

use super::fields::{enter_field, fmt, focus_end};
use egui::{Color32, Context, Id, Rect, Vec2};
use shape_edit::{calc, vars};

/// The keys that matter to the suggestions this frame. The caller reads them from the input (Tab
/// may have been taken out of it already, see `take_tabs`) and hands them over; a suggestion that
/// uses a key resets it, so the caller can tell it was used.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Keys {
    /// Tab (or, for several fields, a Tab press count above zero).
    pub tab: bool,
    /// Net ↓ presses minus ↑ presses.
    pub arrows: i32,
    /// Enter was pressed (it takes the cursor out of a one-line field in the same frame).
    pub enter: bool,
}

impl Keys {
    pub(super) fn read(ctx: &Context, tab: bool) -> Keys {
        let (enter, down, up) = ctx.input(|i| (i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::ArrowDown), i.key_pressed(egui::Key::ArrowUp)));
        Keys { tab, arrows: down as i32 - up as i32, enter }
    }
}

/// What the suggestions did this frame.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Suggested {
    /// A suggestion was taken (Tab, Enter or click): Enter was used up by it.
    pub taken: bool,
    /// The list is showing.
    pub listing: bool,
    /// Where the list was drawn, so a click in it is not a click beside the field.
    pub area: Option<Rect>,
}

/// The state of the suggestions and the cursor of a group of fields.
#[derive(Default)]
pub(super) struct ConstraintInput {
    /// The chosen line of the list.
    pub sel: usize,
    /// The lines of last frame's list (field, place, name), so a click on one can be told apart.
    pub popup: Vec<(usize, Rect, String)>,
    /// A field to put the cursor in behind a name that was taken.
    pub refocus: Option<usize>,
    /// A field to put the cursor in with all its text selected, as soon as it exists.
    pub select: Option<usize>,
}

impl ConstraintInput {
    /// Starts by entering `field` (its text selected).
    pub(super) fn selecting(field: usize) -> Self {
        ConstraintInput { select: Some(field), ..Default::default() }
    }

    /// Call once a frame before the fields are shown: a click on a line of last frame's list.
    /// Returns the field and the name to put into it (and the cursor goes back there).
    pub(super) fn click(&mut self, ctx: &Context) -> Option<(usize, String)> {
        let press = ctx.input(|i| if i.pointer.primary_pressed() { i.pointer.press_origin() } else { None });
        let hit = press.and_then(|p| self.popup.iter().find(|(_, r, _)| r.contains(p)).map(|(f, _, n)| (*f, n.clone())));
        self.popup.clear();
        if let Some((f, _)) = &hit {
            self.refocus = Some(*f);
            self.sel = 0;
        }
        hit
    }

    /// The suggestions of field `field` (its text edit sits at `rect`, it has the cursor when
    /// `focused`): shows the list under it and takes a name when Tab, Enter or a click say so.
    /// `names` are the names that may be suggested, `value` the number behind a name.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn suggest(&mut self, ctx: &Context, field: usize, id: Id, rect: Rect, text: &mut String, names: &[String], value: &dyn Fn(&str) -> Option<f64>, keys: &mut Keys, focused: bool, gap: f32) -> Suggested {
        let mut out = Suggested::default();
        if !focused {
            return out;
        }
        let Some(token) = vars::completion_token(text).map(|(_, t)| t.to_string()) else { return out };
        // a name that is already complete needs no list
        let matches: Vec<String> = vars::completions(names, &token, 6).into_iter().filter(|m| *m != token).collect();
        if matches.is_empty() {
            return out;
        }
        out.listing = true;
        self.sel = self.sel.min(matches.len() - 1);
        if keys.arrows != 0 {
            self.sel = (self.sel as i32 + keys.arrows).rem_euclid(matches.len() as i32) as usize;
            keys.arrows = 0;
        }
        if keys.tab || keys.enter {
            *text = vars::apply_completion(text, &matches[self.sel]);
            keys.tab = false;
            self.refocus = Some(field);
            self.sel = 0;
            out.taken = true;
            out.listing = false;
            return out;
        }
        let sel = self.sel;
        let mut lines = Vec::new();
        let area = egui::Area::new(id.with("complete")).fixed_pos(rect.left_bottom() + Vec2::new(0.0, gap)).order(egui::Order::Foreground).show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width((rect.width() - 16.0).max(80.0));
                for (k, name) in matches.iter().enumerate() {
                    let shown = match value(name) {
                        Some(v) => format!("{name}   = {}", fmt(v)),
                        None => name.clone(),
                    };
                    let r = ui.selectable_label(k == sel, egui::RichText::new(shown).monospace());
                    lines.push((field, r.rect, name.clone()));
                }
            });
        });
        self.popup.extend(lines);
        out.area = Some(area.response.rect);
        out
    }

    /// Call after the field was drawn: the cursor with all its text selected when a field is
    /// entered, and back behind a taken name.
    pub(super) fn cursor(&mut self, ctx: &Context, field: usize, id: Id, text: &str) {
        if self.select == Some(field) && enter_field(ctx, id, text.chars().count()) {
            self.select = None;
        }
        if self.refocus == Some(field) && focus_end(ctx, id, text.chars().count()) {
            self.refocus = None;
        }
    }
}

/// Why `text` can't be worked out yet (an unknown variable, a broken formula, or a result that
/// must be above zero when `positive`), for the line under a field.
pub(super) fn problem(text: &str, lookup: &dyn calc::Lookup, positive: bool) -> Option<String> {
    let t = text.trim().trim_end_matches('=').trim();
    match calc::eval(t, lookup) {
        Ok(v) if positive && v <= 0.0 => Some(t!("muss größer als 0 sein").into()),
        Ok(_) => None,
        Err(e) => Some(crate::lang::calc_message(&e)),
    }
}

/// The colour of an error line under a field.
pub(super) const ERROR: Color32 = Color32::from_rgb(0xc0, 0x30, 0x30);
