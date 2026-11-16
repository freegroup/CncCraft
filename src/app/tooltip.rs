//! One tooltip for every control: a header and a text, and optionally an icon. With an icon the
//! tip has two columns, the icon on the left and header and text on the right.
//!
//! ```ignore
//! let tip = Tip::new(t!("Raster"), t!("Hin und her in Bahnen")).icon(Icon::Pocket);
//! ui.radio_value(&mut v, 1, t!("Raster")).tip(&tip);
//! egui::ComboBox::from_id_salt("x").show_ui(ui, |ui| { … }).response.tip(&tip);
//! ui.label(t!("Zustellung")).tip(&Tip::new(t!("Zustellung"), t!("So tief je Durchgang")));
//! ```
//! A disabled control shows its tip too, so it can say why it can't be used: like the original's
//! control descriptions, which carry a `tooltip` and a separate `disabledTooltip` (Spark
//! `0x10037ddb4`, keys from `0x1003ad2b0` and `0x1003ad2f0`), a tip can have its own text for
//! that (`when_disabled`).

use super::icons::{Colours, Pen};
use super::Icon;
use crate::theme::{ink, muted};
use egui::{Response, Sense, Vec2};

/// How wide a tip's text may get before it wraps (points).
const MAX_WIDTH: f32 = 300.0;
/// The square the icon is drawn in at normal size, and the gap to the text beside it.
const ICON_BOX: f32 = 38.0;
const ICON_GAP: f32 = 10.0;
/// How much bigger a `big` icon is drawn (a 38 pt icon becomes about 84 pt).
const BIG: f32 = 2.2;
/// The colour of a keyboard shortcut badge: warm grey background.
const KEY_BG: egui::Color32 = egui::Color32::from_rgb(0x58, 0x55, 0x52);
/// Padding inside a key-cap badge.
const KEY_PAD: f32 = 3.0;

const REASON: egui::Color32 = egui::Color32::from_rgb(0xc0, 0x50, 0x00);

/// A tooltip: header, text, an optional icon, an optional keyboard shortcut badge, and an optional
/// text for when the control is disabled.
#[derive(Clone)]
pub(in crate::app) struct Tip {
    header: String,
    text: String,
    icon: Option<Icon>,
    scale: f32,
    shortcut: Option<String>,
    disabled_text: Option<String>,
}

impl Tip {
    pub fn new(header: impl Into<String>, text: impl Into<String>) -> Tip {
        Tip { header: header.into(), text: text.into(), icon: None, scale: 1.0, shortcut: None, disabled_text: None }
    }

    /// Show `icon` on the left, the header and text to the right of it.
    pub fn icon(mut self, icon: Icon) -> Tip {
        self.icon = Some(icon);
        self
    }

    /// Draw the icon large (for tips that explain what a picture means).
    pub fn big(mut self) -> Tip {
        self.scale = BIG;
        self
    }

    /// A keyboard shortcut shown as a small key-cap badge to the right of the header.
    pub fn shortcut(mut self, key: impl Into<String>) -> Tip {
        self.shortcut = Some(key.into());
        self
    }

    /// Why the control can't be used right now: on a disabled control it shows under the tip,
    /// across both columns.
    pub fn when_disabled(mut self, reason: impl Into<String>) -> Tip {
        self.disabled_text = Some(reason.into());
        self
    }

    /// Draw the tip's contents into `ui` (the tooltip frame comes from egui).
    pub fn show(&self, ui: &mut egui::Ui) {
        self.show_with(ui, None);
    }

    /// Draw it as it reads on a disabled control: the same tip, and the reason below it.
    pub fn show_disabled(&self, ui: &mut egui::Ui) {
        self.show_with(ui, self.disabled_text.as_deref());
    }

    fn show_with(&self, ui: &mut egui::Ui, reason: Option<&str>) {
        ui.ctx().data_mut(|d| d.insert_temp(shown_id(), (self.header.clone(), self.icon.is_some(), self.shortcut.clone(), reason.map(str::to_string))));
        let icon_box = ICON_BOX * self.scale;
        let width = MAX_WIDTH + self.icon.map_or(0.0, |_| icon_box + ICON_GAP);
        ui.set_max_width(width);
        ui.horizontal_top(|ui| {
            if let Some(icon) = self.icon {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(icon_box), Sense::hover());
                Pen { p: ui.painter(), c: rect.center(), k: Colours::normal(), scale: self.scale }.draw(icon);
                ui.add_space(ICON_GAP - ui.spacing().item_spacing.x);
            }
            ui.vertical(|ui| {
                ui.set_max_width(MAX_WIDTH);
                ui.spacing_mut().item_spacing.y = 3.0;
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&self.header).strong().color(ink()));
                    if let Some(key) = &self.shortcut {
                        let font = egui::FontId::proportional(11.0);
                        let galley = ui.painter().layout_no_wrap(key.clone(), font.clone(), egui::Color32::WHITE);
                        let w = galley.size().x + KEY_PAD * 2.0;
                        let h = galley.size().y + KEY_PAD * 1.4;
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(w, h), Sense::hover());
                        ui.painter().rect_filled(rect, 3.0, KEY_BG);
                        let text_pos = rect.left_top() + Vec2::new(KEY_PAD, KEY_PAD * 0.7);
                        ui.painter().galley(text_pos, galley, egui::Color32::WHITE);
                    }
                });
                if !self.text.is_empty() {
                    ui.add(egui::Label::new(egui::RichText::new(&self.text).color(muted())).wrap());
                }
            });
        });
        if let Some(reason) = reason {
            ui.add_space(2.0);
            ui.separator();
            ui.add(egui::Label::new(egui::RichText::new(reason).color(REASON)).wrap());
        }
    }
}

/// Where the tip drawn last is noted for the self-test: its header, whether it had an icon, the
/// shortcut badge text, and the reason line when it was shown on a disabled control.
pub(in crate::app) type Shown = (String, bool, Option<String>, Option<String>);

/// The key of [`Shown`] in egui's temporary data.
pub(in crate::app) fn shown_id() -> egui::Id {
    egui::Id::new("cnccraft_tip_shown")
}

/// Gives any control (button, combo box, radio button, label, …) a [`Tip`].
pub(in crate::app) trait WithTip {
    fn tip(self, tip: &Tip) -> Self;
}

impl WithTip for Response {
    fn tip(self, tip: &Tip) -> Response {
        self.on_hover_ui(|ui| tip.show(ui)).on_disabled_hover_ui(|ui| tip.show_disabled(ui))
    }
}
