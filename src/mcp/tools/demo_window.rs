//! `demo_window`: open or close the in-app Demo & remote-control window.

use crate::app::App;
use crate::mcp::tool::{boolean, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct DemoWindow;

impl Tool for DemoWindow {
    fn name(&self) -> &'static str {
        "demo_window"
    }
    fn returns(&self) -> &'static str {
        "Text \"demo window open\" or \"demo window closed\"."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Open or close the in-app Demo & remote-control window.", json!({"open": boolean("true opens the window, false closes it (default true).")}), &[])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        ToolResult::from(app.agent_demo_window(Args(args).bool("open").unwrap_or(true)))
    }
}
