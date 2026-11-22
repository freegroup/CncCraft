use super::Command;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

#[derive(Debug)]
pub struct LayerFlattenCommand {
    before_layers: Option<Layers>,
    before_shapes: Option<Vec<Item>>,
}

impl LayerFlattenCommand {
    pub fn new() -> Self {
        Self { before_layers: None, before_shapes: None }
    }
}

impl Command for LayerFlattenCommand {
    fn execute(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>) {
        self.before_layers = Some(layers.clone());
        self.before_shapes = Some(shapes.clone());
        layers.flatten(shapes);
    }

    fn undo(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>) {
        if let (Some(l), Some(s)) = (self.before_layers.take(), self.before_shapes.take()) {
            *layers = l;
            *shapes = s;
        }
    }

    fn description(&self) -> &str {
        "Alle Ebenen zusammenfassen"
    }
}
