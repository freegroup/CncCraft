//! Geometry used by the toolpaths and the canvas, from the shape library.
//! All coordinates are millimetres in job space: X to the right, Y up, origin at the
//! material's lower-left corner.

pub use shape_edit::geometry::{bounds_of, dist, perimeter, signed_area, Contour};
pub use shape_edit::{Item, Pt, Region, Ring};

