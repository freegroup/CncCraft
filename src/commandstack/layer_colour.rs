use super::Command;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

#[derive(Debug)]
pub struct LayerColourCommand {
    pub id: u32,
    old_colour: Option<usize>,
}

impl LayerColourCommand {
    pub fn new(id: u32) -> Self {
        Self { id, old_colour: None }
    }
}

impl Command for LayerColourCommand {
    fn execute(&mut self, _shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>) {
        self.old_colour = layers.get(self.id).map(|l| l.colour);
        layers.cycle_colour(self.id);
    }

    fn undo(&mut self, _shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>) {
        if let Some(old) = self.old_colour {
            if let Some(l) = layers.get_mut(self.id) {
                l.colour = old;
            }
        }
    }

    fn description(&self) -> &str {
        "Ebenenfarbe ändern"
    }
}
