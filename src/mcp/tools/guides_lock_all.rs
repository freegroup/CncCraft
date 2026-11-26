//! `guides_lock_all`: lock every construction line. The visible mouse clicks the lock in the rulers'
//! corner. Locked lines can't be added, moved, edited or deleted (shapes still snap to them).

use crate::app::App;
use crate::mcp::tool::{spec, Tool, ToolResult};
use serde_json::{json, Value};

pub struct GuidesLockAll;

impl Tool for GuidesLockAll {
    fn name(&self) -> &'static str {
        "guides_lock_all"
    }
    fn returns(&self) -> &'static str {
        "Text \"locking all construction lines\"."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Lock every construction line (the mouse clicks the lock in the rulers' corner). Locked lines can't be added, moved, edited or deleted; shapes still snap to them.", json!({}), &[])
    }
    fn call(&self, app: &mut App, ctx: &egui::Context, _args: &Value) -> ToolResult {
        ToolResult::from(app.agent_guides_set_locked(ctx, true))
    }
}
