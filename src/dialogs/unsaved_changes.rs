//! The question "Änderungen speichern?" before changes would be lost.

use crate::app::*;

impl App {
    /// The question "Save the changes?" when closing or replacing a document with changes.
    pub(in crate::app) fn unsaved_ui(&mut self, ctx: &egui::Context) {
        let Some(what) = self.unsaved else { return };
        let mut answer: Option<u8> = None;
        dialogs::window("Änderungen speichern?", ctx)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(380.0);
                if dialogs::header(ui, &t!("Änderungen speichern?")) {
                    answer = Some(2);
                }
                let name = self.path.as_ref().and_then(|p| p.file_stem()).map_or_else(|| t!("Neu"), |s| s.to_string_lossy().into_owned());
                // the wording of the original's standard question (wxstd catalogue: "Do you want to save
                // changes to %s [before closing it]?")
                let ask = match what {
                    Pending::Close => t!("Möchten Sie die Änderungen vor dem Schließen nach „%{name}“ speichern?", name = name),
                    _ => t!("Möchten Sie die Änderungen nach „%{name}“ speichern?", name = name),
                };
                let then = match what {
                    Pending::Close => t!("Wenn Sie nicht speichern, gehen die Änderungen verloren."),
                    Pending::NewJob => t!("Ein neuer Job ersetzt die Zeichnung; ohne Speichern gehen die Änderungen verloren."),
                    Pending::Open => t!("Die andere Datei ersetzt die Zeichnung; ohne Speichern gehen die Änderungen verloren."),
                    Pending::Example => t!("Das Beispiel ersetzt die Zeichnung; ohne Speichern gehen die Änderungen verloren."),
                };
                ui.label(egui::RichText::new(ask).strong().color(ink()));
                ui.label(egui::RichText::new(then).color(muted()));
                dialog_buttons(ui, |ui| {
                    if ui.add(egui::Button::new(egui::RichText::new(t!("Speichern")).strong().color(INK)).fill(ACCENT).min_size(Vec2::new(100.0, 30.0)).rounding(4.0)).clicked() {
                        answer = Some(0);
                    }
                    if ui.add(egui::Button::new(t!("Nicht speichern")).min_size(Vec2::new(120.0, 30.0)).rounding(4.0)).clicked() {
                        answer = Some(1);
                    }
                    if ui.add(egui::Button::new(t!("Abbrechen")).min_size(Vec2::new(90.0, 30.0)).rounding(4.0)).clicked() {
                        answer = Some(2);
                    }
                });
            });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            answer = Some(2);
        }
        match answer {
            Some(0) => {
                self.save_project(false);
                // a cancelled save window leaves the question open
                if !self.is_dirty() {
                    self.unsaved = None;
                    if self.thumb_pending != 0 {
                        self.after_thumb = Some(what);
                    } else {
                        self.run_pending(Some(ctx), what);
                    }
                }
            }
            Some(1) => {
                self.unsaved = None;
                self.discard = true;
                self.run_pending(Some(ctx), what);
            }
            Some(_) => self.unsaved = None,
            None => {}
        }
    }
}
