use super::Command;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

#[derive(Debug)]
pub struct LayerAddCommand {
    pub name: String,
    pub image: bool,
    added_id: Option<u32>,
}

impl LayerAddCommand {
    pub fn new(name: String, image: bool) -> Self {
        Self { name, image, added_id: None }
    }
}

impl Command for LayerAddCommand {
    fn execute(&mut self, _shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>, _patterns: &mut Vec<crate::app::Pattern>) {
        let id = if self.image {
            layers.add_image(&self.name)
        } else {
            layers.add(&self.name)
        };
        self.added_id = Some(id);
    }

    fn undo(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>, _patterns: &mut Vec<crate::app::Pattern>) {
        if let Some(id) = self.added_id {
            layers.delete(id, crate::layers::Leftovers::Delete, shapes);
        }
    }

    fn description(&self) -> &str {
        "Ebene hinzufügen"
    }
}
