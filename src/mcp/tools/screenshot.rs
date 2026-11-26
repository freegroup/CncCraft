//! `screenshot`: save a PNG of the window (never closes the app).

use crate::app::App;
use crate::mcp::tool::{spec, string, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct Screenshot;

impl Tool for Screenshot {
    fn name(&self) -> &'static str {
        "screenshot"
    }
    fn returns(&self) -> &'static str {
        "Text \"saving to PATH\" (the PNG is written on the next frame)."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Save a PNG of the window to an absolute path (does not close the app).", json!({"path": string("Absolute path of the PNG file to write.")}), &["path"])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        match Args(args).str("path") {
            Some(p) => ToolResult::from(app.agent_screenshot(p)),
            None => ToolResult::err("need path"),
        }
    }
}
