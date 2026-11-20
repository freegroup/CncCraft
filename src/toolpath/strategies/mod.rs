//! The cutting strategies, one file each. A strategy implements [`super::ToolpathKind`]; the
//! pocket's three ways of clearing (offset rings, raster, spiral) each have their own file too.

mod drill;
mod pocket;
mod pocket_offset;
mod pocket_raster;
mod pocket_spiral;
mod profile;
mod slot;
mod vcarve;

#[cfg(test)]
pub use drill::peck_depths;
pub use drill::DrillTool;
pub use pocket::{Clearing, PocketTool};
pub use profile::ProfileTool;
pub use slot::{Cap, SlotTool};
pub use vcarve::VCarveTool;
