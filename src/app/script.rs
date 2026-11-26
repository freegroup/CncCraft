//! The script player glue: it runs a script through the MCP **client** (see
//! [`crate::scripting::client`]) — the engine talks to the main window over the loopback MCP port,
//! like any outside agent, rather than reaching into the app's state. The app only holds the running
//! handle and drains its log lines each frame for the window to show.

use super::App;
use crate::scripting::client::{LogLine, Run};

/// The player state: the running script (if any) and the log collected so far.
#[derive(Default)]
pub(super) struct Script {
    run: Option<Run>,
    pub log: Vec<LogLine>,
}

impl App {
    /// Parse and start playing a script over the MCP port. A stale run is dropped first.
    pub(super) fn script_play(&mut self, text: &str) {
        self.script.log.clear();
        let port = self.mcp.as_ref().map(|m| m.port);
        match port {
            Some(port) => {
                self.demo.on = true;
                self.script.run = Some(crate::scripting::client::start(text, port));
            }
            None => {
                self.script.log = vec![LogLine { line_no: 0, text: t!("Der MCP-Server läuft nicht – das Skript kann nicht abgespielt werden.").to_string(), error: true }];
            }
        }
    }

    /// Stop playback (the running engine finishes its current line, then ends).
    pub(super) fn script_stop(&mut self) {
        if let Some(run) = &self.script.run {
            run.stop();
        }
    }

    /// Whether a script is still running.
    pub(super) fn script_playing(&self) -> bool {
        self.script.run.as_ref().is_some_and(|r| !r.is_done())
    }

    /// Each frame: drain the running script's log lines, and clear the handle once it has finished.
    pub(super) fn drive_script(&mut self, ctx: &egui::Context) {
        let Some(run) = &self.script.run else { return };
        let mut got = false;
        while let Ok(line) = run.rx.try_recv() {
            self.script.log.push(line);
            got = true;
        }
        if got || !run.is_done() {
            ctx.request_repaint();
        }
        if run.is_done() {
            // keep the handle so the log stays, but it reports done; a new play replaces it
        }
    }
}
