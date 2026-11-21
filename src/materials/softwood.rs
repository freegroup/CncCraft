//! Weichholz: Kiefer, Fichte, Pappel. Hell, mit deutlichen dunkleren Spätholzstreifen.

use super::pattern::{arcs, bands, fibres, mix, rgb, scale};
use super::{Look, Material};

pub const KEY: &str = "Softwood";

pub fn material() -> Material {
    Material {
        key: KEY.into(),
        name: "Weichholz".into(),
        description: "Kiefer, Fichte, Pappel: leicht zu fräsen, die Fasern reißen an den Kanten leicht aus.".into(),
        // the fastest softwood cutter in the original feeds 1350 mm/min (tools.vtdb)
        max_feed: 1800.0,
        // the original's softwood tools run at 13000 (V-bits, drills) to 17000 rpm (tools.vtdb)
        rpm_min: 13000,
        rpm_max: 24000,
        depth_ratio: 0.67,
        cutter: "2 Schneiden, HSS oder Hartmetall. Aufwärtsspirale räumt die Späne gut, Abwärtsspirale gibt saubere Oberkanten.".into(),
        coolant: "Keine Kühlung, nur absaugen.".into(),
        look: KEY.into(),
        builtin: true,
    }
}

pub fn look() -> Look {
    Look { top, edge, end, shine: 0.05, tile: 120.0 }
}

const EARLY: u32 = 0xEBCFA0;
const LATE: u32 = 0xC68F52;

fn top(u: f32, v: f32) -> [f32; 3] {
    let ring = bands(u, v, 9, 0.9, 0.25, 11);
    scale(mix(rgb(EARLY), rgb(LATE), ring * 0.85), 0.93 + 0.12 * fibres(u, v, 160, 12))
}

fn edge(u: f32, v: f32) -> [f32; 3] {
    let ring = bands(u, v, 14, 0.25, 0.22, 13);
    scale(mix(rgb(EARLY), rgb(LATE), ring * 0.8), 0.92 + 0.12 * fibres(u, v, 200, 14))
}

fn end(u: f32, v: f32) -> [f32; 3] {
    scale(mix(rgb(0xE0BE8A), rgb(0xB57D45), arcs(u, v, 12, 15) * 0.9), 0.9 + 0.12 * fibres(v, u, 64, 16))
}
