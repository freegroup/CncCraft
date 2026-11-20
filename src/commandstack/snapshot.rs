use super::{Command, Snapshot};
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
    pub fn new(description: &'static str, before_shapes: Vec<Item>, before_layers: Layers) -> Self {
        Self {
            description,
            before: Some((before_shapes, before_layers)),
            after: None,
        }
    }

    pub fn capture_after(&mut self, shapes: &[Item], layers: &Layers) {
        self.after = Some((shapes.to_vec(), layers.clone()));
    }
}

impl Command for SnapshotCommand {
    fn execute(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>) {
        if let Some((s, l)) = &self.after {
            *shapes = s.clone();
            *layers = l.clone();
        }
    }

    fn undo(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>) {
        if self.after.is_none() {
            self.after = Some((shapes.clone(), layers.clone()));
        }
        if let Some((s, l)) = &self.before {
            *shapes = s.clone();
            *layers = l.clone();
        }
    }

    fn description(&self) -> &str {
        self.description
    }
}
