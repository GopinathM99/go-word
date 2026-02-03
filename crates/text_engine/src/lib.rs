//! Text Engine - Text shaping, font discovery, and metrics
//!
//! This crate handles text shaping using rustybuzz and provides
//! font discovery, fallback resolution, and metric calculations.
//!
//! # Modules
//!
//! - `shaper`: Text shaping using rustybuzz
//! - `font`: Font types and basic font management
//! - `metrics`: Text metric calculations
//! - `discovery`: System font discovery and enumeration
//! - `fallback`: Font fallback chains and substitution rules
//! - `font_manager`: Central font management integrating all components
//! - `spellcheck`: Spell checking and dictionary support

mod shaper;
mod font;
mod metrics;
mod error;
pub mod discovery;
pub mod fallback;
pub mod font_manager;
pub mod spellcheck;

pub use shaper::*;
pub use font::*;
pub use metrics::*;
pub use error::*;

// Re-export commonly used types from submodules
pub use discovery::{FontDiscovery, FontIndex, FontInfo};
pub use fallback::{FallbackChain, FontResolution, Script, SubstitutionReason, SubstitutionWarning};
pub use font_manager::{FontManager, FontManagerConfig, FontSubstitutionRecord, FontSubstitutionSummary, LoadedFont, LoadedFontId};
pub use spellcheck::{DictionarySpellChecker, IgnoreRules, Language, SpellChecker, SpellingError};
