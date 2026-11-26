//! The agent API: the operations the [`crate::mcp`] tools carry out, as named `pub(crate)` methods
//! on [`App`]. The MCP tool files (one per command) call these instead of touching the app's
//! private state, so the tool set stays generic and the behaviour lives here with the app — the
//! same split GridCraft draws between its `mcp` crate and its engine commands.
//!
//! Pointer and keyboard actions are played through the visible [`super::demo`] cursor so a recording
//! shows them; the data operations act straight on the model and recompute.

use super::demo::Step;
use super::{App, DrawTool, ViewMode};
use crate::toolpath::Job;
use egui::{Key, Modifiers, PointerButton, Pos2};
use shape_edit::{Circle, Editable, Ellipse, Polygon, Rect as RectShape, Shape, Text};

/// Turn seconds into animation frames (60 fps), at least one.
fn frames(seconds: f64) -> u32 {
    (seconds * 60.0).round().max(1.0) as u32
}

impl App {
    // ---------------- pointer & keyboard (played through the visible cursor) ----------------

    /// Show or hide the virtual cursor.
    pub(crate) fn agent_cursor(&mut self, on: bool) -> Result<String, String> {
        self.demo.on = on;
        Ok(format!("cursor {}", if on { "on" } else { "off" }))
    }

    /// Glide the cursor to a screen point.
    pub(crate) fn agent_move_to(&mut self, p: Pos2) -> Result<String, String> {
        self.demo.push([Step::MoveTo(p)]);
        Ok("moving".into())
    }

    /// Click at a point with the left or right button.
    pub(crate) fn agent_click(&mut self, p: Pos2, right: bool) -> Result<String, String> {
        let b = if right { PointerButton::Secondary } else { PointerButton::Primary };
        self.demo.push([Step::MoveTo(p), Step::Press(b), Step::Wait(6), Step::Release(b), Step::Wait(4)]);
        Ok("clicked".into())
    }

    /// Double-click at a point.
    pub(crate) fn agent_double_click(&mut self, p: Pos2) -> Result<String, String> {
        let b = PointerButton::Primary;
        self.demo.push([Step::MoveTo(p), Step::Press(b), Step::Wait(3), Step::Release(b), Step::Wait(2), Step::Press(b), Step::Wait(3), Step::Release(b), Step::Wait(4)]);
        Ok("double-clicked".into())
    }

    /// Long-press at a point (opens a value for editing, like holding the mouse).
    pub(crate) fn agent_long_press(&mut self, p: Pos2) -> Result<String, String> {
        self.demo.push([Step::MoveTo(p), Step::Hold(34), Step::Wait(4)]);
        Ok("long-pressed".into())
    }

    /// Press at the first point, drag to the second and release.
    pub(crate) fn agent_drag(&mut self, from: Pos2, to: Pos2) -> Result<String, String> {
        self.demo.push([Step::MoveTo(from), Step::Press(PointerButton::Primary), Step::Wait(3), Step::MoveTo(to), Step::Wait(3), Step::Release(PointerButton::Primary), Step::Wait(4)]);
        Ok("dragged".into())
    }

    /// Type text into the focused field.
    pub(crate) fn agent_type_text(&mut self, text: &str) -> Result<String, String> {
        self.demo.push([Step::Type(text.to_string()), Step::Wait(2)]);
        Ok("typed".into())
    }

    /// Press a key with modifiers. Refuses the quit shortcut: the MCP must never close the program.
    pub(crate) fn agent_press_key(&mut self, key: Key, m: Modifiers) -> Result<String, String> {
        if m.command && key == Key::Q {
            return Err("quitting the program is not allowed over the MCP".into());
        }
        self.demo.push([Step::Key(key, m), Step::Wait(2)]);
        Ok("key".into())
    }

    /// Open or close a panel — "tools" (the tool column on the left) or "milling" (the toolpath
    /// panel on the right) — the way a person does: the visible mouse glides to the panel's fold
    /// button (the chevron) and clicks it; the click itself folds or unfolds the panel. Nothing is
    /// clicked when the panel already is as asked. Returns {"panel", "open", "changed"}.
    pub(crate) fn agent_panel(&mut self, panel: &'static str, open: bool) -> Result<String, String> {
        let now = match panel {
            "tools" => self.tools_open,
            "milling" => self.paths_open,
            other => return Err(format!("unknown panel {other}")),
        };
        if now == open {
            return Ok(serde_json::json!({"panel": panel, "open": open, "changed": false}).to_string());
        }
        let key = if panel == "tools" { "panel.tools" } else { "panel.milling" };
        let Some(button) = self.control_rects.get(key).copied() else {
            return Err(format!("the {panel} panel's fold button is not on screen"));
        };
        self.run_script(super::agent_script::Script::new().move_to(button.center()).click());
        Ok(serde_json::json!({"panel": panel, "open": open, "changed": true}).to_string())
    }

    /// Click a button of the milling panel (the right sidebar) — `control` is "milling.preview",
    /// "milling.simulate" or "milling.export" — the way a person does: if the panel is folded, the
    /// mouse first clicks its chevron to open it, then glides to the button and clicks it. The
    /// button's own click does the work, so the result appears only once the mouse is there.
    pub(crate) fn agent_milling_button(&mut self, control: &'static str, what: &str) -> Result<String, String> {
        let mut script = super::agent_script::Script::new();
        if !self.paths_open {
            let Some(chevron) = self.control_rects.get("panel.milling").copied() else {
                return Err("the milling panel's fold button is not on screen".into());
            };
            script = script.move_to(chevron.center()).click();
        }
        // the button is looked up when the mouse sets off for it: the panel may only just have opened
        self.run_script(script.move_to_control(control).click());
        Ok(format!("clicking {what} in the toolpaths panel"))
    }

    /// Turn the "G-code motion in 3D" overlay on or off by clicking its checkbox in the milling
    /// panel (the panel is opened first if folded). Nothing is clicked when it is already as asked.
    pub(crate) fn agent_gcode_motion(&mut self, on: bool) -> Result<String, String> {
        if self.paths_in_3d == on {
            return Ok(serde_json::json!({"gcode_motion": on, "changed": false}).to_string());
        }
        let mut script = super::agent_script::Script::new();
        if !self.paths_open {
            let Some(chevron) = self.control_rects.get("panel.milling").copied() else {
                return Err("the milling panel's fold button is not on screen".into());
            };
            script = script.move_to(chevron.center()).click();
        }
        self.run_script(script.move_to_control("milling.gcode_motion").click());
        Ok(serde_json::json!({"gcode_motion": on, "changed": true}).to_string())
    }

    /// Set the replay speed of the visible cursor (1 normal, 2 twice as fast, 0.5 half as fast), or
    /// just report it when `speed` is `None`. Returns {"speed": …}, the speed now in use.
    pub(crate) fn agent_replay_speed(&mut self, speed: Option<f64>) -> Result<String, String> {
        if let Some(s) = speed {
            self.demo.set_speed(s as f32);
        }
        Ok(serde_json::json!({"speed": self.demo.speed()}).to_string())
    }

    /// Pause for a number of seconds (a beat for the viewer).
    pub(crate) fn agent_wait(&mut self, seconds: f64) -> Result<String, String> {
        self.demo.push([Step::Wait(frames(seconds))]);
        Ok("waiting".into())
    }

    // ---------------- commands (run a catalog command by id) ----------------

    /// Run a command from [`crate::mcp::commands`] by its stable id. When the command lives in a
    /// real menu and that menu is on screen, it is carried out **through the visible menu** — the
    /// cursor opens the menu and clicks the item (see [`super::menu_drive`]) — so a recording shows
    /// a person using the menu. Otherwise (or as a fallback) the action runs directly. Commands that
    /// take arguments are tools of their own.
    pub(crate) fn run_command(&mut self, id: &str) -> Result<String, String> {
        let Some(cmd) = crate::mcp::commands::find(id) else {
            return Err(format!("unknown command {id}"));
        };
        // the buttons of the toolpaths panel on the right are clicked there, where a person finds
        // them: the mouse opens the panel if needed and clicks the button
        if cmd.path.key() == crate::mcp::commands::TOOLPATHS_PANEL {
            return match id {
                "toolpath.simulate" => self.agent_milling_button("milling.simulate", &cmd.label.get()),
                "toolpath.gcodeMotionOn" => self.agent_gcode_motion(true),
                "toolpath.gcodeMotionOff" => self.agent_gcode_motion(false),
                _ => self.agent_milling_button("milling.preview", &cmd.label.get()),
            };
        }
        // menu commands are driven through the visible menu when it is available
        if let Some(menu) = crate::mcp::commands::menu_name(cmd) {
            if self.start_menu_drive(cmd.id, menu) {
                return Ok(format!("opening {} › {} in the menu", t!(menu), cmd.label.get()));
            }
        }
        self.run_command_direct(id)
    }

    /// Carry out a command's effect directly, without the visible menu (the fallback, and the path
    /// for commands that have no menu).
    pub(crate) fn run_command_direct(&mut self, id: &str) -> Result<String, String> {
        use super::shortcuts::Action::*;
        if crate::mcp::commands::find(id).is_none() {
            return Err(format!("unknown command {id}"));
        }
        match id {
            "file.new" => self.run(New),
            "file.open" => self.run(Open),
            "file.save" => self.run(Save),
            "file.saveAs" => self.run(SaveAs),
            "file.exportToolpaths" => self.export_dialog = true,
            "file.startPage" => self.start_page = true,
            "edit.undo" => self.run(Undo),
            "edit.redo" => self.run(Redo),
            "edit.cut" => self.run(Cut),
            "edit.copy" => self.run(Copy),
            "edit.paste" => self.run(Paste),
            "edit.duplicate" => self.run(Duplicate),
            "edit.delete" => self.run(Delete),
            "edit.group" => self.run(Group),
            "edit.ungroup" => self.run(Ungroup),
            "edit.selectAll" => self.run(SelectAll),
            "edit.nodeEdit" => self.run(NodeEdit),
            "edit.jobSetup" => self.job_dialog = Some((self.job.clone(), false)),
            "edit.cancel" => self.run(Cancel),
            "tool.select" => self.run(Select),
            "tool.line" => self.run(Line),
            "tool.spline" => self.run(SplineTool),
            "tool.rectangle" => self.run(Rectangle),
            "tool.circle" => self.run(Circle),
            "tool.offset" => self.run(Offset),
            "tool.dimension" => self.run(Dimension),
            "tool.align" => self.run(Align),
            "view.2d" => self.run(View2D),
            "view.3d" => self.run(View3D),
            "view.fit" => self.run(FitView),
            "view.toggleDimensions" => self.show_dims = !self.show_dims,
            "toolpath.profile" => self.open_editor("profile"),
            "toolpath.pocket" => self.open_editor("pocket"),
            "toolpath.drill" => self.open_editor("drill"),
            "toolpath.vcarve" => self.open_editor("vcarve"),
            "toolpath.cutout" => self.open_editor("cutout"),
            "toolpath.recomputeAll" => self.recompute_all(),
            "toolpath.preview" => self.start_preview(false),
            "toolpath.simulate" => self.start_preview(true),
            other => return Err(format!("command {other} has no binding")),
        }
        Ok(format!("ran {id}"))
    }

    // ---------------- shapes ----------------

    fn add_and_select(&mut self, shape: impl Into<Shape>) -> u64 {
        let id = self.add_shape(shape);
        self.edit.selection = vec![id];
        id
    }

    /// Add a rectangle; returns its id.
    pub(crate) fn agent_rectangle(&mut self, cx: f64, cy: f64, w: f64, h: f64, corner_radius: Option<f64>) -> u64 {
        let mut r = RectShape::new([cx, cy], w, h);
        if let Some(cr) = corner_radius {
            r.corner_radius = cr;
        }
        self.add_and_select(r)
    }

    /// Add a circle; returns its id.
    pub(crate) fn agent_circle(&mut self, cx: f64, cy: f64, radius: f64) -> u64 {
        self.add_and_select(Circle::new([cx, cy], radius))
    }

    /// Add an ellipse by full width/height; returns its id.
    pub(crate) fn agent_ellipse(&mut self, cx: f64, cy: f64, w: f64, h: f64) -> u64 {
        self.add_and_select(Ellipse::new([cx, cy], w / 2.0, h / 2.0))
    }

    /// Add a regular polygon; returns its id.
    pub(crate) fn agent_polygon(&mut self, cx: f64, cy: f64, radius: f64, sides: u32) -> u64 {
        self.add_and_select(Polygon::new([cx, cy], radius, sides.max(3)))
    }

    /// Add text; returns its id.
    pub(crate) fn agent_text(&mut self, cx: f64, cy: f64, text: &str, height: f64) -> u64 {
        self.add_and_select(Text::new([cx, cy], text, height))
    }

    /// Delete shapes by id.
    pub(crate) fn agent_delete_shapes(&mut self, ids: &[u64]) -> Result<String, String> {
        self.push_undo();
        self.shapes.retain(|s| !ids.contains(&s.id));
        self.sync_patterns();
        self.edit.retain_existing(&self.shapes);
        self.recompute_all();
        Ok(format!("deleted {}", ids.len()))
    }

    /// Select shapes by id (clears the previous selection).
    pub(crate) fn agent_select(&mut self, ids: &[u64]) -> Result<String, String> {
        self.edit.selection = self.shapes.iter().filter(|s| ids.contains(&s.id)).map(|s| s.id).collect();
        Ok(format!("selected {}", self.edit.selection.len()))
    }

    /// Drag shape `id` with the visible mouse so the top-left corner of its outline box ends at
    /// (`left`, `top`) mm — like a person: the cursor glides to the shape (its centre, or a point on
    /// its outline where the centre does not pick it), presses, glides to the drop point and lets go.
    /// The move itself is the app's own drag, so the shape snaps to grid and construction lines (and
    /// attaches to a line it lands on) exactly as by hand; the final place can therefore differ a
    /// little from the one asked for. Returns {"id", "from", "to"}.
    /// A point in mm where the mouse can grab shape `id`: its centre when that picks it
    /// (rectangles, circles, ellipses, polygons, text), else the middle of its first outline segment
    /// (vectors are picked on their line). `None` when the shape has no outline.
    fn shape_grab_point(&self, id: u64) -> Option<[f64; 2]> {
        let item = self.shapes.iter().find(|s| s.id == id)?;
        let (lo, hi) = item.bounds()?;
        let tolerance = 3.0 / self.zoom as f64;
        let centre = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
        if item.hit(centre, tolerance) {
            return Some(centre);
        }
        let first = item.shape.contours().into_iter().find_map(|c| c.segments().next());
        first.map(|(a, b)| [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0])
    }

    pub(crate) fn agent_drag_shape(&mut self, id: u64, left: f64, top: f64) -> Result<String, String> {
        let item = self.shapes.iter().find(|s| s.id == id).ok_or_else(|| format!("no shape with id {id}"))?;
        if item.hidden || item.frozen {
            return Err(format!("shape {id} is hidden or locked and cannot be dragged"));
        }
        let (lo, hi) = item.bounds().ok_or_else(|| format!("shape {id} has no size"))?;
        // where to grab it: the centre where that picks it (rectangles, circles, ellipses, polygons,
        // text), else the middle of its first outline segment — vectors are picked on their line
        let tolerance = 3.0 / self.zoom as f64;
        let centre = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
        let grab = if item.hit(centre, tolerance) {
            centre
        } else {
            let first = item.shape.contours().into_iter().find_map(|c| c.segments().next());
            first.map(|(a, b)| [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0]).ok_or_else(|| format!("shape {id} has no outline to grab"))?
        };
        // the box's top-left is (lo.x, hi.y): y grows upwards
        let drop = [grab[0] + left - lo[0], grab[1] + top - hi[1]];
        let (from, to) = (self.to_screen(grab), self.to_screen(drop));
        if !self.canvas_rect.contains(from) || !self.canvas_rect.contains(to) {
            return Err("the shape or its target is outside the visible drawing; run view_fit first".into());
        }
        let script = super::agent_script::Script::new()
            // a drag moves shapes in the 2D view with the select tool
            .act(|app, _| {
                app.mode = ViewMode::TwoD;
                if app.tool != DrawTool::Select {
                    app.pick_tool(DrawTool::Select);
                }
            })
            .move_to(from)
            .press()
            .move_to(to)
            .release();
        self.run_script(script);
        Ok(serde_json::json!({"id": id, "from": {"left": lo[0], "top": hi[1]}, "to": {"left": left, "top": top}}).to_string())
    }

    /// The ids of the current selection (for tools that report what they changed).
    pub(crate) fn selection_ids(&self) -> Vec<u64> {
        self.edit.selection.clone()
    }

    /// Create or change a document variable.
    pub(crate) fn agent_set_variable(&mut self, name: &str, expression: &str) -> Result<String, String> {
        self.vars.set(name, expression);
        self.vars_dirty = true;
        Ok(format!("{name} = {expression}"))
    }

    // ---------------- toolpaths ----------------

    /// Add a milling toolpath the way a person does — all through the UI, nothing in the background:
    /// the cursor goes to each shape and clicks it (the first plain, the rest with Shift, so they
    /// build one selection), opens the milling panel if it is folded, clicks the operation's button
    /// (which opens its editor, pre-filled with the material's depths and cutter), types `cut_depth`
    /// and the per-pass depth `max_depth_per_pass` into their fields when given (each a number or a
    /// variable/formula like `platte_d`), and clicks "Anwenden". Returns {"kind", "shape_ids"}.
    pub(crate) fn agent_add_toolpath(&mut self, kind: &str, ids: Vec<u64>, cut_depth: Option<String>, max_depth_per_pass: Option<String>) -> Result<String, String> {
        if !matches!(kind, "profile" | "cutout" | "pocket" | "slot" | "drill" | "vcarve") {
            return Err("kind must be profile, cutout, pocket, slot, drill or vcarve".into());
        }
        if ids.is_empty() {
            return Err("shape_ids must list at least one shape".into());
        }
        let cut_depth = cut_depth.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        let max_depth_per_pass = max_depth_per_pass.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        // the shapes' grab points on screen (asked from the shapes, not guessed); all must be visible
        let mut points = Vec::with_capacity(ids.len());
        for &id in &ids {
            let p = self.shape_grab_point(id).ok_or_else(|| format!("shape {id} cannot be picked (unknown or has no outline)"))?;
            let at = self.to_screen(p);
            if !self.canvas_rect.contains(at) {
                return Err(format!("shape {id} is outside the visible drawing; run view_fit first"));
            }
            points.push(at);
        }
        let op = super::layout::op_control(kind);
        let mut script = super::agent_script::Script::new()
            // selecting and the operation buttons live in the 2D view with the select tool
            .act(|app, _| {
                app.mode = ViewMode::TwoD;
                if app.tool != DrawTool::Select {
                    app.pick_tool(DrawTool::Select);
                }
            })
            // clear any selection first (Escape), so only the wanted shapes end up selected
            .escape();
        // click each shape: the first replaces the selection, the rest add to it with Shift
        for (i, at) in points.iter().enumerate() {
            script = script.move_to(*at);
            if i == 1 {
                script = script.shift();
            }
            script = script.click();
        }
        if !points.is_empty() {
            script = script.no_mods();
        }
        // open the milling panel if it is folded, then click the operation to open its editor
        if !self.paths_open {
            if let Some(chevron) = self.control_rects.get("panel.milling").copied() {
                script = script.move_to(chevron.center()).click();
            }
        }
        script = script.move_to_control(op).click();
        // type the depths into their fields (through the UI) — a number or a variable/formula —
        // then apply
        if let Some(d) = &cut_depth {
            script = script.type_into_control("editor.cut_depth", d.clone());
        }
        if let Some(d) = &max_depth_per_pass {
            script = script.type_into_control("editor.stepdown", d.clone());
        }
        script = script.move_to_control("editor.apply").click();
        self.run_script(script);
        Ok(serde_json::json!({"kind": kind, "shape_ids": ids, "cut_depth": cut_depth, "max_depth_per_pass": max_depth_per_pass}).to_string())
    }

    /// Delete a toolpath by its index.
    pub(crate) fn agent_delete_toolpath(&mut self, index: i64) -> Result<String, String> {
        if index >= 0 && (index as usize) < self.toolpaths.len() {
            let name = self.toolpaths.remove(index as usize).name;
            self.recompute_all();
            Ok(format!("deleted {name}"))
        } else {
            Err("index out of range".into())
        }
    }

    // ---------------- job, files, view, windows ----------------

    /// Start a new job with the given size, thickness and material.
    pub(crate) fn agent_new_job(&mut self, width: f64, height: f64, thickness: f64, material: Option<&str>) -> Result<String, String> {
        let mut job = Job { width, height, thickness, ..Job::default() };
        if let Some(m) = material {
            job.material = m.to_string();
        }
        self.new_job(job);
        self.start_page = false;
        Ok("new job".into())
    }

    /// The material keys available for a job.
    pub(crate) fn agent_list_materials(&self) -> String {
        serde_json::json!({"materials": self.materials.sorted().iter().map(|m| m.key.clone()).collect::<Vec<_>>()}).to_string()
    }

    /// The cutters available for the current material.
    pub(crate) fn agent_list_tools(&self) -> String {
        let tools: Vec<serde_json::Value> = self.lib.tools_for(&self.job.material).iter().map(|t| serde_json::json!({"name": t.name, "kind": format!("{:?}", t.kind), "diameter": t.diameter})).collect();
        serde_json::json!({"material": self.job.material, "tools": tools}).to_string()
    }

    /// Save the project to a path.
    pub(crate) fn agent_save_project(&self, path: &str) -> Result<String, String> {
        self.write_project(std::path::Path::new(path)).map(|_| format!("saved {path}"))
    }

    /// Export the visible toolpaths as G-code to a path.
    pub(crate) fn agent_export_gcode(&self, path: &str) -> Result<String, String> {
        self.export_gcode_to(std::path::Path::new(path))
    }

    /// Switch to the 2D or 3D view the way a person does: the visible mouse glides to the "2D" or
    /// "3D" half of the slide toggle in the canvas bar and clicks it; the click switches the view
    /// (and the 3D view builds its cut preview when it first shows). Nothing is clicked when the view
    /// already is the one asked for. Returns {"mode", "changed"}.
    pub(crate) fn agent_view(&mut self, three_d: bool) -> Result<String, String> {
        let mode = if three_d { "3d" } else { "2d" };
        if (self.mode == ViewMode::ThreeD) == three_d {
            return Ok(serde_json::json!({"mode": mode, "changed": false}).to_string());
        }
        let Some(toggle) = self.control_rects.get("view.toggle").copied() else {
            return Err("the 2D/3D toggle is not on screen".into());
        };
        // the toggle has two halves, "2D" on the left and "3D" on the right: aim at the label asked for
        let half = toggle.width() / 2.0;
        let at = egui::pos2(toggle.left() + half * if three_d { 1.5 } else { 0.5 }, toggle.center().y);
        self.run_script(super::agent_script::Script::new().move_to(at).click());
        Ok(serde_json::json!({"mode": mode, "changed": true}).to_string())
    }

    /// Report the window, and — when `width`/`height` (logical points) are given — resize it first.
    /// A video wants a steady size, so set it before `recording_start`.
    pub(crate) fn agent_window(&self, ctx: &egui::Context, width: Option<f64>, height: Option<f64>) -> String {
        if width.is_some() || height.is_some() {
            let r = ctx.screen_rect();
            let w = width.map(|w| w as f32).unwrap_or(r.width()).clamp(400.0, 8000.0);
            let h = height.map(|h| h as f32).unwrap_or(r.height()).clamp(300.0, 8000.0);
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(w, h)));
            ctx.request_repaint();
        }
        let r = ctx.screen_rect();
        let port = self.mcp.as_ref().map(|m| m.port).unwrap_or(0);
        serde_json::json!({
            "width": r.width(),
            "height": r.height(),
            "busy": self.demo.busy() || self.menu_drive_busy() || self.script_busy(),
            "port": port,
            "panels": {"tools": self.tools_open, "milling": self.paths_open},
            "view": if self.mode == ViewMode::ThreeD { "3d" } else { "2d" },
            "replay_speed": self.demo.speed(),
            "recording": self.recorder.as_ref().map(|r| r.path().display().to_string()),
        })
        .to_string()
    }

    /// Start recording the window to an `.mp4` file at `fps` frames per second (default
    /// [`DEFAULT_FPS`](crate::recording::DEFAULT_FPS)), the longest side at most `max_side` pixels.
    pub(crate) fn agent_recording_start(&mut self, path: &str, fps: Option<f64>, max_side: Option<f64>) -> Result<String, String> {
        if self.recorder.is_some() {
            return Err("a recording is already running; stop it first with recording_stop".into());
        }
        let fps = fps.map(|f| f.round() as u32).unwrap_or(crate::recording::DEFAULT_FPS);
        let max_side = max_side.map(|m| m.round() as u32).unwrap_or(crate::recording::DEFAULT_MAX_SIDE);
        let rec = crate::recording::Recorder::start(std::path::Path::new(path), fps, max_side)?;
        self.recorder = Some(rec);
        Ok(serde_json::json!({"recording": path, "fps": fps}).to_string())
    }

    /// Stop the recording and close the video file.
    pub(crate) fn agent_recording_stop(&mut self) -> Result<String, String> {
        let rec = self.recorder.take().ok_or("no recording is running")?;
        let s = rec.stop()?;
        Ok(serde_json::json!({"file": s.path.display().to_string(), "frames": s.frames, "seconds": s.seconds, "width": s.width, "height": s.height}).to_string())
    }

    /// Open or close the in-app Demo & remote-control window.
    pub(crate) fn agent_demo_window(&mut self, open: bool) -> Result<String, String> {
        self.demo_open = open;
        Ok(format!("demo window {}", if open { "open" } else { "closed" }))
    }

    /// Open or close the script window (its own OS window).
    pub(crate) fn agent_script_window(&mut self, open: bool) -> Result<String, String> {
        self.script_open = open;
        Ok(format!("script window {}", if open { "open" } else { "closed" }))
    }

    /// Save a PNG of the window (without closing the app), taken at once — the start-up screenshot
    /// waits 110 frames for the window to settle; a running app is settled, so the frame counter
    /// starts just short of that and the picture shows the moment it was asked for.
    pub(crate) fn agent_screenshot(&mut self, path: &str) -> Result<String, String> {
        self.shot = Some((std::path::PathBuf::from(path), 109, false));
        Ok(format!("saving to {path}"))
    }

    /// A JSON summary of the drawing for the `state` tool: job, shapes and toolpaths.
    pub(crate) fn agent_state(&self) -> String {
        let shapes: Vec<serde_json::Value> = self.shapes.iter().map(|s| {
            let kind = match &s.shape {
                Shape::Rect(_) => "rectangle",
                Shape::Circle(_) => "circle",
                Shape::Ellipse(_) => "ellipse",
                Shape::Polygon(_) => "polygon",
                Shape::Text(_) => "text",
                Shape::Path(_) => "path",
                Shape::Spline(_) => "spline",
                Shape::Compound(_) => "compound",
                Shape::RefImage(_) => "image",
            };
            let (c, w, h) = match s.bounds() {
                Some((lo, hi)) => ([(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0], hi[0] - lo[0], hi[1] - lo[1]),
                None => ([0.0, 0.0], 0.0, 0.0),
            };
            serde_json::json!({"id": s.id, "kind": kind, "cx": c[0], "cy": c[1], "width": w, "height": h})
        }).collect();
        let toolpaths: Vec<serde_json::Value> = self.toolpaths.iter().enumerate().map(|(i, tp)| {
            serde_json::json!({"index": i, "name": tp.name, "tool": tp.tool.name, "cut_depth": tp.cut_depth})
        }).collect();
        serde_json::json!({
            "job": {"width": self.job.width, "height": self.job.height, "thickness": self.job.thickness, "material": self.job.material},
            "selection": self.edit.selection,
            "shapes": shapes,
            "toolpaths": toolpaths,
        }).to_string()
    }

}

/// The catalog command id a menu [`Action`](super::shortcuts::Action) belongs to, so a drawn menu
/// item can record its rect under that id for the visible-menu driver. `None` for actions that are
/// not catalog commands (or are reached another way).
pub(super) fn cmd_id_for_action(action: super::shortcuts::Action) -> Option<&'static str> {
    use super::shortcuts::Action::*;
    Some(match action {
        New => "file.new",
        Open => "file.open",
        Save => "file.save",
        SaveAs => "file.saveAs",
        Undo => "edit.undo",
        Redo => "edit.redo",
        Cut => "edit.cut",
        Copy => "edit.copy",
        Paste => "edit.paste",
        Duplicate => "edit.duplicate",
        Delete => "edit.delete",
        Group => "edit.group",
        Ungroup => "edit.ungroup",
        SelectAll => "edit.selectAll",
        NodeEdit => "edit.nodeEdit",
        View2D => "view.2d",
        View3D => "view.3d",
        FitView => "view.fit",
        _ => return None,
    })
}

