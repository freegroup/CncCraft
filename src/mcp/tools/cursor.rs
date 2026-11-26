//! `cursor`: show or hide the virtual mouse cursor used for recordings.

use crate::app::App;
use crate::mcp::tool::{boolean, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct Cursor;

impl Tool for Cursor {
    fn name(&self) -> &'static str {
        "cursor"
    }
    fn returns(&self) -> &'static str {
        "Text \"cursor on\" or \"cursor off\"."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Show or hide the virtual mouse cursor.", json!({"on": boolean("true shows the cursor, false hides it.")}), &["on"])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        ToolResult::from(app.agent_cursor(Args(args).bool("on").unwrap_or(true)))
    }
}
