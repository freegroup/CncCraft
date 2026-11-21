//! Sperrholz: kreuzweise verleimte Lagen. Oben ein heller Birkenfurnier, an den Kanten die Lagen.

use super::pattern::{bands, fibres, mix, noise, rgb, scale, smoothstep};
use super::{Look, Material};

pub const KEY: &str = "Plywood";

pub fn material() -> Material {
    Material {
        key: KEY.into(),
        name: "Sperrholz".into(),
        description: "Birke, Pappel oder Buche, kreuzweise verleimt: die Lagen reißen an den Kanten gern aus.".into(),
        max_feed: 2200.0,
        rpm_min: 16000,
        rpm_max: 22000,
        depth_ratio: 0.5,
        cutter: "2 Schneiden, Hartmetall. Ein Kompressionsfräser (oben ab-, unten aufwärts) gibt auf beiden Seiten saubere Kanten.".into(),
        coolant: "Keine Kühlung, nur absaugen.".into(),
        look: KEY.into(),
        builtin: true,
    }
}

pub fn look() -> Look {
    Look { top, edge: layers, end: layers, shine: 0.06, tile: 72.0 }
}

/// Veneer layers of 1.5 mm across a 72 mm tile.
const LAYERS: u32 = 48;

fn top(u: f32, v: f32) -> [f32; 3] {
    let grain = bands(u, v, 5, 1.2, 0.18, 31);
    scale(mix(rgb(0xEBD8B4), rgb(0xCDAE80), grain * 0.3), 0.94 + 0.08 * fibres(u, v, 180, 32))
}

/// The edge: veneers whose grain runs along it (light) and across it (darker end grain) in turn,
/// each with a thin dark glue line.
fn layers(u: f32, v: f32) -> [f32; 3] {
    // half a layer off, so the tile's edge runs through the middle of a layer, not a glue line
    let t = v * LAYERS as f32 + 0.5;
    let layer = t.floor() as u32 % LAYERS;
    let within = t.fract();
    let face = if layer % 2 == 0 {
        scale(rgb(0xE8D2AA), 0.94 + 0.08 * fibres(u, v, 400, 33))
    } else {
        scale(rgb(0xCFAE80), 0.88 + 0.14 * noise(u, v, 160, 160, 34))
    };
    mix(face, rgb(0x8E6B44), 1.0 - smoothstep(0.0, 0.08, within))
}
