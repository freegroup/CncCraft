use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

/// A multi-option slide toggle with an animated pill that slides between the labels.
///
/// ```ignore
/// let resp = SlideToggle::new("my_toggle", &["Off", "On"])
///     .active_color(MY_COLOR)
///     .show(ui, &mut selected_index);
/// ```
pub struct SlideToggle<'a> {
    id: &'a str,
    labels: &'a [&'a str],
    width: f32,
    height: f32,
    font_size: f32,
    active_color: Color32,
    anim_secs: f32,
}

impl<'a> SlideToggle<'a> {
    pub fn new(id: &'a str, labels: &'a [&'a str]) -> Self {
        let n = labels.len().max(2);
        Self {
            id,
            labels,
            width: n as f32 * 36.0,
            height: 28.0,
            font_size: 13.0,
            active_color: crate::theme::ACCENT,
            anim_secs: 0.15,
        }
    }

    pub fn width(mut self, w: f32) -> Self { self.width = w; self }
    #[allow(dead_code)]
    pub fn height(mut self, h: f32) -> Self { self.height = h; self }
    #[allow(dead_code)]
    pub fn font_size(mut self, s: f32) -> Self { self.font_size = s; self }
    #[allow(dead_code)]
    pub fn active_color(mut self, c: Color32) -> Self { self.active_color = c; self }

    pub fn show(self, ui: &mut egui::Ui, value: &mut usize) -> egui::Response {
        let n = self.labels.len().max(2);
        let seg = self.width / n as f32;
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(self.width, self.height), Sense::click());
        let rounding = self.height / 2.0;
        ui.painter().rect(rect, rounding, crate::theme::field_bg(), Stroke::new(1.0_f32, crate::theme::field_stroke()));

        let target = *value as f32 / (n - 1).max(1) as f32;
        let anim = ui.ctx().animate_value_with_time(egui::Id::new(self.id), target, self.anim_secs);
        let pill_x = rect.left() + 2.0 + anim * (self.width - seg - 2.0);
        let pill = Rect::from_min_size(
            Pos2::new(pill_x, rect.top() + 2.0),
            Vec2::new(seg - 2.0, self.height - 4.0),
        );
        ui.painter().rect_filled(pill, rounding - 2.0, self.active_color);

        let on_color = Color32::WHITE;
        let off_color = crate::theme::muted();
        for (i, label) in self.labels.iter().enumerate() {
            let cx = rect.left() + seg * (i as f32 + 0.5);
            let near = 1.0 - (anim - i as f32 / (n - 1).max(1) as f32).abs().min(0.5) * 2.0;
            let col = if near > 0.5 { on_color } else { off_color };
            ui.painter().text(Pos2::new(cx, rect.center().y), Align2::CENTER_CENTER, *label, FontId::proportional(self.font_size), col);
        }

        if resp.clicked() {
            if n == 2 {
                *value = 1 - *value;
            } else if let Some(pos) = ui.input(|i| i.pointer.latest_pos()) {
                let clicked_seg = ((pos.x - rect.left()) / seg).floor().clamp(0.0, (n - 1) as f32) as usize;
                if clicked_seg != *value {
                    *value = clicked_seg;
                }
            } else {
                *value = (*value + 1) % n;
            }
        }
        resp
    }
}
