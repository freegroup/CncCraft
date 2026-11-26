//! Colours and the egui style for the whole app, fully encapsulated.
//!
//! No UI component knows which theme is active. Every colour comes from the active [`Palette`];
//! adding a new theme means writing one more file that returns a `Palette` and registering it in
//! [`THEMES`].

use std::sync::RwLock;

use egui::{Color32, Stroke, Vec2};

mod palette;
mod light;
mod dark;

pub use palette::Palette;

// ---------------------------------------------------------------------------
// The registry: every theme that can be chosen.
// ---------------------------------------------------------------------------

#[allow(dead_code)]
struct ThemeEntry {
    key: &'static str,
    label: &'static str,
    build: fn() -> Palette,
}

const THEMES: &[ThemeEntry] = &[
    ThemeEntry { key: "light", label: "Hell",   build: light::palette },
    ThemeEntry { key: "dark",  label: "Dunkel", build: dark::palette },
];

/// The names the settings dialog shows.
#[allow(dead_code)]
pub fn available() -> Vec<(&'static str, &'static str)> {
    THEMES.iter().map(|t| (t.key, t.label)).collect()
}

// ---------------------------------------------------------------------------
// The active palette, behind a RwLock so any thread can read it.
// ---------------------------------------------------------------------------

static ACTIVE: RwLock<Option<Palette>> = RwLock::new(None);

fn with<R>(f: impl FnOnce(&Palette) -> R) -> R {
    let guard = ACTIVE.read().unwrap();
    f(guard.as_ref().expect("theme::apply() must be called before any colour is read"))
}

// ---------------------------------------------------------------------------
// Backward-compatible constants (sheet colours, same in every theme).
// ---------------------------------------------------------------------------

pub const ACCENT: Color32 = Color32::from_rgb(0xff, 0xc0, 0x00);
pub const ACCENT_DEEP: Color32 = Color32::from_rgb(0xf8, 0xb0, 0x02);
pub const INK: Color32 = Color32::from_rgb(0x1f, 0x29, 0x37);
pub const MUTED: Color32 = Color32::from_rgb(0x6b, 0x72, 0x80);
pub const SELECT: Color32 = Color32::from_rgb(0xe0, 0x20, 0xa0);
pub const SHEET_LABEL: Color32 = Color32::WHITE;
/// The rulers along the board: a light grey a little darker than the white board, in both themes,
/// so they belong to the sheet and the red guide handles stand out on them.
pub const RULER_FILL: Color32 = Color32::from_gray(232);
/// Ticks and numbers on the rulers (dark on the light ruler, in both themes).
pub const RULER_INK: Color32 = Color32::from_gray(70);
pub const SHEET_LABEL_EDGE: Color32 = Color32::from_rgb(0xb4, 0xb8, 0xc0);
pub const CONSTRAINT: Color32 = Color32::from_rgb(0x1e, 0x6f, 0xd9);
pub const PRIMARY: Color32 = Color32::from_rgb(0x2a, 0x6c, 0xf0);

// ---------------------------------------------------------------------------
// Public colour accessors – each reads from the active Palette.
// ---------------------------------------------------------------------------

pub fn canvas()        -> Color32 { with(|p| p.canvas) }
pub fn ink()           -> Color32 { with(|p| p.ink) }
pub fn muted()         -> Color32 { with(|p| p.muted) }
pub fn disabled()      -> Color32 { with(|p| p.disabled) }
pub fn panel()         -> Color32 { with(|p| p.panel) }
pub fn surface()       -> Color32 { with(|p| p.surface) }
pub fn separator()     -> Color32 { with(|p| p.separator) }
pub fn hairline()      -> Color32 { with(|p| p.hairline) }
pub fn outline()       -> Color32 { with(|p| p.outline) }
pub fn hover()         -> Color32 { with(|p| p.hover) }
pub fn row()           -> Color32 { with(|p| p.row) }
pub fn row_active()    -> Color32 { with(|p| p.row_active) }
pub fn selected_fill() -> Color32 { with(|p| p.selected_fill) }
pub fn icon_line()     -> Color32 { with(|p| p.icon_line) }
pub fn icon_top()      -> Color32 { with(|p| p.icon_top) }
pub fn icon_bottom()   -> Color32 { with(|p| p.icon_bottom) }
pub fn icon_back()     -> Color32 { with(|p| p.icon_back) }
pub fn field_bg()      -> Color32 { with(|p| p.field_bg) }
pub fn field_stroke()  -> Color32 { with(|p| p.field_stroke) }
pub fn chip()          -> Color32 { with(|p| p.chip) }
pub fn chip_hover()    -> Color32 { with(|p| p.chip_hover) }
pub fn card()          -> Color32 { with(|p| p.card) }
pub fn card_edge()     -> Color32 { with(|p| p.card_edge) }
pub fn error()         -> Color32 { with(|p| p.error) }
pub fn grip()          -> Color32 { with(|p| p.grip) }
pub fn warning()       -> Color32 { with(|p| p.warning) }
pub fn key_bg()        -> Color32 { with(|p| p.key_bg) }
pub fn layer_active()  -> Color32 { with(|p| p.layer_active) }
pub fn on_accent()     -> Color32 { with(|p| p.on_accent) }
pub fn spline_arm()    -> Color32 { with(|p| p.spline_arm) }
pub fn halo()          -> Color32 { with(|p| p.halo) }
pub fn gcode_cmd()         -> Color32 { with(|p| p.gcode_cmd) }
pub fn gcode_num()         -> Color32 { with(|p| p.gcode_num) }
pub fn gcode_comment()     -> Color32 { with(|p| p.gcode_comment) }
pub fn gcode_placeholder() -> Color32 { with(|p| p.gcode_placeholder) }
pub fn gcode_text()        -> Color32 { with(|p| p.gcode_text) }
pub fn sim_rapid()     -> Color32 { with(|p| p.sim_rapid) }
pub fn sim_plunge()    -> Color32 { with(|p| p.sim_plunge) }
pub fn board_desk()    -> Color32 { with(|p| p.board_desk) }
/// The red construction lines dragged off the rulers.
pub fn guide()         -> Color32 { with(|p| p.guide) }
/// The alignment lines shown while a point snaps to existing geometry.
pub fn snap_guide()    -> Color32 { with(|p| p.snap_guide) }

// ---------------------------------------------------------------------------
// Choosing and applying a theme.
// ---------------------------------------------------------------------------

/// What the person chose: "System", "Light" or "Dark".
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

    pub fn dark(self, system: Option<egui::Theme>) -> bool {
        match self {
            Choice::Light => false,
            Choice::Dark => true,
            Choice::System => system == Some(egui::Theme::Dark),
        }
    }
}

/// Activate the theme for `dark` and configure egui's visuals and style. Call at start-up and
/// whenever the choice changes.
pub fn apply(ctx: &egui::Context, dark: bool) {
    let key = if dark { "dark" } else { "light" };
    let pal = THEMES.iter().find(|t| t.key == key).map(|t| (t.build)()).unwrap_or_else(light::palette);
    apply_palette(ctx, pal);
}

fn apply_palette(ctx: &egui::Context, pal: Palette) {
    let dark = pal.is_dark;
    ctx.set_theme(if dark { egui::Theme::Dark } else { egui::Theme::Light });
    let mut v = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = pal.canvas;
    v.window_fill = pal.surface;
    v.extreme_bg_color = pal.extreme_bg;
    v.selection.bg_fill = pal.selection_bg;
    v.selection.stroke = Stroke::new(1.0_f32, pal.ink);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, pal.accent_deep);
    v.widgets.active.bg_fill = pal.active_bg;
    v.widgets.active.weak_bg_fill = pal.active_bg;
    v.widgets.inactive.weak_bg_fill = pal.inactive_weak_bg;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, pal.outline);
    v.widgets.hovered.weak_bg_fill = pal.hover;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, pal.separator);
    v.window_stroke = Stroke::new(1.0_f32, pal.outline);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, pal.ink);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, pal.ink);
    let r = egui::Rounding::same(4.0);
    v.window_rounding = egui::Rounding::same(5.0);
    v.menu_rounding = egui::Rounding::same(5.0);
    v.widgets.noninteractive.rounding = r;
    v.widgets.inactive.rounding = r;
    v.widgets.hovered.rounding = r;
    v.widgets.active.rounding = r;
    v.widgets.open.rounding = r;
    let soft = egui::epaint::Shadow {
        offset: Vec2::new(0.0, 6.0),
        blur: 18.0,
        spread: 0.0,
        color: Color32::from_black_alpha(pal.shadow_alpha),
    };
    v.window_shadow = soft;
    v.popup_shadow = soft;
    ctx.set_visuals(v);
    let mut st = (*ctx.style()).clone();
    st.spacing.item_spacing = Vec2::new(8.0, 6.0);
    st.spacing.button_padding = Vec2::new(10.0, 4.0);
    st.spacing.interact_size.y = 26.0;
    use egui::{FontFamily, FontId, TextStyle};
    st.text_styles.insert(TextStyle::Body, FontId::new(13.5, FontFamily::Proportional));
    st.text_styles.insert(TextStyle::Button, FontId::new(13.5, FontFamily::Proportional));
    st.text_styles.insert(TextStyle::Small, FontId::new(11.5, FontFamily::Proportional));
    st.text_styles.insert(TextStyle::Heading, FontId::new(15.5, FontFamily::Proportional));
    st.text_styles.insert(TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace));
    ctx.set_style(st);

    *ACTIVE.write().unwrap() = Some(pal);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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
        ("Text auf dem gelben Hauptknopf", INK, ACCENT),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    pub static THEME_LOCK: Mutex<()> = Mutex::new(());

    fn install(pal: Palette) {
        *ACTIVE.write().unwrap() = Some(pal);
    }

    #[test]
    fn text_is_readable_on_every_surface_in_both_themes() {
        let _guard = THEME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut bad = vec![];
        for (label, build) in [("hell", light::palette as fn() -> Palette), ("dunkel", dark::palette)] {
            install(build());
            for (name, fg, bg) in text_pairs() {
                let c = contrast(fg, bg);
                if c < 4.5 {
                    bad.push(format!("{name} ({label}): Kontrast {c:.1}"));
                }
            }
        }
        install(light::palette());
        assert!(bad.is_empty(), "nicht lesbar: {bad:?}");
    }

    #[test]
    fn contrast_of_white_on_white_is_one() {
        assert!((contrast(Color32::WHITE, Color32::WHITE) - 1.0).abs() < 1e-6);
        assert!(contrast(Color32::BLACK, Color32::WHITE) > 20.0);
    }
}
