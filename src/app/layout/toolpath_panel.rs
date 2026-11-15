//! The toolpath panel on the right: the list of toolpaths with their operations on top, the
//! simulation and export below, and the editor window of one toolpath.

use crate::app::{fmt_time, icon_button, tp_color, App, Editor, Icon};
use crate::sim;
use crate::theme::{ACCENT, hairline, ink, muted};
use crate::toolpath::Strategy;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use super::tool_column::GAP;
use super::widgets::{draw_pencil, eye, group_container, group_title};

/// Where a value between `lo` and `hi` sits on a 0–100 slider (spread logarithmically).
fn to_slider(value: f64, lo: f64, hi: f64) -> f64 {
    (100.0 * (value.clamp(lo, hi) / lo).ln() / (hi / lo).ln()).clamp(0.0, 100.0)
}

/// The value at a place of a 0–100 slider.
fn from_slider(pos: f64, lo: f64, hi: f64) -> f64 {
    lo * (hi / lo).powf((pos / 100.0).clamp(0.0, 1.0))
}

#[cfg(test)]
mod slider_tests {
    use super::{from_slider, to_slider};

    #[test]
    fn the_slider_runs_from_0_to_100_and_back() {
        assert!((to_slider(5.0, 5.0, 400.0)).abs() < 1e-9);
        assert!((to_slider(400.0, 5.0, 400.0) - 100.0).abs() < 1e-9);
        for v in [5.0, 40.0, 123.0, 400.0] {
            assert!((from_slider(to_slider(v, 5.0, 400.0), 5.0, 400.0) - v).abs() < 1e-9);
        }
    }
}

impl App {
    /// The toolpath panel: the list on top, then the create / preview / save tabs. Folds to a
    /// narrow strip with the arrow, as in the original.
    pub(in crate::app) fn toolpath_panel(&mut self, ctx: &egui::Context) {
        if !self.paths_open {
            egui::SidePanel::right("toolpaths_folded").exact_width(30.0).resizable(false).frame(egui::Frame::none().fill(crate::theme::surface()).inner_margin(4.0).stroke(Stroke::new(1.0_f32, hairline()))).show(ctx, |ui| {
                if ui.button("‹").on_hover_text(t!("Werkzeugpfade zeigen")).clicked() {
                    self.paths_open = true;
                }
            });
            return;
        }
        egui::SidePanel::right("toolpaths").exact_width(360.0).resizable(false).frame(egui::Frame::none().fill(crate::theme::panel()).inner_margin(egui::Margin::symmetric(10.0, 8.0)).stroke(Stroke::new(1.0_f32, crate::theme::separator()))).show(ctx, |ui| {
            // the bottom container is laid out first, so it keeps its place; the top container
            // takes the room that is left
            egui::TopBottomPanel::bottom("toolpath_bottom").frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 0.0, top: 8.0, bottom: 0.0 })).show_separator_line(false).show_inside(ui, |ui| {
                group_container().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    group_title(ui, &t!("Simulation & Export"), |_| {});
                    self.preview_tab(ui);
                });
            });
            group_container().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height(ui.available_height().max(60.0));
                group_title(ui, &t!("Werkzeugpfade"), |ui| {
                    if ui.button("›").on_hover_text(t!("Werkzeugpfade einklappen")).clicked() {
                        self.paths_open = false;
                    }
                });
                // the workpiece first: it is what everything else depends on
                self.setup_row(ui);
                ui.add_space(6.0);
                // the operations that can be added
                self.operation_buttons(ui);
                ui.add_space(6.0);
                // the paths made so far: they fill what is left and scroll when there are many
                egui::Frame::none().fill(crate::theme::panel()).stroke(Stroke::new(1.0_f32, hairline())).rounding(4.0).inner_margin(4.0).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_min_height(ui.available_height().max(60.0));
                    egui::ScrollArea::vertical().id_salt("toolpath_list").auto_shrink([false, false]).show(ui, |ui| {
                        self.toolpath_list(ui);
                        self.time_note(ui);
                    });
                });
            });
        });
        self.editor_window(ctx);
    }

    /// The first entry of the panel: the board (size and material), clickable to set it up.
    fn setup_row(&mut self, ui: &mut egui::Ui) {
        let valid = self.job.width > 0.0 && self.job.height > 0.0 && self.job.thickness > 0.0;
        let resp = egui::Frame::none().fill(crate::theme::surface()).stroke(Stroke::new(1.0_f32, hairline())).rounding(4.0).inner_margin(egui::Margin::symmetric(6.0, 5.0)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            // the original's "Platten-Setup" row (0x10052b5f4) with its valid/warning mark
            ui.horizontal(|ui| {
                // only a problem is marked: a tick on every good board said nothing
                if !valid {
                    ui.label(egui::RichText::new("⚠").color(Color32::from_rgb(0xc0, 0x50, 0x00))).on_hover_text(t!("Die Platte hat keine gültige Größe: bitte einrichten"));
                }
                ui.label(egui::RichText::new(t!("Platten-Setup")).color(ink()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("{:.0} × {:.0} × {:.1} mm", self.job.width, self.job.height, self.job.thickness)).color(muted()));
                });
            });
        });
        let click = ui.interact(resp.response.rect, egui::Id::new("setup_row"), Sense::click());
        if click.hovered() {
            ui.painter().rect_stroke(resp.response.rect, 4.0, Stroke::new(1.0_f32, crate::theme::icon_line()));
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if click.on_hover_text(t!("Werkstück einrichten: Größe, Material, Nullpunkt")).clicked() {
            self.job_dialog = Some((self.job.clone(), false));
        }
    }

    /// The five operations a toolpath can be made with.
    fn operation_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(GAP);
            // in the original's order (0x10052b5f4): cutout, profile, pocket, drilling, V-carve
            for (icon, kind, hint) in [
                (Icon::Cutout, "cutout", t!("Ausschnitt (Teil heraustrennen)")),
                (Icon::Profile, "profile", t!("Profil-Werkzeugweg")),
                (Icon::Pocket, "pocket", t!("Taschen-Werkzeugweg")),
                (Icon::Drill, "drill", t!("Bohr-Werkzeugweg")),
                (Icon::VCarve, "vcarve", t!("V-Carving-Werkzeugweg")),
            ] {
                let on = self.editor.as_ref().is_some_and(|e| e.kind == kind && e.index.is_none());
                if icon_button(ui, icon, on, &hint).clicked() {
                    self.open_editor(kind);
                }
            }
        });
    }

    fn preview_tab(&mut self, ui: &mut egui::Ui) {
        // the three buttons in one row: the animated simulation, the finished carving at once, and
        // the export (which asks in its own window); a reset only while there is something to reset
        let gap = ui.spacing().item_spacing.x;
        let width = ((ui.available_width() - 2.0 * gap) / 3.0).floor();
        ui.horizontal(|ui| {
            let size = Vec2::new(width, 28.0);
            if ui.add(egui::Button::new(t!("▶ Simulieren")).min_size(size)).clicked() {
                self.start_preview(true);
            }
            if ui.add(egui::Button::new(t!("Vorschau")).min_size(size)).on_hover_text(t!("Zeigt das fertig gefräste Material sofort")).clicked() {
                self.start_preview(false);
            }
            if ui.add(egui::Button::new(t!("Exportieren…")).min_size(size)).on_hover_text(t!("Werkzeugpfade exportieren")).clicked() {
                self.export_dialog = true;
            }
        });
        if self.sim.is_some() {
            ui.horizontal(|ui| {
                if ui.button("↺").on_hover_text(t!("Material zurücksetzen")).clicked() {
                    self.sim = None;
                }
                ui.label(egui::RichText::new(t!("Material zurücksetzen")).small().color(muted()));
            });
        }
        // "Preview Type" of the original: which toolpaths are simulated
        ui.horizontal_wrapped(|ui| {
            use crate::app::PreviewScope;
            for (scope, name) in [(PreviewScope::All, t!("Alle")), (PreviewScope::Visible, t!("Sichtbare"))] {
                ui.radio_value(&mut self.preview_scope, scope, name);
            }
        })
        .response
        .on_hover_text(t!("Welche Werkzeugwege die Vorschau zeigt (der ausgewählte ist der in der Liste angeklickte)"));
        // the sliders take all the width that is left beside their labels
        const LABEL: f32 = 84.0;
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(Vec2::new(LABEL, 18.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.set_min_width(LABEL);
                ui.label(t!("Tempo"))
            });
            ui.spacing_mut().slider_width = ui.available_width().max(60.0);
            // both sliders run from 0 to 100, as the original's (wxSlider 0..100 at 0x1000fc110 and
            // 0x1000fd37c); the value behind is spread logarithmically
            // the middle is real time (1x); the ends are a sixtieth and sixty times that
            let (lo, hi) = (crate::app::SIM_RATE_MIN, crate::app::SIM_RATE_MAX);
            let mut pos = to_slider(self.sim_rate, lo, hi);
            if ui.add(egui::Slider::new(&mut pos, 0.0..=100.0).integer().show_value(false)).on_hover_text(t!("%{x}× (Echtzeit: 1×)", x = format!("{:.2}", from_slider(pos, lo, hi)))).changed() {
                self.sim_rate = if (pos - 50.0).abs() < 2.0 { 1.0 } else { from_slider(pos, lo, hi) };
            }
        });
        // the original's simulation quality, in grid points per mm; called "Darstellung" here,
        // because "Qualität" sounds like the quality of the toolpaths, and this only sets how
        // finely the material is drawn in the preview
        let spacing = sim::grid_spacing(self.job.width, self.job.height, self.sim_quality);
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(Vec2::new(LABEL, 18.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.set_min_width(LABEL);
                ui.label(t!("Darstellung"))
            });
            ui.spacing_mut().slider_width = ui.available_width().max(60.0);
            let mut pos = to_slider(self.sim_quality, 1.0, sim::SIMULATION_QUALITY_MAX);
            if ui.add(egui::Slider::new(&mut pos, 0.0..=100.0).integer().show_value(false))
                .on_hover_text(t!("%{pos} %\nWie fein das Material in der Vorschau gezeichnet wird (ändert nicht die Werkzeugwege)\n%{ppm} Punkte je mm · Raster %{spacing} mm", pos = format!("{pos:.0}"), ppm = (self.sim_quality * 10.0).round() / 10.0, spacing = format!("{spacing:.2}")))
                .changed()
            {
                self.sim_quality = from_slider(pos, 1.0, sim::SIMULATION_QUALITY_MAX);
            }
        });
        // the ends of the original's "Preview quality" slider: "Faster" and "Quality", one under each end
        ui.horizontal(|ui| {
            ui.add_space(LABEL + ui.spacing().item_spacing.x);
            let width = ui.available_width();
            ui.allocate_ui_with_layout(Vec2::new(width, 12.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(t!("schneller")).small().color(muted()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(t!("genauer")).small().color(muted()));
                });
            });
        });
        ui.checkbox(&mut self.paths_in_3d, t!("G-Code-Fahrt im 3D einblenden"))
            .on_hover_text(t!("Zeigt die Fahrt so, wie der Postprozessor sie schreibt: von Home über die sichere Höhe zum ersten Werkzeugweg, zwischen den Werkzeugwegen und am Ende. Eilgang gestrichelt, Eintauchen gelb."));
    }

    /// The toolpath list: hide/show all, then one row per toolpath with its eye, colour, name
    /// and time; ordering and deleting appear on the row under the pointer.
    /// The milling time of what is shown, a small read-only line at the end of the list.
    fn time_note(&self, ui: &mut egui::Ui) {
        if self.toolpaths.is_empty() {
            return;
        }
        let total: f64 = self.toolpaths.iter().filter(|t| t.visible).map(|t| t.stats().minutes).sum();
        ui.add_space(4.0);
        ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), 16.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(t!("Σ Fräszeit (sichtbar) %{time}", time = fmt_time(total))).small().color(muted()))
                .on_hover_text(t!("Gesamte Fräszeit der sichtbaren Werkzeugwege: %{minutes} Minuten", minutes = format!("{total:.2}")));
        });
    }

    /// The rows of the toolpaths that cut at least one of the selected shapes.
    pub(in crate::app) fn toolpaths_of_selection(&self) -> Vec<usize> {
        if self.edit.selection.is_empty() {
            return Vec::new();
        }
        (0..self.toolpaths.len()).filter(|&i| self.toolpaths[i].shape_ids.iter().any(|id| self.edit.selection.contains(id))).collect()
    }

    fn toolpath_list(&mut self, ui: &mut egui::Ui) {
        let all_hidden = self.toolpaths.iter().all(|t| !t.visible);
        let label = if all_hidden { t!("Alle einblenden") } else { t!("Alle ausblenden") };
        ui.horizontal(|ui| {
            if eye(ui, !all_hidden, &label).clicked() {
                for t in &mut self.toolpaths {
                    t.visible = all_hidden;
                }
                self.sim = None;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("↻").on_hover_text(t!("Alle Werkzeugpfade neu berechnen")).clicked() {
                    self.recompute_all();
                    self.status = "Alle Werkzeugpfade neu berechnet".into();
                }
            });
        });
        if self.toolpaths.is_empty() {
            ui.label(egui::RichText::new(t!("Noch keine Werkzeugpfade.")).color(muted()));
            return;
        }
        let (mut remove, mut edit) = (None, None);
        let mut select_shapes: Option<Vec<u64>> = None;
        let mut pick_row: Option<usize> = None;
        let mut rows: Vec<Rect> = Vec::new();
        let mut grips: Vec<Rect> = Vec::new();
        let mut drag_started = None;
        let dragging = self.tp_drag;
        // a small entry above the first toolpath and wherever the tool changes: from here on this tool cuts
        let tool_entries = crate::toolgroups::headers(&self.toolpaths.iter().map(|t| &t.tool).collect::<Vec<_>>());
        // the toolpaths that cut the selected shapes light up, as the operations of a selected body
        // do in Fusion 360; the list scrolls to the first of them when the selection changed
        let related_rows = self.toolpaths_of_selection();
        let selection_changed = self.tp_scrolled_for != self.edit.selection;
        let mut scrolled = false;
        for (i, tp) in self.toolpaths.iter_mut().enumerate() {
            if tool_entries[i] {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(t!("Werkzeug: %{name}", name = crate::tools::shown_name(&tp.tool.name))).small().strong().color(muted()))
                        .on_hover_text(t!("Ab hier fräst dieses Werkzeug, bis ein anderes eingetragen ist. Beim Speichern lässt sich je Werkzeug eine eigene Datei schreiben."));
                });
            }
            let active = self.editor.as_ref().is_some_and(|e| e.index == Some(i));
            let related = related_rows.contains(&i);
            // every row has the same frame, so the one that lights up does not change the layout
            let frame = egui::Frame::none()
                .fill(if active || self.tp_selected == Some(i) { crate::theme::row_active() } else { crate::theme::row() })
                .stroke(Stroke::new(1.5_f32, if related { ACCENT } else { Color32::TRANSPARENT }))
                .rounding(4.0)
                .inner_margin(egui::Margin::symmetric(4.5, 1.5));
            let row = frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    // the grip: drag it to put this toolpath somewhere else in the list; it lights up
                    // when the toolpath cuts a selected shape
                    let (grip, grip_resp) = ui.allocate_exact_size(Vec2::new(10.0, 18.0), Sense::drag());
                    if related {
                        ui.painter().rect_filled(grip.expand(4.0), 7.0, ACCENT.gamma_multiply(0.22));
                        ui.painter().rect_filled(grip, 4.0, ACCENT);
                    }
                    let tone = if related {
                        crate::theme::INK
                    } else if grip_resp.hovered() || dragging == Some(i) {
                        ink()
                    } else {
                        crate::theme::grip()
                    };
                    for dx in [-2.0, 2.0] {
                        for dy in [-5.0, 0.0, 5.0] {
                            ui.painter().circle_filled(grip.center() + Vec2::new(dx, dy), 1.2, tone);
                        }
                    }
                    if grip_resp.hovered() || dragging == Some(i) {
                        ui.ctx().set_cursor_icon(if dragging == Some(i) { egui::CursorIcon::Grabbing } else { egui::CursorIcon::Grab });
                    }
                    if grip_resp.drag_started() {
                        drag_started = Some(i);
                    }
                    grips.push(grip);
                    if eye(ui, tp.visible, "").clicked() {
                        tp.visible = !tp.visible;
                    }
                    let minutes = tp.stats().minutes;
                    let tip = t!("%{name}\n%{kind}: Start %{start} mm, Schnitt %{cut} mm\nGesamte Fräszeit: %{minutes} Minuten (%{time})\nDoppelklick: bearbeiten", name = tp.name, kind = tp.strategy.label(), start = format!("{:.1}", tp.start_depth), cut = format!("{:.1}", tp.cut_depth), minutes = format!("{minutes:.2}"), time = fmt_time(minutes));
                    let (sq, sq_resp) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                    ui.painter().rect_filled(sq, 2.0, tp_color(&tp.strategy));
                    sq_resp.on_hover_text(&tip);
                    // what the toolpath does, as the little slab of the operation's button
                    let (pic, pic_resp) = ui.allocate_exact_size(Vec2::new(22.0, 18.0), Sense::hover());
                    crate::app::icons::operation_icon(ui.painter(), pic.center(), &tp.strategy, 22.0);
                    pic_resp.on_hover_text(&tip);
                    let name = ui.add(egui::Label::new(egui::RichText::new(&tp.name).italics().color(ink())).sense(Sense::click()).truncate()).on_hover_text(&tip);
                    if name.double_clicked() {
                        edit = Some(i);
                    } else if name.clicked() {
                        // one click shows what the path cuts, and makes it "the selected toolpath"
                        select_shapes = Some(tp.shape_ids.clone());
                        pick_row = Some(i);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let hovered = ui.rect_contains_pointer(ui.max_rect());
                        if hovered {
                            if ui.small_button("🗑").on_hover_text(t!("Löschen")).clicked() {
                                remove = Some(i);
                            }
                            let edit_btn = ui.add(egui::Button::new("").small().min_size(Vec2::new(24.0, 20.0)));
                            if edit_btn.clicked() {
                                edit = Some(i);
                            }
                            draw_pencil(ui.painter(), edit_btn.rect.center(), if edit_btn.hovered() { ink() } else { muted() });
                            edit_btn.on_hover_text(t!("Bearbeiten"));
                        }
                        ui.label(egui::RichText::new(fmt_time(tp.stats().minutes)).small().color(muted()));
                    });
                });
                if let Some(w) = &tp.warning {
                    ui.label(egui::RichText::new(format!("⚠ {w}")).small().color(Color32::from_rgb(0xc0, 0x50, 0x00)));
                }
            });
            rows.push(row.response.rect);
            if related && selection_changed && !scrolled && !ui.is_rect_visible(row.response.rect) {
                ui.scroll_to_rect(row.response.rect, Some(egui::Align::Center));
                scrolled = true;
            }
        }
        self.tp_scrolled_for = self.edit.selection.clone();
        self.tp_grips = grips;
        if drag_started.is_some() {
            self.tp_drag = drag_started;
        }
        // while dragging: a line where it would land; when the button is let go, it lands there
        if let Some(from) = self.tp_drag {
            let pointer = ui.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos()));
            let to = pointer.map_or(from, |p| rows.iter().filter(|r| r.center().y < p.y).count());
            if let (Some(first), Some(last)) = (rows.first(), rows.last()) {
                let y = if to == 0 { first.top() - 2.0 } else if to >= rows.len() { last.bottom() + 2.0 } else { (rows[to - 1].bottom() + rows[to].top()) / 2.0 };
                ui.painter().line_segment([Pos2::new(first.left(), y), Pos2::new(first.right(), y)], Stroke::new(2.5_f32, crate::theme::ACCENT_DEEP));
            }
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            if !ui.input(|i| i.pointer.primary_down()) {
                self.tp_drag = None;
                // the toolpath goes before row `to` (counted before it was taken out)
                let target = if to > from { to - 1 } else { to };
                if target != from && from < self.toolpaths.len() {
                    let item = self.toolpaths.remove(from);
                    self.toolpaths.insert(target.min(self.toolpaths.len()), item);
                    // an open editor keeps pointing at the same toolpath
                    if let Some(ed) = &mut self.editor {
                        if let Some(k) = ed.index {
                            ed.index = Some(if k == from {
                                target
                            } else {
                                let after_removal = if k > from { k - 1 } else { k };
                                if after_removal >= target { after_removal + 1 } else { after_removal }
                            });
                        }
                    }
                    self.sim = None;
                    self.tp_selected = None;
                    self.status = "Werkzeugpfad verschoben".into();
                }
            } else {
                ui.ctx().request_repaint();
            }
        }
        if pick_row.is_some() {
            self.tp_selected = pick_row;
        }
        if let Some(ids) = select_shapes {
            self.edit.selection = ids;
            self.edit.retain_existing(&self.shapes);
        }
        if let Some(i) = remove {
            self.tp_selected = None;
            self.toolpaths.remove(i);
            self.editor = None;
            self.sim = None;
        }
        if let Some(i) = edit {
            let tp = self.toolpaths[i].clone();
            let kind = match tp.strategy {
                Strategy::Pocket { .. } => "pocket",
                Strategy::Drill => "drill",
                Strategy::VCarve { .. } => "vcarve",
                Strategy::Profile { .. } => "profile",
            };
            self.edit.selection = tp.shape_ids.clone();
            self.editor = Some(Editor::new(tp, Some(i), kind));
        }
    }
}
