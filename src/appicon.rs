//! The program's icon, drawn in code (so there is no picture file to ship): a teal rounded square
//! with a white mark of eight rounded petals around the centre – four straight ones and four that
//! reach into the corners – after the picture the person chose.
//!
//! The rounded square does not fill the whole canvas: it sits inside a transparent margin, on the
//! macOS icon grid (the content square is ~82% of the tile, the rest clear). Without that margin the
//! icon touched every edge and the Dock drew it noticeably larger than its neighbours.

const TEAL_TOP: [f32; 3] = [0.20, 0.78, 0.75];
const TEAL_BOTTOM: [f32; 3] = [0.10, 0.62, 0.62];

/// Transparent border around the rounded square, in parts of the tile (each side). macOS's icon
/// grid insets the content to ~824/1024, i.e. ~0.098 per side.
const MARGIN: f32 = 0.098;

/// Half the width of the mark, in parts of the icon's width.
const HALF: f32 = 0.31;
/// How round the petals' corners are, in parts of `HALF`.
const ROUND: f32 = 0.05;

/// The straight petal pointing up, without its rounding (in parts of `HALF`, centre at 0, y down).
const STRAIGHT: [[f32; 2]; 5] = [[-0.15, -0.95], [0.15, -0.95], [0.215, -0.75], [0.0, -0.19], [-0.215, -0.75]];
/// The petal reaching into the upper left corner, without its rounding.
const DIAGONAL: [[f32; 2]; 5] = [[-0.95, -0.95], [-0.72, -0.95], [-0.27, -0.33], [-0.33, -0.27], [-0.95, -0.72]];

/// All eight petals: each base petal turned by 0°, 90°, 180° and 270°.
fn petals() -> Vec<Vec<[f32; 2]>> {
    let turn = |p: [f32; 2], k: usize| (0..k).fold(p, |q, _| [-q[1], q[0]]);
    (0..4).flat_map(|k| [STRAIGHT, DIAGONAL].map(|base| base.iter().map(|&p| turn(p, k)).collect())).collect()
}

fn dist_to_segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (ab, ap) = ([b[0] - a[0], b[1] - a[1]], [p[0] - a[0], p[1] - a[1]]);
    let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-9)).clamp(0.0, 1.0);
    ((p[0] - a[0] - ab[0] * t).powi(2) + (p[1] - a[1] - ab[1] * t).powi(2)).sqrt()
}

fn inside_polygon(p: [f32; 2], poly: &[[f32; 2]]) -> bool {
    let mut inside = false;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < a[0] + (p[1] - a[1]) / (b[1] - a[1]) * (b[0] - a[0]) {
            inside = !inside;
        }
    }
    inside
}

/// Whether the point (0..1, 0..1) lies in the white mark.
fn in_mark(p: [f32; 2], petals: &[Vec<[f32; 2]>]) -> bool {
    let q = [(p[0] - 0.5) / HALF, (p[1] - 0.5) / HALF];
    petals.iter().any(|poly| inside_polygon(q, poly) || (0..poly.len()).any(|i| dist_to_segment(q, poly[i], poly[(i + 1) % poly.len()]) < ROUND))
}

/// Map a tile point (0..1 over the whole canvas) into the content square's own 0..1 space, or
/// `None` if it falls in the transparent margin.
fn to_content(p: [f32; 2]) -> Option<[f32; 2]> {
    let span = 1.0 - 2.0 * MARGIN;
    let q = [(p[0] - MARGIN) / span, (p[1] - MARGIN) / span];
    (q[0] >= 0.0 && q[0] <= 1.0 && q[1] >= 0.0 && q[1] <= 1.0).then_some(q)
}

/// Whether the point lies in the rounded square.
fn in_square(p: [f32; 2]) -> bool {
    let r = 0.22;
    let q = [(p[0] - 0.5).abs() - (0.5 - r), (p[1] - 0.5).abs() - (0.5 - r)];
    let outside = [q[0].max(0.0), q[1].max(0.0)];
    (outside[0] * outside[0] + outside[1] * outside[1]).sqrt() + q[0].max(q[1]).min(0.0) <= r
}

/// The icon as RGBA pixels, `size` × `size`.
pub fn rgba(size: u32) -> Vec<u8> {
    const SAMPLES: u32 = 3;
    let petals = petals();
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let (mut cover, mut white, mut r, mut g, mut b) = (0.0, 0.0, 0.0, 0.0, 0.0);
            for sy in 0..SAMPLES {
                for sx in 0..SAMPLES {
                    let tile = [(x as f32 + (sx as f32 + 0.5) / SAMPLES as f32) / size as f32, (y as f32 + (sy as f32 + 0.5) / SAMPLES as f32) / size as f32];
                    // into the content square's own space; the margin around it stays transparent
                    let Some(p) = to_content(tile) else { continue };
                    if !in_square(p) {
                        continue;
                    }
                    cover += 1.0;
                    if in_mark(p, &petals) {
                        white += 1.0;
                        continue;
                    }
                    let t = p[1];
                    r += TEAL_TOP[0] + (TEAL_BOTTOM[0] - TEAL_TOP[0]) * t;
                    g += TEAL_TOP[1] + (TEAL_BOTTOM[1] - TEAL_TOP[1]) * t;
                    b += TEAL_TOP[2] + (TEAL_BOTTOM[2] - TEAL_TOP[2]) * t;
                }
            }
            let total = (SAMPLES * SAMPLES) as f32;
            if cover == 0.0 {
                out.extend([0, 0, 0, 0]);
                continue;
            }
            let mix = |c: f32| ((c + white) / cover * 255.0).round().clamp(0.0, 255.0) as u8;
            out.extend([mix(r), mix(g), mix(b), (cover / total * 255.0).round() as u8]);
        }
    }
    out
}

pub fn icon_data(size: u32) -> egui::IconData {
    egui::IconData { rgba: rgba(size), width: size, height: size }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_corners_are_clear_the_petals_white_and_the_centre_open() {
        let n = 128;
        let px = rgba(n);
        let at = |x: u32, y: u32| &px[((y * n + x) * 4) as usize..((y * n + x) * 4 + 4) as usize];
        assert_eq!(at(0, 0)[3], 0, "a corner of the tile is transparent");
        // the margin around the rounded square is transparent (macOS icon grid)
        assert_eq!(at(n / 2, (n as f32 * MARGIN * 0.5) as u32)[3], 0, "the top margin is clear");
        let centre = at(n / 2, n / 2);
        assert!(centre[3] == 255 && centre[0] < 120, "the petals leave the centre teal");
        // petal/edge samples are taken inside the content square (map a content-space y into the tile)
        let tile_y = |cy: f32| (n as f32 * (MARGIN + (1.0 - 2.0 * MARGIN) * cy)) as u32;
        let top_petal = at(n / 2, tile_y(0.5 - 0.6 * HALF));
        assert!(top_petal[0] > 240 && top_petal[1] > 240 && top_petal[2] > 240, "the upper petal is white");
        let tile_xy = |c: f32| (n as f32 * (MARGIN + (1.0 - 2.0 * MARGIN) * c)) as u32;
        let corner_petal = at(tile_xy(0.5 - 0.85 * HALF), tile_xy(0.5 - 0.85 * HALF));
        assert!(corner_petal[0] > 240, "the corner petal is white");
        let edge = at(n / 2, tile_y(0.05));
        assert!(edge[3] == 255 && edge[0] < 120 && edge[1] > 120, "the square is teal just inside its top edge");
    }
}
