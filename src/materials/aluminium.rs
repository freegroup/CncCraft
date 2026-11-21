//! Aluminium: silbergrau, gebürstet; gefräste Flächen glänzen.

use super::pattern::{brushed, rgb, scale};
use super::{Look, Material};

pub const KEY: &str = "Aluminium";

pub fn material() -> Material {
    Material {
        key: KEY.into(),
        name: "Aluminium".into(),
        description: "Weiche Legierungen wie 6060 oder 6082. Kleine Zustellung, die Späne müssen weg.".into(),
        max_feed: 800.0,
        rpm_min: 10000,
        rpm_max: 18000,
        depth_ratio: 0.25,
        cutter: "1 oder 2 Schneiden, Hartmetall mit polierter Spannut (für Aluminium).".into(),
        coolant: "Isopropanol (Spiritus) aufsprühen oder Minimalmengenschmierung, Späne wegblasen.".into(),
        look: KEY.into(),
        builtin: true,
    }
}

pub fn look() -> Look {
    Look { top, edge: side, end: side, shine: 0.9, tile: 100.0 }
}

fn top(u: f32, v: f32) -> [f32; 3] {
    scale(rgb(0xC9CED4), 0.86 + 0.2 * brushed(u, v, 81))
}

fn side(u: f32, v: f32) -> [f32; 3] {
    scale(rgb(0xB9BEC4), 0.84 + 0.2 * brushed(u, v, 82))
}
