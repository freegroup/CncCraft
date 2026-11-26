//! `window`: report the window (size, panels, view, speed, recording) and, when width/height are
//! given, resize it — a steady size for recording.

use crate::app::App;
use crate::mcp::tool::{num, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct Window;

impl Tool for Window {
    fn name(&self) -> &'static str {
        "window"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"width\", \"height\", \"busy\", \"port\", \"panels\": {\"tools\", \"milling\"}, \"view\", \"replay_speed\", \"recording\": path or null}."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Report the window size, whether the cursor is still playing (busy), the MCP port, which side panels are open, the view (2d or 3d), the replay speed and the recording. With width and/or height (logical points), it resizes the window first — set a steady size before recording_start.",
            json!({
                "width": num("New window width in points (optional; keeps the current width if left out)."),
                "height": num("New window height in points (optional; keeps the current height if left out)."),
            }),
            &[],
        )
    }
    fn call(&self, app: &mut App, ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        ToolResult::ok(app.agent_window(ctx, a.num("width"), a.num("height")))
    }
}
