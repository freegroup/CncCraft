//! `replay_speed`: how fast the visible cursor plays — its glides, clicks and pauses — so a
//! recording can be paced. Also a script line (`replay_speed speed=2`).

use crate::app::App;
use crate::mcp::tool::{num, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct ReplaySpeed;

impl Tool for ReplaySpeed {
    fn name(&self) -> &'static str {
        "replay_speed"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"speed\": N} — the speed now in use."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Set how fast the visible cursor plays its moves, clicks and pauses: 1 is normal, 2 twice as fast, 0.5 half as fast (0.25 to 4). A long press is never made shorter than it must be. Without speed, it reports the current one. Returns {\"speed\": ...}.",
            json!({"speed": num("The replay speed: 1 normal, 2 twice as fast, 0.5 half as fast (kept between 0.25 and 4).")}),
            &[],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        ToolResult::from(app.agent_replay_speed(Args(args).num("speed")))
    }
}
