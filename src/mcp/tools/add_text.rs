//! `add_text`: add a line of text by centre, content and letter height. Returns the new shape id.

use crate::app::App;
use crate::mcp::tool::{num, spec, string, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct AddText;

impl Tool for AddText {
    fn name(&self) -> &'static str {
        "add_text"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"id\": N} — the new text's shape id."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Add text by centre (cx, cy in mm), the text and the letter height (mm, default 20). Returns {\"id\": ...}.",
            json!({
                "cx": num("Centre x in mm (from the left edge of the material)."),
                "cy": num("Centre y in mm (from the bottom edge of the material)."),
                "text": string("The text to write."),
                "height": num("Letter height in mm (default 20)."),
            }),
            &["cx", "cy", "text"],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        match (a.num("cx"), a.num("cy"), a.str("text")) {
            (Some(cx), Some(cy), Some(t)) => ToolResult::ok(json!({"id": app.agent_text(cx, cy, t, a.num("height").unwrap_or(20.0))}).to_string()),
            _ => ToolResult::err("need cx, cy, text"),
        }
    }
}
