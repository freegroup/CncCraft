//! Exporting the drawing to other programs' files (see `crate::export`): the window with the
//! formats on the left and the chosen format's settings on the right.

use crate::app::{dialog_buttons, dialog_group, App, ACCENT, INK};
use crate::export::{dxf, svg, Drawing, Piece, Unit, FORMATS};
use crate::theme::muted;
use egui::{Align2, RichText, Vec2};

/// Which shapes go into the file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app) enum ExportScope {
    All,
    Selected,
}

impl App {
    /// The drawing as the export sees it: the shapes on shown layers (or only the selected ones),
    /// layer by layer in the layers' order.
    fn export_drawing(&self) -> Drawing {
        let mut pieces = Vec::new();
        for layer in &self.layers.list {
            if !layer.visible {
                continue;
            }
            for item in self.shapes.iter().filter(|s| s.layer == layer.id) {
                if self.export_scope == ExportScope::Selected && !self.edit.selection.contains(&item.id) {
                    continue;
                }
                pieces.push(Piece { shape: item.shape.clone(), layer: layer.name.clone(), groups: item.groups.iter().rev().copied().collect() });
            }
        }
        Drawing { pieces, layers: self.layers.list.iter().filter(|l| l.visible).map(|l| l.name.clone()).collect(), plate: [self.job.width, self.job.height] }
    }

    /// Write the drawing in the chosen format to a file the person picks.
    fn export_drawing_to_file(&mut self) {
        let drawing = self.export_drawing();
        if drawing.pieces.is_empty() {
            self.status = t!("Keine Formen zum Exportieren").into();
            return;
        }
        // the self-test must never open a native file window: nobody would close it
        if self.selftest.is_some() {
            return;
        }
        let format = &FORMATS[self.export_format.min(FORMATS.len() - 1)];
        let Some(path) = rfd::FileDialog::new()
            .add_filter(t!(format.description), &[format.extension])
            .set_file_name(format!("{}.{}", self.export_stem(), format.extension))
            .save_file()
        else {
            return;
        };
        let text = (format.write)(&drawing, &self.export_settings);
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        self.status = match std::fs::write(&path, text) {
            Ok(()) => t!("%{format} gespeichert: %{name} (%{n} Formen)", format = (format.title)(&self.export_settings), name = name, n = drawing.pieces.len()).into(),
            Err(e) => t!("Export fehlgeschlagen: %{e}", e = e.to_string()).into(),
        };
    }

    /// The export window, while it is open.
    pub(in crate::app) fn export_file_ui(&mut self, ctx: &egui::Context) {
        if !self.export_file {
            return;
        }
        let (mut go, mut cancel) = (false, false);
        let count = self.export_drawing().pieces.len();
        super::window("export_file_win", ctx)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(620.0);
                if super::header(ui, &t!("Exportieren")) {
                    cancel = true;
                }
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(160.0);
                        dialog_group(ui, &t!("Dateiformat"), |ui| {
                            ui.set_min_height(300.0);
                            for (i, format) in FORMATS.iter().enumerate() {
                                if format_entry(ui, format, self.export_format == i).clicked() {
                                    self.export_format = i;
                                }
                            }
                        });
                    });
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width());
                        let content = Content { scope: &mut self.export_scope, any_selected: !self.edit.selection.is_empty(), count };
                        let settings = &mut self.export_settings;
                        match FORMATS[self.export_format.min(FORMATS.len() - 1)].name {
                            "DXF" => dxf_settings(ui, &mut settings.dxf, content),
                            _ => svg_settings(ui, &mut settings.svg, content),
                        }
                    });
                });
                dialog_buttons(ui, |ui| {
                    let export = egui::Button::new(RichText::new(t!("Exportieren…")).strong().color(INK)).fill(ACCENT).min_size(Vec2::new(120.0, 30.0)).rounding(4.0);
                    if ui.add_enabled(count > 0, export).clicked() {
                        go = true;
                    }
                    if ui.add(egui::Button::new(t!("Abbrechen")).min_size(Vec2::new(90.0, 30.0)).rounding(4.0)).clicked() {
                        cancel = true;
                    }
                });
            });
        if go {
            self.export_file = false;
            self.export_drawing_to_file();
        } else if cancel || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.export_file = false;
        }
    }

    /// Open the export window; "only selected" when something is selected.
    pub(in crate::app) fn open_export_file(&mut self) {
        self.export_scope = if self.edit.selection.is_empty() { ExportScope::All } else { ExportScope::Selected };
        self.export_file = true;
    }
}

/// One format in the list: its name, and under it what it is; the chosen one is marked.
fn format_entry(ui: &mut egui::Ui, format: &crate::export::Format, chosen: bool) -> egui::Response {
    let (rect, r) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 44.0), egui::Sense::click());
    let visuals = ui.style().visuals.clone();
    if chosen {
        ui.painter().rect(rect, 4.0, visuals.selection.bg_fill, visuals.selection.stroke);
    } else if r.hovered() {
        ui.painter().rect_filled(rect, 4.0, visuals.widgets.hovered.weak_bg_fill);
    }
    let text = if chosen { visuals.selection.stroke.color } else { crate::theme::ink() };
    let left = rect.left() + 10.0;
    ui.painter().text(egui::pos2(left, rect.top() + 7.0), Align2::LEFT_TOP, format.name, egui::FontId::proportional(14.5), text);
    ui.painter().text(egui::pos2(left, rect.bottom() - 7.0), Align2::LEFT_BOTTOM, t!(format.description), egui::FontId::proportional(11.5), muted());
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(11.5).color(muted()));
}

/// The rows of a group: a label on the left, its field on the right, all fields in one column.
fn rows(ui: &mut egui::Ui, id: &str, add: impl FnOnce(&mut egui::Ui)) {
    egui::Grid::new(id).num_columns(2).spacing([14.0, 8.0]).min_col_width(90.0).show(ui, add);
}

/// A drop-down list of the values `all`, each shown by `name`.
fn choice<T: Copy + PartialEq>(ui: &mut egui::Ui, id: &str, value: &mut T, all: &[T], name: fn(T) -> &'static str) {
    egui::ComboBox::from_id_salt(id).width(220.0).selected_text(t!(name(*value))).show_ui(ui, |ui| {
        for &v in all {
            ui.selectable_value(value, v, t!(name(v)));
        }
    });
}

/// What every format's "Inhalt" group needs to know and may change besides the format's own settings.
struct Content<'a> {
    scope: &'a mut ExportScope,
    any_selected: bool,
    count: usize,
}

fn scope_name(scope: ExportScope) -> &'static str {
    match scope {
        ExportScope::All => "Alle Formen auf sichtbaren Ebenen",
        ExportScope::Selected => "Nur ausgewählte Formen",
    }
}

/// The settings every format shows, in the same groups and order:
/// "Format" (the file's own settings: `format`), "Inhalt" (which shapes, the workpiece) and
/// "Struktur" (layers and groups; `groups` is `None` where the format has none).
fn groups(ui: &mut egui::Ui, format: impl FnOnce(&mut egui::Ui), content: Content, frame: &mut bool, layers: &mut bool, groups: Option<&mut bool>) {
    dialog_group(ui, &t!("Format"), format);
    dialog_group(ui, &t!("Inhalt"), |ui| {
        rows(ui, "export_content", |ui| {
            ui.label(t!("Umfang"));
            let all: &[ExportScope] = if content.any_selected { &[ExportScope::All, ExportScope::Selected] } else { &[ExportScope::All] };
            choice(ui, "export_scope", content.scope, all, scope_name);
            ui.end_row();
        });
        ui.add_space(4.0);
        ui.checkbox(frame, t!("Werkstück als Rechteck"));
        hint(ui, &if content.count == 1 { t!("1 Form") } else { t!("%{n} Formen", n = content.count) });
    });
    dialog_group(ui, &t!("Struktur"), |ui| {
        ui.checkbox(layers, t!("Ebenen übernehmen"));
        match groups {
            Some(g) => ui.checkbox(g, t!("Gruppen übernehmen")),
            None => ui.add_enabled(false, egui::Checkbox::new(&mut false, t!("Gruppen übernehmen"))).on_disabled_hover_text(t!("Dieses Format kennt keine Gruppen.")),
        };
    });
}

fn dxf_settings(ui: &mut egui::Ui, o: &mut dxf::Options, content: Content) {
    let (version, unit) = (&mut o.version, &mut o.unit);
    let format = |ui: &mut egui::Ui| {
        rows(ui, "export_dxf_format", |ui| {
            ui.label(t!("Version"));
            choice(ui, "export_dxf_version", version, &dxf::Version::ALL, dxf::Version::name);
            ui.end_row();
            ui.label(t!("Einheit"));
            choice(ui, "export_dxf_unit", unit, &Unit::ALL, Unit::name);
            ui.end_row();
        });
        ui.add_space(4.0);
        hint(ui, &t!(version.hint()));
        if *version == dxf::Version::R12 {
            hint(ui, &t!("R12 speichert keine Einheit: das lesende Programm muss sie kennen."));
        }
    };
    groups(ui, format, content, &mut o.frame, &mut o.layers, None);
}

fn svg_settings(ui: &mut egui::Ui, o: &mut svg::Options, content: Content) {
    let (unit, area, stroke) = (&mut o.unit, &mut o.area, &mut o.stroke);
    let format = |ui: &mut egui::Ui| {
        rows(ui, "export_svg_format", |ui| {
            ui.label(t!("Version"));
            ui.label("SVG 1.1");
            ui.end_row();
            ui.label(t!("Einheit"));
            choice(ui, "export_svg_unit", unit, &Unit::ALL, Unit::name);
            ui.end_row();
            ui.label(t!("Bildfläche"));
            choice(ui, "export_svg_area", area, &svg::Area::ALL, svg::Area::name);
            ui.end_row();
            ui.label(t!("Linienstärke"));
            ui.add_sized([220.0, 26.0], egui::DragValue::new(stroke).range(0.01..=5.0).speed(0.01).suffix(" mm"));
            ui.end_row();
        });
        ui.add_space(4.0);
        hint(ui, &t!("Eine Einheit im Bild ist ein Millimeter oder ein Zoll."));
    };
    groups(ui, format, content, &mut o.frame, &mut o.layers, Some(&mut o.groups));
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_format_writes_something() {
        let drawing = crate::export::Drawing { pieces: vec![], layers: vec![], plate: [100.0, 50.0] };
        for f in crate::export::FORMATS {
            assert!(!(f.write)(&drawing, &crate::export::Settings::default()).is_empty(), "{}", f.name);
        }
    }
}
