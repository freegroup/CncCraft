//! Kupfer: rötlich, gebürstet, mit leichten dunkleren Anlaufstellen.

use super::pattern::{brushed, fbm, mix, rgb, scale};
use super::{Look, Material};

pub const KEY: &str = "Copper";

pub fn material() -> Material {
    Material {
        key: KEY.into(),
        name: "Kupfer".into(),
        description: "Reines Kupfer ist weich und zäh: es schmiert und klebt am Fräser.".into(),
        max_feed: 500.0,
        rpm_min: 10000,
        rpm_max: 16000,
        depth_ratio: 0.2,
        cutter: "2 Schneiden, Hartmetall, scharf und poliert (wie für Aluminium).".into(),
        coolant: "Schneidöl oder Isopropanol, Späne wegblasen.".into(),
        look: KEY.into(),
        builtin: true,
    }
}

pub fn look() -> Look {
    Look { top, edge: side, end: side, shine: 0.8, tile: 100.0 }
}

fn top(u: f32, v: f32) -> [f32; 3] {
    let tarnish = fbm(u, v, 3, 3, 3, 101);
    scale(mix(rgb(0xCD7E4E), rgb(0xA4603A), (tarnish - 0.45) * 1.6), 0.86 + 0.18 * brushed(u, v, 102))
}

fn side(u: f32, v: f32) -> [f32; 3] {
    // freshly cut copper is bright, without tarnish
    scale(rgb(0xD98A58), 0.84 + 0.18 * brushed(u, v, 103))
}
