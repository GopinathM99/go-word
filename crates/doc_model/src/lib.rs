//! Document Model - Core document tree structure and types
//!
//! This crate provides the foundational document model for the word processor,
//! implementing a persistent/immutable tree structure with stable node IDs.

mod node;
mod document;
mod paragraph;
mod run;
mod selection;
mod node_id;
mod tree;
mod error;
mod hyperlink;
pub mod style;
mod image;
mod bookmark;
pub mod table;
pub mod list;
pub mod shape;
pub mod textbox;
pub mod section;
pub mod field;
mod comment;
pub mod caption;
pub mod pagination;
pub mod footnote;
pub mod crossref;
pub mod content_control;
pub mod protection;

pub use node::*;
pub use document::*;
pub use paragraph::*;
pub use run::*;
pub use selection::*;
pub use node_id::*;
pub use tree::*;
pub use error::*;
pub use hyperlink::*;
pub use style::*;
pub use image::*;
pub use bookmark::*;
pub use table::*;
pub use list::*;
pub use shape::*;
pub use textbox::*;
pub use section::*;
pub use field::*;
pub use comment::*;
pub use caption::*;
pub use pagination::*;
pub use footnote::*;
pub use crossref::*;
pub use content_control::*;
pub use protection::*;
