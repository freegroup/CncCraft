//! `type_text`: type text into the focused field.

use crate::app::App;
use crate::mcp::tool::{spec, string, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct TypeText;

impl Tool for TypeText {
    fn name(&self) -> &'static str {
        "type_text"
    }
    fn returns(&self) -> &'static str {
        "Text \"typed\"."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Type text into the focused field.", json!({"text": string("The text to type into the focused field.")}), &["text"])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        match Args(args).str("text") {
            Some(t) => ToolResult::from(app.agent_type_text(t)),
            None => ToolResult::err("need text"),
        }
    }
}
