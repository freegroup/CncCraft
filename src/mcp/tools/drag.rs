//! `drag`: press at one point, drag to another and release.

use crate::app::App;
use crate::mcp::tool::{num, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct Drag;

impl Tool for Drag {
    fn name(&self) -> &'static str {
        "drag"
    }
    fn returns(&self) -> &'static str {
        "Text \"dragged\"; it plays on the visible cursor."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Press at (x1,y1), drag to (x2,y2) and release.", json!({
            "x1": num("Start x in screen pixels (where the button goes down)."),
            "y1": num("Start y in screen pixels."),
            "x2": num("End x in screen pixels (where the button is released)."),
            "y2": num("End y in screen pixels."),
        }), &["x1", "y1", "x2", "y2"])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        match (a.pos("x1", "y1"), a.pos("x2", "y2")) {
            (Some(from), Some(to)) => ToolResult::from(app.agent_drag(from, to)),
            _ => ToolResult::err("need x1, y1, x2, y2"),
        }
    }
}
