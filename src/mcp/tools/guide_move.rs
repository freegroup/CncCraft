//! `guide_move`: move a construction line to a new distance. The visible mouse plays the drag on
//! the ruler. Returns the guide id.

use crate::app::App;
use crate::mcp::tool::{num, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct GuideMove;

impl Tool for GuideMove {
    fn name(&self) -> &'static str {
        "guide_move"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"id\": N}."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Move a construction line (from guides_list) to a new distance (mm). The visible mouse drags it on the ruler; any attached shapes follow. Returns {\"id\": ...}.",
            json!({
                "id": num("The construction line's id (from guide_add or guides_list)."),
                "offset": num("The new distance in mm from the left edge (vertical) or the bottom edge (horizontal)."),
            }),
            &["id", "offset"],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        match (a.num("id"), a.num("offset")) {
            (Some(id), Some(offset)) => ToolResult::from(app.agent_guide_move(id as u64, offset)),
            _ => ToolResult::err("need id and offset (mm)"),
        }
    }
}
