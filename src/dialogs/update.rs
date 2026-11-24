//! The "Check for updates" status dialog: a focused card, opened straight from the Help menu, that
//! checks for a newer release at once and shows the result the way the other Craft apps do — the
//! program's icon and name at the top, the running version below it, a plain status line, and a
//! single button to close (or, when a newer release exists, to open its download page).
//!
//! The check itself lives in [`crate::app::update`]; this file only draws its current state.

use crate::app::*;
use crate::app::update::Status;

impl App {
    pub(in crate::app) fn update_ui(&mut self, ctx: &egui::Context) {
        if !self.update_dialog {
            return;
        }
        let mut close = false;
        let mut open: Option<String> = None;
        dialogs::window("update_status", ctx)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(380.0);
                // the heading: icon on the left, name + author on the first line, version below
                let icon = dialogs::app_icon(ctx);
                ui.horizontal(|ui| {
                    ui.add(egui::Image::new((icon.id(), Vec2::splat(40.0))));
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("CncCraft").size(18.0).strong().color(INK));
                            ui.label(egui::RichText::new(t!("von freegroup")).color(muted()));
                        });
                        ui.monospace(egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).color(muted()));
                    });
                    // the close cross sits at the top-right, like the other dialogs
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if dialogs::close_button(ui).on_hover_text(t!("Schließen")).clicked() {
                            close = true;
                        }
                    });
                });
                ui.add_space(10.0);
                // the status line: the same words the other Craft apps use
                match &self.update.status {
                    Status::Idle | Status::Checking => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(t!("Suche nach Updates …"));
                        });
                    }
                    Status::UpToDate => {
                        ui.label(t!("CncCraft %{v} ist die neueste verfügbare Version.", v = env!("CARGO_PKG_VERSION")));
                    }
                    Status::Newer { version, url } => {
                        ui.label(egui::RichText::new(t!("Neue Version verfügbar: CncCraft %{v}.", v = version.clone())).strong().color(ACCENT_DEEP));
                        let _ = url;
                    }
                    Status::Failed(why) => {
                        ui.label(egui::RichText::new(t!("Update-Prüfung fehlgeschlagen: %{why}", why = t!(why.as_str()).to_string())).color(muted()));
                    }
                }
                dialog_buttons(ui, |ui| {
                    // a newer release offers the download first; otherwise just OK
                    if let Status::Newer { url, .. } = &self.update.status {
                        let button = egui::Button::new(egui::RichText::new(t!("Herunterladen")).color(INK)).fill(ACCENT);
                        if ui.add(button).clicked() {
                            open = Some(url.clone());
                        }
                        if ui.button(t!("Schließen")).clicked() {
                            close = true;
                        }
                    } else {
                        let button = egui::Button::new(egui::RichText::new(t!("OK")).color(INK)).fill(ACCENT);
                        if ui.add(button).clicked() {
                            close = true;
                        }
                    }
                });
            });
        if close {
            self.update_dialog = false;
        }
        if let Some(url) = open {
            ctx.open_url(egui::OpenUrl::new_tab(url));
        }
    }
}
