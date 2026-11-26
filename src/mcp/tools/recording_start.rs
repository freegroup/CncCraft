//! `recording_start`: begin recording the window to an MP4 video (H.264), built in-app — no external
//! program. Pair with `recording_stop`. Set a steady window size with `window` first.

use crate::app::App;
use crate::mcp::tool::{num, spec, string, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct RecordingStart;

impl Tool for RecordingStart {
    fn name(&self) -> &'static str {
        "recording_start"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"recording\": path, \"fps\": N}."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Start recording the window to an .mp4 video (H.264, encoded inside the app). Everything shown is captured, including the virtual mouse and captions. Stop it with recording_stop. Set the window size with window first, since the video keeps the size it starts with.",
            json!({
                "file": string("Absolute path of the .mp4 file to write."),
                "fps": num("Frames per second (default 30, 1 to 60)."),
                "max_side": num("Largest side of the video in pixels (default 1920); a bigger window is scaled down by a whole factor."),
            }),
            &["file"],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        match a.str("file") {
            Some(file) => ToolResult::from(app.agent_recording_start(file, a.num("fps"), a.num("max_side"))),
            None => ToolResult::err("need file (an absolute .mp4 path)"),
        }
    }
}
