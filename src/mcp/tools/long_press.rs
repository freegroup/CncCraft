//! `long_press`: long-press at a screen point (opens a value for editing, like holding the mouse).

use crate::app::App;
use crate::mcp::tool::{num, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct LongPress;

impl Tool for LongPress {
    fn name(&self) -> &'static str {
        "long_press"
    }
    fn returns(&self) -> &'static str {
        "Text \"long-pressed\"; it plays on the visible cursor."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Long-press at a screen point (to open a value for editing, like a person holding the mouse).", json!({"x": num("Screen x in pixels from the left of the window."), "y": num("Screen y in pixels from the top of the window.")}), &["x", "y"])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        match Args(args).pos("x", "y") {
            Some(p) => ToolResult::from(app.agent_long_press(p)),
            None => ToolResult::err("need x, y"),
        }
    }
}
