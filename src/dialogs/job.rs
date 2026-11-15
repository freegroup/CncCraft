//! The window that creates or sets up the job: size, material, zero point, travel heights.

use crate::app::*;

impl App {
    pub(in crate::app) fn job_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some((job, is_new)) = &mut self.job_dialog else { return };
        let mut ok = false;
        let mut cancel = false;
        let title = if *is_new { t!("Job erstellen") } else { t!("Job einrichten") };
        dialogs::window("job_dialog_win", ctx)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(360.0);
                if dialogs::header(ui, &title) {
                    cancel = true;
                }
                let f = if job.inches { 1.0 / 25.4 } else { 1.0 };
                let unit = if job.inches { "in" } else { "mm" };

                dialog_group(ui, &t!("Auftragsgröße"), |ui| {
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
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add_sized(Vec2::new(78.0, 0.0), egui::Label::new(egui::RichText::new(t!("Material")).color(ink())));
                        egui::ComboBox::from_id_salt("mat").width(150.0).selected_text(material_label(&job.material)).show_ui(ui, |ui| {
                            for m in MATERIALS {
                                ui.selectable_value(&mut job.material, m.to_string(), material_label(m));
                            }
                        });
                    });
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

                dialog_buttons(ui, |ui| {
                    let primary = if *is_new { t!("Erstellen") } else { t!("Übernehmen") };
                    if ui.add(egui::Button::new(egui::RichText::new(primary).strong().color(INK)).fill(ACCENT).min_size(Vec2::new(100.0, 30.0)).rounding(4.0)).clicked() {
                        ok = true;
                    }
                    ui.add_space(4.0);
                    if ui.add(egui::Button::new(t!("Abbrechen")).min_size(Vec2::new(100.0, 30.0)).rounding(4.0)).clicked() {
                        cancel = true;
                    }
                });
            });
        if ok {
            let (job, is_new) = self.job_dialog.take().unwrap();
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
