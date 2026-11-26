//! `list_materials`: the material keys available for a job (for `new_job`).

use crate::app::App;
use crate::mcp::tool::{spec, Tool, ToolResult};
use serde_json::{json, Value};

pub struct ListMaterials;

impl Tool for ListMaterials {
    fn name(&self) -> &'static str {
        "list_materials"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"materials\": [keys]}."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "List the material keys available for a job (e.g. for new_job).", json!({}), &[])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, _args: &Value) -> ToolResult {
        ToolResult::ok(app.agent_list_materials())
    }
}
