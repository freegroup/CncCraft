//! The window "Neue Variable": a name, a value or formula, and a comment.

use crate::app::*;
use crate::app::input::Keys;
use crate::app::variables::*;
use egui::RichText;
use shape_edit::{calc, vars};

impl App {
    /// The "Neue Variable" dialog: Name, Ausdruck, Wert (worked out as it is typed), Kommentar,
    /// OK and Abbrechen. OK is only possible with a valid name and an expression that works out;
    /// Enter is OK, Esc is Abbrechen.
    pub(in crate::app) fn new_variable_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut nv) = self.new_var.take() else { return };
        let red = crate::theme::error();
        let dup = self.vars.list.iter().any(|v| v.name == nv.name);
        let name_problem = name_problem(&nv.name, dup);
        let value = calc::eval(&nv.expr, &self.vars_resolved);
        let valid = name_problem.is_none() && value.is_ok();
        let (mut ok, mut cancel) = (false, false);
        let mut expr_rect = egui::Rect::NOTHING;
        let mut expr_lost_focus = false;
        crate::app::dialogs::window("new_variable_dialog", ctx).anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -40.0)).fixed_size([400.0, 0.0]).show(ctx, |ui| {
            crate::app::dialogs::header(ui, &t!("Neue Variable"), false);
            ui.spacing_mut().item_spacing = egui::vec2(10.0, 12.0);
            // visible outlines round the fields, as in the table
            ui.visuals_mut().extreme_bg_color = crate::theme::field_bg();
            ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, crate::theme::field_stroke());
            egui::Grid::new("new_variable_grid").num_columns(2).spacing([18.0, 12.0]).min_col_width(78.0).show(ui, |ui| {
                ui.label(t!("Name"));
                let name_id = egui::Id::new("new_variable_name");
                ui.vertical(|ui| {
                    let colour = if name_problem.is_some() { red } else { ink() };
                    ui.add(egui::TextEdit::singleline(&mut nv.name).id(name_id).font(egui::TextStyle::Monospace).text_color(colour).desired_width(f32::INFINITY).margin(egui::Margin::symmetric(8.0, 6.0)));
                    if let Some((short, full)) = name_problem {
                        ui.label(RichText::new(short).color(red).size(12.0)).on_hover_text(full);
                    }
                });
                ui.end_row();

                ui.label(t!("Ausdruck"));
                let expr_resp = ui.add(egui::TextEdit::singleline(&mut nv.expr).id(egui::Id::new("new_variable_expr")).font(egui::TextStyle::Monospace).hint_text(t!("Zahl oder Formel, z. B. breite/2")).desired_width(f32::INFINITY).margin(egui::Margin::symmetric(8.0, 6.0)));
                expr_rect = expr_resp.rect;
                expr_lost_focus = expr_resp.lost_focus();
                ui.end_row();

                ui.label(t!("Wert"));
                match &value {
                    Ok(v) => ui.label(RichText::new(fmt(*v)).monospace().color(ink())),
                    Err(e) => ui.label(RichText::new(crate::lang::calc_message(&e)).color(red)),
                };
                ui.end_row();

                ui.label(t!("Kommentar"));
                ui.add(egui::TextEdit::multiline(&mut nv.comment).hint_text(t!("Kommentar eingeben")).desired_rows(4).desired_width(f32::INFINITY).margin(egui::Margin::symmetric(8.0, 6.0)));
                ui.end_row();
            });
            dialog_buttons(ui, |ui| {
                if ui.add_enabled(valid, egui::Button::new(RichText::new(t!("OK")).color(ink())).min_size(egui::vec2(84.0, 30.0))).clicked() {
                    ok = true;
                }
                if ui.add(egui::Button::new(RichText::new(t!("Abbrechen")).color(ink())).min_size(egui::vec2(84.0, 30.0))).clicked() {
                    cancel = true;
                }
            });
        });
        // a click on one of last frame's suggestions takes that name into the expression
        let expr_id = egui::Id::new("new_variable_expr");
        if let Some((_, name)) = nv.input.click(ctx) {
            nv.expr = vars::apply_completion(&nv.expr, &name);
        }
        let (enter, esc, tab) = ctx.input(|i| (i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Escape), i.key_pressed(egui::Key::Tab)));
        // Tab from the name into the expression: its text is selected, so typing replaces it
        if tab && nv.name_had_focus {
            nv.input.select = Some(1);
        }
        nv.name_had_focus = ctx.memory(|m| m.has_focus(egui::Id::new("new_variable_name")));
        nv.input.cursor(ctx, 0, egui::Id::new("new_variable_name"), &nv.name);
        nv.input.cursor(ctx, 1, expr_id, &nv.expr);
        // suggestions under the expression (Enter takes the cursor out of a one-line field in the
        // same frame, so that counts as having it)
        let focused = ctx.memory(|m| m.has_focus(expr_id)) || expr_lost_focus;
        let names = self.var_names();
        let value = |n: &str| self.vars_resolved.value(n).ok();
        let mut keys = Keys::read(ctx, tab);
        let suggested = nv.input.suggest(ctx, 1, expr_id.with("list"), expr_rect, &mut nv.expr, &names, &value, &mut keys, focused, 4.0);
        let (listing, taken) = (suggested.listing, suggested.taken);
        if esc {
            if listing {
                // Esc closes the list first (by leaving the field), a second Esc cancels
                ctx.memory_mut(|m| m.surrender_focus(expr_id));
            } else {
                cancel = true;
            }
        } else if enter && valid && !taken {
            ok = true;
        }
        if ok && valid {
            self.add_variable(&nv.name, &nv.expr, &nv.comment);
            self.refresh_vars();
        } else if !cancel {
            self.new_var = Some(nv);
        }
    }
}
