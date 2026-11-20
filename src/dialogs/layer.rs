//! The small windows of the layer list: a name for a new layer, a new name for a layer, and
//! what happens to the shapes of a layer being deleted.

use crate::app::*;
use crate::app::layers_ui::*;
use crate::layers::Leftovers;

impl App {
    /// The windows of the layer list: new, rename, delete.
    pub(in crate::app) fn layer_window(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.layer_dialog.take() else { return };
        let mut keep = true;
        let mut next = None;
        match dialog {
            LayerDialog::New { mut text, fresh, mut image } => {
                let (ok, cancel) = new_layer_window(ctx, &mut text, &mut image, fresh);
                if ok {
                    self.last_layer_image = image;
                    let cmd = crate::commandstack::LayerAddCommand::new(text.clone(), image);
                    self.execute_command(Box::new(cmd));
                    let id = self.layers.list.last().map_or(0, |l| l.id);
                    if image {
                        self.tool = DrawTool::Select;
                        self.tool_form = None;
                    }
                    self.status = format!("Ebene \u{201e}{}\u{201c} angelegt", self.layers.name(id));
                    keep = false;
                } else if cancel {
                    keep = false;
                }
                if keep {
                    next = Some(LayerDialog::New { text, fresh: false, image });
                }
            }
            LayerDialog::Rename { id, mut text, fresh } => {
                let (ok, cancel) = name_window(ctx, &t!("Ebene umbenennen"), &mut text, fresh);
                if ok {
                    self.execute_command(Box::new(crate::commandstack::LayerRenameCommand::new(id, text.clone())));
                    keep = false;
                } else if cancel {
                    keep = false;
                }
                if keep {
                    next = Some(LayerDialog::Rename { id, text, fresh: false });
                }
            }
            LayerDialog::Delete { id } => {
                let count = self.layers.count_on(id, &self.shapes);
                let name = self.layers.name(id).to_string();
                let mut choice = None;
                dialogs::window("Ebene löschen", ctx).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
                    if dialogs::header(ui, &t!("Ebene löschen")) {
                        choice = Some(None);
                    }
                    ui.label(t!("Ebene „%{name}“ enthält %{count} Form(en).", name = name, count = count));
                    dialog_buttons(ui, |ui| {
                        if ui.button(t!("Formen auf andere Ebene verschieben")).clicked() {
                            choice = Some(Some(Leftovers::MoveAway));
                        }
                        if ui.button(t!("Formen mit löschen")).clicked() {
                            choice = Some(Some(Leftovers::Delete));
                        }
                        if ui.button(t!("Abbrechen")).clicked() {
                            choice = Some(None);
                        }
                    });
                });
                match choice {
                    Some(Some(how)) => self.delete_layer(id, how),
                    Some(None) => {}
                    None => next = Some(LayerDialog::Delete { id }),
                }
            }
        }
        self.layer_dialog = next;
    }
}

/// The "new layer" window: a type selector (Vector / Image) and a name field.
fn new_layer_window(ctx: &egui::Context, text: &mut String, image: &mut bool, fresh: bool) -> (bool, bool) {
    let (mut ok, mut cancel) = (false, false);
    dialogs::window("new_layer_window", ctx).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
        if dialogs::header(ui, &t!("Neue Ebene")) {
            cancel = true;
        }
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(t!("Typ")).strong());
            ui.add_space(8.0);
            if ui.radio(!*image, t!("Vektor")).clicked() {
                *image = false;
            }
            if ui.radio(*image, t!("Bild")).clicked() {
                *image = true;
            }
        });
        ui.add_space(6.0);
        ui.label(egui::RichText::new(t!("Name")).strong());
        ui.add_space(2.0);
        let resp = ui.add(egui::TextEdit::singleline(text).id(egui::Id::new("layer_name")).desired_width(240.0));
        if fresh {
            resp.request_focus();
            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), resp.id) {
                state.cursor.set_char_range(Some(egui::text::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(text.chars().count()))));
                state.store(ui.ctx(), resp.id);
            }
        }
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            ok = true;
        }
        dialog_buttons(ui, |ui| {
            ok |= ui.add(egui::Button::new(egui::RichText::new(t!("OK")).strong().color(INK)).fill(ACCENT).min_size(Vec2::new(80.0, 30.0)).rounding(4.0)).clicked();
            cancel |= ui.button(t!("Abbrechen")).clicked();
        });
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        cancel = true;
    }
    (ok, cancel)
}

/// A window with one name field; returns (confirmed, cancelled).
fn name_window(ctx: &egui::Context, title: &str, text: &mut String, fresh: bool) -> (bool, bool) {
    let (mut ok, mut cancel) = (false, false);
    dialogs::window("name_window", ctx).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
        if dialogs::header(ui, title) {
            cancel = true;
        }
        let resp = ui.add(egui::TextEdit::singleline(text).id(egui::Id::new("layer_name")).desired_width(240.0));
        if fresh {
            resp.request_focus();
            // select the whole name so typing replaces it
            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), resp.id) {
                state.cursor.set_char_range(Some(egui::text::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(text.chars().count()))));
                state.store(ui.ctx(), resp.id);
            }
        }
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            ok = true;
        }
        dialog_buttons(ui, |ui| {
            ok |= ui.add(egui::Button::new(egui::RichText::new(t!("OK")).strong().color(INK)).fill(ACCENT).min_size(Vec2::new(80.0, 30.0)).rounding(4.0)).clicked();
            cancel |= ui.button(t!("Abbrechen")).clicked();
        });
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        cancel = true;
    }
    (ok, cancel)
}
