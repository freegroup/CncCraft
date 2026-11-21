//! HDPE und ähnliche Kunststoffe (POM, PE): mattweiß, leicht wolkig, wachsartig.

use super::pattern::{fbm, noise, rgb, scale};
use super::{Look, Material};

pub const KEY: &str = "HDPE";

pub fn material() -> Material {
    Material {
        key: KEY.into(),
        name: "HDPE / Kunststoff".into(),
        description: "Polyethylen, POM und ähnliche Kunststoffe: zäh, schmilzt bei zu wenig Vorschub.".into(),
        max_feed: 2000.0,
        rpm_min: 12000,
        rpm_max: 18000,
        depth_ratio: 0.5,
        cutter: "1 Schneide (Einschneider, O-Nut), Hartmetall, scharf: große Späne nehmen die Wärme mit.".into(),
        coolant: "Druckluft; Flüssigkeit ist nicht nötig.".into(),
        look: KEY.into(),
        builtin: true,
    }
}

pub fn look() -> Look {
    Look { top, edge: side, end: side, shine: 0.2, tile: 120.0 }
}

fn top(u: f32, v: f32) -> [f32; 3] {
    scale(rgb(0xF1F0EA), 0.95 + 0.05 * fbm(u, v, 8, 8, 3, 61))
}

fn side(u: f32, v: f32) -> [f32; 3] {
    scale(rgb(0xE6E5DE), 0.93 + 0.07 * noise(u, v, 2, 240, 62))
}
