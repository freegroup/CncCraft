//! A real 3D cutter drawn at the current simulation position: a cylinder for the shaft, plus
//! the flute body whose shape depends on the tool kind (flat end-mill, ball-nose hemisphere,
//! V-bit / drill cone). The mesh is built on the CPU each frame the position changes and
//! handed to the [`Renderer`](crate::view3d::Renderer) which draws it in the same GL pass as
//! the board, with shared depth testing.

use super::{View3dContext, ViewDecoration};
use crate::tools::ToolKind;
use crate::view3d::Mesh;

const SIDES: usize = 24;

const STEEL: [f32; 3] = [0.55, 0.55, 0.58];
const SHAFT: [f32; 3] = [0.68, 0.68, 0.70];
const COLLET: [f32; 3] = [0.42, 0.42, 0.44];

pub struct CutterOverlay {
    last_key: u64,
}

impl Default for CutterOverlay {
    fn default() -> Self {
        CutterOverlay { last_key: u64::MAX }
    }
}

impl CutterOverlay {
    fn tool_key(pos: [f64; 3], diameter: f64) -> u64 {
        let a = (pos[0] * 1000.0) as i64 as u64;
        let b = (pos[1] * 1000.0) as i64 as u64;
        let c = (pos[2] * 1000.0) as i64 as u64;
        let d = (diameter * 1000.0) as u64;
        a.wrapping_mul(2654435761) ^ b.wrapping_mul(40503) ^ c.wrapping_mul(73856093) ^ d
    }
}

impl ViewDecoration for CutterOverlay {
    fn name(&self) -> &'static str {
        "Fräser"
    }

    fn show(&mut self, _ui: &mut egui::Ui, view: &mut View3dContext) -> Option<String> {
        let Some((pos, tool)) = view.cutter else {
            view.renderer.clear_tool();
            self.last_key = u64::MAX;
            return None;
        };

        let key = Self::tool_key(pos, tool.diameter);
        if key == self.last_key {
            return None;
        }
        self.last_key = key;

        let r = (tool.diameter / 2.0) as f32;
        let tip = [pos[0] as f32, pos[1] as f32, pos[2] as f32];
        let shaft_len = (view.thickness * 1.5).max(15.0);
        let shaft_r = (r * 0.35).max(1.0);
        let flute_h = (r * 3.0).max(3.0);
        let collet_h: f32 = 2.0_f32.min(shaft_len * 0.1);
        let collet_r = shaft_r * 1.5;
        let proj = view.proj;

        let mut m = Mesh::default();
        m.version = key;

        match tool.kind {
            ToolKind::EndMill => {
                cylinder(&mut m, proj, tip, r, flute_h, STEEL);
                disk(&mut m, proj, tip, r, [0.0, 0.0, -1.0], STEEL);
            }
            ToolKind::BallNose => {
                let ball_top = [tip[0], tip[1], tip[2] + r];
                cylinder(&mut m, proj, ball_top, r, flute_h - r, STEEL);
                hemisphere(&mut m, proj, tip, r, STEEL);
            }
            ToolKind::VBit | ToolKind::Drill => {
                let cone_top = [tip[0], tip[1], tip[2] + flute_h];
                cone(&mut m, proj, tip, cone_top, r, STEEL);
            }
        }

        let shaft_bottom = [tip[0], tip[1], tip[2] + flute_h];
        let shaft_top = [tip[0], tip[1], tip[2] + flute_h + shaft_len];
        cylinder(&mut m, proj, shaft_bottom, shaft_r, shaft_len, SHAFT);
        disk(&mut m, proj, shaft_top, shaft_r, [0.0, 0.0, 1.0], SHAFT);

        let collet_bottom = [tip[0], tip[1], tip[2] + flute_h - 0.5];
        cylinder(&mut m, proj, collet_bottom, collet_r, collet_h + 0.5, COLLET);
        disk(&mut m, proj, collet_bottom, collet_r, [0.0, 0.0, -1.0], COLLET);
        disk(&mut m, proj, [collet_bottom[0], collet_bottom[1], collet_bottom[2] + collet_h + 0.5], collet_r, [0.0, 0.0, 1.0], COLLET);

        view.renderer.update_tool(m);
        None
    }
}

fn cylinder(m: &mut Mesh, proj: &crate::sim::Projection, base: [f32; 3], r: f32, h: f32, col: [f32; 3]) {
    let first = (m.vertices.len() / 9) as u32;
    for i in 0..SIDES {
        let a = std::f32::consts::TAU * i as f32 / SIDES as f32;
        let (ca, sa) = (a.cos(), a.sin());
        let nx = ca;
        let ny = sa;
        let n = [nx, ny, 0.0];
        let bot = proj.centered([base[0] + ca * r, base[1] + sa * r, base[2]]);
        let top = proj.centered([base[0] + ca * r, base[1] + sa * r, base[2] + h]);
        m.vertex(bot, n, col);
        m.vertex(top, n, col);
    }
    for i in 0..SIDES as u32 {
        let j = (i + 1) % SIDES as u32;
        let (b0, b1) = (first + i * 2, first + i * 2 + 1);
        let (c0, c1) = (first + j * 2, first + j * 2 + 1);
        m.indices.extend_from_slice(&[b0, c0, c1, b0, c1, b1]);
    }
}

fn disk(m: &mut Mesh, proj: &crate::sim::Projection, center: [f32; 3], r: f32, normal: [f32; 3], col: [f32; 3]) {
    let ci = m.vertex(proj.centered(center), normal, col);
    let first = (m.vertices.len() / 9) as u32;
    for i in 0..SIDES {
        let a = std::f32::consts::TAU * i as f32 / SIDES as f32;
        let p = proj.centered([center[0] + a.cos() * r, center[1] + a.sin() * r, center[2]]);
        m.vertex(p, normal, col);
    }
    for i in 0..SIDES as u32 {
        let j = (i + 1) % SIDES as u32;
        if normal[2] > 0.0 {
            m.indices.extend_from_slice(&[ci, first + i, first + j]);
        } else {
            m.indices.extend_from_slice(&[ci, first + j, first + i]);
        }
    }
}

fn hemisphere(m: &mut Mesh, proj: &crate::sim::Projection, tip: [f32; 3], r: f32, col: [f32; 3]) {
    let rings = 8;
    let first = (m.vertices.len() / 9) as u32;
    // bottom pole
    m.vertex(proj.centered(tip), [0.0, 0.0, -1.0], col);
    for ring in 1..=rings {
        let phi = std::f32::consts::FRAC_PI_2 * ring as f32 / rings as f32;
        let (sp, cp) = phi.sin_cos();
        let z = tip[2] + r * (1.0 - cp);
        let rr = r * sp;
        for i in 0..SIDES {
            let a = std::f32::consts::TAU * i as f32 / SIDES as f32;
            let (ca, sa) = (a.cos(), a.sin());
            let p = proj.centered([tip[0] + ca * rr, tip[1] + sa * rr, z]);
            let n = normalize([ca * sp, sa * sp, -cp]);
            m.vertex(p, n, col);
        }
    }
    // bottom fan
    for i in 0..SIDES as u32 {
        let j = (i + 1) % SIDES as u32;
        m.indices.extend_from_slice(&[first, first + 1 + j, first + 1 + i]);
    }
    // ring strips
    for ring in 0..rings - 1 {
        let r0 = first + 1 + (ring as u32) * SIDES as u32;
        let r1 = r0 + SIDES as u32;
        for i in 0..SIDES as u32 {
            let j = (i + 1) % SIDES as u32;
            m.indices.extend_from_slice(&[r0 + i, r0 + j, r1 + j, r0 + i, r1 + j, r1 + i]);
        }
    }
}

fn cone(m: &mut Mesh, proj: &crate::sim::Projection, tip: [f32; 3], top: [f32; 3], r: f32, col: [f32; 3]) {
    let h = top[2] - tip[2];
    let slope = h / (r * r + h * h).sqrt();
    let ti = m.vertex(proj.centered(tip), [0.0, 0.0, -1.0], col);
    let first = (m.vertices.len() / 9) as u32;
    for i in 0..SIDES {
        let a = std::f32::consts::TAU * i as f32 / SIDES as f32;
        let (ca, sa) = (a.cos(), a.sin());
        let p = proj.centered([top[0] + ca * r, top[1] + sa * r, top[2]]);
        let n = normalize([ca, sa, slope]);
        m.vertex(p, n, col);
    }
    for i in 0..SIDES as u32 {
        let j = (i + 1) % SIDES as u32;
        m.indices.extend_from_slice(&[ti, first + j, first + i]);
    }
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-9);
    [v[0] / l, v[1] / l, v[2] / l]
}
