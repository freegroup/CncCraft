//! The window listing the variables: those from the job, then the person's own.

use crate::app::*;
use crate::app::variables::*;
use egui::{Id, RichText};
use shape_edit::VarEvent;

impl App {
    /// The variables dialog: the job's values on top, below them the person's own variables, one
    /// row each (name = expression, with the result next to it) and a button to add one.
    pub(in crate::app) fn variables_dialog(&mut self, ctx: &egui::Context) {
        if !self.vars_open {
            return;
        }
        let mut open = true;
        let mut edited = false;
        let mut close = false;
        let red = crate::theme::error();
        let chip = crate::theme::chip();
        crate::app::dialogs::window("variablen_dialog_v2", ctx).resizable(true).default_size([500.0, 470.0]).min_height(380.0).default_pos(egui::pos2(360.0, 110.0)).show(ctx, |ui| {
            if crate::app::dialogs::header(ui, &t!("Variablen")) {
                open = false;
            }
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);
            ui.label(RichText::new(t!("Benannte Werte für die Maßfelder, z. B. „hoehe“ oder „breite/2 - rand“. Ändert man hier einen Wert, passen sich alle Formen an, die ihn benutzen.")).color(muted()).size(12.5));

            dialog_group(ui, &t!("Aus dem Job"), |ui| {
                for (name, what) in FROM_JOB {
                    ui.horizontal(|ui| {
                        ui.allocate_ui_with_layout(egui::vec2(110.0, 22.0), egui::Layout::left_to_right(egui::Align::Center), |ui| ui.label(RichText::new(name).monospace().color(ink())));
                        ui.label(RichText::new(what.get()).color(muted()));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let value = self.vars_resolved.value(name).map_or_else(|_| "–".into(), fmt);
                            value_chip(ui, &value, chip, ink());
                        });
                    });
                }
            });

            dialog_group(ui, &t!("Eigene Variablen"), |ui| {
                if self.vars.list.is_empty() {
                    ui.label(RichText::new(t!("Noch keine Variable. Mit „Neue Variable“ legst du eine an.")).color(muted()));
                }
                // the names the document knows the variables by (reset after loading a file)
                if self.vars_committed.len() != self.vars.list.len() {
                    self.vars_committed = self.vars.list.iter().map(|v| v.name.clone()).collect();
                }
                let mut remove = None;
                for i in 0..self.vars.list.len() {
                    let dup = self.vars.list.iter().filter(|v| v.name == self.vars.list[i].name).count() > 1;
                    let problem = name_problem(&self.vars.list[i].name, dup);
                    let error = self.vars_resolved.error(&self.vars.list[i].name).map(|e| crate::lang::calc_message(&e));
                    let value = self.vars_resolved.value(&self.vars.list[i].name).map_or_else(|_| "–".into(), fmt);
                    let mut renamed = false;
                    let comment = self.vars.list[i].comment.clone();
                    let v = &mut self.vars.list[i];
                    ui.horizontal(|ui| {
                        // visible field outlines on the white card
                        ui.visuals_mut().extreme_bg_color = crate::theme::field_bg();
                        ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, crate::theme::field_stroke());
                        let expr_w = (ui.available_width() - 120.0 - 86.0 - 26.0 - 12.0 - 4.0 * ui.spacing().item_spacing.x).max(90.0);
                        let name_colour = if problem.is_some() { red } else { ink() };
                        let name = egui::TextEdit::singleline(&mut v.name).id(Id::new(("variable_name", i))).font(egui::TextStyle::Monospace).text_color(name_colour).hint_text(t!("name")).margin(egui::Margin::symmetric(8.0, 4.0));
                        let name_resp = ui.add_sized([120.0, 28.0], name);
                        renamed = name_resp.changed();
                        edited |= renamed;
                        if !comment.is_empty() {
                            name_resp.on_hover_text(comment.as_str());
                        }
                        ui.label(RichText::new("=").color(muted()));
                        let expr = egui::TextEdit::singleline(&mut v.expr).font(egui::TextStyle::Monospace).hint_text(t!("Zahl oder Formel")).margin(egui::Margin::symmetric(8.0, 4.0));
                        edited |= ui.add_sized([expr_w, 28.0], expr).changed();
                        let failed = problem.is_some() || error.is_some();
                        value_chip(ui, if failed { "–" } else { &value }, chip, if failed { red } else { ink() });
                        if glyph_button(ui, Glyph::Cross).on_hover_text(t!("Variable löschen")).clicked() {
                            remove = Some(i);
                        }
                    });
                    // a valid new name is passed on to everything that used the old one
                    if renamed {
                        let new = self.vars.list[i].name.clone();
                        let dup = self.vars.list.iter().filter(|v| v.name == new).count() > 1;
                        let old = self.vars_committed[i].clone();
                        if name_problem(&new, dup).is_none() && new != old {
                            self.var_events.push(VarEvent::Renamed { from: old, to: new.clone() });
                            self.vars_committed[i] = new;
                        }
                    }
                    // what is wrong, under the row it belongs to (the original's full sentence on hover)
                    match (problem, error) {
                        (Some((short, full)), _) => {
                            ui.label(RichText::new(short).color(red).size(12.0)).on_hover_text(full);
                        }
                        (None, Some(message)) => {
                            ui.label(RichText::new(message).color(red).size(12.0));
                        }
                        (None, None) => {}
                    }
                }
                if let Some(i) = remove {
                    self.vars.list.remove(i);
                    self.var_events.push(VarEvent::Removed(self.vars_committed.remove(i)));
                    edited = true;
                }
                ui.add_space(4.0);
                if add_button(ui, &t!("Neue Variable")).clicked() {
                    self.open_new_variable();
                }
            });
            // the original's dialog ends with a "Done" button (0x1001425d8): at the bottom edge
            // and the right, however tall the window is made
            ui.add_space((ui.available_height() - 30.0 - ui.spacing().item_spacing.y).max(8.0));
            ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), 30.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new(RichText::new(t!("Fertig")).color(ink())).min_size(egui::vec2(90.0, 30.0))).clicked() {
                    close = true;
                }
            });
        });
        if edited {
            self.vars_dirty = true;
            // react at once, so the table below already shows the new names and values
            self.refresh_vars();
        }
        if !open || close {
            self.vars_open = false;
        }
    }
}
