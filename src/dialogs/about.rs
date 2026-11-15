//! The window "Über CncCraft": version, what the program is, where it lives.

use crate::app::*;

impl App {
    pub(in crate::app) fn about_ui(&mut self, ctx: &egui::Context) {
        if !self.about {
            return;
        }
        dialogs::window("Über CncCraft", ctx).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
            if dialogs::header(ui, &t!("Über CncCraft")) {
                self.about = false;
            }
            ui.label(egui::RichText::new(t!("CncCraft %{version}", version = env!("CARGO_PKG_VERSION"))).strong());
            ui.label(t!("CncCraft – Entwurf und Werkzeugwege für CNC-Fräsen."));
            ui.hyperlink_to("https://github.com/freegroup/CncCraft", "https://github.com/freegroup/CncCraft");
            ui.label(t!("Werkzeugdaten und Voreinstellungen nach dem Vorbild von Vectric Spark."));
            dialog_buttons(ui, |ui| {
                if ui.button(t!("Schließen")).clicked() {
                    self.about = false;
                }
            });
        });
    }
}
