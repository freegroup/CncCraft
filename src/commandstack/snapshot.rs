use super::{Command, Snapshot};
use crate::app::Pattern;
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::constraints::Distance;
use shape_edit::Item;

#[derive(Debug)]
pub struct SnapshotCommand {
    #[allow(dead_code)]
    description: &'static str,
    before: Option<Snapshot>,
    after: Option<Snapshot>,
    // the distance constraints before and after the edit, kept with the snapshot so that undo/redo
    // restores them too — in particular a constraint the edit dropped (a parallel-line dimension
    // whose lines stopped being parallel) comes back on undo.
    before_distances: Vec<Distance>,
    after_distances: Option<Vec<Distance>>,
}

impl SnapshotCommand {
    pub fn new(description: &'static str, before_shapes: Vec<Item>, before_layers: Layers, before_patterns: Vec<Pattern>, before_distances: Vec<Distance>) -> Self {
        Self {
            description,
            before: Some((before_shapes, before_layers, before_patterns)),
            after: None,
            before_distances,
            after_distances: None,
        }
    }

    pub fn capture_after(&mut self, shapes: &[Item], layers: &Layers, patterns: &[Pattern], distances: &[Distance]) {
        self.after = Some((shapes.to_vec(), layers.clone(), patterns.to_vec()));
        self.after_distances = Some(distances.to_vec());
    }
}

impl Command for SnapshotCommand {
    fn execute(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>, patterns: &mut Vec<Pattern>, distances: &mut Vec<Distance>) {
        if let Some((s, l, p)) = &self.after {
            *shapes = s.clone();
            *layers = l.clone();
            *patterns = p.clone();
        }
        if let Some(d) = &self.after_distances {
            *distances = d.clone();
        }
    }

    fn undo(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, _guides: &mut Vec<crate::guides::Guide>, patterns: &mut Vec<Pattern>, distances: &mut Vec<Distance>) {
        if self.after.is_none() {
            self.after = Some((shapes.clone(), layers.clone(), patterns.clone()));
            self.after_distances = Some(distances.clone());
        }
        if let Some((s, l, p)) = &self.before {
            *shapes = s.clone();
            *layers = l.clone();
            *patterns = p.clone();
        }
        *distances = self.before_distances.clone();
    }

    fn description(&self) -> &str {
        self.description
    }
}
