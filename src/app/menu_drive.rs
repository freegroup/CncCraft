//! Driving the real menus with the visible cursor for the MCP menu commands: when an agent runs a
//! menu command, CncCraft opens the actual menu the way a person would — the cursor glides to the
//! menu button, clicks it open, glides to the item and clicks it — so a screen recording shows the
//! menu in action rather than the effect appearing out of nowhere.
//!
//! The menu buttons and items record their screen rects while they are drawn (see `menus.rs`); this
//! driver reads those rects and plays the clicks through the [`super::demo`] cursor, which injects
//! the same pointer events a mouse would. egui lays a menu's items out only while it is open, so the
//! driver works in phases, one gesture at a time, waiting for the cursor to finish each before the
//! next. If a rect never turns up (the menu or item isn't on screen) it gives up and the caller's
//! fallback runs the action directly, so nothing ever hangs.

use super::demo::Step;
use super::App;
use egui::{PointerButton, Pos2};

/// How many frames to wait for a menu's items to be laid out after it is opened before giving up.
const OPEN_WAIT: u32 = 30;

/// One menu command being carried out through the visible menu.
pub(super) struct MenuDrive {
    /// The command id (e.g. "file.new").
    pub cmd: &'static str,
    /// The top-level menu it lives under, by its (untranslated) name key, e.g. "Datei".
    pub menu: &'static str,
    phase: Phase,
    /// Frames spent waiting in the current phase.
    waited: u32,
}

#[derive(PartialEq)]
enum Phase {
    /// Move the cursor to the menu button and click it open.
    OpenMenu,
    /// Wait for the item's rect to appear, then click it.
    ClickItem,
    /// Done (or given up).
    Done,
}

impl MenuDrive {
    pub(super) fn new(cmd: &'static str, menu: &'static str) -> MenuDrive {
        MenuDrive { cmd, menu, phase: Phase::OpenMenu, waited: 0 }
    }
}

impl App {
    /// Start driving a command through its visible menu. Returns false when the menu button isn't on
    /// screen (then the caller runs the action directly instead).
    pub(super) fn start_menu_drive(&mut self, cmd: &'static str, menu: &'static str) -> bool {
        if !self.menu_button_rects.contains_key(menu) {
            return false;
        }
        self.demo.on = true;
        self.menu_drive = Some(MenuDrive::new(cmd, menu));
        true
    }

    /// Advance the visible-menu driver one frame. Called every frame from `update`.
    pub(super) fn drive_menu(&mut self, ctx: &egui::Context) {
        let Some(mut drive) = self.menu_drive.take() else { return };
        // a gesture is still playing: let it finish first
        if self.demo.busy() {
            self.menu_drive = Some(drive);
            ctx.request_repaint();
            return;
        }
        match drive.phase {
            Phase::OpenMenu => {
                if let Some(rect) = self.menu_button_rects.get(drive.menu).copied() {
                    click_at(self, rect.center());
                    drive.phase = Phase::ClickItem;
                    drive.waited = 0;
                } else {
                    drive.phase = Phase::Done;
                }
                self.menu_drive = Some(drive);
                ctx.request_repaint();
            }
            Phase::ClickItem => {
                if let Some(rect) = self.menu_item_rects.get(drive.cmd).copied() {
                    click_at(self, rect.center());
                    drive.phase = Phase::Done;
                } else if drive.waited < OPEN_WAIT {
                    // the menu is open but the item hasn't been laid out yet this frame: wait
                    drive.waited += 1;
                    ctx.request_repaint();
                } else {
                    // the item never appeared: run the action directly so nothing is lost
                    let _ = self.run_command_direct(drive.cmd);
                    drive.phase = Phase::Done;
                }
                self.menu_drive = Some(drive);
                ctx.request_repaint();
            }
            Phase::Done => {
                // keep the driver gone
            }
        }
    }

    /// Whether a visible-menu drive is in progress (the client waits on it with `window`).
    pub(crate) fn menu_drive_busy(&self) -> bool {
        self.menu_drive.as_ref().is_some_and(|d| d.phase != Phase::Done) || self.demo.busy()
    }
}

/// Glide the cursor to a point and click it (press, short beat, release).
fn click_at(app: &mut App, at: Pos2) {
    app.demo.push([Step::MoveTo(at), Step::Press(PointerButton::Primary), Step::Wait(6), Step::Release(PointerButton::Primary), Step::Wait(4)]);
}
