//! Hartschaum (PU-Blockmaterial für Modelle und Formen): hellgelb mit feinen Poren.

use super::pattern::{fbm, rgb, scale, speckle};
use super::{Look, Material};

pub const KEY: &str = "Foam";

pub fn material() -> Material {
    Material {
        key: KEY.into(),
        name: "Hartschaum".into(),
        description: "PU-Blockmaterial für Modelle und Formen: sehr leicht zu fräsen, staubt stark.".into(),
        max_feed: 3000.0,
        rpm_min: 12000,
        rpm_max: 20000,
        depth_ratio: 1.0,
        cutter: "2 Schneiden, HSS genügt; für Reliefs ein Kugelfräser.".into(),
        coolant: "Keine Kühlung, gut absaugen.".into(),
        look: KEY.into(),
        builtin: true,
    }
}

pub fn look() -> Look {
    Look { top: surface, edge: surface, end: surface, shine: 0.0, tile: 60.0 }
}

fn surface(u: f32, v: f32) -> [f32; 3] {
    scale(rgb(0xE6D490), (0.95 + 0.08 * fbm(u, v, 32, 32, 2, 71)) * (1.0 - 0.18 * speckle(u, v, 180, 0.74, 72)))
}
