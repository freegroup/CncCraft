//! `double_click`: double-click at a screen point.

use crate::app::App;
use crate::mcp::tool::{num, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct DoubleClick;

impl Tool for DoubleClick {
    fn name(&self) -> &'static str {
        "double_click"
    }
    fn returns(&self) -> &'static str {
        "Text \"double-clicked\"; it plays on the visible cursor."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Double-click at a screen point.", json!({"x": num("Screen x in pixels from the left of the window."), "y": num("Screen y in pixels from the top of the window.")}), &["x", "y"])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        match Args(args).pos("x", "y") {
            Some(p) => ToolResult::from(app.agent_double_click(p)),
            None => ToolResult::err("need x, y"),
        }
    }
}
