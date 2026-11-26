//! `draw_ellipse`: add an ellipse by centre and full width/height. Returns the new shape id.

use crate::app::App;
use crate::mcp::tool::{num, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct DrawEllipse;

impl Tool for DrawEllipse {
    fn name(&self) -> &'static str {
        "draw_ellipse"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"id\": N} — the new ellipse's shape id."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Add an ellipse by centre (cx, cy in mm) and full width/height (mm). Returns {\"id\": ...}.",
            json!({
                "cx": num("Centre x in mm (from the left edge of the material)."),
                "cy": num("Centre y in mm (from the bottom edge of the material)."),
                "width": num("Full width in mm."),
                "height": num("Full height in mm."),
            }),
            &["cx", "cy", "width", "height"],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        match (a.num("cx"), a.num("cy"), a.num("width"), a.num("height")) {
            (Some(cx), Some(cy), Some(w), Some(h)) => ToolResult::ok(json!({"id": app.agent_ellipse(cx, cy, w, h)}).to_string()),
            _ => ToolResult::err("need cx, cy, width, height"),
        }
    }
}
