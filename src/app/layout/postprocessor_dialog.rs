//! The post-processor manager: the list, and the texts a post-processor puts into the file.

use crate::app::App;
use crate::theme::muted;
use egui::Vec2;

impl App {
    pub(in crate::app) fn pp_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(dlg) = &mut self.pp_dialog else { return };
        let title = if dlg.index.is_some() { t!("Postprozessor-Verwaltung") } else { t!("Neuer Postprozessor") };
        let mut save = false;
        let mut delete = false;
        let mut pick: Option<usize> = None;
        let mut close = false;
        let is_builtin = dlg.editing.builtin;

        let mut open = true;
        let screen = ctx.screen_rect();
        egui::Window::new(title)
            .id(egui::Id::new("pp_dialog_win"))
            .open(&mut open)
            .order(egui::Order::Foreground)
            .resizable(true)
            .default_width(560.0)
            .min_width(400.0)
            .default_height((screen.height() - 120.0).clamp(300.0, 520.0))
            .max_height(screen.height() - 60.0)
            .show(ctx, |ui| {
                // Bottom buttons first so they are always visible
                egui::TopBottomPanel::bottom("pp_buttons").frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 0.0, top: 6.0, bottom: 4.0 })).show_separator_line(true).show_inside(ui, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(egui::RichText::new(t!("Speichern")).strong().color(crate::app::INK)).fill(crate::app::ACCENT).min_size(Vec2::new(100.0, 30.0)).rounding(4.0)).clicked() {
                            save = true;
                        }
                        ui.add_space(4.0);
                        if ui.add(egui::Button::new(t!("Abbrechen")).min_size(Vec2::new(90.0, 30.0)).rounding(4.0)).clicked() {
                            close = true;
                        }
                    });
                });

                // Left list panel
                egui::SidePanel::left("pp_list_panel").exact_width(150.0).resizable(false).frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 4.0, top: 0.0, bottom: 0.0 })).show_inside(ui, |ui| {
                    ui.label(egui::RichText::new(t!("Postprozessoren")).strong());
                    // +/- buttons pinned at the bottom
                    egui::TopBottomPanel::bottom("pp_list_buttons").frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 0.0, top: 4.0, bottom: 0.0 })).show_separator_line(false).show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.small_button("+").on_hover_text(t!("Neuen Postprozessor anlegen")).clicked() {
                                pick = Some(usize::MAX);
                            }
                            let can_delete = dlg.index.is_some() && !is_builtin;
                            if ui.add_enabled(can_delete, egui::Button::new("−").small()).on_hover_text(t!("Ausgewählten Postprozessor löschen")).clicked() {
                                delete = true;
                            }
                        });
                    });
                    // List fills the rest and scrolls
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        for (i, pp) in self.postprocs.iter().enumerate() {
                            let selected = dlg.index == Some(i);
                            if ui.selectable_label(selected, &pp.name).clicked() {
                                pick = Some(i);
                            }
                        }
                    });
                });

                // Right side: editor fills the rest
                Self::pp_editor(ui, dlg);
            });

        if let Some(p) = pick {
            if p == usize::MAX {
                self.pp_dialog = Some(crate::app::PpDialog {
                    editing: crate::gcode::PostProcessor {
                        name: t!("Neuer Postprozessor").into(),
                        inch: false, ext: "nc".into(), tool_change: true,
                        job_start: String::new(), job_end: String::new(),
                        on_tool_change: "G0 Z{safe_z}\nM5\nT{tool_number} M6\nM0 ; Werkzeugwechsel – weiter mit Cycle Start".into(),
                        before_path: String::new(), after_path: String::new(),
                        builtin: false,
                    },
                    index: None,
                    hook_tab: 0,
                });
                return;
            } else if p < self.postprocs.len() {
                if let Some(dlg) = &mut self.pp_dialog {
                    dlg.editing = self.postprocs[p].clone();
                    dlg.index = Some(p);
                }
            }
            return;
        }

        if save {
            if let Some(dlg) = self.pp_dialog.take() {
                if let Some(idx) = dlg.index {
                    if idx < self.postprocs.len() {
                        self.postprocs[idx] = dlg.editing;
                    }
                } else {
                    self.postprocs.push(dlg.editing);
                    self.post_idx = self.postprocs.len() - 1;
                }
                self.status = "Postprozessor gespeichert".into();
            }
            return;
        }

        if delete {
            if let Some(dlg) = self.pp_dialog.take() {
                if let Some(idx) = dlg.index {
                    if idx < self.postprocs.len() && !self.postprocs[idx].builtin {
                        self.postprocs.remove(idx);
                        if self.post_idx >= self.postprocs.len() {
                            self.post_idx = self.postprocs.len().saturating_sub(1);
                        }
                        self.status = "Postprozessor gelöscht".into();
                    }
                }
            }
            return;
        }

        if close {
            self.pp_dialog = None;
        }
    }

    fn pp_editor(ui: &mut egui::Ui, dlg: &mut crate::app::PpDialog) {
        let pp = &mut dlg.editing;
        ui.label(egui::RichText::new(t!("Name")).small().strong());
        ui.text_edit_singleline(&mut pp.name);
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(t!("Dateiendung:"));
            egui::TextEdit::singleline(&mut pp.ext).desired_width(60.0).show(ui);
        });
        ui.checkbox(&mut pp.tool_change, t!("Werkzeugwechsel unterstützt"));
        ui.add_space(6.0);

        const TABS: &[(crate::lang::Tr, crate::lang::Tr)] = &[
            (tr!("Job Start"), tr!("Wird einmal am Anfang eingefügt.\nPlatzhalter: {safe_z} {home_z} {tool_number} {rpm}")),
            (tr!("Vor Werkzeugweg"), tr!("Vor jedem Werkzeugweg (nach S/M3).")),
            (tr!("Werkzeugwechsel"), tr!("Zwischen zwei Werkzeugwegen mit unterschiedlichem Werkzeug.")),
            (tr!("Nach Werkzeugweg"), tr!("Am Ende jedes Werkzeugwegs.")),
            (tr!("Job Ende"), tr!("Wird einmal am Ende eingefügt.")),
        ];

        ui.horizontal(|ui| {
            let filled = [
                !pp.job_start.trim().is_empty(),
                !pp.before_path.trim().is_empty(),
                !pp.on_tool_change.trim().is_empty(),
                !pp.after_path.trim().is_empty(),
                !pp.job_end.trim().is_empty(),
            ];
            for (i, &(label, _)) in TABS.iter().enumerate() {
                let text = if filled[i] {
                    egui::RichText::new(label).color(crate::app::ACCENT)
                } else {
                    egui::RichText::new(label)
                };
                if ui.selectable_label(dlg.hook_tab == i, text).clicked() {
                    dlg.hook_tab = i;
                }
            }
        });
        ui.add_space(2.0);

        let (_, tip) = TABS[dlg.hook_tab];
        ui.label(egui::RichText::new(tip).small().color(muted()));

        let val = match dlg.hook_tab {
            0 => &mut pp.job_start,
            1 => &mut pp.before_path,
            2 => &mut pp.on_tool_change,
            3 => &mut pp.after_path,
            _ => &mut pp.job_end,
        };

        let remaining = (ui.available_height() - 24.0).max(80.0);
        let layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
            crate::app::gcode_highlight::layout_gcode(ui, text, wrap_width)
        };
        let output = egui::TextEdit::multiline(val)
            .font(egui::FontId::monospace(12.0))
            .desired_width(f32::INFINITY)
            .hint_text(t!("(leer = kein Einfügen)"))
            .layouter(&mut layouter.clone())
            .min_size(Vec2::new(0.0, remaining))
            .show(ui);
        crate::app::gcode_highlight::gcode_tooltips(val, &output.response, &output.galley, output.galley_pos);

        ui.add_space(2.0);
        ui.label(egui::RichText::new(t!("Platzhalter: {safe_z} {home_z} {tool_number} {rpm}")).small().color(muted()));
    }
}
