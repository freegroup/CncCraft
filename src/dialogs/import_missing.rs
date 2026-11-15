//! The friendly window shown when the original file of an import is not there any more.

use crate::app::*;
use crate::app::importing::file_dialog;
use std::path::Path;

impl App {
    /// The friendly window shown when the original file of an import is not there any more.
    pub(in crate::app) fn import_missing_ui(&mut self, ctx: &egui::Context) {
        let Some(file) = self.import_missing.clone() else { return };
        let name = Path::new(&file).file_name().map_or_else(|| file.clone(), |n| n.to_string_lossy().into_owned());
        let (mut search, mut close) = (false, false);
        dialogs::window("Datei nicht gefunden", ctx).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
            ui.set_width(380.0);
            if dialogs::header(ui, &t!("Datei nicht gefunden")) {
                close = true;
            }
            ui.label(egui::RichText::new(t!("Die Originaldatei „%{name}“ wurde leider nicht gefunden.", name = name)).strong().color(ink()));
            ui.add_space(4.0);
            ui.label(t!("Sie wurde vielleicht verschoben, umbenannt oder gelöscht. Die importierten Formen bleiben unverändert in der Zeichnung."));
            ui.add_space(4.0);
            ui.label(egui::RichText::new(&file).small().color(muted()));
            dialog_buttons(ui, |ui| {
                if ui.button(t!("Datei suchen…")).clicked() {
                    search = true;
                }
                if ui.button(t!("Schließen")).clicked() {
                    close = true;
                }
            });
        });
        if close {
            self.import_missing = None;
        }
        if search {
            if let Some(found) = file_dialog().pick_file() {
                self.reload_import(&file, Some(found));
            }
        }
    }
}
