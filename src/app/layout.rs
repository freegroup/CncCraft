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
/// `binary_operation` and `group_ungroup`; the job setup sits apart at the end. The original's
/// `bitmap`, `join` and `create_fillets` groups have no tools here yet, so they are left out;
/// duplicating is no tool there either (menu and ⌘D).
const GROUPS: [&[Icon]; GROUP_COUNT] = [
    &[Icon::Draw(DrawTool::Select), Icon::Transform, Icon::Draw(DrawTool::NodeEdit)],
    &[Icon::Draw(DrawTool::Polyline)],
    &[Icon::Draw(DrawTool::Rect), Icon::Draw(DrawTool::Circle), Icon::Draw(DrawTool::Ellipse), Icon::Draw(DrawTool::Polygon), Icon::ImportDxf],
    &[Icon::Draw(DrawTool::Text)],
    &[Icon::Delete, Icon::Curves],
    &[Icon::Center, Icon::MirrorH, Icon::MirrorV],
    &[Icon::Offset],
    &[Icon::Union, Icon::Subtract, Icon::Intersect],
    &[Icon::Group, Icon::Ungroup],
];

/// How many groups the tool column has (one remembered choice each when folded).
pub(super) const GROUP_COUNT: usize = 9;

/// What each group is, for the folded column's tooltips.
const GROUP_NAMES: [crate::lang::Tr; GROUP_COUNT] = [tr!("Modus"), tr!("Linien zeichnen"), tr!("Formen zeichnen"), tr!("Text"), tr!("Objekte bearbeiten"), tr!("Ausrichten und Spiegeln"), tr!("Versatz"), tr!("Boolesche Operationen"), tr!("Gruppieren")];

/// Icons per row in the expanded column, and the size of one icon cell.
const ROW: usize = 5;
const CELL: f32 = 38.0;
const GAP: f32 = 4.0;

fn tip(icon: Icon) -> String {
    match icon {
        Icon::Draw(t) => t.tip(),
        Icon::Transform => t!("Objekte transformieren (Lage und Größe eingeben)"),
        Icon::Offset => t!("Vektoren versetzen"),
        Icon::MirrorH => t!("Waagrecht spiegeln"),
        Icon::MirrorV => t!("Senkrecht spiegeln"),
        Icon::Center => t!("Auf dem Material zentrieren"),
        Icon::Curves => t!("In Kurven umwandeln"),
        Icon::Delete => t!("Löschen"),
        Icon::Union => t!("Vereinen (mindestens zwei Formen)"),
        Icon::Subtract => t!("Abziehen: die zuerst gewählte Form minus die anderen"),
        Icon::Intersect => t!("Schnittmenge (mindestens zwei Formen)"),
        Icon::Job => t!("Job einrichten (Größe, Material, Nullpunkt)"),
        Icon::Group => t!("Gruppieren (⌘G): die Auswahl wird eine Gruppe; auch Gruppen lassen sich gruppieren"),
        Icon::Ungroup => t!("Gruppe auflösen (⌘⇧G): löst die äußerste Gruppe der Auswahl auf, eine Ebene je Klick"),
        Icon::ImportDxf => t!("DXF-Zeichnung importieren (R12, R14 und neuer); Rechtsklick auf eine importierte Form: neu von Platte laden"),
        Icon::Variables => t!("Variablen (f(x): benannte Werte für die Maße)"),
        _ => String::new(),
    }
}

/// Space between the theme switch and the language switch's centre.
const LANG_SLOT: f32 = 38.0;

impl App {
    /// The menu bar on the left, the layer in the middle, the document's name on the right.
    pub(super) fn title_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("title").exact_height(34.0).frame(egui::Frame::none().fill(crate::theme::surface()).inner_margin(egui::Margin::symmetric(10.0, 4.0)).stroke(Stroke::new(1.0_f32, hairline()))).show(ctx, |ui| {
            let full = ui.max_rect();
            // the start page has no menus: only the switches at the right end
            if !self.start_page {
                // the usual menu bar, every menu at the top level
                ui.horizontal_centered(|ui| {
                    ui.label(egui::RichText::new("⚡").color(ACCENT).size(16.0));
                    egui::menu::bar(ui, |ui| self.menus(ui));
                });
                // the document's name at the right end
                let name = self.path.as_ref().and_then(|p| p.file_stem()).map_or_else(|| t!("Neu"), |s| s.to_string_lossy().into_owned());
                let right = full.right_center() - Vec2::new(38.0 + LANG_SLOT, 0.0);
                // unsaved changes: a dot behind the name (drawn, the font has no such glyph)
                let dot_room = if self.dirty_shown { 16.0 } else { 0.0 };
                let label = ui.painter().text(right - Vec2::new(dot_room, 0.0), Align2::RIGHT_CENTER, name, FontId::proportional(14.0), muted());
                if self.dirty_shown {
                    let dot = Rect::from_center_size(Pos2::new(right.x - 5.0, right.y), Vec2::splat(10.0));
                    ui.painter().circle_filled(dot.center(), 3.5, muted());
                    ui.interact(label.union(dot), egui::Id::new("dirty_mark"), Sense::hover()).on_hover_text(t!("Ungespeicherte Änderungen"));
                }
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
            if resp.on_hover_text(if self.dark { t!("Zum hellen Design wechseln") } else { t!("Zum dunklen Design wechseln") }).clicked() {
                let choice = if self.dark { crate::theme::Choice::Light } else { crate::theme::Choice::Dark };
                self.set_theme_choice(ctx, choice);
            }
            // the language switch, left of the theme switch: the code of the language in use, and
            // a list of the languages under it
            let lang_slot = Rect::from_center_size(slot.center() - Vec2::new(LANG_SLOT, 0.0), Vec2::new(34.0, 26.0));
            let lang_resp = ui.interact(lang_slot, egui::Id::new("lang_switch"), Sense::click());
            if lang_resp.hovered() {
                ui.painter().rect_filled(lang_slot, 5.0, crate::theme::hover());
            }
            ui.painter().text(lang_slot.center(), Align2::CENTER_CENTER, self.lang.code().to_uppercase(), FontId::proportional(13.0), crate::theme::icon_line());
            let popup_id = egui::Id::new("lang_popup");
            if lang_resp.clone().on_hover_text(t!("Sprache wählen")).clicked() {
                ui.memory_mut(|m| m.toggle_popup(popup_id));
            }
            let mut picked = None;
            egui::popup::popup_below_widget(ui, popup_id, &lang_resp, egui::popup::PopupCloseBehavior::CloseOnClick, |ui| {
                ui.set_min_width(110.0);
                for lang in crate::lang::Lang::ALL {
                    if ui.selectable_label(self.lang == lang, lang.native_name()).clicked() {
                        picked = Some(lang);
                    }
                }
            });
            if let Some(lang) = picked {
                self.set_lang(ctx, lang);
            }
            // the layer, centred over the window as in the original (one layer for now)
            if !self.start_page {
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(Rect::from_center_size(full.center(), Vec2::new(230.0, 26.0))), |ui| self.layer_selector(ui));
            }
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
                let hint = if self.tools_open { t!("Werkzeugleiste verkleinern") } else { t!("Werkzeugleiste erweitern") };
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
                        let resp = resp.on_hover_text(GROUP_NAMES[g].get());
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
                ui.label(egui::RichText::new(GROUP_NAMES[g].get()).small().color(muted()));
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
            // the import is an action, not a tool: the folded button keeps showing the last tool
            if icon != Icon::ImportDxf {
                self.tool_choice[g] = k;
            }
            self.flyout = None;
            return Some(icon);
        }
        let pressed_outside = ctx.input(|i| i.pointer.any_pressed() && i.pointer.interact_pos().is_some_and(|p| !area.response.rect.contains(p) && p.x > at.x));
        if pressed_outside {
            self.flyout = None;
        }
        None
    }

    /// Whether a tool column icon is switched on, and whether it can be used now.
    pub(super) fn icon_state(&self, icon: Icon) -> (bool, bool) {
        let any = !self.edit.selection.is_empty();
        match icon {
            Icon::Draw(t) => (self.tool == t, true),
            Icon::Transform => (self.tool_form == Some(ToolForm::Transform), true),
            Icon::Offset => (self.tool_form == Some(ToolForm::Offset), true),
            Icon::Union | Icon::Subtract | Icon::Intersect => (false, self.edit.selection.len() >= 2),
            Icon::Group => (false, self.selection_can_group()),
            Icon::Ungroup => (false, self.selection_has_group()),
            Icon::Job | Icon::ImportDxf => (false, true),
            Icon::Variables => (self.vars_open, true),
            _ => (false, any),
        }
    }

    fn column_icon(&self, ui: &mut egui::Ui, icon: Icon) -> egui::Response {
        let (selected, enabled) = self.icon_state(icon);
        ui.add_enabled_ui(enabled, |ui| icon_button(ui, icon, selected, &tip(icon))).inner
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
            Icon::Group => self.group_selection(),
            Icon::Ungroup => self.ungroup_selection(),
            Icon::Job => self.job_dialog = Some((self.job.clone(), false)),
            Icon::ImportDxf => self.import_dialog(),
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
                    ToolForm::Transform => t!("Objekte transformieren"),
                    ToolForm::Offset => t!("Versatz"),
                };
                ui.label(egui::RichText::new(title).size(15.0).color(ink()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕").on_hover_text(t!("Schließen")).clicked() {
                        self.tool_form = None;
                    }
                });
            });
            ui.separator();
            match form {
                ToolForm::Transform => self.properties(ui),
                ToolForm::Offset => {
                    ui.horizontal(|ui| {
                        ui.label(t!("Abstand"));
                        ui.add(egui::DragValue::new(&mut self.offset_dist).range(0.1..=500.0).speed(0.1).suffix(" mm"));
                    });
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.offset_outside, true, t!("Außen"));
                        ui.radio_value(&mut self.offset_outside, false, t!("Innen"));
                    });
                    ui.add_space(6.0);
                    let any = !self.edit.selection.is_empty();
                    if ui.add_enabled(any, egui::Button::new(t!("Versetzen")).min_size(Vec2::new(100.0, 24.0))).clicked() {
                        self.offset_selection();
                    }
                    if !any {
                        ui.label(egui::RichText::new(t!("Erst die Vektoren auswählen.")).color(muted()));
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
                toolbar_group(ui, &t!("Dekoration"), |ui| {
                    let shown = self.origin_shown();
                    if icon_button(ui, Icon::Origin, shown, &t!("Nullpunkt anzeigen / ausblenden (Achsenkreuz im Blatt und in der 3D-Ansicht)")).clicked() {
                        self.set_origin_shown(!shown);
                    }
                    if icon_button(ui, Icon::Grid, self.snap_grid, &t!("Am Raster einrasten")).clicked() {
                        self.snap_grid = !self.snap_grid;
                    }
                    if icon_button(ui, Icon::Snap, self.snap_geometry, &t!("An Geometrie einrasten")).clicked() {
                        self.snap_geometry = !self.snap_geometry;
                    }
                    if icon_button(ui, Icon::Dimensions, self.show_dims, &t!("Maße anzeigen / ausblenden (Maße blau = Constraint; Linien sind immer blau = frei, schwarz = Länge festgelegt)")).clicked() {
                        self.show_dims = !self.show_dims;
                    }
                });
                toolbar_group(ui, &t!("Constraints"), |ui| {
                    if icon_button(ui, Icon::Distance, self.dim_mode.is_some(), &t!("Abstand bemaßen (D): zwei Punkte oder Punkt und Linie wählen, dann den Abstand eingeben")).clicked() {
                        self.toggle_dim_mode();
                    }
                    if icon_button(ui, Icon::Align, self.align_mode.is_some(), &t!("Waagrecht/Senkrecht (H): eine Linie oder zwei Punkte wählen, CncCraft legt sie waagrecht oder senkrecht fest")).clicked() {
                        self.toggle_align_mode();
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icon_button(ui, Icon::ZoomSelection, false, &t!("Auf die Auswahl zoomen")).clicked() {
                        self.zoom_to_selection();
                    }
                    if icon_button(ui, Icon::ZoomFit, false, &t!("Ganze Ansicht")).clicked() {
                        self.run(Action::FitView);
                    }
                    let (label, next, hint) = match self.mode {
                        ViewMode::TwoD => ("2D", ViewMode::ThreeD, t!("Zur 3D-Ansicht wechseln")),
                        ViewMode::ThreeD => ("3D", ViewMode::TwoD, t!("Zur 2D-Ansicht wechseln")),
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
                if ui.button("‹").on_hover_text(t!("Werkzeugpfade zeigen")).clicked() {
                    self.paths_open = true;
                }
            });
            return;
        }
        egui::SidePanel::right("toolpaths").exact_width(320.0).resizable(false).frame(egui::Frame::none().fill(crate::theme::panel()).inner_margin(egui::Margin::symmetric(10.0, 8.0)).stroke(Stroke::new(1.0_f32, crate::theme::separator()))).show(ctx, |ui| {
            // the bottom container is laid out first, so it keeps its place; the top container
            // takes the room that is left
            if self.toolpath_tab == ToolpathTab::Create {
                self.toolpath_tab = ToolpathTab::Preview;
            }
            egui::TopBottomPanel::bottom("toolpath_bottom").frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 0.0, top: 8.0, bottom: 0.0 })).show_separator_line(false).show_inside(ui, |ui| {
                group_container().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    group_title(ui, &t!("Simulation & Export"), |_| {});
                    ui.horizontal(|ui| {
                        for (tab, icon, hint) in [(ToolpathTab::Preview, Icon::Preview, t!("Werkzeugpfade-Vorschau")), (ToolpathTab::Save, Icon::Save, t!("Werkzeugpfade speichern"))] {
                            ui.add_space(22.0);
                            if icon_button(ui, icon, self.toolpath_tab == tab, &hint).clicked() {
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

    /// The editor of a toolpath as its own small window (a double click on a path, or the pencil
    /// on its row, or one of the operation buttons), so the list stays short.
    fn editor_window(&mut self, ctx: &egui::Context) {
        if self.editor.is_none() {
            return;
        }
        let screen = ctx.screen_rect();
        egui::Window::new(t!("Werkzeugweg"))
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
        ui.label(egui::RichText::new(t!("Vorschau")).size(15.0).color(ink()));
        ui.horizontal(|ui| {
            if ui.button(t!("▶ Simulieren")).clicked() {
                self.start_preview(true);
            }
            if ui.button(t!("⏭ Sofort")).clicked() {
                self.start_preview(false);
            }
            if ui.button("↺").on_hover_text(t!("Material zurücksetzen")).clicked() {
                self.sim = None;
            }
        });
        // "Preview Type" of the original: which toolpaths are simulated
        ui.horizontal_wrapped(|ui| {
            use super::PreviewScope;
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

    fn save_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new(t!("Export")).size(15.0).color(ink()));
        // the three choices of the original's save form
        use super::OutputScope;
        for (scope, name) in [
            (OutputScope::All, t!("Alle Werkzeugwege")),
            (OutputScope::Visible, t!("Sichtbare Werkzeugwege")),
        ] {
            ui.radio_value(&mut self.output_scope, scope, name);
        }
        let count = self.output_indices().len();
        let tools = self.tool_groups(&self.output_indices()).len();
        ui.label(egui::RichText::new(if tools > 1 {
            t!("%{count} Werkzeugwege mit %{tools} Werkzeugwechseln: beim Speichern wählen Sie eine Datei oder je Werkzeug eine.", count = count, tools = tools)
        } else {
            t!("%{count} Werkzeugwege kommen in eine Datei.", count = count)
        }).color(muted()));
        ui.add_space(4.0);
        ui.label(t!("Postprozessor"));
        let selected_label = self.pp().label();
        let pp_manage;
        {
            let post_idx = &mut self.post_idx;
            let postprocs = &self.postprocs;
            pp_manage = combo_with_manage(ui, "post", 260.0, &selected_label, |ui| {
                for (i, pp) in postprocs.iter().enumerate() {
                    if ui.selectable_label(*post_idx == i, pp.label()).clicked() {
                        *post_idx = i;
                    }
                }
            });
        }
        if pp_manage {
            let idx = self.post_idx.min(self.postprocs.len().saturating_sub(1));
            self.pp_dialog = Some(super::PpDialog {
                editing: self.postprocs[idx].clone(),
                index: Some(idx),
                hook_tab: 0,
            });
        }

        ui.add_space(6.0);
        if ui.add_enabled(count > 0, egui::Button::new(t!("Werkzeugpfade speichern…")).min_size(Vec2::new(200.0, 26.0))).clicked() {
            self.export_gcode();
        }
    }

    pub(super) fn pp_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(dlg) = &mut self.pp_dialog else { return };
        let title = if dlg.index.is_some() { t!("Postprozessor-Verwaltung") } else { t!("Neuer Postprozessor") };
        let mut save = false;
        let mut delete = false;
        let mut pick: Option<usize> = None;
        let mut close = false;
        let is_builtin = dlg.editing.builtin;

        let mut open = true;
        let screen = ctx.screen_rect();
        egui::Window::new(title)
            .id(egui::Id::new("pp_dialog_win"))
            .open(&mut open)
            .order(egui::Order::Foreground)
            .resizable(true)
            .default_width(560.0)
            .min_width(400.0)
            .default_height((screen.height() - 120.0).clamp(300.0, 520.0))
            .max_height(screen.height() - 60.0)
            .show(ctx, |ui| {
                // Bottom buttons first so they are always visible
                egui::TopBottomPanel::bottom("pp_buttons").frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 0.0, top: 6.0, bottom: 4.0 })).show_separator_line(true).show_inside(ui, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(egui::RichText::new(t!("Speichern")).strong().color(super::INK)).fill(super::ACCENT).min_size(Vec2::new(100.0, 30.0)).rounding(4.0)).clicked() {
                            save = true;
                        }
                        ui.add_space(4.0);
                        if ui.add(egui::Button::new(t!("Abbrechen")).min_size(Vec2::new(90.0, 30.0)).rounding(4.0)).clicked() {
                            close = true;
                        }
                    });
                });

                // Left list panel
                egui::SidePanel::left("pp_list_panel").exact_width(150.0).resizable(false).frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 4.0, top: 0.0, bottom: 0.0 })).show_inside(ui, |ui| {
                    ui.label(egui::RichText::new(t!("Postprozessoren")).strong());
                    // +/- buttons pinned at the bottom
                    egui::TopBottomPanel::bottom("pp_list_buttons").frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 0.0, top: 4.0, bottom: 0.0 })).show_separator_line(false).show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.small_button("+").on_hover_text(t!("Neuen Postprozessor anlegen")).clicked() {
                                pick = Some(usize::MAX);
                            }
                            let can_delete = dlg.index.is_some() && !is_builtin;
                            if ui.add_enabled(can_delete, egui::Button::new("−").small()).on_hover_text(t!("Ausgewählten Postprozessor löschen")).clicked() {
                                delete = true;
                            }
                        });
                    });
                    // List fills the rest and scrolls
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        for (i, pp) in self.postprocs.iter().enumerate() {
                            let selected = dlg.index == Some(i);
                            if ui.selectable_label(selected, &pp.name).clicked() {
                                pick = Some(i);
                            }
                        }
                    });
                });

                // Right side: editor fills the rest
                Self::pp_editor(ui, dlg);
            });

        if let Some(p) = pick {
            if p == usize::MAX {
                self.pp_dialog = Some(super::PpDialog {
                    editing: crate::gcode::PostProcessor {
                        name: t!("Neuer Postprozessor").into(),
                        inch: false, ext: "nc".into(), tool_change: true,
                        job_start: String::new(), job_end: String::new(),
                        on_tool_change: "G0 Z{safe_z}\nM5\nT{tool_number} M6\nM0 (Werkzeugwechsel – weiter mit Cycle Start)".into(),
                        before_path: String::new(), after_path: String::new(),
                        builtin: false,
                    },
                    index: None,
                    hook_tab: 0,
                });
                return;
            } else if p < self.postprocs.len() {
                if let Some(dlg) = &mut self.pp_dialog {
                    dlg.editing = self.postprocs[p].clone();
                    dlg.index = Some(p);
                }
            }
            return;
        }

        if save {
            if let Some(dlg) = self.pp_dialog.take() {
                if let Some(idx) = dlg.index {
                    if idx < self.postprocs.len() {
                        self.postprocs[idx] = dlg.editing;
                    }
                } else {
                    self.postprocs.push(dlg.editing);
                    self.post_idx = self.postprocs.len() - 1;
                }
                self.status = "Postprozessor gespeichert".into();
            }
            return;
        }

        if delete {
            if let Some(dlg) = self.pp_dialog.take() {
                if let Some(idx) = dlg.index {
                    if idx < self.postprocs.len() && !self.postprocs[idx].builtin {
                        self.postprocs.remove(idx);
                        if self.post_idx >= self.postprocs.len() {
                            self.post_idx = self.postprocs.len().saturating_sub(1);
                        }
                        self.status = "Postprozessor gelöscht".into();
                    }
                }
            }
            return;
        }

        if close {
            self.pp_dialog = None;
        }
    }

    fn pp_editor(ui: &mut egui::Ui, dlg: &mut super::PpDialog) {
        let pp = &mut dlg.editing;
        ui.label(egui::RichText::new(t!("Name")).small().strong());
        ui.text_edit_singleline(&mut pp.name);
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(t!("Dateiendung:"));
            egui::TextEdit::singleline(&mut pp.ext).desired_width(60.0).show(ui);
        });
        ui.checkbox(&mut pp.tool_change, t!("Werkzeugwechsel unterstützt"));
        ui.add_space(6.0);

        const TABS: &[(crate::lang::Tr, crate::lang::Tr)] = &[
            (tr!("Job Start"), tr!("Wird einmal am Anfang eingefügt.\nPlatzhalter: {safe_z} {home_z} {tool_number} {rpm}")),
            (tr!("Vor Werkzeugweg"), tr!("Vor jedem Werkzeugweg (nach S/M3).")),
            (tr!("Werkzeugwechsel"), tr!("Zwischen zwei Werkzeugwegen mit unterschiedlichem Werkzeug.")),
            (tr!("Nach Werkzeugweg"), tr!("Am Ende jedes Werkzeugwegs.")),
            (tr!("Job Ende"), tr!("Wird einmal am Ende eingefügt.")),
        ];

        ui.horizontal(|ui| {
            let filled = [
                !pp.job_start.trim().is_empty(),
                !pp.before_path.trim().is_empty(),
                !pp.on_tool_change.trim().is_empty(),
                !pp.after_path.trim().is_empty(),
                !pp.job_end.trim().is_empty(),
            ];
            for (i, &(label, _)) in TABS.iter().enumerate() {
                let text = if filled[i] {
                    egui::RichText::new(label).color(super::ACCENT)
                } else {
                    egui::RichText::new(label)
                };
                if ui.selectable_label(dlg.hook_tab == i, text).clicked() {
                    dlg.hook_tab = i;
                }
            }
        });
        ui.add_space(2.0);

        let (_, tip) = TABS[dlg.hook_tab];
        ui.label(egui::RichText::new(tip).small().color(muted()));

        let val = match dlg.hook_tab {
            0 => &mut pp.job_start,
            1 => &mut pp.before_path,
            2 => &mut pp.on_tool_change,
            3 => &mut pp.after_path,
            _ => &mut pp.job_end,
        };

        let remaining = (ui.available_height() - 24.0).max(80.0);
        let layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
            super::gcode_highlight::layout_gcode(ui, text, wrap_width)
        };
        let output = egui::TextEdit::multiline(val)
            .font(egui::FontId::monospace(12.0))
            .desired_width(f32::INFINITY)
            .hint_text(t!("(leer = kein Einfügen)"))
            .layouter(&mut layouter.clone())
            .min_size(Vec2::new(0.0, remaining))
            .show(ui);
        super::gcode_highlight::gcode_tooltips(val, &output.response, &output.galley, output.galley_pos);

        ui.add_space(2.0);
        ui.label(egui::RichText::new(t!("Platzhalter: {safe_z} {home_z} {tool_number} {rpm}")).small().color(muted()));
    }

    pub(super) fn tool_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(dlg) = &mut self.tool_dialog else { return };
        let mut save = false;
        let mut delete = false;
        let mut pick: Option<usize> = None;
        let mut close = false;

        let mat = dlg.material.clone();
        let mut open = true;
        let screen = ctx.screen_rect();
        egui::Window::new(t!("Werkzeug-Verwaltung"))
            .id(egui::Id::new("tool_dialog_win"))
            .open(&mut open)
            .order(egui::Order::Foreground)
            .resizable(true)
            .default_width(520.0)
            .min_width(380.0)
            .default_height((screen.height() - 120.0).clamp(300.0, 520.0))
            .max_height(screen.height() - 60.0)
            .show(ctx, |ui| {
                egui::TopBottomPanel::bottom("tool_buttons").frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 0.0, top: 6.0, bottom: 4.0 })).show_separator_line(true).show_inside(ui, |ui| {
                    ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), 30.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(egui::RichText::new(t!("Speichern")).strong().color(super::INK)).fill(super::ACCENT).min_size(Vec2::new(100.0, 30.0)).rounding(4.0)).clicked() {
                            save = true;
                        }
                        ui.add_space(4.0);
                        if ui.add(egui::Button::new(t!("Abbrechen")).min_size(Vec2::new(90.0, 30.0)).rounding(4.0)).clicked() {
                            close = true;
                        }
                    });
                });

                egui::SidePanel::left("tool_list_panel").exact_width(160.0).resizable(false).frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 4.0, top: 0.0, bottom: 0.0 })).show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(t!("Werkzeuge")).strong());
                        ui.label(egui::RichText::new(format!("({})", crate::tools::material_label(&mat))).small().color(muted()));
                    });
                    egui::TopBottomPanel::bottom("tool_list_buttons").frame(egui::Frame::none().inner_margin(egui::Margin { left: 0.0, right: 0.0, top: 4.0, bottom: 0.0 })).show_separator_line(false).show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.small_button("+").on_hover_text(t!("Neues Werkzeug anlegen")).clicked() {
                                pick = Some(usize::MAX);
                            }
                            let can_delete = dlg.index.is_some();
                            if ui.add_enabled(can_delete, egui::Button::new("−").small()).on_hover_text(t!("Ausgewähltes Werkzeug löschen")).clicked() {
                                delete = true;
                            }
                        });
                    });
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        for (i, entry) in dlg.entries.iter().enumerate() {
                            if entry.material != mat {
                                continue;
                            }
                            let is_selected = dlg.index == Some(i);
                            let label = &crate::tools::shown_name(if is_selected { &dlg.editing.name } else { &entry.tool.name });
                            let orig_tool = dlg.original.get(i).map(|e| &e.tool);
                            let changed = if is_selected {
                                orig_tool.map_or(true, |b| dlg.editing != *b)
                            } else {
                                orig_tool.map_or(true, |b| entry.tool != *b)
                            };
                            let text = if changed {
                                egui::RichText::new(label).color(super::ACCENT)
                            } else {
                                egui::RichText::new(label)
                            };
                            if ui.selectable_label(is_selected, text).clicked() {
                                pick = Some(i);
                            }
                        }
                    });
                });

                egui::Frame::none().inner_margin(egui::Margin { left: 10.0, right: 0.0, top: 0.0, bottom: 0.0 }).show(ui, |ui| {
                    Self::tool_editor(ui, dlg);
                });
            });

        if let Some(p) = pick {
            // write current edits back to working copy before switching
            if let Some(dlg) = &mut self.tool_dialog {
                if let Some(idx) = dlg.index {
                    if idx < dlg.entries.len() {
                        dlg.entries[idx].tool = dlg.editing.clone();
                    }
                } else {
                    dlg.entries.push(crate::tools::ToolEntry { material: dlg.material.clone(), tool: dlg.editing.clone() });
                }
            }
            if p == usize::MAX {
                let new_tool = crate::tools::ToolLibrary::default_tool(crate::tools::ToolKind::EndMill);
                if let Some(dlg) = &mut self.tool_dialog {
                    dlg.editing = new_tool;
                    dlg.index = None;
                }
            } else if let Some(dlg) = &mut self.tool_dialog {
                if p < dlg.entries.len() {
                    dlg.editing = dlg.entries[p].tool.clone();
                    dlg.index = Some(p);
                }
            }
            return;
        }

        if save {
            // write current edits to working copy, then propagate to lib
            if let Some(mut dlg) = self.tool_dialog.take() {
                if let Some(idx) = dlg.index {
                    if idx < dlg.entries.len() {
                        dlg.entries[idx].tool = dlg.editing;
                    }
                } else {
                    dlg.entries.push(crate::tools::ToolEntry { material: dlg.material, tool: dlg.editing });
                }
                self.lib.entries = dlg.entries;
                self.status = "Werkzeuge gespeichert".into();
            }
            return;
        }

        if delete {
            if let Some(dlg) = &mut self.tool_dialog {
                if let Some(idx) = dlg.index {
                    if idx < dlg.entries.len() {
                        dlg.entries.remove(idx);
                        self.status = "Werkzeug gelöscht".into();
                    }
                }
                let mat_clone = dlg.material.clone();
                let first = dlg.entries.iter().position(|e| e.material == mat_clone);
                match first {
                    Some(i) => {
                        dlg.editing = dlg.entries[i].tool.clone();
                        dlg.index = Some(i);
                    }
                    None => {
                        dlg.editing = crate::tools::ToolLibrary::default_tool(crate::tools::ToolKind::EndMill);
                        dlg.index = None;
                    }
                }
            }
            return;
        }

        if close || !open {
            // cancel: discard working copy, lib stays unchanged
            self.tool_dialog = None;
        }
    }

    fn tool_editor(ui: &mut egui::Ui, dlg: &mut super::ToolDialog) {
        let t = &mut dlg.editing;
        ui.label(egui::RichText::new(t!("Name")).small().strong());
        ui.text_edit_singleline(&mut t.name);
        ui.add_space(4.0);
        egui::Grid::new("tool_edit_grid").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
            ui.label(t!("Typ"));
            egui::ComboBox::from_id_salt("tool_kind").width(140.0).selected_text(t.kind.group()).show_ui(ui, |ui| {
                for &k in crate::tools::ToolKind::all() {
                    if ui.selectable_label(t.kind == k, k.group()).clicked() {
                        t.kind = k;
                    }
                }
            });
            ui.end_row();
            ui.label(t!("Durchmesser"));
            ui.add(egui::DragValue::new(&mut t.diameter).speed(0.1).range(0.1..=100.0).max_decimals(2).suffix(" mm"));
            ui.end_row();
            if t.kind == crate::tools::ToolKind::VBit || t.kind == crate::tools::ToolKind::Drill {
                ui.label(t!("Winkel"));
                ui.add(egui::DragValue::new(&mut t.angle).speed(1.0).range(1.0..=180.0).suffix(" °"));
                ui.end_row();
            }
            ui.label(t!("Drehzahl"));
            ui.add(egui::DragValue::new(&mut t.rpm).speed(100).range(0..=60000).suffix(t!(" U/min")));
            ui.end_row();
            ui.label(t!("Vorschub"));
            ui.add(egui::DragValue::new(&mut t.feed).speed(10).range(1.0..=20000.0).suffix(" mm/min"));
            ui.end_row();
            ui.label(t!("Eintauchen"));
            ui.add(egui::DragValue::new(&mut t.plunge).speed(10).range(1.0..=20000.0).suffix(" mm/min"));
            ui.end_row();
            ui.label(t!("Zustellung"));
            ui.add(egui::DragValue::new(&mut t.stepdown).speed(0.05).range(0.01..=50.0).max_decimals(3).suffix(" mm"));
            ui.end_row();
            ui.label(t!("Querzustellung"));
            let pct = if t.diameter > 0.0 { t.stepover / t.diameter * 100.0 } else { 0.0 };
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut t.stepover).speed(0.05).range(0.01..=50.0).max_decimals(3).suffix(" mm"));
                ui.label(egui::RichText::new(format!("{pct:.0} %")).small().color(muted()));
            });
            ui.end_row();
        });
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
    pub(super) fn toolpaths_of_selection(&self) -> Vec<usize> {
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
/// A pencil lying at 45°, tip at the lower left: a body with an eraser band and a pointed tip
/// (drawn, because the font has no pencil glyph).
fn draw_pencil(painter: &egui::Painter, c: Pos2, tone: Color32) {
    let d = Vec2::new(0.7071, -0.7071);
    let n = Vec2::new(0.7071, 0.7071);
    let h = 2.8;
    let tip = c - d * 7.5;
    let join = c - d * 3.0;
    let end = c + d * 7.0;
    let stroke = Stroke::new(1.3_f32, tone);
    painter.add(egui::Shape::convex_polygon(vec![join + n * h, end + n * h, end - n * h, join - n * h], Color32::TRANSPARENT, stroke));
    painter.add(egui::Shape::convex_polygon(vec![tip, join + n * h, join - n * h], tone.gamma_multiply(0.45), stroke));
    let band = end - d * 2.4;
    painter.line_segment([band + n * h, band - n * h], stroke);
}

/// The box round what belongs together in the toolpath panel.
fn group_container() -> egui::Frame {
    egui::Frame::none().fill(crate::theme::surface()).stroke(Stroke::new(1.0_f32, hairline())).rounding(6.0).inner_margin(egui::Margin::same(8.0))
}

/// The title of a group box, a line under it, and room at the right end for a control.
fn group_title(ui: &mut egui::Ui, title: &str, right: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(title).size(14.5).strong().color(ink()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), right);
    });
    ui.add_space(2.0);
    let y = ui.cursor().top();
    let (l, r) = (ui.min_rect().left(), ui.min_rect().right());
    ui.painter().line_segment([Pos2::new(l, y), Pos2::new(r, y)], Stroke::new(1.0_f32, hairline()));
    ui.add_space(8.0);
}

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

/// A ComboBox whose popup has a scrollable item list and a pinned "Verwalten…" button below it.
/// `items` draws the selectable rows; returns `true` when "Verwalten…" was clicked.
pub(super) fn combo_with_manage(
    ui: &mut egui::Ui,
    id: &str,
    width: f32,
    selected: &str,
    items: impl FnOnce(&mut egui::Ui),
) -> bool {
    let widget_id = ui.make_persistent_id(id);
    let popup_id = widget_id.with("mgr_popup");
    let is_open = ui.memory(|m| m.is_popup_open(popup_id));

    let margin = ui.spacing().button_padding;
    let icon_size = Vec2::splat(ui.spacing().icon_width);
    let total_w = width.max(ui.spacing().combo_width);
    let galley = ui.painter().layout_no_wrap(
        selected.to_string(),
        egui::FontId::default(),
        ui.visuals().text_color(),
    );
    let desired = Vec2::new(total_w, galley.size().y.max(icon_size.y) + 2.0 * margin.y);
    let (rect, button) = ui.allocate_exact_size(desired, Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = if is_open {
            &ui.visuals().widgets.open
        } else {
            ui.style().interact(&button)
        };
        ui.painter().rect(rect, visuals.rounding, visuals.bg_fill, visuals.bg_stroke);
        let inner = rect.shrink2(margin);
        let icon_rect = egui::Align2::RIGHT_CENTER.align_size_within_rect(icon_size, inner);
        let tri = Rect::from_center_size(icon_rect.center(), Vec2::new(icon_rect.width() * 0.7, icon_rect.height() * 0.45));
        ui.painter().add(egui::Shape::convex_polygon(
            vec![tri.left_top(), tri.right_top(), tri.center_bottom()],
            visuals.fg_stroke.color,
            Stroke::NONE,
        ));
        let text_rect = egui::Align2::LEFT_CENTER.align_size_within_rect(galley.size(), inner);
        ui.painter().galley(text_rect.min, galley, visuals.text_color());
    }

    if button.clicked() {
        ui.memory_mut(|m| m.toggle_popup(popup_id));
    }

    let mut manage = false;
    let above_or_below = if button.rect.center().y > ui.ctx().screen_rect().center().y {
        egui::AboveOrBelow::Above
    } else {
        egui::AboveOrBelow::Below
    };
    egui::popup::popup_above_or_below_widget(
        ui,
        popup_id,
        &button,
        above_or_below,
        egui::PopupCloseBehavior::CloseOnClick,
        |ui| {
            ui.set_min_width(width);
            let max_h = (ui.ctx().screen_rect().height() * 0.45).max(120.0);
            egui::ScrollArea::vertical().max_height(max_h).show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                items(ui);
            });
            ui.separator();
            if ui.button(t!("Verwalten\u{2026}")).clicked() {
                manage = true;
                ui.memory_mut(|m| m.close_popup());
            }
        },
    );
    manage
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
