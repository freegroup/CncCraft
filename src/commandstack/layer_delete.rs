use super::Command;
use crate::layers::{Layers, Leftovers};
use crate::toolpath::Toolpath;
use shape_edit::Item;

#[derive(Debug)]
pub struct LayerDeleteCommand {
    pub id: u32,
    pub leftovers: Leftovers,
    before_layers: Option<Layers>,
    before_shapes: Option<Vec<Item>>,
}

impl LayerDeleteCommand {
    pub fn new(id: u32, leftovers: Leftovers) -> Self {
        Self { id, leftovers, before_layers: None, before_shapes: None }
    }
}

impl Command for LayerDeleteCommand {
    fn execute(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>) {
        self.before_layers = Some(layers.clone());
        self.before_shapes = Some(shapes.clone());
        layers.delete(self.id, self.leftovers, shapes);
    }

    fn undo(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>) {
        if let (Some(l), Some(s)) = (self.before_layers.take(), self.before_shapes.take()) {
            *layers = l;
            *shapes = s;
        }
    }

    fn description(&self) -> &str {
        "Ebene löschen"
    }
}
