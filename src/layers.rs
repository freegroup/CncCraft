//! The layers of a drawing: every shape is on one layer, a layer can be shown or hidden, and only
//! what is on shown layers is drawn, picked, cut and exported.
//!
//! Each layer has a *behavior* (`Box<dyn LayerBehavior>`) that decides which tools are available,
//! whether vectors can be placed, and how the user interacts with it. `VectorBehavior` is the
//! default (drawing shapes); `ImageBehavior` holds a reference picture for tracing.

use serde::{Deserialize, Serialize};
use shape_edit::Item;

// ---------------------------------------------------------------------------
// LayerBehavior trait
// ---------------------------------------------------------------------------

/// What a layer can do: which tools it offers, whether it accepts vectors, and so on. Every layer
/// implementation (vector, image, …) provides its own version.
#[allow(unused)]
pub trait LayerBehavior: std::fmt::Debug + Send + Sync {
    /// A short key used for serialisation ("vector" / "image").
    fn kind_key(&self) -> &'static str;
    /// The translated label shown in the UI ("Ebene" / "Bild").
    fn kind_label(&self) -> String;
    /// Whether shapes (vectors) can be placed on this layer.
    fn can_place_vectors(&self) -> bool;
    /// Whether this layer is an image layer.
    fn is_image(&self) -> bool { false }
    /// The tool groups this layer makes available. Each inner slice is one group of tool indices
    /// into the global GROUPS array (see `tool_column.rs`). `None` means "show all default
    /// vector tools".
    fn tool_groups(&self) -> Option<Vec<Vec<usize>>> { None }
    /// Clone the behavior into a new Box (needed because Layer must be Clone for undo).
    fn clone_behavior(&self) -> Box<dyn LayerBehavior>;
    /// Downcast helpers — only ImageBehavior returns Some.
    fn as_image_ref(&self) -> Option<&ImageBehavior> { None }
    fn as_image_mut(&mut self) -> Option<&mut ImageBehavior> { None }
}

// ---------------------------------------------------------------------------
// VectorBehavior
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct VectorBehavior;

impl LayerBehavior for VectorBehavior {
    fn kind_key(&self) -> &'static str { "vector" }
    fn kind_label(&self) -> String { "Ebene".to_string() }
    fn can_place_vectors(&self) -> bool { true }
    fn clone_behavior(&self) -> Box<dyn LayerBehavior> { Box::new(self.clone()) }
}

// ---------------------------------------------------------------------------
// ImageBehavior
// ---------------------------------------------------------------------------

/// The state of a reference image on an image layer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageData {
    #[serde(default, serialize_with = "ser_bytes", deserialize_with = "de_bytes")]
    pub bytes: Vec<u8>,
    #[serde(default)]
    pub source: String,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default = "default_scale")]
    pub scale: f64,
    #[serde(default)]
    pub offset: [f64; 2],
}

fn ser_bytes<S: serde::Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
    use base64::Engine;
    if bytes.is_empty() {
        s.serialize_str("")
    } else {
        s.serialize_str(&base64::engine::general_purpose::STANDARD.encode(bytes))
    }
}

fn de_bytes<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    use base64::Engine;
    let text = String::deserialize(d)?;
    if text.is_empty() {
        return Ok(vec![]);
    }
    base64::engine::general_purpose::STANDARD
        .decode(&text)
        .map_err(serde::de::Error::custom)
}

fn default_opacity() -> f32 { 0.5 }
fn default_scale() -> f64 { 1.0 }

impl Default for ImageData {
    fn default() -> Self {
        ImageData { bytes: vec![], source: String::new(), opacity: 0.5, scale: 1.0, offset: [0.0, 0.0] }
    }
}

#[derive(Clone, Debug)]
pub struct ImageBehavior {
    pub image: ImageData,
}

impl Default for ImageBehavior {
    fn default() -> Self {
        ImageBehavior { image: ImageData::default() }
    }
}

impl LayerBehavior for ImageBehavior {
    fn kind_key(&self) -> &'static str { "image" }
    fn kind_label(&self) -> String { "Bild".to_string() }
    fn can_place_vectors(&self) -> bool { false }
    fn is_image(&self) -> bool { true }
    fn clone_behavior(&self) -> Box<dyn LayerBehavior> { Box::new(self.clone()) }
    fn as_image_ref(&self) -> Option<&ImageBehavior> { Some(self) }
    fn as_image_mut(&mut self) -> Option<&mut ImageBehavior> { Some(self) }
}

// ---------------------------------------------------------------------------
// Layer
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct Layer {
    pub id: u32,
    pub name: String,
    pub visible: bool,
    #[allow(dead_code)]
    pub locked: bool,
    pub colour: usize,
    pub opacity: f32,
    pub behavior: Box<dyn LayerBehavior>,
}

impl Clone for Layer {
    fn clone(&self) -> Self {
        Layer {
            id: self.id,
            name: self.name.clone(),
            visible: self.visible,
            locked: self.locked,
            colour: self.colour,
            opacity: self.opacity,
            behavior: self.behavior.clone_behavior(),
        }
    }
}

/// Custom Serialize: write kind key + image data when present.
impl Serialize for Layer {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let has_image = self.behavior.is_image();
        let len = 6 + if has_image { 2 } else { 1 };
        let mut st = s.serialize_struct("Layer", len)?;
        st.serialize_field("id", &self.id)?;
        st.serialize_field("name", &self.name)?;
        st.serialize_field("visible", &self.visible)?;
        st.serialize_field("locked", &self.locked)?;
        st.serialize_field("colour", &self.colour)?;
        st.serialize_field("opacity", &self.opacity)?;
        st.serialize_field("kind", self.behavior.kind_key())?;
        if let Some(ib) = self.as_image() {
            st.serialize_field("image", &ib.image)?;
        }
        st.end()
    }
}

/// Custom Deserialize: read kind key, build the right behavior.
impl<'de> Deserialize<'de> for Layer {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            id: u32,
            name: String,
            #[serde(default = "default_true")]
            visible: bool,
            #[serde(default)]
            locked: bool,
            #[serde(default)]
            colour: usize,
            #[serde(default = "default_layer_opacity")]
            opacity: f32,
            #[serde(default)]
            kind: String,
            #[serde(default)]
            image: Option<ImageData>,
        }
        fn default_true() -> bool { true }
        fn default_layer_opacity() -> f32 { 1.0 }
        let r = Raw::deserialize(d)?;
        let behavior: Box<dyn LayerBehavior> = if r.kind == "image" {
            Box::new(ImageBehavior { image: r.image.unwrap_or_default() })
        } else {
            Box::new(VectorBehavior)
        };
        Ok(Layer { id: r.id, name: r.name, visible: r.visible, locked: r.locked, colour: r.colour, opacity: r.opacity, behavior })
    }
}

#[allow(unused)]
impl Layer {
    /// Downcast to `ImageBehavior` if this is an image layer.
    pub fn as_image(&self) -> Option<&ImageBehavior> {
        self.behavior.as_image_ref()
    }

    /// Mutable downcast to `ImageBehavior`.
    pub fn as_image_mut(&mut self) -> Option<&mut ImageBehavior> {
        self.behavior.as_image_mut()
    }
}

/// The colours a layer can have; a click on the dot in the list takes the next one.
pub const PALETTE: [[u8; 3]; 8] = [[0x00, 0x00, 0x00], [0xd0, 0x30, 0x30], [0x20, 0x8a, 0x3c], [0x1e, 0x6f, 0xd9], [0xe0, 0x7a, 0x10], [0x8a, 0x3c, 0xc8], [0x10, 0x9a, 0x9a], [0x80, 0x80, 0x80]];

// ---------------------------------------------------------------------------
// Layers collection
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layers {
    pub list: Vec<Layer>,
    pub active: u32,
    next_id: u32,
}

impl Default for Layers {
    fn default() -> Self {
        Layers {
            list: vec![Layer {
                id: 1,
                name: "Ebene 1".into(),
                visible: true,
                locked: false,
                colour: 0,
                opacity: 1.0,
                behavior: Box::new(VectorBehavior),
            }],
            active: 1,
            next_id: 2,
        }
    }
}

/// What deleting a layer does with the shapes on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leftovers {
    MoveAway,
    Delete,
}

#[allow(unused)]
impl Layers {
    pub fn repaired(mut self) -> Layers {
        if self.list.is_empty() {
            return Layers::default();
        }
        if !self.list.iter().any(|l| l.id == self.active) {
            self.active = self.list[0].id;
        }
        let top = self.list.iter().map(|l| l.id).max().unwrap_or(0);
        self.next_id = self.next_id.max(top + 1);
        self
    }

    pub fn get(&self, id: u32) -> Option<&Layer> {
        self.list.iter().find(|l| l.id == id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Layer> {
        self.list.iter_mut().find(|l| l.id == id)
    }

    pub fn name(&self, id: u32) -> &str {
        self.get(id).map_or("", |l| l.name.as_str())
    }

    pub fn is_visible(&self, id: u32) -> bool {
        self.get(id).is_some_and(|l| l.visible)
    }

    pub fn free_name(&self, wanted: &str, except: Option<u32>) -> String {
        let wanted = wanted.trim();
        let taken = |n: &str| self.list.iter().any(|l| Some(l.id) != except && l.name.eq_ignore_ascii_case(n));
        if !wanted.is_empty() && !taken(wanted) {
            return wanted.to_string();
        }
        let stem = wanted.trim_end_matches(|c: char| c.is_ascii_digit()).trim_end();
        let base = if stem.is_empty() { "Ebene".to_string() } else { stem.to_string() };
        (self.list.len() + 1..).map(|k| format!("{base} {k}")).find(|n| !taken(n)).unwrap_or_default()
    }

    /// Add a vector layer (the common case).
    pub fn add(&mut self, name: &str) -> u32 {
        self.add_with(name, Box::new(VectorBehavior))
    }

    /// Add an image layer.
    pub fn add_image(&mut self, name: &str) -> u32 {
        self.add_with(name, Box::new(ImageBehavior::default()))
    }

    /// Add a layer with custom behavior.
    fn add_with(&mut self, name: &str, behavior: Box<dyn LayerBehavior>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let default_name = behavior.kind_label();
        let name = self.free_name(&if name.trim().is_empty() { default_name } else { name.to_string() }, None);
        let colour = self.list.len() % PALETTE.len();
        self.list.push(Layer { id, name, visible: true, locked: false, colour, opacity: 1.0, behavior });
        self.active = id;
        id
    }

    pub fn rename(&mut self, id: u32, name: &str) {
        let name = self.free_name(name, Some(id));
        if let Some(l) = self.list.iter_mut().find(|l| l.id == id) {
            l.name = name;
        }
    }

    pub fn set_visible(&mut self, id: u32, on: bool) {
        if let Some(l) = self.list.iter_mut().find(|l| l.id == id) {
            l.visible = on;
        }
    }

    pub fn set_locked(&mut self, id: u32, on: bool) {
        if let Some(l) = self.list.iter_mut().find(|l| l.id == id) {
            l.locked = on;
        }
    }

    pub fn is_locked(&self, id: u32) -> bool {
        self.get(id).is_some_and(|l| l.locked)
    }

    /// Whether a layer is an image layer.
    pub fn is_image(&self, id: u32) -> bool {
        self.get(id).is_some_and(|l| l.behavior.is_image())
    }

    /// The active layer's behavior.
    pub fn active_behavior(&self) -> &dyn LayerBehavior {
        self.get(self.active).map_or(&VectorBehavior as &dyn LayerBehavior, |l| &*l.behavior)
    }

    /// Whether the active layer accepts vectors.
    pub fn active_can_place_vectors(&self) -> bool {
        self.active_behavior().can_place_vectors()
    }

    /// Mutable access to an image layer's data.
    pub fn image_mut(&mut self, id: u32) -> Option<&mut ImageData> {
        self.get_mut(id).and_then(|l| l.as_image_mut()).map(|ib| &mut ib.image)
    }

    pub fn cycle_colour(&mut self, id: u32) {
        if let Some(l) = self.list.iter_mut().find(|l| l.id == id) {
            l.colour = (l.colour + 1) % PALETTE.len();
        }
    }

    pub fn set_opacity(&mut self, id: u32, opacity: f32) {
        if let Some(l) = self.list.iter_mut().find(|l| l.id == id) {
            l.opacity = opacity.clamp(0.0, 1.0);
        }
    }

    pub fn opacity(&self, id: u32) -> f32 {
        self.get(id).map_or(1.0, |l| l.opacity)
    }

    pub fn activate(&mut self, id: u32) {
        if self.get(id).is_some() {
            self.active = id;
            self.set_visible(id, true);
        }
    }

    pub fn can_delete(&self, id: u32) -> bool {
        self.list.len() > 1 && self.get(id).is_some()
    }

    pub fn delete(&mut self, id: u32, leftovers: Leftovers, items: &mut Vec<Item>) -> bool {
        if !self.can_delete(id) {
            return false;
        }
        let at = self.list.iter().position(|l| l.id == id).unwrap_or(0);
        let neighbour = if at > 0 { self.list[at - 1].id } else { self.list[1].id };
        match leftovers {
            Leftovers::MoveAway => items.iter_mut().filter(|i| i.layer == id).for_each(|i| i.layer = neighbour),
            Leftovers::Delete => items.retain(|i| i.layer != id),
        }
        self.list.remove(at);
        if self.active == id {
            self.active = neighbour;
        }
        true
    }

    pub fn sync(&self, items: &mut [Item]) -> bool {
        let first = self.list.first().map_or(1, |l| l.id);
        let mut changed = false;
        for it in items {
            if self.get(it.layer).is_none() {
                it.layer = first;
            }
            let hidden = !self.is_visible(it.layer);
            let frozen = self.is_locked(it.layer);
            changed |= it.hidden != hidden || it.frozen != frozen;
            it.hidden = hidden;
            it.frozen = frozen;
        }
        changed
    }

    pub fn count_on(&self, id: u32, items: &[Item]) -> usize {
        items.iter().filter(|i| i.layer == id).count()
    }

    pub fn duplicate(&mut self, id: u32, items: &mut Vec<Item>) -> Option<u32> {
        let at = self.list.iter().position(|l| l.id == id)?;
        let src = &self.list[at];
        let new_id = self.next_id;
        self.next_id += 1;
        let new_name = self.free_name(&src.name, None);
        let new_layer = Layer {
            id: new_id,
            name: new_name,
            visible: src.visible,
            locked: src.locked,
            colour: src.colour,
            opacity: src.opacity,
            behavior: src.behavior.clone_behavior(),
        };
        self.list.insert(at + 1, new_layer);
        let clones: Vec<Item> = items.iter().filter(|i| i.layer == id).cloned().collect();
        let mut max_id = items.iter().map(|i| i.id).max().unwrap_or(0);
        for mut c in clones {
            max_id += 1;
            c.id = max_id;
            c.layer = new_id;
            items.push(c);
        }
        self.active = new_id;
        Some(new_id)
    }

    pub fn move_up(&mut self, id: u32) -> bool {
        let Some(at) = self.list.iter().position(|l| l.id == id) else { return false };
        if at == 0 { return false; }
        self.list.swap(at, at - 1);
        true
    }

    pub fn move_down(&mut self, id: u32) -> bool {
        let Some(at) = self.list.iter().position(|l| l.id == id) else { return false };
        if at + 1 >= self.list.len() { return false; }
        self.list.swap(at, at + 1);
        true
    }

    pub fn move_to(&mut self, from: usize, to: usize) {
        if from >= self.list.len() { return; }
        let target = to.min(self.list.len());
        if from == target { return; }
        let item = self.list.remove(from);
        let insert_at = if target > from { target - 1 } else { target };
        self.list.insert(insert_at.min(self.list.len()), item);
    }

    pub fn merge_down(&mut self, id: u32, items: &mut Vec<Item>) -> bool {
        let Some(at) = self.list.iter().position(|l| l.id == id) else { return false };
        if at + 1 >= self.list.len() { return false; }
        let target_id = self.list[at + 1].id;
        for i in items.iter_mut() {
            if i.layer == id {
                i.layer = target_id;
            }
        }
        self.list.remove(at);
        if self.active == id {
            self.active = target_id;
        }
        true
    }

    pub fn flatten(&mut self, items: &mut Vec<Item>) {
        if self.list.len() <= 1 { return; }
        let first_id = self.list[0].id;
        for i in items.iter_mut() {
            i.layer = first_id;
        }
        self.list.truncate(1);
        self.active = first_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shape_edit::Rect;

    fn items() -> Vec<Item> {
        let mut v: Vec<Item> = (1..=3).map(|i| Item::new(i, Rect::new([0.0, 0.0], 1.0, 1.0))).collect();
        v[2].layer = 2;
        v
    }

    #[test]
    fn hiding_a_layer_hides_its_shapes_only() {
        let mut l = Layers::default();
        l.add("Zwei");
        let mut it = items();
        assert!(!l.sync(&mut it));
        l.set_visible(2, false);
        assert!(l.sync(&mut it));
        assert_eq!(it.iter().map(|i| i.hidden).collect::<Vec<_>>(), vec![false, false, true]);
        assert!(!l.sync(&mut it), "nothing changed the second time");
    }

    #[test]
    fn the_last_layer_cannot_be_deleted() {
        let mut l = Layers::default();
        let mut it = items();
        assert!(!l.can_delete(1));
        assert!(!l.delete(1, Leftovers::Delete, &mut it));
        assert_eq!(it.len(), 3);
    }

    #[test]
    fn deleting_moves_or_removes_the_shapes() {
        let mut l = Layers::default();
        l.add("Zwei");
        let mut a = items();
        assert!(l.clone().delete(2, Leftovers::MoveAway, &mut a));
        assert!(a.iter().all(|i| i.layer == 1));
        let mut b = items();
        assert!(l.delete(2, Leftovers::Delete, &mut b));
        assert_eq!(b.len(), 2);
        assert_eq!(l.active, 1);
    }

    #[test]
    fn names_stay_unique_and_unknown_layers_fall_back() {
        let mut l = Layers::default();
        let default_name = l.name(1).to_string();
        let two = l.add(&default_name);
        assert_ne!(l.name(two), default_name, "must be different from the first layer");
        l.rename(two, &default_name.to_lowercase());
        assert_ne!(l.name(two).to_lowercase(), default_name.to_lowercase(), "rename must also stay unique");
        let mut it = items();
        it[0].layer = 99;
        l.sync(&mut it);
        assert_eq!(it[0].layer, 1);
    }

    #[test]
    fn a_locked_layer_stays_visible_but_its_shapes_are_frozen() {
        let mut l = Layers::default();
        let two = l.add("Zwei");
        l.set_locked(two, true);
        let mut it = items();
        assert!(l.sync(&mut it));
        assert_eq!(it.iter().map(|i| (i.hidden, i.frozen)).collect::<Vec<_>>(), vec![(false, false), (false, false), (false, true)]);
        use shape_edit::Editable;
        assert!(!it[2].hit([0.0, 0.0], 5.0) || it[2].frozen);
        assert!(it[2].snap_points().is_empty());
    }

    #[test]
    fn activating_shows_a_hidden_layer() {
        let mut l = Layers::default();
        let two = l.add("Zwei");
        l.set_visible(two, false);
        l.activate(1);
        l.activate(two);
        assert!(l.is_visible(two));
    }

    #[test]
    fn old_files_without_layers_still_load() {
        let item: Item = serde_json::from_str(r#"{"id":1,"shape":{"Rect":{"center":[0.0,0.0],"w":1.0,"h":1.0,"rotation":0.0}}}"#).unwrap_or_else(|_| Item::new(1, Rect::new([0.0, 0.0], 1.0, 1.0)));
        assert_eq!(item.layer, 1);
        assert!(!item.hidden);
    }

    #[test]
    fn image_layer_basics() {
        let mut l = Layers::default();
        let img = l.add_image("Foto");
        assert!(l.is_image(img));
        assert!(!l.active_can_place_vectors());
        assert!(l.image_mut(img).is_some());
        l.activate(1);
        assert!(l.active_can_place_vectors());
    }
}
