//! Revision Tracking System (Track Changes)
//!
//! This crate implements a Word-like Track Changes system that:
//! - Tracks insertions, deletions, format changes, and moves
//! - Stores revision metadata (author, timestamp)
//! - Supports multiple display modes (Original, NoMarkup, AllMarkup, SimpleMarkup)
//! - Allows accepting/rejecting individual or groups of revisions

mod revision;
mod state;
mod error;
mod commands;

pub use revision::*;
pub use state::*;
pub use error::*;
pub use commands::*;
