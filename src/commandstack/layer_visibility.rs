use super::Command;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

#[derive(Debug)]
pub struct LayerVisibilityCommand {
    pub id: u32,
    pub visible: bool,
}

impl LayerVisibilityCommand {
    pub fn new(id: u32, visible: bool) -> Self {
        Self { id, visible }
    }
}

impl Command for LayerVisibilityCommand {
    fn execute(&mut self, _shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>, _patterns: &mut Vec<crate::app::Pattern>, _distances: &mut Vec<shape_edit::constraints::Distance>) {
        layers.set_visible(self.id, self.visible);
    }

    fn undo(&mut self, _shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>, _patterns: &mut Vec<crate::app::Pattern>, _distances: &mut Vec<shape_edit::constraints::Distance>) {
        layers.set_visible(self.id, !self.visible);
    }

    fn description(&self) -> &str {
        "Sichtbarkeit ändern"
    }
}
