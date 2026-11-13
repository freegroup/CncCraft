//! The program's menus: Datei, Bearbeiten, Ansicht, Werkzeugwege and Hilfe, each at the top
//! level of the menu bar in the title bar (see `layout::title_bar`). Every entry with keys
//! shows them from the shortcut table.

use super::shortcuts::{self, Action};
use super::{App, DrawTool, SNAP_RADIUS_MAX_PX, SNAP_RADIUS_MIN_PX};

impl App {
    /// The menus, one button each in the menu bar.
    pub(super) fn menus(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Datei", |ui| self.file_menu(ui));
        ui.menu_button("Bearbeiten", |ui| self.edit_menu(ui));
        ui.menu_button("Ansicht", |ui| self.view_menu(ui));
        ui.menu_button("Werkzeugwege", |ui| self.toolpath_menu(ui));
        ui.menu_button("Hilfe", |ui| {
            self.menu_item(ui, "Tastenkürzel", Action::Shortcuts, true);
            if ui.button("Über CncCraft").clicked() {
                self.about = true;
                ui.close_menu();
            }
        });
    }

    fn file_menu(&mut self, ui: &mut egui::Ui) {
        self.menu_item(ui, "Neu…", Action::New, true);
        self.menu_item(ui, "Öffnen…", Action::Open, true);
        if ui.button("Beispiel öffnen").clicked() {
            self.guarded(Some(ui.ctx()), super::Pending::Example);
            ui.close_menu();
        }
        ui.separator();
        self.menu_item(ui, "Speichern", Action::Save, true);
        self.menu_item(ui, "Speichern unter…", Action::SaveAs, true);
        if ui.button("Werkzeugwege speichern…").clicked() {
            self.export_gcode();
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Startseite").clicked() {
            self.start_page = true;
            ui.close_menu();
        }
    }

    fn edit_menu(&mut self, ui: &mut egui::Ui) {
        let (can_undo, can_redo, any) = (!self.undo.is_empty(), !self.redo.is_empty(), !self.edit.selection.is_empty());
        self.menu_item(ui, "Rückgängig", Action::Undo, can_undo);
        self.menu_item(ui, "Wiederholen", Action::Redo, can_redo);
        ui.separator();
        self.menu_item(ui, "Ausschneiden", Action::Cut, any);
        self.menu_item(ui, "Kopieren", Action::Copy, any);
        let can_paste = !self.clipboard.is_empty();
        self.menu_item(ui, "Einfügen", Action::Paste, can_paste);
        self.menu_item(ui, "Duplizieren", Action::Duplicate, any);
        self.menu_item(ui, "Löschen", Action::Delete, any);
        ui.separator();
        self.menu_item(ui, "Alles auswählen", Action::SelectAll, true);
        self.menu_item(ui, "Knoten bearbeiten", Action::NodeEdit, any || self.tool == DrawTool::NodeEdit);
        if ui.add_enabled(self.edit.selection.len() == 1, egui::Button::new("Maße bearbeiten").shortcut_text("↩")).clicked() {
            self.open_selected_dims();
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Job einrichten…").clicked() {
            self.job_dialog = Some((self.job.clone(), false));
            ui.close_menu();
        }
    }

    fn view_menu(&mut self, ui: &mut egui::Ui) {
        self.menu_item(ui, "2D-Ansicht", Action::View2D, true);
        self.menu_item(ui, "3D-Ansicht", Action::View3D, true);
        if ui.checkbox(&mut self.cam.perspective, "Perspektivische Darstellung (3D)").clicked() {
            ui.close_menu();
        }
        self.menu_item(ui, "Ganze Ansicht", Action::FitView, true);
        // the original's Appearance settings offer System, Light and Dark (0x10018da14)
        ui.menu_button("Design", |ui| {
            use crate::theme::Choice;
            for (choice, name) in [(Choice::System, "System"), (Choice::Light, "Hell"), (Choice::Dark, "Dunkel")] {
                if ui.radio(self.theme_choice == choice, name).clicked() {
                    let ctx = ui.ctx().clone();
                    self.set_theme_choice(&ctx, choice);
                    ui.close_menu();
                }
            }
        });
        ui.separator();
        // every worksheet decoration can be shown or hidden on its own
        ui.label(egui::RichText::new("Arbeitsfläche").strong());
        for (name, shown) in self.sheet.switches() {
            ui.checkbox(shown, name);
        }
        ui.separator();
        // the 3D view's decorations, too
        ui.label(egui::RichText::new("3D-Ansicht").strong());
        for (name, shown) in self.view_decos.switches() {
            ui.checkbox(shown, name);
        }
        ui.separator();
        ui.checkbox(&mut self.show_dims, "Maße anzeigen");
        ui.separator();
        ui.label(egui::RichText::new("Einrasten").strong());
        ui.checkbox(&mut self.snap_geometry, "An Geometrie (Knoten, Mitten)");
        ui.checkbox(&mut self.snap_job, "An Job-Ecken und -Mitte");
        ui.horizontal(|ui| {
            ui.label("Radius");
            ui.add(egui::Slider::new(&mut self.snap_radius_px, SNAP_RADIUS_MIN_PX..=SNAP_RADIUS_MAX_PX).step_by(1.0).suffix(" px"));
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.snap_grid, "Am Raster");
            ui.add(egui::DragValue::new(&mut self.grid_spacing).range(0.1..=500.0).speed(0.5).suffix(" mm"));
        });
    }

    fn toolpath_menu(&mut self, ui: &mut egui::Ui) {
        for (k, l) in [("profile", "Profil"), ("pocket", "Tasche"), ("drill", "Bohren"), ("vcarve", "V-Carving"), ("cutout", "Ausschnitt")] {
            if ui.button(l).clicked() {
                self.open_editor(k);
                ui.close_menu();
            }
        }
        ui.separator();
        if ui.button("Alle neu berechnen").clicked() {
            self.recompute_all();
            ui.close_menu();
        }
        if ui.button("Vorschau").clicked() {
            self.start_preview(false);
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Postprozessor-Verwaltung…").clicked() {
            let idx = self.post_idx.min(self.postprocs.len().saturating_sub(1));
            self.pp_dialog = Some(super::PpDialog {
                editing: self.postprocs[idx].clone(),
                index: Some(idx),
                hook_tab: 0,
            });
            ui.close_menu();
        }
    }

    /// A menu entry showing its keys from the shortcut table; runs the action when clicked.
    pub(super) fn menu_item(&mut self, ui: &mut egui::Ui, label: &str, action: Action, enabled: bool) {
        let button = egui::Button::new(label).shortcut_text(shortcuts::keys_for(action));
        if ui.add_enabled(enabled, button).clicked() {
            self.run(action);
            ui.close_menu();
        }
    }
}
