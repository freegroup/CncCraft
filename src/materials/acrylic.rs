//! Acryl (Plexiglas): glatt und hell, die Schnittkanten grünlich und matt mit feinen Riefen.

use super::pattern::{fbm, mix, noise, rgb, scale};
use super::{Look, Material};

pub const KEY: &str = "Acrylic";

pub fn material() -> Material {
    Material {
        key: KEY.into(),
        name: "Acryl".into(),
        description: "Plexiglas (PMMA): schmilzt, wenn der Fräser reibt statt schneidet.".into(),
        // the fastest acrylic cutter in the original feeds 1200 mm/min (tools.vtdb)
        max_feed: 1600.0,
        rpm_min: 12000,
        rpm_max: 18000,
        depth_ratio: 0.5,
        cutter: "1 Schneide (Einschneider, O-Nut), Hartmetall: großer Spanraum, damit die Späne nicht anschmelzen.".into(),
        coolant: "Druckluft, um zu kühlen und die Späne wegzublasen.".into(),
        look: KEY.into(),
        builtin: true,
    }
}

pub fn look() -> Look {
    Look { top, edge: side, end: side, shine: 0.75, tile: 150.0 }
}

fn top(u: f32, v: f32) -> [f32; 3] {
    // a clear sheet over a light table: almost even, with soft reflections
    mix(rgb(0xE6F0F4), rgb(0xCFE0E8), fbm(u, v, 2, 2, 2, 51) * 0.6)
}

fn side(u: f32, v: f32) -> [f32; 3] {
    scale(rgb(0xBADDD4), 0.9 + 0.12 * noise(u, v, 2, 300, 52))
}
