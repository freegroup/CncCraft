//! The menu items, tool buttons and view switches of the command catalog
//! ([`crate::mcp::commands`]) as commands of their own — `file_new`, `edit_undo`, `view_fit`,
//! `toolpath_simulate` … — so every one of them is an MCP command and a script command with its own
//! name, autocomplete and help, rather than an id handed to a generic command. One type serves them
//! all because they are data: each is carried out like a person would, through the visible menu (or
//! the panel the button is in), by `App::run_command`.

use crate::app::App;
use crate::mcp::commands::{menu_name, tool_name, Command};
use crate::mcp::tool::{spec, Tool, ToolResult};
use serde_json::{json, Value};

/// One catalog command as a tool.
pub struct CatalogCommand {
    /// The command's name ("edit_select_all"), kept for the program's life like the other names.
    name: &'static str,
    command: &'static Command,
}

impl CatalogCommand {
    pub fn new(command: &'static Command) -> CatalogCommand {
        // the tool set is built once per call of `all()`; the names are leaked once and reused
        static NAMES: std::sync::OnceLock<std::collections::HashMap<&'static str, &'static str>> = std::sync::OnceLock::new();
        let names = NAMES.get_or_init(|| crate::mcp::commands::COMMANDS.iter().map(|c| (c.id, &*Box::leak(tool_name(c.id).into_boxed_str()))).collect());
        CatalogCommand { name: names[command.id], command }
    }
}

impl Tool for CatalogCommand {
    fn name(&self) -> &'static str {
        self.name
    }
    fn returns(&self) -> &'static str {
        "Text saying what is being done, e.g. \"opening Edit › Undo in the menu\" (the mouse opens the menu and clicks the item) or \"clicking Preview in the toolpaths panel\"."
    }
    fn spec(&self) -> Value {
        let c = self.command;
        let place = match menu_name(c) {
            Some(_) => format!("The mouse opens the {} menu and clicks \"{}\".", c.path.get(), c.label.get()),
            None => format!("{} › {}.", c.path.get(), c.label.get()),
        };
        let mut description = format!("{} — {place}", c.label.get());
        if let Some(first) = c.params.chars().next() {
            // the note, as a sentence of its own
            description.push_str(&format!(" {}{}.", first.to_uppercase(), &c.params[first.len_utf8()..]));
        }
        if !c.keys.is_empty() {
            description.push_str(&format!(" Keys: {}.", c.keys));
        }
        spec(self.name, &description, json!({}), &[])
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, _args: &Value) -> ToolResult {
        ToolResult::from(app.run_command(self.command.id))
    }
}
