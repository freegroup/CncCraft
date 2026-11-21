//! Construction lines (guides), like Sketch's ruler guides: red lines across the whole workpiece
//! that other shapes can snap to. A guide is a soft constraint — once a shape has snapped to a
//! guide, it moves along when the guide is dragged or its distance is changed, but dragging the
//! shape itself away releases it.
//!
//! The guide owns its attachments: it knows which shapes hang on it and by which edge (or their
//! centre). A shape does not know which guide it hangs on. Shapes only snap through user
//! interaction (dragging a shape onto a guide); moving a guide onto a resting shape never attaches
//! it, even within the snap radius.
//!
//! Guides are only grabbed and moved on the ruler (as in Sketch), never in the drawing area; there
//! they have no interaction at all. The original has the same feature: `vcGuideLine`,
//! `vcGuideLineManager`, `uvs::RulerManager`, `uvs::GuideLinesDrawer` and
//! `GuideLineEditBehaviour` (MECHANICS.md Nachtrag 84).

use serde::{Deserialize, Serialize};

/// Which way a guide runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuideAxis {
    /// A vertical line at a fixed distance from the left edge (an X value).
    Vertical,
    /// A horizontal line at a fixed distance from the bottom edge (a Y value).
    Horizontal,
}

/// Which part of a shape sits on a guide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    /// The low edge (left for vertical guides, bottom for horizontal).
    Low,
    /// The shape's centre.
    Centre,
    /// The high edge (right for vertical guides, top for horizontal).
    High,
}

/// One shape resting on a guide, and by which part.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Attachment {
    pub shape: u64,
    pub side: Side,
}

/// A construction line.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Guide {
    pub id: u64,
    pub axis: GuideAxis,
    /// Distance from the left edge (vertical) or the bottom edge (horizontal), in mm.
    pub offset: f64,
    /// The shapes soft-attached to this guide. The guide owns these; a shape doesn't know.
    #[serde(default)]
    pub attached: Vec<Attachment>,
}

impl Guide {
    pub fn new(id: u64, axis: GuideAxis, offset: f64) -> Guide {
        Guide { id, axis, offset, attached: vec![] }
    }

    /// Attach `shape` by `side` (or move it to this side if already attached).
    pub fn attach(&mut self, shape: u64, side: Side) {
        match self.attached.iter_mut().find(|a| a.shape == shape) {
            Some(a) => a.side = side,
            None => self.attached.push(Attachment { shape, side }),
        }
    }

    /// Release `shape` from this guide; returns whether it was attached.
    pub fn detach(&mut self, shape: u64) -> bool {
        let before = self.attached.len();
        self.attached.retain(|a| a.shape != shape);
        self.attached.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_guide_owns_its_attachments_and_a_shape_attaches_once() {
        let mut g = Guide::new(1, GuideAxis::Vertical, 100.0);
        g.attach(7, Side::Low);
        g.attach(7, Side::High); // same shape, new side: still one entry
        assert_eq!(g.attached, vec![Attachment { shape: 7, side: Side::High }]);
        assert!(g.detach(7));
        assert!(!g.detach(7));
        assert!(g.attached.is_empty());
    }
}
