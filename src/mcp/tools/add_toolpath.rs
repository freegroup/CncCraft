//! `add_toolpath`: create a milling toolpath, driven entirely through the UI — the mouse selects the
//! shapes, opens the operation's editor and clicks Anwenden. Returns the kind and shapes.

use crate::app::App;
use crate::mcp::tool::{num_array, one_of, spec, string, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct AddToolpath;

impl Tool for AddToolpath {
    fn name(&self) -> &'static str {
        "add_toolpath"
    }
    fn returns(&self) -> &'static str {
        "JSON {\"kind\", \"shape_ids\": [ids], \"cut_depth\", \"max_depth_per_pass\"} — the toolpath is computed once the mouse has clicked Anwenden; read it back with state."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Create a milling toolpath, all through the UI: the visible mouse clicks the shapes to select them (Shift for the extra ones), opens the milling panel if needed, clicks the operation's button, types the depths into the editor's fields when given, and clicks Anwenden. Fields left out keep the material's standard value. All shapes must be on screen (run view_fit first if needed).",
            json!({
                "kind": one_of(&["profile", "cutout", "pocket", "slot", "drill", "vcarve"], "The milling operation: profile (along the outline), cutout (cut the part free), pocket (clear the inside), slot, drill (holes) or vcarve."),
                "shape_ids": num_array("The ids of the shapes to cut (from a drawing tool's result or the state tool)."),
                "cut_depth": string("How deep the cut goes in total (the 'Zieltiefe' field): a number in mm or a variable/formula like platte_d or platte_d/2 (optional)."),
                "max_depth_per_pass": string("The most the cutter takes per pass (the 'Zustellung' field, 'Bohrtiefe je Hub' for drilling): a number in mm or a formula (optional)."),
            }),
            &["kind", "shape_ids"],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        match a.str("kind") {
            Some(kind) => ToolResult::from(app.agent_add_toolpath(kind, a.ids("shape_ids"), a.value_str("cut_depth"), a.value_str("max_depth_per_pass"))),
            None => ToolResult::err("need kind (profile, cutout, pocket, slot, drill or vcarve)"),
        }
    }
}
