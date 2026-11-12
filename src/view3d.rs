//! The 3D preview, drawn on the graphics card (OpenGL through eframe) with a depth buffer:
//! the carved height map at full resolution with smooth shading, side walls that follow the
//! carved edges, the bottom face, and cut-through areas shown as cut out.

use crate::sim::{HeightMap, Projection};
use eframe::egui_glow::{self, ShaderVersion};
use eframe::glow::{self, HasContext};
use std::sync::{Arc, Mutex};

/// Uncut top of the board.
const WOOD: [f32; 3] = [0.86, 0.71, 0.50];
/// Freshly cut surfaces are a little darker.
const CUT: [f32; 3] = [0.78, 0.61, 0.40];
/// End grain on the sides.
const SIDE: [f32; 3] = [0.74, 0.58, 0.38];
/// Where the cutter went right through.
const THROUGH: [f32; 3] = [0.80, 0.82, 0.85];

/// Floats per vertex: position (3), normal (3), colour (3).
const STRIDE: usize = 9;

/// Triangles ready for the GPU, built on the CPU from a height map.
#[derive(Default)]
pub struct Mesh {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    /// The height map version this was built from.
    pub version: u64,
}

impl Mesh {
    fn vertex(&mut self, p: [f32; 3], n: [f32; 3], c: [f32; 3]) -> u32 {
        let i = (self.vertices.len() / STRIDE) as u32;
        self.vertices.extend_from_slice(&[p[0], p[1], p[2], n[0], n[1], n[2], c[0], c[1], c[2]]);
        i
    }

    fn quad(&mut self, a: u32, b: u32, c: u32, d: u32) {
        self.indices.extend_from_slice(&[a, b, c, a, c, d]);
    }

    /// Build the board from the height map, in centred coordinates (see [`Projection::centered`]).
    pub fn from_heightmap(hm: &HeightMap, proj: &Projection) -> Mesh {
        let mut m = Mesh { version: hm.version, ..Mesh::default() };
        let (nx, ny, cell) = (hm.nx, hm.ny, hm.cell as f32);
        let x_of = |i: usize| (i as f32 * cell).min((nx - 1) as f32 * cell);
        let y_of = |j: usize| (j as f32 * cell).min((ny - 1) as f32 * cell);
        let through = hm.bottom + 0.01;

        // top surface: one vertex per grid point, normal from the neighbouring heights
        m.vertices.reserve(nx * ny * STRIDE + (nx + ny) * 4 * STRIDE);
        for j in 0..ny {
            for i in 0..nx {
                let z = hm.at(i, j);
                let dzdx = (hm.at(i + 1, j) - hm.at(i.saturating_sub(1), j)) / (cell * if i == 0 || i == nx - 1 { 1.0 } else { 2.0 });
                let dzdy = (hm.at(i, j + 1) - hm.at(i, j.saturating_sub(1))) / (cell * if j == 0 || j == ny - 1 { 1.0 } else { 2.0 });
                let n = normalize([-dzdx, -dzdy, 1.0]);
                let colour = if z <= through {
                    THROUGH
                } else if z < -0.01 {
                    CUT
                } else {
                    WOOD
                };
                m.vertex(proj.centered([x_of(i), y_of(j), z]), n, colour);
            }
        }
        for j in 0..ny - 1 {
            for i in 0..nx - 1 {
                let a = (j * nx + i) as u32;
                let (b, c, d) = (a + 1, a + 1 + nx as u32, a + nx as u32);
                // split each square along the diagonal whose corners are closest in height, so
                // a wall running diagonally across the grid stays a straight edge instead of a
                // zigzag of alternating triangles
                let (za, zb, zc, zd) = (hm.at(i, j), hm.at(i + 1, j), hm.at(i + 1, j + 1), hm.at(i, j + 1));
                if (za - zc).abs() <= (zb - zd).abs() {
                    m.indices.extend_from_slice(&[a, b, c, a, c, d]);
                } else {
                    m.indices.extend_from_slice(&[a, b, d, b, c, d]);
                }
            }
        }

        // side walls from the (possibly carved) edge down to the bottom, and the bottom itself
        let bottom = hm.bottom;
        let edges: [(Box<dyn Fn(usize) -> (usize, usize)>, usize, [f32; 3]); 4] = [
            (Box::new(|k| (k, 0)), nx, [0.0, -1.0, 0.0]),
            (Box::new(move |k| (nx - 1, k)), ny, [1.0, 0.0, 0.0]),
            (Box::new(move |k| (k, ny - 1)), nx, [0.0, 1.0, 0.0]),
            (Box::new(|k| (0, k)), ny, [-1.0, 0.0, 0.0]),
        ];
        for (at, count, n) in edges {
            let mut prev: Option<(u32, u32)> = None;
            for k in 0..count {
                let (i, j) = at(k);
                let top = hm.at(i, j);
                let (x, y) = (x_of(i), y_of(j));
                let t = m.vertex(proj.centered([x, y, top]), n, SIDE);
                let b = m.vertex(proj.centered([x, y, bottom]), n, SIDE);
                if let Some((pt, pb)) = prev {
                    m.quad(pt, t, b, pb);
                }
                prev = Some((t, b));
            }
        }
        let (w, h) = (x_of(nx - 1), y_of(ny - 1));
        let down = [0.0, 0.0, -1.0];
        let c = [m.vertex(proj.centered([0.0, 0.0, bottom]), down, SIDE), m.vertex(proj.centered([w, 0.0, bottom]), down, SIDE), m.vertex(proj.centered([w, h, bottom]), down, SIDE), m.vertex(proj.centered([0.0, h, bottom]), down, SIDE)];
        m.quad(c[0], c[3], c[2], c[1]);
        m
    }
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-9);
    [v[0] / l, v[1] / l, v[2] / l]
}

/// GPU objects, made on first use inside the paint callback.
struct Gpu {
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
    count: i32,
    version: Option<u64>,
}

/// Shared between the app (which builds the mesh) and the paint callback (which draws it).
#[derive(Clone, Default)]
pub struct Renderer {
    mesh: Arc<Mutex<Mesh>>,
    gpu: Arc<Mutex<Option<Gpu>>>,
}

const VERTEX: &str = r#"
uniform mat4 u_mvp;
IN vec3 a_pos;
IN vec3 a_normal;
IN vec3 a_color;
OUT vec3 v_normal;
OUT vec3 v_color;
void main() {
    v_normal = a_normal;
    v_color = a_color;
    gl_Position = u_mvp * vec4(a_pos, 1.0);
}
"#;

const FRAGMENT: &str = r#"
uniform vec3 u_light;
IN vec3 v_normal;
IN vec3 v_color;
void main() {
    vec3 n = normalize(v_normal);
    float diffuse = max(dot(n, u_light), 0.0);
    float sky = 0.5 + 0.5 * n.z;
    vec3 c = v_color * (0.30 + 0.62 * diffuse + 0.12 * sky);
    FRAG_COLOR = vec4(min(c, vec3(1.0)), 1.0);
}
"#;

fn shader_source(version: ShaderVersion, body: &str, vertex: bool) -> String {
    let new = version.is_new_shader_interface();
    let mut s = String::from(version.version_declaration());
    s.push_str("\n#ifdef GL_ES\nprecision highp float;\n#endif\n");
    if new {
        s.push_str(if vertex { "#define IN in\n#define OUT out\n" } else { "#define IN in\nout vec4 frag_color;\n#define FRAG_COLOR frag_color\n" });
    } else {
        s.push_str(if vertex { "#define IN attribute\n#define OUT varying\n" } else { "#define IN varying\n#define FRAG_COLOR gl_FragColor\n" });
    }
    s.push_str(body);
    s
}

unsafe fn compile(gl: &glow::Context) -> Result<Gpu, String> {
    let version = ShaderVersion::get(gl);
    let program = gl.create_program()?;
    let mut shaders = Vec::new();
    for (kind, body, vertex) in [(glow::VERTEX_SHADER, VERTEX, true), (glow::FRAGMENT_SHADER, FRAGMENT, false)] {
        let sh = gl.create_shader(kind)?;
        gl.shader_source(sh, &shader_source(version, body, vertex));
        gl.compile_shader(sh);
        if !gl.get_shader_compile_status(sh) {
            return Err(gl.get_shader_info_log(sh));
        }
        gl.attach_shader(program, sh);
        shaders.push(sh);
    }
    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        return Err(gl.get_program_info_log(program));
    }
    for sh in shaders {
        gl.detach_shader(program, sh);
        gl.delete_shader(sh);
    }
    let vao = gl.create_vertex_array()?;
    let vbo = gl.create_buffer()?;
    let ebo = gl.create_buffer()?;
    gl.bind_vertex_array(Some(vao));
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
    let stride = (STRIDE * 4) as i32;
    for (name, offset) in [("a_pos", 0), ("a_normal", 12), ("a_color", 24)] {
        if let Some(loc) = gl.get_attrib_location(program, name) {
            gl.enable_vertex_attrib_array(loc);
            gl.vertex_attrib_pointer_f32(loc, 3, glow::FLOAT, false, stride, offset);
        }
    }
    gl.bind_vertex_array(None);
    Ok(Gpu { program, vao, vbo, ebo, count: 0, version: None })
}

impl Renderer {
    /// Rebuild the mesh if the height map changed since the last time.
    pub fn update(&self, hm: &HeightMap, proj: &Projection, layout_key: u64) {
        let key = hm.version ^ layout_key.rotate_left(17);
        let mut mesh = self.mesh.lock().unwrap();
        if mesh.version != key {
            *mesh = Mesh::from_heightmap(hm, proj);
            mesh.version = key;
        }
    }

    /// A paint callback that draws the board into `rect` with the given projection.
    pub fn callback(&self, rect: egui::Rect, proj: &Projection) -> egui::PaintCallback {
        let (mesh, gpu) = (self.mesh.clone(), self.gpu.clone());
        let (mvp, light) = (proj.matrix(), proj.light());
        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            let gl = painter.gl();
            let mut slot = gpu.lock().unwrap();
            unsafe {
                if slot.is_none() {
                    match compile(gl) {
                        Ok(g) => *slot = Some(g),
                        Err(e) => {
                            eprintln!("3D-Vorschau: Shader-Fehler: {e}");
                            return;
                        }
                    }
                }
                let g = slot.as_mut().unwrap();
                let mesh = mesh.lock().unwrap();
                gl.bind_vertex_array(Some(g.vao));
                if g.version != Some(mesh.version) {
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(g.vbo));
                    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck_f32(&mesh.vertices), glow::DYNAMIC_DRAW);
                    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(g.ebo));
                    gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, bytemuck_u32(&mesh.indices), glow::DYNAMIC_DRAW);
                    g.count = mesh.indices.len() as i32;
                    g.version = Some(mesh.version);
                }
                gl.use_program(Some(g.program));
                gl.uniform_matrix_4_f32_slice(gl.get_uniform_location(g.program, "u_mvp").as_ref(), false, &mvp);
                gl.uniform_3_f32(gl.get_uniform_location(g.program, "u_light").as_ref(), light[0], light[1], light[2]);
                // our own depth test, cleared inside this view only (egui keeps the scissor set)
                gl.enable(glow::DEPTH_TEST);
                gl.depth_func(glow::LEQUAL);
                gl.depth_mask(true);
                gl.clear_depth_f32(1.0);
                gl.clear(glow::DEPTH_BUFFER_BIT);
                gl.disable(glow::CULL_FACE);
                gl.disable(glow::BLEND);
                gl.draw_elements(glow::TRIANGLES, g.count, glow::UNSIGNED_INT, 0);
                gl.disable(glow::DEPTH_TEST);
                gl.enable(glow::BLEND);
                gl.bind_vertex_array(None);
            }
        });
        egui::PaintCallback { rect, callback: Arc::new(cb) }
    }
}

fn bytemuck_f32(v: &[f32]) -> &[u8] {
    // SAFETY: f32 has no padding and any byte pattern of the slice is valid to read as u8
    unsafe { std::slice::from_raw_parts(v.as_ptr().cast::<u8>(), std::mem::size_of_val(v)) }
}

fn bytemuck_u32(v: &[u32]) -> &[u8] {
    // SAFETY: as above
    unsafe { std::slice::from_raw_parts(v.as_ptr().cast::<u8>(), std::mem::size_of_val(v)) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::Camera;
    use crate::toolpath::Job;

    #[test]
    fn mesh_has_top_walls_and_bottom() {
        let job = Job { width: 40.0, height: 20.0, thickness: 10.0, ..Job::default() };
        let hm = HeightMap::new(&job, 40);
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 300.0));
        let mesh = Mesh::from_heightmap(&hm, &Projection::new(rect, &job, &Camera::default()));
        let verts = mesh.vertices.len() / STRIDE;
        let top = hm.nx * hm.ny;
        assert_eq!(verts, top + 2 * (2 * hm.nx + 2 * hm.ny) + 4);
        let tris = mesh.indices.len() / 3;
        assert_eq!(tris, 2 * (hm.nx - 1) * (hm.ny - 1) + 2 * (2 * (hm.nx - 1) + 2 * (hm.ny - 1)) + 2);
        assert!(mesh.indices.iter().all(|&i| (i as usize) < verts));
        // the block is centred: top at +t/2, bottom at −t/2
        let zs: Vec<f32> = mesh.vertices.chunks(STRIDE).map(|v| v[2]).collect();
        assert_eq!(zs.iter().cloned().fold(f32::MIN, f32::max), 5.0);
        assert_eq!(zs.iter().cloned().fold(f32::MAX, f32::min), -5.0);
    }
}
