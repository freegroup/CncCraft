//! `guide_delete`: remove a construction line (its shapes stay put). The visible mouse glides to its
//! handle on the ruler first.

use crate::app::App;
use crate::mcp::tool::{num, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct GuideDelete;

impl Tool for GuideDelete {
    fn name(&self) -> &'static str {
        "guide_delete"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"deleted\": id}."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Delete a construction line by its id (from guides_list); its shapes stay put. Returns {\"deleted\": id}.",
            json!({"id": num("The construction line's id (from guide_add or guides_list).")}),
            &["id"],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        match Args(args).num("id") {
            Some(id) => ToolResult::from(app.agent_guide_delete(id as u64)),
            None => ToolResult::err("need id"),
        }
    }
}
