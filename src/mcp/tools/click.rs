//! `click`: click at a screen point with the left (default) or right button.

use crate::app::App;
use crate::mcp::tool::{num, one_of, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct Click;

impl Tool for Click {
    fn name(&self) -> &'static str {
        "click"
    }
    fn returns(&self) -> &'static str {
        "Text \"clicked\"; the click plays on the visible cursor."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Click at a screen point with the left (default) or right button.", json!({"x": num("Screen x in pixels from the left of the window."), "y": num("Screen y in pixels from the top of the window."), "button": one_of(&["left", "right"], "Which mouse button (default left).")}), &["x", "y"])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        match a.pos("x", "y") {
            Some(p) => ToolResult::from(app.agent_click(p, a.str("button") == Some("right"))),
            None => ToolResult::err("need x, y"),
        }
    }
}
