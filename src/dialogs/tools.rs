//! The tool manager: the list of tools and the editor of one tool (all tools can be changed).

use crate::app::App;
use crate::theme::muted;
use egui::Vec2;

impl App {
    pub(in crate::app) fn tool_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(dlg) = &mut self.tool_dialog else { return };
        let mut save = false;
        let mut delete = false;
        let mut pick: Option<usize> = None;
        let mut close = false;

        let mat = dlg.material.clone();
        let mut open = true;
        let screen = ctx.screen_rect();
        super::window("tool_dialog_win", ctx)
            .order(egui::Order::Foreground)
            .resizable(true)
            .default_width(520.0)
            .min_width(380.0)
            .default_height((screen.height() - 120.0).clamp(300.0, 520.0))
            .max_height(screen.height() - 60.0)
            .show(ctx, |ui| {
                if super::header(ui, &t!("Werkzeug-Verwaltung"), true) {
                    open = false;
                }
                egui::TopBottomPanel::bottom("tool_buttons").frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 0.0, top: 6.0, bottom: 4.0 })).show_separator_line(true).show_inside(ui, |ui| {
                    ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), 30.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(egui::RichText::new(t!("Speichern")).strong().color(crate::app::INK)).fill(crate::app::ACCENT).min_size(Vec2::new(100.0, 30.0)).rounding(4.0)).clicked() {
                            save = true;
                        }
                        ui.add_space(4.0);
                        if ui.add(egui::Button::new(t!("Abbrechen")).min_size(Vec2::new(90.0, 30.0)).rounding(4.0)).clicked() {
                            close = true;
                        }
                    });
                });

                egui::SidePanel::left("tool_list_panel").exact_width(160.0).resizable(false).frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 4.0, top: 0.0, bottom: 0.0 })).show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(t!("Werkzeuge")).strong());
                        ui.label(egui::RichText::new(format!("({})", crate::tools::material_label(&mat))).small().color(muted()));
                    });
                    egui::TopBottomPanel::bottom("tool_list_buttons").frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 0.0, top: 4.0, bottom: 0.0 })).show_separator_line(false).show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.small_button("+").on_hover_text(t!("Neues Werkzeug anlegen")).clicked() {
                                pick = Some(usize::MAX);
                            }
                            let can_delete = dlg.index.is_some();
                            if ui.add_enabled(can_delete, egui::Button::new("−").small()).on_hover_text(t!("Ausgewähltes Werkzeug löschen")).clicked() {
                                delete = true;
                            }
                        });
                    });
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        for (i, entry) in dlg.entries.iter().enumerate() {
                            if entry.material != mat {
                                continue;
                            }
                            let is_selected = dlg.index == Some(i);
                            let label = &crate::tools::shown_name(if is_selected { &dlg.editing.name } else { &entry.tool.name });
                            let orig_tool = dlg.original.get(i).map(|e| &e.tool);
                            let changed = if is_selected {
                                orig_tool.map_or(true, |b| dlg.editing != *b)
                            } else {
                                orig_tool.map_or(true, |b| entry.tool != *b)
                            };
                            let text = if changed {
                                egui::RichText::new(label).color(crate::app::ACCENT)
                            } else {
                                egui::RichText::new(label)
                            };
                            if ui.selectable_label(is_selected, text).clicked() {
                                pick = Some(i);
                            }
                        }
                    });
                });

                egui::Frame::none().inner_margin(egui::Margin { left: 10.0, right: 0.0, top: 0.0, bottom: 0.0 }).show(ui, |ui| {
                    Self::tool_editor(ui, dlg);
                });
            });

        if let Some(p) = pick {
            // write current edits back to working copy before switching
            if let Some(dlg) = &mut self.tool_dialog {
                if let Some(idx) = dlg.index {
                    if idx < dlg.entries.len() {
                        dlg.entries[idx].tool = dlg.editing.clone();
                    }
                } else {
                    dlg.entries.push(crate::tools::ToolEntry { material: dlg.material.clone(), tool: dlg.editing.clone() });
                }
            }
            if p == usize::MAX {
                let new_tool = crate::tools::ToolLibrary::default_tool(crate::tools::ToolKind::EndMill);
                if let Some(dlg) = &mut self.tool_dialog {
                    dlg.editing = new_tool;
                    dlg.index = None;
                }
            } else if let Some(dlg) = &mut self.tool_dialog {
                if p < dlg.entries.len() {
                    dlg.editing = dlg.entries[p].tool.clone();
                    dlg.index = Some(p);
                }
            }
            return;
        }

        if save {
            // write current edits to working copy, then propagate to lib
            if let Some(mut dlg) = self.tool_dialog.take() {
                if let Some(idx) = dlg.index {
                    if idx < dlg.entries.len() {
                        dlg.entries[idx].tool = dlg.editing;
                    }
                } else {
                    dlg.entries.push(crate::tools::ToolEntry { material: dlg.material, tool: dlg.editing });
                }
                self.lib.entries = dlg.entries;
                self.status = "Werkzeuge gespeichert".into();
            }
            return;
        }

        if delete {
            if let Some(dlg) = &mut self.tool_dialog {
                if let Some(idx) = dlg.index {
                    if idx < dlg.entries.len() {
                        dlg.entries.remove(idx);
                        self.status = "Werkzeug gelöscht".into();
                    }
                }
                let mat_clone = dlg.material.clone();
                let first = dlg.entries.iter().position(|e| e.material == mat_clone);
                match first {
                    Some(i) => {
                        dlg.editing = dlg.entries[i].tool.clone();
                        dlg.index = Some(i);
                    }
                    None => {
                        dlg.editing = crate::tools::ToolLibrary::default_tool(crate::tools::ToolKind::EndMill);
                        dlg.index = None;
                    }
                }
            }
            return;
        }

        if close || !open {
            // cancel: discard working copy, lib stays unchanged
            self.tool_dialog = None;
        }
    }

    fn tool_editor(ui: &mut egui::Ui, dlg: &mut crate::app::ToolDialog) {
        let t = &mut dlg.editing;
        ui.label(egui::RichText::new(t!("Name")).small().strong());
        ui.text_edit_singleline(&mut t.name);
        ui.add_space(4.0);
        egui::Grid::new("tool_edit_grid").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
            ui.label(t!("Typ"));
            egui::ComboBox::from_id_salt("tool_kind").width(140.0).selected_text(t.kind.group()).show_ui(ui, |ui| {
                for &k in crate::tools::ToolKind::all() {
                    if ui.selectable_label(t.kind == k, k.group()).clicked() {
                        t.kind = k;
                    }
                }
            });
            ui.end_row();
            ui.label(t!("Durchmesser"));
            ui.add(egui::DragValue::new(&mut t.diameter).speed(0.1).range(0.1..=100.0).max_decimals(2).suffix(" mm"));
            ui.end_row();
            if t.kind == crate::tools::ToolKind::VBit || t.kind == crate::tools::ToolKind::Drill {
                ui.label(t!("Winkel"));
                ui.add(egui::DragValue::new(&mut t.angle).speed(1.0).range(1.0..=180.0).suffix(" °"));
                ui.end_row();
            }
            ui.label(t!("Drehzahl"));
            ui.add(egui::DragValue::new(&mut t.rpm).speed(100).range(0..=60000).suffix(t!(" U/min")));
            ui.end_row();
            ui.label(t!("Vorschub"));
            ui.add(egui::DragValue::new(&mut t.feed).speed(10).range(1.0..=20000.0).suffix(" mm/min"));
            ui.end_row();
            ui.label(t!("Eintauchen"));
            ui.add(egui::DragValue::new(&mut t.plunge).speed(10).range(1.0..=20000.0).suffix(" mm/min"));
            ui.end_row();
            ui.label(t!("Zustellung"));
            ui.add(egui::DragValue::new(&mut t.stepdown).speed(0.05).range(0.01..=50.0).max_decimals(3).suffix(" mm"));
            ui.end_row();
            ui.label(t!("Querzustellung"));
            let pct = if t.diameter > 0.0 { t.stepover / t.diameter * 100.0 } else { 0.0 };
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut t.stepover).speed(0.05).range(0.01..=50.0).max_decimals(3).suffix(" mm"));
                ui.label(egui::RichText::new(format!("{pct:.0} %")).small().color(muted()));
            });
            ui.end_row();
        });
    }
}
