//! The 3D preview, drawn on the graphics card (OpenGL through eframe) with a depth buffer:
//! the carved height map at full resolution with smooth shading, side walls that follow the
//! carved edges, the bottom face, and cut-through areas shown as cut out. The board wears the
//! job material's look (see `materials::Look`): one picture from above, one for the long sides and
//! one for the ends, laid onto each face by the way it faces, so carved walls show the grain or
//! layers the material has inside.

use crate::sim::{HeightMap, Projection};
use eframe::egui_glow::{self, ShaderVersion};
use eframe::glow::{self, HasContext};
use std::sync::{Arc, Mutex};

/// The material's own colour, untouched.
const UNCUT: [f32; 3] = [1.0, 1.0, 1.0];
/// Freshly cut floors are a little darker than the board's top.
const CUT: [f32; 3] = [0.9, 0.9, 0.9];
/// Where the cutter went right through: the spoil board below, not the material.
const THROUGH: [f32; 3] = [0.80, 0.82, 0.85];

/// Floats per vertex: position (3), normal (3), colour (3), whether the material's look is laid on
/// (1; the colour then tints it) or the colour is all (0).
const STRIDE: usize = 10;
/// The pictures of a look are baked this many pixels square.
const LOOK_SIZE: usize = 512;

/// Triangles ready for the GPU, built on the CPU from a height map.
#[derive(Default)]
pub struct Mesh {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    /// The height map version this was built from.
    pub version: u64,
}

impl Mesh {
    /// A point in a plain colour (the cutter, the spoil board).
    pub fn vertex(&mut self, p: [f32; 3], n: [f32; 3], c: [f32; 3]) -> u32 {
        self.push(p, n, c, 0.0)
    }

    /// A point of the material: its look, tinted by `tint`.
    fn surface(&mut self, p: [f32; 3], n: [f32; 3], tint: [f32; 3]) -> u32 {
        self.push(p, n, tint, 1.0)
    }

    fn push(&mut self, p: [f32; 3], n: [f32; 3], c: [f32; 3], textured: f32) -> u32 {
        let i = (self.vertices.len() / STRIDE) as u32;
        self.vertices.extend_from_slice(&[p[0], p[1], p[2], n[0], n[1], n[2], c[0], c[1], c[2], textured]);
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
                let p = proj.centered([x_of(i), y_of(j), z]);
                if z <= through {
                    m.vertex(p, n, THROUGH);
                } else {
                    m.surface(p, n, if z < -0.01 { CUT } else { UNCUT });
                }
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
                let t = m.surface(proj.centered([x, y, top]), n, UNCUT);
                let b = m.surface(proj.centered([x, y, bottom]), n, UNCUT);
                if let Some((pt, pb)) = prev {
                    m.quad(pt, t, b, pb);
                }
                prev = Some((t, b));
            }
        }
        let (w, h) = (x_of(nx - 1), y_of(ny - 1));
        let down = [0.0, 0.0, -1.0];
        let c = [m.surface(proj.centered([0.0, 0.0, bottom]), down, UNCUT), m.surface(proj.centered([w, 0.0, bottom]), down, UNCUT), m.surface(proj.centered([w, h, bottom]), down, UNCUT), m.surface(proj.centered([0.0, h, bottom]), down, UNCUT)];
        m.quad(c[0], c[3], c[2], c[1]);
        m
    }
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-9);
    [v[0] / l, v[1] / l, v[2] / l]
}

/// The job material's look, baked into pixels for the graphics card.
struct Surface {
    /// The material key the look belongs to (none before the first one is set).
    key: Option<String>,
    /// Counts up with every new look, so the card's copy is replaced.
    version: u64,
    size: usize,
    /// RGBA pixels: from above, long sides, ends.
    pictures: [Vec<u8>; 3],
    shine: f32,
    tile: f32,
}

impl Default for Surface {
    fn default() -> Self {
        Surface { key: None, version: 0, size: 1, pictures: [vec![255; 4], vec![255; 4], vec![255; 4]], shine: 0.0, tile: 100.0 }
    }
}

/// GPU objects, made on first use inside the paint callback.
struct Gpu {
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
    count: i32,
    version: Option<u64>,
    tool_vbo: glow::Buffer,
    tool_ebo: glow::Buffer,
    tool_count: i32,
    tool_version: Option<u64>,
    /// The three pictures of the look, and which version of it they hold.
    textures: [glow::Texture; 3],
    surface_version: Option<u64>,
}

/// Shared between the app (which builds the mesh) and the paint callback (which draws it).
#[derive(Clone, Default)]
pub struct Renderer {
    mesh: Arc<Mutex<Mesh>>,
    tool_mesh: Arc<Mutex<Mesh>>,
    surface: Arc<Mutex<Surface>>,
    gpu: Arc<Mutex<Option<Gpu>>>,
}

const VERTEX: &str = r#"
uniform mat4 u_mvp;
IN vec3 a_pos;
IN vec3 a_normal;
IN vec3 a_color;
IN float a_textured;
OUT vec3 v_pos;
OUT vec3 v_normal;
OUT vec3 v_color;
OUT float v_textured;
void main() {
    v_pos = a_pos;
    v_normal = a_normal;
    v_color = a_color;
    v_textured = a_textured;
    gl_Position = u_mvp * vec4(a_pos, 1.0);
}
"#;

// the look's picture is chosen by the way the face looks: up or down the top picture, along X the
// long-side picture, along Y the end picture; v runs down into the material
const FRAGMENT: &str = r#"
uniform vec3 u_light;
uniform vec3 u_eye;
uniform sampler2D u_top;
uniform sampler2D u_edge;
uniform sampler2D u_end;
uniform float u_tile;
uniform float u_shine;
IN vec3 v_pos;
IN vec3 v_normal;
IN vec3 v_color;
IN float v_textured;
void main() {
    vec3 n = normalize(v_normal);
    vec3 base = v_color;
    float textured = step(0.5, v_textured);
    if (textured > 0.5) {
        vec3 a = abs(n);
        vec3 t;
        if (a.z >= a.x && a.z >= a.y) {
            t = TEX(u_top, v_pos.xy / u_tile).rgb;
        } else if (a.y >= a.x) {
            t = TEX(u_edge, vec2(v_pos.x, -v_pos.z) / u_tile).rgb;
        } else {
            t = TEX(u_end, vec2(v_pos.y, -v_pos.z) / u_tile).rgb;
        }
        // the original's relief shader takes the picture a little towards grey (0x1009b6454)
        base = (0.85 * t + 0.15 * 0.5) * v_color;
    }
    // the original's lighting (0x1009b6454): ambient 0.35, diffuse 0.65, a broad highlight of
    // strength 0.5 with shininess 2.5; its light's intensity isn't known, 0.8 is estimated
    float diffuse = 0.65 * max(dot(n, u_light), 0.0);
    float broad = 0.5 * pow(max(dot(u_eye, reflect(-u_light, n)), 0.0), 2.5);
    vec3 c = base * (0.35 + diffuse + broad) * 0.8;
    // CncCraft's own: polished materials get a tight white highlight on top
    vec3 h = normalize(u_light + u_eye);
    c += vec3(pow(max(dot(n, h), 0.0), mix(6.0, 48.0, u_shine)) * u_shine * 0.45 * textured);
    FRAG_COLOR = vec4(min(c, vec3(1.0)), 1.0);
}
"#;

fn shader_source(version: ShaderVersion, body: &str, vertex: bool) -> String {
    let new = version.is_new_shader_interface();
    let mut s = String::from(version.version_declaration());
    s.push_str("\n#ifdef GL_ES\nprecision highp float;\n#endif\n");
    if new {
        s.push_str(if vertex { "#define IN in\n#define OUT out\n" } else { "#define IN in\nout vec4 frag_color;\n#define FRAG_COLOR frag_color\n#define TEX texture\n" });
    } else {
        s.push_str(if vertex { "#define IN attribute\n#define OUT varying\n" } else { "#define IN varying\n#define FRAG_COLOR gl_FragColor\n#define TEX texture2D\n" });
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
    for (name, size, offset) in ATTRIBUTES {
        if let Some(loc) = gl.get_attrib_location(program, name) {
            gl.enable_vertex_attrib_array(loc);
            gl.vertex_attrib_pointer_f32(loc, size, glow::FLOAT, false, (STRIDE * 4) as i32, offset);
        }
    }
    gl.bind_vertex_array(None);
    let tool_vbo = gl.create_buffer()?;
    let tool_ebo = gl.create_buffer()?;
    let textures = [gl.create_texture()?, gl.create_texture()?, gl.create_texture()?];
    Ok(Gpu { program, vao, vbo, ebo, count: 0, version: None, tool_vbo, tool_ebo, tool_count: 0, tool_version: None, textures, surface_version: None })
}

/// The vertex attributes: name, floats, byte offset.
const ATTRIBUTES: [(&str, i32, i32); 4] = [("a_pos", 3, 0), ("a_normal", 3, 12), ("a_color", 3, 24), ("a_textured", 1, 36)];

unsafe fn point_attributes(gl: &glow::Context, program: glow::Program) {
    for (name, size, offset) in ATTRIBUTES {
        if let Some(loc) = gl.get_attrib_location(program, name) {
            gl.vertex_attrib_pointer_f32(loc, size, glow::FLOAT, false, (STRIDE * 4) as i32, offset);
        }
    }
}

/// Give the card the look's three pictures (repeating, smoothed when seen from afar).
unsafe fn upload_surface(gl: &glow::Context, textures: &[glow::Texture; 3], s: &Surface) {
    for (tex, pixels) in textures.iter().zip(&s.pictures) {
        gl.bind_texture(glow::TEXTURE_2D, Some(*tex));
        gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA as i32, s.size as i32, s.size as i32, 0, glow::RGBA, glow::UNSIGNED_BYTE, Some(pixels));
        gl.generate_mipmap(glow::TEXTURE_2D);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR_MIPMAP_LINEAR as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
    }
    gl.bind_texture(glow::TEXTURE_2D, None);
}

impl Renderer {
    /// Dress the board in the look `key`; the look is asked for and baked only when it changes.
    pub fn set_look(&self, key: &str, look: impl FnOnce() -> crate::materials::Look) {
        let mut s = self.surface.lock().unwrap();
        if s.key.as_deref() == Some(key) {
            return;
        }
        let look = look();
        let pictures = [look.top, look.edge, look.end].map(|p| crate::materials::bake(p, LOOK_SIZE));
        *s = Surface { key: Some(key.to_string()), version: s.version + 1, size: LOOK_SIZE, pictures, shine: look.shine, tile: look.tile };
    }

    /// Rebuild the mesh if the height map changed since the last time.
    pub fn update(&self, hm: &HeightMap, proj: &Projection, layout_key: u64) {
        let key = hm.version ^ layout_key.rotate_left(17);
        let mut mesh = self.mesh.lock().unwrap();
        if mesh.version != key {
            *mesh = Mesh::from_heightmap(hm, proj);
            mesh.version = key;
        }
    }

    /// Replace the tool mesh (built by the cutter decoration).
    pub fn update_tool(&self, m: Mesh) {
        *self.tool_mesh.lock().unwrap() = m;
    }

    /// Clear the tool mesh (no cutter to show).
    pub fn clear_tool(&self) {
        let mut tm = self.tool_mesh.lock().unwrap();
        if !tm.indices.is_empty() {
            *tm = Mesh::default();
        }
    }

    /// A paint callback that draws the board and (optionally) the tool into `rect`.
    pub fn callback(&self, rect: egui::Rect, proj: &Projection) -> egui::PaintCallback {
        let (mesh, tool_mesh, surface, gpu) = (self.mesh.clone(), self.tool_mesh.clone(), self.surface.clone(), self.gpu.clone());
        let (mvp, light, eye) = (proj.matrix(), proj.light(), proj.eye_dir());
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
                let s = surface.lock().unwrap();
                if g.surface_version != Some(s.version) {
                    upload_surface(gl, &g.textures, &s);
                    g.surface_version = Some(s.version);
                }
                gl.use_program(Some(g.program));
                let uniform = |name: &str| gl.get_uniform_location(g.program, name);
                gl.uniform_matrix_4_f32_slice(uniform("u_mvp").as_ref(), false, &mvp);
                gl.uniform_3_f32(uniform("u_light").as_ref(), light[0], light[1], light[2]);
                gl.uniform_3_f32(uniform("u_eye").as_ref(), eye[0], eye[1], eye[2]);
                gl.uniform_1_f32(uniform("u_tile").as_ref(), s.tile);
                gl.uniform_1_f32(uniform("u_shine").as_ref(), s.shine);
                for (unit, (tex, name)) in g.textures.iter().zip(["u_top", "u_edge", "u_end"]).enumerate() {
                    gl.active_texture(glow::TEXTURE0 + unit as u32);
                    gl.bind_texture(glow::TEXTURE_2D, Some(*tex));
                    gl.uniform_1_i32(uniform(name).as_ref(), unit as i32);
                }
                drop(s);
                // our own depth test, cleared inside this view only (egui keeps the scissor set)
                gl.enable(glow::DEPTH_TEST);
                gl.depth_func(glow::LEQUAL);
                gl.depth_mask(true);
                gl.clear_depth_f32(1.0);
                gl.clear(glow::DEPTH_BUFFER_BIT);
                gl.disable(glow::CULL_FACE);
                gl.disable(glow::BLEND);
                gl.draw_elements(glow::TRIANGLES, g.count, glow::UNSIGNED_INT, 0);
                // tool mesh (same shader, same depth buffer, drawn on top of the board)
                let tm = tool_mesh.lock().unwrap();
                if !tm.indices.is_empty() {
                    if g.tool_version != Some(tm.version) {
                        gl.bind_buffer(glow::ARRAY_BUFFER, Some(g.tool_vbo));
                        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck_f32(&tm.vertices), glow::DYNAMIC_DRAW);
                        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(g.tool_ebo));
                        gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, bytemuck_u32(&tm.indices), glow::DYNAMIC_DRAW);
                        g.tool_count = tm.indices.len() as i32;
                        g.tool_version = Some(tm.version);
                    } else {
                        gl.bind_buffer(glow::ARRAY_BUFFER, Some(g.tool_vbo));
                        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(g.tool_ebo));
                    }
                    // re-bind vertex attributes to the tool buffers
                    point_attributes(gl, g.program);
                    gl.draw_elements(glow::TRIANGLES, g.tool_count, glow::UNSIGNED_INT, 0);
                    // restore board buffers for next frame
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(g.vbo));
                    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(g.ebo));
                    point_attributes(gl, g.program);
                }
                // leave the texture units as egui expects them
                for unit in (0..3).rev() {
                    gl.active_texture(glow::TEXTURE0 + unit);
                    gl.bind_texture(glow::TEXTURE_2D, None);
                }
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
