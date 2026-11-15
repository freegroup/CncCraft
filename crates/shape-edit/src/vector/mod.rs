//! Vector operations: cutting a vector where it crosses others ([`trim`]), joining points with
//! a line ([`connect`]) and turning corners into true arcs ([`fillet`]). Plain geometry, no UI.

pub mod connect;
pub mod fillet;
pub mod trim;

pub use fillet::{Arc, Corner, CornerKind, Piece};
