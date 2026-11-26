//! `delete_toolpath`: remove a toolpath by its index in the list.

use crate::app::App;
use crate::mcp::tool::{num, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct DeleteToolpath;

impl Tool for DeleteToolpath {
    fn name(&self) -> &'static str {
        "delete_toolpath"
    }
    fn returns(&self) -> &'static str {
        "Text \"deleted NAME\"."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Delete a toolpath by its index in the list (0-based).", json!({"index": num("Position of the toolpath in the list, starting at 0 (see the state tool).")}), &["index"])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        ToolResult::from(app.agent_delete_toolpath(Args(args).num("index").unwrap_or(-1.0) as i64))
    }
}
