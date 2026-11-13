//! Colours and the egui style for the whole app, in one place.

use std::sync::atomic::{AtomicBool, Ordering};

use egui::{Color32, Stroke, Vec2};

pub const ACCENT: Color32 = Color32::from_rgb(0xff, 0xc0, 0x00);
pub const ACCENT_DEEP: Color32 = Color32::from_rgb(0xf8, 0xb0, 0x02);
/// INK and MUTED are the colours for drawing on the sheet itself (the board stays light in both
/// themes, like a sheet of material); `ink()`, `muted()` and the other functions below follow the
/// theme and are for the interface around it.
pub const INK: Color32 = Color32::from_rgb(0x1f, 0x29, 0x37);
pub const MUTED: Color32 = Color32::from_rgb(0x6b, 0x72, 0x80);
pub const SELECT: Color32 = Color32::from_rgb(0xe0, 0x20, 0xa0);
/// The sheet is white in both themes, so what is written on it (the values of the dimensions and
/// the constraint symbols) sits on white with a light grey edge whichever theme is on.
pub const SHEET_LABEL: Color32 = Color32::WHITE;
pub const SHEET_LABEL_EDGE: Color32 = Color32::from_rgb(0xb4, 0xb8, 0xc0);
/// Sizes tied to a variable or formula (constraints): sides, handles and dimension lines.
pub const CONSTRAINT: Color32 = Color32::from_rgb(0x1e, 0x6f, 0xd9);
pub const PRIMARY: Color32 = Color32::from_rgb(0x2a, 0x6c, 0xf0);

static DARK: AtomicBool = AtomicBool::new(false);

pub fn is_dark() -> bool {
    DARK.load(Ordering::Relaxed)
}

pub fn set_dark(on: bool) {
    DARK.store(on, Ordering::Relaxed);
}

fn pick(light: [u8; 3], dark: [u8; 3]) -> Color32 {
    let c = if is_dark() { dark } else { light };
    Color32::from_rgb(c[0], c[1], c[2])
}

/// The grey round the sheet (and the welcome screen).
pub fn canvas() -> Color32 { pick([0xf2, 0xf2, 0xf2], [0x17, 0x18, 0x1b]) }
/// Text and icons of the interface.
pub fn ink() -> Color32 { pick([0x1f, 0x29, 0x37], [0xe9, 0xeb, 0xef]) }
pub fn muted() -> Color32 { pick([0x63, 0x6a, 0x77], [0xa9, 0xaf, 0xbb]) }
/// Icons that cannot be used right now.
pub fn disabled() -> Color32 { pick([0xb9, 0xb9, 0xb9], [0x5b, 0x5e, 0x68]) }
/// The side panels and the tool column: a step behind the cards.
pub fn panel() -> Color32 { pick([0xf7, 0xf7, 0xf9], [0x20, 0x22, 0x26]) }
/// Cards, the title bar, entries and menus: the surface closest to the person.
pub fn surface() -> Color32 { pick([0xff, 0xff, 0xff], [0x2a, 0x2c, 0x32]) }
/// The lightest line (between panels and cards), and a stronger one (outlines of fields).
pub fn separator() -> Color32 { pick([0xe4, 0xe5, 0xe9], [0x37, 0x3a, 0x41]) }
pub fn hairline() -> Color32 { pick([0xe6, 0xe8, 0xeb], [0x37, 0x3a, 0x41]) }
pub fn outline() -> Color32 { pick([0xd5, 0xd7, 0xdd], [0x62, 0x66, 0x72]) }
/// A button under the pointer, and a list row at rest / the one in use.
pub fn hover() -> Color32 { pick([0xec, 0xed, 0xf1], [0x37, 0x3a, 0x42]) }
pub fn row() -> Color32 { pick([0xf2, 0xf2, 0xf4], [0x2a, 0x2c, 0x32]) }
pub fn row_active() -> Color32 { pick([0xe1, 0xe1, 0xe4], [0x3a, 0x3d, 0x45]) }
/// A selected button: the accent, quietly.
pub fn selected_fill() -> Color32 { pick([0xff, 0xec, 0xb0], [0x5a, 0x47, 0x12]) }
/// The colours of the icons: a mid-blue line, a sky-blue gradient (light on top, deeper below)
/// and the plain fill of a part in front.
pub fn icon_line() -> Color32 { pick([0x2a, 0x78, 0xd6], [0x6c, 0xb8, 0xff]) }
pub fn icon_top() -> Color32 { pick([0x8d, 0xe6, 0xff], [0x3e, 0x8f, 0xd4]) }
pub fn icon_bottom() -> Color32 { pick([0x3e, 0xaf, 0xf6], [0x24, 0x62, 0xa2]) }
pub fn icon_back() -> Color32 { pick([0xff, 0xff, 0xff], [0x1d, 0x24, 0x30]) }
/// A text field's background and edge, and the grey chips and buttons of dialogs (and under the pointer).
pub fn field_bg() -> Color32 { pick([0xfa, 0xfa, 0xfb], [0x18, 0x19, 0x1d]) }
pub fn field_stroke() -> Color32 { pick([0xd1, 0xd5, 0xdb], [0x4a, 0x4d, 0x56]) }
pub fn chip() -> Color32 { pick([0xf3, 0xf4, 0xf6], [0x3b, 0x3e, 0x47]) }
pub fn chip_hover() -> Color32 { pick([0xe5, 0xe7, 0xeb], [0x4c, 0x50, 0x5a]) }
/// A card on the start page and its edge (more contrast than a plain surface).
pub fn card() -> Color32 { pick([0xff, 0xff, 0xff], [0x30, 0x32, 0x39]) }
pub fn card_edge() -> Color32 { pick([0xe6, 0xe8, 0xeb], [0x64, 0x68, 0x74]) }
/// The red of an error text, light enough on dark surfaces.
pub fn error() -> Color32 { pick([0xc0, 0x30, 0x30], [0xff, 0x8f, 0x8f]) }

pub fn grip() -> Color32 { pick([0xaa, 0xaa, 0xaa], [0x70, 0x74, 0x7e]) }

/// Apply the theme (light or dark, as `set_dark` says); called at start-up and when it is switched.
pub fn apply(ctx: &egui::Context) {
    // the theme is ours, not the system's: stop egui from following the operating system
    ctx.set_theme(if is_dark() { egui::Theme::Dark } else { egui::Theme::Light });
    let mut v = if is_dark() { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = canvas();
    v.window_fill = surface();
    v.extreme_bg_color = if is_dark() { Color32::from_rgb(0x1a, 0x1b, 0x1f) } else { Color32::WHITE };
    v.selection.bg_fill = if is_dark() { Color32::from_rgb(0x6b, 0x55, 0x10) } else { Color32::from_rgb(0xff, 0xe0, 0x80) };
    v.selection.stroke = Stroke::new(1.0_f32, ink());
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT_DEEP);
    v.widgets.active.bg_fill = ACCENT;
    // buttons stand out from the surface they sit on: white on a light one, a lighter grey on a dark one
    v.widgets.inactive.weak_bg_fill = if is_dark() { chip() } else { surface() };
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, outline());
    v.widgets.hovered.weak_bg_fill = hover();
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, separator());
    v.window_stroke = Stroke::new(1.0_f32, outline());
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, ink());
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, ink());
    // (egui writes strong text in the "active" text colour, so that colour must stay light in the
    // dark theme; a pressed button is therefore filled with a dark grey there, not the yellow)
    if is_dark() {
        v.widgets.active.bg_fill = Color32::from_rgb(0x4a, 0x4d, 0x56);
        v.widgets.active.weak_bg_fill = Color32::from_rgb(0x4a, 0x4d, 0x56);
    }
    // a crisper, quieter look in the spirit of Apple's professional apps: small corner radii,
    // hairlines, soft shadows
    let r = egui::Rounding::same(4.0);
    v.window_rounding = egui::Rounding::same(5.0);
    v.menu_rounding = egui::Rounding::same(5.0);
    v.widgets.noninteractive.rounding = r;
    v.widgets.inactive.rounding = r;
    v.widgets.hovered.rounding = r;
    v.widgets.active.rounding = r;
    v.widgets.open.rounding = r;
    let soft = egui::epaint::Shadow { offset: Vec2::new(0.0, 6.0), blur: 18.0, spread: 0.0, color: Color32::from_black_alpha(if is_dark() { 90 } else { 34 }) };
    v.window_shadow = soft;
    v.popup_shadow = soft;
    ctx.set_visuals(v);
    let mut st = (*ctx.style()).clone();
    st.spacing.item_spacing = Vec2::new(8.0, 6.0);
    st.spacing.button_padding = Vec2::new(10.0, 4.0);
    st.spacing.interact_size.y = 26.0;
    // one calm type scale: 13.5 for text and buttons, 11.5 for small print, 15.5 for headings
    use egui::{FontFamily, FontId, TextStyle};
    st.text_styles.insert(TextStyle::Body, FontId::new(13.5, FontFamily::Proportional));
    st.text_styles.insert(TextStyle::Button, FontId::new(13.5, FontFamily::Proportional));
    st.text_styles.insert(TextStyle::Small, FontId::new(11.5, FontFamily::Proportional));
    st.text_styles.insert(TextStyle::Heading, FontId::new(15.5, FontFamily::Proportional));
    st.text_styles.insert(TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace));
    ctx.set_style(st);
}

/// What the person chose in the original's Appearance settings: "System", "Light" or "Dark"
/// (the choice list of `sub_10018da14`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Choice {
    System,
    Light,
    Dark,
}

impl Choice {
    pub fn key(self) -> &'static str {
        match self {
            Choice::System => "system",
            Choice::Light => "light",
            Choice::Dark => "dark",
        }
    }

    pub fn from_key(s: &str) -> Choice {
        match s {
            "light" => Choice::Light,
            "dark" => Choice::Dark,
            _ => Choice::System,
        }
    }

    /// Whether the dark colours are used, given what the operating system prefers.
    pub fn dark(self, system: Option<egui::Theme>) -> bool {
        match self {
            Choice::Light => false,
            Choice::Dark => true,
            Choice::System => system == Some(egui::Theme::Dark),
        }
    }
}

/// How much two colours differ in brightness (WCAG contrast ratio, 1 to 21): text needs 4.5.
#[cfg(test)]
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let lum = |c: Color32| {
        let f = |v: u8| {
            let v = v as f32 / 255.0;
            if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
    };
    let (la, lb) = (lum(a), lum(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// The text-on-background pairs the dialogs and panels use, with a name each.
#[cfg(test)]
pub fn text_pairs() -> Vec<(&'static str, Color32, Color32)> {
    vec![
        ("Text auf Fenster", ink(), surface()),
        ("Hinweistext auf Fenster", muted(), surface()),
        ("Text auf Panel", ink(), panel()),
        ("Hinweistext auf Panel", muted(), panel()),
        ("Text im Eingabefeld", ink(), field_bg()),
        ("Fehler im Eingabefeld", error(), field_bg()),
        ("Text auf Ergebnis-Chip", ink(), chip()),
        ("Fehler auf Ergebnis-Chip", error(), chip()),
        ("Hinweis auf Chip", muted(), chip()),
        ("Text auf Knopf", ink(), chip()),
        ("Text auf Knopf unter dem Zeiger", ink(), chip_hover()),
        ("Text auf Listenzeile", ink(), row()),
        ("Text auf Karte der Startseite", ink(), card()),
        ("Hinweis auf Karte der Startseite", muted(), card()),
        ("Text auf gewählter Listenzeile", ink(), row_active()),
        ("Text auf gewähltem Knopf", ink(), selected_fill()),
        // the main button is yellow in both themes: its text is the dark ink in both
        ("Text auf dem gelben Hauptknopf", INK, ACCENT),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// The theme is a global switch: tests that flip it take turns.
    pub static THEME_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn text_is_readable_on_every_surface_in_both_themes() {
        let _guard = THEME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let before = is_dark();
        let mut bad = vec![];
        for dark in [false, true] {
            set_dark(dark);
            for (name, fg, bg) in text_pairs() {
                let c = contrast(fg, bg);
                if c < 4.5 {
                    bad.push(format!("{} ({}): Kontrast {c:.1}", name, if dark { "dunkel" } else { "hell" }));
                }
            }
        }
        set_dark(before);
        assert!(bad.is_empty(), "nicht lesbar: {bad:?}");
    }

    #[test]
    fn contrast_of_white_on_white_is_one() {
        assert!((contrast(Color32::WHITE, Color32::WHITE) - 1.0).abs() < 1e-6);
        assert!(contrast(Color32::BLACK, Color32::WHITE) > 20.0);
    }
}
