//! `script_window`: open or close the script window (its own OS window for writing and playing a
//! script of MCP commands).

use crate::app::App;
use crate::mcp::tool::{boolean, spec, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct ScriptWindow;

impl Tool for ScriptWindow {
    fn name(&self) -> &'static str {
        "script_window"
    }
    fn returns(&self) -> &'static str {
        "Text \"script window open\" or \"script window closed\"."
    }
    fn spec(&self) -> Value {
        spec(self.name(), "Open or close the script window (its own OS window for writing and playing a script of commands).", json!({"open": boolean("true opens the window, false closes it (default true).")}), &[])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        ToolResult::from(app.agent_script_window(Args(args).bool("open").unwrap_or(true)))
    }
}
