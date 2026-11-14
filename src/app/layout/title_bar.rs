//! The title bar: the menu bar, the layer in the middle, the document's name and the switches
//! for language and theme at the right end.

use crate::app::App;
use crate::theme::{ACCENT, hairline, muted};
use egui::{Align2, FontId, Pos2, Rect, Sense, Stroke, Vec2};

/// Space between the theme switch and the language switch's centre.
const LANG_SLOT: f32 = 38.0;

impl App {
    /// The menu bar on the left, the layer in the middle, the document's name on the right.
    pub(in crate::app) fn title_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("title").exact_height(34.0).frame(egui::Frame::none().fill(crate::theme::surface()).inner_margin(egui::Margin::symmetric(10.0, 4.0)).stroke(Stroke::new(1.0_f32, hairline()))).show(ctx, |ui| {
            let full = ui.max_rect();
            // the start page has no menus: only the switches at the right end
            if !self.start_page {
                // the usual menu bar, every menu at the top level
                ui.horizontal_centered(|ui| {
                    ui.label(egui::RichText::new("⚡").color(ACCENT).size(16.0));
                    egui::menu::bar(ui, |ui| self.menus(ui));
                });
                // the document's name at the right end
                let name = self.path.as_ref().and_then(|p| p.file_stem()).map_or_else(|| t!("Neu"), |s| s.to_string_lossy().into_owned());
                let right = full.right_center() - Vec2::new(38.0 + LANG_SLOT, 0.0);
                // unsaved changes: a dot behind the name (drawn, the font has no such glyph)
                let dot_room = if self.dirty_shown { 16.0 } else { 0.0 };
                let label = ui.painter().text(right - Vec2::new(dot_room, 0.0), Align2::RIGHT_CENTER, name, FontId::proportional(14.0), muted());
                if self.dirty_shown {
                    let dot = Rect::from_center_size(Pos2::new(right.x - 5.0, right.y), Vec2::splat(10.0));
                    ui.painter().circle_filled(dot.center(), 3.5, muted());
                    ui.interact(label.union(dot), egui::Id::new("dirty_mark"), Sense::hover()).on_hover_text(t!("Ungespeicherte Änderungen"));
                }
            }
            // the switch for the dark theme, at the right end: a sun in the dark, a moon in the light
            let slot = Rect::from_center_size(full.right_center() - Vec2::new(14.0, 0.0), Vec2::splat(26.0));
            let resp = ui.interact(slot, egui::Id::new("theme_switch"), Sense::click());
            if resp.hovered() {
                ui.painter().rect_filled(slot, 5.0, crate::theme::hover());
            }
            let c = slot.center();
            let tone = crate::theme::icon_line();
            if self.dark {
                ui.painter().circle_filled(c, 4.0, tone);
                for k in 0..8 {
                    let a = k as f32 * std::f32::consts::FRAC_PI_4;
                    let d = Vec2::new(a.cos(), a.sin());
                    ui.painter().line_segment([c + d * 6.5, c + d * 9.0], Stroke::new(1.5_f32, tone));
                }
            } else {
                ui.painter().circle_filled(c, 7.0, tone);
                ui.painter().circle_filled(c + Vec2::new(3.5, -3.0), 6.0, crate::theme::surface());
            }
            if resp.on_hover_text(if self.dark { t!("Zum hellen Design wechseln") } else { t!("Zum dunklen Design wechseln") }).clicked() {
                let choice = if self.dark { crate::theme::Choice::Light } else { crate::theme::Choice::Dark };
                self.set_theme_choice(ctx, choice);
            }
            // the language switch, left of the theme switch: the code of the language in use, and
            // a list of the languages under it
            let lang_slot = Rect::from_center_size(slot.center() - Vec2::new(LANG_SLOT, 0.0), Vec2::new(34.0, 26.0));
            let lang_resp = ui.interact(lang_slot, egui::Id::new("lang_switch"), Sense::click());
            if lang_resp.hovered() {
                ui.painter().rect_filled(lang_slot, 5.0, crate::theme::hover());
            }
            ui.painter().text(lang_slot.center(), Align2::CENTER_CENTER, self.lang.code().to_uppercase(), FontId::proportional(13.0), crate::theme::icon_line());
            let popup_id = egui::Id::new("lang_popup");
            if lang_resp.clone().on_hover_text(t!("Sprache wählen")).clicked() {
                ui.memory_mut(|m| m.toggle_popup(popup_id));
            }
            let mut picked = None;
            egui::popup::popup_below_widget(ui, popup_id, &lang_resp, egui::popup::PopupCloseBehavior::CloseOnClick, |ui| {
                ui.set_min_width(110.0);
                for lang in crate::lang::Lang::ALL {
                    if ui.selectable_label(self.lang == lang, lang.native_name()).clicked() {
                        picked = Some(lang);
                    }
                }
            });
            if let Some(lang) = picked {
                self.set_lang(ctx, lang);
            }
            // the layer, centred over the window as in the original (one layer for now)
            if !self.start_page {
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(Rect::from_center_size(full.center(), Vec2::new(230.0, 26.0))), |ui| self.layer_selector(ui));
            }
        });
    }
}
