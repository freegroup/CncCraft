//! `new_job`: start a new job with a size, thickness and material.

use crate::app::App;
use crate::mcp::tool::{num, spec, string, Args, Tool, ToolResult};
use serde_json::{json, Value};

pub struct NewJob;

impl Tool for NewJob {
    fn name(&self) -> &'static str {
        "new_job"
    }
    fn returns(&self) -> &'static str {
        "Text \"new job\"."
    }
    fn spec(&self) -> Value {
        spec(
            self.name(),
            "Start a new job with a size (mm), thickness (mm) and material key (e.g. Softwood, Hardwood, Plywood, MDF, Acrylic, Aluminium).",
            json!({
                "width": num("Width of the material in mm (x)."),
                "height": num("Height of the material in mm (y)."),
                "thickness": num("Material thickness in mm (default 18)."),
                "material": string("Material key from list_materials, e.g. Plywood or MDF."),
            }),
            &["width", "height"],
        )
    }
    fn call(&self, app: &mut App, _ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        ToolResult::from(app.agent_new_job(a.num("width").unwrap_or(300.0), a.num("height").unwrap_or(200.0), a.num("thickness").unwrap_or(18.0), a.str("material")))
    }
}
