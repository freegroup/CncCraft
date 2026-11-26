//! `export_gcode`: write the visible toolpaths as a G-code file.

use crate::app::App;
use crate::mcp::tool::{spec, string, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct ExportGcode;

impl Tool for ExportGcode {
    fn name(&self) -> &'static str {
        "export_gcode"
    }
    fn returns(&self) -> &'static str {
        "Text saying which file was written."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Write the visible toolpaths as a G-code file to an absolute path.", json!({"path": string("Absolute path of the G-code file to write.")}), &["path"])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        match Args(args).str("path") {
            Some(p) => ToolResult::from(app.agent_export_gcode(p)),
            None => ToolResult::err("need path"),
        }
    }
}
