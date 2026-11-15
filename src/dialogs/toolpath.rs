//! The window that creates or edits one toolpath: name, vectors and depths on top, then the
//! tool, the strategy and the travel heights.

use crate::app::*;
use egui::Pos2;

/// The editor's width: wide rather than tall.
const EDITOR_WIDTH: f32 = 760.0;

impl App {
    /// The editor of a toolpath as its own small window (a double click on a path, or the pencil
    /// on its row, or one of the operation buttons), so the list stays short.
    pub(in crate::app) fn editor_window(&mut self, ctx: &egui::Context) {
        if self.editor.is_none() {
            self.editor_layer = None;
            return;
        }
        let screen = ctx.screen_rect();
        // a title like every dialog's: new or edited, and which kind of toolpath
        let title = self.editor.as_ref().map(|ed| {
            let kind = if ed.kind == "cutout" { t!("Ausschnitt") } else { ed.tp.strategy.label() };
            if ed.index.is_some() { t!("Werkzeugweg bearbeiten – %{kind}", kind = kind) } else { t!("Neuer Werkzeugweg – %{kind}", kind = kind) }
        });
        let title = title.unwrap_or_default();
        let mut close = false;
        let shown = dialogs::window("toolpath_editor_window", ctx)
            .default_pos(Pos2::new((screen.right() - 360.0 - 800.0).max(40.0), 80.0))
            .show(ctx, |ui| {
                // the width first, so the header's line ends where the contents do
                ui.set_width(EDITOR_WIDTH);
                close = dialogs::header(ui, &title);
                self.editor_ui(ui);
            });
        // the cross does what "Schließen" does: the window goes, nothing is computed
        if close {
            self.editor = None;
            return;
        }
        if let Some(shown) = shown {
            self.editor_layer = Some(shown.response.layer_id);
            ctx.move_to_top(shown.response.layer_id);
        }
    }

    pub(in crate::app) fn editor_ui(&mut self, ui: &mut egui::Ui) {
        let mut close = false;
        let mut apply = false;
        let mut open_tool_dlg = false;
        let sel_count = self.edit.selection.len();
        let tools: Vec<Tool> = self.lib.tools_for(&self.job.material).into_iter().cloned().collect();
        let thickness = self.job.thickness;
        // for the depth fields: the variables they may use, and their values
        let names = self.var_names();
        let resolved = self.vars_resolved.clone();
        // the job's travel heights, shown here too (they hold for every toolpath)
        let (mut safe_z, mut home_z) = (self.job.safe_z, self.job.home_z);
        let Some(ed) = &mut self.editor else { return };
        ui.vertical(|ui| {
            // wide rather than tall: the essentials on top, then two columns of groups
            ui.set_width(EDITOR_WIDTH);
            let tp = &mut ed.tp;
            let (depth_text, depth_input) = (&mut ed.depth_text, &mut ed.depth_input);
            // a click on a suggestion of last frame's list goes into its field
            if let Some((field, name)) = depth_input.click(ui.ctx()) {
                depth_text[field] = shape_edit::vars::apply_completion(&depth_text[field], &name);
            }
            fn mm(v: &mut f64, max: f64) -> egui::DragValue<'_> {
                egui::DragValue::new(v).speed(0.05).range(0.0..=max).max_decimals(3).suffix(" mm")
            }
            // one heading per group of fields, all groups with the same label column
            fn group(ui: &mut egui::Ui, title: &str, id: &str, add: impl FnOnce(&mut egui::Ui)) {
                group_down_to(ui, title, id, None, add);
            }
            // a group whose frame reaches down to `bottom` (a screen position), if given
            fn group_down_to(ui: &mut egui::Ui, title: &str, id: &str, bottom: Option<f32>, add: impl FnOnce(&mut egui::Ui)) {
                dialog_group(ui, title, |ui| {
                    if let Some(b) = bottom {
                        ui.set_min_height(b - ui.min_rect().top());
                    }
                    egui::Grid::new(id).num_columns(2).spacing([8.0, 4.0]).min_col_width(LABELS).show(ui, add);
                });
            }
            // the label column of every group the same width, so all fields line up (the groups'
            // frames indent their content by 13 px, the top rows have no frame)
            const LABELS: f32 = 104.0;
            const COLUMN_GAP: f32 = 16.0;
            let column = (EDITOR_WIDTH - COLUMN_GAP) / 2.0;
            let gap = |ui: &mut egui::Ui| ui.add_space(COLUMN_GAP - ui.spacing().item_spacing.x);

            // on top, without a frame: what the toolpath is called, which vectors it cuts and how deep
            let cut_label = if matches!(tp.strategy, Strategy::VCarve { .. }) { t!("Max. Tiefe") } else { t!("Zieltiefe") };
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(column);
                    egui::Grid::new("tp_top_left").num_columns(2).spacing([8.0, 6.0]).min_col_width(LABELS + 13.0).show(ui, |ui| {
                        ui.label(t!("Name"));
                        ui.add(egui::TextEdit::singleline(&mut tp.name).desired_width(200.0));
                        ui.end_row();
                        ui.label(t!("Vektoren"));
                        ui.horizontal(|ui| {
                            ui.label(tp.shape_ids.len().to_string());
                            if ui.button(t!("Auswahl übernehmen (%{n})", n = sel_count)).clicked() {
                                apply = true;
                            }
                        });
                        ui.end_row();
                    });
                });
                gap(ui);
                ui.vertical(|ui| {
                    ui.set_width(column);
                    // the depths take a number, a formula or variables ("platte_d/2"): a formula
                    // is kept on the toolpath and follows the variables
                    egui::Grid::new("tp_top_right").num_columns(2).spacing([8.0, 6.0]).min_col_width(LABELS + 13.0).show(ui, |ui| {
                        depth_row(ui, &cut_label, 1, &mut tp.cut_depth, &mut tp.cut_expr, thickness + 5.0, depth_text, depth_input, &names, &resolved);
                        depth_row(ui, &t!("Starttiefe"), 0, &mut tp.start_depth, &mut tp.start_expr, thickness, depth_text, depth_input, &names, &resolved);
                    });
                });
            });
            ui.add_space(4.0);

            // left: the tool and how it cuts; right: the strategy over the travel moves. The left
            // frame reaches down as far as the right column (measured the frame before): the
            // column's end, less the space after a group (6), the frame's bottom margin (10), its
            // line and the spacing below the grid (6)
            let right_end = egui::Id::new("tp_right_column_bottom");
            let right_bottom: Option<f32> = ui.ctx().data(|d| d.get_temp(right_end));
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(column);
                    group_down_to(ui, &t!("Werkzeug"), "tp_tool", right_bottom.map(|b| b - 6.0 - 10.0 - 1.0 - 5.0), |ui| {
                        ui.label(t!("Fräser"));
                        let allowed: Vec<&Tool> = tools
                            .iter()
                            .filter(|t| match tp.strategy {
                                Strategy::Drill => t.kind == ToolKind::Drill,
                                Strategy::VCarve { .. } => t.kind == ToolKind::VBit,
                                _ => t.kind != ToolKind::Drill,
                            })
                            .collect();
                        let tool_label = crate::tools::shown_name(&tp.tool.name);
                        if layout::combo_with_manage(ui, "tool", 160.0, &tool_label, |ui| {
                            for t in &allowed {
                                if ui.selectable_label(tp.tool.name == t.name && tp.tool.diameter == t.diameter, crate::tools::shown_name(&t.name)).clicked() {
                                    tp.tool = (*t).clone();
                                }
                            }
                        }) {
                            open_tool_dlg = true;
                        }
                        ui.end_row();
                        ui.label(t!("Zustellung"));
                        ui.add(mm(&mut tp.tool.stepdown, 50.0));
                        ui.end_row();
                        if !matches!(tp.strategy, Strategy::Drill) {
                            ui.label(t!("Querzustellung"));
                            let pct = if tp.tool.diameter > 0.0 { tp.tool.stepover / tp.tool.diameter * 100.0 } else { 0.0 };
                            ui.horizontal(|ui| {
                                ui.add(mm(&mut tp.tool.stepover, 50.0));
                                ui.label(egui::RichText::new(format!("{pct:.0} %")).small().color(muted()));
                            });
                            ui.end_row();
                        }
                        ui.label(t!("Drehzahl"));
                        ui.add(egui::DragValue::new(&mut tp.tool.rpm).speed(100).range(0..=60000).suffix(t!(" U/min")));
                        ui.end_row();
                        ui.label(t!("Vorschub"));
                        ui.add(egui::DragValue::new(&mut tp.tool.feed).speed(10).range(1.0..=20000.0).suffix(" mm/min"));
                        ui.end_row();
                        ui.label(t!("Eintauchen"));
                        ui.add(egui::DragValue::new(&mut tp.tool.plunge).speed(10).range(1.0..=20000.0).suffix(" mm/min"));
                        ui.end_row();
                    });
                });
                gap(ui);
                let right = ui.vertical(|ui| {
                    ui.set_width(column);
                    // Strategie: where the cutter goes, in which order and direction, and how it
                    // enters the material
                    group(ui, &t!("Strategie"), "tp_strategy", |ui| {
                        match &mut tp.strategy {
                            Strategy::Profile { side, climb, tabs, tab_count } => {
                                ui.label(t!("Seite"));
                                ui.horizontal(|ui| {
                                    ui.radio_value(side, Side::Outside, t!("Außen"));
                                    ui.radio_value(side, Side::Inside, t!("Innen"));
                                    ui.radio_value(side, Side::On, t!("Auf der Linie"));
                                });
                                ui.end_row();
                                ui.label(t!("Haltestege"));
                                ui.horizontal(|ui| {
                                    ui.checkbox(tabs, "");
                                    ui.add_enabled(*tabs, egui::DragValue::new(tab_count).range(1..=12).suffix(t!(" Stück")));
                                });
                                ui.end_row();
                                ui.label(t!("Fräsrichtung"));
                                ui.horizontal(|ui| {
                                    ui.radio_value(climb, true, t!("Gleichlauf"));
                                    ui.radio_value(climb, false, t!("Gegenlauf"));
                                });
                                ui.end_row();
                            }
                            Strategy::Pocket { raster, climb, raster_angle } => {
                                ui.label(t!("Ausräumen"));
                                ui.horizontal(|ui| {
                                    ui.radio_value(raster, false, t!("Versatz"));
                                    ui.radio_value(raster, true, t!("Raster"));
                                });
                                ui.end_row();
                                if *raster {
                                    ui.label(t!("Rasterwinkel"));
                                    ui.add(egui::DragValue::new(raster_angle).speed(1.0).range(0.0..=180.0).suffix(" °"));
                                    ui.end_row();
                                }
                                ui.label(t!("Fräsrichtung"));
                                ui.horizontal(|ui| {
                                    ui.radio_value(climb, true, t!("Gleichlauf"));
                                    ui.radio_value(climb, false, t!("Gegenlauf"));
                                });
                                ui.end_row();
                            }
                            Strategy::Drill => {
                                ui.label(t!("Bohren"));
                                ui.label(egui::RichText::new(t!("in der Mitte jeder Kontur, Zustellung = Peck-Tiefe")).small().color(muted()));
                                ui.end_row();
                            }
                            Strategy::VCarve { max_depth } => {
                                ui.label(t!("Tiefenbegrenzung"));
                                ui.add(egui::DragValue::new(max_depth).speed(0.1).range(0.0..=100.0).suffix(" mm")).on_hover_text(t!("0 = nur durch den Fräserdurchmesser begrenzt"));
                                ui.end_row();
                            }
                        }
                        if !matches!(tp.strategy, Strategy::Drill) {
                            ui.label(t!("Rampenwinkel"));
                            ui.add(egui::DragValue::new(&mut tp.ramp_angle).speed(0.5).range(0.0..=45.0).max_decimals(1).suffix(" °")).on_hover_text(t!("Der Fräser fährt schräg entlang der Kontur ins Material statt senkrecht. 0° = senkrecht eintauchen."));
                            ui.end_row();
                        }
                    });
                    // Eilgang: the heights the cutter travels at between cuts (the job's, for all toolpaths)
                    group(ui, &t!("Eilgang"), "tp_travel", |ui| {
                        ui.label(t!("Sicher Z"));
                        ui.add(egui::DragValue::new(&mut safe_z).speed(0.1).range(0.5..=100.0).suffix(" mm")).on_hover_text(t!("Höhe über dem Material für die Fahrten zwischen den Schnitten; gilt für alle Werkzeugwege"));
                        ui.end_row();
                        ui.label(t!("Start Z"));
                        ui.add(egui::DragValue::new(&mut home_z).speed(0.1).range(0.5..=200.0).suffix(" mm")).on_hover_text(t!("Höhe am Anfang und Ende des Jobs; gilt für alle Werkzeugwege"));
                        ui.end_row();
                    });
                });
                let bottom = right.response.rect.bottom();
                if right_bottom.is_none_or(|b| (b - bottom).abs() > 0.5) {
                    ui.ctx().data_mut(|d| d.insert_temp(right_end, bottom));
                    ui.ctx().request_repaint();
                }
            });
            dialog_buttons(ui, |ui| {
                let b = egui::Button::new(egui::RichText::new(t!("Anwenden")).color(INK).strong()).fill(ACCENT).min_size(Vec2::new(110.0, 28.0));
                if ui.add(b).clicked() {
                    close = true;
                }
                if ui.add(egui::Button::new(t!("Schließen")).min_size(Vec2::new(90.0, 28.0))).clicked() {
                    ed.index = None;
                    ed.tp.shape_ids.clear();
                    ed.tp.name.clear();
                    ed.kind = "";
                }
            });
        });
        // the travel heights belong to the job: every toolpath follows them
        if (safe_z - self.job.safe_z).abs() > 1e-9 || (home_z - self.job.home_z).abs() > 1e-9 {
            self.job.safe_z = safe_z;
            self.job.home_z = home_z;
            self.recompute_all();
        }
        if apply {
            if let Some(ed) = &mut self.editor {
                ed.tp.shape_ids = self.edit.selection.clone();
            }
        }
        if let Some(ed) = &self.editor {
            if ed.kind.is_empty() {
                self.editor = None;
                return;
            }
        }
        if close {
            let ed = self.editor.take().unwrap();
            let mut tp = ed.tp;
            if tp.shape_ids.is_empty() {
                tp.shape_ids = self.edit.selection.clone();
            }
            self.compute_in_background(tp, ed.index, ui.ctx());
        }
        if open_tool_dlg && self.tool_dialog.is_none() {
            let mat = self.job.material.clone();
            let entries = self.lib.entries.clone();
            let original = entries.clone();
            let first = entries.iter().position(|e| e.material == mat);
            let (tool, idx) = match first {
                Some(i) => (entries[i].tool.clone(), Some(i)),
                None => (ToolLibrary::default_tool(ToolKind::EndMill), None),
            };
            self.tool_dialog = Some(ToolDialog { editing: tool, original, material: mat, index: idx, entries });
        }
    }
}

/// A depth row of the toolpath form: the label, a text field taking a number, a formula or
/// variables (with the suggestions every number field has), the result when it is a formula, and
/// what is wrong with it when it can't be worked out.
#[allow(clippy::too_many_arguments)]
fn depth_row(ui: &mut egui::Ui, label: &str, field: usize, value: &mut f64, expr: &mut Option<String>, max: f64, texts: &mut [String; 2], input: &mut input::ConstraintInput, names: &[String], resolved: &shape_edit::Resolved) {
    ui.label(label);
    ui.vertical(|ui| {
      let mut error: Option<String> = None;
      ui.horizontal(|ui| {
        let id = egui::Id::new(("depth_field", field));
        let resp = ui.add(egui::TextEdit::singleline(&mut texts[field]).id(id).font(egui::TextStyle::Monospace).desired_width(110.0));
        let ctx = ui.ctx().clone();
        let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
        let tab = ctx.input(|i| i.key_pressed(egui::Key::Tab));
        let focused = ctx.memory(|m| m.has_focus(id));
        let mut keys = input::Keys::read(&ctx, tab && focused);
        let lookup = |n: &str| resolved.value(n).ok();
        let suggested = input.suggest(&ctx, field, id, resp.rect, &mut texts[field], names, &lookup, &mut keys, focused || (resp.lost_focus() && enter), 4.0);
        input.cursor(&ctx, field, id, &texts[field]);
        // a finished formula ("=") shows its result
        let text = texts[field].clone();
        let t = text.trim().trim_end_matches('=').trim().to_string();
        if text.trim_end().ends_with('=') {
            if let Ok(v) = shape_edit::calc::eval(&t, resolved) {
                texts[field] = fields::fmt(v);
            }
        }
        // the text is the source of truth: a plain number is a plain value, anything else a formula
        let _ = suggested;
        let plain = !t.is_empty() && !shape_edit::calc::is_formula(&t) && shape_edit::calc::names_in(&t).is_empty();
        match shape_edit::calc::eval(&t, resolved) {
            Ok(v) => {
                *value = v.clamp(0.0, max);
                *expr = if plain { None } else { Some(t.clone()) };
                if expr.is_some() {
                    ui.label(egui::RichText::new(format!("= {} mm", fields::fmt(*value))).monospace().color(muted()));
                } else {
                    ui.label(egui::RichText::new(t!("mm")).color(muted()));
                }
            }
            Err(e) => error = Some(crate::lang::calc_message(&e)),
        }
      });
      // what is wrong, under the field (a long text would run out of the panel beside it)
      if let Some(e) = error {
          ui.label(egui::RichText::new(e).small().color(input::ERROR));
      }
    });
    ui.end_row();
}
