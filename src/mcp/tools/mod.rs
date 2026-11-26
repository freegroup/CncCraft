//! The MCP tool set: one file per command, each a [`Tool`](super::tool::Tool). [`all`] assembles
//! the list the server advertises and dispatches to. Adding a command is adding a file here and a
//! line in [`all`]; a command that grows complex keeps its logic in its own file.
//!
//! Every command is its own tool, and the tools are exactly the script commands — nothing is reached
//! through a generic "run this id" command. They split into:
//! - strongly-typed high-level builders (`draw_rectangle`, `add_toolpath`, `set_variable` …);
//! - the menu items, tool buttons and view switches of the [`super::commands`] catalog (`file_new`,
//!   `edit_undo`, `toolpath_simulate` …), carried out through the visible menu or panel;
//! - the visible pointer/keyboard primitives that drive the cursor for recordings.

use super::tool::Tool;
use super::tool::ToolResult;
use crate::app::App;
use serde_json::Value;

mod add_text;
mod agent_md;
mod add_toolpath;
mod clear_overlays;
mod click;
mod cursor;
mod delete_shapes;
mod delete_toolpath;
mod demo_window;
mod double_click;
mod drag;
mod catalog_command;
mod drag_shape;
mod draw_circle;
mod draw_ellipse;
mod draw_polygon;
mod draw_rectangle;
mod export_gcode;
mod guide_add;
mod guide_delete;
mod guide_move;
mod guides_delete_all;
mod guides_list;
mod guides_lock_all;
mod guides_unlock_all;
mod list_materials;
mod list_tools;
mod long_press;
mod move_to;
mod new_job;
mod panel;
mod press_key;
mod recording_start;
mod recording_stop;
mod replay_speed;
mod save_project;
mod screenshot;
mod script_window;
mod select;
mod set_variable;
mod state;
mod text_overlay;
mod type_text;
mod view;
mod wait;
mod window;

/// Every tool the server exposes, in `tools/list` order: the agent guide, state and discovery, the
/// high-level builders, the menu and panel commands, then the visible pointer/keyboard primitives.
pub fn all() -> Vec<Box<dyn Tool>> {
    let mut tools: Vec<Box<dyn Tool>> = vec![
        // the agent guide first: an agent reads it to learn the rest
        Box::new(agent_md::AgentMd),
        // state & discovery
        Box::new(state::State),
        Box::new(list_materials::ListMaterials),
        Box::new(list_tools::ListTools),
        Box::new(window::Window),
        // high-level builders
        Box::new(new_job::NewJob),
        Box::new(draw_rectangle::DrawRectangle),
        Box::new(draw_circle::DrawCircle),
        Box::new(draw_ellipse::DrawEllipse),
        Box::new(draw_polygon::DrawPolygon),
        Box::new(add_text::AddText),
        Box::new(drag_shape::DragShape),
        Box::new(delete_shapes::DeleteShapes),
        Box::new(select::Select),
        Box::new(set_variable::SetVariable),
        Box::new(add_toolpath::AddToolpath),
        Box::new(delete_toolpath::DeleteToolpath),
        // construction lines (ruler guides)
        Box::new(guides_list::GuidesList),
        Box::new(guide_add::GuideAdd),
        Box::new(guide_move::GuideMove),
        Box::new(guide_delete::GuideDelete),
        Box::new(guides_delete_all::GuidesDeleteAll),
        Box::new(guides_lock_all::GuidesLockAll),
        Box::new(guides_unlock_all::GuidesUnlockAll),
        Box::new(save_project::SaveProject),
        Box::new(export_gcode::ExportGcode),
        Box::new(view::View),
        // the side panels, opened and closed by clicking their fold buttons
        Box::new(panel::Panel { name: "panel_tools_open", panel: "tools", open: true }),
        Box::new(panel::Panel { name: "panel_tools_close", panel: "tools", open: false }),
        Box::new(panel::Panel { name: "panel_milling_open", panel: "milling", open: true }),
        Box::new(panel::Panel { name: "panel_milling_close", panel: "milling", open: false }),
        Box::new(demo_window::DemoWindow),
        Box::new(script_window::ScriptWindow),
        Box::new(screenshot::Screenshot),
    ];
    // the menu items, tool buttons and view switches, each a command of its own
    tools.extend(super::commands::COMMANDS.iter().map(|c| Box::new(catalog_command::CatalogCommand::new(c)) as Box<dyn Tool>));
    tools.extend::<Vec<Box<dyn Tool>>>(vec![
        // visible pointer & keyboard primitives
        Box::new(cursor::Cursor),
        Box::new(move_to::MoveTo),
        Box::new(click::Click),
        Box::new(double_click::DoubleClick),
        Box::new(long_press::LongPress),
        Box::new(drag::Drag),
        Box::new(type_text::TypeText),
        Box::new(press_key::PressKey),
        Box::new(wait::Wait),
        Box::new(replay_speed::ReplaySpeed),
        Box::new(text_overlay::Caption { name: "text_title", headline: true }),
        Box::new(text_overlay::Caption { name: "text_subtitle", headline: false }),
        Box::new(clear_overlays::ClearOverlays),
        Box::new(recording_start::RecordingStart),
        Box::new(recording_stop::RecordingStop),
    ]);
    tools
}

/// The JSON specs of every tool, for `tools/list` (no app needed — runs on the server thread).
pub fn specs() -> Vec<Value> {
    all().iter().map(|t| t.spec()).collect()
}

/// One parameter of a tool, for the script editor's autocomplete and the command help: its name,
/// type ("number", "string", "boolean", "array"), whether it must be given, what it means, and the
/// fixed values it takes (empty when any value goes).
pub struct ParamInfo {
    pub name: String,
    pub kind: String,
    pub required: bool,
    pub description: String,
    pub values: Vec<String>,
}

/// A tool described for the script editor and the command help: its name, description and
/// parameters (required ones first). Derived from the same specs the server advertises.
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub params: Vec<ParamInfo>,
    /// What a call gives back (see [`Tool::returns`]).
    pub returns: String,
}

impl ToolInfo {
    /// The parameter called `name`, if the tool has one.
    pub fn param(&self, name: &str) -> Option<&ParamInfo> {
        self.params.iter().find(|p| p.name == name)
    }
}

/// Every tool as [`ToolInfo`], in `tools/list` order. Built once (the tool set never changes while
/// the program runs), so the editor can ask for it every frame.
pub fn catalog() -> &'static [ToolInfo] {
    static CATALOG: std::sync::OnceLock<Vec<ToolInfo>> = std::sync::OnceLock::new();
    CATALOG.get_or_init(|| all().iter().map(|t| tool_info(&t.spec(), t.returns())).collect())
}

/// Read one tool spec (and what it returns) into a [`ToolInfo`].
fn tool_info(s: &Value, returns: &str) -> ToolInfo {
    let text = |v: &Value| v.as_str().unwrap_or_default().to_string();
    let required: Vec<String> = s["inputSchema"]["required"].as_array().map(|a| a.iter().map(text).collect()).unwrap_or_default();
    let mut params: Vec<ParamInfo> = s["inputSchema"]["properties"]
        .as_object()
        .map(|o| {
            o.iter()
                .map(|(name, p)| ParamInfo {
                    name: name.clone(),
                    kind: text(&p["type"]),
                    required: required.contains(name),
                    description: text(&p["description"]),
                    values: p["enum"].as_array().map(|a| a.iter().map(text).collect()).unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default();
    // required parameters first, in the order the spec lists them as required
    params.sort_by_key(|p| required.iter().position(|r| *r == p.name).unwrap_or(usize::MAX));
    ToolInfo { name: text(&s["name"]), description: text(&s["description"]), params, returns: returns.to_string() }
}

/// Run a named tool on the app (on the UI thread). Returns `None` for an unknown tool so the
/// dispatcher can report it.
pub fn dispatch(app: &mut App, ctx: &egui::Context, name: &str, args: &Value) -> Option<ToolResult> {
    all().iter().find(|t| t.name() == name).map(|t| t.call(app, ctx, args))
}
