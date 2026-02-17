//! Charts - Chart import, rendering, and export for Go Word
//!
//! This crate provides support for:
//! - Parsing DrawingML chart XML from DOCX packages
//! - Representing charts with a strongly-typed model
//! - Calculating chart layouts
//! - Rendering charts to SVG or render primitives
//! - Writing charts back to DrawingML XML
//! - Editing charts with commands (undo/redo support)
//! - Spreadsheet-like data editing
//! - Style presets and color schemes
//! - Chart insertion wizard

mod model;
mod error;
mod drawingml_parser;
mod drawingml_writer;
mod layout;
mod render;
mod commands;
mod editor;
mod styles;
mod wizard;

pub use model::*;
pub use error::*;
pub use drawingml_parser::*;
pub use drawingml_writer::*;
pub use layout::*;
pub use render::*;
pub use commands::*;
pub use editor::*;
pub use styles::*;
pub use wizard::*;
