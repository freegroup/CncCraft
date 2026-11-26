//! `delete_shapes`: delete shapes by id. Returns the ids that were removed.

use crate::app::App;
use crate::mcp::tool::{num_array, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct DeleteShapes;

impl Tool for DeleteShapes {
    fn name(&self) -> &'static str {
        "delete_shapes"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"deleted\": [ids]}."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Delete shapes by their ids. Returns {\"deleted\": [...]}.", json!({"ids": num_array("The ids of the shapes to delete.")}), &["ids"])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let ids = Args(args).ids("ids");
        match app.agent_delete_shapes(&ids) {
            Ok(_) => ToolResult::ok(json!({"deleted": ids}).to_string()),
            Err(e) => ToolResult::err(e),
        }
    }
}
