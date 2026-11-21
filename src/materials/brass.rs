//! Messing: goldgelb, gebürstet, glänzend.

use super::pattern::{brushed, rgb, scale};
use super::{Look, Material};

pub const KEY: &str = "Brass";

pub fn material() -> Material {
    Material {
        key: KEY.into(),
        name: "Messing".into(),
        description: "Spröde Kupferlegierung: bricht kurze Späne, zieht den Fräser gern ins Material.".into(),
        max_feed: 600.0,
        rpm_min: 10000,
        rpm_max: 16000,
        depth_ratio: 0.2,
        cutter: "2 oder 3 Schneiden, Hartmetall mit geradem Spanwinkel (0°), damit er nicht ins Material zieht.".into(),
        coolant: "Trocken mit Druckluft oder etwas Schneidöl.".into(),
        look: KEY.into(),
        builtin: true,
    }
}

pub fn look() -> Look {
    Look { top, edge: side, end: side, shine: 0.85, tile: 100.0 }
}

fn top(u: f32, v: f32) -> [f32; 3] {
    scale(rgb(0xD9B25E), 0.86 + 0.18 * brushed(u, v, 91))
}

fn side(u: f32, v: f32) -> [f32; 3] {
    scale(rgb(0xC9A04E), 0.84 + 0.18 * brushed(u, v, 92))
}
