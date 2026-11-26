use crate::layers::Layers;
use crate::toolpath::Toolpath;
use crate::guides::Guide;
use shape_edit::Item;

pub type Snapshot = (Vec<Item>, Layers, Vec<crate::app::Pattern>);

pub trait Command: std::fmt::Debug {
    fn execute(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, toolpaths: &mut Vec<Toolpath>, guides: &mut Vec<Guide>, patterns: &mut Vec<crate::app::Pattern>);
    fn undo(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, toolpaths: &mut Vec<Toolpath>, guides: &mut Vec<Guide>, patterns: &mut Vec<crate::app::Pattern>);
    #[allow(dead_code)]
    fn description(&self) -> &str;
}

/// What an undo or redo changed, so the host can refresh only the caches it owns (the selection,
/// the derived variables, the toolpath simulation) without reaching into the stack's bookkeeping.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HistoryChange {
    /// Whether anything was applied at all (false when there was nothing to undo/redo).
    pub changed: bool,
    /// Whether undoing dropped pasted shapes and so truncated the toolpaths (view caches stale).
    pub toolpaths_truncated: bool,
}

pub struct CommandStack {
    undo: Vec<Box<dyn Command>>,
    redo: Vec<Box<dyn Command>>,
    limit: usize,
    /// Paste bookkeeping the stack owns: each entry is (the ids a paste added, the toolpath count
    /// before it). When an undo removes those shapes, the stack also truncates the toolpaths the
    /// paste grew — toolpaths are not part of the edit snapshot, so the history must track this
    /// itself rather than leave it to the host.
    paste_log: Vec<(Vec<u64>, usize)>,
}

impl CommandStack {
    pub fn new(limit: usize) -> Self {
        Self { undo: Vec::new(), redo: Vec::new(), limit, paste_log: Vec::new() }
    }

    /// Record that a paste added `ids` while the toolpath list stood at `toolpaths_before`.
    pub fn note_paste(&mut self, ids: Vec<u64>, toolpaths_before: usize) {
        self.paste_log.push((ids, toolpaths_before));
    }

    pub fn execute(&mut self, mut cmd: Box<dyn Command>, shapes: &mut Vec<Item>, layers: &mut Layers, toolpaths: &mut Vec<Toolpath>, guides: &mut Vec<Guide>, patterns: &mut Vec<crate::app::Pattern>) {
        cmd.execute(shapes, layers, toolpaths, guides, patterns);
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

    pub fn undo(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, toolpaths: &mut Vec<Toolpath>, guides: &mut Vec<Guide>, patterns: &mut Vec<crate::app::Pattern>) -> HistoryChange {
        let Some(mut cmd) = self.undo.pop() else { return HistoryChange::default() };
        cmd.undo(shapes, layers, toolpaths, guides, patterns);
        self.redo.push(cmd);
        // undoing may have removed pasted shapes: drop the toolpaths those pastes grew
        let truncated = self.truncate_undone_pastes(shapes, toolpaths);
        HistoryChange { changed: true, toolpaths_truncated: truncated }
    }

    pub fn redo(&mut self, shapes: &mut Vec<Item>, layers: &mut Layers, toolpaths: &mut Vec<Toolpath>, guides: &mut Vec<Guide>, patterns: &mut Vec<crate::app::Pattern>) -> HistoryChange {
        let Some(mut cmd) = self.redo.pop() else { return HistoryChange::default() };
        cmd.execute(shapes, layers, toolpaths, guides, patterns);
        self.undo.push(cmd);
        HistoryChange { changed: true, toolpaths_truncated: false }
    }

    /// While the newest paste's shapes are all gone (undone), roll the toolpaths back to the count
    /// before that paste. Returns whether any toolpaths were dropped.
    fn truncate_undone_pastes(&mut self, shapes: &[Item], toolpaths: &mut Vec<Toolpath>) -> bool {
        let mut truncated = false;
        while let Some((ids, before)) = self.paste_log.last() {
            if ids.iter().all(|id| !shapes.iter().any(|s| s.id == *id)) {
                toolpaths.truncate((*before).min(toolpaths.len()));
                truncated = true;
                self.paste_log.pop();
            } else {
                break;
            }
        }
        truncated
    }

    pub fn pop_last(&mut self) -> Option<Box<dyn Command>> {
        self.undo.pop()
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.paste_log.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}
