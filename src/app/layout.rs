//! The window's arrangement, following the original (the person's picture-12):
//!
//! - a title bar with the usual menu bar (every menu at the top level), the layer in the
//!   middle and the document's name on the right;
//! - one tool column on the left, grouped like the original's; a tool with settings opens its
//!   form beside the column;
//! - a toolbar over the drawing area only, with 2D/3D and zoom at its right end;
//! - prompts and help floating at the bottom right of the drawing;
//! - the pointer position and the selection's size along the bottom;
//! - the toolpath panel on the right: the list on top, then three tabs (create, preview, save).

use super::shortcuts::Action;
use super::{fmt_time, icon_button, tp_color, App, Combine, DrawTool, Editor, Icon, ViewMode};
use crate::gcode::Post;
use crate::sim;
use crate::theme::{ACCENT, hairline, ink, muted};
use crate::toolpath::Strategy;
use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use shape_edit::{selection_bounds, Flip};

/// A tool that opens a form beside the tool column.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ToolForm {
    /// Move and size the selection by numbers (the selected shape's properties).
    Transform,
    /// Offset (expand or shrink) the selected vectors.
    Offset,
}

/// The tabs under the toolpath list.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ToolpathTab {
    Create,
    Preview,
    Save,
}

/// The tool column's groups, top to bottom, as the original registers them (0x10011d034):
/// `mode` (select, transform, node edit), `draw_line`, `draw_shape` (rectangle, circle,
/// ellipse, polygon), `draw_text`, `edit_object`, `align` (align, mirror), `offset` and
/// `binary_operation`; the job setup sits apart at the end. The original's `bitmap`, `join`,
/// `create_fillets` and `group_ungroup` groups have no tools here yet, so they are left out;
/// duplicating is no tool there either (menu and ⌘D).
const GROUPS: [&[Icon]; GROUP_COUNT] = [
    &[Icon::Draw(DrawTool::Select), Icon::Transform, Icon::Draw(DrawTool::NodeEdit)],
    &[Icon::Draw(DrawTool::Polyline)],
    &[Icon::Draw(DrawTool::Rect), Icon::Draw(DrawTool::Circle), Icon::Draw(DrawTool::Ellipse), Icon::Draw(DrawTool::Polygon)],
    &[Icon::Draw(DrawTool::Text)],
    &[Icon::Delete, Icon::Curves],
    &[Icon::Center, Icon::MirrorH, Icon::MirrorV],
    &[Icon::Offset],
    &[Icon::Union, Icon::Subtract, Icon::Intersect],
];

/// How many groups the tool column has (one remembered choice each when folded).
pub(super) const GROUP_COUNT: usize = 8;

/// What each group is, for the folded column's tooltips.
const GROUP_NAMES: [&str; GROUP_COUNT] = ["Modus", "Linien zeichnen", "Formen zeichnen", "Text", "Objekte bearbeiten", "Ausrichten und Spiegeln", "Versatz", "Boolesche Operationen"];

/// Icons per row in the expanded column, and the size of one icon cell.
const ROW: usize = 5;
const CELL: f32 = 38.0;
const GAP: f32 = 4.0;

fn tip(icon: Icon) -> &'static str {
    match icon {
        Icon::Draw(t) => t.tip(),
        Icon::Transform => "Objekte transformieren (Lage und Größe eingeben)",
        Icon::Offset => "Vektoren versetzen",
        Icon::MirrorH => "Waagrecht spiegeln",
        Icon::MirrorV => "Senkrecht spiegeln",
        Icon::Center => "Auf dem Material zentrieren",
        Icon::Curves => "In Kurven umwandeln",
        Icon::Delete => "Löschen",
        Icon::Union => "Vereinen (mindestens zwei Formen)",
        Icon::Subtract => "Abziehen: die zuerst gewählte Form minus die anderen",
        Icon::Intersect => "Schnittmenge (mindestens zwei Formen)",
        Icon::Job => "Job einrichten (Größe, Material, Nullpunkt)",
        Icon::Variables => "Variablen (f(x): benannte Werte für die Maße)",
        _ => "",
    }
}

impl App {
    /// The menu bar on the left, the layer in the middle, the document's name on the right.
    pub(super) fn title_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("title").exact_height(34.0).frame(egui::Frame::none().fill(crate::theme::surface()).inner_margin(egui::Margin::symmetric(10.0, 4.0)).stroke(Stroke::new(1.0_f32, hairline()))).show(ctx, |ui| {
            let full = ui.max_rect();
            // the usual menu bar, every menu at the top level
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new("⚡").color(ACCENT).size(16.0));
                egui::menu::bar(ui, |ui| self.menus(ui));
            });
            // the document's name at the right end
            let mut name = self.path.as_ref().and_then(|p| p.file_stem()).map_or_else(|| "Neu".to_string(), |s| s.to_string_lossy().into_owned());
            if self.dirty_shown {
                name.push_str(" ●");
            }
            let label = ui.painter().text(full.right_center() - Vec2::new(38.0, 0.0), Align2::RIGHT_CENTER, name, FontId::proportional(14.0), muted());
            if self.dirty_shown {
                ui.interact(label, egui::Id::new("dirty_mark"), Sense::hover()).on_hover_text("Ungespeicherte Änderungen");
            }
            // the switch for the dark theme, at the right end: a sun in the dark, a moon in the light
            let slot = Rect::from_center_size(full.right_center() - Vec2::new(14.0, 0.0), Vec2::splat(26.0));
            let resp = ui.interact(slot, egui::Id::new("theme_switch"), Sense::click());
            if resp.hovered() {
                ui.painter().rect_filled(slot, 5.0, crate::theme::hover());
            }
            let c = slot.center();
            let tone = crate::theme::icon_line();
            if self.dark {
                ui.painter().circle_filled(c, 4.0, tone);
                for k in 0..8 {
                    let a = k as f32 * std::f32::consts::FRAC_PI_4;
                    let d = Vec2::new(a.cos(), a.sin());
                    ui.painter().line_segment([c + d * 6.5, c + d * 9.0], Stroke::new(1.5_f32, tone));
                }
            } else {
                ui.painter().circle_filled(c, 7.0, tone);
                ui.painter().circle_filled(c + Vec2::new(3.5, -3.0), 6.0, crate::theme::surface());
            }
            if resp.on_hover_text(if self.dark { "Zum hellen Design wechseln" } else { "Zum dunklen Design wechseln" }).clicked() {
                let choice = if self.dark { crate::theme::Choice::Light } else { crate::theme::Choice::Dark };
                self.set_theme_choice(ctx, choice);
            }
            // the layer, centred over the window as in the original (one layer for now)
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(Rect::from_center_size(full.center(), Vec2::new(230.0, 26.0))), |ui| self.layer_selector(ui));
        });
    }

    /// The tool column. Expanded, it shows every tool, grouped in rows of up to five. Folded, it
    /// is one narrow column with one button per group, showing the group's current tool; a
    /// group of several tools opens them to the right, so the folded column never scrolls.
    pub(super) fn tool_column(&mut self, ctx: &egui::Context) {
        let per_row = if self.tools_open { ROW } else { 1 };
        let width = 14.0 + per_row as f32 * (CELL + GAP);
        let mut clicked = None;
        egui::SidePanel::left("tool_column")
            .exact_width(width)
            .resizable(false)
            .frame(egui::Frame::none().fill(crate::theme::panel()).inner_margin(egui::Margin { left: 6.0, right: 6.0, top: 6.0, bottom: 6.0 }).stroke(Stroke::new(1.0_f32, crate::theme::separator())))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(GAP);
                let chevron = if self.tools_open { "‹" } else { "›" };
                let hint = if self.tools_open { "Werkzeugleiste verkleinern" } else { "Werkzeugleiste erweitern" };
                if ui.add(egui::Button::new(egui::RichText::new(chevron).size(16.0).color(ink())).min_size(Vec2::new(26.0, 22.0)).rounding(4.0)).on_hover_text(hint).clicked() {
                    self.tools_open = !self.tools_open;
                    self.flyout = None;
                }
                if self.tools_open {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for group in GROUPS {
                            ui.add_space(4.0);
                            for row in group.chunks(per_row) {
                                ui.horizontal(|ui| {
                                    for &icon in row {
                                        if self.column_icon(ui, icon).clicked() {
                                            clicked = Some(icon);
                                        }
                                    }
                                });
                            }
                        }
                        ui.add_space(6.0);
                        ui.separator();
                        // job setup and, next to it, the variables dialog (f(x))
                        ui.horizontal(|ui| {
                            if self.column_icon(ui, Icon::Job).clicked() {
                                clicked = Some(Icon::Job);
                            }
                            if self.column_icon(ui, Icon::Variables).clicked() {
                                clicked = Some(Icon::Variables);
                            }
                        });
                    });
                } else {
                    for (g, group) in GROUPS.iter().enumerate() {
                        ui.add_space(2.0);
                        let shown = self.group_icon(g);
                        let resp = self.column_icon(ui, shown);
                        if group.len() > 1 {
                            // a small corner mark: this button holds more tools
                            let r = resp.rect;
                            ui.painter().add(egui::Shape::convex_polygon(vec![r.right_bottom() - Vec2::new(2.0, 2.0), r.right_bottom() - Vec2::new(8.0, 2.0), r.right_bottom() - Vec2::new(2.0, 8.0)], ink(), Stroke::NONE));
                        }
                        // an open flyout stays beside its button
                        if let Some((open, at)) = &mut self.flyout {
                            if *open == g {
                                *at = resp.rect.right_top() + Vec2::new(6.0, 0.0);
                            }
                        }
                        let resp = resp.on_hover_text(GROUP_NAMES[g]);
                        if resp.clicked() {
                            if group.len() == 1 {
                                clicked = Some(shown);
                            } else {
                                self.flyout = if self.flyout.is_some_and(|(open, _)| open == g) { None } else { Some((g, resp.rect.right_top() + Vec2::new(6.0, 0.0))) };
                            }
                        }
                    }
                    ui.add_space(6.0);
                    ui.separator();
                    if self.column_icon(ui, Icon::Job).clicked() {
                        clicked = Some(Icon::Job);
                    }
                    if self.column_icon(ui, Icon::Variables).clicked() {
                        clicked = Some(Icon::Variables);
                    }
                }
            });
        if !self.tools_open {
            if let Some(icon) = self.flyout_ui(ctx) {
                clicked = Some(icon);
            }
        }
        if let Some(icon) = clicked {
            self.column_click(icon);
        }
    }

    /// The tool a folded group shows: the one in use if it belongs to the group, else the one
    /// last picked from it.
    fn group_icon(&self, g: usize) -> Icon {
        let group = GROUPS[g];
        let active = group.iter().copied().find(|&icon| match icon {
            Icon::Draw(t) => self.tool == t && t != DrawTool::Select,
            Icon::Transform => self.tool_form == Some(ToolForm::Transform),
            Icon::Offset => self.tool_form == Some(ToolForm::Offset),
            _ => false,
        });
        active.unwrap_or(group[self.tool_choice[g].min(group.len() - 1)])
    }

    /// The open group's tools beside the folded column; returns the one picked. A press
    /// anywhere else closes it.
    fn flyout_ui(&mut self, ctx: &egui::Context) -> Option<Icon> {
        let (g, at) = self.flyout?;
        let mut picked = None;
        let area = egui::Area::new(egui::Id::new("tool_flyout")).order(egui::Order::Foreground).fixed_pos(at).show(ctx, |ui| {
            egui::Frame::popup(ui.style()).fill(crate::theme::surface()).rounding(5.0).show(ui, |ui| {
                ui.label(egui::RichText::new(GROUP_NAMES[g]).small().color(muted()));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = GAP;
                    for (k, &icon) in GROUPS[g].iter().enumerate() {
                        if self.column_icon(ui, icon).clicked() {
                            picked = Some((k, icon));
                        }
                    }
                });
            });
        });
        if let Some((k, icon)) = picked {
            self.tool_choice[g] = k;
            self.flyout = None;
            return Some(icon);
        }
        let pressed_outside = ctx.input(|i| i.pointer.any_pressed() && i.pointer.interact_pos().is_some_and(|p| !area.response.rect.contains(p) && p.x > at.x));
        if pressed_outside {
            self.flyout = None;
        }
        None
    }

    fn column_icon(&self, ui: &mut egui::Ui, icon: Icon) -> egui::Response {
        let any = !self.edit.selection.is_empty();
        let (selected, enabled) = match icon {
            Icon::Draw(t) => (self.tool == t, true),
            Icon::Transform => (self.tool_form == Some(ToolForm::Transform), true),
            Icon::Offset => (self.tool_form == Some(ToolForm::Offset), true),
            Icon::Union | Icon::Subtract | Icon::Intersect => (false, self.edit.selection.len() >= 2),
            Icon::Job => (false, true),
            Icon::Variables => (self.vars_open, true),
            _ => (false, any),
        };
        ui.add_enabled_ui(enabled, |ui| icon_button(ui, icon, selected, tip(icon))).inner
    }

    pub(super) fn column_click(&mut self, icon: Icon) {
        self.flyout = None;
        match icon {
            Icon::Draw(t) => {
                self.tool = t;
                self.poly.clear();
                self.mode = ViewMode::TwoD;
            }
            Icon::Transform | Icon::Offset => {
                let form = if icon == Icon::Transform { ToolForm::Transform } else { ToolForm::Offset };
                self.tool_form = if self.tool_form == Some(form) { None } else { Some(form) };
                self.tool = DrawTool::Select;
                self.mode = ViewMode::TwoD;
            }
            Icon::MirrorH => self.mirror(Flip::Horizontal),
            Icon::MirrorV => self.mirror(Flip::Vertical),
            Icon::Center => self.center_on_job(),
            Icon::Curves => self.to_curves(),
            Icon::Delete => self.delete_selection(),
            Icon::Union => self.combine(Combine::Union),
            Icon::Subtract => self.combine(Combine::Subtract),
            Icon::Intersect => self.combine(Combine::Intersect),
            Icon::Job => self.job_dialog = Some((self.job.clone(), false)),
            Icon::Variables => self.vars_open = !self.vars_open,
            _ => {}
        }
    }

    /// The open tool's form, beside the tool column.
    pub(super) fn tool_form_panel(&mut self, ctx: &egui::Context) {
        let Some(form) = self.tool_form else { return };
        egui::SidePanel::left("tool_form").exact_width(250.0).resizable(false).frame(egui::Frame::none().fill(crate::theme::panel()).inner_margin(10.0).stroke(Stroke::new(1.0_f32, crate::theme::separator()))).show(ctx, |ui| {
            ui.horizontal(|ui| {
                let title = match form {
                    ToolForm::Transform => "Objekte transformieren",
                    ToolForm::Offset => "Versatz",
                };
                ui.label(egui::RichText::new(title).size(15.0).color(ink()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕").on_hover_text("Schließen").clicked() {
                        self.tool_form = None;
                    }
                });
            });
            ui.separator();
            match form {
                ToolForm::Transform => self.properties(ui),
                ToolForm::Offset => {
                    ui.horizontal(|ui| {
                        ui.label("Abstand");
                        ui.add(egui::DragValue::new(&mut self.offset_dist).range(0.1..=500.0).speed(0.1).suffix(" mm"));
                    });
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.offset_outside, true, "Außen");
                        ui.radio_value(&mut self.offset_outside, false, "Innen");
                    });
                    ui.add_space(6.0);
                    let any = !self.edit.selection.is_empty();
                    if ui.add_enabled(any, egui::Button::new("Versetzen").min_size(Vec2::new(100.0, 24.0))).clicked() {
                        self.offset_selection();
                    }
                    if !any {
                        ui.label(egui::RichText::new("Erst die Vektoren auswählen.").color(muted()));
                    }
                }
            }
        });
    }

    /// The toolbar over the drawing area: snapping and grid on the left; 2D/3D and zoom at the
    /// right end, as in the original.
    pub(super) fn canvas_toolbar(&mut self, ui: &mut egui::Ui) {
        let (bar, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 68.0), Sense::hover());
        ui.painter().rect_filled(bar, 0.0, crate::theme::panel());
        ui.painter().line_segment([bar.left_bottom(), bar.right_bottom()], Stroke::new(1.0_f32, hairline()));
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(bar.shrink2(Vec2::new(8.0, 4.0))), |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = GAP;
                // the tools are in two groups, each in a thin box with its name under it (like the
                // ribbon of Fusion 360): what is drawn on the sheet, and the constraints
                // (the job is set up from the tool column only, as in the original: job_setup is
                // the last entry of its tool list, 0x10011d034)
                toolbar_group(ui, "Dekoration", |ui| {
                    if icon_button(ui, Icon::Grid, self.snap_grid, "Am Raster einrasten").clicked() {
                        self.snap_grid = !self.snap_grid;
                    }
                    if icon_button(ui, Icon::Snap, self.snap_geometry, "An Geometrie einrasten").clicked() {
                        self.snap_geometry = !self.snap_geometry;
                    }
                    if icon_button(ui, Icon::Dimensions, self.show_dims, "Maße anzeigen / ausblenden (Maße blau = Constraint; Linien sind immer blau = frei, schwarz = Länge festgelegt)").clicked() {
                        self.show_dims = !self.show_dims;
                    }
                });
                toolbar_group(ui, "Constraints", |ui| {
                    if icon_button(ui, Icon::Distance, self.dim_mode.is_some(), "Abstand bemaßen (D): zwei Punkte oder Punkt und Linie wählen, dann den Abstand eingeben").clicked() {
                        self.toggle_dim_mode();
                    }
                    if icon_button(ui, Icon::Align, self.align_mode.is_some(), "Waagrecht/Senkrecht (H): eine Linie oder zwei Punkte wählen, CncCraft legt sie waagrecht oder senkrecht fest").clicked() {
                        self.toggle_align_mode();
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icon_button(ui, Icon::ZoomSelection, false, "Auf die Auswahl zoomen").clicked() {
                        self.zoom_to_selection();
                    }
                    if icon_button(ui, Icon::ZoomFit, false, "Ganze Ansicht").clicked() {
                        self.run(Action::FitView);
                    }
                    let shown = self.origin_shown();
                    if icon_button(ui, Icon::Origin, shown, "Nullpunkt anzeigen / ausblenden (Achsenkreuz im Blatt und in der 3D-Ansicht)").clicked() {
                        self.set_origin_shown(!shown);
                    }
                    let (label, next, hint) = match self.mode {
                        ViewMode::TwoD => ("2D", ViewMode::ThreeD, "Zur 3D-Ansicht wechseln"),
                        ViewMode::ThreeD => ("3D", ViewMode::TwoD, "Zur 2D-Ansicht wechseln"),
                    };
                    let b = egui::Button::new(egui::RichText::new(label).strong().size(15.0).color(ink())).min_size(Vec2::new(38.0, 30.0)).stroke(Stroke::new(1.5_f32, ink())).rounding(4.0);
                    if ui.add(b).on_hover_text(hint).clicked() {
                        self.mode = next;
                    }
                });
            });
        });
    }

    /// Whether the origin is drawn in the view that is showing: the sheet's axis cross in 2D, the
    /// zero cross in 3D (the original's "Draw Origin" setting and its overlay switch,
    /// `Toggle_Origin_On/Off`).
    fn origin_shown(&mut self) -> bool {
        match self.mode {
            ViewMode::TwoD => self.sheet.switches().find(|(n, _)| *n == "Koordinatenkreuz").is_some_and(|(_, on)| *on),
            ViewMode::ThreeD => self.view_decos.switches().find(|(n, _)| *n == "Nullpunkt-Achsenkreuz").is_some_and(|(_, on)| *on),
        }
    }

    /// Show or hide the origin in both views at once.
    pub(super) fn set_origin_shown(&mut self, on: bool) {
        for (name, shown) in self.sheet.switches() {
            if name == "Koordinatenkreuz" {
                *shown = on;
            }
        }
        for (name, shown) in self.view_decos.switches() {
            if name == "Nullpunkt-Achsenkreuz" {
                *shown = on;
            }
        }
        self.status = if on { "Nullpunkt wird gezeigt".into() } else { "Nullpunkt ausgeblendet".into() };
    }

    /// Zoom the drawing so the selection fills it.
    fn zoom_to_selection(&mut self) {
        let Some((lo, hi)) = selection_bounds(&self.shapes, &self.edit.selection) else {
            self.run(Action::FitView);
            return;
        };
        let r = self.canvas_rect;
        let (w, h) = ((hi[0] - lo[0]).max(1.0) as f32, (hi[1] - lo[1]).max(1.0) as f32);
        self.mode = ViewMode::TwoD;
        self.zoom = ((r.width() - 120.0) / w).min((r.height() - 120.0) / h).clamp(0.05, 200.0);
        let c = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
        self.origin = Pos2::new(r.center().x - c[0] as f32 * self.zoom, r.center().y + c[1] as f32 * self.zoom);
        self.fitted = true;
    }

    /// Let the worksheet's interactive decorations (the hints) react, over the 2D drawing and
    /// the 3D view alike, and do what they ask for.
    pub(super) fn sheet_interact(&mut self, ctx: &egui::Context) {
        // the decorations are set aside while they read the view, which borrows the app
        let mut sheet = std::mem::replace(&mut self.sheet, crate::sheet::Decorations::empty());
        let actions = sheet.interact(&self.sheet_view(self.canvas_rect), ctx);
        self.sheet = sheet;
        for action in actions {
            match action {
                crate::sheet::SheetAction::ShowToolHelp => {
                    self.status = self.tool.tip().into();
                    self.shortcuts_open = true;
                }
            }
        }
    }

    /// Pointer position, the selection's size and the layer, as the original's bottom line.
    pub(super) fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").exact_height(24.0).frame(egui::Frame::none().fill(crate::theme::surface()).inner_margin(egui::Margin::symmetric(12.0, 3.0))).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let mut text = String::new();
                if let Some(p) = self.mouse_mm {
                    text.push_str(&format!("X[{:.4} mm] | Y[{:.4} mm]", p[0], p[1]));
                }
                if let Some((lo, hi)) = selection_bounds(&self.shapes, &self.edit.selection) {
                    text.push_str(&format!("   W[{:.3} mm] | H[{:.3} mm]", hi[0] - lo[0], hi[1] - lo[1]));
                }
                text.push_str(&format!(" | {}", self.layers.name(self.layers.active)));
                ui.label(egui::RichText::new(text).monospace().size(12.0).color(ink()));
            });
        });
    }

    /// The toolpath panel: the list on top, then the create / preview / save tabs. Folds to a
    /// narrow strip with the arrow, as in the original.
    pub(super) fn toolpath_panel(&mut self, ctx: &egui::Context) {
        if !self.paths_open {
            egui::SidePanel::right("toolpaths_folded").exact_width(30.0).resizable(false).frame(egui::Frame::none().fill(crate::theme::surface()).inner_margin(4.0).stroke(Stroke::new(1.0_f32, hairline()))).show(ctx, |ui| {
                if ui.button("‹").on_hover_text("Werkzeugpfade zeigen").clicked() {
                    self.paths_open = true;
                }
            });
            return;
        }
        egui::SidePanel::right("toolpaths").exact_width(320.0).resizable(false).frame(egui::Frame::none().fill(crate::theme::panel()).inner_margin(egui::Margin::symmetric(10.0, 8.0)).stroke(Stroke::new(1.0_f32, crate::theme::separator()))).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Werkzeugpfade").size(14.5).strong().color(ink()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("›").on_hover_text("Werkzeugpfade einklappen").clicked() {
                        self.paths_open = false;
                    }
                });
            });
            ui.add_space(4.0);
            // 1. the workpiece first: it is what everything else depends on
            self.setup_row(ui);
            ui.add_space(6.0);
            // 2. the operations that can be added
            self.operation_buttons(ui);
            ui.add_space(6.0);
            // 4. preview and saving stay fixed at the bottom of the panel; the list takes the room
            //    between (the bottom parts are laid out first, so they keep their place)
            if self.toolpath_tab == ToolpathTab::Create {
                self.toolpath_tab = ToolpathTab::Preview;
            }
            egui::TopBottomPanel::bottom("toolpath_bottom").frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 0.0, top: 6.0, bottom: 0.0 })).show_separator_line(true).show_inside(ui, |ui| {
                // the second group of the panel, named like the first one at the top
                ui.label(egui::RichText::new("Simulation & Export").size(14.5).strong().color(ink()));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    for (tab, icon, hint) in [(ToolpathTab::Preview, Icon::Preview, "Werkzeugpfade-Vorschau"), (ToolpathTab::Save, Icon::Save, "Werkzeugpfade speichern")] {
                        ui.add_space(22.0);
                        if icon_button(ui, icon, self.toolpath_tab == tab, hint).clicked() {
                            self.toolpath_tab = tab;
                            if tab == ToolpathTab::Preview {
                                self.mode = ViewMode::ThreeD;
                                self.start_page = false;
                            }
                        }
                    }
                });
                ui.add_space(4.0);
                match self.toolpath_tab {
                    ToolpathTab::Preview | ToolpathTab::Create => self.preview_tab(ui),
                    ToolpathTab::Save => self.save_tab(ui),
                }
                ui.add_space(4.0);
            });
            // 3a. the paths made so far: they fill what is left and scroll when there are many
            egui::Frame::none().fill(crate::theme::surface()).stroke(Stroke::new(1.0_f32, hairline())).rounding(4.0).inner_margin(4.0).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height((ui.available_height() - 8.0).max(60.0));
                egui::ScrollArea::vertical().id_salt("toolpath_list").auto_shrink([false, false]).show(ui, |ui| {
                    self.toolpath_list(ui);
                    self.time_note(ui);
                });
            });
        });
        self.editor_window(ctx);
    }

    /// The editor of a toolpath as its own small window (a double click on a path, or the pencil
    /// on its row, or one of the operation buttons), so the list stays short.
    fn editor_window(&mut self, ctx: &egui::Context) {
        if self.editor.is_none() {
            return;
        }
        let screen = ctx.screen_rect();
        egui::Window::new("Werkzeugweg")
            .id(egui::Id::new("toolpath_editor_window"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .default_pos(Pos2::new((screen.right() - 320.0 - 380.0).max(40.0), 80.0))
            .default_width(340.0)
            .show(ctx, |ui| self.editor_ui(ui));
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
                    ui.label(egui::RichText::new("⚠").color(Color32::from_rgb(0xc0, 0x50, 0x00))).on_hover_text("Die Platte hat keine gültige Größe: bitte einrichten");
                }
                ui.label(egui::RichText::new("Platten-Setup").color(ink()));
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
        if click.on_hover_text("Werkstück einrichten: Größe, Material, Nullpunkt").clicked() {
            self.job_dialog = Some((self.job.clone(), false));
        }
    }

    /// The five operations a toolpath can be made with.
    fn operation_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::splat(GAP);
            // in the original's order (0x10052b5f4): cutout, profile, pocket, drilling, V-carve
            for (icon, kind, hint) in [
                (Icon::Cutout, "cutout", "Ausschnitt (Teil heraustrennen)"),
                (Icon::Profile, "profile", "Profil-Werkzeugweg"),
                (Icon::Pocket, "pocket", "Taschen-Werkzeugweg"),
                (Icon::Drill, "drill", "Bohr-Werkzeugweg"),
                (Icon::VCarve, "vcarve", "V-Carving-Werkzeugweg"),
            ] {
                let on = self.editor.as_ref().is_some_and(|e| e.kind == kind && e.index.is_none());
                if icon_button(ui, icon, on, hint).clicked() {
                    self.open_editor(kind);
                }
            }
        });
    }

    fn preview_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Vorschau").size(15.0).color(ink()));
        ui.horizontal(|ui| {
            if ui.button("▶ Abspielen").clicked() {
                self.start_preview(true);
            }
            if ui.button("⏭ Sofort").clicked() {
                self.start_preview(false);
            }
            if ui.button("↺").on_hover_text("Material zurücksetzen").clicked() {
                self.sim = None;
            }
        });
        // "Preview Type" of the original: which toolpaths are simulated
        ui.horizontal_wrapped(|ui| {
            use super::PreviewScope;
            for (scope, name) in [(PreviewScope::All, "Alle"), (PreviewScope::Selected, "Ausgewählter"), (PreviewScope::Visible, "Sichtbare")] {
                ui.radio_value(&mut self.preview_scope, scope, name);
            }
        })
        .response
        .on_hover_text("Welche Werkzeugwege die Vorschau zeigt (der ausgewählte ist der in der Liste angeklickte)");
        // the sliders take all the width that is left beside their labels
        const LABEL: f32 = 84.0;
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(Vec2::new(LABEL, 18.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.set_min_width(LABEL);
                ui.label("Tempo")
            });
            ui.spacing_mut().slider_width = ui.available_width().max(60.0);
            // both sliders run from 0 to 100, as the original's (wxSlider 0..100 at 0x1000fc110 and
            // 0x1000fd37c); the value behind is spread logarithmically
            let mut pos = to_slider(self.sim_speed as f64, 5.0, 400.0);
            if ui.add(egui::Slider::new(&mut pos, 0.0..=100.0).integer().show_value(false)).on_hover_text(format!("{pos:.0} %")).changed() {
                self.sim_speed = from_slider(pos, 5.0, 400.0).round() as usize;
            }
        });
        // the original's simulation quality, in grid points per mm; called "Darstellung" here,
        // because "Qualität" sounds like the quality of the toolpaths, and this only sets how
        // finely the material is drawn in the preview
        let spacing = sim::grid_spacing(self.job.width, self.job.height, self.sim_quality);
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(Vec2::new(LABEL, 18.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.set_min_width(LABEL);
                ui.label("Darstellung")
            });
            ui.spacing_mut().slider_width = ui.available_width().max(60.0);
            let mut pos = to_slider(self.sim_quality, 1.0, sim::SIMULATION_QUALITY_MAX);
            if ui.add(egui::Slider::new(&mut pos, 0.0..=100.0).integer().show_value(false))
                .on_hover_text(format!("{pos:.0} %\nWie fein das Material in der Vorschau gezeichnet wird (ändert nicht die Werkzeugwege)\n{} Punkte je mm · Raster {:.2} mm", (self.sim_quality * 10.0).round() / 10.0, spacing))
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
                ui.label(egui::RichText::new("schneller").small().color(muted()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("genauer").small().color(muted()));
                });
            });
        });
        ui.checkbox(&mut self.paths_in_3d, "Werkzeugpfade im 3D einblenden");
    }

    fn save_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Export").size(15.0).color(ink()));
        // the three choices of the original's save form
        use super::OutputScope;
        for (scope, name) in [
            (OutputScope::Selected, "Ausgewählter Werkzeugweg"),
            (OutputScope::Visible, "Sichtbare Werkzeugwege"),
        ] {
            ui.radio_value(&mut self.output_scope, scope, name);
        }
        let count = self.output_indices().len();
        ui.label(egui::RichText::new(match self.output_scope {
            OutputScope::Selected => format!("{count} Werkzeugweg wird gespeichert (in der Liste anklicken)."),
            OutputScope::Visible => {
                let tools = self.tool_groups(&self.output_indices()).len();
                if tools > 1 {
                    format!("{count} Werkzeugwege mit {tools} Werkzeugwechseln: beim Speichern wählen Sie eine Datei oder je Werkzeug eine.")
                } else {
                    format!("{count} Werkzeugwege kommen in eine Datei.")
                }
            }
        }).color(muted()));
        ui.add_space(4.0);
        // the original has two lists, "Machine" and "Post-processor" (0x100110f38); here the
        // post-processor is all there is to choose
        ui.label("Postprozessor").on_hover_text("Bestimmt, wie der G-Code geschrieben wird (Einheit, Dialekt). Das Original trennt zusätzlich eine Maschinenliste ab; sie fehlt hier noch.");
        egui::ComboBox::from_id_salt("post").width(260.0).selected_text(self.post.label()).show_ui(ui, |ui| {
            for p in Post::ALL {
                ui.selectable_value(&mut self.post, p, p.label());
            }
        });
        ui.add_space(6.0);
        if ui.add_enabled(count > 0, egui::Button::new("Werkzeugpfade speichern…").min_size(Vec2::new(200.0, 26.0))).clicked() {
            self.export_gcode();
        }
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
            ui.label(egui::RichText::new(format!("Σ Fräszeit (sichtbar) {}", fmt_time(total))).small().color(muted()))
                .on_hover_text(format!("Gesamte Fräszeit der sichtbaren Werkzeugwege: {total:.2} Minuten"));
        });
    }

    fn toolpath_list(&mut self, ui: &mut egui::Ui) {
        let all_hidden = self.toolpaths.iter().all(|t| !t.visible);
        let label = if all_hidden { "Alle einblenden" } else { "Alle ausblenden" };
        ui.horizontal(|ui| {
            if eye(ui, !all_hidden, label).clicked() {
                for t in &mut self.toolpaths {
                    t.visible = all_hidden;
                }
                self.sim = None;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("↻").on_hover_text("Alle Werkzeugpfade neu berechnen").clicked() {
                    self.recompute_all();
                    self.status = "Alle Werkzeugpfade neu berechnet".into();
                }
            });
        });
        if self.toolpaths.is_empty() {
            ui.label(egui::RichText::new("Noch keine Werkzeugpfade.").color(muted()));
            return;
        }
        let (mut remove, mut edit, mut swap) = (None, None, None);
        let mut select_shapes: Option<Vec<u64>> = None;
        let mut pick_row: Option<usize> = None;
        let n = self.toolpaths.len();
        let mut rows: Vec<Rect> = Vec::new();
        let mut grips: Vec<Rect> = Vec::new();
        let mut drag_started = None;
        let dragging = self.tp_drag;
        // a small entry above the first toolpath and wherever the tool changes: from here on this tool cuts
        let tool_entries = crate::toolgroups::headers(&self.toolpaths.iter().map(|t| &t.tool).collect::<Vec<_>>());
        for (i, tp) in self.toolpaths.iter_mut().enumerate() {
            if tool_entries[i] {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(format!("Werkzeug: {}", tp.tool.name)).small().strong().color(muted()))
                        .on_hover_text("Ab hier fräst dieses Werkzeug, bis ein anderes eingetragen ist. Beim Speichern lässt sich je Werkzeug eine eigene Datei schreiben.");
                });
            }
            let active = self.editor.as_ref().is_some_and(|e| e.index == Some(i));
            let frame = egui::Frame::none().fill(if active || self.tp_selected == Some(i) { crate::theme::row_active() } else { crate::theme::row() }).rounding(4.0).inner_margin(egui::Margin::symmetric(6.0, 3.0));
            let row = frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    // the grip: drag it to put this toolpath somewhere else in the list
                    let (grip, grip_resp) = ui.allocate_exact_size(Vec2::new(10.0, 18.0), Sense::drag());
                    let tone = if grip_resp.hovered() || dragging == Some(i) { ink() } else { crate::theme::grip() };
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
                    let tip = format!("{}\n{}: Start {:.1} mm, Schnitt {:.1} mm\nGesamte Fräszeit: {:.2} Minuten ({})\nDoppelklick: bearbeiten", tp.name, tp.strategy.label(), tp.start_depth, tp.cut_depth, minutes, fmt_time(minutes));
                    let (sq, sq_resp) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                    ui.painter().rect_filled(sq, 2.0, tp_color(&tp.strategy));
                    sq_resp.on_hover_text(&tip);
                    // what the toolpath does, as the little slab of the operation's button
                    let (pic, pic_resp) = ui.allocate_exact_size(Vec2::new(22.0, 18.0), Sense::hover());
                    super::icons::operation_icon(ui.painter(), pic.center(), &tp.strategy, 22.0);
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
                            if ui.small_button("🗑").on_hover_text("Löschen").clicked() {
                                remove = Some(i);
                            }
                            if ui.small_button("✎").on_hover_text("Bearbeiten").clicked() {
                                edit = Some(i);
                            }
                            if i + 1 < n && ui.small_button("⏷").on_hover_text("Nach unten").clicked() {
                                swap = Some(i);
                            }
                            if i > 0 && ui.small_button("⏶").on_hover_text("Nach oben").clicked() {
                                swap = Some(i - 1);
                            }
                        }
                        ui.label(egui::RichText::new(fmt_time(tp.stats().minutes)).small().color(muted()));
                    });
                });
                if let Some(w) = &tp.warning {
                    ui.label(egui::RichText::new(format!("⚠ {w}")).small().color(Color32::from_rgb(0xc0, 0x50, 0x00)));
                }
            });
            rows.push(row.response.rect);
        }
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
        if let Some(i) = swap {
            self.tp_selected = None;
            self.toolpaths.swap(i, i + 1);
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

/// A small eye (crossed out when hidden) with an optional label, clickable.
pub(super) fn eye(ui: &mut egui::Ui, open: bool, label: &str) -> egui::Response {
    let text_w = if label.is_empty() { 0.0 } else { ui.fonts(|f| f.layout_no_wrap(label.to_owned(), FontId::proportional(13.0), ink()).size().x) + 8.0 };
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(22.0 + text_w, 20.0), Sense::click());
    let p = ui.painter();
    let c = Pos2::new(rect.left() + 11.0, rect.center().y);
    super::icons::paint_eye(p, c, open, super::icons::Colours::normal(), resp.hovered());
    if !label.is_empty() {
        p.text(Pos2::new(rect.left() + 26.0, rect.center().y), Align2::LEFT_CENTER, label, FontId::proportional(13.0), if resp.hovered() { crate::theme::icon_line() } else { ink() });
    }
    resp
}

/// One group of toolbar buttons in a thin rounded box with its name centred under the buttons.
fn toolbar_group(ui: &mut egui::Ui, name: &str, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none().stroke(Stroke::new(1.0_f32, crate::theme::separator())).rounding(5.0).fill(crate::theme::surface()).inner_margin(egui::Margin { left: 7.0, right: 7.0, top: 3.0, bottom: 2.0 }).show(ui, |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            let row = ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = GAP;
                add(ui);
            });
            // the name under the buttons, as wide as they are, so it is centred and the box stays
            // only as wide as its content
            ui.allocate_ui_with_layout(Vec2::new(row.response.rect.width(), 12.0), egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(name).size(10.0).color(muted()));
            });
        });
    });
}

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
