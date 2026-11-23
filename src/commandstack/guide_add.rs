use super::Command;
use crate::guides::Guide;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

/// Add a construction line (undo removes it again).
#[derive(Debug)]
pub struct GuideAddCommand {
    guide: Guide,
    added: bool,
}

impl GuideAddCommand {
    pub fn new(guide: Guide) -> Self {
        Self { guide, added: false }
    }
}

impl Command for GuideAddCommand {
    fn execute(&mut self, _shapes: &mut Vec<Item>, _layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, guides: &mut Vec<Guide>, _patterns: &mut Vec<crate::app::Pattern>) {
        if !guides.iter().any(|g| g.id == self.guide.id) {
            guides.push(self.guide.clone());
            self.added = true;
        }
    }

    fn undo(&mut self, _shapes: &mut Vec<Item>, _layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, guides: &mut Vec<Guide>, _patterns: &mut Vec<crate::app::Pattern>) {
        if self.added {
            guides.retain(|g| g.id != self.guide.id);
        }
    }

    fn description(&self) -> &str {
        "Hilfslinie anlegen"
    }
}
