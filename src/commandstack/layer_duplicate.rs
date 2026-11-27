use super::Command;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

#[derive(Debug)]
pub struct LayerDuplicateCommand {
    pub source_id: u32,
    added_id: Option<u32>,
    before_layers: Option<Layers>,
    before_shapes: Option<Vec<Item>>,
}

impl LayerDuplicateCommand {
    pub fn new(source_id: u32) -> Self {
        Self { source_id, added_id: None, before_layers: None, before_shapes: None }
    }
}

impl Command for LayerDuplicateCommand {
    fn execute(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>, _patterns: &mut Vec<crate::app::Pattern>, _distances: &mut Vec<shape_edit::constraints::Distance>) {
        self.before_layers = Some(layers.clone());
        self.before_shapes = Some(shapes.clone());
        self.added_id = layers.duplicate(self.source_id, shapes);
    }

    fn undo(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>, _patterns: &mut Vec<crate::app::Pattern>, _distances: &mut Vec<shape_edit::constraints::Distance>) {
        if let (Some(l), Some(s)) = (self.before_layers.take(), self.before_shapes.take()) {
            *layers = l;
            *shapes = s;
        }
    }

    fn description(&self) -> &str {
        "Ebene duplizieren"
    }
}
