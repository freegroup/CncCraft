//! Vereinen (join): the selected shapes become one, their union.

use super::super::App;
use shape_edit::ops;

impl App {
    pub(in crate::app) fn join_selection(&mut self) {
        let shapes = self.selected_shapes();
        self.replace_selection(shapes.len(), ops::union(&shapes), t!("Formen vereint"));
    }
}
