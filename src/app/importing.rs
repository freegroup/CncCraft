//! Bringing drawings in from files of other programs (see `crate::import`), and reading them from
//! disk again when the file changed. Every imported shape remembers its file (`Item::source`).

use super::{App, ViewMode};
use crate::import;
use crate::theme::{ink, muted};
use egui::{Align2, Vec2};
use shape_edit::{Shape, Source};
use std::path::{Path, PathBuf};

fn file_dialog() -> rfd::FileDialog {
    let mut dialog = rfd::FileDialog::new();
    for format in import::FORMATS {
        dialog = dialog.add_filter(t!(format.description), format.extensions);
    }
    dialog
}

/// What the person should hear about an import: only what was converted or left out.
fn note(format: &str, imp: &import::Import) -> Option<String> {
    let (unit, skipped) = (imp.unit, &imp.skipped);
    let format = if imp.version.is_empty() { format.to_string() } else { format!("{format} {}", imp.version) };
    let mut parts = Vec::new();
    if unit != "mm" {
        parts.push(format!("Einheit {unit}, in mm umgerechnet"));
    }
    if !skipped.is_empty() {
        let list: Vec<String> = skipped.iter().map(|(kind, n)| format!("{n}× {kind}")).collect();
        parts.push(format!("nicht übernommen: {}", list.join(", ")));
    }
    (!parts.is_empty()).then(|| format!("{format}-Import ({})", parts.join("; ")))
}

/// Read the shapes of a file (`fresh`, as they lie in the file) back into every item that came from
/// `file`, copies too, each by its number in the file, at the offset it was imported with. The items
/// keep their ids; one whose number the file no longer has is removed, a number no item has is added.
/// `new_file` is where the file is now (it may have been looked up again).
fn reload_items(items: &mut Vec<shape_edit::Item>, file: &str, new_file: &str, format: &str, fresh: &[Shape], mut new_id: impl FnMut() -> u64) {
    let group: Vec<usize> = (0..items.len()).filter(|&i| items[i].source.as_ref().is_some_and(|s| s.file == file)).collect();
    let Some(&first) = group.first() else { return };
    let (first_offset, layer, groups) = (items[first].source.as_ref().map_or([0.0, 0.0], |s| s.offset), items[first].layer, items[first].groups.clone());
    let mut present = vec![false; fresh.len()];
    let mut gone = Vec::new();
    for &i in &group {
        let item = &mut items[i];
        let Some(old) = item.source.clone() else { continue };
        if old.index < fresh.len() {
            present[old.index] = true;
            item.shape = fresh[old.index].translated(old.offset[0], old.offset[1]);
            item.bound.clear();
            item.source = Some(Source { file: new_file.to_string(), format: format.to_string(), ..old });
        } else {
            gone.push(item.id);
        }
    }
    for index in (0..fresh.len()).filter(|&k| !present[k]) {
        let mut item = shape_edit::Item::new(new_id(), fresh[index].translated(first_offset[0], first_offset[1]));
        item.layer = layer;
        item.groups = groups.clone();
        item.source = Some(Source { file: new_file.to_string(), format: format.to_string(), index, offset: first_offset });
        items.push(item);
    }
    items.retain(|s| !gone.contains(&s.id));
}

impl App {
    /// The import button: pick a file, then bring its drawing in.
    pub(super) fn import_dialog(&mut self) {
        // the self-test must never open a native file window: nobody would close it
        if self.selftest.is_some() {
            return;
        }
        let Some(path) = file_dialog().pick_file() else { return };
        self.import_file(&path);
    }

    /// Put the shapes of a file into the drawing, selected, each remembering the file.
    pub(super) fn import_file(&mut self, path: &Path) {
        let (format, imp) = match import::read_file(path) {
            Ok(r) => r,
            Err(e) => {
                self.status = format!("Import fehlgeschlagen: {e}");
                return;
            }
        };
        if imp.shapes.is_empty() {
            self.status = note(format.name, &imp).unwrap_or_else(|| format!("In der {}-Datei wurde nichts Übernehmbares gefunden", format.name));
            return;
        }
        let offset = self.placement(&imp.shapes);
        let source = Source { file: path.to_string_lossy().into_owned(), format: format.name.into(), index: 0, offset };
        self.push_undo();
        let mut ids = Vec::new();
        for (index, shape) in imp.shapes.iter().enumerate() {
            let id = self.add_shape(shape.translated(offset[0], offset[1]));
            if let Some(item) = self.shapes.iter_mut().find(|s| s.id == id) {
                item.source = Some(Source { index, ..source.clone() });
            }
            ids.push(id);
        }
        // everything the file brought in is one group, even a single shape (a rectangle of four
        // lines comes in as one curve): it moves and scales as one, "Gruppe auflösen" takes it
        // apart, and a shape the file gains later joins it
        let group = self.new_id();
        for item in self.shapes.iter_mut().filter(|s| ids.contains(&s.id)) {
            item.groups.push(group);
        }
        self.start_page = false;
        self.mode = ViewMode::TwoD;
        self.edit.selection = ids;
        if let Some(text) = note(format.name, &imp) {
            self.status = text;
        }
    }

    /// How far to move an imported drawing: not at all, unless it lies completely off the plate,
    /// then onto its middle.
    fn placement(&self, shapes: &[Shape]) -> [f64; 2] {
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for (a, b) in shapes.iter().filter_map(|s| s.bounds()) {
            lo = [lo[0].min(a[0]), lo[1].min(a[1])];
            hi = [hi[0].max(b[0]), hi[1].max(b[1])];
        }
        if lo[0] > hi[0] {
            return [0.0, 0.0];
        }
        let (w, h) = (self.job.width, self.job.height);
        if hi[0] < 0.0 || lo[0] > w || hi[1] < 0.0 || lo[1] > h {
            [w / 2.0 - (lo[0] + hi[0]) / 2.0, h / 2.0 - (lo[1] + hi[1]) / 2.0]
        } else {
            [0.0, 0.0]
        }
    }

    /// Read the file `file` of an import once more (or, after "Datei suchen", the file `from`) and
    /// read its shapes back into every shape that came from it (copies too), each by its number
    /// in the file. The shapes keep their ids, so toolpaths cutting them stay attached; a shape
    /// the file no longer has is removed, one the drawing lacks is added.
    pub(super) fn reload_import(&mut self, file: &str, from: Option<PathBuf>) {
        let path = from.unwrap_or_else(|| PathBuf::from(file));
        if !path.exists() {
            self.import_missing = Some(file.to_string());
            return;
        }
        let (format, imp) = match import::read_file(&path) {
            Ok(r) => r,
            Err(e) => {
                self.status = format!("Neu laden fehlgeschlagen: {e}");
                return;
            }
        };
        if !self.shapes.iter().any(|s| s.source.as_ref().is_some_and(|src| src.file == file)) {
            return;
        }
        if imp.shapes.is_empty() {
            self.status = format!("Die {}-Datei enthält nichts Übernehmbares mehr, die Zeichnung blieb unverändert", format.name);
            return;
        }
        self.push_undo();
        self.import_missing = None;
        let mut shapes = std::mem::take(&mut self.shapes);
        reload_items(&mut shapes, file, &path.to_string_lossy(), format.name, &imp.shapes, || self.new_id());
        self.shapes = shapes;
        self.edit.retain_existing(&self.shapes);
        self.recompute_all();
        self.sim = None;
        if let Some(text) = note(format.name, &imp) {
            self.status = text;
        }
    }

    /// The friendly window shown when the original file of an import is not there any more.
    pub(super) fn import_missing_ui(&mut self, ctx: &egui::Context) {
        let Some(file) = self.import_missing.clone() else { return };
        let name = Path::new(&file).file_name().map_or_else(|| file.clone(), |n| n.to_string_lossy().into_owned());
        let (mut search, mut close) = (false, false);
        egui::Window::new(t!("Datei nicht gefunden")).id(egui::Id::new("Datei nicht gefunden")).collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
            ui.set_width(380.0);
            ui.label(egui::RichText::new(t!("Die Originaldatei „%{name}“ wurde leider nicht gefunden.", name = name)).strong().color(ink()));
            ui.add_space(4.0);
            ui.label(t!("Sie wurde vielleicht verschoben, umbenannt oder gelöscht. Die importierten Formen bleiben unverändert in der Zeichnung."));
            ui.add_space(4.0);
            ui.label(egui::RichText::new(&file).small().color(muted()));
            super::dialog_buttons(ui, |ui| {
                if ui.button(t!("Datei suchen…")).clicked() {
                    search = true;
                }
                if ui.button(t!("Schließen")).clicked() {
                    close = true;
                }
            });
        });
        if close {
            self.import_missing = None;
        }
        if search {
            if let Some(found) = file_dialog().pick_file() {
                self.reload_import(&file, Some(found));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shape_edit::{Circle, Item};

    fn imported(id: u64, index: usize, r: f64, file: &str) -> Item {
        let mut item = Item::new(id, Circle::new([0.0, 0.0], r));
        item.source = Some(Source { file: file.into(), format: "DXF".into(), index, offset: [100.0, 0.0] });
        item
    }

    fn fresh(radii: &[f64]) -> Vec<Shape> {
        radii.iter().map(|&r| Circle::new([1.0, 2.0], r).into()).collect()
    }

    fn radius(item: &Item) -> f64 {
        let Shape::Circle(c) = &item.shape else { panic!("a circle") };
        c.radius
    }

    #[test]
    fn a_reload_reads_every_shape_of_the_file_back_by_its_number() {
        let mut items = vec![imported(1, 0, 1.0, "/a.dxf"), imported(2, 1, 2.0, "/a.dxf"), Item::new(3, Circle::new([0.0, 0.0], 9.0)), imported(4, 0, 1.0, "/b.dxf")];
        reload_items(&mut items, "/a.dxf", "/a.dxf", "DXF", &fresh(&[10.0, 20.0]), || unreachable!());
        assert_eq!(items.iter().map(|i| i.id).collect::<Vec<_>>(), vec![1, 2, 3, 4], "ids stay");
        assert_eq!((radius(&items[0]), radius(&items[1])), (10.0, 20.0));
        assert_eq!(radius(&items[2]), 9.0, "a shape of the person's own is left alone");
        assert_eq!(radius(&items[3]), 1.0, "so is one from another file");
        let Shape::Circle(c) = &items[0].shape else { panic!() };
        assert_eq!(c.center, [101.0, 2.0], "put at the offset of the import");
    }

    #[test]
    fn a_copy_is_read_back_too() {
        let mut items = vec![imported(1, 0, 1.0, "/a.dxf"), imported(7, 0, 1.0, "/a.dxf")];
        reload_items(&mut items, "/a.dxf", "/a.dxf", "DXF", &fresh(&[5.0]), || unreachable!());
        assert_eq!(items.len(), 2, "the copy is not taken for a leftover");
        assert!(items.iter().all(|i| radius(i) == 5.0));
    }

    #[test]
    fn shapes_the_file_lost_go_and_new_ones_come() {
        let mut items = vec![imported(1, 0, 1.0, "/a.dxf"), imported(2, 1, 2.0, "/a.dxf")];
        reload_items(&mut items, "/a.dxf", "/a.dxf", "DXF", &fresh(&[5.0]), || unreachable!());
        assert_eq!(items.iter().map(|i| i.id).collect::<Vec<_>>(), vec![1], "number 1 is gone from the file");

        items[0].groups = vec![5];
        let mut next = 10;
        reload_items(&mut items, "/a.dxf", "/a.dxf", "DXF", &fresh(&[5.0, 6.0, 7.0]), || {
            next += 1;
            next
        });
        assert_eq!(items.iter().map(|i| i.id).collect::<Vec<_>>(), vec![1, 11, 12]);
        assert_eq!(items[2].source.as_ref().map(|s| (s.index, s.offset)), Some((2, [100.0, 0.0])));
        assert_eq!(items[2].groups, vec![5], "a shape the file gained joins the group of the import");
    }

    #[test]
    fn a_file_found_elsewhere_becomes_the_new_source() {
        let mut items = vec![imported(1, 0, 1.0, "/alt.dxf")];
        reload_items(&mut items, "/alt.dxf", "/neu/ort.dxf", "DXF", &fresh(&[3.0]), || unreachable!());
        assert_eq!(items[0].source.as_ref().map(|s| s.file.as_str()), Some("/neu/ort.dxf"));
    }
}
