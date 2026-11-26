//! `set_variable`: create or change a document variable that shapes and toolpaths can be bound to.

use crate::app::App;
use crate::mcp::tool::{spec, string, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct SetVariable;

impl Tool for SetVariable {
    fn name(&self) -> &'static str {
        "set_variable"
    }
    fn returns(&self) -> &'static str {
        "Text \"NAME = EXPRESSION\"."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Create or change a document variable: a name and an expression (a number or formula). Shapes and toolpaths bound to it follow.",
            json!({
                "name": string("The variable's name, e.g. plate_w."),
                "expression": string("A number or formula, e.g. 120 or plate_w/2."),
            }),
            &["name", "expression"],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        match (a.str("name"), a.str("expression")) {
            (Some(n), Some(e)) => ToolResult::from(app.agent_set_variable(n, e)),
            _ => ToolResult::err("need name, expression"),
        }
    }
}
