//! The layer selector in the title bar and its list, laid out after the original's: a layers
//! symbol, the name of the active layer and a chevron; the list below has one row per layer with
//! an eye (show / hide), a padlock (lock), the layer's symbol, its name in bold and a colour dot,
//! the active layer on a dark blue bar, and "+" / "−" under the list. Plus the small windows for
//! a new layer, a new name and a layer being deleted.

use super::icons::Colours;
use super::App;
use crate::layers::{Leftovers, PALETTE};
use crate::theme::{ink, muted};
use egui::{Align2, Color32, FontId, Id, Pos2, Rect, Sense, Stroke, Vec2};

/// The dark blue bar of the active layer in the original's list.
const ACTIVE_BAR: Color32 = Color32::from_rgb(0x22, 0x49, 0x85);

pub(super) enum LayerDialog {
    /// A name for a new layer.
    New { text: String, fresh: bool },
    /// A new name for a layer.
    Rename { id: u32, text: String, fresh: bool },
    /// What to do with the shapes of a layer that is being deleted.
    Delete { id: u32 },
}

fn stroke(c: Color32) -> Stroke {
    Stroke::new(1.4_f32, c)
}

/// The eye of a row: the blue icon, or plain white on the active bar.
fn paint_eye(p: &egui::Painter, c: Pos2, open: bool, on_bar: bool) {
    let k = if on_bar { Colours { line: Color32::WHITE, top: Color32::WHITE, bottom: Color32::WHITE, back: ACTIVE_BAR } } else { Colours::normal() };
    super::icons::paint_eye(p, c, open, k, false);
}

/// A padlock, closed or open.
fn paint_lock(p: &egui::Painter, c: Pos2, locked: bool, tone: Color32) {
    let body = Rect::from_center_size(c + Vec2::new(0.0, 3.0), Vec2::new(11.0, 8.0));
    if locked {
        p.rect_filled(body, 1.5, tone);
    } else {
        p.rect_stroke(body, 1.5, stroke(tone));
    }
    let top = if locked { -5.5 } else { -7.0 };
    let right_end = if locked { -1.0 } else { -4.0 };
    let shackle = vec![c + Vec2::new(-3.5, -1.0), c + Vec2::new(-3.5, top + 1.5), c + Vec2::new(-2.0, top), c + Vec2::new(2.0, top), c + Vec2::new(3.5, top + 1.5), c + Vec2::new(3.5, right_end)];
    p.add(egui::Shape::line(shackle, stroke(tone)));
}

/// The small sheet shown in the layer's colour.
fn paint_sheet(p: &egui::Painter, c: Pos2, fill: Color32, edge: Color32) {
    let pts = vec![c + Vec2::new(-7.0, 4.5), c + Vec2::new(-4.0, -4.5), c + Vec2::new(7.0, -4.5), c + Vec2::new(4.0, 4.5)];
    p.add(egui::Shape::convex_polygon(pts, fill, Stroke::new(1.0_f32, edge)));
}

impl App {
    /// The middle of the title bar: symbol, name of the active layer, chevron. A click opens the
    /// list.
    pub(super) fn layer_selector(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let resp = ui.interact(rect, Id::new("layer_selector"), Sense::click());
        let active = self.layers.active;
        let mut name = self.layers.name(active).to_string();
        if !self.layers.is_visible(active) {
            name.push_str("  (ausgeblendet)");
        } else if self.layers.is_locked(active) {
            name.push_str("  (gesperrt)");
        }
        let p = ui.painter();
        if resp.hovered() || self.layer_open {
            p.rect_filled(rect, 5.0, crate::theme::hover());
        }
        super::icons::paint_layers_symbol(p, rect.left_center() + Vec2::new(16.0, 0.0), Colours::normal());
        p.text(rect.left_center() + Vec2::new(36.0, 0.0), Align2::LEFT_CENTER, name, FontId::proportional(14.0), ink());
        // the chevron
        let c = rect.right_center() - Vec2::new(16.0, 0.0);
        p.add(egui::Shape::line(vec![c + Vec2::new(-4.5, -2.0), c + Vec2::new(0.0, 2.5), c + Vec2::new(4.5, -2.0)], Stroke::new(1.6_f32, ink())));
        if resp.on_hover_text("Ebenen").clicked() {
            self.layer_open = !self.layer_open;
        }
        if self.layer_open {
            self.layer_list(ui.ctx(), rect);
        }
    }

    /// The list under the selector.
    fn layer_list(&mut self, ctx: &egui::Context, anchor: Rect) {
        let width = 330.0;
        let pos = Pos2::new(anchor.center().x - width / 2.0, anchor.bottom() + 4.0);
        let mut close = false;
        let area = egui::Area::new(Id::new("layer_list")).order(egui::Order::Foreground).fixed_pos(pos).show(ctx, |ui| {
            egui::Frame::popup(ui.style()).rounding(12.0).inner_margin(10.0).show(ui, |ui| {
                ui.set_width(width - 20.0);
                ui.spacing_mut().item_spacing.y = 4.0;
                egui::Frame::none().stroke(Stroke::new(1.0_f32, crate::theme::separator())).rounding(8.0).inner_margin(6.0).show(ui, |ui| {
                    ui.set_min_height(96.0);
                    ui.set_width(width - 20.0 - 14.0);
                    let layers: Vec<(u32, String, bool, bool, usize)> = self.layers.list.iter().map(|l| (l.id, l.name.clone(), l.visible, l.locked, l.colour)).collect();
                    for (id, name, visible, locked, colour) in layers {
                        self.layer_row(ui, id, &name, visible, locked, colour, &mut close);
                    }
                });
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    egui::Frame::none().fill(crate::theme::hover()).rounding(6.0).inner_margin(egui::Margin::symmetric(2.0, 0.0)).show(ui, |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        if ui.add(egui::Button::new(egui::RichText::new("+").strong().size(15.0)).frame(false).min_size(Vec2::new(26.0, 22.0))).on_hover_text("Neue Ebene").clicked() {
                            self.layer_dialog = Some(LayerDialog::New { text: self.layers.free_name("Ebene", None), fresh: true });
                            close = true;
                        }
                        let (sep, _) = ui.allocate_exact_size(Vec2::new(1.0, 14.0), Sense::hover());
                        ui.painter().rect_filled(sep, 0.0, crate::theme::outline());
                        let can_delete = self.layers.can_delete(self.layers.active);
                        // the original refuses the same way: "Cannot delete last layer" (0x100bc44b0)
                        if ui.add_enabled(can_delete, egui::Button::new(egui::RichText::new("−").strong().size(15.0)).frame(false).min_size(Vec2::new(26.0, 22.0))).on_hover_text("Aktive Ebene löschen").on_disabled_hover_text("Die letzte Ebene kann nicht gelöscht werden").clicked() {
                            self.layer_dialog = Some(LayerDialog::Delete { id: self.layers.active });
                            close = true;
                        }
                    });
                    let can_move = !self.edit.selection.is_empty();
                    if ui.add_enabled(can_move, egui::Button::new("Auswahl hierher").small()).on_hover_text("Die gewählten Formen auf die aktive Ebene verschieben").on_disabled_hover_text("Erst Formen auswählen").clicked() {
                        self.move_selection_to_active_layer();
                    }
                });
            });
        });
        // a click elsewhere, or Esc, closes the list
        let outside = ctx.input(|i| i.pointer.any_pressed() && i.pointer.interact_pos().is_some_and(|p| !area.response.rect.contains(p) && !anchor.contains(p)));
        if close || outside || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.layer_open = false;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn layer_row(&mut self, ui: &mut egui::Ui, id: u32, name: &str, visible: bool, locked: bool, colour: usize, _close: &mut bool) {
        let active = id == self.layers.active;
        let (row, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::click());
        let key = Id::new(("layer_row", id));
        let tone = if active { Color32::WHITE } else if visible { crate::theme::icon_line() } else { crate::theme::disabled() };
        let text_tone = if active { Color32::WHITE } else if visible { ink() } else { muted() };
        if active {
            ui.painter().rect_filled(row, 15.0, ACTIVE_BAR);
        } else if resp.hovered() {
            ui.painter().rect_filled(row, 15.0, crate::theme::hover());
        }
        let cell = |dx: f32| Rect::from_center_size(Pos2::new(row.left() + dx, row.center().y), Vec2::new(26.0, 26.0));
        let eye = ui.interact(cell(24.0), key.with("eye"), Sense::click());
        let lock = ui.interact(cell(52.0), key.with("lock"), Sense::click());
        let dot_rect = Rect::from_center_size(Pos2::new(row.right() - 22.0, row.center().y), Vec2::new(22.0, 22.0));
        let dot = ui.interact(dot_rect, key.with("dot"), Sense::click());
        let p = ui.painter();
        paint_eye(p, eye.rect.center(), visible, active);
        paint_lock(p, lock.rect.center(), locked, tone);
        let c = PALETTE[colour % PALETTE.len()];
        let layer_colour = Color32::from_rgb(c[0], c[1], c[2]);
        paint_sheet(p, Pos2::new(row.left() + 80.0, row.center().y), if active { Color32::WHITE } else { layer_colour }, if active { Color32::WHITE } else { tone });
        p.text(Pos2::new(row.left() + 100.0, row.center().y), Align2::LEFT_CENTER, name, FontId::proportional(14.0), text_tone);
        p.circle_filled(dot_rect.center(), 7.5, layer_colour);
        p.circle_stroke(dot_rect.center(), 7.5, Stroke::new(1.0_f32, if active { Color32::WHITE } else { crate::theme::outline() }));
        if eye.on_hover_text(if visible { "Sichtbar (klicken zum Ausblenden: Formen werden dann nicht gezeigt, nicht gefräst und nicht exportiert)" } else { "Ausgeblendet (klicken zum Einblenden)" }).clicked() {
            self.layers.set_visible(id, !visible);
        } else if lock.on_hover_text(if locked { "Gesperrt (klicken zum Entsperren)" } else { "Entsperrt (klicken zum Sperren: bleibt sichtbar, aber nichts darauf lässt sich anklicken oder ändern)" }).clicked() {
            self.layers.set_locked(id, !locked);
        } else if dot.on_hover_text("Ebenenfarbe wechseln").clicked() {
            self.layers.cycle_colour(id);
        } else if resp.double_clicked() {
            self.layer_dialog = Some(LayerDialog::Rename { id, text: name.to_string(), fresh: true });
            self.layer_open = false;
        } else if resp.on_hover_text("Auf dieser Ebene zeichnen (Doppelklick: umbenennen)").clicked() {
            self.layers.activate(id);
        }
    }

    /// Put the selected shapes on the active layer.
    pub(super) fn move_selection_to_active_layer(&mut self) {
        if self.edit.selection.is_empty() {
            return;
        }
        self.push_undo();
        let (sel, to) = (self.edit.selection.clone(), self.layers.active);
        for it in self.shapes.iter_mut().filter(|s| sel.contains(&s.id)) {
            it.layer = to;
        }
        self.status = format!("{} Form(en) auf „{}“ verschoben", sel.len(), self.layers.name(to));
    }

    fn delete_layer(&mut self, id: u32, leftovers: Leftovers) {
        let name = self.layers.name(id).to_string();
        self.push_undo();
        if self.layers.delete(id, leftovers, &mut self.shapes) {
            self.edit.retain_existing(&self.shapes);
            self.recompute_all();
            self.status = format!("Ebene „{name}“ gelöscht");
        }
    }

    /// The windows of the layer list: new, rename, delete.
    pub(super) fn layer_window(&mut self, ctx: &egui::Context) {
        let Some(dialog) = self.layer_dialog.take() else { return };
        let mut keep = true;
        let mut next = None;
        match dialog {
            LayerDialog::New { mut text, fresh } => {
                let (ok, cancel) = name_window(ctx, "Neue Ebene", &mut text, fresh);
                if ok {
                    let id = self.layers.add(&text);
                    self.status = format!("Ebene „{}“ angelegt", self.layers.name(id));
                    keep = false;
                } else if cancel {
                    keep = false;
                }
                if keep {
                    next = Some(LayerDialog::New { text, fresh: false });
                }
            }
            LayerDialog::Rename { id, mut text, fresh } => {
                let (ok, cancel) = name_window(ctx, "Ebene umbenennen", &mut text, fresh);
                if ok {
                    self.layers.rename(id, &text);
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
                egui::Window::new("Ebene löschen").collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
                    ui.label(format!("Ebene „{name}“ enthält {count} Form(en)."));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("Formen auf andere Ebene verschieben").clicked() {
                            choice = Some(Some(Leftovers::MoveAway));
                        }
                        if ui.button("Formen mit löschen").clicked() {
                            choice = Some(Some(Leftovers::Delete));
                        }
                        if ui.button("Abbrechen").clicked() {
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

/// A window with one name field; returns (confirmed, cancelled).
fn name_window(ctx: &egui::Context, title: &str, text: &mut String, fresh: bool) -> (bool, bool) {
    let (mut ok, mut cancel) = (false, false);
    egui::Window::new(title).collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
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
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ok |= ui.button("OK").clicked();
            cancel |= ui.button("Abbrechen").clicked();
        });
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        cancel = true;
    }
    (ok, cancel)
}
