use super::*;

impl App {
    /// Gather what copying `ids` takes along: the shapes, the constraints that belong to a single
    /// one of them (width, height, diameter, point positions, level/upright) and the toolpaths
    /// that cut them.
    pub(super) fn make_clip(&self, ids: &[u64]) -> Clip {
        let items: Vec<Item> = ids.iter().filter_map(|id| self.shapes.iter().find(|s| s.id == *id)).cloned().collect();
        let own = |a: u64, b: u64| a == b && ids.contains(&a);
        let distances = self.distances.iter().filter(|c| own(c.from.shape(), c.to.shape())).cloned().collect();
        let aligns = self.aligns.iter().filter(|c| own(c.a.shape(), c.b.shape())).cloned().collect();
        let toolpaths = self
            .toolpaths
            .iter()
            .filter(|tp| tp.shape_ids.iter().any(|id| ids.contains(id)))
            .map(|tp| Toolpath { shape_ids: tp.shape_ids.iter().copied().filter(|id| ids.contains(id)).collect(), ..tp.clone() })
            .collect();
        Clip { items, distances, aligns, toolpaths }
    }

    /// Give the copies (old id → new id) the constraints and toolpaths of what they were copied from;
    /// `shift` is how far the copies lie from the originals.
    pub(super) fn add_extras(&mut self, clip: &Clip, map: &[(u64, u64)], shift: [f64; 2]) {
        let new = |id: u64| map.iter().find(|(old, _)| *old == id).map(|(_, n)| *n);
        let anchor = |a: shape_edit::Anchor| match a {
            shape_edit::Anchor::Point(id, k) => new(id).map(|n| shape_edit::Anchor::Point(n, k)),
            shape_edit::Anchor::Edge(id, k) => new(id).map(|n| shape_edit::Anchor::Edge(n, k)),
        };
        for c in &clip.distances {
            if let (Some(from), Some(to)) = (anchor(c.from), anchor(c.to)) {
                self.distances.push(shape_edit::Distance { from, to, place: [c.place[0] + shift[0], c.place[1] + shift[1]], ..c.clone() });
            }
        }
        for c in &clip.aligns {
            if let (Some(a), Some(b)) = (anchor(c.a), anchor(c.b)) {
                self.aligns.push(shape_edit::Align { a, b, ..c.clone() });
            }
        }
        let before = self.toolpaths.len();
        for tp in &clip.toolpaths {
            let ids: Vec<u64> = tp.shape_ids.iter().filter_map(|id| new(*id)).collect();
            if ids.is_empty() {
                continue;
            }
            let copy = Toolpath { shape_ids: ids, name: t!("%{name} (Kopie)", name = tp.name), ..tp.clone() };
            self.toolpaths.push(copy);
        }
        if self.toolpaths.len() != before {
            self.paste_log.push((map.iter().map(|(_, n)| *n).collect(), before));
            self.recompute_all();
        }
        self.vars_dirty = true;
    }

    pub(super) fn copy_selection(&mut self) {
        let clip = self.make_clip(&self.edit.selection.clone());
        if !clip.is_empty() {
            self.status = format!("{} Objekt(e) kopiert, mit {} Werkzeugweg(en) und {} eigenen Constraint(s)", clip.items.len(), clip.toolpaths.len(), clip.distances.len() + clip.aligns.len());
            self.clipboard = clip;
        }
    }

    /// Paste the copied shapes a little offset, like a duplicate, and select them.
    pub(super) fn paste(&mut self) {
        let clip = self.clipboard.clone();
        self.paste_clip(&clip);
    }

    /// Put the shapes of `clip` in, a little offset, with their own constraints and toolpaths.
    pub(super) fn paste_clip(&mut self, clip: &Clip) {
        if clip.is_empty() {
            return;
        }
        self.push_undo();
        self.edit.selection.clear();
        let mut map = vec![];
        let first_copy = self.shapes.len();
        for it in &clip.items {
            let id = self.new_id();
            let mut copy = it.translated(10.0, -10.0);
            copy.id = id;
            let usable = self.layers.is_visible(it.layer) && !self.layers.is_locked(it.layer);
            copy.layer = if usable { it.layer } else { self.layers.active };
            self.layers.set_visible(copy.layer, true);
            self.shapes.push(copy);
            map.push((it.id, id));
            self.edit.selection.push(id);
        }
        let mut pasted = self.shapes.split_off(first_copy);
        self.regroup_copies(&mut pasted);
        self.shapes.extend(pasted);
        self.add_extras(clip, &map, [10.0, -10.0]);
        self.status = format!("{} Objekt(e) eingefügt", map.len());
    }

    /// After a ⌃-drag: the moved shapes become copies and the originals go back where they were.
    pub(super) fn keep_originals(&mut self, orig: Vec<Item>) {
        let mut copies = vec![];
        let mut map = vec![];
        for o in orig {
            let id = self.new_id();
            if let Some(moved) = self.shapes.iter_mut().find(|s| s.id == o.id) {
                moved.id = id;
                copies.push(id);
                map.push((o.id, id));
            }
            self.shapes.push(o);
        }
        let mut moved: Vec<Item> = self.shapes.iter().filter(|s| copies.contains(&s.id)).cloned().collect();
        self.regroup_copies(&mut moved);
        for m in moved {
            if let Some(s) = self.shapes.iter_mut().find(|s| s.id == m.id) {
                s.groups = m.groups;
            }
        }
        let old: Vec<u64> = map.iter().map(|(o, _)| *o).collect();
        let clip = self.make_clip(&old);
        self.add_extras(&clip, &map, [0.0, 0.0]);
        self.edit.selection = copies;
    }
}
