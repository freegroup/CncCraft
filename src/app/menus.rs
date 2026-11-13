//! The program's menus: Datei, Bearbeiten, Ansicht, Werkzeugwege and Hilfe, each at the top
//! level of the menu bar in the title bar (see `layout::title_bar`). Every entry with keys
//! shows them from the shortcut table.

use super::shortcuts::{self, Action};
use super::{App, DrawTool, SNAP_RADIUS_MAX_PX, SNAP_RADIUS_MIN_PX};

impl App {
    /// The menus, one button each in the menu bar.
    pub(super) fn menus(&mut self, ui: &mut egui::Ui) {
        ui.menu_button(t!("Datei"), |ui| self.file_menu(ui));
        ui.menu_button(t!("Bearbeiten"), |ui| self.edit_menu(ui));
        ui.menu_button(t!("Ansicht"), |ui| self.view_menu(ui));
        ui.menu_button(t!("Werkzeugwege"), |ui| self.toolpath_menu(ui));
        ui.menu_button(t!("Hilfe"), |ui| {
            self.menu_item(ui, &t!("Tastenkürzel"), Action::Shortcuts, true);
            if ui.button(t!("Über CncCraft")).clicked() {
                self.about = true;
                ui.close_menu();
            }
        });
    }

    fn file_menu(&mut self, ui: &mut egui::Ui) {
        self.menu_item(ui, &t!("Neu…"), Action::New, true);
        self.menu_item(ui, &t!("Öffnen…"), Action::Open, true);
        if ui.button(t!("Beispiel öffnen")).clicked() {
            self.guarded(Some(ui.ctx()), super::Pending::Example);
            ui.close_menu();
        }
        ui.separator();
        self.menu_item(ui, &t!("Speichern"), Action::Save, true);
        self.menu_item(ui, &t!("Speichern unter…"), Action::SaveAs, true);
        if ui.button(t!("Werkzeugwege speichern…")).clicked() {
            self.export_gcode();
            ui.close_menu();
        }
        ui.separator();
        if ui.button(t!("Startseite")).clicked() {
            self.start_page = true;
            ui.close_menu();
        }
    }

    fn edit_menu(&mut self, ui: &mut egui::Ui) {
        let (can_undo, can_redo, any) = (!self.undo.is_empty(), !self.redo.is_empty(), !self.edit.selection.is_empty());
        self.menu_item(ui, &t!("Rückgängig"), Action::Undo, can_undo);
        self.menu_item(ui, &t!("Wiederholen"), Action::Redo, can_redo);
        ui.separator();
        self.menu_item(ui, &t!("Ausschneiden"), Action::Cut, any);
        self.menu_item(ui, &t!("Kopieren"), Action::Copy, any);
        let can_paste = !self.clipboard.is_empty();
        self.menu_item(ui, &t!("Einfügen"), Action::Paste, can_paste);
        self.menu_item(ui, &t!("Duplizieren"), Action::Duplicate, any);
        self.menu_item(ui, &t!("Löschen"), Action::Delete, any);
        ui.separator();
        self.menu_item(ui, &t!("Gruppieren"), Action::Group, self.selection_can_group());
        self.menu_item(ui, &t!("Gruppe auflösen"), Action::Ungroup, self.selection_has_group());
        ui.separator();
        self.menu_item(ui, &t!("Alles auswählen"), Action::SelectAll, true);
        self.menu_item(ui, &t!("Knoten bearbeiten"), Action::NodeEdit, any || self.tool == DrawTool::NodeEdit);
        if ui.add_enabled(self.edit.selection.len() == 1, egui::Button::new(t!("Maße bearbeiten")).shortcut_text("↩")).clicked() {
            self.open_selected_dims();
            ui.close_menu();
        }
        ui.separator();
        if ui.button(t!("Job einrichten…")).clicked() {
            self.job_dialog = Some((self.job.clone(), false));
            ui.close_menu();
        }
    }

    fn view_menu(&mut self, ui: &mut egui::Ui) {
        self.menu_item(ui, &t!("2D-Ansicht"), Action::View2D, true);
        self.menu_item(ui, &t!("3D-Ansicht"), Action::View3D, true);
        if ui.checkbox(&mut self.cam.perspective, t!("Perspektivische Darstellung (3D)")).clicked() {
            ui.close_menu();
        }
        self.menu_item(ui, &t!("Ganze Ansicht"), Action::FitView, true);
        // the original's Appearance settings offer System, Light and Dark (0x10018da14)
        ui.menu_button(t!("Design"), |ui| {
            use crate::theme::Choice;
            for (choice, name) in [(Choice::System, t!("System")), (Choice::Light, t!("Hell")), (Choice::Dark, t!("Dunkel"))] {
                if ui.radio(self.theme_choice == choice, name).clicked() {
                    let ctx = ui.ctx().clone();
                    self.set_theme_choice(&ctx, choice);
                    ui.close_menu();
                }
            }
        });
        ui.separator();
        // every worksheet decoration can be shown or hidden on its own
        ui.label(egui::RichText::new(t!("Arbeitsfläche")).strong());
        for (name, shown) in self.sheet.switches() {
            ui.checkbox(shown, t!(name));
        }
        ui.separator();
        // the 3D view's decorations, too
        ui.label(egui::RichText::new(t!("3D-Ansicht")).strong());
        for (name, shown) in self.view_decos.switches() {
            ui.checkbox(shown, t!(name));
        }
        ui.separator();
        ui.checkbox(&mut self.show_dims, t!("Maße anzeigen"));
        ui.separator();
        ui.label(egui::RichText::new(t!("Einrasten")).strong());
        ui.checkbox(&mut self.snap_geometry, t!("An Geometrie (Knoten, Mitten)"));
        ui.checkbox(&mut self.snap_job, t!("An Job-Ecken und -Mitte"));
        ui.horizontal(|ui| {
            ui.label(t!("Radius"));
            ui.add(egui::Slider::new(&mut self.snap_radius_px, SNAP_RADIUS_MIN_PX..=SNAP_RADIUS_MAX_PX).step_by(1.0).suffix(" px"));
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.snap_grid, t!("Am Raster"));
            ui.add(egui::DragValue::new(&mut self.grid_spacing).range(0.1..=500.0).speed(0.5).suffix(" mm"));
        });
    }

    fn toolpath_menu(&mut self, ui: &mut egui::Ui) {
        for (k, l) in [("profile", t!("Profil")), ("pocket", t!("Tasche")), ("drill", t!("Bohren")), ("vcarve", t!("V-Carving")), ("cutout", t!("Ausschnitt"))] {
            if ui.button(l).clicked() {
                self.open_editor(k);
                ui.close_menu();
            }
        }
        ui.separator();
        if ui.button(t!("Alle neu berechnen")).clicked() {
            self.recompute_all();
            ui.close_menu();
        }
        if ui.button(t!("Vorschau")).clicked() {
            self.start_preview(false);
            ui.close_menu();
        }
        ui.separator();
        if ui.button(t!("Postprozessor-Verwaltung…")).clicked() {
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
