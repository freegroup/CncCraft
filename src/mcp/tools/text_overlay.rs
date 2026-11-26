//! The caption commands, like the text in TikTok or tutorial videos: `text_headline` is a standing
//! banner (set once, kept — e.g. the app's name at the top), `text_subtitle` is the changing line
//! that says what is happening (replaced again and again). `text_overlay` is kept as an older name
//! for the subtitle. One type serves all three, since they differ only in which slot they set.

use crate::app::App;
use crate::mcp::tool::{num, one_of, spec, string, Args, Tool, ToolResult};
use serde_json::{json, Value};

/// One caption command for a slot ("headline" or "subtitle").
pub struct Caption {
    pub name: &'static str,
    pub headline: bool,
}

impl Tool for Caption {
    fn name(&self) -> &'static str {
        self.name
    }
    fn returns(&self) -> &'static str {
        "JSON {\"shown\": true, \"duration\": seconds}, or {\"shown\": false} when the text was empty (the caption is taken away)."
    }
    fn spec(&self) -> Value {
        let (what, defaults) = if self.headline {
            ("a standing title over the window (e.g. the app's name), kept until it is replaced or cleared", "top centre, 40 pt, no time limit")
        } else {
            ("the subtitle that says what is happening, replaced each time it is called", "bottom centre, 28 pt, 3 seconds")
        };
        let description = format!(
            "Show {what}, like the text in TikTok or tutorial videos: wrapped on a rounded box, fading in and out. An empty text removes it. It does not wait; add wait to hold the scene. Defaults: {defaults}. Headline and subtitle are independent — both can show at once."
        );
        spec(
            self.name,
            &description,
            json!({
                "text": string("The caption. An empty text takes the current one away."),
                "duration": num("How long it shows, in seconds (0 keeps it until replaced)."),
                "opacity": num("Overall opacity from 0 to 1 (default 1)."),
                "color": string("Text colour as #RRGGBB, #RRGGBBAA or r,g,b,a with 0-255 (default white)."),
                "bg_color": string("Box colour as #RRGGBB, #RRGGBBAA or r,g,b,a (default black at 70%). Also accepted as bg-color."),
                "font": num("Font size in points."),
                "vertical": one_of(&["top", "center", "bottom"], "Where it sits vertically."),
                "horizontal": one_of(&["left", "center", "right"], "Where it sits horizontally, and how its lines align (default center)."),
            }),
            &["text"],
        )
    }
    fn call(&self, app: &mut App, ctx: &egui::Context, args: &Value) -> ToolResult {
        let a = Args(args);
        let Some(text) = a.str("text") else { return ToolResult::err("need text (an empty text removes the caption)") };
        // the box colour is written bg_color or, as in CSS, bg-color
        let background = a.str("bg_color").or_else(|| a.str("bg-color"));
        let slot = if self.headline { crate::app::OverlaySlot::Title } else { crate::app::OverlaySlot::Subtitle };
        ToolResult::from(app.agent_set_overlay(ctx, slot, text, a.num("duration"), a.num("opacity"), a.str("color"), background, a.num("font"), a.str("vertical"), a.str("horizontal")))
    }
}
