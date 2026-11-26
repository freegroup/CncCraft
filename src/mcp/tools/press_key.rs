//! `press_key`: press a key with optional modifiers. The quit shortcut is refused by the app.

use crate::app::{parse_key, App};
use crate::mcp::tool::{boolean, spec, string, Args, Tool, ToolResult};
use egui::Modifiers;
use serde_json::{json, Value};

pub struct PressKey;

impl Tool for PressKey {
    fn name(&self) -> &'static str {
        "press_key"
    }
    fn returns(&self) -> &'static str {
        "Text \"key\"; the keys show as keycaps next to the big mouse."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Press a key (Enter, Escape, Tab, Delete, Backspace, Space, arrows, or a single letter/digit) with optional modifiers cmd/shift/alt/ctrl.",
            json!({
                "key": string("The key: Enter, Escape, Tab, Delete, Backspace, Space, Up, Down, Left, Right, or a single letter/digit."),
                "cmd": boolean("Hold ⌘ (Ctrl on Windows/Linux)."),
                "shift": boolean("Hold Shift."),
                "alt": boolean("Hold Alt/Option."),
                "ctrl": boolean("Hold Ctrl (treated like cmd)."),
            }),
            &["key"],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        match a.str("key").and_then(parse_key) {
            Some(key) => {
                let m = Modifiers {
                    command: a.bool("cmd").unwrap_or(false) || a.bool("ctrl").unwrap_or(false),
                    shift: a.bool("shift").unwrap_or(false),
                    alt: a.bool("alt").unwrap_or(false),
                    ..Default::default()
                };
                ToolResult::from(app.agent_press_key(key, m))
            }
            None => ToolResult::err("unknown key"),
        }
    }
}
