//! `list_tools`: the cutters available for the current job's material.

use crate::app::App;
use crate::mcp::tool::{spec, Tool, ToolResult};
use serde_json::{json, Value};

pub struct ListTools;

impl Tool for ListTools {
    fn name(&self) -> &'static str {
        "list_tools"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"material\", \"tools\": [{\"name\", \"kind\", \"diameter\"}]}."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "List the cutters available for the current job's material (name, kind, diameter in mm).", json!({}), &[])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, _args: &Value) -> ToolResult {
        ToolResult::ok(app.agent_list_tools())
    }
}
