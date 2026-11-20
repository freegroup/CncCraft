use super::Command;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

#[derive(Debug)]
pub struct LayerLockCommand {
    pub id: u32,
    pub locked: bool,
}

impl LayerLockCommand {
    pub fn new(id: u32, locked: bool) -> Self {
        Self { id, locked }
    }
}

impl Command for LayerLockCommand {
    fn execute(&mut self, _shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>) {
        layers.set_locked(self.id, self.locked);
    }

    fn undo(&mut self, _shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>) {
        layers.set_locked(self.id, !self.locked);
    }

    fn description(&self) -> &str {
        "Sperre ändern"
    }
}
