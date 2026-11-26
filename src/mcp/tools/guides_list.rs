//! `guides_list`: every construction line (ruler guide) with its id, axis, distance and attached
//! shapes, plus whether the lines are locked and the rulers shown.

use crate::app::App;
use crate::mcp::tool::{spec, Tool, ToolResult};
use serde_json::{json, Value};

pub struct GuidesList;

impl Tool for GuidesList {
    fn name(&self) -> &'static str {
        "guides_list"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"rulers_shown\", \"locked\", \"guides\": [{\"id\", \"axis\", \"offset\", \"expr\", \"attached\": [shape ids]}]}."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "List the construction lines (ruler guides): id, axis (vertical = an X value, horizontal = a Y value), offset in mm, formula and attached shape ids, plus whether the lines are locked and the rulers shown.",
            json!({}),
            &[],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, _args: &Value) -> ToolResult {
        ToolResult::ok(app.agent_guides_list())
    }
}
