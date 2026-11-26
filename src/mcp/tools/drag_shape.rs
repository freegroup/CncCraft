//! `drag_shape`: drag a shape with the visible mouse to a new place — cursor to the shape, button
//! down, glide, button up — so it snaps to construction lines like a hand drag. Returns the id.

use crate::app::App;
use crate::mcp::tool::{num, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct DragShape;

impl Tool for DragShape {
    fn name(&self) -> &'static str {
        "drag_shape"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"id\", \"from\": {\"left\", \"top\"}, \"to\": {\"left\", \"top\"}} — the place asked for; the shape may snap a little to the grid or a construction line (read it back with state)."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Drag a shape with the visible mouse: the cursor goes to the shape, presses, glides and releases so the top-left corner of the shape's outline box lands at (left, top). It is the app's own drag, so the shape snaps to the grid and to construction lines (and attaches to a line it lands on), as by hand. Returns {\"id\", \"from\", \"to\"}.",
            json!({
                "id": num("The shape's id (from a drawing tool's result or the state tool)."),
                "left": num("Where the left edge of the shape should end up, in mm from the left edge of the material."),
                "top": num("Where the top edge of the shape should end up, in mm from the bottom edge of the material."),
            }),
            &["id", "left", "top"],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        match (a.num("id"), a.num("left"), a.num("top")) {
            (Some(id), Some(left), Some(top)) => ToolResult::from(app.agent_drag_shape(id as u64, left, top)),
            _ => ToolResult::err("need id, left and top (mm)"),
        }
    }
}
