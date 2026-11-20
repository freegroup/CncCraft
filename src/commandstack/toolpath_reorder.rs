use super::Command;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

#[derive(Debug)]
pub struct ToolpathReorderCommand {
    from: usize,
    to: usize,
}

impl ToolpathReorderCommand {
    pub fn new(from: usize, to: usize) -> Self {
        Self { from, to }
    }
}

impl Command for ToolpathReorderCommand {
    fn execute(&mut self, _shapes: &mut Vec<Item>, _layers: &mut Layers, toolpaths: &mut Vec<Toolpath>) {
        if self.from < toolpaths.len() {
            let item = toolpaths.remove(self.from);
            let target = if self.to > self.from { self.to - 1 } else { self.to };
            toolpaths.insert(target.min(toolpaths.len()), item);
        }
    }

    fn undo(&mut self, _shapes: &mut Vec<Item>, _layers: &mut Layers, toolpaths: &mut Vec<Toolpath>) {
        let target = if self.to > self.from { self.to - 1 } else { self.to };
        let target = target.min(toolpaths.len().saturating_sub(1));
        if target < toolpaths.len() {
            let item = toolpaths.remove(target);
            toolpaths.insert(self.from.min(toolpaths.len()), item);
        }
    }

    fn description(&self) -> &str {
        "Werkzeugpfad verschieben"
    }
}
