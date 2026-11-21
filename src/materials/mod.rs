//! The material library: what a material is, which cutting values it allows, what it needs and how
//! it looks. Every job is cut from one material; the toolpath form and the G-code header warn when
//! a value goes beyond the material's limits. The shipped materials, each in its own file here, can
//! be edited, and the person can add their own (they borrow the look of a shipped one).
//!
//! The original only names its materials (`material` table of tools.vtdb: id and name) and keeps
//! the cutting data per tool; the limits, recommendations and looks here are CncCraft's own.

mod pattern;

mod acrylic;
mod aluminium;
mod brass;
mod copper;
mod foam;
mod hardwood;
mod hdpe;
mod mdf;
mod plywood;
mod softwood;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Material {
    /// Stable key kept in jobs and in the tool library; the shipped ones use the original's names.
    pub key: String,
    pub name: String,
    pub description: String,
    /// The fastest feed the material takes, mm/min.
    pub max_feed: f64,
    /// Recommended spindle speed, rpm.
    pub rpm_min: u32,
    pub rpm_max: u32,
    /// Deepest cut per pass as a share of the cutter's diameter (the 2/3 rule is 0.67).
    pub depth_ratio: f64,
    /// Which cutters suit it: flutes, cutter material, spiral direction (free text).
    pub cutter: String,
    /// Whether and with what to cool (free text).
    pub coolant: String,
    /// The key of the shipped material whose look it has in the 3D view.
    #[serde(default)]
    pub look: String,
    /// Shipped with the program: its texts are translated, it can't be deleted.
    #[serde(default)]
    pub builtin: bool,
}

impl Material {
    /// A text of the material as shown: the shipped ones are translated, typed ones stay as they are.
    fn shown(&self, text: &str) -> String {
        if self.builtin {
            t!(text)
        } else {
            text.to_string()
        }
    }

    pub fn shown_name(&self) -> String {
        self.shown(&self.name)
    }

    pub fn shown_description(&self) -> String {
        self.shown(&self.description)
    }

    pub fn shown_cutter(&self) -> String {
        self.shown(&self.cutter)
    }

    pub fn shown_coolant(&self) -> String {
        self.shown(&self.coolant)
    }

    /// The deepest cut per pass for a cutter of `diameter` mm.
    pub fn max_depth(&self, diameter: f64) -> f64 {
        self.depth_ratio * diameter
    }

    /// The allowed pass depth when `stepdown` cuts deeper than the material takes.
    pub fn depth_exceeded(&self, stepdown: f64, diameter: f64) -> Option<f64> {
        let max = self.max_depth(diameter);
        (diameter > 0.0 && stepdown > max + 1e-9).then_some(max)
    }

    /// The fastest allowed feed when `feed` is faster.
    pub fn feed_exceeded(&self, feed: f64) -> Option<f64> {
        (self.max_feed > 0.0 && feed > self.max_feed + 1e-9).then_some(self.max_feed)
    }

    /// The recommended range when `rpm` lies outside it.
    pub fn rpm_outside(&self, rpm: u32) -> Option<(u32, u32)> {
        (rpm < self.rpm_min || rpm > self.rpm_max).then_some((self.rpm_min, self.rpm_max))
    }
}

/// How a material's surface is drawn: three pictures of one tile, each a colour (0..1) for a point
/// (u, v) of the tile (0..1 each). The pictures repeat without seams.
#[derive(Clone, Copy)]
pub struct Look {
    /// Seen from above, and on the floors of cuts; wood's grain runs along u (X).
    pub top: fn(f32, f32) -> [f32; 3],
    /// The long sides and walls along X: u along X, v down into the material.
    pub edge: fn(f32, f32) -> [f32; 3],
    /// The ends and walls along Y: u along Y, v down into the material (end grain for wood).
    pub end: fn(f32, f32) -> [f32; 3],
    /// How much it shines: 0 matt, 1 polished metal.
    pub shine: f32,
    /// The size of one tile, mm.
    pub tile: f32,
}

/// One shipped material: its file's data and look.
struct Shipped {
    material: fn() -> Material,
    look: fn() -> Look,
}

const SHIPPED: [Shipped; 10] = [
    Shipped { material: softwood::material, look: softwood::look },
    Shipped { material: hardwood::material, look: hardwood::look },
    Shipped { material: plywood::material, look: plywood::look },
    Shipped { material: mdf::material, look: mdf::look },
    Shipped { material: acrylic::material, look: acrylic::look },
    Shipped { material: hdpe::material, look: hdpe::look },
    Shipped { material: foam::material, look: foam::look },
    Shipped { material: aluminium::material, look: aluminium::look },
    Shipped { material: brass::material, look: brass::look },
    Shipped { material: copper::material, look: copper::look },
];

fn builtins() -> Vec<Material> {
    SHIPPED.iter().map(|s| (s.material)()).collect()
}

/// The look of the shipped material `key`; softwood for a key no shipped material has.
pub fn look(key: &str) -> Look {
    SHIPPED.iter().find(|s| (s.material)().key == key).map_or_else(softwood::look, |s| (s.look)())
}

/// The looks a material can have: the shipped materials' keys and names, in their order.
pub fn looks() -> Vec<(String, String)> {
    builtins().into_iter().map(|m| (m.key.clone(), m.shown_name())).collect()
}

pub struct MaterialLibrary {
    pub list: Vec<Material>,
}

impl MaterialLibrary {
    pub fn load() -> Self {
        MaterialLibrary { list: builtins() }
    }

    /// The saved list; shipped materials missing from it are added again, so a newer version's
    /// materials appear, and a material without a look gets its own (or softwood's).
    pub fn load_from(json: &str) -> Self {
        let Ok(mut list) = serde_json::from_str::<Vec<Material>>(json) else { return Self::load() };
        for shipped in builtins() {
            match list.iter_mut().find(|m| m.key == shipped.key) {
                Some(m) => m.builtin = true,
                None => list.push(shipped),
            }
        }
        let shipped_keys: Vec<String> = builtins().into_iter().map(|m| m.key).collect();
        for m in &mut list {
            if !shipped_keys.contains(&m.look) {
                m.look = if shipped_keys.contains(&m.key) { m.key.clone() } else { softwood::KEY.into() };
            }
        }
        MaterialLibrary { list }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.list).unwrap_or_default()
    }

    pub fn get(&self, key: &str) -> Option<&Material> {
        self.list.iter().find(|m| m.key == key)
    }

    /// The material's name as shown; a key no material has (a project from another computer)
    /// shows as it is.
    pub fn label(&self, key: &str) -> String {
        self.get(key).map_or_else(|| key.to_string(), Material::shown_name)
    }

    /// The materials as listed to the person: by name, as the original lists them
    /// (`SELECT … FROM material ORDER BY name ASC`, 0x101e6e578).
    pub fn sorted(&self) -> Vec<&Material> {
        let mut all: Vec<&Material> = self.list.iter().collect();
        all.sort_by_key(|m| m.shown_name().to_lowercase());
        all
    }

    /// Put `list` in the order it is listed in: by name.
    pub fn sort(list: &mut [Material]) {
        list.sort_by_key(|m| m.shown_name().to_lowercase());
    }

    /// A key no material in `list` has yet.
    pub fn new_key(list: &[Material]) -> String {
        (1..).map(|n| format!("custom-{n}")).find(|k| !list.iter().any(|m| &m.key == k)).unwrap_or_default()
    }
}

/// Bake one picture of a look into RGBA pixels, `size` × `size`.
pub fn bake(picture: fn(f32, f32) -> [f32; 3], size: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let c = picture((x as f32 + 0.5) / size as f32, (y as f32 + 0.5) / size as f32);
            for ch in c {
                out.push((ch.clamp(0.0, 1.0) * 255.0).round() as u8);
            }
            out.push(255);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_thirds_of_a_6mm_cutter_is_4mm_per_pass() {
        let mut m = builtins().remove(0);
        m.depth_ratio = 2.0 / 3.0;
        assert!((m.max_depth(6.0) - 4.0).abs() < 1e-9);
        assert_eq!(m.depth_exceeded(4.0, 6.0), None);
        assert!(m.depth_exceeded(4.5, 6.0).is_some());
    }

    #[test]
    fn hardwood_warns_a_6mm_pass_with_a_5mm_cutter() {
        let lib = MaterialLibrary::load();
        let hard = lib.get("Hardwood").unwrap();
        assert_eq!(hard.depth_exceeded(6.0, 5.0), Some(2.5));
    }

    #[test]
    fn feed_and_speed_limits() {
        let lib = MaterialLibrary::load();
        let alu = lib.get("Aluminium").unwrap();
        assert_eq!(alu.feed_exceeded(700.0), None);
        assert_eq!(alu.feed_exceeded(1200.0), Some(800.0));
        assert_eq!(alu.rpm_outside(14000), None);
        assert_eq!(alu.rpm_outside(24000), Some((10000, 18000)));
    }

    #[test]
    fn every_shipped_material_has_its_own_key_and_look() {
        let all = builtins();
        for (i, m) in all.iter().enumerate() {
            assert!(m.builtin && m.key == m.look, "{}", m.key);
            assert!(all.iter().skip(i + 1).all(|o| o.key != m.key), "{} twice", m.key);
            assert!(m.rpm_min <= m.rpm_max && m.depth_ratio > 0.0 && m.max_feed > 0.0, "{}", m.key);
        }
    }

    #[test]
    fn looks_repeat_without_seams() {
        // the colour just inside the right edge meets the colour just inside the left edge
        for m in builtins() {
            let look = look(&m.look);
            for picture in [look.top, look.edge, look.end] {
                for k in 0..8 {
                    let t = k as f32 / 8.0;
                    let (a, b) = (picture(1e-6, t), picture(1.0 - 1e-6, t));
                    let (c, d) = (picture(t, 1e-6), picture(t, 1.0 - 1e-6));
                    let close = |p: [f32; 3], q: [f32; 3]| p.iter().zip(q).all(|(x, y)| (x - y).abs() < 0.06);
                    assert!(close(a, b) && close(c, d), "{} has a seam", m.key);
                }
            }
        }
    }

    #[test]
    fn saved_list_keeps_own_materials_and_gets_new_shipped_ones() {
        let own = Material { key: "custom-1".into(), name: "Kork".into(), builtin: false, ..builtins().remove(0) };
        let json = serde_json::to_string(&vec![own.clone()]).unwrap();
        let lib = MaterialLibrary::load_from(&json);
        assert_eq!(lib.list[0], own);
        assert!(lib.get("Brass").is_some_and(|m| m.builtin));
        assert_eq!(MaterialLibrary::new_key(&lib.list), "custom-2");
    }
}
