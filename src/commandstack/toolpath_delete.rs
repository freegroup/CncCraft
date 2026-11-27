use super::Command;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

#[derive(Debug)]
pub struct ToolpathDeleteCommand {
    index: usize,
    removed: Option<Toolpath>,
}

impl ToolpathDeleteCommand {
    pub fn new(index: usize) -> Self {
        Self { index, removed: None }
    }
}

impl Command for ToolpathDeleteCommand {
    fn execute(&mut self, _shapes: &mut Vec<Item>, _layers: &mut Layers, toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>, _patterns: &mut Vec<crate::app::Pattern>, _distances: &mut Vec<shape_edit::constraints::Distance>) {
        if self.index < toolpaths.len() {
            self.removed = Some(toolpaths.remove(self.index));
        }
    }

    fn undo(&mut self, _shapes: &mut Vec<Item>, _layers: &mut Layers, toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>, _patterns: &mut Vec<crate::app::Pattern>, _distances: &mut Vec<shape_edit::constraints::Distance>) {
        if let Some(tp) = self.removed.take() {
            toolpaths.insert(self.index.min(toolpaths.len()), tp);
        }
    }

    fn description(&self) -> &str {
        "Werkzeugpfad löschen"
    }
}
