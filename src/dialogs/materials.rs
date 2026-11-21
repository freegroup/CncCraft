//! The material manager: the list of materials and the editor of one. Every material can be
//! changed; the shipped ones can't be deleted, nor the one the job is cut from. A new material
//! starts as a copy of the one selected, with that one's tools (held within its own limits).

use crate::app::*;
use crate::materials::{looks, Material, MaterialLibrary};
use crate::widgets::material_swatch;

/// Width of the field labels in the editor.
const LABELS: f32 = 140.0;

impl App {
    /// Open the manager on the material `key` (the first one when no material has that key).
    pub(in crate::app) fn open_material_dialog(&mut self, key: &str) {
        let mut list = self.materials.list.clone();
        MaterialLibrary::sort(&mut list);
        let index = list.iter().position(|m| m.key == key).unwrap_or(0);
        self.material_dialog = Some(MaterialDialog { list, index, added: vec![], removed: vec![] });
    }

    pub(in crate::app) fn material_dialog_ui(&mut self, ctx: &egui::Context) {
        // the job's material, and the one being chosen for a job, can't be deleted
        let in_use: Vec<String> = std::iter::once(self.job.material.clone()).chain(self.job_dialog.as_ref().map(|(j, _)| j.material.clone())).collect();
        let Some(dlg) = &mut self.material_dialog else { return };
        let (mut save, mut close, mut add, mut delete) = (false, false, false, false);
        let mut pick: Option<usize> = None;
        let selected = &dlg.list[dlg.index];
        let can_delete = !selected.builtin && !in_use.contains(&selected.key);
        let delete_reason = if selected.builtin { t!("Mitgelieferte Materialien können nicht gelöscht werden.") } else { t!("Das Werkstück wird aus diesem Material gefräst.") };
        let names_ok = dlg.list.iter().all(|m| !m.name.trim().is_empty());
        let screen = ctx.screen_rect();
        let shown = super::window("material_dialog_win", ctx)
            .resizable(true)
            .default_width(680.0)
            .min_width(560.0)
            .default_height((screen.height() - 120.0).clamp(360.0, 620.0))
            .max_height(screen.height() - 60.0)
            .show(ctx, |ui| {
                if super::header(ui, &t!("Material-Verwaltung")) {
                    close = true;
                }
                // the buttons first, so they stay visible however tall the editor gets
                egui::TopBottomPanel::bottom("material_buttons").frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 0.0, top: 6.0, bottom: 4.0 })).show_separator_line(true).show_inside(ui, |ui| {
                    ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), 30.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let primary = egui::Button::new(egui::RichText::new(t!("Speichern")).strong().color(INK)).fill(ACCENT).min_size(Vec2::new(100.0, 30.0)).rounding(4.0);
                        let tip = Tip::new(t!("Speichern"), t!("Übernimmt alle Änderungen an den Materialien.")).when_disabled(t!("Jedes Material braucht einen Namen."));
                        if ui.add_enabled(names_ok, primary).tip(&tip).clicked() {
                            save = true;
                        }
                        ui.add_space(4.0);
                        if ui.add(egui::Button::new(t!("Abbrechen")).min_size(Vec2::new(90.0, 30.0)).rounding(4.0)).clicked() {
                            close = true;
                        }
                    });
                });

                egui::SidePanel::left("material_list_panel").exact_width(190.0).resizable(false).frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 6.0, top: 0.0, bottom: 0.0 })).show_inside(ui, |ui| {
                    ui.label(egui::RichText::new(t!("Materialien")).strong());
                    egui::TopBottomPanel::bottom("material_list_buttons").frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 0.0, top: 4.0, bottom: 0.0 })).show_separator_line(false).show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.small_button("+").tip(&Tip::new(t!("Neues Material"), t!("Legt eine Kopie des ausgewählten Materials an, mit dessen Werkzeugen."))).clicked() {
                                add = true;
                            }
                            let tip = Tip::new(t!("Material löschen"), t!("Löscht das ausgewählte Material und seine Werkzeuge.")).when_disabled(delete_reason.clone());
                            if ui.add_enabled(can_delete, egui::Button::new("−").small()).tip(&tip).clicked() {
                                delete = true;
                            }
                        });
                    });
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        for (i, m) in dlg.list.iter().enumerate() {
                            ui.horizontal(|ui| {
                                material_swatch(ui, &m.look, 18.0);
                                if ui.selectable_label(dlg.index == i, m.shown_name()).clicked() {
                                    pick = Some(i);
                                }
                            });
                        }
                    });
                });

                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    material_editor(ui, &mut dlg.list[dlg.index]);
                });
            });
        if let Some(shown) = shown {
            // it opens over the job's window: keep it in front
            ctx.move_to_top(shown.response.layer_id);
        }

        if let Some(i) = pick {
            dlg.index = i;
        }
        if add {
            let source = dlg.list[dlg.index].clone();
            let key = MaterialLibrary::new_key(&dlg.list);
            let copy = Material {
                key: key.clone(),
                name: t!("Neues Material"),
                description: source.shown_description(),
                cutter: source.shown_cutter(),
                coolant: source.shown_coolant(),
                builtin: false,
                ..source.clone()
            };
            dlg.list.push(copy);
            MaterialLibrary::sort(&mut dlg.list);
            dlg.index = dlg.list.iter().position(|m| m.key == key).unwrap_or(0);
            dlg.added.push((key, source.key));
        }
        if delete && can_delete {
            let gone = dlg.list.remove(dlg.index);
            dlg.removed.push(gone.key);
            dlg.index = dlg.index.min(dlg.list.len() - 1);
        }
        if save {
            self.save_material_dialog();
        } else if close {
            self.material_dialog = None;
        }
    }

    /// Take the manager's list: new materials get the tools of the one they were copied from,
    /// deleted ones lose theirs; a job still without a material gets the one selected.
    fn save_material_dialog(&mut self) {
        let Some(dlg) = self.material_dialog.take() else { return };
        let chosen = dlg.list[dlg.index].key.clone();
        self.materials.list = dlg.list;
        for (key, from) in &dlg.added {
            if let Some(m) = self.materials.get(key) {
                self.lib.add_material(from, m);
            }
        }
        for key in &dlg.removed {
            self.lib.remove_material(key);
        }
        if let Some((job, _)) = &mut self.job_dialog {
            if self.materials.get(&job.material).is_none() {
                job.material = chosen;
            }
        }
        self.status = t!("Materialien gespeichert");
    }
}

/// The fields of one material, in three groups.
fn material_editor(ui: &mut egui::Ui, m: &mut Material) {
    let builtin = m.builtin;
    dialog_group(ui, &t!("Allgemein"), |ui| {
        egui::Grid::new("material_general").num_columns(2).spacing([10.0, 6.0]).min_col_width(LABELS).show(ui, |ui| {
            ui.label(t!("Name"));
            text_field(ui, &mut m.name, builtin, false);
            ui.end_row();
            ui.label(t!("Beschreibung"));
            text_field(ui, &mut m.description, builtin, true);
            ui.end_row();
            let tip = Tip::new(t!("Aussehen"), t!("So sieht das Werkstück in der 3D-Vorschau aus."));
            ui.label(t!("Aussehen")).tip(&tip);
            ui.horizontal(|ui| {
                material_swatch(ui, &m.look, 48.0);
                let all = looks();
                let current = all.iter().find(|(k, _)| *k == m.look).map_or_else(String::new, |(_, n)| n.clone());
                egui::ComboBox::from_id_salt("material_look").width(180.0).selected_text(current).show_ui(ui, |ui| {
                    for (key, name) in &all {
                        ui.horizontal(|ui| {
                            material_swatch(ui, key, 16.0);
                            ui.selectable_value(&mut m.look, key.clone(), name);
                        });
                    }
                })
                .response
                .tip(&tip);
            });
            ui.end_row();
        });
    });

    dialog_group(ui, &t!("Schnittwerte"), |ui| {
        egui::Grid::new("material_cutting").num_columns(2).spacing([10.0, 6.0]).min_col_width(LABELS).show(ui, |ui| {
            let tip = Tip::new(t!("Max. Vorschub"), t!("Schneller sollte der Fräser nicht durch das Material fahren. Werkzeugwege, die schneller sind, werden gewarnt."));
            ui.label(t!("Max. Vorschub")).tip(&tip);
            ui.add(egui::DragValue::new(&mut m.max_feed).speed(10).range(1.0..=20000.0).max_decimals(0).suffix(" mm/min")).tip(&tip);
            ui.end_row();

            let tip = Tip::new(t!("Drehzahl"), t!("Die empfohlene Drehzahl der Spindel für dieses Material."));
            ui.label(t!("Drehzahl")).tip(&tip);
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut m.rpm_min).speed(100).range(0..=60000)).tip(&tip);
                ui.label("–");
                ui.add(egui::DragValue::new(&mut m.rpm_max).speed(100).range(0..=60000).suffix(t!(" U/min"))).tip(&tip);
            });
            m.rpm_max = m.rpm_max.max(m.rpm_min);
            ui.end_row();

            let tip = Tip::new(
                t!("Max. Zustellung"),
                t!("So tief darf der Fräser je Durchgang höchstens schneiden, als Anteil seines Durchmessers. Eine Faustregel für Holz: höchstens 2/3 des Durchmessers."),
            );
            ui.label(t!("Max. Zustellung")).tip(&tip);
            ui.vertical(|ui| {
                let mut pct = m.depth_ratio * 100.0;
                if ui.add(egui::DragValue::new(&mut pct).speed(1).range(1.0..=200.0).max_decimals(0).suffix(t!(" % vom Fräser-Ø"))).tip(&tip).changed() {
                    m.depth_ratio = pct / 100.0;
                }
                ui.label(egui::RichText::new(t!("Bei einem 6-mm-Fräser: höchstens %{mm} mm je Durchgang", mm = format!("{:.1}", m.max_depth(6.0)))).small().color(muted()));
            });
            ui.end_row();
        });
    });

    dialog_group(ui, &t!("Empfehlungen"), |ui| {
        egui::Grid::new("material_advice").num_columns(2).spacing([10.0, 6.0]).min_col_width(LABELS).show(ui, |ui| {
            let tip = Tip::new(t!("Fräser"), t!("Welche Fräser passen: Anzahl der Schneiden, Werkstoff des Fräsers, Spiralrichtung."));
            ui.label(t!("Fräser")).tip(&tip);
            text_field(ui, &mut m.cutter, builtin, true);
            ui.end_row();
            let tip = Tip::new(t!("Kühlung"), t!("Ob und womit gekühlt oder geschmiert wird."));
            ui.label(t!("Kühlung")).tip(&tip);
            text_field(ui, &mut m.coolant, builtin, true);
            ui.end_row();
        });
    });
}

/// A text of a material: the shipped ones show their translation, and keep their original until it
/// is changed.
fn text_field(ui: &mut egui::Ui, value: &mut String, builtin: bool, multiline: bool) {
    let mut text = if builtin { t!(value.as_str()) } else { value.clone() };
    let edit = if multiline { egui::TextEdit::multiline(&mut text).desired_rows(2) } else { egui::TextEdit::singleline(&mut text) };
    if ui.add(edit.desired_width(f32::INFINITY)).changed() {
        *value = text;
    }
}
