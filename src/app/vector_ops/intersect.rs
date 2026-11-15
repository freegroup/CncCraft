//! Schnittmenge (intersect): the selected shapes become what they all cover.

use super::super::App;
use shape_edit::ops;

impl App {
    pub(in crate::app) fn intersect_selection(&mut self) {
        let shapes = self.selected_shapes();
        self.replace_selection(shapes.len(), ops::intersect(&shapes), t!("Schnittmenge gebildet"));
    }
}
