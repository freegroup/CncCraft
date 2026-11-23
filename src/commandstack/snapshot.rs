use super::{Command, Snapshot};
use crate::app::Pattern;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

#[derive(Debug)]
pub struct SnapshotCommand {
    #[allow(dead_code)]
    description: &'static str,
    before: Option<Snapshot>,
    after: Option<Snapshot>,
}

impl SnapshotCommand {
    pub fn new(description: &'static str, before_shapes: Vec<Item>, before_layers: Layers, before_patterns: Vec<Pattern>) -> Self {
        Self {
            description,
            before: Some((before_shapes, before_layers, before_patterns)),
            after: None,
        }
    }

    pub fn capture_after(&mut self, shapes: &[Item], layers: &Layers, patterns: &[Pattern]) {
        self.after = Some((shapes.to_vec(), layers.clone(), patterns.to_vec()));
    }
}

impl Command for SnapshotCommand {
    fn execute(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>, patterns: &mut Vec<Pattern>) {
        if let Some((s, l, p)) = &self.after {
            *shapes = s.clone();
            *layers = l.clone();
            *patterns = p.clone();
        }
    }

    fn undo(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>, patterns: &mut Vec<Pattern>) {
        if self.after.is_none() {
            self.after = Some((shapes.clone(), layers.clone(), patterns.clone()));
        }
        if let Some((s, l, p)) = &self.before {
            *shapes = s.clone();
            *layers = l.clone();
            *patterns = p.clone();
        }
    }

    fn description(&self) -> &str {
        self.description
    }
}
