//! Grouping shapes: a group is picked, moved, scaled, rotated and copied as one. Groups can hold
//! groups, and ungrouping takes one level off at a time. A shape lists its groups in
//! `Item::groups` (see `shape_edit::group_items`).

use super::App;
use shape_edit::{group_items, ungroup_items};

impl App {
    /// The selected shapes (and the rest of their groups) become one group.
    pub(super) fn group_selection(&mut self) {
        if !self.selection_can_group() {
            self.status = "Zum Gruppieren mindestens zwei Formen oder Gruppen auswählen".into();
            return;
        }
        let before = self.snapshot();
        let (ids, group) = (self.edit.selection.clone(), self.new_id());
        if group_items(&mut self.shapes, &ids, group) {
            self.push_undo_from(before);
        }
    }

    /// The outermost group of the selection is dissolved; a group inside it stays.
    pub(super) fn ungroup_selection(&mut self) {
        let ids = self.edit.selection.clone();
        self.ungroup(&ids);
    }

    /// Dissolve the outermost group of the shapes `ids`.
    pub(super) fn ungroup(&mut self, ids: &[u64]) {
        let before = self.snapshot();
        // a pattern group is freed specially: the children become independent shapes, each gets the
        // source's toolpaths copied onto it, and the pattern record goes away
        let pattern_group = ids.iter().find_map(|id| {
            let item = self.shapes.iter().find(|s| s.id == *id)?;
            self.patterns.iter().find(|p| item.groups.last() == Some(&p.group)).map(|p| p.group)
        });
        if let Some(group) = pattern_group {
            self.unlink_pattern(group);
            self.push_undo_from(before);
            self.recompute_all();
            self.status = t!("Muster aufgelöst: die Kopien stehen jetzt für sich").into();
            return;
        }
        if ungroup_items(&mut self.shapes, ids) {
            self.push_undo_from(before);
        } else {
            self.status = "Die Auswahl enthält keine Gruppe".into();
        }
    }

    /// Whether the selection is two or more things (a whole group counts as one).
    pub(super) fn selection_can_group(&self) -> bool {
        shape_edit::can_group(&self.shapes, &self.edit.selection)
    }

    /// Whether anything selected is in a group.
    pub(super) fn selection_has_group(&self) -> bool {
        self.shapes.iter().any(|s| !s.groups.is_empty() && self.edit.selection.contains(&s.id))
    }

    /// Copies get groups of their own: every group the copied shapes are in is made anew (inner and
    /// outer alike), so a copy never joins the group of its original.
    pub(super) fn regroup_copies(&mut self, copies: &mut [shape_edit::Item]) {
        let mut made: Vec<(u64, u64)> = Vec::new();
        for item in copies {
            for group in &mut item.groups {
                let new = match made.iter().find(|(old, _)| old == group) {
                    Some((_, new)) => *new,
                    None => {
                        let new = self.new_id();
                        made.push((*group, new));
                        new
                    }
                };
                *group = new;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use shape_edit::{group_items, Circle, Item};

    #[test]
    fn groups_survive_saving_and_old_files_have_none() {
        let mut items: Vec<Item> = (1..=2).map(|id| Item::new(id, Circle::new([id as f64, 0.0], 1.0))).collect();
        group_items(&mut items, &[1, 2], 7);
        let json = serde_json::to_string(&items).unwrap();
        let back: Vec<Item> = serde_json::from_str(&json).unwrap();
        assert_eq!(back[0].groups, vec![7]);
        let plain = serde_json::to_string(&Item::new(9, Circle::new([0.0, 0.0], 1.0))).unwrap();
        assert!(!plain.contains("groups"), "a shape in no group writes nothing");
        assert!(serde_json::from_str::<Item>(&plain).unwrap().groups.is_empty());
    }
}
