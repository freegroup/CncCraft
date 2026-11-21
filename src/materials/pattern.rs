//! What the material looks are made of: noise that repeats at the edges of the tile (so the pictures
//! tile without seams), fibres, rings and colour helpers. Coordinates are those of the tile, 0..1.

use std::f32::consts::TAU;

/// A colour from `0xRRGGBB`.
pub fn rgb(hex: u32) -> [f32; 3] {
    [((hex >> 16) & 0xff) as f32 / 255.0, ((hex >> 8) & 0xff) as f32 / 255.0, (hex & 0xff) as f32 / 255.0]
}

pub fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

pub fn scale(c: [f32; 3], f: f32) -> [f32; 3] {
    [c[0] * f, c[1] * f, c[2] * f]
}

pub fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A pseudo-random number in 0..1 for a lattice point.
fn hash(x: u32, y: u32, seed: u32) -> f32 {
    let mut h = x.wrapping_mul(0x8da6_b343) ^ y.wrapping_mul(0xd816_3841) ^ seed.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0x00ff_ffff) as f32 / 0x0100_0000 as f32
}

/// Smooth value noise (0..1) on a lattice of `px` × `py` cells over the tile; it repeats at the
/// tile's edges.
pub fn noise(u: f32, v: f32, px: u32, py: u32, seed: u32) -> f32 {
    let (px, py) = (px.max(1), py.max(1));
    let (x, y) = (u.rem_euclid(1.0) * px as f32, v.rem_euclid(1.0) * py as f32);
    let (x0, y0) = (x.floor() as u32 % px, y.floor() as u32 % py);
    let (x1, y1) = ((x0 + 1) % px, (y0 + 1) % py);
    let (fx, fy) = (x.fract(), y.fract());
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let a = hash(x0, y0, seed) + (hash(x1, y0, seed) - hash(x0, y0, seed)) * sx;
    let b = hash(x0, y1, seed) + (hash(x1, y1, seed) - hash(x0, y1, seed)) * sx;
    a + (b - a) * sy
}

/// `octaves` layers of [`noise`], each twice as fine and half as strong (0..1).
pub fn fbm(u: f32, v: f32, px: u32, py: u32, octaves: u32, seed: u32) -> f32 {
    let (mut sum, mut amp, mut total) = (0.0, 1.0, 0.0);
    for o in 0..octaves {
        sum += noise(u, v, px << o, py << o, seed.wrapping_add(o * 101)) * amp;
        total += amp;
        amp *= 0.5;
    }
    sum / total
}

/// Long thin fibres along u: noise stretched across v (0..1).
pub fn fibres(u: f32, v: f32, across: u32, seed: u32) -> f32 {
    fbm(u, v, 3, across, 3, seed)
}

/// Growth rings or layers across v: 0 in the light part, 1 in the dark band. `count` bands fit the
/// tile, `wobble` bends them with noise, `band` is the dark share of each.
pub fn bands(u: f32, v: f32, count: u32, wobble: f32, band: f32, seed: u32) -> f32 {
    let bent = v * count as f32 + wobble * (fbm(u, v, 2, 4, 3, seed) - 0.5) * 2.0;
    let t = bent.rem_euclid(1.0);
    smoothstep(1.0 - band - 0.08, 1.0 - band, t) * (1.0 - smoothstep(0.94, 1.0, t))
}

/// End-grain rings: arcs across the tile, as a log's rings meet a cut across it.
pub fn arcs(u: f32, v: f32, count: u32, seed: u32) -> f32 {
    let bent = v + 0.06 * (TAU * u).cos() + 0.03 * (fbm(u, v, 3, 3, 2, seed) - 0.5);
    bands(0.0, bent, count, 0.0, 0.3, seed)
}

/// Fine streaks along u, as left by brushing or milling metal (0..1).
pub fn brushed(u: f32, v: f32, seed: u32) -> f32 {
    0.6 * noise(u, v, 4, 512, seed) + 0.4 * noise(u, v, 2, 128, seed.wrapping_add(7))
}

/// Small grains or pores scattered over the tile: 1 where one is (0..1).
pub fn speckle(u: f32, v: f32, density: u32, threshold: f32, seed: u32) -> f32 {
    smoothstep(threshold, threshold + 0.06, noise(u, v, density, density, seed))
}
