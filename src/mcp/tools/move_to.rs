//! `move_to`: glide the cursor to a screen point.

use crate::app::App;
use crate::mcp::tool::{num, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct MoveTo;

impl Tool for MoveTo {
    fn name(&self) -> &'static str {
        "move_to"
    }
    fn returns(&self) -> &'static str {
        "Text \"moving\"; the cursor glides there."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Glide the cursor to a screen point (pixels from the top-left of the window).", json!({"x": num("Screen x in pixels from the left of the window."), "y": num("Screen y in pixels from the top of the window.")}), &["x", "y"])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        match Args(args).pos("x", "y") {
            Some(p) => ToolResult::from(app.agent_move_to(p)),
            None => ToolResult::err("need x, y"),
        }
    }
}
