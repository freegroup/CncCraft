//! `draw_circle`: add a circle by centre and radius. Returns the new shape id.

use crate::app::App;
use crate::mcp::tool::{num, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct DrawCircle;

impl Tool for DrawCircle {
    fn name(&self) -> &'static str {
        "draw_circle"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"id\": N} — the new circle's shape id."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Add a circle by centre (cx, cy in mm) and radius (mm). Returns {\"id\": ...}.",
            json!({
                "cx": num("Centre x in mm (from the left edge of the material)."),
                "cy": num("Centre y in mm (from the bottom edge of the material)."),
                "radius": num("Radius in mm."),
            }),
            &["cx", "cy", "radius"],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        match (a.num("cx"), a.num("cy"), a.num("radius")) {
            (Some(cx), Some(cy), Some(r)) => ToolResult::ok(json!({"id": app.agent_circle(cx, cy, r)}).to_string()),
            _ => ToolResult::err("need cx, cy, radius"),
        }
    }
}
