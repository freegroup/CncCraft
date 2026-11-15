//! Writing DXF drawings (ASCII): AutoCAD R12 with POLYLINE entities, or R14 with LWPOLYLINE and
//! the tables, blocks and objects a newer reader expects. Circles stay true circles.

use super::{outlines, Drawing, Outline, Unit};
use std::fmt::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Version {
    R12,
    R14,
}

impl Version {
    pub const ALL: [Version; 2] = [Version::R12, Version::R14];

    pub fn name(self) -> &'static str {
        match self {
            Version::R12 => "R12",
            Version::R14 => "R14",
        }
    }

    /// What the version is good for, shown under the choice.
    pub fn hint(self) -> &'static str {
        match self {
            Version::R12 => "Am weitesten verbreitet; liest fast jedes Programm.",
            Version::R14 => "Kompaktere Polylinien und Einheit in der Datei; für neuere Programme.",
        }
    }

    fn code(self) -> &'static str {
        match self {
            Version::R12 => "AC1009",
            Version::R14 => "AC1014",
        }
    }
}

/// The `$INSUNITS` code of a unit.
fn insunits(unit: Unit) -> i32 {
    match unit {
        Unit::Mm => 4,
        Unit::Inch => 1,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Options {
    pub version: Version,
    pub unit: Unit,
    /// Each shape on a DXF layer named like its layer, else all on layer "0".
    pub layers: bool,
    /// The workpiece's outline as a closed rectangle (on its own layer when layers are kept).
    pub frame: bool,
}

/// The layer the workpiece's outline goes on.
const FRAME_LAYER: &str = "Werkstück";

impl Default for Options {
    fn default() -> Self {
        Options { version: Version::R12, unit: Unit::Mm, layers: true, frame: false }
    }
}

/// A layer name DXF accepts: the characters it forbids become "_".
fn layer_name(name: &str) -> String {
    let n: String = name.chars().map(|c| if "<>/\\\":;?*|=`".contains(c) || c.is_control() { '_' } else { c }).collect();
    if n.trim().is_empty() {
        "0".into()
    } else {
        n
    }
}

fn num(v: f64) -> String {
    let s = format!("{:.6}", v);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".into()
    } else {
        s.to_string()
    }
}

/// The file being written: group codes and values, and the next free handle (R14).
struct Out {
    text: String,
    handle: u32,
}

impl Out {
    fn pair(&mut self, code: i32, value: impl std::fmt::Display) {
        let _ = write!(self.text, "{code:>3}\n{value}\n");
    }

    fn handle(&mut self) -> String {
        self.handle += 1;
        format!("{:X}", self.handle)
    }

    /// Handle (5) and owner (330) of an R14 object or table record.
    fn owned(&mut self, owner: &str) -> String {
        let h = self.handle();
        self.pair(5, &h);
        self.pair(330, owner);
        h
    }
}

pub fn write(drawing: &Drawing, o: &Options) -> String {
    let k = o.unit.scale();
    let r14 = o.version == Version::R14;
    let mut out = Out { text: String::new(), handle: 0x20 };
    let mut layers: Vec<String> = if o.layers { drawing.layers.iter().map(|l| layer_name(l)).collect() } else { vec![] };
    if o.frame && o.layers && !layers.iter().any(|l| l == FRAME_LAYER) {
        layers.push(FRAME_LAYER.into());
    }
    let (mut lo, mut hi) = drawing.bounds().unwrap_or(([0.0; 2], drawing.plate));
    if o.frame {
        (lo, hi) = ([lo[0].min(0.0), lo[1].min(0.0)], [hi[0].max(drawing.plate[0]), hi[1].max(drawing.plate[1])]);
    }

    // HEADER
    out.pair(0, "SECTION");
    out.pair(2, "HEADER");
    out.pair(9, "$ACADVER");
    out.pair(1, o.version.code());
    if r14 {
        out.pair(9, "$HANDSEED");
        out.pair(5, "FFFF");
        out.pair(9, "$INSUNITS");
        out.pair(70, insunits(o.unit));
    }
    out.pair(9, "$MEASUREMENT");
    out.pair(70, i32::from(o.unit == Unit::Mm));
    for (name, p) in [("$EXTMIN", lo), ("$EXTMAX", hi)] {
        out.pair(9, name);
        out.pair(10, num(p[0] * k));
        out.pair(20, num(p[1] * k));
        out.pair(30, 0);
    }
    out.pair(0, "ENDSEC");

    let model = if r14 { tables_r14(&mut out, &layers) } else { tables_r12(&mut out, &layers) };

    out.pair(0, "SECTION");
    out.pair(2, "ENTITIES");
    if o.frame {
        let [w, h] = drawing.plate;
        let plate = Outline::Line { pts: vec![[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]], closed: true };
        entity(&mut out, &plate, if o.layers { FRAME_LAYER } else { "0" }, k, model.as_deref());
    }
    for piece in &drawing.pieces {
        let layer = if o.layers { layer_name(&piece.layer) } else { "0".into() };
        for outline in outlines(&piece.shape) {
            entity(&mut out, &outline, &layer, k, model.as_deref());
        }
    }
    out.pair(0, "ENDSEC");

    if r14 {
        out.pair(0, "SECTION");
        out.pair(2, "OBJECTS");
        out.pair(0, "DICTIONARY");
        let root = out.owned("0");
        out.pair(100, "AcDbDictionary");
        let groups = out.handle.wrapping_add(1);
        out.pair(3, "ACAD_GROUP");
        out.pair(350, format!("{groups:X}"));
        out.pair(0, "DICTIONARY");
        out.owned(&root);
        out.pair(100, "AcDbDictionary");
        out.pair(0, "ENDSEC");
    }
    out.pair(0, "EOF");
    out.text
}

fn entity(out: &mut Out, outline: &Outline, layer: &str, k: f64, model: Option<&str>) {
    let head = |out: &mut Out, kind: &str, sub: &str| {
        out.pair(0, kind);
        if let Some(model) = model {
            out.owned(model);
            out.pair(100, "AcDbEntity");
        }
        out.pair(8, layer);
        if model.is_some() {
            out.pair(100, sub);
        }
    };
    match outline {
        Outline::Circle { center, radius } => {
            head(out, "CIRCLE", "AcDbCircle");
            out.pair(10, num(center[0] * k));
            out.pair(20, num(center[1] * k));
            out.pair(30, 0);
            out.pair(40, num(radius * k));
        }
        Outline::Line { pts, closed } => {
            // a closed outline whose last point repeats the first: the flag closes it
            let mut pts = pts.as_slice();
            if *closed && pts.len() > 2 && pts.first() == pts.last() {
                pts = &pts[..pts.len() - 1];
            }
            if model.is_some() {
                head(out, "LWPOLYLINE", "AcDbPolyline");
                out.pair(90, pts.len());
                out.pair(70, i32::from(*closed));
                for p in pts {
                    out.pair(10, num(p[0] * k));
                    out.pair(20, num(p[1] * k));
                }
            } else {
                head(out, "POLYLINE", "");
                out.pair(66, 1);
                out.pair(10, 0);
                out.pair(20, 0);
                out.pair(30, 0);
                out.pair(70, i32::from(*closed));
                for p in pts {
                    out.pair(0, "VERTEX");
                    out.pair(8, layer);
                    out.pair(10, num(p[0] * k));
                    out.pair(20, num(p[1] * k));
                    out.pair(30, 0);
                }
                out.pair(0, "SEQEND");
                out.pair(8, layer);
            }
        }
    }
}

/// R12: the layer table only (everything else is optional there). No model space block.
fn tables_r12(out: &mut Out, layers: &[String]) -> Option<String> {
    out.pair(0, "SECTION");
    out.pair(2, "TABLES");
    out.pair(0, "TABLE");
    out.pair(2, "LTYPE");
    out.pair(70, 1);
    out.pair(0, "LTYPE");
    out.pair(2, "CONTINUOUS");
    out.pair(70, 0);
    out.pair(3, "Solid line");
    out.pair(72, 65);
    out.pair(73, 0);
    out.pair(40, 0);
    out.pair(0, "ENDTAB");
    out.pair(0, "TABLE");
    out.pair(2, "LAYER");
    out.pair(70, layers.len() + 1);
    for name in std::iter::once("0").chain(layers.iter().map(String::as_str).filter(|n| *n != "0")) {
        out.pair(0, "LAYER");
        out.pair(2, name);
        out.pair(70, 0);
        out.pair(62, 7);
        out.pair(6, "CONTINUOUS");
    }
    out.pair(0, "ENDTAB");
    out.pair(0, "ENDSEC");
    None
}

/// R14: all nine tables, the model and paper space blocks; returns the model space's block record
/// handle, the owner of every entity.
fn tables_r14(out: &mut Out, layers: &[String]) -> Option<String> {
    out.pair(0, "SECTION");
    out.pair(2, "CLASSES");
    out.pair(0, "ENDSEC");
    out.pair(0, "SECTION");
    out.pair(2, "TABLES");
    let table = |out: &mut Out, name: &str, count: usize| -> String {
        out.pair(0, "TABLE");
        out.pair(2, name);
        let h = out.owned("0");
        out.pair(100, "AcDbSymbolTable");
        out.pair(70, count);
        h
    };
    let record = |out: &mut Out, kind: &str, owner: &str, sub: &str, name: &str| -> String {
        out.pair(0, kind);
        let h = out.owned(owner);
        out.pair(100, "AcDbSymbolTableRecord");
        out.pair(100, sub);
        out.pair(2, name);
        out.pair(70, 0);
        h
    };
    let t = table(out, "VPORT", 1);
    record(out, "VPORT", &t, "AcDbViewportTableRecord", "*ACTIVE");
    for (code, v) in [(10, 0.0), (20, 0.0), (11, 1.0), (21, 1.0), (12, 0.0), (22, 0.0), (40, 100.0), (41, 1.5)] {
        out.pair(code, num(v));
    }
    out.pair(0, "ENDTAB");
    let t = table(out, "LTYPE", 3);
    for name in ["ByBlock", "ByLayer", "Continuous"] {
        record(out, "LTYPE", &t, "AcDbLinetypeTableRecord", name);
        out.pair(3, if name == "Continuous" { "Solid line" } else { "" });
        out.pair(72, 65);
        out.pair(73, 0);
        out.pair(40, 0);
    }
    out.pair(0, "ENDTAB");
    let names: Vec<&str> = std::iter::once("0").chain(layers.iter().map(String::as_str).filter(|n| *n != "0")).collect();
    let t = table(out, "LAYER", names.len());
    for name in names {
        record(out, "LAYER", &t, "AcDbLayerTableRecord", name);
        out.pair(62, 7);
        out.pair(6, "Continuous");
    }
    out.pair(0, "ENDTAB");
    let t = table(out, "STYLE", 1);
    record(out, "STYLE", &t, "AcDbTextStyleTableRecord", "Standard");
    for (code, v) in [(40, "0"), (41, "1"), (50, "0"), (71, "0"), (42, "2.5"), (3, "txt"), (4, "")] {
        out.pair(code, v);
    }
    out.pair(0, "ENDTAB");
    for name in ["VIEW", "UCS"] {
        table(out, name, 0);
        out.pair(0, "ENDTAB");
    }
    let t = table(out, "APPID", 1);
    record(out, "APPID", &t, "AcDbRegAppTableRecord", "ACAD");
    out.pair(0, "ENDTAB");
    table(out, "DIMSTYLE", 0);
    out.pair(0, "ENDTAB");
    let t = table(out, "BLOCK_RECORD", 2);
    let model = record(out, "BLOCK_RECORD", &t, "AcDbBlockTableRecord", "*Model_Space");
    let paper = record(out, "BLOCK_RECORD", &t, "AcDbBlockTableRecord", "*Paper_Space");
    out.pair(0, "ENDTAB");
    out.pair(0, "ENDSEC");

    out.pair(0, "SECTION");
    out.pair(2, "BLOCKS");
    for (owner, name) in [(&model, "*Model_Space"), (&paper, "*Paper_Space")] {
        out.pair(0, "BLOCK");
        out.owned(owner);
        out.pair(100, "AcDbEntity");
        out.pair(8, "0");
        out.pair(100, "AcDbBlockBegin");
        out.pair(2, name);
        out.pair(70, 0);
        out.pair(10, 0);
        out.pair(20, 0);
        out.pair(30, 0);
        out.pair(3, name);
        out.pair(1, "");
        out.pair(0, "ENDBLK");
        out.owned(owner);
        out.pair(100, "AcDbEntity");
        out.pair(8, "0");
        out.pair(100, "AcDbBlockEnd");
    }
    out.pair(0, "ENDSEC");
    Some(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::Piece;
    use shape_edit::{Circle, Rect, Shape};

    fn drawing() -> Drawing {
        Drawing {
            pieces: vec![
                Piece { shape: Shape::Rect(Rect::new([50.0, 40.0], 60.0, 30.0)), layer: "Ebene 1".into(), groups: vec![] },
                Piece { shape: Shape::Circle(Circle::new([120.0, 40.0], 10.0)), layer: "Löcher".into(), groups: vec![] },
            ],
            layers: vec!["Ebene 1".into(), "Löcher".into()],
            plate: [200.0, 100.0],
        }
    }

    fn read_back(text: &str) -> crate::import::Import {
        (crate::import::FORMATS[0].read)(text.as_bytes()).unwrap()
    }

    #[test]
    fn both_versions_read_back_with_the_same_shapes() {
        for version in Version::ALL {
            let text = write(&drawing(), &Options { version, ..Options::default() });
            assert!(text.contains(version.code()) && text.ends_with("EOF\n"));
            let imp = read_back(&text);
            assert_eq!(imp.shapes.len(), 2, "{}: a rectangle and a circle", version.name());
            assert!(imp.shapes.iter().any(|s| matches!(s, Shape::Circle(c) if (c.radius - 10.0).abs() < 1e-6)));
            let (lo, hi) = imp.shapes.iter().filter_map(|s| s.bounds()).next().unwrap();
            assert!((hi[0] - lo[0] - 60.0).abs() < 1e-6 && (hi[1] - lo[1] - 30.0).abs() < 1e-6);
        }
    }

    #[test]
    fn layers_and_units_follow_the_options_and_circles_stay_circles() {
        let text = write(&drawing(), &Options::default());
        assert!(text.contains("\nLöcher\n") && text.contains("\nCIRCLE\n"));
        let flat = write(&drawing(), &Options { layers: false, version: Version::R14, unit: Unit::Inch, frame: false });
        assert!(!flat.contains("Löcher") && flat.contains("\nCIRCLE\n"));
        let imp = read_back(&flat);
        assert_eq!(imp.unit, "Zoll");
        let widest = imp.shapes.iter().filter_map(|s| s.bounds()).map(|(lo, hi)| hi[0] - lo[0]).fold(0.0, f64::max);
        assert!((widest - 60.0).abs() < 1e-3, "read back in mm: {widest}");
    }

    #[test]
    fn the_workpiece_comes_as_a_rectangle_on_its_own_layer() {
        for version in Version::ALL {
            let text = write(&drawing(), &Options { frame: true, version, ..Options::default() });
            assert!(text.contains("\nWerkstück\n"));
            let imp = read_back(&text);
            assert_eq!(imp.shapes.len(), 3, "{}", version.name());
            assert!(imp.shapes.iter().filter_map(|s| s.bounds()).any(|(lo, hi)| lo == [0.0, 0.0] && hi == [200.0, 100.0]));
        }
    }

    #[test]
    fn forbidden_characters_leave_layer_names() {
        assert_eq!(layer_name("Teil: A/B"), "Teil_ A_B");
        assert_eq!(layer_name("  "), "0");
    }
}
