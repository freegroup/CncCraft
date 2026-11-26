//! Captions over the main window, like the text in TikTok or tutorial videos: a line (or a few,
//! wrapped) on a rounded box, placed top/centre/bottom and left/centre/right, fading in and out.
//! Two independent slots: a **title** (`text_title`, set once and kept — e.g. the app's name) and
//! a **subtitle** (`text_subtitle`, replaced again and again to say what is happening). Each is one
//! caption; an empty text takes its slot away.

use super::App;
use egui::{Color32, Pos2, Rect, Vec2};

/// How long a caption takes to fade in and out (seconds).
const FADE: f64 = 0.25;
/// Space between a caption and the window's edge (points); the bottom one keeps clear of the
/// status bar.
const EDGE: Vec2 = Vec2::new(40.0, 64.0);

/// Which caption slot: the kept title or the changing subtitle. Each has its own place on screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Slot {
    Title,
    Subtitle,
}

/// Where a caption sits vertically.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Vertical {
    Top,
    Center,
    Bottom,
}

/// Where a caption sits horizontally (and how its lines are aligned).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Horizontal {
    Left,
    Center,
    Right,
}

/// A caption on screen.
pub(super) struct TextOverlay {
    text: String,
    /// When it was set (egui time, seconds).
    since: f64,
    /// How long it shows (seconds); 0 keeps it until it is replaced or cleared.
    duration: f64,
    opacity: f32,
    color: Color32,
    background: Color32,
    size: f32,
    vertical: Vertical,
    horizontal: Horizontal,
}

impl App {
    /// Set one caption slot (title or subtitle). Colours are `#RRGGBB`, `#RRGGBBAA` or `r,g,b[,a]`.
    /// An empty `text` takes that slot away. Returns {"shown", "duration"}.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn agent_set_overlay(&mut self, ctx: &egui::Context, slot: Slot, text: &str, duration: Option<f64>, opacity: Option<f64>, color: Option<&str>, background: Option<&str>, size: Option<f64>, vertical: Option<&str>, horizontal: Option<&str>) -> Result<String, String> {
        let dst = match slot {
            Slot::Title => &mut self.title_overlay,
            Slot::Subtitle => &mut self.text_overlay,
        };
        if text.trim().is_empty() {
            *dst = None;
            return Ok(serde_json::json!({"shown": false}).to_string());
        }
        let colour = |s: Option<&str>, default: Color32, what: &str| match s {
            None => Ok(default),
            Some(s) => parse_color(s).ok_or_else(|| format!("{what} must be #RRGGBB, #RRGGBBAA or r,g,b,a (0-255), not \"{s}\"")),
        };
        // the title stands at the top by default; the subtitle at the bottom
        let default_v = if slot == Slot::Title { "top" } else { "bottom" };
        let vertical = match vertical.unwrap_or(default_v) {
            "top" => Vertical::Top,
            "center" | "centre" | "middle" => Vertical::Center,
            "bottom" => Vertical::Bottom,
            other => return Err(format!("vertical must be top, center or bottom, not \"{other}\"")),
        };
        let horizontal = match horizontal.unwrap_or("center") {
            "left" => Horizontal::Left,
            "center" | "centre" | "middle" => Horizontal::Center,
            "right" => Horizontal::Right,
            other => return Err(format!("horizontal must be left, center or right, not \"{other}\"")),
        };
        // the title is kept (no time limit) by default and a touch larger
        let duration = duration.unwrap_or(if slot == Slot::Title { 0.0 } else { 3.0 }).max(0.0);
        let size = size.unwrap_or(if slot == Slot::Title { 40.0 } else { 28.0 }).clamp(8.0, 160.0) as f32;
        *dst = Some(TextOverlay {
            text: text.to_string(),
            since: ctx.input(|i| i.time),
            duration,
            opacity: opacity.unwrap_or(1.0).clamp(0.0, 1.0) as f32,
            color: colour(color, Color32::WHITE, "color")?,
            background: colour(background, Color32::from_rgba_unmultiplied(0, 0, 0, 180), "bg_color")?,
            size,
            vertical,
            horizontal,
        });
        ctx.request_repaint();
        Ok(serde_json::json!({"shown": true, "duration": duration}).to_string())
    }

    /// Take both captions away (title and subtitle), e.g. when a script ends.
    pub(crate) fn agent_clear_overlays(&mut self) -> Result<String, String> {
        self.title_overlay = None;
        self.text_overlay = None;
        Ok(serde_json::json!({"cleared": true}).to_string())
    }

    /// Draw both captions (title and subtitle), dropping each once its time is up.
    pub(super) fn draw_text_overlay(&mut self, ctx: &egui::Context) {
        self.draw_one_overlay(ctx, Slot::Title);
        self.draw_one_overlay(ctx, Slot::Subtitle);
    }

    /// Draw one caption slot.
    fn draw_one_overlay(&mut self, ctx: &egui::Context, slot: Slot) {
        let (o, id) = match slot {
            Slot::Title => (&self.title_overlay, "cnccraft_title_overlay"),
            Slot::Subtitle => (&self.text_overlay, "cnccraft_text_overlay"),
        };
        let Some(o) = o else { return };
        let now = ctx.input(|i| i.time);
        let age = now - o.since;
        if o.duration > 0.0 && age >= o.duration {
            match slot {
                Slot::Title => self.title_overlay = None,
                Slot::Subtitle => self.text_overlay = None,
            }
            return;
        }
        // fade in at the start and out at the end
        let mut fade = (age / FADE).min(1.0);
        if o.duration > 0.0 {
            fade = fade.min((o.duration - age) / FADE);
        }
        let alpha = (fade.clamp(0.0, 1.0) as f32) * o.opacity;
        let screen = ctx.screen_rect();
        let font = egui::FontId::proportional(o.size);
        let mut job = egui::text::LayoutJob::simple(o.text.clone(), font, o.color.gamma_multiply(alpha), screen.width() * 0.8);
        job.halign = match o.horizontal {
            Horizontal::Left => egui::Align::LEFT,
            Horizontal::Center => egui::Align::Center,
            Horizontal::Right => egui::Align::RIGHT,
        };
        let galley = ctx.fonts(|f| f.layout_job(job));
        let pad = Vec2::new(o.size * 0.6, o.size * 0.35);
        let size = galley.rect.size() + pad * 2.0;
        let area = screen.shrink2(EDGE);
        let x = match o.horizontal {
            Horizontal::Left => area.left(),
            Horizontal::Center => area.center().x - size.x / 2.0,
            Horizontal::Right => area.right() - size.x,
        };
        let y = match o.vertical {
            Vertical::Top => area.top(),
            Vertical::Center => area.center().y - size.y / 2.0,
            Vertical::Bottom => area.bottom() - size.y,
        };
        let card = Rect::from_min_size(Pos2::new(x, y), size);
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new(id)));
        painter.rect_filled(card, o.size * 0.4, o.background.gamma_multiply(alpha));
        // the galley's rows may start left of its origin (centred or right-aligned text): place it
        // by its own rect so it sits inside the card's padding
        painter.galley(card.min + pad - galley.rect.min.to_vec2(), galley, o.color.gamma_multiply(alpha));
        ctx.request_repaint();
    }
}

/// A colour from `#RRGGBB`, `#RRGGBBAA` or `r,g,b` / `r,g,b,a` (0–255), alpha unmultiplied.
pub(super) fn parse_color(s: &str) -> Option<Color32> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        let byte = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
        return match hex.len() {
            6 => Some(Color32::from_rgb(byte(0)?, byte(2)?, byte(4)?)),
            8 => Some(Color32::from_rgba_unmultiplied(byte(0)?, byte(2)?, byte(4)?, byte(6)?)),
            _ => None,
        };
    }
    let parts: Vec<u8> = s.split(',').map(|p| p.trim().parse::<u8>()).collect::<Result<_, _>>().ok()?;
    match parts[..] {
        [r, g, b] => Some(Color32::from_rgb(r, g, b)),
        [r, g, b, a] => Some(Color32::from_rgba_unmultiplied(r, g, b, a)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_are_read_in_hex_and_as_numbers() {
        assert_eq!(parse_color("#ff0000"), Some(Color32::from_rgb(255, 0, 0)));
        assert_eq!(parse_color("#00000080"), Some(Color32::from_rgba_unmultiplied(0, 0, 0, 128)));
        assert_eq!(parse_color("255, 255, 255, 200"), Some(Color32::from_rgba_unmultiplied(255, 255, 255, 200)));
        assert_eq!(parse_color("10,20,30"), Some(Color32::from_rgb(10, 20, 30)));
        assert_eq!(parse_color("#12345"), None);
        assert_eq!(parse_color("red"), None);
        assert_eq!(parse_color("300,0,0"), None);
    }
}
