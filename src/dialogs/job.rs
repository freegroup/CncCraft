//! The window that creates or sets up the job: size, material, zero point, travel heights.

use crate::app::*;
use crate::materials::Material;
use crate::widgets::{combo_with_manage, material_swatch};

impl App {
    pub(in crate::app) fn job_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some((job, is_new)) = &mut self.job_dialog else { return };
        let mut ok = false;
        let mut cancel = false;
        // "Verwalten…" was clicked: open the manager on this material
        let mut manage_materials: Option<String> = None;
        let title = if *is_new { t!("Werkstück erstellen") } else { t!("Werkstück einrichten") };
        dialogs::window("job_dialog_win", ctx)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(360.0);
                if dialogs::header(ui, &title) {
                    cancel = true;
                }
                let f = if job.inches { 1.0 / 25.4 } else { 1.0 };
                let unit = if job.inches { "in" } else { "mm" };

                dialog_group(ui, &t!("Größe"), |ui| {
                    // size fields on the left, a 3D block sketch on the right
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            for (label, v, max) in [(t!("Breite (X)"), &mut job.width, 5000.0), (t!("Länge (Y)"), &mut job.height, 5000.0), (t!("Dicke (Z)"), &mut job.thickness, 500.0)] {
                                dim_row(ui, &label, v, f, unit, max);
                            }
                        });
                        ui.add_space(10.0);
                        block_sketch(ui);
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_sized(Vec2::new(78.0, 0.0), egui::Label::new(egui::RichText::new(t!("Einheiten")).color(ink())));
                        segmented(ui, &mut job.inches, [(true, t!("Zoll")), (false, "mm".to_string())]);
                    });
                });

                dialog_group(ui, &t!("Material"), |ui| {
                    ui.horizontal(|ui| {
                        ui.add_sized(Vec2::new(78.0, 0.0), egui::Label::new(egui::RichText::new(t!("Material")).color(ink())));
                        let chosen = self.materials.get(&job.material);
                        let label = chosen.map_or_else(|| t!("Bitte wählen …"), |m| m.shown_name());
                        let tip = Tip::new(t!("Material"), t!("Woraus das Werkstück ist. Es bestimmt die Werkzeuge, die empfohlenen Schnittwerte und das Aussehen in der 3D-Vorschau. Verwalten… öffnet die Materialeinstellungen: dort lassen sich Materialien bearbeiten und eigene anlegen."));
                        if combo_with_manage(ui, "job_material", 220.0, &label, Some(&tip), |ui| {
                            for m in self.materials.sorted() {
                                ui.horizontal(|ui| {
                                    material_swatch(ui, &m.look, 16.0);
                                    ui.selectable_value(&mut job.material, m.key.clone(), m.shown_name());
                                });
                            }
                        }) {
                            manage_materials = Some(job.material.clone());
                        }
                    });
                    match self.materials.get(&job.material) {
                        // the details only on request: the box is enough to choose
                        Some(m) => {
                            ui.add_space(2.0);
                            egui::CollapsingHeader::new(egui::RichText::new(t!("Empfehlungen")).color(ink())).id_salt("job_material_advice_fold").default_open(false).show(ui, |ui| material_advice(ui, m));
                        }
                        None => {
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(t!("Bitte das Material wählen, aus dem das Werkstück gefräst wird.")).color(crate::theme::warning()));
                        }
                    }
                });

                dialog_group(ui, &t!("Nullpunkt"), |ui| {
                    ui.horizontal(|ui| {
                        ui.add_sized(Vec2::new(78.0, 0.0), egui::Label::new(egui::RichText::new(t!("Z-Null")).color(ink())));
                        let mut surf = job.z_zero == ZZero::MaterialSurface;
                        if segmented(ui, &mut surf, [(true, t!("Oberfläche")), (false, t!("Tisch"))]) {
                            job.z_zero = if surf { ZZero::MaterialSurface } else { ZZero::MachineBed };
                        }
                    });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add_sized(Vec2::new(78.0, 0.0), egui::Label::new(egui::RichText::new(t!("XY-Null")).color(ink())));
                        let mut ll = job.datum == Datum::LowerLeft;
                        if segmented(ui, &mut ll, [(true, t!("Ecke")), (false, t!("Mitte"))]) {
                            job.datum = if ll { Datum::LowerLeft } else { Datum::Center };
                        }
                    });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add_sized(Vec2::new(78.0, 0.0), egui::Label::new(egui::RichText::new(t!("Sicher Z")).color(ink())));
                        ui.add(egui::DragValue::new(&mut job.safe_z).speed(0.1).range(0.5..=100.0).suffix(" mm"));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(t!("Start Z")).color(ink()));
                        ui.add(egui::DragValue::new(&mut job.home_z).speed(0.1).range(0.5..=200.0).suffix(" mm"));
                    });
                });

                let has_material = self.materials.get(&job.material).is_some();
                dialog_buttons(ui, |ui| {
                    let primary = if *is_new { t!("Erstellen") } else { t!("Übernehmen") };
                    let button = egui::Button::new(egui::RichText::new(&primary).strong().color(INK)).fill(ACCENT).min_size(Vec2::new(100.0, 30.0)).rounding(4.0);
                    let tip = Tip::new(primary.clone(), t!("Richtet das Werkstück so ein.")).when_disabled(t!("Zuerst das Material wählen."));
                    if ui.add_enabled(has_material, button).tip(&tip).clicked() {
                        ok = true;
                    }
                    ui.add_space(4.0);
                    if ui.add(egui::Button::new(t!("Abbrechen")).min_size(Vec2::new(100.0, 30.0)).rounding(4.0)).clicked() {
                        cancel = true;
                    }
                });
            });
        if let Some(key) = manage_materials {
            self.open_material_dialog(&key);
        }
        if ok {
            let (job, is_new) = self.job_dialog.take().unwrap();
            // the choices made here are where the next new job starts
            self.last_job = Some(job.clone());
            if is_new {
                self.new_job(job);
            } else {
                self.job = job;
                self.fitted = false;
                self.recompute_all();
            }
        } else if cancel {
            self.job_dialog = None;
        }
    }
}

/// What the chosen material asks for: its look, description, cutting values, cutters and cooling.
fn material_advice(ui: &mut egui::Ui, m: &Material) {
    ui.add_space(2.0);
    ui.horizontal_top(|ui| {
        material_swatch(ui, &m.look, 44.0);
        ui.add_space(4.0);
        ui.vertical(|ui| {
            ui.add(egui::Label::new(egui::RichText::new(m.shown_description()).color(muted())).wrap());
        });
    });
    ui.add_space(4.0);
    egui::Grid::new("job_material_advice").num_columns(2).spacing([8.0, 3.0]).min_col_width(78.0).show(ui, |ui| {
        let row = |ui: &mut egui::Ui, label: String, value: String| {
            ui.label(egui::RichText::new(label).small().color(ink()));
            ui.add(egui::Label::new(egui::RichText::new(value).small().color(muted())).wrap());
            ui.end_row();
        };
        row(ui, t!("Drehzahl"), format!("{} – {}{}", m.rpm_min, m.rpm_max, t!(" U/min")));
        row(ui, t!("Max. Vorschub"), format!("{:.0} mm/min", m.max_feed));
        row(ui, t!("Max. Zustellung"), t!("%{pct} % des Fräser-Ø je Durchgang", pct = format!("{:.0}", m.depth_ratio * 100.0)));
        row(ui, t!("Fräser"), m.shown_cutter());
        row(ui, t!("Kühlung"), m.shown_coolant());
    });
}
