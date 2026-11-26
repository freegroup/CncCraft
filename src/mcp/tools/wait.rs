//! `wait`: pause for a number of seconds (a beat for the viewer).

use crate::app::App;
use crate::mcp::tool::{num, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct Wait;

impl Tool for Wait {
    fn name(&self) -> &'static str {
        "wait"
    }
    fn returns(&self) -> &'static str {
        "Text \"waiting\"."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Pause for a number of seconds (a beat for the viewer).", json!({"seconds": num("How long to pause, in seconds (decimals allowed).")}), &["seconds"])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        ToolResult::from(app.agent_wait(Args(args).num("seconds").unwrap_or(1.0)))
    }
}
