//! Toolpath preview: a height map of the material, carved by sweeping the cutter's shape along
//! every cutting move, and the camera that looks at it. Drawing is in [`crate::view3d`].

use crate::tools::{Tool, ToolKind};
use crate::toolpath::{Job, Toolpath};
use egui::{Pos2, Rect, Stroke};
use std::sync::atomic::{AtomicU64, Ordering};

/// Gives every height map its own range of version numbers, so a new one never looks unchanged.
static NEXT_MAP: AtomicU64 = AtomicU64::new(1);

/// The original's simulation quality: grid points per millimetre (toolpath.simulation_quality;
/// starts at 5.0, global 0x102112ed8), kept between 0.01 and 40 (setter 0x10129edf8).
pub const SIMULATION_QUALITY: f64 = 5.0;
pub const SIMULATION_QUALITY_MIN: f64 = 0.01;
pub const SIMULATION_QUALITY_MAX: f64 = 40.0;
/// The grid has width × height × quality² points, but never fewer or more than these
/// (0x10129eeac: 50000.0 and 16000000.0).
pub const SIMULATION_POINTS_MIN: f64 = 50000.0;
pub const SIMULATION_POINTS_MAX: f64 = 16000000.0;

/// Grid spacing in mm for a board of `width` × `height` mm, as the original works it out:
/// points = clamp(width × height × quality², 50 000, 16 000 000) (0x10129eeac), spacing =
/// √(width × height / points) (0x10129f008).
pub fn grid_spacing(width: f64, height: f64, quality: f64) -> f64 {
    let q = quality.clamp(SIMULATION_QUALITY_MIN, SIMULATION_QUALITY_MAX);
    let area = (width * height).max(1e-9);
    let points = (area * q * q).clamp(SIMULATION_POINTS_MIN, SIMULATION_POINTS_MAX).round();
    (area / points).sqrt()
}

/// Material surface heights on a regular grid. Z is 0 at the top of the material and
/// `bottom` (minus the thickness) at the machine bed.
pub struct HeightMap {
    pub nx: usize,
    pub ny: usize,
    /// Grid spacing in millimetres.
    pub cell: f64,
    pub z: Vec<f32>,
    pub bottom: f32,
    /// Changes whenever the surface changes, so the renderer knows when to rebuild.
    pub version: u64,
}

impl HeightMap {
    /// The preview grid for a job at the given simulation quality (points per mm).
    pub fn for_preview(job: &Job, quality: f64) -> Self {
        Self::with_spacing(job, grid_spacing(job.width, job.height, quality))
    }

    #[cfg(test)]
    pub fn new(job: &Job, max_cells: usize) -> Self {
        Self::with_spacing(job, (job.width.max(job.height) / max_cells as f64).max(0.05))
    }

    fn with_spacing(job: &Job, cell: f64) -> Self {
        let nx = (job.width / cell).ceil() as usize + 1;
        let ny = (job.height / cell).ceil() as usize + 1;
        let version = NEXT_MAP.fetch_add(1, Ordering::Relaxed) << 32;
        HeightMap { nx, ny, cell, z: vec![0.0; nx * ny], bottom: -job.thickness as f32, version }
    }

    /// Height of the cutter's lowest point `rho` mm from its axis, when its tip is at `z`.
    fn tool_height(tool: &Tool, z: f64, rho: f64) -> f64 {
        let r = tool.diameter / 2.0;
        match tool.kind {
            ToolKind::EndMill => z,
            ToolKind::BallNose => z + r - (r * r - rho * rho).max(0.0).sqrt(),
            ToolKind::VBit | ToolKind::Drill => z + rho / (tool.angle.max(1.0) / 2.0).to_radians().tan(),
        }
    }

    /// Lower the surface wherever the cutter reaches while its tip moves in a straight line
    /// from `a` to `b`. Every grid point is measured against the exact swept outline (the
    /// nearest point of the move), so walls follow the path smoothly instead of showing the
    /// scallops of separate stamps. A move that changes depth is cut in pieces no longer than
    /// one grid step, each at its own depth.
    fn sweep(&mut self, a: [f64; 3], b: [f64; 3], tool: &Tool) {
        if a[2] >= 0.0 && b[2] >= 0.0 {
            return;
        }
        let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        let pieces = if (b[2] - a[2]).abs() > 1e-9 { (len / self.cell).ceil().max(1.0) as usize } else { 1 };
        let at = |t: f64| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
        for s in 0..pieces {
            self.sweep_piece(at(s as f64 / pieces as f64), at((s + 1) as f64 / pieces as f64), tool);
        }
    }

    fn sweep_piece(&mut self, a: [f64; 3], b: [f64; 3], tool: &Tool) {
        let r = tool.diameter / 2.0;
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let dd = dx * dx + dy * dy;
        let span = |lo: f64, hi: f64, n: usize| {
            let first = ((lo - r) / self.cell).ceil().max(0.0) as usize;
            let last = (((hi + r) / self.cell).floor().max(-1.0) as i64).min(n as i64 - 1);
            (first, last)
        };
        let (i0, i1) = span(a[0].min(b[0]), a[0].max(b[0]), self.nx);
        let (j0, j1) = span(a[1].min(b[1]), a[1].max(b[1]), self.ny);
        for j in j0 as i64..=j1 {
            let py = j as f64 * self.cell;
            for i in i0 as i64..=i1 {
                let px = i as f64 * self.cell;
                // the point of the move nearest to this grid point, and the depth there
                let (t, z) = if dd > 1e-18 {
                    let t = (((px - a[0]) * dx + (py - a[1]) * dy) / dd).clamp(0.0, 1.0);
                    (t, a[2] + (b[2] - a[2]) * t)
                } else {
                    // a plunge: the deepest end counts
                    (0.0, a[2].min(b[2]))
                };
                let (ex, ey) = (px - (a[0] + dx * t), py - (a[1] + dy * t));
                let rho = (ex * ex + ey * ey).sqrt();
                if rho > r {
                    continue;
                }
                let h = (Self::tool_height(tool, z, rho) as f32).max(self.bottom);
                let k = j as usize * self.nx + i as usize;
                if h < self.z[k] {
                    self.z[k] = h;
                }
            }
        }
    }

    /// Carve the first `limit` moves of a toolpath (all when None).
    pub fn carve(&mut self, tp: &Toolpath, limit: Option<usize>) {
        let mut prev: Option<[f64; 3]> = None;
        for m in tp.moves.iter().take(limit.unwrap_or(usize::MAX)) {
            if let Some(p) = prev {
                if !m.rapid {
                    self.sweep(p, m.to, &tp.tool);
                }
            }
            prev = Some(m.to);
        }
        self.version += 1;
    }

    pub fn at(&self, i: usize, j: usize) -> f32 {
        self.z[j.min(self.ny - 1) * self.nx + i.min(self.nx - 1)]
    }

    /// Height at a point in mm (nearest grid cell).
    #[cfg(test)]
    pub fn at_mm(&self, x: f64, y: f64) -> f32 {
        self.at((x / self.cell).round().max(0.0) as usize, (y / self.cell).round().max(0.0) as usize)
    }
}

/// Orbit camera for the 3D view: turn (yaw), tilt (pitch), zoom and pan.
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub pan: egui::Vec2,
    /// Perspective (things further away look smaller) instead of the orthographic view.
    pub perspective: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Camera { yaw: -0.5, pitch: 0.85, zoom: 1.0, pan: egui::Vec2::ZERO, perspective: false }
    }
}

/// How far the eye is from the middle of the block, as a multiple of the job's largest size, in
/// the perspective view (estimated: far enough that the corners don't stretch too much).
pub const PERSPECTIVE_DISTANCE: f32 = 4.0;

/// The orthographic view of the job inside a screen rectangle. One definition for the GPU
/// renderer and for the lines drawn on top, so they always line up.
#[derive(Clone, Copy)]
pub struct Projection {
    rect: Rect,
    /// Screen points per millimetre.
    scale: f32,
    pan: egui::Vec2,
    sy: f32,
    cy: f32,
    sp: f32,
    cp: f32,
    /// Half the job's size, to centre it.
    half: [f32; 3],
    /// Largest depth any point of the job can have, for the depth buffer range.
    depth_range: f32,
    /// The eye's distance from the middle of the block in the perspective view; none when
    /// the view is orthographic.
    eye: Option<f32>,
}

impl Projection {
    pub fn new(rect: Rect, job: &Job, cam: &Camera) -> Self {
        let (w, h, t) = (job.width as f32, job.height as f32, job.thickness as f32);
        let size = w.max(h).max(t);
        let (sy, cy) = cam.yaw.sin_cos();
        let (sp, cp) = cam.pitch.sin_cos();
        Projection {
            rect,
            scale: rect.width().min(rect.height()) / size * 0.75 * cam.zoom,
            pan: cam.pan,
            sy,
            cy,
            sp,
            cp,
            half: [w / 2.0, h / 2.0, t / 2.0],
            depth_range: (w * w + h * h + t * t).sqrt() * 0.6 + 1.0,
            eye: cam.perspective.then_some(size * PERSPECTIVE_DISTANCE),
        }
    }

    /// A job point moved so the middle of the block sits at the origin.
    pub fn centered(&self, p: [f32; 3]) -> [f32; 3] {
        [p[0] - self.half[0], p[1] - self.half[1], p[2] + self.half[2]]
    }

    /// Screen position of a job point (mm, Z up from the material top).
    pub fn to_screen(&self, p: [f32; 3]) -> Pos2 {
        let [x, y, z] = self.centered(p);
        let rx = x * self.cy - y * self.sy;
        let ry = x * self.sy + y * self.cy;
        let up = ry * self.cp + z * self.sp;
        // in the perspective view what is further from the eye is drawn smaller
        let f = match self.eye {
            Some(d) => d / (d + ry * self.sp - z * self.cp),
            None => 1.0,
        };
        let c = self.rect.center() + self.pan;
        Pos2::new(c.x + rx * self.scale * f, c.y - up * self.scale * f)
    }

    /// Column-major matrix taking centred job coordinates to clip space, matching `to_screen`.
    pub fn matrix(&self) -> [f32; 16] {
        let (hw, hh) = (self.rect.width() / 2.0, self.rect.height() / 2.0);
        let s = self.scale;
        let (sy, cy, sp, cp, r) = (self.sy, self.cy, self.sp, self.cp, self.depth_range);
        // rows: screen x, screen y (up), depth (near = −1), w
        let rows = [
            [cy * s / hw, -sy * s / hw, 0.0, self.pan.x / hw],
            [sy * cp * s / hh, cy * cp * s / hh, sp * s / hh, -self.pan.y / hh],
            [sy * sp / r, cy * sp / r, -cp / r, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let mut rows = rows;
        if let Some(d) = self.eye {
            // clip = (x, y, z, w) with w = 1 + depth / d, so that x/w, y/w shrink with distance (the
            // pan is added after the shrinking, as in `to_screen`); z is chosen so the depth
            // buffer spans the same range as in the orthographic view
            let depth = rows[2];
            let (px, py) = (rows[0][3], rows[1][3]);
            for k in 0..3 {
                rows[0][k] += px / d * (depth[k] * r);
                rows[1][k] += py / d * (depth[k] * r);
            }
            rows[2][3] = r / d;
            rows[3] = [depth[0] * r / d, depth[1] * r / d, depth[2] * r / d, 1.0];
        }
        let mut m = [0.0; 16];
        for (r, row) in rows.iter().enumerate() {
            for (c, v) in row.iter().enumerate() {
                m[c * 4 + r] = *v;
            }
        }
        m
    }

    /// A light from the upper left, fixed relative to the viewer, in job coordinates.
    pub fn light(&self) -> [f32; 3] {
        let l = [-0.45f32, 0.65, -0.62]; // right, up, depth (towards the viewer is negative)
        let n = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
        let (lr, lu, ld) = (l[0] / n, l[1] / n, l[2] / n);
        let (sy, cy, sp, cp) = (self.sy, self.cy, self.sp, self.cp);
        // the view rotation's rows are right = (cy, −sy, 0), up = (sy·cp, cy·cp, sp) and
        // depth = (sy·sp, cy·sp, −cp); its transpose brings the light back into job space
        [lr * cy + lu * sy * cp + ld * sy * sp, -lr * sy + lu * cy * cp + ld * cy * sp, lu * sp - ld * cp]
    }
}

/// Toolpath lines drawn over the 3D view: rapid/travel dashed cyan, plunge yellow, cut in strategy colour.
pub fn paint_paths(painter: &egui::Painter, proj: &Projection, paths: &[(&Toolpath, egui::Color32)]) {
    let rapid_col = egui::Color32::from_rgba_premultiplied(0x00, 0xbb, 0xcc, 0x70);
    let plunge_col = egui::Color32::from_rgb(0xe0, 0xb0, 0x20).gamma_multiply(0.55);
    for (tp, col) in paths {
        let mut prev: Option<[f64; 3]> = None;
        for m in &tp.moves {
            let p = proj.to_screen([m.to[0] as f32, m.to[1] as f32, (m.to[2] as f32).min(0.3)]);
            if let Some(pr) = prev {
                let from = proj.to_screen([pr[0] as f32, pr[1] as f32, (pr[2] as f32).min(0.3)]);
                if m.rapid {
                    paint_dashed_3d(painter, from, p, 4.0, 3.0, Stroke::new(0.6_f32, rapid_col));
                } else {
                    let lateral = (m.to[0] - pr[0]).abs() > 1e-6 || (m.to[1] - pr[1]).abs() > 1e-6;
                    let c = if lateral { *col } else { plunge_col };
                    painter.line_segment([from, p], Stroke::new(1.0_f32, c));
                }
            }
            prev = Some(m.to);
        }
    }
}

fn paint_dashed_3d(painter: &egui::Painter, a: Pos2, b: Pos2, dash: f32, gap: f32, stroke: Stroke) {
    let d = b - a;
    let len = d.length();
    if len < 0.5 {
        return;
    }
    let dir = d / len;
    let mut t = 0.0_f32;
    while t < len {
        let end = (t + dash).min(len);
        painter.line_segment([a + dir * t, a + dir * end], stroke);
        t = end + gap;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toolpath::{Move, Strategy};

    fn job() -> Job {
        Job { width: 100.0, height: 60.0, thickness: 18.0, ..Job::default() }
    }

    fn tool(kind: ToolKind, diameter: f64, angle: f64) -> Tool {
        Tool { name: "test".into(), kind, diameter, angle, feed: 1000.0, plunge: 300.0, rpm: 18000, stepdown: 1.0, stepover: 1.0, number: 1 }
    }

    /// A single straight cut along Y = 30 from X = 20 to X = 80 at depth `z`.
    fn cut(tool: Tool, z: f64) -> HeightMap {
        let tp = Toolpath {
            name: "test".into(),
            strategy: Strategy::Drill,
            tool,
            start_depth: 0.0,
            cut_depth: -z,
            start_expr: None,
            cut_expr: None,
            shape_ids: vec![],
            visible: true,
            moves: vec![
                Move { to: [20.0, 30.0, 5.0], rapid: true },
                Move { to: [20.0, 30.0, z], rapid: false },
                Move { to: [80.0, 30.0, z], rapid: false },
                Move { to: [80.0, 30.0, 5.0], rapid: true },
            ],
            warning: None,
        };
        let mut hm = HeightMap::new(&job(), 500);
        hm.carve(&tp, None);
        hm
    }

    #[test]
    fn end_mill_cuts_a_flat_slot_of_its_width() {
        let hm = cut(tool(ToolKind::EndMill, 6.0, 0.0), -3.0);
        assert_eq!(hm.at_mm(50.0, 30.0), -3.0, "slot floor");
        assert_eq!(hm.at_mm(50.0, 32.8), -3.0, "still inside the 3 mm radius");
        assert_eq!(hm.at_mm(50.0, 34.0), 0.0, "beyond the cutter");
        assert_eq!(hm.at_mm(10.0, 30.0), 0.0, "before the start");
        assert_eq!(hm.at_mm(50.0, 50.0), 0.0, "untouched material");
    }

    #[test]
    fn v_bit_cuts_a_v_groove() {
        // a 90° V-bit rises 1 mm for every mm away from its axis
        let hm = cut(tool(ToolKind::VBit, 12.0, 90.0), -4.0);
        for d in [0.0, 1.0, 2.0, 3.0] {
            let z = hm.at_mm(50.0, 30.0 + d);
            assert!((z - (-4.0 + d as f32)).abs() < 0.15, "{d} mm off centre: {z}");
        }
        assert_eq!(hm.at_mm(50.0, 35.0), 0.0, "the groove ends at the surface");
    }

    #[test]
    fn ball_nose_cuts_a_round_bottom() {
        let hm = cut(tool(ToolKind::BallNose, 6.0, 0.0), -3.0);
        assert_eq!(hm.at_mm(50.0, 30.0), -3.0);
        let z = hm.at_mm(50.0, 32.0); // 2 mm off centre: 3 − √(9 − 4) ≈ 0.76 higher
        assert!((z - (-3.0 + 0.764)).abs() < 0.12, "{z}");
    }

    #[test]
    fn nothing_cuts_below_the_bed_and_rapids_cut_nothing() {
        let hm = cut(tool(ToolKind::EndMill, 6.0, 0.0), -30.0);
        assert_eq!(hm.at_mm(50.0, 30.0), -18.0, "stops at the material's bottom");
        assert_eq!(hm.at_mm(20.0, 10.0), 0.0);
    }

    #[test]
    fn a_round_groove_has_round_walls() {
        // a 3 mm cutter round a 20 mm radius circle, as a profile cut sends it (72 segments)
        let (cx, cy, rad) = (50.0, 30.0, 20.0);
        let mut moves = vec![Move { to: [cx + rad, cy, 5.0], rapid: true }, Move { to: [cx + rad, cy, -2.0], rapid: false }];
        for k in 1..=72 {
            let a = k as f64 / 72.0 * std::f64::consts::TAU;
            moves.push(Move { to: [cx + rad * a.cos(), cy + rad * a.sin(), -2.0], rapid: false });
        }
        let tp = Toolpath { name: "ring".into(), strategy: Strategy::Drill, tool: tool(ToolKind::EndMill, 3.0, 0.0), start_depth: 0.0, cut_depth: 2.0, start_expr: None, cut_expr: None, shape_ids: vec![], visible: true, moves, warning: None };
        let mut hm = HeightMap::for_preview(&job(), SIMULATION_QUALITY);
        hm.carve(&tp, None);
        // all the way round, the groove's floor is cut and just past its walls nothing is:
        // the wall stays within one grid step of the true circle, with no scallops
        let slack = hm.cell * 1.5 + 0.05;
        for k in 0..360 {
            let a = (k as f64).to_radians();
            let at = |r: f64| hm.at_mm(cx + r * a.cos(), cy + r * a.sin());
            assert_eq!(at(rad), -2.0, "floor at {k}°");
            assert_eq!(at(rad + 1.5 - slack), -2.0, "inside the outer wall at {k}°");
            assert_eq!(at(rad + 1.5 + slack), 0.0, "outside the outer wall at {k}°");
            assert_eq!(at(rad - 1.5 - slack), 0.0, "inside the inner wall at {k}°");
        }
    }

    #[test]
    fn grid_spacing_follows_the_original() {
        // 300 × 200 mm at 5 points per mm: 1.5 million points, 0.2 mm apart
        assert!((grid_spacing(300.0, 200.0, 5.0) - 0.2).abs() < 1e-12);
        // a tiny board still gets 50 000 points, a huge one at most 16 million
        assert!((grid_spacing(10.0, 10.0, 5.0) - (100.0f64 / 50000.0).sqrt()).abs() < 1e-12);
        assert!((grid_spacing(3000.0, 2000.0, 40.0) - (6.0e6f64 / 16.0e6).sqrt()).abs() < 1e-12);
        // the quality is kept between 0.01 and 40
        assert_eq!(grid_spacing(300.0, 200.0, 100.0), grid_spacing(300.0, 200.0, 40.0));
    }

    #[test]
    fn a_plunge_cuts_at_its_deepest_point() {
        let tp = Toolpath {
            name: "drill".into(),
            strategy: Strategy::Drill,
            tool: tool(ToolKind::EndMill, 4.0, 0.0),
            start_depth: 0.0,
            cut_depth: 5.0,
            start_expr: None,
            cut_expr: None,
            shape_ids: vec![],
            visible: true,
            moves: vec![Move { to: [30.0, 30.0, 2.0], rapid: true }, Move { to: [30.0, 30.0, -5.0], rapid: false }, Move { to: [30.0, 30.0, 2.0], rapid: false }],
            warning: None,
        };
        let mut hm = HeightMap::for_preview(&job(), SIMULATION_QUALITY);
        hm.carve(&tp, None);
        assert_eq!(hm.at_mm(30.0, 30.0), -5.0);
        assert_eq!(hm.at_mm(31.5, 30.0), -5.0);
        assert_eq!(hm.at_mm(32.5, 30.0), 0.0);
    }

    #[test]
    fn projection_matrix_matches_screen_mapping() {
        // the orthographic view and the perspective one: the matrix (for the GPU) and `to_screen`
        // (for the lines drawn over it) must say the same
        for perspective in [false, true] {
            let rect = Rect::from_min_size(Pos2::new(10.0, 20.0), egui::vec2(800.0, 500.0));
            let cam = Camera { yaw: 0.7, pitch: 0.6, zoom: 1.3, pan: egui::vec2(15.0, -8.0), perspective };
            let p = Projection::new(rect, &job(), &cam);
            let m = p.matrix();
            for pt in [[0.0f32, 0.0, 0.0], [100.0, 60.0, -18.0], [37.0, 12.0, -4.0], [100.0, 0.0, 0.0]] {
                let c = p.centered(pt);
                let clip = |r: usize| m[r] * c[0] + m[4 + r] * c[1] + m[8 + r] * c[2] + m[12 + r];
                let w = clip(3);
                let screen = p.to_screen(pt);
                let from_clip = Pos2::new(rect.center().x + clip(0) / w * rect.width() / 2.0, rect.center().y - clip(1) / w * rect.height() / 2.0);
                assert!((screen - from_clip).length() < 1e-2, "perspective {perspective}, {pt:?}: {screen:?} vs {from_clip:?}");
                assert!((clip(2) / w).abs() <= 1.0, "inside the depth range");
            }
        }
    }

    #[test]
    fn perspective_makes_what_is_far_smaller() {
        let rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 500.0));
        let cam = Camera { yaw: 0.0, pitch: 0.6, zoom: 1.0, pan: egui::Vec2::ZERO, perspective: true };
        let ortho = Projection::new(rect, &job(), &Camera { perspective: false, ..Camera { yaw: 0.0, pitch: 0.6, zoom: 1.0, pan: egui::Vec2::ZERO, perspective: false } });
        let persp = Projection::new(rect, &job(), &cam);
        // the width of the front edge (y = 0) and of the back edge (y = 60): equal when flat, the
        // back one shorter in perspective
        let width = |p: &Projection, y: f32| (p.to_screen([100.0, y, 0.0]) - p.to_screen([0.0, y, 0.0])).x;
        assert!((width(&ortho, 0.0) - width(&ortho, 60.0)).abs() < 1e-3);
        assert!(width(&persp, 60.0) < width(&persp, 0.0) - 1.0, "far edge looks shorter: {} vs {}", width(&persp, 60.0), width(&persp, 0.0));
    }
}
