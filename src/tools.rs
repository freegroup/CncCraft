//! Tool library: the metric cutters and cutting data shipped with the original
//! (ProgramData/ToolDatabase/tools.vtdb.vectric), see MECHANICS.md.  Every tool is editable.

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
    pub fn all() -> &'static [ToolKind] {
        &[ToolKind::EndMill, ToolKind::BallNose, ToolKind::VBit, ToolKind::Drill]
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

/// One entry in the tool library: the tool and the material it is for.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolEntry {
    pub material: String,
    pub tool: Tool,
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
    pub entries: Vec<ToolEntry>,
}

impl ToolLibrary {
    fn builtins() -> Vec<ToolEntry> {
        let rows: Vec<Row> = serde_json::from_str(include_str!("../data/tools.json")).unwrap_or_default();
        rows.into_iter()
            .filter_map(|r| {
                let kind = ToolKind::from_db(r.kind)?;
                let feed = if r.feed > 0.0 { r.feed } else { r.plunge };
                Some(ToolEntry {
                    material: r.material,
                    tool: Tool {
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
                })
            })
            .collect()
    }

    pub fn load() -> Self {
        ToolLibrary { entries: Self::builtins() }
    }

    /// Replace the tool list with a persisted version; falls back to builtins if the JSON is bad.
    pub fn load_from(json: &str) -> Self {
        match serde_json::from_str::<Vec<ToolEntry>>(json) {
            Ok(entries) if !entries.is_empty() => ToolLibrary { entries },
            _ => Self::load(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.entries).unwrap_or_default()
    }

    pub fn tools_for(&self, material: &str) -> Vec<&Tool> {
        self.entries.iter().filter(|e| e.material == material).map(|e| &e.tool).collect()
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

    pub fn default_tool(kind: ToolKind) -> Tool {
        let dia = match kind {
            ToolKind::EndMill => 3.0,
            ToolKind::BallNose => 3.0,
            ToolKind::VBit => 6.0,
            ToolKind::Drill => 3.0,
        };
        Tool {
            name: format!("{} ({dia} mm)", kind.group()),
            kind,
            diameter: dia,
            angle: if kind == ToolKind::VBit { 90.0 } else if kind == ToolKind::Drill { 118.0 } else { 0.0 },
            feed: 1000.0,
            plunge: 300.0,
            rpm: 16000,
            stepdown: dia / 3.0,
            stepover: dia * 0.4,
            number: 1,
        }
    }
}
