//! Ohr (outer fillet): the corner clicked gets a circle round the corner point; the two edges end
//! where they meet it, and the outline runs round the outside of the circle, so a round cutter
//! reaches right into the corner. The corner point stays; the radius can be changed or the ear
//! taken away at any time (see the shared corner form in `vector_ops`).

use super::super::App;
use shape_edit::{CornerKind, Pt};

impl App {
    pub(super) fn fillet_outer_click(&mut self, p: Pt) {
        self.corner_click(p, CornerKind::Ear);
    }
}
