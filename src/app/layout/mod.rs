//! The window's arrangement, following the original (the person's picture-12):
//!
//! - a title bar with the usual menu bar (every menu at the top level), the layer in the
//!   middle and the document's name on the right;
//! - one tool column on the left, grouped like the original's; a tool with settings opens its
//!   form beside the column;
//! - a toolbar over the drawing area only, with 2D/3D and zoom at its right end;
//! - prompts and help floating at the bottom right of the drawing;
//! - the pointer position and the selection's size along the bottom;
//! - the toolpath panel on the right: the list on top, then simulation and export;
//! - the dialogs for exporting, post-processors and tools.
//!
//! Each part has its own file here; `widgets` holds the small pieces they share.

mod canvas_bars;
mod export_dialog;
mod postprocessor_dialog;
mod title_bar;
mod tool_column;
mod tool_dialog;
mod toolpath_panel;
mod widgets;

pub(in crate::app) use tool_column::{ToolForm, GROUP_COUNT};
pub(in crate::app) use widgets::combo_with_manage;
