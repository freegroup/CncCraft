//! `select`: select shapes by id (clears the previous selection). Returns the selected ids.

use crate::app::App;
use crate::mcp::tool::{num_array, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct Select;

impl Tool for Select {
    fn name(&self) -> &'static str {
        "select"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"selected\": [ids]}."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Select shapes by their ids (clears the previous selection). Returns {\"selected\": [...]}.", json!({"ids": num_array("The ids of the shapes to select.")}), &["ids"])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let ids = Args(args).ids("ids");
        match app.agent_select(&ids) {
            Ok(_) => ToolResult::ok(json!({"selected": app.selection_ids()}).to_string()),
            Err(e) => ToolResult::err(e),
        }
    }
}
