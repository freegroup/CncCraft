//! The program's icon, drawn in code (so there is no picture file to ship): a teal rounded square
//! with a white hub – a centre node with six arms that end in smaller nodes – after the picture the
//! person chose.

const TEAL_TOP: [f32; 3] = [0.20, 0.78, 0.75];
const TEAL_BOTTOM: [f32; 3] = [0.10, 0.62, 0.62];

/// The arms: their direction in degrees (0 = right, 90 = down) and length, in parts of the icon's width.
const ARMS: [(f32, f32); 6] = [(-135.0, 0.31), (-75.0, 0.33), (-15.0, 0.32), (45.0, 0.30), (105.0, 0.32), (165.0, 0.33)];

fn dist_to_segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (ab, ap) = ([b[0] - a[0], b[1] - a[1]], [p[0] - a[0], p[1] - a[1]]);
    let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1]).max(1e-9)).clamp(0.0, 1.0);
    ((p[0] - a[0] - ab[0] * t).powi(2) + (p[1] - a[1] - ab[1] * t).powi(2)).sqrt()
}

/// Whether the point (0..1, 0..1) lies in the white hub.
fn in_hub(p: [f32; 2]) -> bool {
    let c = [0.5, 0.5];
    if ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2)).sqrt() < 0.125 {
        return true;
    }
    ARMS.iter().any(|&(deg, len)| {
        let (s, k) = (deg.to_radians().sin(), deg.to_radians().cos());
        let end = [c[0] + k * len, c[1] + s * len];
        // the arm is thicker at the centre and thinner at its node
        let along = dist_to_segment(p, c, end);
        let width = 0.045 - 0.012 * (((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2)).sqrt() / len).min(1.0);
        along < width || ((p[0] - end[0]).powi(2) + (p[1] - end[1]).powi(2)).sqrt() < 0.075
    })
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
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let (mut cover, mut white, mut r, mut g, mut b) = (0.0, 0.0, 0.0, 0.0, 0.0);
            for sy in 0..SAMPLES {
                for sx in 0..SAMPLES {
                    let p = [(x as f32 + (sx as f32 + 0.5) / SAMPLES as f32) / size as f32, (y as f32 + (sy as f32 + 0.5) / SAMPLES as f32) / size as f32];
                    if !in_square(p) {
                        continue;
                    }
                    cover += 1.0;
                    if in_hub(p) {
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
    fn the_corners_are_clear_and_the_middle_is_white() {
        let n = 128;
        let px = rgba(n);
        let at = |x: u32, y: u32| &px[((y * n + x) * 4) as usize..((y * n + x) * 4 + 4) as usize];
        assert_eq!(at(0, 0)[3], 0, "a corner of the rounded square is transparent");
        let mid = at(n / 2, n / 2);
        assert!(mid[0] > 240 && mid[1] > 240 && mid[2] > 240 && mid[3] == 255, "the hub is white");
        let edge = at(n / 2, 6);
        assert!(edge[3] == 255 && edge[0] < 120 && edge[1] > 120, "the square is teal");
    }
}
