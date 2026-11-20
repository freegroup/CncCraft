//! The layer panel: a Photoshop-style list with thumbnail previews, per-layer opacity, and
//! operations (duplicate, merge down, drag-drop reorder). Opened by clicking the layer selector
//! in the title bar.

use super::icons::Colours;
use super::App;
use crate::layers::{Leftovers, PALETTE};
use crate::theme::{ink, muted};
use egui::{Align2, Color32, FontId, Id, Pos2, Rect, Sense, Stroke, Vec2};

pub(super) enum LayerDialog {
    /// A name for a new layer (and whether it is an image layer).
    New { text: String, fresh: bool, image: bool },
    /// A new name for a layer.
    Rename { id: u32, text: String, fresh: bool },
    /// What to do with the shapes of a layer that is being deleted.
    Delete { id: u32 },
}

fn stroke(c: Color32) -> Stroke {
    Stroke::new(1.4_f32, c)
}

fn paint_eye(p: &egui::Painter, c: Pos2, open: bool, on_bar: bool) {
    let oa = crate::theme::on_accent();
    let k = if on_bar { Colours { line: oa, top: oa, bottom: oa, back: crate::theme::layer_active() } } else { Colours::normal() };
    super::icons::paint_eye(p, c, open, k, false);
}

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

fn paint_checkerboard(p: &egui::Painter, rect: Rect) {
    let light = Color32::from_gray(200);
    let dark = Color32::from_gray(150);
    p.rect_filled(rect, 2.0, light);
    let step = 6.0;
    let cols = ((rect.width() / step).ceil() as usize).min(20);
    let rows = ((rect.height() / step).ceil() as usize).min(20);
    for r in 0..rows {
        for c in 0..cols {
            if (r + c) % 2 == 0 { continue; }
            let x0 = rect.left() + c as f32 * step;
            let y0 = rect.top() + r as f32 * step;
            let cell = Rect::from_min_max(
                Pos2::new(x0.max(rect.left()), y0.max(rect.top())),
                Pos2::new((x0 + step).min(rect.right()), (y0 + step).min(rect.bottom())),
            );
            p.rect_filled(cell, 0.0, dark);
        }
    }
}

fn paint_op_icon(p: &egui::Painter, c: Pos2, kind: &str, tone: Color32) {
    let s = Stroke::new(1.3_f32, tone);
    match kind {
        "duplicate" => {
            let a = Rect::from_center_size(c + Vec2::new(-2.0, 2.0), Vec2::splat(9.0));
            let b = Rect::from_center_size(c + Vec2::new(2.0, -2.0), Vec2::splat(9.0));
            p.rect_stroke(a, 1.0, s);
            p.rect_stroke(b, 1.0, s);
        }
        "merge" => {
            p.line_segment([c + Vec2::new(0.0, -5.0), c + Vec2::new(0.0, 5.0)], s);
            p.add(egui::Shape::line(vec![c + Vec2::new(-3.0, 2.0), c + Vec2::new(0.0, 5.0), c + Vec2::new(3.0, 2.0)], s));
        }
        "flatten" => {
            p.line_segment([c + Vec2::new(-5.0, 4.0), c + Vec2::new(5.0, 4.0)], s);
            p.rect_stroke(Rect::from_center_size(c + Vec2::new(0.0, -1.0), Vec2::new(8.0, 6.0)), 1.0, s);
        }
        _ => {}
    }
}

/// Paint the drag grip dots (two columns of three dots).
fn paint_grip(p: &egui::Painter, center: Pos2, tone: Color32) {
    for dx in [-2.0, 2.0] {
        for dy in [-5.0, 0.0, 5.0] {
            p.circle_filled(center + Vec2::new(dx, dy), 1.2, tone);
        }
    }
}

impl App {
    pub(super) fn layer_selector(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let resp = ui.interact(rect, Id::new("layer_selector"), Sense::click());
        let active = self.layers.active;
        let is_image = self.layers.is_image(active);
        let colour_idx = self.layers.get(active).map_or(0, |l| l.colour);
        let c3 = PALETTE[colour_idx % PALETTE.len()];
        let layer_colour = Color32::from_rgb(c3[0], c3[1], c3[2]);
        let mut name = self.layers.name(active).to_string();
        if is_image {
            name.push_str("  \u{1f5bc}");
        }
        if !self.layers.is_visible(active) {
            name.push_str(&t!("  (ausgeblendet)"));
        } else if self.layers.is_locked(active) {
            name.push_str(&t!("  (gesperrt)"));
        }
        let p = ui.painter();
        let bg = if resp.hovered() || self.layer_open {
            layer_colour.gamma_multiply(0.35)
        } else {
            layer_colour.gamma_multiply(0.2)
        };
        p.rect(rect, 6.0, bg, Stroke::new(1.0_f32, layer_colour.gamma_multiply(0.5)));
        super::icons::paint_layers_symbol(p, rect.left_center() + Vec2::new(18.0, 0.0), Colours::normal());
        p.text(rect.left_center() + Vec2::new(40.0, 0.0), Align2::LEFT_CENTER, name, FontId::proportional(15.0), ink());
        let c = rect.right_center() - Vec2::new(16.0, 0.0);
        p.add(egui::Shape::line(vec![c + Vec2::new(-5.0, -2.5), c + Vec2::new(0.0, 3.0), c + Vec2::new(5.0, -2.5)], Stroke::new(1.8_f32, ink())));
        if resp.on_hover_text(t!("Ebenen")).clicked() {
            self.layer_open = !self.layer_open;
        }
        if self.layer_open {
            self.layer_list(ui.ctx(), rect);
        }
    }

    fn layer_list(&mut self, ctx: &egui::Context, anchor: Rect) {
        let width = 340.0;
        let pos = Pos2::new(anchor.center().x - width / 2.0, anchor.bottom() + 4.0);
        let mut close = false;
        let area = egui::Area::new(Id::new("layer_list")).order(egui::Order::Foreground).fixed_pos(pos).show(ctx, |ui| {
            egui::Frame::popup(ui.style()).rounding(12.0).inner_margin(10.0).show(ui, |ui| {
                ui.set_width(width - 20.0);
                ui.spacing_mut().item_spacing.y = 2.0;

                self.layer_opacity_bar(ui, width - 20.0);
                ui.add_space(4.0);

                let mut rows: Vec<Rect> = Vec::new();
                let mut drag_started: Option<usize> = None;

                egui::Frame::none().stroke(Stroke::new(1.0_f32, crate::theme::separator())).rounding(8.0).inner_margin(6.0).show(ui, |ui| {
                    ui.set_min_height(96.0);
                    ui.set_width(width - 20.0 - 14.0);
                    ui.spacing_mut().item_spacing.y = 3.0;
                    let layers: Vec<_> = self.layers.list.iter().enumerate().map(|(idx, l)| {
                        (idx, l.id, l.name.clone(), l.visible, l.locked, l.colour, l.behavior.is_image(), l.opacity)
                    }).collect();
                    for (idx, id, name, visible, locked, colour, is_img, opacity) in layers {
                        let row_rect = self.layer_row(ui, idx, id, &name, visible, locked, colour, is_img, opacity, &mut close, &mut drag_started);
                        rows.push(row_rect);
                    }
                });

                // Drag-drop logic
                if drag_started.is_some() {
                    self.layer_drag = drag_started;
                }
                if let Some(from) = self.layer_drag {
                    let pointer = ui.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos()));
                    let to = pointer.map_or(from, |p| rows.iter().filter(|r| r.center().y < p.y).count());
                    if let (Some(first), Some(last)) = (rows.first(), rows.last()) {
                        let y = if to == 0 {
                            first.top() - 2.0
                        } else if to >= rows.len() {
                            last.bottom() + 2.0
                        } else {
                            (rows[to - 1].bottom() + rows[to].top()) / 2.0
                        };
                        ui.painter().line_segment(
                            [Pos2::new(first.left(), y), Pos2::new(first.right(), y)],
                            Stroke::new(2.5_f32, crate::theme::ACCENT_DEEP),
                        );
                    }
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                    if !ui.input(|i| i.pointer.primary_down()) {
                        self.layer_drag = None;
                        if from < self.layers.list.len() {
                            self.execute_command(Box::new(crate::commandstack::LayerReorderCommand::new(from, to)));
                        }
                    }
                }

                ui.add_space(4.0);
                self.layer_ops_bar(ui, &mut close);
            });
        });
        let outside = ctx.input(|i| i.pointer.any_pressed() && i.pointer.interact_pos().is_some_and(|p| !area.response.rect.contains(p) && !anchor.contains(p)));
        if close || outside || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.layer_open = false;
            self.layer_drag = None;
        }
    }

    fn layer_opacity_bar(&mut self, ui: &mut egui::Ui, width: f32) {
        let active = self.layers.active;
        let opacity = self.layers.opacity(active);
        let (row, _) = ui.allocate_exact_size(Vec2::new(width, 22.0), Sense::hover());
        let p = ui.painter();
        let label_tone = muted();

        p.text(Pos2::new(row.left() + 2.0, row.center().y), Align2::LEFT_CENTER, t!("Deckkraft"), FontId::proportional(11.0), label_tone);

        let pct = (opacity * 100.0).round() as u32;
        let pct_text = format!("{pct}%");
        p.text(Pos2::new(row.right() - 2.0, row.center().y), Align2::RIGHT_CENTER, &pct_text, FontId::proportional(11.0), ink());

        let bar_left = row.left() + 68.0;
        let bar_right = row.right() - 40.0;
        let bar_w = bar_right - bar_left;
        if bar_w > 20.0 {
            let bar_y = row.center().y;
            let bar_rect = Rect::from_min_max(Pos2::new(bar_left, bar_y - 3.0), Pos2::new(bar_right, bar_y + 3.0));
            p.rect_filled(bar_rect, 3.0, crate::theme::separator());
            let fill_right = bar_left + bar_w * opacity;
            let fill_rect = Rect::from_min_max(Pos2::new(bar_left, bar_y - 3.0), Pos2::new(fill_right, bar_y + 3.0));
            p.rect_filled(fill_rect, 3.0, crate::theme::icon_line().gamma_multiply(0.6));

            let slider_resp = ui.interact(
                Rect::from_min_max(Pos2::new(bar_left - 5.0, bar_y - 8.0), Pos2::new(bar_right + 5.0, bar_y + 8.0)),
                Id::new("layer_opacity_bar"),
                Sense::click_and_drag(),
            );
            if slider_resp.drag_started() {
                self.opacity_before_drag = Some(opacity);
            }
            if slider_resp.dragged() || slider_resp.clicked() {
                if let Some(pos) = slider_resp.interact_pointer_pos() {
                    let t = ((pos.x - bar_left) / bar_w).clamp(0.0, 1.0);
                    self.layers.set_opacity(active, t);
                }
            }
            let commit_opacity = if slider_resp.drag_stopped() {
                self.opacity_before_drag.take()
            } else if slider_resp.clicked() && self.opacity_before_drag.is_none() {
                Some(opacity)
            } else {
                None
            };
            if let Some(old) = commit_opacity {
                let new_opacity = self.layers.opacity(active);
                if (old - new_opacity).abs() > 0.001 {
                    self.layers.set_opacity(active, old);
                    self.execute_command(Box::new(crate::commandstack::LayerOpacityCommand::new(active, new_opacity)));
                }
            }
            let knob_col = if slider_resp.hovered() || slider_resp.dragged() { ink() } else { crate::theme::icon_line() };
            p.circle_filled(Pos2::new(fill_right, bar_y), 5.0, knob_col);
        }
    }

    fn layer_ops_bar(&mut self, ui: &mut egui::Ui, close: &mut bool) {
        ui.horizontal(|ui| {
            egui::Frame::none().fill(crate::theme::hover()).rounding(6.0).inner_margin(egui::Margin::symmetric(2.0, 0.0)).show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let tone = crate::theme::icon_line();
                let disabled_tone = crate::theme::disabled();
                let active_id = self.layers.active;
                let active_idx = self.layers.list.iter().position(|l| l.id == active_id);
                let layer_count = self.layers.list.len();
                let can_merge = active_idx.is_some_and(|i| i + 1 < layer_count);
                let can_delete = self.layers.can_delete(active_id);

                // Duplicate
                let dup_btn = ui.allocate_exact_size(Vec2::new(26.0, 22.0), Sense::click());
                paint_op_icon(ui.painter(), dup_btn.0.center(), "duplicate", tone);
                if dup_btn.1.on_hover_text(t!("Ebene duplizieren")).clicked() {
                    self.execute_command(Box::new(crate::commandstack::LayerDuplicateCommand::new(active_id)));
                    self.recompute_all();
                }

                // Merge down
                let merge_btn = ui.allocate_exact_size(Vec2::new(26.0, 22.0), Sense::click());
                paint_op_icon(ui.painter(), merge_btn.0.center(), "merge", if can_merge { tone } else { disabled_tone });
                if can_merge && merge_btn.1.on_hover_text(t!("Auf darunterliegende Ebene reduzieren")).clicked() {
                    self.execute_command(Box::new(crate::commandstack::LayerMergeCommand::new(active_id)));
                    self.recompute_all();
                }

                // Flatten
                let flatten_btn = ui.allocate_exact_size(Vec2::new(26.0, 22.0), Sense::click());
                paint_op_icon(ui.painter(), flatten_btn.0.center(), "flatten", if layer_count > 1 { tone } else { disabled_tone });
                if layer_count > 1 && flatten_btn.1.on_hover_text(t!("Alle Ebenen zusammenfassen")).clicked() {
                    self.execute_command(Box::new(crate::commandstack::LayerFlattenCommand::new()));
                    self.recompute_all();
                }

                let (sep, _) = ui.allocate_exact_size(Vec2::new(1.0, 14.0), Sense::hover());
                ui.painter().rect_filled(sep, 0.0, crate::theme::outline());

                // New layer
                if ui.add(egui::Button::new(egui::RichText::new("+").strong().size(15.0)).frame(false).min_size(Vec2::new(26.0, 22.0))).on_hover_text(t!("Neue Ebene")).clicked() {
                    let default_name = if self.last_layer_image { "Bild" } else { "Ebene" };
                    self.layer_dialog = Some(LayerDialog::New { text: self.layers.free_name(default_name, None), fresh: true, image: self.last_layer_image });
                    *close = true;
                }
                let (sep2, _) = ui.allocate_exact_size(Vec2::new(1.0, 14.0), Sense::hover());
                ui.painter().rect_filled(sep2, 0.0, crate::theme::outline());
                if ui.add_enabled(can_delete, egui::Button::new(egui::RichText::new("−").strong().size(15.0)).frame(false).min_size(Vec2::new(26.0, 22.0))).on_hover_text(t!("Aktive Ebene löschen")).on_disabled_hover_text(t!("Die letzte Ebene kann nicht gelöscht werden")).clicked() {
                    self.layer_dialog = Some(LayerDialog::Delete { id: active_id });
                    *close = true;
                }
            });
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn layer_row(&mut self, ui: &mut egui::Ui, idx: usize, id: u32, name: &str, visible: bool, locked: bool, colour: usize, is_image: bool, _opacity: f32, _close: &mut bool, drag_started: &mut Option<usize>) -> Rect {
        let active = id == self.layers.active;
        let dragging = self.layer_drag;
        let row_h = 52.0;
        let thumb_size = 40.0;
        let (row, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), row_h), Sense::click());
        let key = Id::new(("layer_row", id));
        let oa = crate::theme::on_accent();
        let tone = if active { oa } else if visible { crate::theme::icon_line() } else { crate::theme::disabled() };
        let text_tone = if active { oa } else if visible { ink() } else { muted() };
        if active {
            ui.painter().rect_filled(row, 6.0, crate::theme::layer_active());
        } else if resp.hovered() {
            ui.painter().rect_filled(row, 6.0, crate::theme::hover());
        }

        let cy = row.center().y;
        let left_x = row.left();

        // Drag grip
        let grip_rect = Rect::from_center_size(Pos2::new(left_x + 8.0, cy), Vec2::new(14.0, row_h));
        let grip_resp = ui.interact(grip_rect, key.with("grip"), Sense::drag());
        let grip_tone = if grip_resp.hovered() || dragging == Some(idx) {
            ink()
        } else {
            crate::theme::grip()
        };
        paint_grip(ui.painter(), grip_rect.center(), grip_tone);
        if grip_resp.hovered() || dragging == Some(idx) {
            ui.ctx().set_cursor_icon(if dragging == Some(idx) { egui::CursorIcon::Grabbing } else { egui::CursorIcon::Grab });
        }
        if grip_resp.drag_started() {
            *drag_started = Some(idx);
        }

        // Eye icon
        let eye_rect = Rect::from_center_size(Pos2::new(left_x + 24.0, cy), Vec2::new(24.0, 24.0));
        let eye = ui.interact(eye_rect, key.with("eye"), Sense::click());
        paint_eye(ui.painter(), eye.rect.center(), visible, active);

        // Lock icon
        let lock_rect = Rect::from_center_size(Pos2::new(left_x + 48.0, cy), Vec2::new(24.0, 24.0));
        let lock = ui.interact(lock_rect, key.with("lock"), Sense::click());
        paint_lock(ui.painter(), lock.rect.center(), locked, tone);

        // Thumbnail preview
        let thumb_left = left_x + 64.0;
        let thumb_rect = Rect::from_min_size(
            Pos2::new(thumb_left, cy - thumb_size / 2.0),
            Vec2::splat(thumb_size),
        );
        paint_checkerboard(ui.painter(), thumb_rect);
        self.paint_layer_thumbnail(ui.painter(), thumb_rect, id, is_image);
        ui.painter().rect_stroke(thumb_rect, 2.0, Stroke::new(1.0_f32, if active { oa.gamma_multiply(0.5) } else { crate::theme::outline() }));

        // Name + type label
        let text_x = thumb_left + thumb_size + 8.0;
        let p = ui.painter();
        let c = PALETTE[colour % PALETTE.len()];
        let layer_colour = Color32::from_rgb(c[0], c[1], c[2]);

        p.text(Pos2::new(text_x, cy - 6.0), Align2::LEFT_CENTER, name, FontId::proportional(13.0), text_tone);

        let type_label = if is_image { t!("Bild") } else { t!("Vektor") };
        let sub_tone = if active { oa.gamma_multiply(0.7) } else { muted() };
        p.text(Pos2::new(text_x, cy + 9.0), Align2::LEFT_CENTER, type_label, FontId::proportional(10.0), sub_tone);

        // Colour dot
        let dot_center = Pos2::new(row.right() - 18.0, cy);
        let dot_rect = Rect::from_center_size(dot_center, Vec2::new(22.0, 22.0));
        let dot = ui.interact(dot_rect, key.with("dot"), Sense::click());
        p.circle_filled(dot_center, 7.5, layer_colour);
        p.circle_stroke(dot_center, 7.5, Stroke::new(1.0_f32, if active { oa } else { crate::theme::outline() }));

        // Interactions
        if eye.on_hover_text(if visible { t!("Sichtbar (klicken zum Ausblenden: Formen werden dann nicht gezeigt, nicht gefräst und nicht exportiert)") } else { t!("Ausgeblendet (klicken zum Einblenden)") }).clicked() {
            self.execute_command(Box::new(crate::commandstack::LayerVisibilityCommand::new(id, !visible)));
        } else if lock.on_hover_text(if locked { t!("Gesperrt (klicken zum Entsperren)") } else { t!("Entsperrt (klicken zum Sperren: bleibt sichtbar, aber nichts darauf lässt sich anklicken oder ändern)") }).clicked() {
            self.execute_command(Box::new(crate::commandstack::LayerLockCommand::new(id, !locked)));
        } else if dot.on_hover_text(t!("Ebenenfarbe wechseln")).clicked() {
            self.execute_command(Box::new(crate::commandstack::LayerColourCommand::new(id)));
        } else if resp.double_clicked() {
            self.layer_dialog = Some(LayerDialog::Rename { id, text: name.to_string(), fresh: true });
            self.layer_open = false;
        } else if resp.on_hover_text(t!("Auf dieser Ebene zeichnen (Doppelklick: umbenennen)")).clicked() {
            self.layers.activate(id);
            if !self.layers.active_can_place_vectors() {
                self.tool = crate::app::DrawTool::Select;
                self.tool_form = None;
            }
        }

        row
    }

    fn paint_layer_thumbnail(&self, painter: &egui::Painter, thumb: Rect, layer_id: u32, is_image: bool) {
        if is_image {
            let has_image = self.layers.get(layer_id)
                .and_then(|l| l.as_image())
                .is_some_and(|ib| !ib.image.bytes.is_empty());
            let has_ref_shape = self.shapes.iter().any(|s| matches!(&s.shape, shape_edit::Shape::RefImage(r) if r.layer_id == layer_id));
            if has_image && has_ref_shape {
                if let Some((tex, [pw, ph])) = self.image_textures.get(&layer_id) {
                    let aspect = *pw as f32 / (*ph as f32).max(1.0);
                    let (tw, th) = if aspect > 1.0 {
                        (thumb.width(), thumb.width() / aspect)
                    } else {
                        (thumb.height() * aspect, thumb.height())
                    };
                    let img_rect = Rect::from_center_size(thumb.center(), Vec2::new(tw, th));
                    let opacity = self.layers.opacity(layer_id);
                    let alpha = (opacity * 255.0) as u8;
                    let tint = Color32::from_rgba_unmultiplied(255, 255, 255, alpha);
                    painter.image(tex.id(), img_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), tint);
                }
            }
            return;
        }

        let layer_shapes: Vec<_> = self.shapes.iter().filter(|s| s.layer == layer_id).collect();
        if layer_shapes.is_empty() { return; }

        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for s in &layer_shapes {
            if let Some((sl, sh)) = s.shape.bounds() {
                lo = [lo[0].min(sl[0]), lo[1].min(sl[1])];
                hi = [hi[0].max(sh[0]), hi[1].max(sh[1])];
            }
        }
        if lo[0] >= hi[0] || lo[1] >= hi[1] { return; }

        let margin = 3.0_f32;
        let avail_w = (thumb.width() - margin * 2.0) as f64;
        let avail_h = (thumb.height() - margin * 2.0) as f64;
        let range_w = hi[0] - lo[0];
        let range_h = hi[1] - lo[1];
        let scale = (avail_w / range_w).min(avail_h / range_h);
        let cx = thumb.center().x as f64;
        let cy = thumb.center().y as f64;
        let mid_x = (lo[0] + hi[0]) / 2.0;
        let mid_y = (lo[1] + hi[1]) / 2.0;

        let to_thumb = |pt: &[f64; 2]| -> Pos2 {
            Pos2::new(
                (cx + (pt[0] - mid_x) * scale) as f32,
                (cy - (pt[1] - mid_y) * scale) as f32,
            )
        };

        let colour_idx = self.layers.get(layer_id).map_or(0, |l| l.colour);
        let c = PALETTE[colour_idx % PALETTE.len()];
        let line_col = Color32::from_rgb(c[0], c[1], c[2]);
        let s = Stroke::new(1.0_f32, line_col);

        for shape in &layer_shapes {
            if matches!(&shape.shape, shape_edit::Shape::RefImage(_)) { continue; }
            for contour in shape.shape.contours() {
                if contour.pts.len() < 2 { continue; }
                let pts: Vec<Pos2> = contour.pts.iter().map(|p| to_thumb(p)).collect();
                if contour.closed {
                    let mut closed_pts = pts;
                    if let Some(&first) = closed_pts.first() {
                        closed_pts.push(first);
                    }
                    painter.add(egui::Shape::line(closed_pts, s));
                } else {
                    painter.add(egui::Shape::line(pts, s));
                }
            }
        }
    }

    pub(in crate::app) fn delete_layer(&mut self, id: u32, leftovers: Leftovers) {
        let name = self.layers.name(id).to_string();
        self.execute_command(Box::new(crate::commandstack::LayerDeleteCommand::new(id, leftovers)));
        self.image_textures.remove(&id);
        self.edit.retain_existing(&self.shapes);
        self.recompute_all();
        self.status = format!("Ebene \u{201e}{}\u{201c} gelöscht", name);
    }
}
