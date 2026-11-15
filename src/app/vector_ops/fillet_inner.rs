//! Abrunden (inner fillet): the corner clicked is rounded off by a true circular arc that touches
//! both edges. The corner point stays; the radius can be changed or the rounding taken away at any
//! time (see the shared corner form in `vector_ops`).

use super::super::App;
use shape_edit::{CornerKind, Pt};

impl App {
    pub(super) fn fillet_inner_click(&mut self, p: Pt) {
        self.corner_click(p, CornerKind::Round);
    }
}
