use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::Item;

pub type Snapshot = (Vec<Item>, Layers);

pub trait Command: std::fmt::Debug {
    fn execute(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, toolpaths: &mut Vec<Toolpath>);
    fn undo(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, toolpaths: &mut Vec<Toolpath>);
    #[allow(dead_code)]
    fn description(&self) -> &str;
}

pub struct CommandStack {
    undo: Vec<Box<dyn Command>>,
    redo: Vec<Box<dyn Command>>,
    limit: usize,
}

impl CommandStack {
    pub fn new(limit: usize) -> Self {
        Self { undo: Vec::new(), redo: Vec::new(), limit }
    }

    pub fn execute(&mut self, mut cmd: Box<dyn Command>, shapes: &mut Vec<Item>, layers: &mut Layers, toolpaths: &mut Vec<Toolpath>) {
        cmd.execute(shapes, layers, toolpaths);
        self.undo.push(cmd);
        if self.undo.len() > self.limit {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn push_executed(&mut self, cmd: Box<dyn Command>) {
        self.undo.push(cmd);
        if self.undo.len() > self.limit {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn undo(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, toolpaths: &mut Vec<Toolpath>) -> bool {
        if let Some(mut cmd) = self.undo.pop() {
            cmd.undo(shapes, layers, toolpaths);
            self.redo.push(cmd);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, toolpaths: &mut Vec<Toolpath>) -> bool {
        if let Some(mut cmd) = self.redo.pop() {
            cmd.execute(shapes, layers, toolpaths);
            self.undo.push(cmd);
            true
        } else {
            false
        }
    }

    pub fn pop_last(&mut self) -> Option<Box<dyn Command>> {
        self.undo.pop()
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}
