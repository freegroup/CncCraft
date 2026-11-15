//! Abziehen (subtract): the shape selected first, less the others.

use super::super::App;
use shape_edit::ops;

impl App {
    pub(in crate::app) fn subtract_selection(&mut self) {
        let shapes = self.selected_shapes();
        let result = shapes.split_first().and_then(|(first, rest)| ops::subtract(first, rest));
        self.replace_selection(shapes.len(), result, t!("Formen abgezogen"));
    }
}
