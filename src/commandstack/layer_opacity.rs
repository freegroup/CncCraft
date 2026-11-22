use super::Command;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

#[derive(Debug)]
pub struct LayerOpacityCommand {
    pub id: u32,
    pub new_opacity: f32,
    old_opacity: Option<f32>,
}

impl LayerOpacityCommand {
    pub fn new(id: u32, new_opacity: f32) -> Self {
        Self { id, new_opacity, old_opacity: None }
    }
}

impl Command for LayerOpacityCommand {
    fn execute(&mut self, _shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>) {
        self.old_opacity = Some(layers.opacity(self.id));
        layers.set_opacity(self.id, self.new_opacity);
    }

    fn undo(&mut self, _shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>) {
        if let Some(old) = self.old_opacity {
            layers.set_opacity(self.id, old);
        }
    }

    fn description(&self) -> &str {
        "Deckkraft ändern"
    }
}
