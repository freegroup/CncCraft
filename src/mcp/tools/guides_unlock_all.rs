//! `guides_unlock_all`: unlock every construction line. The visible mouse clicks the lock in the
//! rulers' corner, so the lines can be moved, edited and deleted again.

use crate::app::App;
use crate::mcp::tool::{spec, Tool, ToolResult};
use serde_json::{json, Value};

pub struct GuidesUnlockAll;

impl Tool for GuidesUnlockAll {
    fn name(&self) -> &'static str {
        "guides_unlock_all"
    }
    fn returns(&self) -> &'static str {
        "Text \"unlocking all construction lines\"."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Unlock every construction line (the mouse clicks the lock in the rulers' corner), so they can be moved, edited and deleted again.", json!({}), &[])
    }
    fn call(&self, app: &mut App, ctx: &egui::Context, _args: &Value) -> ToolResult {
        ToolResult::from(app.agent_guides_set_locked(ctx, false))
    }
}
