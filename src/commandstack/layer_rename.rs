use super::Command;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

#[derive(Debug)]
pub struct LayerRenameCommand {
    pub id: u32,
    pub new_name: String,
    old_name: Option<String>,
}

impl LayerRenameCommand {
    pub fn new(id: u32, new_name: String) -> Self {
        Self { id, new_name, old_name: None }
    }
}

impl Command for LayerRenameCommand {
    fn execute(&mut self, _shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>) {
        self.old_name = Some(layers.name(self.id).to_string());
        layers.rename(self.id, &self.new_name);
    }

    fn undo(&mut self, _shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>) {
        if let Some(old) = &self.old_name {
            layers.rename(self.id, old);
        }
    }

    fn description(&self) -> &str {
        "Ebene umbenennen"
    }
}
