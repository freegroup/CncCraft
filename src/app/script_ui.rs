//! The script window and the command help window. Both open as their **own OS windows** (egui
//! immediate viewports), so they can be moved anywhere on the desktop, including outside the main
//! window — handy while recording; on a backend without multiple viewports they fall back to
//! in-window panes. Their contents are the widgets of [`crate::scripting`]: the editor with
//! highlighting and autocomplete (`scripting/editor.rs`) and the help (`scripting/help.rs`); this
//! file only hosts them, with Play/Stop and the log of what ran.

use super::App;
use crate::app::dialogs;
use crate::theme::{ACCENT, INK};

/// The script offered when the window opens on an empty editor, so the format is obvious; an editor
/// the person has cleared stays empty while it is open, and is only offered the sample again when it
/// is reopened still empty.
const SAMPLE: &str = "# A short demo. Lines run top to bottom; the cursor is shown doing each step.\n\
view mode=2d\n\
panel_milling_close\n\
panel_tools_close\n\
\n\
window height=1200 width=900\n\
\n\
# recording_start file=/absolute/path/to/video.mp4\n\
\n\
new_job width=300 height=200 material=Plywood\n\
guides_delete_all\n\
text_title text=\"CncCraft — CAM for the hobbyist\"\n\
text_subtitle text=\"create a simple circle to carve\"\n\
\n\
g = guide_add axis=vertical offset=60\n\
\n\
c1 = draw_circle cx=155 cy=50 radius=20\n\
drag_shape id=$c1.id left=60 top=60\n\
\n\
c2 = draw_circle cx=130 cy=60 radius=20\n\
drag_shape id=$c2.id left=60 top=110\n\
\n\
c3 = draw_circle cx=160 cy=55 radius=20\n\
drag_shape id=$c3.id left=60 top=160\n\
\n\
guide_move id=$g.id offset=100\n\
wait seconds=1\n\
\n\
panel_tools_close\n\
panel_milling_open\n\
\n\
add_toolpath kind=profile shape_ids=[$c1.id] cut_depth=platte_d\n\
wait seconds=1\n\
add_toolpath kind=pocket shape_ids=[$c2.id]\n\
wait seconds=1\n\
add_toolpath kind=profile shape_ids=[$c3.id]\n\
wait seconds=1\n\
\n\
text_subtitle text=\"simulate the milling\"\n\
view mode=3d\n\
wait seconds=3\n\
\n\
toolpath_simulate\n\
panel_milling_close\n\
toolpath_gcode_motion_off\n\
\n\
wait seconds=3\n\
\n\
# recording_stop\n";

impl App {
    /// The script window, in its own OS window so it is not trapped inside the main one.
    pub(in crate::app) fn script_ui(&mut self, ctx: &egui::Context) {
        if !self.script_open {
            self.script_was_open = false;
            return;
        }
        // offer the sample only when the window is (re)opened on an empty editor; once open, an
        // editor the person has cleared stays empty — an empty start can be what they want
        if !self.script_was_open {
            self.script_was_open = true;
            if self.script_text.is_empty() {
                self.script_text = SAMPLE.to_string();
            }
        }
        let title = t!("Skript – Aufzeichnung steuern");
        let builder = egui::ViewportBuilder::default()
            .with_title(&title)
            .with_inner_size([680.0, 560.0])
            .with_min_inner_size([460.0, 320.0]);
        ctx.show_viewport_immediate(egui::ViewportId::from_hash_of("script_window"), builder, |ctx, class| {
            if class == egui::ViewportClass::Embedded {
                // this backend has no separate OS windows: show a movable in-window pane instead
                let mut open = self.script_open;
                egui::Window::new(&title)
                    .id(egui::Id::new("script_window_embedded"))
                    .open(&mut open)
                    .resizable(true)
                    .default_size([640.0, 480.0])
                    .frame(egui::Frame::window(&ctx.style()).fill(crate::theme::panel()))
                    .show(ctx, |ui| self.script_body(ui));
                self.script_open = open;
            } else {
                egui::CentralPanel::default()
                    .frame(egui::Frame::central_panel(&ctx.style()).fill(crate::theme::panel()))
                    .show(ctx, |ui| self.script_body(ui));
                // the OS window's close button
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.script_open = false;
                }
            }
        });
    }

    /// The contents of the script window (shared by the OS-window and the embedded fallback). The
    /// editor fills the window; the log and buttons sit at the bottom and stay put as it is resized.
    fn script_body(&mut self, ui: &mut egui::Ui) {
        let mut play: Option<String> = None;
        let mut stop = false;
        ui.label(egui::RichText::new(t!("Ein Kommando pro Zeile (dieselben wie über MCP). „name = kommando …“ merkt sich das Ergebnis; „$name.id“ nutzt es später. # oder // ist ein Kommentar. Strg+Leertaste zeigt Vorschläge.")).small().color(crate::theme::muted()));
        ui.add_space(6.0);

        // buttons pinned to the bottom, the log just above them, the editor fills the rest
        egui::TopBottomPanel::bottom("script_buttons").frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(0.0, 6.0))).show_inside(ui, |ui| {
            dialogs::dialog_buttons(ui, |ui| {
                if self.script_playing() {
                    if ui.add(egui::Button::new(egui::RichText::new(t!("⏹  Stopp")).color(INK)).fill(ACCENT)).clicked() {
                        stop = true;
                    }
                } else if ui.add(egui::Button::new(egui::RichText::new(t!("▶  Abspielen")).color(INK)).fill(ACCENT)).clicked() {
                    play = Some(self.script_text.clone());
                }
                if ui.button(t!("Schließen")).clicked() {
                    self.script_open = false;
                }
                if ui.button(t!("Befehlshilfe")).clicked() {
                    self.command_help_open = true;
                }
                // the agent guide (a "skill" file): everything an AI agent needs to write scripts
                let copied = self.agent_md_copied.is_some_and(|t| ui.input(|i| i.time) - t < 2.5);
                let label = if copied { t!("✓ kopiert") } else { "agent.md".to_string() };
                let tip = t!("Kopiert eine Anleitung für KI-Agenten in die Zwischenablage: was diese Skriptsprache ist, alle Befehle mit Parametern und Rückgabewerten, Regeln und Beispielskripte.");
                if ui.button(label).on_hover_text(tip).clicked() {
                    ui.ctx().copy_text(crate::scripting::skill::agent_md());
                    self.agent_md_copied = Some(ui.input(|i| i.time));
                }
                if copied {
                    ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
                }
                // the replay speed on the left of the row (the same as the replay_speed command)
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(t!("Tempo")).color(crate::theme::ink()));
                    let mut speed = self.demo.speed();
                    let slider = egui::Slider::new(&mut speed, super::demo::SPEED_RANGE).logarithmic(true).fixed_decimals(2).suffix("×");
                    if ui.add(slider).on_hover_text(t!("Wie schnell die sichtbare Maus abspielt: 1× normal, 2× doppelt so schnell, 0,5× halb so schnell.")).changed() {
                        self.demo.set_speed(speed);
                    }
                });
            });
        });
        if !self.script.log.is_empty() {
            egui::TopBottomPanel::bottom("script_log").resizable(true).default_height(130.0).frame(egui::Frame::none()).show_inside(ui, |ui| {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(t!("Protokoll")).strong().color(crate::theme::ink()));
                egui::ScrollArea::vertical().id_salt("script_log_scroll").auto_shrink([false, false]).stick_to_bottom(true).show(ui, |ui| {
                    for line in &self.script.log {
                        let colour = if line.error { crate::theme::warning() } else { crate::theme::muted() };
                        ui.label(egui::RichText::new(format!("{}: {}", line.line_no, line.text)).monospace().color(colour));
                    }
                });
            });
        }
        // the editor fills the remaining space
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| crate::scripting::editor::editor_ui(ui, &mut self.script_text, &mut self.script_editor));

        if let Some(text) = play {
            self.script_play(&text);
        }
        if stop {
            self.script_stop();
        }
    }

    /// The command help window (every command, its parameters and what they mean), when open.
    /// "In Skript einfügen" puts the command's ready line into the script.
    pub(in crate::app) fn command_help_ui(&mut self, ctx: &egui::Context) {
        if !self.command_help_open {
            return;
        }
        let title = t!("Befehlshilfe");
        let builder = egui::ViewportBuilder::default().with_title(&title).with_inner_size([720.0, 640.0]).with_min_inner_size([420.0, 300.0]);
        ctx.show_viewport_immediate(egui::ViewportId::from_hash_of("command_help"), builder, |ctx, class| {
            let mut insert = None;
            if class == egui::ViewportClass::Embedded {
                // no separate OS windows on this backend: a movable pane instead
                let mut open = self.command_help_open;
                egui::Window::new(&title)
                    .id(egui::Id::new("command_help_embedded"))
                    .open(&mut open)
                    .resizable(true)
                    .default_size([620.0, 520.0])
                    .show(ctx, |ui| insert = crate::scripting::help::help_ui(ui, &mut self.command_help_filter));
                self.command_help_open = open;
            } else {
                egui::CentralPanel::default()
                    .frame(egui::Frame::central_panel(&ctx.style()).fill(crate::theme::panel()))
                    .show(ctx, |ui| insert = crate::scripting::help::help_ui(ui, &mut self.command_help_filter));
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.command_help_open = false;
                }
            }
            if let Some(line) = insert {
                self.script_open = true;
                crate::scripting::editor::insert_line(ctx, &mut self.script_text, &mut self.script_editor, &line);
            }
        });
    }
}
