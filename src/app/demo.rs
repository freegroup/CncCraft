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
/// instead of jumping (about 0.3 s at 60 fps for a long move).
const MOVE_FRAMES_PER_100PX: f32 = 11.0;

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
}

impl Default for Demo {
    fn default() -> Self {
        Demo { on: false, pos: Pos2::new(40.0, 40.0), moving: None, down: None, holding: None, queue: VecDeque::new(), wait: 0 }
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
                raw.events.push(button_event(self.pos, PointerButton::Primary, false));
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
                let frames = ((dist / 100.0) * MOVE_FRAMES_PER_100PX).round().max(1.0) as u32;
                self.moving = Some((self.pos, to, frames, 0));
            }
            Step::Press(b) => {
                raw.events.push(button_event(self.pos, b, true));
                self.down = Some(b);
            }
            Step::Release(b) => {
                raw.events.push(button_event(self.pos, b, false));
                self.down = None;
            }
            Step::Hold(frames) => {
                raw.events.push(button_event(self.pos, PointerButton::Primary, true));
                self.down = Some(PointerButton::Primary);
                self.holding = Some((frames, 0));
            }
            Step::Type(text) => {
                raw.events.push(Event::Text(text));
            }
            Step::Key(key, modifiers) => {
                raw.events.push(Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers });
                raw.events.push(Event::Key { key, physical_key: None, pressed: false, repeat: false, modifiers });
            }
            Step::Wait(frames) => self.wait = frames,
        }
    }

    /// Draw the cursor over everything, with the pressed button and the long-press ring.
    pub fn draw(&self, ctx: &egui::Context) {
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
        cursor_arrow(&p, at, self.down);
    }
}

/// A full mouse-button press-or-release event at `pos` (egui needs the move first; the caller sends
/// the move as part of the glide, so here only the button change).
fn button_event(pos: Pos2, button: PointerButton, pressed: bool) -> Event {
    Event::PointerButton { pos, button, pressed, modifiers: Modifiers::default() }
}

/// Draw a classic arrow pointer with its tip at `at`; the little button badge shows which mouse
/// button is held (left or right), like screen-demo tools.
fn cursor_arrow(p: &egui::Painter, at: Pos2, down: Option<PointerButton>) {
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
    // the pressed-button badge beside the cursor
    if let Some(b) = down {
        let centre = at + Vec2::new(22.0, 4.0);
        let body = egui::Rect::from_center_size(centre, Vec2::new(16.0, 20.0));
        p.rect_stroke(body, 4.0, egui::Stroke::new(1.4_f32, Color32::from_rgb(0x1c, 0x1c, 0x1c)));
        // the half that is pressed is filled with the accent
        let (left, right) = (b == PointerButton::Primary, b == PointerButton::Secondary);
        let top = egui::Rect::from_min_max(body.min, Pos2::new(body.max.x, body.min.y + 9.0));
        if left {
            p.rect_filled(egui::Rect::from_min_max(top.min, Pos2::new(top.center().x, top.max.y)), 0.0, crate::theme::ACCENT);
        }
        if right {
            p.rect_filled(egui::Rect::from_min_max(Pos2::new(top.center().x, top.min.y), top.max), 0.0, crate::theme::ACCENT);
        }
        p.line_segment([Pos2::new(centre.x, body.min.y), Pos2::new(centre.x, body.min.y + 9.0)], egui::Stroke::new(1.0_f32, Color32::from_rgb(0x1c, 0x1c, 0x1c)));
    }
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
            if dialogs::header(ui, &t!("Demo & Fernsteuerung")) {
                self.demo_open = false;
            }
            ui.label(egui::RichText::new(t!("CncCraft lässt sich fernsteuern, um Funktions-Demos aufzunehmen. Eine KI (oder ein Skript) spricht mit dem eingebauten MCP-Server; eine sichtbare Maus zeigt jede Aktion.")).small().color(crate::theme::muted()));
            ui.add_space(6.0);
            match port {
                Some(p) => {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(t!("MCP-Server:")).color(crate::theme::ink()));
                        ui.monospace(format!("http://127.0.0.1:{p}/"));
                    });
                    ui.label(egui::RichText::new(t!("Nur über localhost erreichbar, ohne Anmeldung.")).small().color(crate::theme::muted()));
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

    #[test]
    fn off_by_default_emits_nothing() {
        let mut demo = Demo::default();
        demo.on = false;
        demo.queue.push_back(Step::Press(PointerButton::Primary));
        assert!(drain(&mut demo, 5).is_empty(), "nothing plays while the cursor is off");
    }
}

