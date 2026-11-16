//! The window every export of the toolpaths goes through.

use crate::app::{App, Tip};
use crate::theme::{ink, muted};
use egui::{Align2, Vec2};
use crate::app::layout::combo_with_manage;

impl App {
    /// The window every export of the toolpaths goes through (the menu and the disk icon): which
    /// toolpaths, which post-processor, then the file.
    pub(in crate::app) fn export_dialog_ui(&mut self, ctx: &egui::Context) {
        if !self.export_dialog {
            return;
        }
        use crate::app::OutputScope;
        let (mut go, mut cancel, mut manage) = (false, false, false);
        let indices = self.output_indices();
        let count = indices.len();
        let scope_matters = self.output_scope_matters();
        // with more than one tool the window also asks for one file or one for each tool
        let tool_lines: Vec<String> = if self.export_choice(&indices) {
            self.tool_groups(&indices)
                .iter()
                .enumerate()
                .map(|(n, group)| {
                    let tool = &self.toolpaths[indices[group[0]]].tool;
                    let what = if group.len() == 1 { t!("1 Werkzeugweg") } else { t!("%{n} Werkzeugwege", n = group.len()) };
                    format!("{}.  {} – {what}", n + 1, crate::tools::shown_name(&tool.name))
                })
                .collect()
        } else {
            vec![]
        };
        super::window("export_dialog_win", ctx)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(460.0);
                if super::header(ui, &t!("Werkzeugpfade exportieren")) {
                    cancel = true;
                }
                // the groups one under the other: which toolpaths (only when some are hidden, else
                // "all" and "visible" are the same), the format, and (only when the toolpaths need
                // more than one tool) how the tools are split into files
                if scope_matters {
                    crate::app::dialog_group(ui, &t!("Werkzeugpfade"), |ui| {
                        for (scope, name) in [(OutputScope::All, t!("Alle Werkzeugwege")), (OutputScope::Visible, t!("Sichtbare Werkzeugwege"))] {
                            ui.radio_value(&mut self.output_scope, scope, name);
                        }
                        ui.label(egui::RichText::new(t!("%{n} Werkzeugwege", n = count)).color(muted()));
                    });
                }
                crate::app::dialog_group(ui, &t!("Exportformat"), |ui| {
                    let selected_label = self.pp().label();
                    let post_idx = &mut self.post_idx;
                    let postprocs = &self.postprocs;
                    manage = combo_with_manage(ui, "post", 260.0, &selected_label, Some(&Tip::new(t!("Postprozessor"), t!("Übersetzt die Werkzeugwege in die G-Code-Sprache deiner Maschine. Verwalten… legt eigene an oder ändert sie."))), |ui| {
                        for (i, pp) in postprocs.iter().enumerate() {
                            if ui.selectable_label(*post_idx == i, pp.label()).clicked() {
                                *post_idx = i;
                            }
                        }
                    });
                    if !scope_matters {
                        ui.label(egui::RichText::new(t!("%{n} Werkzeugwege", n = count)).color(muted()));
                    }
                });
                if !tool_lines.is_empty() {
                    crate::app::dialog_group(ui, &t!("Werkzeugwechsel"), |ui| {
                        ui.label(egui::RichText::new(t!("Die Werkzeugwege brauchen %{n} Werkzeuge nacheinander:", n = tool_lines.len())).color(ink()));
                        for line in &tool_lines {
                            ui.label(egui::RichText::new(line).color(muted()));
                        }
                        ui.add_space(4.0);
                        // several tools in one file only with a post-processor that can change tools
                        let one_file_possible = self.pp().tool_change;
                        let per_tool = self.export_per_tool || !one_file_possible;
                        if ui.add_enabled(one_file_possible, egui::RadioButton::new(!per_tool, t!("Alles in eine Datei"))).on_disabled_hover_text(t!("Die gewählte Ausgabe unterstützt keinen Werkzeugwechsel: bitte in separate Dateien speichern.")).clicked() {
                            self.export_per_tool = false;
                        }
                        if ui.add(egui::RadioButton::new(per_tool, t!("Je Werkzeug eine Datei (zusammen in einer ZIP-Datei)"))).clicked() {
                            self.export_per_tool = true;
                        }
                        if !one_file_possible {
                            ui.label(egui::RichText::new(t!("Die gewählte Ausgabe unterstützt keinen Werkzeugwechsel: bitte in separate Dateien speichern.")).color(muted()));
                        }
                    });
                }
                crate::app::dialog_buttons(ui, |ui| {
                    let export = egui::Button::new(egui::RichText::new(t!("Exportieren…")).strong().color(crate::app::INK)).fill(crate::app::ACCENT).min_size(Vec2::new(120.0, 30.0)).rounding(4.0);
                    if ui.add_enabled(count > 0, export).clicked() {
                        go = true;
                    }
                    if ui.add(egui::Button::new(t!("Abbrechen")).min_size(Vec2::new(90.0, 30.0)).rounding(4.0)).clicked() {
                        cancel = true;
                    }
                });
            });
        if manage {
            let idx = self.post_idx.min(self.postprocs.len().saturating_sub(1));
            self.pp_dialog = Some(crate::app::PpDialog { editing: self.postprocs[idx].clone(), index: Some(idx), hook_tab: 0 });
        }
        if go {
            self.export_dialog = false;
            self.export_gcode();
        } else if cancel || (ctx.input(|i| i.key_pressed(egui::Key::Escape)) && self.pp_dialog.is_none()) {
            self.export_dialog = false;
        }
    }
}
