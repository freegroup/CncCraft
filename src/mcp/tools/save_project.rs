//! `save_project`: save the project to an absolute .cnccraft path.

use crate::app::App;
use crate::mcp::tool::{spec, string, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct SaveProject;

impl Tool for SaveProject {
    fn name(&self) -> &'static str {
        "save_project"
    }
    fn returns(&self) -> &'static str {
        "Text \"saved PATH\"."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Save the project to an absolute .cnccraft path.", json!({"path": string("Absolute path of the .cnccraft project file.")}), &["path"])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        match Args(args).str("path") {
            Some(p) => ToolResult::from(app.agent_save_project(p)),
            None => ToolResult::err("need path"),
        }
    }
}
