//! `agent_md`: the agent guide (a "skill" file) — what CncCraft's script language is for, its syntax,
//! the rules, every command with parameters and return values, and example scripts — so an agent
//! connected over MCP can learn to write scripts by itself. The same text the script window's
//! "agent.md" button copies.

use crate::app::App;
use crate::mcp::tool::{spec, Tool, ToolResult};
use serde_json::{json, Value};

pub struct AgentMd;

impl Tool for AgentMd {
    fn name(&self) -> &'static str {
        "agent_md"
    }
    fn returns(&self) -> &'static str {
        "Text: the agent guide as Markdown."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Get the agent guide (Markdown): what CncCraft and its script language are for, the syntax, coordinates and pacing rules, every command with its parameters and return value, and example scripts. Read it first to write scripts.",
            json!({}),
            &[],
        )
    }
    fn call(&self, _app: &mut App, _ctx: &egui::Context, _args: &Value) -> ToolResult {
        ToolResult::ok(crate::scripting::skill::agent_md())
    }
}
