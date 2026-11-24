//! The window "Über CncCraft": version, what the program is, where it lives, and whether a newer
//! release is available.

use crate::app::*;
use crate::app::update::Status;

impl App {
    pub(in crate::app) fn about_ui(&mut self, ctx: &egui::Context) {
        if !self.about {
            return;
        }
        let mut check = false;
        let mut open: Option<String> = None;
        dialogs::window("Über CncCraft", ctx).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
            if dialogs::header(ui, &t!("Über CncCraft")) {
                self.about = false;
            }
            ui.label(egui::RichText::new(t!("CncCraft %{version}", version = env!("CARGO_PKG_VERSION"))).strong());
            ui.label(t!("CncCraft – Entwurf und Werkzeugwege für CNC-Fräsen."));
            ui.hyperlink_to("https://github.com/freegroup/CncCraft", "https://github.com/freegroup/CncCraft");
            ui.label(t!("Werkzeugdaten und Voreinstellungen nach dem Vorbild von Vectric Spark."));
            ui.separator();
            // the update line: state of the check and, if newer, a download link
            match &self.update.status {
                Status::Idle => { check = ui.button(t!("Nach Updates suchen")).clicked(); }
                Status::Checking => { ui.horizontal(|ui| { ui.spinner(); ui.label(t!("Suche nach Updates …")); }); }
                Status::UpToDate => { ui.label(egui::RichText::new(t!("Sie haben die neueste Version.")).color(muted())); }
                Status::Failed(why) => {
                    ui.label(egui::RichText::new(t!("Update-Prüfung fehlgeschlagen: %{why}", why = t!(why.as_str()).to_string())).color(muted()));
                    check = ui.button(t!("Erneut suchen")).clicked();
                }
                Status::Newer { version, url } => {
                    ui.label(egui::RichText::new(t!("Neue Version verfügbar: %{v}", v = version.clone())).strong().color(crate::theme::ACCENT_DEEP));
                    let button = egui::Button::new(egui::RichText::new(t!("Herunterladen")).color(INK)).fill(ACCENT);
                    if ui.add(button).clicked() {
                        open = Some(url.clone());
                    }
                }
            }
            dialog_buttons(ui, |ui| {
                if ui.button(t!("Schließen")).clicked() {
                    self.about = false;
                }
            });
        });
        if check {
            self.update.check();
        }
        if let Some(url) = open {
            ctx.open_url(egui::OpenUrl::new_tab(url));
        }
    }
}
