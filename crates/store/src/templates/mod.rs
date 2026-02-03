//! Template system for document templates
//!
//! This module provides:
//! - Template package format (.wdt - ZIP archive)
//! - Template metadata and locked regions
//! - Template CRUD operations
//! - Style pack export/import

mod metadata;
mod package;
mod manager;
mod style_pack;
mod error;

#[cfg(test)]
mod tests;

pub use metadata::*;
pub use package::*;
pub use manager::*;
pub use style_pack::*;
pub use error::*;
