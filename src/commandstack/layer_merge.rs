use super::Command;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

#[derive(Debug)]
pub struct LayerMergeCommand {
    pub id: u32,
    before_layers: Option<Layers>,
    before_shapes: Option<Vec<Item>>,
}

impl LayerMergeCommand {
    pub fn new(id: u32) -> Self {
        Self { id, before_layers: None, before_shapes: None }
    }
}

impl Command for LayerMergeCommand {
    fn execute(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>) {
        self.before_layers = Some(layers.clone());
        self.before_shapes = Some(shapes.clone());
        layers.merge_down(self.id, shapes);
    }

    fn undo(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>) {
        if let (Some(l), Some(s)) = (self.before_layers.take(), self.before_shapes.take()) {
            *layers = l;
            *shapes = s;
        }
    }

    fn description(&self) -> &str {
        "Ebene zusammenführen"
    }
}
