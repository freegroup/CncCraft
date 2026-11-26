//! `draw_rectangle`: add a rectangle by centre and size. Returns the new shape id so a later call
//! can act on it.

use crate::app::App;
use crate::mcp::tool::{num, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct DrawRectangle;

impl Tool for DrawRectangle {
    fn name(&self) -> &'static str {
        "draw_rectangle"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"id\": N} — the new rectangle's shape id."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Add a rectangle by its centre (cx, cy in mm) and width/height (mm); optional corner_radius (mm). Returns {\"id\": ...}.",
            json!({
                "cx": num("Centre x in mm (from the left edge of the material)."),
                "cy": num("Centre y in mm (from the bottom edge of the material)."),
                "width": num("Width in mm."),
                "height": num("Height in mm."),
                "corner_radius": num("Rounded corners: their radius in mm (default 0, sharp)."),
            }),
            &["cx", "cy", "width", "height"],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        match (a.num("cx"), a.num("cy"), a.num("width"), a.num("height")) {
            (Some(cx), Some(cy), Some(w), Some(h)) => {
                let id = app.agent_rectangle(cx, cy, w, h, a.num("corner_radius"));
                ToolResult::ok(json!({"id": id}).to_string())
            }
            _ => ToolResult::err("need cx, cy, width, height"),
        }
    }
}
