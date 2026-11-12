//! The layers of a drawing: every shape is on one layer, a layer can be shown or hidden, and only
//! what is on shown layers is drawn, picked, cut and exported.
//!
//! The original keeps them in a layer manager (`vcCadLayerManager`) with a list form, a "new
//! layer" and a "layer deletion" dialog; the last layer cannot be deleted (`sub_100bc44b0`:
//! "Cannot delete last layer") and its filters know "All visible layers" / "Selected layers
//! only" (`sub_1000f72a4`).

use serde::{Deserialize, Serialize};
use shape_edit::Item;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub id: u32,
    pub name: String,
    pub visible: bool,
    /// Locked: still shown, but nothing on it can be picked, selected or changed.
    #[serde(default)]
    pub locked: bool,
    /// The layer's colour, shown in the list (an index into `PALETTE`).
    #[serde(default)]
    pub colour: usize,
}

/// The colours a layer can have; a click on the dot in the list takes the next one.
pub const PALETTE: [[u8; 3]; 8] = [[0x00, 0x00, 0x00], [0xd0, 0x30, 0x30], [0x20, 0x8a, 0x3c], [0x1e, 0x6f, 0xd9], [0xe0, 0x7a, 0x10], [0x8a, 0x3c, 0xc8], [0x10, 0x9a, 0x9a], [0x80, 0x80, 0x80]];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layers {
    pub list: Vec<Layer>,
    /// The layer new shapes go on.
    pub active: u32,
    next_id: u32,
}

impl Default for Layers {
    fn default() -> Self {
        Layers { list: vec![Layer { id: 1, name: "Ebene 1".into(), visible: true, locked: false, colour: 0 }], active: 1, next_id: 2 }
    }
}

/// What deleting a layer does with the shapes on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leftovers {
    /// They move to the layer before it (or after it, for the first).
    MoveAway,
    /// They are deleted with it.
    Delete,
}

impl Layers {
    /// Make sure a loaded set is usable: at least one layer, an existing active layer.
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

    pub fn name(&self, id: u32) -> &str {
        self.get(id).map_or("", |l| l.name.as_str())
    }

    pub fn is_visible(&self, id: u32) -> bool {
        self.get(id).is_some_and(|l| l.visible)
    }

    /// A name that no other layer has ("Ebene 2", "Ebene 3", …) unless `wanted` is free.
    pub fn free_name(&self, wanted: &str, except: Option<u32>) -> String {
        let wanted = wanted.trim();
        let taken = |n: &str| self.list.iter().any(|l| Some(l.id) != except && l.name.eq_ignore_ascii_case(n));
        if !wanted.is_empty() && !taken(wanted) {
            return wanted.to_string();
        }
        // "Ebene 1" taken: continue the numbering ("Ebene 2"), not "Ebene 1 2"
        let stem = wanted.trim_end_matches(|c: char| c.is_ascii_digit()).trim_end();
        let base = if stem.is_empty() { "Ebene" } else { stem };
        (self.list.len() + 1..).map(|k| format!("{base} {k}")).find(|n| !taken(n)).unwrap_or_default()
    }

    /// Add a layer (it becomes the active one) and return its id.
    pub fn add(&mut self, name: &str) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let name = self.free_name(if name.trim().is_empty() { "Ebene" } else { name }, None);
        let colour = self.list.len() % PALETTE.len();
        self.list.push(Layer { id, name, visible: true, locked: false, colour });
        self.active = id;
        id
    }

    pub fn rename(&mut self, id: u32, name: &str) {
        let name = self.free_name(name, Some(id));
        if let Some(l) = self.list.iter_mut().find(|l| l.id == id) {
            l.name = name;
        }
    }

    /// Switch a layer on or off. The active layer stays active but new shapes on a hidden layer
    /// would vanish at once, so making a layer active shows it (see `activate`).
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

    /// The next colour of the palette for a layer.
    pub fn cycle_colour(&mut self, id: u32) {
        if let Some(l) = self.list.iter_mut().find(|l| l.id == id) {
            l.colour = (l.colour + 1) % PALETTE.len();
        }
    }

    /// Make a layer the one new shapes go on; a hidden layer is shown for that.
    pub fn activate(&mut self, id: u32) {
        if self.get(id).is_some() {
            self.active = id;
            self.set_visible(id, true);
        }
    }

    /// Whether a layer can be deleted: the last one cannot.
    pub fn can_delete(&self, id: u32) -> bool {
        self.list.len() > 1 && self.get(id).is_some()
    }

    /// Delete a layer, doing `leftovers` with its shapes. Returns whether it was deleted.
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

    /// Bring the shapes in line with the layers: a shape on a layer that no longer exists goes to
    /// the first one, and each shape knows whether its layer is shown. Returns whether any
    /// shape's visibility changed.
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
        let two = l.add("Ebene 1");
        assert_eq!(l.name(two), "Ebene 2");
        l.rename(two, "ebene 1");
        assert_eq!(l.name(two).to_lowercase(), "ebene 3");
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
}
