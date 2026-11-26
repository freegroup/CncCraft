//! The agent script: a queue that interleaves visible-cursor gestures with the app changes they
//! stand for, so an MCP command only *happens* once the mouse has arrived. This is the rule for
//! every agent command that acts somewhere on screen: move the cursor there first, and only when it
//! has reached the spot carry the change out — never change the model while the cursor is still
//! gliding (that looked wrong: guides vanished before the mouse got to them).
//!
//! A script is a list of steps, each either a cursor gesture (handed to the [`super::demo`] queue)
//! or a deferred action (a closure run on the app). The runner, advanced every frame from `update`,
//! waits for the cursor to finish the current gesture before it runs the next action or starts the
//! next gesture. Actions are `FnOnce(&mut App, &Context)`; the queue is taken out of `self` while an
//! action runs, so it can borrow the app freely.

use super::demo::Step;
use super::App;

/// One step of an agent script.
pub(super) enum ScriptStep {
    /// Play these cursor gestures (move, press, release, wait …) through the visible cursor.
    Cursor(Vec<Step>),
    /// Glide to the control reported under this name (see `App::control_rects`), looked up when the
    /// step plays — so it works for a control that only appears after an earlier step (a button in a
    /// panel the script has just opened). If it is not on screen then, the rest of the script is
    /// dropped and the status line says why.
    MoveToControl(&'static str),
    /// After a glide to a control: if the control has moved meanwhile (a panel that just opened
    /// settles its layout over a frame or two), glide again to where it is now — at most this many
    /// more times — so the click that follows lands on it.
    OnControl(&'static str, u8),
    /// Carry out an app change — run only once the cursor has finished the gestures before it.
    Act(Box<dyn FnOnce(&mut App, &egui::Context)>),
}

/// A queue of steps played in order, the cursor gestures and the changes they stand for interleaved.
#[derive(Default)]
pub(super) struct AgentScript {
    steps: std::collections::VecDeque<ScriptStep>,
}

impl AgentScript {
    fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// A builder for a script, so an agent command reads top to bottom: go here, then do this.
#[derive(Default)]
pub(crate) struct Script {
    steps: Vec<ScriptStep>,
}

impl Script {
    pub(crate) fn new() -> Script {
        Script::default()
    }

    /// Glide the cursor to a screen point (and pause a beat so the arrival reads).
    pub(crate) fn move_to(mut self, at: egui::Pos2) -> Script {
        self.steps.push(ScriptStep::Cursor(vec![Step::MoveTo(at), Step::Wait(6)]));
        self
    }

    /// Glide to the control named `key` (looked up when this step plays) and pause a beat.
    pub(crate) fn move_to_control(mut self, key: &'static str) -> Script {
        self.steps.push(ScriptStep::MoveToControl(key));
        self
    }

    /// Click at the cursor's current place (press, beat, release).
    pub(crate) fn click(mut self) -> Script {
        self.steps.push(ScriptStep::Cursor(vec![Step::Press(egui::PointerButton::Primary), Step::Wait(6), Step::Release(egui::PointerButton::Primary), Step::Wait(4)]));
        self
    }

    /// Hold Shift for the clicks that follow (a shift-click adds to the selection); [`Script::no_mods`]
    /// lets go.
    pub(crate) fn shift(mut self) -> Script {
        self.steps.push(ScriptStep::Cursor(vec![Step::Modifiers(egui::Modifiers { shift: true, ..Default::default() })]));
        self
    }

    /// Let go of any held modifiers.
    pub(crate) fn no_mods(mut self) -> Script {
        self.steps.push(ScriptStep::Cursor(vec![Step::Modifiers(egui::Modifiers::default())]));
        self
    }

    /// Press Escape (clears the selection / cancels the current mode), shown on the keycaps.
    pub(crate) fn escape(mut self) -> Script {
        self.steps.push(ScriptStep::Cursor(vec![Step::Key(egui::Key::Escape, egui::Modifiers::default()), Step::Wait(3)]));
        self
    }

    /// Press the primary button and hold it (for a drag); pair with [`Script::release`].
    pub(crate) fn press(mut self) -> Script {
        self.steps.push(ScriptStep::Cursor(vec![Step::Press(egui::PointerButton::Primary), Step::Wait(3)]));
        self
    }

    /// Release the primary button.
    pub(crate) fn release(mut self) -> Script {
        self.steps.push(ScriptStep::Cursor(vec![Step::Wait(3), Step::Release(egui::PointerButton::Primary), Step::Wait(4)]));
        self
    }

    /// Carry out an app change here — only after the cursor has reached the point above.
    pub(crate) fn act(mut self, f: impl FnOnce(&mut App, &egui::Context) + 'static) -> Script {
        self.steps.push(ScriptStep::Act(Box::new(f)));
        self
    }

    /// Type `text` into the control named `key`: glide to it, click to focus, select everything and
    /// type the new value, then Enter. For a number field (a text box or a drag field).
    pub(crate) fn type_into_control(mut self, key: &'static str, text: String) -> Script {
        self.steps.push(ScriptStep::MoveToControl(key));
        let select_all = egui::Modifiers { command: true, ..Default::default() };
        self.steps.push(ScriptStep::Cursor(vec![
            Step::Press(egui::PointerButton::Primary),
            Step::Wait(4),
            Step::Release(egui::PointerButton::Primary),
            Step::Wait(4),
            Step::Key(egui::Key::A, select_all),
            Step::Wait(2),
            Step::Type(text),
            Step::Wait(2),
            Step::Key(egui::Key::Enter, egui::Modifiers::default()),
            Step::Wait(4),
        ]));
        self
    }
}

impl App {
    /// Queue a script to play. Turns the cursor on; it runs over the next frames from
    /// [`App::drive_agent_script`].
    pub(crate) fn run_script(&mut self, script: Script) {
        self.demo.on = true;
        self.agent_script.steps.extend(script.steps);
    }

    /// Whether a script is still playing (its own steps, or a cursor gesture it started).
    pub(crate) fn script_busy(&self) -> bool {
        !self.agent_script.is_empty() || self.demo.busy()
    }

    /// Advance the agent script one frame: run queued actions and start cursor gestures, waiting for
    /// each gesture to finish before the next step. Called every frame from `update`.
    pub(super) fn drive_agent_script(&mut self, ctx: &egui::Context) {
        // a cursor gesture is still playing: let it finish before the next step
        if self.demo.busy() {
            if !self.agent_script.is_empty() {
                ctx.request_repaint();
            }
            return;
        }
        // run steps until one starts a cursor gesture (then wait for it next frame)
        while let Some(step) = self.agent_script.steps.pop_front() {
            match step {
                ScriptStep::Act(f) => f(self, ctx),
                ScriptStep::Cursor(steps) => {
                    self.demo.push(steps);
                    ctx.request_repaint();
                    return;
                }
                ScriptStep::MoveToControl(key) => {
                    match self.control_rects.get(key).copied() {
                        Some(rect) => {
                            self.demo.push([Step::MoveTo(rect.center()), Step::Wait(6)]);
                            // once there, make sure it is still there
                            self.agent_script.steps.push_front(ScriptStep::OnControl(key, 3));
                        }
                        None => {
                            // nothing to click: stop here rather than act somewhere else
                            self.agent_script.steps.clear();
                            self.status = format!("{key}: not on screen");
                        }
                    }
                    ctx.request_repaint();
                    return;
                }
                ScriptStep::OnControl(key, tries) => {
                    let Some(rect) = self.control_rects.get(key).copied() else {
                        self.agent_script.steps.clear();
                        self.status = format!("{key}: not on screen");
                        return;
                    };
                    if rect.shrink(1.0).contains(self.demo.pos) {
                        continue; // on it: the click can follow
                    }
                    if tries == 0 {
                        // it keeps moving away: rather no click than one in the wrong place
                        self.agent_script.steps.clear();
                        self.status = format!("{key}: could not be reached");
                        return;
                    }
                    self.demo.push([Step::MoveTo(rect.center()), Step::Wait(4)]);
                    self.agent_script.steps.push_front(ScriptStep::OnControl(key, tries - 1));
                    ctx.request_repaint();
                    return;
                }
            }
        }
    }
}
