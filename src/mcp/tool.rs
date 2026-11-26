//! What every MCP tool is: a name, a JSON-Schema spec for `tools/list`, and a `call` that acts on
//! the app and returns a result. One tool per file under [`super::tools`], so a tool can grow its
//! own helpers and a more complex command stays self-contained.
//!
//! A tool never touches the app's private state directly; it goes through the `pub(crate)` agent
//! API on [`App`](crate::app::App) (see `src/app/agent.rs`), the way GridCraft's tools go through a
//! `Backend`. That keeps this module generic: the transport and the tool set here, the behaviour in
//! the app.

use crate::app::App;
use serde_json::{json, Value};

/// The result of a tool call: the text shown to the client and whether it is an error, turned into
/// MCP `content` blocks by the server.
pub struct ToolResult {
    pub text: String,
    pub is_error: bool,
}

impl ToolResult {
    pub fn ok(text: impl Into<String>) -> Self {
        ToolResult { text: text.into(), is_error: false }
    }
    pub fn err(text: impl Into<String>) -> Self {
        ToolResult { text: text.into(), is_error: true }
    }
    /// From the `Result<String, String>` most agent-API methods return.
    pub fn from(r: Result<String, String>) -> Self {
        match r {
            Ok(t) => ToolResult::ok(t),
            Err(e) => ToolResult::err(e),
        }
    }
}

/// One tool the MCP server exposes.
pub trait Tool: Send + Sync {
    /// The tool's name, as used in `tools/call`.
    fn name(&self) -> &'static str;
    /// A one-line description and the JSON-Schema of its arguments, for `tools/list`.
    fn spec(&self) -> Value;
    /// What a call gives back, for the agent guide (`agent.md`): the JSON fields or the text, so a
    /// script knows what a later line can use (`$name.id`). Every tool must say it.
    fn returns(&self) -> &'static str;
    /// Carry the call out on the app (on the UI thread, so it may touch the UI).
    fn call(&self, app: &mut App, ctx: &egui::Context, args: &Value) -> ToolResult;
}

// ---- small schema helpers shared by every tool's `spec` ----
//
// Every parameter carries a description of what it means: the MCP client (Claude) reads it, and the
// script editor shows it in its autocomplete and in the command help.

/// A number parameter.
pub fn num(description: &str) -> Value {
    json!({"type": "number", "description": description})
}
/// A text parameter.
pub fn string(description: &str) -> Value {
    json!({"type": "string", "description": description})
}
/// A yes/no parameter.
pub fn boolean(description: &str) -> Value {
    json!({"type": "boolean", "description": description})
}
/// A list of numbers (e.g. shape ids).
pub fn num_array(description: &str) -> Value {
    json!({"type": "array", "items": {"type": "number"}, "description": description})
}
/// A text parameter that takes one of fixed `values`.
pub fn one_of(values: &[&str], description: &str) -> Value {
    json!({"type": "string", "enum": values, "description": description})
}
/// A tool spec object: name, description and an input schema from `props`/`required`.
pub fn spec(name: &str, description: &str, props: Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {"type": "object", "properties": props, "required": required},
    })
}

// ---- argument readers (tools read their typed args through these) ----

/// A helper over the `arguments` object of one call.
pub struct Args<'a>(pub &'a Value);

impl Args<'_> {
    pub fn num(&self, key: &str) -> Option<f64> {
        self.0.get(key).and_then(|v| v.as_f64())
    }
    /// A value as text, whether it came as a number (`12` → "12") or a word/formula (`platte_d`,
    /// `platte_d/2`). For fields that take a number or a variable.
    pub fn value_str(&self, key: &str) -> Option<String> {
        match self.0.get(key) {
            Some(Value::String(s)) => Some(s.clone()),
            Some(Value::Number(n)) => Some(n.to_string()),
            _ => None,
        }
    }
    pub fn str(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.as_str())
    }
    pub fn bool(&self, key: &str) -> Option<bool> {
        self.0.get(key).and_then(|v| v.as_bool())
    }
    /// The ids of a numeric array argument (e.g. shape ids), or empty.
    pub fn ids(&self, key: &str) -> Vec<u64> {
        self.0
            .get(key)
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_u64()).collect())
            .unwrap_or_default()
    }

    /// A screen point from two numeric keys (e.g. "x"/"y"), if both are present.
    pub fn pos(&self, kx: &str, ky: &str) -> Option<egui::Pos2> {
        match (self.num(kx), self.num(ky)) {
            (Some(x), Some(y)) => Some(egui::Pos2::new(x as f32, y as f32)),
            _ => None,
        }
    }
}
