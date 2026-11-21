//! Hartholz: Eiche, Buche, Ahorn, Nuss. Mittelbraun, dichte feine Jahresringe und dunkle Poren.

use super::pattern::{arcs, bands, fibres, mix, noise, rgb, scale, smoothstep};
use super::{Look, Material};

pub const KEY: &str = "Hardwood";

pub fn material() -> Material {
    Material {
        key: KEY.into(),
        name: "Hartholz".into(),
        description: "Eiche, Buche, Ahorn, Nuss: fest und dicht, braucht weniger Zustellung als Weichholz.".into(),
        // the fastest hardwood cutter in the original feeds 960 mm/min (tools.vtdb)
        max_feed: 1400.0,
        // the original's hardwood tools run at 15000 (V-bits, drills) to 19000 rpm (tools.vtdb)
        rpm_min: 15000,
        rpm_max: 22000,
        depth_ratio: 0.5,
        cutter: "2 Schneiden, Hartmetall. Bei dunklen Brandspuren schneller fahren oder die Drehzahl senken.".into(),
        coolant: "Keine Kühlung, nur absaugen.".into(),
        look: KEY.into(),
        builtin: true,
    }
}

pub fn look() -> Look {
    Look { top, edge, end, shine: 0.12, tile: 100.0 }
}

const LIGHT: u32 = 0xC39C6C;
const DARK: u32 = 0x8A6239;

/// Open pores: short dark dashes along the grain.
fn pores(u: f32, v: f32) -> f32 {
    smoothstep(0.76, 0.84, noise(u, v, 24, 420, 22))
}

fn top(u: f32, v: f32) -> [f32; 3] {
    let ring = bands(u, v, 18, 0.6, 0.35, 21);
    let c = mix(rgb(LIGHT), rgb(DARK), ring * 0.55);
    scale(c, (0.92 + 0.12 * fibres(u, v, 220, 23)) * (1.0 - 0.25 * pores(u, v)))
}

fn edge(u: f32, v: f32) -> [f32; 3] {
    let ring = bands(u, v, 26, 0.2, 0.3, 24);
    scale(mix(rgb(LIGHT), rgb(DARK), ring * 0.5), (0.9 + 0.12 * fibres(u, v, 260, 25)) * (1.0 - 0.2 * pores(u, v)))
}

fn end(u: f32, v: f32) -> [f32; 3] {
    let pore = smoothstep(0.8, 0.86, noise(u, v, 90, 90, 26));
    scale(mix(rgb(0xB48C5C), rgb(0x7A5531), arcs(u, v, 22, 27) * 0.7), 0.9 - 0.25 * pore)
}
