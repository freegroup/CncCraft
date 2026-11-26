//! The demo engine: a virtual cursor and an action queue that drive the app the way a person
//! would, for screen recordings. Everything an [`crate::app::mcp`] tool asks for becomes a queue of
//! primitive [`Step`]s (move, press, release, type, key, wait). Each frame [`Demo::feed`] advances
//! the queue and pushes the matching synthetic `egui` events into the raw input — the same path the
//! self-test uses — while [`Demo::draw`] paints a cursor on top so every move, button press and
//! long press is visible in the recording.
//!
//! The cursor is *drawn*, not the real OS pointer: the app is in full control of where it is and
//! what it shows, and the actions still happen through event injection, so picture and effect
//! always agree.

use egui::{Color32, Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use std::collections::VecDeque;

/// How many frames a pointer move is spread over per 100 screen points, so the cursor glides
/// instead of jumping (about 0.7 s at 60 fps for a long move — a calm, readable speed).
const MOVE_FRAMES_PER_100PX: f32 = 22.0;

/// One primitive the queue carries out, one at a time.
#[derive(Clone, Debug)]
pub enum Step {
    /// Glide the cursor to this screen point over a number of frames from the distance.
    MoveTo(Pos2),
    /// Press a mouse button at the current cursor position.
    Press(PointerButton),
    /// Release a mouse button at the current cursor position.
    Release(PointerButton),
    /// Hold the left button down at the cursor for `frames` (a long press), showing a ring.
    Hold(u32),
    /// Type text, one character at a time.
    Type(String),
    /// Press and release a key with modifiers.
    Key(Key, Modifiers),
    /// Hold these modifiers on the following clicks (for a shift-click); `Modifiers::default()` lets
    /// go. Shown as keycaps while held.
    Modifiers(Modifiers),
    /// Wait this many frames (a beat for the viewer).
    Wait(u32),
}

/// The virtual cursor and the queue of steps it works through.
pub struct Demo {
    /// Whether the cursor is shown and the queue is being worked. Off by default.
    pub on: bool,
    /// Where the cursor is now, in screen points.
    pub pos: Pos2,
    /// A move in progress: (from, to, total frames, frames done).
    moving: Option<(Pos2, Pos2, u32, u32)>,
    /// The mouse button currently held down (for the pressed look), if any.
    down: Option<PointerButton>,
    /// A long press in progress: (frames total, frames done), for the ring.
    holding: Option<(u32, u32)>,
    /// The steps still to carry out.
    queue: VecDeque<Step>,
    /// Frames left to wait before the next step (from `Wait` or a click hold).
    wait: u32,
    /// Modifiers held on the clicks that follow (a shift-click), until set back to default.
    mods: Modifiers,
    /// The keys last pressed, shown as keycaps beside the cursor for a moment: (caps, frames left).
    keys: Option<(Vec<String>, u32)>,
    /// The replay speed: 1 is normal, 2 twice as fast, 0.5 half as fast. It scales the glides and
    /// the pauses; a long press is never made shorter than the app needs to see it as one.
    speed: f32,
}

/// How long pressed keys stay on screen as keycaps (frames, about a second).
const KEYCAP_FRAMES: u32 = 60;

/// The slowest and fastest replay speed.
pub const SPEED_RANGE: std::ops::RangeInclusive<f32> = 0.25..=4.0;

impl Default for Demo {
    fn default() -> Self {
        Demo { on: false, pos: Pos2::new(40.0, 40.0), moving: None, down: None, holding: None, queue: VecDeque::new(), wait: 0, mods: Modifiers::default(), keys: None, speed: 1.0 }
    }
}

impl Demo {
    /// Add steps to the end of the queue (and turn the cursor on, since something is to be shown).
    pub fn push(&mut self, steps: impl IntoIterator<Item = Step>) {
        self.on = true;
        self.queue.extend(steps);
    }

    /// Whether anything is still playing (a move, a hold, a wait or queued steps).
    pub fn busy(&self) -> bool {
        self.moving.is_some() || self.holding.is_some() || self.wait > 0 || !self.queue.is_empty()
    }

    /// The replay speed (1 = normal).
    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// Set the replay speed, kept within [`SPEED_RANGE`]; returns the speed now in use. Steps already
    /// playing keep their pace, the next ones take the new one.
    pub fn set_speed(&mut self, speed: f32) -> f32 {
        self.speed = if speed.is_finite() { speed.clamp(*SPEED_RANGE.start(), *SPEED_RANGE.end()) } else { 1.0 };
        self.speed
    }

    /// `frames` at the replay speed (at least one, so a press and its release never share a frame).
    fn paced(&self, frames: u32) -> u32 {
        ((frames as f32 / self.speed).round() as u32).max(1)
    }

    /// Whether something is still fading on screen (the keycaps), so frames must keep coming; unlike
    /// [`Demo::busy`] it does not hold up the next command.
    pub fn animating(&self) -> bool {
        self.keys.is_some()
    }

    /// Stop and clear everything in progress (the Stop button); the cursor stays where it is.
    pub fn clear(&mut self) {
        self.moving = None;
        self.holding = None;
        self.wait = 0;
        self.queue.clear();
    }

    /// Advance the queue by one frame and push the resulting synthetic events into `raw`. The
    /// cursor position is sent as a `PointerMoved` every frame it changes, so hovers work too.
    pub fn feed(&mut self, raw: &mut egui::RawInput) {
        if !self.on {
            return;
        }
        // held modifiers (a shift-click) apply to every frame, not just the one that set them:
        // egui reads `i.modifiers` on the press frame, which comes after the glide
        raw.modifiers = self.mods;
        // the keycaps fade out on their own
        if let Some((_, left)) = &mut self.keys {
            *left = left.saturating_sub(1);
            if *left == 0 {
                self.keys = None;
            }
        }
        // a move in progress: step the cursor along and report it
        if let Some((from, to, total, done)) = self.moving {
            let done = done + 1;
            let t = (done as f32 / total as f32).min(1.0);
            // ease-in-out so the glide looks natural
            let e = t * t * (3.0 - 2.0 * t);
            self.pos = from + (to - from) * e;
            raw.events.push(Event::PointerMoved(self.pos));
            self.moving = if done >= total { None } else { Some((from, to, total, done)) };
            return;
        }
        // a long press in progress: keep the button down and grow the ring
        if let Some((total, done)) = self.holding {
            let done = done + 1;
            self.holding = if done >= total {
                // release at the end
                raw.events.push(button_event(self.pos, PointerButton::Primary, false, self.mods));
                self.down = None;
                None
            } else {
                Some((total, done))
            };
            return;
        }
        if self.wait > 0 {
            self.wait -= 1;
            return;
        }
        let Some(step) = self.queue.pop_front() else { return };
        match step {
            Step::MoveTo(to) => {
                let dist = (to - self.pos).length();
                let frames = self.paced(((dist / 100.0) * MOVE_FRAMES_PER_100PX).round() as u32);
                self.moving = Some((self.pos, to, frames, 0));
            }
            Step::Press(b) => {
                raw.events.push(button_event(self.pos, b, true, self.mods));
                self.down = Some(b);
            }
            Step::Release(b) => {
                raw.events.push(button_event(self.pos, b, false, self.mods));
                self.down = None;
            }
            Step::Hold(frames) => {
                raw.events.push(button_event(self.pos, PointerButton::Primary, true, self.mods));
                self.down = Some(PointerButton::Primary);
                // slower replays hold longer, faster ones never shorter: the app needs the full hold
                // to see a long press
                self.holding = Some((self.paced(frames).max(frames), 0));
            }
            Step::Type(text) => {
                raw.events.push(Event::Text(text));
            }
            Step::Key(key, modifiers) => {
                raw.events.push(Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers });
                raw.events.push(Event::Key { key, physical_key: None, pressed: false, repeat: false, modifiers });
                self.keys = Some((keycaps(key, modifiers, cfg!(target_os = "macos")), KEYCAP_FRAMES));
            }
            Step::Modifiers(m) => {
                // egui reads modifiers from each event; also tell it they changed, and show them
                raw.modifiers = m;
                self.mods = m;
                self.keys = if m == Modifiers::default() { None } else { Some((modifier_caps(m, cfg!(target_os = "macos")), KEYCAP_FRAMES)) };
            }
            Step::Wait(frames) => self.wait = self.paced(frames),
        }
    }

    /// Draw the cursor over everything, with the pressed button and the long-press ring.
    /// `area` is where the status panel goes in its bottom-right corner: the drawing area, so it
    /// never covers the side panels' controls (the whole window if there is none yet).
    pub fn draw(&self, ctx: &egui::Context, area: egui::Rect) {
        if !self.on {
            return;
        }
        let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Debug, egui::Id::new("cnccraft_virtual_cursor")));
        let at = self.pos;
        // the long-press ring fills up around the cursor tip
        if let Some((total, done)) = self.holding {
            let frac = (done as f32 / total as f32).clamp(0.0, 1.0);
            p.circle_stroke(at, 16.0, egui::Stroke::new(2.0_f32, Color32::from_black_alpha(40)));
            let n = (frac * 48.0).round() as usize;
            let pts: Vec<Pos2> = (0..=n).map(|i| {
                let a = -std::f32::consts::FRAC_PI_2 + (i as f32 / 48.0) * std::f32::consts::TAU;
                at + Vec2::angled(a) * 16.0
            }).collect();
            if pts.len() > 1 {
                p.add(egui::Shape::line(pts, egui::Stroke::new(2.6_f32, crate::theme::ACCENT)));
            }
        }
        // a click ripple while a button is down
        if self.down.is_some() {
            p.circle_filled(at, 13.0, crate::theme::ACCENT.gamma_multiply(0.30));
        }
        cursor_arrow(&p, at);
        // like screen-demo tools, but apart from the cursor: a large mouse in the bottom-right
        // corner shows which button is held, and the keys last pressed appear beside it
        let area = if area.is_positive() { area } else { ctx.screen_rect() };
        self.draw_status(&p, area);
    }

    /// The status panel in the bottom-right corner of `screen`: a large mouse whose held button
    /// lights up (a long press fills its ring), the button's name, and the last keys as keycaps.
    /// Shown whenever the virtual cursor is, so pointer and buttons can be read separately.
    fn draw_status(&self, p: &egui::Painter, screen: egui::Rect) {
        // kept clear of the drawing area's edge and the help bubble in its bottom corner
        const MARGIN: Vec2 = Vec2::new(18.0, 100.0);
        const PAD: f32 = 14.0;
        let mouse = Vec2::new(54.0, 80.0);
        let caps: Vec<String> = self.keys.as_ref().map(|(c, _)| c.clone()).unwrap_or_default();
        let fade = self.keys.as_ref().map_or(1.0, |(_, left)| (*left as f32 / 12.0).min(1.0));
        let caps_w = keycaps_width(p, &caps);
        let name = self.down.map(|b| match b {
            PointerButton::Secondary => t!("Rechte Maustaste"),
            _ => t!("Linke Maustaste"),
        });
        // the card: keycaps on the left, the mouse on the right, the button's name under the mouse
        let inner = Vec2::new(mouse.x + if caps_w > 0.0 { caps_w + 12.0 } else { 0.0 }, mouse.y + 26.0);
        let size = inner + Vec2::splat(2.0 * PAD);
        let card = egui::Rect::from_min_size(screen.right_bottom() - MARGIN - size, size);
        p.rect(card, 12.0, Color32::from_rgba_unmultiplied(0x1f, 0x23, 0x29, 245), egui::Stroke::new(1.0_f32, Color32::from_white_alpha(40)));
        let body = egui::Rect::from_min_size(Pos2::new(card.max.x - PAD - mouse.x, card.min.y + PAD), mouse);
        let held = self.holding.map(|(total, done)| (done as f32 / total as f32).clamp(0.0, 1.0));
        mouse_glyph(p, body, self.down, held);
        let caption = name.unwrap_or_else(|| t!("Maus"));
        let colour = if self.down.is_some() { crate::theme::ACCENT } else { Color32::from_white_alpha(140) };
        p.text(Pos2::new(body.center().x, body.max.y + 16.0), egui::Align2::CENTER_CENTER, caption, egui::FontId::proportional(12.0), colour);
        if caps_w > 0.0 {
            keycap_row(p, Pos2::new(card.min.x + PAD, body.center().y - 12.0), &caps, fade);
        }
    }
}

/// The width a row of keycaps takes (0 when there are none).
fn keycaps_width(p: &egui::Painter, caps: &[String]) -> f32 {
    caps.iter().map(|c| p.layout_no_wrap(c.clone(), egui::FontId::proportional(14.0), Color32::WHITE).size().x.max(14.0) + 14.0 + 4.0).sum::<f32>().max(4.0) - 4.0
}

/// The keycaps for a key with its modifiers, in the platform's way of writing them (⌃⌥⇧⌘ on a Mac,
/// Strg/Alt/Umschalt elsewhere).
fn keycaps(key: Key, m: Modifiers, mac: bool) -> Vec<String> {
    let mut caps = modifier_caps(m, mac);
    caps.push(match key {
        Key::Enter => "↩".to_string(),
        Key::Escape => "Esc".to_string(),
        Key::Tab => "⇥".to_string(),
        Key::Space => t!("Leertaste"),
        Key::Delete => t!("Entf"),
        Key::Backspace => "⌫".to_string(),
        Key::ArrowLeft => "←".to_string(),
        Key::ArrowRight => "→".to_string(),
        Key::ArrowUp => "↑".to_string(),
        Key::ArrowDown => "↓".to_string(),
        k => k.name().to_string(),
    });
    caps
}

/// Just the modifier keycaps held, in the platform's way of writing them (⌃⌥⇧⌘ on a Mac,
/// Strg/Alt/Umschalt elsewhere).
fn modifier_caps(m: Modifiers, mac: bool) -> Vec<String> {
    let mut caps = Vec::new();
    if mac {
        for (on, cap) in [(m.ctrl && !m.command, "⌃"), (m.alt, "⌥"), (m.shift, "⇧"), (m.command || m.mac_cmd, "⌘")] {
            if on {
                caps.push(cap.to_string());
            }
        }
    } else {
        for (on, cap) in [(m.ctrl || m.command, t!("Strg")), (m.alt, "Alt".to_string()), (m.shift, t!("Umschalt"))] {
            if on {
                caps.push(cap);
            }
        }
    }
    caps
}

/// A row of keycaps starting at `at`, faded by `alpha` (1 = solid).
fn keycap_row(p: &egui::Painter, at: Pos2, caps: &[String], alpha: f32) {
    let mut x = at.x;
    for cap in caps {
        let galley = p.layout_no_wrap(cap.clone(), egui::FontId::proportional(14.0), Color32::WHITE.gamma_multiply(alpha));
        let w = galley.size().x.max(14.0) + 14.0;
        let r = egui::Rect::from_min_size(Pos2::new(x, at.y), Vec2::new(w, 24.0));
        p.rect(r, 5.0, Color32::from_rgb(0x26, 0x2a, 0x33).gamma_multiply(alpha * 0.92), egui::Stroke::new(1.0_f32, Color32::WHITE.gamma_multiply(alpha * 0.5)));
        p.galley(r.center() - galley.size() / 2.0, galley, Color32::WHITE);
        x += w + 4.0;
    }
}

/// A large mouse filling `body`: the held button filled in the accent (none when `down` is
/// `None`), and during a long press a bar under it filling up with `held` (0..1).
fn mouse_glyph(p: &egui::Painter, body: egui::Rect, down: Option<PointerButton>, held: Option<f32>) {
    let ink = Color32::from_rgb(0x1c, 0x1c, 0x1c);
    let r = body.width() / 2.0;
    p.rect(body, r, Color32::WHITE, egui::Stroke::new(2.0_f32, ink));
    // the two buttons across the top, split in the middle
    let split = body.min.y + body.height() * 0.42;
    let left = egui::Rect::from_min_max(body.min, Pos2::new(body.center().x, split)).shrink(1.5);
    let right = egui::Rect::from_min_max(Pos2::new(body.center().x, body.min.y), Pos2::new(body.max.x, split)).shrink(1.5);
    match down {
        Some(PointerButton::Primary) => {
            p.rect_filled(left, egui::Rounding { nw: r, ..Default::default() }, crate::theme::ACCENT);
        }
        Some(PointerButton::Secondary) => {
            p.rect_filled(right, egui::Rounding { ne: r, ..Default::default() }, crate::theme::ACCENT);
        }
        _ => {}
    }
    p.line_segment([Pos2::new(body.min.x, split), Pos2::new(body.max.x, split)], egui::Stroke::new(1.6_f32, ink));
    p.line_segment([Pos2::new(body.center().x, body.min.y), Pos2::new(body.center().x, split)], egui::Stroke::new(1.6_f32, ink));
    // the wheel
    let wheel = egui::Rect::from_center_size(Pos2::new(body.center().x, body.min.y + body.height() * 0.24), Vec2::new(5.0, 12.0));
    p.rect(wheel, 2.5, Color32::from_gray(220), egui::Stroke::new(1.2_f32, ink));
    // a long press: a bar under the mouse fills up
    if let Some(frac) = held {
        let track = egui::Rect::from_min_size(Pos2::new(body.min.x, body.max.y + 3.0), Vec2::new(body.width(), 3.0));
        p.rect_filled(track, 1.5, Color32::from_white_alpha(50));
        p.rect_filled(egui::Rect::from_min_size(track.min, Vec2::new(track.width() * frac, track.height())), 1.5, crate::theme::ACCENT);
    }
}

/// A full mouse-button press-or-release event at `pos` (egui needs the move first; the caller sends
/// the move as part of the glide, so here only the button change).
fn button_event(pos: Pos2, button: PointerButton, pressed: bool, modifiers: Modifiers) -> Event {
    Event::PointerButton { pos, button, pressed, modifiers }
}

/// Draw a classic arrow pointer with its tip at `at`.
fn cursor_arrow(p: &egui::Painter, at: Pos2) {
    // the arrow: tip at `at`, pointing up-left, filled white with a dark outline
    let pts = [
        at,
        at + Vec2::new(0.0, 19.0),
        at + Vec2::new(4.6, 14.6),
        at + Vec2::new(8.0, 20.4),
        at + Vec2::new(11.2, 19.0),
        at + Vec2::new(7.8, 13.2),
        at + Vec2::new(13.6, 13.2),
    ];
    let outline = egui::Stroke::new(1.6_f32, Color32::from_rgb(0x1c, 0x1c, 0x1c));
    p.add(egui::Shape::convex_polygon(pts.to_vec(), Color32::WHITE, outline));
}

impl crate::app::App {
    /// The Demo & remote-control window: where the MCP server listens (so a client like Claude can
    /// drive the app), plus a couple of built-in demos that play through the visible cursor right
    /// now, so the feature is easy to find and try.
    pub(in crate::app) fn demo_ui(&mut self, ctx: &egui::Context) {
        use crate::app::dialogs;
        if !self.demo_open {
            return;
        }
        let port = self.mcp.as_ref().map(|m| m.port);
        let mut play: Option<Vec<Step>> = None;
        let was_open = self.demo_open;
        dialogs::window("demo_window", ctx).anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
            ui.set_width(380.0);
            if dialogs::header(ui, &t!("Agentensteuerung (MCP)")) {
                self.demo_open = false;
            }
            ui.label(egui::RichText::new(t!("CncCraft lässt sich vollständig von Agenten bedienen. Jeder Menüeintrag, jeder Knopf und jede Geste ist ein Befehl – eine sichtbare Maus zeigt dabei jede Aktion, damit sich Funktions-Demos aufnehmen lassen.")).small().color(crate::theme::muted()));
            ui.add_space(6.0);
            match port {
                Some(p) => {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(t!("MCP-Server:")).color(crate::theme::ink()));
                        ui.monospace(format!("http://127.0.0.1:{p}/"));
                    });
                    ui.label(egui::RichText::new(t!("Nur über localhost erreichbar, ohne Anmeldung.")).small().color(crate::theme::muted()));
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(t!("%{n} Befehle – jeder Menüeintrag, Knopf und Zeichen- oder Fräsbefehl ist ein eigener Befehl.", n = crate::mcp::tools::catalog().len())).small().color(crate::theme::muted()));
                }
                None => {
                    ui.label(egui::RichText::new(t!("Der MCP-Server konnte nicht gestartet werden.")).color(crate::theme::warning()));
                }
            }
            ui.add_space(8.0);
            ui.label(egui::RichText::new(t!("Zum Ausprobieren – jetzt abspielen:")).color(crate::theme::ink()));
            ui.add_space(4.0);
            let r = ctx.screen_rect();
            let c = r.center();
            if ui.button(t!("▶  Mauszeiger vorführen")).clicked() {
                // glide the cursor in a little tour and do one click in the middle of the window
                play = Some(vec![
                    Step::MoveTo(Pos2::new(r.left() + 60.0, r.top() + 120.0)),
                    Step::Wait(10),
                    Step::MoveTo(Pos2::new(c.x, c.y)),
                    Step::Wait(6),
                    Step::Press(PointerButton::Primary),
                    Step::Wait(8),
                    Step::Release(PointerButton::Primary),
                    Step::Wait(10),
                    Step::MoveTo(Pos2::new(r.right() - 120.0, r.bottom() - 120.0)),
                ]);
            }
            if ui.button(t!("▶  Langen Druck zeigen")).clicked() {
                play = Some(vec![Step::MoveTo(Pos2::new(c.x, c.y)), Step::Wait(6), Step::Hold(40), Step::Wait(8)]);
            }
            if self.demo.busy() && ui.button(t!("⏹  Stopp")).clicked() {
                self.demo.clear();
            }
            dialog_buttons_row(ui, &mut self.demo_open);
        });
        if let Some(steps) = play {
            self.demo.push(steps);
        }
        // closing the window hides the cursor it showed and stops its little demo
        if was_open && !self.demo_open {
            self.demo.clear();
            self.demo.on = false;
        }
    }
}

/// A right-aligned "Close" button row (kept local so demo_ui needs no extra imports).
fn dialog_buttons_row(ui: &mut egui::Ui, open: &mut bool) {
    crate::app::dialogs::dialog_buttons(ui, |ui| {
        if ui.button(t!("Schließen")).clicked() {
            *open = false;
        }
    });
}


#[cfg(test)]
mod tests {
    use super::*;

    fn drain(demo: &mut Demo, frames: usize) -> Vec<Event> {
        let mut out = Vec::new();
        for _ in 0..frames {
            let mut raw = egui::RawInput::default();
            demo.feed(&mut raw);
            out.extend(raw.events);
        }
        out
    }

    #[test]
    fn a_click_glides_the_cursor_then_presses_and_releases() {
        let mut demo = Demo::default();
        demo.pos = Pos2::new(0.0, 0.0);
        demo.push([Step::MoveTo(Pos2::new(200.0, 0.0)), Step::Press(PointerButton::Primary), Step::Wait(3), Step::Release(PointerButton::Primary)]);
        let events = drain(&mut demo, 60);
        // the move produced several PointerMoved events ending at the target (it glides, not jumps)
        let moves: Vec<Pos2> = events.iter().filter_map(|e| if let Event::PointerMoved(p) = e { Some(*p) } else { None }).collect();
        assert!(moves.len() > 3, "the cursor glides over several frames, got {}", moves.len());
        assert!((moves.last().unwrap().x - 200.0).abs() < 0.01, "ends at the target");
        // then a press and a release of the primary button, in that order
        let buttons: Vec<bool> = events.iter().filter_map(|e| if let Event::PointerButton { button: PointerButton::Primary, pressed, .. } = e { Some(*pressed) } else { None }).collect();
        assert_eq!(buttons, vec![true, false], "one press then one release");
        assert!(!demo.busy(), "the queue is empty afterwards");
    }

    #[test]
    fn a_long_press_holds_the_button_down_for_the_whole_hold() {
        let mut demo = Demo::default();
        demo.push([Step::Hold(10)]);
        let events = drain(&mut demo, 20);
        let presses = events.iter().filter(|e| matches!(e, Event::PointerButton { pressed: true, .. })).count();
        let releases = events.iter().filter(|e| matches!(e, Event::PointerButton { pressed: false, .. })).count();
        assert_eq!((presses, releases), (1, 1), "pressed once at the start, released once at the end");
    }

    /// Frames until the queue has played out.
    fn frames_to_finish(demo: &mut Demo) -> usize {
        (1..10_000).find(|_| {
            demo.feed(&mut egui::RawInput::default());
            !demo.busy()
        }).unwrap()
    }

    #[test]
    fn double_speed_plays_a_glide_and_a_pause_in_about_half_the_frames() {
        let steps = || [Step::MoveTo(Pos2::new(400.0, 0.0)), Step::Wait(30)];
        let mut normal = Demo { pos: Pos2::ZERO, ..Demo::default() };
        normal.push(steps());
        let mut fast = Demo { pos: Pos2::ZERO, ..Demo::default() };
        assert_eq!(fast.set_speed(2.0), 2.0);
        fast.push(steps());
        let (n, f) = (frames_to_finish(&mut normal), frames_to_finish(&mut fast));
        assert!((f as f32 - n as f32 / 2.0).abs() <= 3.0, "normal {n} frames, double speed {f}");
    }

    #[test]
    fn a_fast_replay_never_shortens_a_long_press() {
        let mut fast = Demo::default();
        fast.set_speed(4.0);
        fast.push([Step::Hold(30)]);
        assert!(frames_to_finish(&mut fast) >= 30, "the app needs the whole hold to see a long press");
    }

    #[test]
    fn the_speed_stays_within_its_range() {
        let mut demo = Demo::default();
        assert_eq!(demo.set_speed(100.0), *SPEED_RANGE.end());
        assert_eq!(demo.set_speed(0.0), *SPEED_RANGE.start());
        assert_eq!(demo.set_speed(f32::NAN), 1.0);
    }

    #[test]
    fn off_by_default_emits_nothing() {
        let mut demo = Demo::default();
        demo.on = false;
        demo.queue.push_back(Step::Press(PointerButton::Primary));
        assert!(drain(&mut demo, 5).is_empty(), "nothing plays while the cursor is off");
    }
}

