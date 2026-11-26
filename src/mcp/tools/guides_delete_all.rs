//! `guides_delete_all`: delete every construction line, with the visible mouse moving from one
//! guide to the next and removing each (its shapes stay put).

use crate::app::App;
use crate::mcp::tool::{spec, Tool, ToolResult};
use serde_json::{json, Value};

pub struct GuidesDeleteAll;

impl Tool for GuidesDeleteAll {
    fn name(&self) -> &'static str {
        "guides_delete_all"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"deleted\": [ids]}."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Delete every construction line: the mouse glides from one guide's handle to the next and removes each. Shapes stay put. Returns {\"deleted\": [ids...]}.",
            json!({}),
            &[],
        )
    }
    fn call(&self, app: &mut App, ctx: &egui::Context, _args: &Value) -> ToolResult {
        ToolResult::from(app.agent_guides_delete_all(ctx))
    }
}
