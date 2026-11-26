//! `recording_stop`: finish the video recording started with `recording_start` and close the file.

use crate::app::App;
use crate::mcp::tool::{spec, Tool, ToolResult};
use serde_json::{json, Value};

pub struct RecordingStop;

impl Tool for RecordingStop {
    fn name(&self) -> &'static str {
        "recording_stop"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"file\", \"frames\", \"seconds\", \"width\", \"height\"}."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Stop the video recording and close the .mp4 file. Returns where it was written and how long it is.", json!({}), &[])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, _args: &Value) -> ToolResult {
        ToolResult::from(app.agent_recording_stop())
    }
}
