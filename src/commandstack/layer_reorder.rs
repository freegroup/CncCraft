use super::Command;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

#[derive(Debug)]
pub struct LayerReorderCommand {
    pub from: usize,
    pub to: usize,
}

impl LayerReorderCommand {
    pub fn new(from: usize, to: usize) -> Self {
        Self { from, to }
    }

    fn effective_target(&self) -> usize {
        if self.to > self.from { self.to - 1 } else { self.to }
    }

    fn reverse_target(&self) -> (usize, usize) {
        let target = self.effective_target();
        if self.from < self.to {
            (target, self.from)
        } else {
            (target, self.from)
        }
    }
}

impl Command for LayerReorderCommand {
    fn execute(&mut self, _shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>, _patterns: &mut Vec<crate::app::Pattern>, _distances: &mut Vec<shape_edit::constraints::Distance>) {
        layers.move_to(self.from, self.to);
    }

    fn undo(&mut self, _shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>, _patterns: &mut Vec<crate::app::Pattern>, _distances: &mut Vec<shape_edit::constraints::Distance>) {
        let (from, to) = self.reverse_target();
        layers.move_to(from, to);
    }

    fn description(&self) -> &str {
        "Ebene verschieben"
    }
}
