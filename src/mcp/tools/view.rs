//! `view`: switch between the 2D and 3D view.

use crate::app::App;
use crate::mcp::tool::{one_of, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct View;

impl Tool for View {
    fn name(&self) -> &'static str {
        "view"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"mode\": \"2d\"|\"3d\", \"changed\": true|false}."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Switch to the 2D or 3D view: the visible mouse clicks the 2D/3D toggle in the canvas bar (nothing is clicked if the view already is the one asked for). Returns {\"mode\", \"changed\"}.", json!({"mode": one_of(&["2d", "3d"], "2d: the drawing; 3d: the board in 3D (the cuts show once the preview runs: toolpath_preview).")}), &["mode"])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        ToolResult::from(app.agent_view(Args(args).str("mode") == Some("3d")))
    }
}
