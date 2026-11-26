//! The panel commands: `panel_tools_open`, `panel_tools_close`, `panel_milling_open` and
//! `panel_milling_close`. Each moves the visible mouse to the panel's fold button (the chevron) and
//! really clicks it, so a recording shows the panel being opened or closed by hand. One type serves
//! all four, since they differ only in the panel and the direction.

use crate::app::App;
use crate::mcp::tool::{spec, Tool, ToolResult};
use serde_json::{json, Value};

/// Open or close one panel by clicking its fold button with the visible mouse.
pub struct Panel {
    /// The tool's name, e.g. "panel_tools_open".
    pub name: &'static str,
    /// Which panel: "tools" (the tool column, left) or "milling" (the toolpath panel, right).
    pub panel: &'static str,
    /// Whether the panel is to be opened (else closed).
    pub open: bool,
}

impl Tool for Panel {
    fn name(&self) -> &'static str {
        self.name
    }
    fn returns(&self) -> &'static str {
        "JSON {\"panel\": \"tools\"|\"milling\", \"open\": true|false, \"changed\": true|false}."
    }
    fn spec(&self) -> Value {
        let what = if self.panel == "tools" { "the tool column on the left" } else { "the milling (toolpath) panel on the right" };
        let verb = if self.open { "Open" } else { "Close" };
        let description = format!(
            "{verb} {what}: the visible mouse moves to the panel's fold button (the chevron) and clicks it. Nothing is clicked if it already is {}. Returns {{\"panel\", \"open\", \"changed\"}}.",
            if self.open { "open" } else { "closed" }
        );
        spec(self.name, &description, json!({}), &[])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, _args: &Value) -> ToolResult {
        ToolResult::from(app.agent_panel(self.panel, self.open))
    }
}
