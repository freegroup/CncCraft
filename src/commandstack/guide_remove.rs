use super::Command;
use crate::guides::Guide;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

/// Remove a construction line (its attachments go with it; the shapes stay put). Undo puts it
/// back where it was in the list, with its attachments.
#[derive(Debug)]
pub struct GuideRemoveCommand {
    id: u64,
    /// The line and where it sat, captured on removal, so undo restores both.
    removed: Option<(usize, Guide)>,
}

impl GuideRemoveCommand {
    pub fn new(id: u64) -> Self {
        Self { id, removed: None }
    }
}

impl Command for GuideRemoveCommand {
    fn execute(&mut self, _shapes: &mut Vec<Item>, _layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, guides: &mut Vec<Guide>) {
        if let Some(i) = guides.iter().position(|g| g.id == self.id) {
            self.removed = Some((i, guides.remove(i)));
        }
    }

    fn undo(&mut self, _shapes: &mut Vec<Item>, _layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, guides: &mut Vec<Guide>) {
        if let Some((i, g)) = self.removed.take() {
            guides.insert(i.min(guides.len()), g);
        }
    }

    fn description(&self) -> &str {
        "Hilfslinie löschen"
    }
}
