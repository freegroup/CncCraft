//! `clear_overlays`: take both captions (title and subtitle) off the window at once. The script
//! engine calls it when a run ends, so a standing `text_title` does not stay on screen afterwards.

use crate::app::App;
use crate::mcp::tool::{spec, Tool, ToolResult};
use serde_json::{json, Value};

pub struct ClearOverlays;

impl Tool for ClearOverlays {
    fn name(&self) -> &'static str {
        "clear_overlays"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"cleared\": true}."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Remove both captions (the title and the subtitle) from the window at once.", json!({}), &[])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, _args: &Value) -> ToolResult {
        ToolResult::from(app.agent_clear_overlays())
    }
}
