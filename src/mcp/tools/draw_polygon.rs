//! `draw_polygon`: add a regular polygon by centre, radius and side count. Returns the new shape id.

use crate::app::App;
use crate::mcp::tool::{num, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct DrawPolygon;

impl Tool for DrawPolygon {
    fn name(&self) -> &'static str {
        "draw_polygon"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"id\": N} — the new polygon's shape id."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Add a regular polygon by centre (cx, cy in mm), radius (mm) and number of sides (>=3). Returns {\"id\": ...}.",
            json!({
                "cx": num("Centre x in mm (from the left edge of the material)."),
                "cy": num("Centre y in mm (from the bottom edge of the material)."),
                "radius": num("Distance from the centre to a corner, in mm."),
                "sides": num("Number of sides (3 or more)."),
            }),
            &["cx", "cy", "radius", "sides"],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        match (a.num("cx"), a.num("cy"), a.num("radius"), a.num("sides")) {
            (Some(cx), Some(cy), Some(r), Some(s)) => ToolResult::ok(json!({"id": app.agent_polygon(cx, cy, r, s as u32)}).to_string()),
            _ => ToolResult::err("need cx, cy, radius, sides"),
        }
    }
}
