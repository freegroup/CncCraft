//! The tools available on an image layer: inserting a reference image, calibrating it, and
//! adjusting its transparency.

use super::App;
use crate::theme::{ACCENT, INK};
use egui::{Align2, FontId, Pos2, Stroke, Vec2};
use shape_edit::{RefImage, Pt};

/// The calibrate-image mode: pick two points on the reference image, then enter the real-world
/// distance. The image is scaled so the pixel distance between the two points equals that value.
pub(in crate::app) struct CalibrateMode {
    pub first: Option<Pt>,
    pub second: Option<Pt>,
    pub layer_id: u32,
    /// The text typed in the distance field.
    pub text: String,
    /// Whether the field should be focused on the next frame.
    pub fresh: bool,
}

impl CalibrateMode {
    pub fn new(layer_id: u32) -> Self {
        CalibrateMode { first: None, second: None, layer_id, text: String::new(), fresh: true }
    }
}

impl App {
    /// Open a file dialog to pick an image (PNG/JPEG) for the active image layer.
    pub(in crate::app) fn insert_image_dialog(&mut self) {
        let layer_id = self.layers.active;
        if !self.layers.is_image(layer_id) {
            return;
        }
        let path = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif", "webp"])
            .pick_file();
        if let Some(path) = path {
            match std::fs::read(&path) {
                Ok(bytes) => {
                    let dims = image::load_from_memory(&bytes).ok().map(|i| (i.width(), i.height()));

                    // store the pixel bytes in the layer for rendering
                    if let Some(img) = self.layers.image_mut(layer_id) {
                        img.source = path.display().to_string();
                        img.bytes = bytes;
                    }
                    self.image_textures.remove(&layer_id);

                    // remove any existing RefImage shape for this layer
                    self.shapes.retain(|s| !matches!(&s.shape, shape_edit::Shape::RefImage(r) if r.layer_id == layer_id));

                    // compute size to fit the workspace
                    let (w_mm, h_mm) = if let Some((pw, ph)) = dims {
                        let jw = self.job.width;
                        let jh = self.job.height;
                        let margin = 0.9;
                        let sx = (jw * margin) / pw as f64;
                        let sy = (jh * margin) / ph as f64;
                        let scale = sx.min(sy).min(1.0);
                        (pw as f64 * scale, ph as f64 * scale)
                    } else {
                        (self.job.width * 0.5, self.job.height * 0.5)
                    };

                    // update the layer's scale/offset to match (for texture rendering)
                    if let Some((pw, _)) = dims {
                        if let Some(img) = self.layers.image_mut(layer_id) {
                            img.scale = w_mm / pw as f64;
                            img.offset = [0.0, 0.0];
                        }
                    }

                    // add the RefImage shape
                    let center = [w_mm / 2.0, h_mm / 2.0];
                    let shape = RefImage::new(center, w_mm, h_mm, layer_id);
                    let sid = self.add_shape(shape);
                    self.edit.selection = vec![sid];

                    self.status = format!("Bild geladen: {}", path.file_name().unwrap_or_default().to_string_lossy());
                }
                Err(e) => {
                    self.status = format!("Fehler beim Laden: {e}");
                }
            }
        }
    }

    /// Sync the layer's ImageData offset/scale from the RefImage shape's geometry (called after
    /// the shape is moved or resized by the editor).
    pub(in crate::app) fn sync_image_from_shape(&mut self) {
        for s in &self.shapes {
            if let shape_edit::Shape::RefImage(r) = &s.shape {
                if let Some(img) = self.layers.image_mut(r.layer_id) {
                    let (_, [w, h]) = match self.image_textures.get(&r.layer_id) {
                        Some(v) => v,
                        None => continue,
                    };
                    img.offset = [r.center[0] - r.width / 2.0, r.center[1] - r.height / 2.0];
                    img.scale = r.width / *w as f64;
                    let _ = h;
                }
            }
        }
    }

    /// Enter the calibrate mode: click two points, then type the distance.
    pub(in crate::app) fn start_calibrate(&mut self) {
        let id = self.layers.active;
        let has_image = self.layers.get(id)
            .and_then(|l| l.as_image())
            .is_some_and(|ib| !ib.image.bytes.is_empty());
        if !has_image {
            self.status = t!("Erst ein Bild einfügen.");
            return;
        }
        self.calibrate_mode = Some(CalibrateMode::new(id));
        self.status = t!("Kalibrieren: ersten Punkt im Bild anklicken.");
    }

    /// Cancel the calibrate mode. Returns whether it did anything.
    pub(in crate::app) fn cancel_calibrate(&mut self) -> bool {
        if let Some(mode) = &mut self.calibrate_mode {
            if mode.second.is_some() {
                mode.second = None;
                mode.text.clear();
                mode.fresh = true;
                self.status = t!("Zweiten Punkt im Bild anklicken.");
                return true;
            }
            if mode.first.is_some() {
                mode.first = None;
                self.status = t!("Kalibrieren: ersten Punkt im Bild anklicken.");
                return true;
            }
        }
        if self.calibrate_mode.is_some() {
            self.calibrate_mode = None;
            self.status = t!("Kalibrieren beendet.");
            return true;
        }
        false
    }

    /// Handle clicks in calibrate mode (called from `canvas_input`).
    pub(in crate::app) fn calibrate_input(&mut self, resp: &egui::Response, mouse: Option<Pt>) {
        let Some(mode) = &self.calibrate_mode else { return };
        if mode.second.is_some() {
            return;
        }
        let Some(m) = mouse else { return };
        if !resp.clicked() {
            return;
        }
        let mode = self.calibrate_mode.as_mut().unwrap();
        if mode.first.is_none() {
            mode.first = Some(m);
            self.status = t!("Zweiten Punkt im Bild anklicken.");
        } else {
            mode.second = Some(m);
            mode.fresh = true;
            let p1 = mode.first.unwrap();
            let pixel_dist = ((m[0] - p1[0]).powi(2) + (m[1] - p1[1]).powi(2)).sqrt();
            mode.text = format!("{:.1}", pixel_dist);
            self.status = format!("{} {:.1} mm", t!("Gemessener Abstand im Bild:"), pixel_dist);
        }
    }

    /// Paint the calibrate picks and the measurement line on the canvas.
    pub(in crate::app) fn calibrate_paint(&self, painter: &egui::Painter) {
        let Some(mode) = &self.calibrate_mode else { return };
        let cross = |p: Pt| {
            let s = self.to_screen(p);
            let arm = 8.0;
            let stroke = Stroke::new(2.0_f32, ACCENT);
            painter.line_segment([s - Vec2::new(arm, 0.0), s + Vec2::new(arm, 0.0)], stroke);
            painter.line_segment([s - Vec2::new(0.0, arm), s + Vec2::new(0.0, arm)], stroke);
            painter.circle_stroke(s, 5.0, stroke);
        };
        if let Some(p1) = mode.first {
            cross(p1);
        }
        if let Some(p2) = mode.second {
            cross(p2);
            if let Some(p1) = mode.first {
                let s1 = self.to_screen(p1);
                let s2 = self.to_screen(p2);
                painter.line_segment([s1, s2], Stroke::new(1.5_f32, ACCENT));
                let mid = Pos2::new((s1.x + s2.x) / 2.0, (s1.y + s2.y) / 2.0);
                let pixel_dist = ((p2[0] - p1[0]).powi(2) + (p2[1] - p1[1]).powi(2)).sqrt();
                painter.text(mid + Vec2::new(0.0, -12.0), Align2::CENTER_BOTTOM, format!("{:.1} mm", pixel_dist), FontId::proportional(12.0), ACCENT);
            }
        }
    }

    /// The calibrate dialog: enter the real-world distance. Called from the dialog loop.
    pub(in crate::app) fn calibrate_dialog(&mut self, ctx: &egui::Context) {
        if self.calibrate_mode.as_ref().is_none_or(|m| m.second.is_none()) {
            return;
        }
        let (mut ok, mut cancel) = (false, false);
        super::dialogs::window("calibrate_dialog", ctx)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                if super::dialogs::header(ui, &t!("Bild kalibrieren")) {
                    cancel = true;
                }
                ui.label(t!("Geben Sie den realen Abstand zwischen den zwei Punkten ein:"));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(t!("Abstand")).strong());
                    ui.add_space(8.0);
                    let mode = self.calibrate_mode.as_mut().unwrap();
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut mode.text)
                            .id(egui::Id::new("calibrate_distance"))
                            .desired_width(120.0)
                            .hint_text("mm"),
                    );
                    if mode.fresh {
                        resp.request_focus();
                        mode.fresh = false;
                    }
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        ok = true;
                    }
                    ui.label("mm");
                });
                ui.add_space(4.0);
                super::dialogs::dialog_buttons(ui, |ui| {
                    ok |= ui.add(
                        egui::Button::new(egui::RichText::new(t!("OK")).strong().color(INK))
                            .fill(ACCENT)
                            .min_size(Vec2::new(80.0, 30.0))
                            .rounding(4.0),
                    ).clicked();
                    cancel |= ui.button(t!("Abbrechen")).clicked();
                });
            });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            cancel = true;
        }
        if ok {
            self.apply_calibration();
        }
        if cancel {
            self.calibrate_mode = None;
            self.status = t!("Kalibrieren abgebrochen.");
        }
    }

    /// Apply the typed distance: scale the image so the pixel distance between the two points
    /// matches the entered real-world distance.
    fn apply_calibration(&mut self) {
        let Some(mode) = &self.calibrate_mode else { return };
        let (Some(p1), Some(p2)) = (mode.first, mode.second) else { return };
        let layer_id = mode.layer_id;

        let real_dist: f64 = match mode.text.trim().replace(',', ".").parse::<f64>() {
            Ok(v) if v > 0.0 && v.is_finite() => v,
            _ => {
                self.status = t!("Bitte eine positive Zahl eingeben.");
                return;
            }
        };

        let current_dist = ((p2[0] - p1[0]).powi(2) + (p2[1] - p1[1]).powi(2)).sqrt();
        if current_dist < 1e-9 {
            self.status = t!("Die zwei Punkte liegen aufeinander.");
            return;
        }

        let factor = real_dist / current_dist;

        // find the RefImage shape for this layer and scale it
        if let Some(item) = self.shapes.iter_mut().find(|s| matches!(&s.shape, shape_edit::Shape::RefImage(r) if r.layer_id == layer_id)) {
            if let shape_edit::Shape::RefImage(r) = &mut item.shape {
                let old_center = r.center;
                r.width *= factor;
                r.height *= factor;
                // keep center where it was: the image grows outward
                r.center = old_center;
            }
        }

        self.sync_image_from_shape();
        self.calibrate_mode = None;
        self.status = format!("{} {:.1}x", t!("Bild kalibriert, Faktor:"), factor);
    }
}
