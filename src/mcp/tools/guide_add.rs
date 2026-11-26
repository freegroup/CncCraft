//! `guide_add`: add a construction line at a distance. Returns the new guide id. The visible mouse
//! glides to where the line meets its ruler.

use crate::app::App;
use crate::mcp::tool::{num, one_of, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct GuideAdd;

impl Tool for GuideAdd {
    fn name(&self) -> &'static str {
        "guide_add"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"id\": N} — the new construction line's id."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Add a construction line (ruler guide): axis \"vertical\" (an X value, measured from the left edge) or \"horizontal\" (a Y value, from the bottom), at offset mm. Turns the rulers on if needed. Returns {\"id\": ...}.",
            json!({
                "axis": one_of(&["vertical", "horizontal"], "vertical: a line at an x value; horizontal: a line at a y value."),
                "offset": num("Distance in mm from the left edge (vertical) or the bottom edge (horizontal)."),
            }),
            &["axis", "offset"],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        match (a.str("axis"), a.num("offset")) {
            (Some(axis), Some(offset)) => ToolResult::from(app.agent_guide_add(axis, offset)),
            _ => ToolResult::err("need axis (vertical|horizontal) and offset (mm)"),
        }
    }
}
