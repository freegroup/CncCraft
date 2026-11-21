//! MDF: gepresste Holzfasern, gleichmäßig hellbraun, an den Kanten etwas dunkler und faserig.

use super::pattern::{fbm, rgb, scale, speckle};
use super::{Look, Material};

pub const KEY: &str = "MDF";

pub fn material() -> Material {
    Material {
        key: KEY.into(),
        name: "MDF".into(),
        description: "Holzfaserplatte: gleichmäßig, fräst sich sauber, macht sehr feinen Staub.".into(),
        // the fastest MDF cutter in the original feeds 2000 mm/min (tools.vtdb)
        max_feed: 2000.0,
        rpm_min: 16000,
        rpm_max: 24000,
        depth_ratio: 0.67,
        cutter: "2 Schneiden, Hartmetall: der Leim in der Platte macht HSS-Fräser schnell stumpf.".into(),
        coolant: "Keine Kühlung, aber unbedingt absaugen: der Staub ist gesundheitsschädlich.".into(),
        look: KEY.into(),
        builtin: true,
    }
}

pub fn look() -> Look {
    Look { top, edge: side, end: side, shine: 0.03, tile: 80.0 }
}

fn top(u: f32, v: f32) -> [f32; 3] {
    scale(rgb(0xBC9563), (0.94 + 0.1 * fbm(u, v, 48, 48, 3, 41)) * (1.0 - 0.12 * speckle(u, v, 220, 0.8, 42)))
}

fn side(u: f32, v: f32) -> [f32; 3] {
    scale(rgb(0xA8824F), (0.9 + 0.16 * fbm(u, v, 160, 160, 2, 43)) * (1.0 - 0.15 * speckle(u, v, 300, 0.78, 44)))
}
