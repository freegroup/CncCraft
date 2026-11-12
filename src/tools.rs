//! Tool library: the metric cutters and cutting data shipped with the original
//! (ProgramData/ToolDatabase/tools.vtdb.vectric), see MECHANICS.md.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolKind {
    BallNose,
    EndMill,
    VBit,
    Drill,
}

impl ToolKind {
    fn from_db(k: u32) -> Option<Self> {
        match k {
            0 => Some(ToolKind::BallNose),
            1 => Some(ToolKind::EndMill),
            3 => Some(ToolKind::VBit),
            6 => Some(ToolKind::Drill),
            _ => None,
        }
    }
    pub fn group(&self) -> &'static str {
        match self {
            ToolKind::EndMill => "Schaftfräser",
            ToolKind::BallNose => "Kugelfräser",
            ToolKind::VBit => "V-Fräser",
            ToolKind::Drill => "Bohrer",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    pub name: String,
    pub kind: ToolKind,
    /// mm
    pub diameter: f64,
    /// included angle in degrees (V-bits, drills)
    pub angle: f64,
    /// mm/min
    pub feed: f64,
    pub plunge: f64,
    pub rpm: u32,
    /// pass depth, mm
    pub stepdown: f64,
    /// mm
    pub stepover: f64,
    pub number: u32,
}

#[derive(Deserialize)]
struct Row {
    name: String,
    kind: u32,
    diameter: f64,
    angle: f64,
    material: String,
    feed: f64,
    plunge: f64,
    rpm: u32,
    stepdown: f64,
    stepover: f64,
}

pub const MATERIALS: [&str; 4] = ["Softwood", "Hardwood", "MDF", "Acrylic"];

pub fn material_label(m: &str) -> &'static str {
    match m {
        "Softwood" => "Weichholz",
        "Hardwood" => "Hartholz",
        "MDF" => "MDF",
        "Acrylic" => "Acryl",
        _ => "Material",
    }
}

pub struct ToolLibrary {
    rows: Vec<(String, Tool)>,
}

impl ToolLibrary {
    pub fn load() -> Self {
        let rows: Vec<Row> = serde_json::from_str(include_str!("../data/tools.json")).unwrap_or_default();
        let rows = rows
            .into_iter()
            .filter_map(|r| {
                let kind = ToolKind::from_db(r.kind)?;
                // drills have no feed in the library: they only plunge
                let feed = if r.feed > 0.0 { r.feed } else { r.plunge };
                Some((
                    r.material,
                    Tool {
                        name: r.name,
                        kind,
                        diameter: r.diameter,
                        angle: r.angle,
                        feed,
                        plunge: r.plunge,
                        rpm: r.rpm,
                        stepdown: r.stepdown,
                        stepover: r.stepover,
                        number: 1,
                    },
                ))
            })
            .collect();
        ToolLibrary { rows }
    }

    pub fn tools_for(&self, material: &str) -> Vec<&Tool> {
        self.rows.iter().filter(|(m, _)| m == material).map(|(_, t)| t).collect()
    }

    /// The first tool of a kind for a material whose diameter is closest to `dia`.
    pub fn pick(&self, material: &str, kind: ToolKind, dia: f64, angle: Option<f64>) -> Tool {
        let mut best: Option<&Tool> = None;
        for t in self.tools_for(material) {
            if t.kind != kind {
                continue;
            }
            if let Some(a) = angle {
                if (t.angle - a).abs() > 0.5 {
                    continue;
                }
            }
            if best.is_none_or(|b| (t.diameter - dia).abs() < (b.diameter - dia).abs()) {
                best = Some(t);
            }
        }
        best.cloned().unwrap_or(Tool {
            name: format!("{} ({dia} mm)", kind.group()),
            kind,
            diameter: dia,
            angle: angle.unwrap_or(90.0),
            feed: 1000.0,
            plunge: 300.0,
            rpm: 16000,
            stepdown: dia / 3.0,
            stepover: dia * 0.4,
            number: 1,
        })
    }
}
