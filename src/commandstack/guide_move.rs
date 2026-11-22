use super::Command;
use crate::guides::{Guide, GuideAxis};
use crate::layers::Layers;
use crate::toolpath::Toolpath;
use shape_edit::{Editable, Item};

/// Move a construction line from one offset to another; its soft-attached shapes move with it.
/// The new formula (`expr`) is set too — `None` for a plain number (a hand-moved line), or the
/// formula the person typed. One undo step covers the line, the shapes and the formula.
#[derive(Debug)]
pub struct GuideMoveCommand {
    id: u64,
    from: f64,
    to: f64,
    /// The formula to put on the line (the typed one, or `None` to make it a plain number).
    new_expr: Option<String>,
    /// The formula the line had before the move (restored on undo).
    old_expr: Option<String>,
    captured: bool,
}

impl GuideMoveCommand {
    pub fn new(id: u64, from: f64, to: f64) -> Self {
        Self { id, from, to, new_expr: None, old_expr: None, captured: false }
    }

    /// Set a formula on the line as part of the move (so typing a formula is one undo step).
    pub fn with_expr(mut self, expr: Option<String>) -> Self {
        self.new_expr = expr;
        self
    }

    /// Shift the shapes attached to guide `id` by `delta` along its axis.
    fn shift_attached(id: u64, delta: f64, shapes: &mut [Item], guides: &[Guide]) {
        let Some(g) = guides.iter().find(|g| g.id == id) else { return };
        let (dx, dy) = if g.axis == GuideAxis::Vertical { (delta, 0.0) } else { (0.0, delta) };
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        let attached: Vec<u64> = g.attached.iter().map(|a| a.shape).collect();
        for s in shapes.iter_mut() {
            if attached.contains(&s.id) {
                *s = s.translated(dx, dy);
            }
        }
    }
}

impl Command for GuideMoveCommand {
    fn execute(&mut self, shapes: &mut Vec<Item>, _layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, guides: &mut Vec<Guide>) {
        Self::shift_attached(self.id, self.to - self.from, shapes, guides);
        if let Some(g) = guides.iter_mut().find(|g| g.id == self.id) {
            if !self.captured {
                self.old_expr = g.expr.clone();
                self.captured = true;
            }
            g.expr = self.new_expr.clone();
            g.offset = self.to;
        }
    }

    fn undo(&mut self, shapes: &mut Vec<Item>, _layers: &mut Layers, _toolpaths: &mut Vec<Toolpath>, guides: &mut Vec<Guide>) {
        Self::shift_attached(self.id, self.from - self.to, shapes, guides);
        if let Some(g) = guides.iter_mut().find(|g| g.id == self.id) {
            g.offset = self.from;
            g.expr = self.old_expr.clone();
        }
    }

    fn description(&self) -> &str {
        "Hilfslinie verschieben"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guides::{Attachment, GuideAxis, Side};
    use shape_edit::Draw;

    #[test]
    fn moving_a_guide_drags_its_shape_and_undo_puts_both_back() {
        let mut guides = vec![Guide { id: 1, axis: GuideAxis::Vertical, offset: 50.0, expr: None, attached: vec![Attachment { shape: 9, side: Side::Low }] }];
        let mut shapes = vec![Item::new(9, shape_edit::Rect::from_drag([0.0, 0.0], [10.0, 10.0]))];
        let (mut layers, mut tps) = (Layers::default(), vec![]);
        let mut cmd = GuideMoveCommand::new(1, 50.0, 80.0);
        cmd.execute(&mut shapes, &mut layers, &mut tps, &mut guides);
        assert_eq!(guides[0].offset, 80.0);
        assert!((shapes[0].shape.bounds().unwrap().0[0] - 30.0).abs() < 1e-9, "shape followed by +30 in x");
        cmd.undo(&mut shapes, &mut layers, &mut tps, &mut guides);
        assert_eq!(guides[0].offset, 50.0);
        assert!((shapes[0].shape.bounds().unwrap().0[0]).abs() < 1e-9, "shape back at x=0");
    }
}
