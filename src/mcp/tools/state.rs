//! `state`: a JSON summary of the job, shapes and toolpaths — how an agent finds shape ids.

use crate::app::App;
use crate::mcp::tool::{spec, Tool, ToolResult};
use serde_json::{json, Value};

pub struct State;

impl Tool for State {
    fn name(&self) -> &'static str {
        "state"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"job\": {\"width\", \"height\", \"thickness\", \"material\"}, \"selection\": [ids], \"shapes\": [{\"id\", \"kind\", \"cx\", \"cy\", \"width\", \"height\"}], \"toolpaths\": [{\"index\", \"name\", \"tool\", \"cut_depth\"}]}."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Report the current job (size, material), the shapes (id, kind, centre, size in mm) and the toolpaths (index, name, tool, depth). Use it to find shape ids before adding toolpaths.",
            json!({}),
            &[],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, _args: &Value) -> ToolResult {
        ToolResult::ok(app.agent_state())
    }
}
