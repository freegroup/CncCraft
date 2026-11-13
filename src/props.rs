//! The properties panel: editable fields for one selected shape, one block per shape class.

use crate::theme::{ink, muted};
use shape_edit::{Circle, Compound, Ellipse, Path, Polygon, Pt, Rect, Shape, Text};

fn mm(v: &mut f64) -> egui::DragValue<'_> {
    egui::DragValue::new(v).speed(0.5).max_decimals(2).suffix(" mm")
}

fn row(ui: &mut egui::Ui, label: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.label(label);
    add(ui);
    ui.end_row();
}

fn center(ui: &mut egui::Ui, c: &mut Pt) {
    row(ui, &t!("Mitte X"), |ui| {
        ui.add(mm(&mut c[0]));
    });
    row(ui, &t!("Mitte Y"), |ui| {
        ui.add(mm(&mut c[1]));
    });
}

/// A length shown as twice its value (a diameter or width for a stored radius).
fn doubled(ui: &mut egui::Ui, label: &str, half: &mut f64) {
    let mut full = *half * 2.0;
    row(ui, label, |ui| {
        if ui.add(mm(&mut full).range(0.1..=10000.0)).changed() {
            *half = full / 2.0;
        }
    });
}

fn rect(ui: &mut egui::Ui, r: &mut Rect) {
    center(ui, &mut r.center);
    row(ui, &t!("Breite"), |ui| {
        ui.add(mm(&mut r.width).range(0.1..=10000.0));
    });
    row(ui, &t!("Höhe"), |ui| {
        ui.add(mm(&mut r.height).range(0.1..=10000.0));
    });
    row(ui, &t!("Eckradius"), |ui| {
        ui.add(mm(&mut r.corner_radius).range(0.0..=1000.0));
    });
}

fn circle(ui: &mut egui::Ui, c: &mut Circle) {
    center(ui, &mut c.center);
    doubled(ui, &t!("Durchmesser"), &mut c.radius);
}

fn ellipse(ui: &mut egui::Ui, e: &mut Ellipse) {
    center(ui, &mut e.center);
    doubled(ui, &t!("Breite"), &mut e.rx);
    doubled(ui, &t!("Höhe"), &mut e.ry);
}

fn polygon(ui: &mut egui::Ui, p: &mut Polygon) {
    center(ui, &mut p.center);
    row(ui, &t!("Radius"), |ui| {
        ui.add(mm(&mut p.radius).range(0.1..=10000.0));
    });
    row(ui, &t!("Ecken"), |ui| {
        // at least 3 sides (0x10084a4cc checks the 'S' field against 3.0)
        ui.add(egui::DragValue::new(&mut p.sides).range(3..=1000));
    });
}

fn text(ui: &mut egui::Ui, t: &mut Text) {
    center(ui, &mut t.center);
    row(ui, &t!("Text"), |ui| {
        ui.add(egui::TextEdit::multiline(&mut t.text).desired_rows(2).desired_width(150.0));
    });
    row(ui, &t!("Höhe"), |ui| {
        ui.add(mm(&mut t.height).range(1.0..=2000.0));
    });
}

fn path(ui: &mut egui::Ui, p: &mut Path) {
    row(ui, &t!("Punkte"), |ui| {
        ui.label(p.points.len().to_string());
    });
    row(ui, &t!("Geschlossen"), |ui| {
        ui.checkbox(&mut p.closed, "");
    });
}

fn compound(ui: &mut egui::Ui, c: &Compound) {
    row(ui, &t!("Konturen"), |ui| {
        ui.label(c.paths.len().to_string());
    });
    row(ui, &t!("Punkte"), |ui| {
        ui.label(c.paths.iter().map(|p| p.points.len()).sum::<usize>().to_string());
    });
}

/// Curves have no stored centre: edit the middle of their bounds and move them with it.
fn curves_center(ui: &mut egui::Ui, shape: &mut Shape) {
    let before = shape.center();
    let mut c = before;
    center(ui, &mut c);
    if c != before {
        *shape = shape.translated(c[0] - before[0], c[1] - before[1]);
    }
}

/// Fields for `shape`. Returns whether anything changed.
pub fn edit(ui: &mut egui::Ui, shape: &mut Shape) -> bool {
    let before = shape.clone();
    ui.label(egui::RichText::new(t!(shape.label())).strong().color(ink()));
    egui::Grid::new("props").num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
        if matches!(shape, Shape::Path(_) | Shape::Compound(_)) {
            curves_center(ui, shape);
        }
        match shape {
            Shape::Rect(r) => rect(ui, r),
            Shape::Circle(c) => circle(ui, c),
            Shape::Ellipse(e) => ellipse(ui, e),
            Shape::Polygon(p) => polygon(ui, p),
            Shape::Text(t) => text(ui, t),
            Shape::Path(p) => path(ui, p),
            Shape::Compound(c) => compound(ui, c),
        }
    });
    if let Some((lo, hi)) = shape.bounds() {
        ui.label(egui::RichText::new(t!("Größe %{w} × %{h} mm", w = format!("{:.1}", hi[0] - lo[0]), h = format!("{:.1}", hi[1] - lo[1]))).small().color(muted()));
    }
    *shape != before
}
