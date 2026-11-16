//! The tool column on the left, grouped like the original's, with the form a tool can open beside it.

use crate::app::{icon_button_tip, App, DrawTool, Icon, Tip, ViewMode};
use crate::theme::{ink, muted};
use egui::{Stroke, Vec2};
use shape_edit::Flip;

/// A tool that opens a form beside the tool column.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::app) enum ToolForm {
    /// Move and size the selection by numbers (the selected shape's properties).
    Transform,
    /// Offset (expand or shrink) the selected vectors.
    Offset,
    /// The radius of the corner tools (rounded corners and ears); shown while one of them is on.
    Corner,
}

/// The tool column's groups, top to bottom, as the original registers them (0x10011d034):
/// `mode` (select, transform, node edit), `draw_line`, `draw_shape` (rectangle, circle,
/// ellipse, polygon), `draw_text`, `edit_object`, `align` (align, mirror), `offset` and
/// `binary_operation` and `group_ungroup`; the job setup sits apart at the end. The original's
/// `bitmap`, `join` and `create_fillets` groups have no tools here yet, so they are left out;
/// duplicating is no tool there either (menu and ⌘D).
const GROUPS: [&[Icon]; GROUP_COUNT] = [
    &[Icon::Draw(DrawTool::Select), Icon::Transform, Icon::Draw(DrawTool::NodeEdit)],
    &[Icon::Draw(DrawTool::Polyline), Icon::Draw(DrawTool::Spline)],
    &[Icon::Draw(DrawTool::Rect), Icon::Draw(DrawTool::Circle), Icon::Draw(DrawTool::Ellipse), Icon::Draw(DrawTool::Polygon), Icon::ImportDxf],
    &[Icon::Draw(DrawTool::Text)],
    &[Icon::Delete, Icon::Curves],
    &[Icon::Center, Icon::MirrorH, Icon::MirrorV],
    &[Icon::Offset],
    &[Icon::Draw(DrawTool::Trim), Icon::Draw(DrawTool::Erase), Icon::Draw(DrawTool::Connect), Icon::Draw(DrawTool::FilletOuter), Icon::Draw(DrawTool::FilletInner)],
    &[Icon::Union, Icon::Subtract, Icon::Intersect],
    &[Icon::Group, Icon::Ungroup],
];

/// How many groups the tool column has (one remembered choice each when folded).
pub(in crate::app) const GROUP_COUNT: usize = 10;

/// What each group is, for the folded column's tooltips.
const GROUP_NAMES: [crate::lang::Tr; GROUP_COUNT] = [tr!("Modus"), tr!("Linien zeichnen"), tr!("Formen zeichnen"), tr!("Text"), tr!("Objekte bearbeiten"), tr!("Ausrichten und Spiegeln"), tr!("Versatz"), tr!("Vektoroperationen"), tr!("Boolesche Operationen"), tr!("Gruppieren")];

/// Icons per row in the expanded column, and the size of one icon cell.
const ROW: usize = 5;

const CELL: f32 = 38.0;

pub(super) const GAP: f32 = 4.0;

/// The command key as the person's keyboard writes it.
fn cmd(key: &str) -> String {
    if cfg!(target_os = "macos") { format!("⌘{key}") } else { format!("Strg+{key}") }
}

/// The tip of a tool column icon: what it is called (with its key), what it does in one or two
/// short sentences, and, while it can't be used, what it needs first.
pub(in crate::app) fn tool_tip(icon: Icon) -> Tip {
    let pick_one = t!("Erst eine oder mehrere Formen auswählen.");
    let pick_two = t!("Erst mindestens zwei Formen auswählen.");
    let tip = match icon {
        Icon::Draw(DrawTool::Select) => Tip::new(t!("Auswählen"), t!("Formen anklicken oder mit einem Rahmen wählen und verschieben. Mit Umschalt kommen weitere dazu.")).shortcut("S"),
        Icon::Draw(DrawTool::NodeEdit) => Tip::new(t!("Knoten bearbeiten"), t!("Die Punkte eines Vektors ziehen. Doppelklick auf eine Linie fügt einen Punkt ein, Entf löscht ihn.")).shortcut("N"),
        Icon::Draw(DrawTool::Polyline) => Tip::new(t!("Linie"), t!("Gerade Linien Punkt für Punkt. Doppelklick oder Enter beendet, Tab schließt den Linienzug.")).shortcut("L"),
        Icon::Draw(DrawTool::Spline) => Tip::new(t!("Spline"), t!("Eine weiche Kurve durch die geklickten Punkte. Doppelklick oder Enter beendet, ein Klick auf den Anfang schließt sie.")).shortcut("P"),
        Icon::Draw(DrawTool::Rect) => Tip::new(t!("Rechteck"), t!("Von Ecke zu Ecke aufziehen. Breite und Höhe lassen sich danach genau eintippen.")).shortcut("R"),
        Icon::Draw(DrawTool::Circle) => Tip::new(t!("Kreis"), t!("Vom Mittelpunkt aus aufziehen. Den Durchmesser danach genau eintippen.")).shortcut("C"),
        Icon::Draw(DrawTool::Ellipse) => Tip::new(t!("Ellipse"), t!("Von Ecke zu Ecke aufziehen. Breite und Höhe danach genau eintippen.")),
        Icon::Draw(DrawTool::Polygon) => Tip::new(t!("Polygon"), t!("Ein regelmäßiges Vieleck vom Mittelpunkt aus aufziehen. Die Zahl der Ecken unter „Eigenschaften“ ändern.")),
        Icon::Draw(DrawTool::Text) => Tip::new(t!("Text"), t!("Ein Klick setzt einen Text; den Inhalt unter „Eigenschaften“ ändern. Die Buchstaben lassen sich fräsen oder gravieren.")),
        Icon::Draw(DrawTool::Trim) => Tip::new(t!("Trennen"), t!("Einen Vektor anklicken: Er wird an jeder Kreuzung mit anderen Vektoren geteilt.")).shortcut("T"),
        Icon::Draw(DrawTool::Erase) => Tip::new(t!("Vektor löschen"), t!("Einen Vektor oder ein getrenntes Stück anklicken: Es wird gelöscht.")),
        Icon::Draw(DrawTool::Connect) => Tip::new(t!("Verbinden"), t!("Zwei Punkte nacheinander anklicken: Sie werden mit einer Linie verbunden. Beide Enden eines Linienzugs schließen ihn.")),
        Icon::Draw(DrawTool::FilletOuter) => Tip::new(t!("Ohr"), t!("Eine Ecke anklicken: Sie bekommt einen Kreis um den Eckpunkt, damit ein runder Fräser ganz in die Ecke kommt. Radius im Formular links.")),
        Icon::Draw(DrawTool::FilletInner) => Tip::new(t!("Abrunden"), t!("Eine Ecke anklicken: Sie wird mit dem Radius aus dem Formular links abgerundet. Der Radius bleibt jederzeit änderbar.")),
        Icon::Transform => Tip::new(t!("Transformieren"), t!("Lage und Größe der Auswahl genau eintippen.")),
        Icon::Offset => Tip::new(t!("Versetzen"), t!("Eine neue Kontur im festen Abstand außen oder innen um die Auswahl.")).shortcut("O"),
        Icon::MirrorH => Tip::new(t!("Waagrecht spiegeln"), t!("Spiegelt die Auswahl an ihrer senkrechten Mitte (links wird rechts).")).when_disabled(pick_one),
        Icon::MirrorV => Tip::new(t!("Senkrecht spiegeln"), t!("Spiegelt die Auswahl an ihrer waagrechten Mitte (oben wird unten).")).when_disabled(pick_one),
        Icon::Center => Tip::new(t!("Auf dem Material zentrieren"), t!("Schiebt die Auswahl in die Mitte der Platte.")).when_disabled(pick_one),
        Icon::Curves => Tip::new(t!("In Kurven umwandeln"), t!("Macht aus Rechteck, Kreis, Text und Co. frei bearbeitbare Vektoren. Ihre Maße lassen sich danach nicht mehr eintippen.")).when_disabled(pick_one),
        Icon::Delete => Tip::new(t!("Löschen"), t!("Entfernt die ausgewählten Formen.")).shortcut("Entf").when_disabled(pick_one),
        Icon::Union => Tip::new(t!("Vereinen"), t!("Macht aus sich überlappenden Formen eine gemeinsame Umrissform.")).when_disabled(pick_two),
        Icon::Subtract => Tip::new(t!("Abziehen"), t!("Die zuerst gewählte Form minus alle anderen.")).when_disabled(pick_two),
        Icon::Intersect => Tip::new(t!("Schnittmenge"), t!("Behält nur die Fläche, die alle gewählten Formen gemeinsam haben.")).when_disabled(pick_two),
        Icon::Group => Tip::new(t!("Gruppieren"), t!("Die Auswahl wird eine Gruppe: Ein Klick wählt dann alles. Gruppen lassen sich wieder gruppieren.")).shortcut(cmd("G")).when_disabled(pick_two),
        Icon::Ungroup => Tip::new(t!("Gruppe auflösen"), t!("Löst die äußerste Gruppe der Auswahl auf, eine Ebene je Klick.")).shortcut(cmd("⇧G")).when_disabled(t!("Erst eine Gruppe auswählen.")),
        Icon::ImportDxf => Tip::new(t!("DXF importieren"), t!("Holt eine Zeichnung aus einer DXF-Datei (R12, R14 und neuer) als eine Gruppe. Rechtsklick auf die Form lädt sie neu von der Platte.")),
        Icon::Job => Tip::new(t!("Job einrichten"), t!("Größe und Dicke der Platte, Material und wo der Nullpunkt liegt.")),
        Icon::Variables => Tip::new(t!("Variablen"), t!("Benannte Werte wie „breite“, die Maße, Tiefen und Abstände verwenden. Ändert man den Wert, folgt alles.")),
        _ => Tip::new(String::new(), String::new()),
    };
    tip.icon(icon)
}

impl App {
    /// The tool column. Expanded, it shows every tool, grouped in rows of up to five. Folded, it
    /// is one narrow column with one button per group, showing the group's current tool; a
    /// group of several tools opens them to the right, so the folded column never scrolls.
    pub(in crate::app) fn tool_column(&mut self, ctx: &egui::Context) {
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
    pub(in crate::app) fn icon_state(&self, icon: Icon) -> (bool, bool) {
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
        let resp = ui.add_enabled_ui(enabled, |ui| icon_button_tip(ui, icon, selected, tool_tip(icon))).inner;
        if icon == Icon::Union {
            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("cnccraft_union_button"), resp.rect));
        }
        resp
    }

    pub(in crate::app) fn column_click(&mut self, icon: Icon) {
        self.flyout = None;
        match icon {
            Icon::Draw(t) => {
                self.tool = t;
                self.poly.clear();
                self.vops.connect_from = None;
                self.vops.picked = None;
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
            Icon::Union => self.join_selection(),
            Icon::Subtract => self.subtract_selection(),
            Icon::Intersect => self.intersect_selection(),
            Icon::Group => self.group_selection(),
            Icon::Ungroup => self.ungroup_selection(),
            Icon::Job => self.job_dialog = Some((self.job.clone(), false)),
            Icon::ImportDxf => self.import_dialog(),
            Icon::Variables => self.vars_open = !self.vars_open,
            _ => {}
        }
    }

    /// The open tool's form, beside the tool column.
    pub(in crate::app) fn tool_form_panel(&mut self, ctx: &egui::Context) {
        let Some(form) = self.tool_form.filter(|f| *f != ToolForm::Corner) else { return };
        egui::SidePanel::left("tool_form").exact_width(250.0).resizable(false).frame(egui::Frame::none().fill(crate::theme::panel()).inner_margin(10.0).stroke(Stroke::new(1.0_f32, crate::theme::separator()))).show(ctx, |ui| {
            ui.horizontal(|ui| {
                let title = match form {
                    ToolForm::Transform => t!("Objekte transformieren"),
                    ToolForm::Offset => t!("Versatz"),
                    ToolForm::Corner => t!("Abrunden"),
                };
                ui.label(egui::RichText::new(title).size(15.0).color(ink()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if crate::app::dialogs::close_button(ui).on_hover_text(t!("Schließen")).clicked() {
                        self.tool_form = None;
                    }
                });
            });
            ui.separator();
            match form {
                ToolForm::Corner => self.corner_form(ui),
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
}
