//! Lazy loading module for large documents
//!
//! This module provides infrastructure for loading documents on-demand,
//! which is critical for handling large documents without consuming excessive memory.

use crate::{Result, StoreError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

// =============================================================================
// Configuration
// =============================================================================

/// Configuration for lazy loading behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LazyLoadConfig {
    /// File size threshold in bytes above which lazy loading is used
    pub threshold_bytes: usize,
    /// Number of sections to preload ahead of current position
    pub preload_sections: usize,
    /// Maximum number of sections to keep loaded in memory
    pub max_loaded_sections: usize,
    /// Whether to use memory-mapped files for large documents
    pub use_mmap: bool,
    /// Chunk size for reading (in bytes)
    pub chunk_read_size: usize,
}

impl Default for LazyLoadConfig {
    fn default() -> Self {
        Self {
            threshold_bytes: 1024 * 1024, // 1MB threshold
            preload_sections: 2,          // Preload 2 sections ahead
            max_loaded_sections: 10,      // Keep at most 10 sections loaded
            use_mmap: false,              // Disabled by default for portability
            chunk_read_size: 64 * 1024,   // 64KB chunks
        }
    }
}

impl LazyLoadConfig {
    /// Create a config optimized for low memory usage
    pub fn low_memory() -> Self {
        Self {
            threshold_bytes: 512 * 1024, // 512KB threshold
            preload_sections: 1,
            max_loaded_sections: 5,
            use_mmap: false,
            chunk_read_size: 32 * 1024,
        }
    }

    /// Create a config optimized for performance
    pub fn high_performance() -> Self {
        Self {
            threshold_bytes: 5 * 1024 * 1024, // 5MB threshold
            preload_sections: 4,
            max_loaded_sections: 20,
            use_mmap: true,
            chunk_read_size: 256 * 1024,
        }
    }
}

// =============================================================================
// Section State
// =============================================================================

/// State of a section in a lazy-loaded document
#[derive(Debug, Clone)]
pub enum SectionState {
    /// Section is not loaded, only metadata available
    Unloaded {
        /// Estimated size in bytes
        estimated_size: usize,
    },
    /// Section is currently being loaded
    Loading,
    /// Section is fully loaded
    Loaded {
        /// The loaded section data as JSON
        data: Vec<u8>,
    },
}

impl SectionState {
    /// Check if the section is loaded
    pub fn is_loaded(&self) -> bool {
        matches!(self, SectionState::Loaded { .. })
    }

    /// Check if the section is currently loading
    pub fn is_loading(&self) -> bool {
        matches!(self, SectionState::Loading)
    }
}

// =============================================================================
// Section Index
// =============================================================================

/// Index entry for a section in the chunked file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionIndexEntry {
    /// Section index
    pub index: usize,
    /// File offset where this section starts
    pub offset: u64,
    /// Size of the section data in bytes
    pub size: usize,
    /// Number of paragraphs in this section
    pub paragraph_count: usize,
    /// Number of nodes in this section
    pub node_count: usize,
    /// Checksum for integrity verification
    pub checksum: u32,
}

/// Complete index for a chunked document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIndex {
    /// Document title
    pub title: String,
    /// Total number of sections
    pub section_count: usize,
    /// Total document size in bytes
    pub total_size: usize,
    /// Index entries for each section
    pub sections: Vec<SectionIndexEntry>,
    /// File format version
    pub version: u32,
}

// =============================================================================
// Lazy Document
// =============================================================================

/// A lazily-loaded document that loads sections on demand
#[derive(Debug)]
pub struct LazyDocument {
    /// Set of currently loaded section indices
    loaded_sections: HashSet<usize>,
    /// File offsets for each section
    section_offsets: Vec<u64>,
    /// Section sizes for each section
    section_sizes: Vec<usize>,
    /// Path to the source file
    file_path: PathBuf,
    /// Section states
    section_states: Vec<SectionState>,
    /// Document index
    index: DocumentIndex,
    /// Configuration
    config: LazyLoadConfig,
    /// Load order tracking for LRU eviction
    load_order: Vec<usize>,
}

impl LazyDocument {
    /// Open a document lazily from a file path
    pub fn open_lazy(path: &Path) -> Result<Self> {
        Self::open_lazy_with_config(path, LazyLoadConfig::default())
    }

    /// Open a document lazily with custom configuration
    pub fn open_lazy_with_config(path: &Path, config: LazyLoadConfig) -> Result<Self> {
        if !path.exists() {
            return Err(StoreError::FileNotFound(path.display().to_string()));
        }

        // Read the file to extract the index
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Read the index from the beginning of the file
        let index = Self::read_index(&mut reader)?;

        // Build section states
        let section_states: Vec<SectionState> = index
            .sections
            .iter()
            .map(|entry| SectionState::Unloaded {
                estimated_size: entry.size,
            })
            .collect();

        let section_offsets: Vec<u64> = index.sections.iter().map(|e| e.offset).collect();
        let section_sizes: Vec<usize> = index.sections.iter().map(|e| e.size).collect();

        Ok(Self {
            loaded_sections: HashSet::new(),
            section_offsets,
            section_sizes,
            file_path: path.to_path_buf(),
            section_states,
            index,
            config,
            load_order: Vec::new(),
        })
    }

    /// Read the document index from the file
    fn read_index<R: Read>(reader: &mut R) -> Result<DocumentIndex> {
        // Read index size (first 8 bytes)
        let mut size_buf = [0u8; 8];
        reader.read_exact(&mut size_buf)?;
        let index_size = u64::from_le_bytes(size_buf) as usize;

        // Read index data
        let mut index_data = vec![0u8; index_size];
        reader.read_exact(&mut index_data)?;

        // Deserialize index
        let index: DocumentIndex = serde_json::from_slice(&index_data)?;
        Ok(index)
    }

    /// Load a specific section by index
    pub fn load_section(&mut self, index: usize) -> Result<&[u8]> {
        if index >= self.section_states.len() {
            return Err(StoreError::InvalidFormat(format!(
                "Section index {} out of range (max: {})",
                index,
                self.section_states.len()
            )));
        }

        // Check if already loaded
        if self.loaded_sections.contains(&index) {
            // Move to end of load order (most recently used)
            self.load_order.retain(|&i| i != index);
            self.load_order.push(index);

            return self.get_loaded_section_data(index);
        }

        // Evict sections if needed
        self.evict_if_needed()?;

        // Mark as loading
        self.section_states[index] = SectionState::Loading;

        // Load from file
        let data = self.load_section_from_file(index)?;

        // Update state
        self.section_states[index] = SectionState::Loaded { data };
        self.loaded_sections.insert(index);
        self.load_order.push(index);

        self.get_loaded_section_data(index)
    }

    /// Load a section from the file
    fn load_section_from_file(&self, index: usize) -> Result<Vec<u8>> {
        let file = File::open(&self.file_path)?;
        let mut reader = BufReader::new(file);

        let offset = self.section_offsets[index];
        let size = self.section_sizes[index];

        // Seek to section
        reader.seek(SeekFrom::Start(offset))?;

        // Read section data
        let mut data = vec![0u8; size];
        reader.read_exact(&mut data)?;

        // Verify checksum
        let expected_checksum = self.index.sections[index].checksum;
        let actual_checksum = crate::integrity::crc32(&data);
        if actual_checksum != expected_checksum {
            return Err(StoreError::InvalidFormat(format!(
                "Section {} checksum mismatch: expected {}, got {}",
                index, expected_checksum, actual_checksum
            )));
        }

        Ok(data)
    }

    /// Get a reference to loaded section data
    fn get_loaded_section_data(&self, index: usize) -> Result<&[u8]> {
        match &self.section_states[index] {
            SectionState::Loaded { data } => Ok(data),
            _ => Err(StoreError::InvalidFormat(format!(
                "Section {} not loaded",
                index
            ))),
        }
    }

    /// Check if a section is loaded
    pub fn is_section_loaded(&self, index: usize) -> bool {
        self.loaded_sections.contains(&index)
    }

    /// Unload a specific section to free memory
    pub fn unload_section(&mut self, index: usize) {
        if index >= self.section_states.len() {
            return;
        }

        if let SectionState::Loaded { .. } = &self.section_states[index] {
            let estimated_size = self.section_sizes[index];
            self.section_states[index] = SectionState::Unloaded { estimated_size };
            self.loaded_sections.remove(&index);
            self.load_order.retain(|&i| i != index);
        }
    }

    /// Get the count of currently loaded sections
    pub fn loaded_section_count(&self) -> usize {
        self.loaded_sections.len()
    }

    /// Get the total number of sections
    pub fn section_count(&self) -> usize {
        self.section_states.len()
    }

    /// Get the file path
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Evict sections if we exceed the maximum loaded count
    fn evict_if_needed(&mut self) -> Result<()> {
        while self.loaded_sections.len() >= self.config.max_loaded_sections {
            if let Some(oldest_index) = self.load_order.first().copied() {
                self.unload_section(oldest_index);
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Preload adjacent sections in preparation for scrolling
    pub fn preload_adjacent(&mut self, current_section: usize) -> Result<()> {
        let preload_count = self.config.preload_sections;
        let total = self.section_count();

        // Preload sections ahead
        for i in 1..=preload_count {
            let next = current_section + i;
            if next < total && !self.is_section_loaded(next) {
                self.load_section(next)?;
            }
        }

        // Preload one section behind
        if current_section > 0 && !self.is_section_loaded(current_section - 1) {
            self.load_section(current_section - 1)?;
        }

        Ok(())
    }

    /// Check if file should use lazy loading based on size
    pub fn should_use_lazy_loading(path: &Path, config: &LazyLoadConfig) -> Result<bool> {
        let metadata = std::fs::metadata(path)?;
        Ok(metadata.len() as usize > config.threshold_bytes)
    }

    /// Get section info for UI display
    pub fn section_info(&self, index: usize) -> Option<SectionInfo> {
        let entry = self.index.sections.get(index)?;
        let state = self.section_states.get(index)?;

        Some(SectionInfo {
            index,
            paragraph_count: entry.paragraph_count,
            node_count: entry.node_count,
            size_bytes: entry.size,
            is_loaded: state.is_loaded(),
            is_loading: state.is_loading(),
        })
    }

    /// Get all section infos
    pub fn all_section_info(&self) -> Vec<SectionInfo> {
        (0..self.section_count())
            .filter_map(|i| self.section_info(i))
            .collect()
    }
}

// =============================================================================
// Section Info
// =============================================================================

/// Information about a section for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionInfo {
    /// Section index
    pub index: usize,
    /// Number of paragraphs
    pub paragraph_count: usize,
    /// Number of nodes
    pub node_count: usize,
    /// Size in bytes
    pub size_bytes: usize,
    /// Whether the section is loaded
    pub is_loaded: bool,
    /// Whether the section is currently loading
    pub is_loading: bool,
}

// =============================================================================
// Lazy Document Handle
// =============================================================================

/// Thread-safe handle to a lazy document
#[derive(Clone)]
pub struct LazyDocumentHandle {
    inner: Arc<RwLock<LazyDocument>>,
}

impl LazyDocumentHandle {
    /// Create a new handle from a lazy document
    pub fn new(doc: LazyDocument) -> Self {
        Self {
            inner: Arc::new(RwLock::new(doc)),
        }
    }

    /// Open a lazy document and wrap it in a handle
    pub fn open(path: &Path) -> Result<Self> {
        let doc = LazyDocument::open_lazy(path)?;
        Ok(Self::new(doc))
    }

    /// Load a section
    pub fn load_section(&self, index: usize) -> Result<()> {
        let mut doc = self
            .inner
            .write()
            .map_err(|_| StoreError::InvalidFormat("Lock poisoned".to_string()))?;
        doc.load_section(index)?;
        Ok(())
    }

    /// Check if a section is loaded
    pub fn is_section_loaded(&self, index: usize) -> Result<bool> {
        let doc = self
            .inner
            .read()
            .map_err(|_| StoreError::InvalidFormat("Lock poisoned".to_string()))?;
        Ok(doc.is_section_loaded(index))
    }

    /// Get loaded section count
    pub fn loaded_section_count(&self) -> Result<usize> {
        let doc = self
            .inner
            .read()
            .map_err(|_| StoreError::InvalidFormat("Lock poisoned".to_string()))?;
        Ok(doc.loaded_section_count())
    }

    /// Get total section count
    pub fn section_count(&self) -> Result<usize> {
        let doc = self
            .inner
            .read()
            .map_err(|_| StoreError::InvalidFormat("Lock poisoned".to_string()))?;
        Ok(doc.section_count())
    }

    /// Preload adjacent sections
    pub fn preload_adjacent(&self, current_section: usize) -> Result<()> {
        let mut doc = self
            .inner
            .write()
            .map_err(|_| StoreError::InvalidFormat("Lock poisoned".to_string()))?;
        doc.preload_adjacent(current_section)
    }

    /// Get section info
    pub fn section_info(&self, index: usize) -> Result<Option<SectionInfo>> {
        let doc = self
            .inner
            .read()
            .map_err(|_| StoreError::InvalidFormat("Lock poisoned".to_string()))?;
        Ok(doc.section_info(index))
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_load_config_default() {
        let config = LazyLoadConfig::default();
        assert_eq!(config.threshold_bytes, 1024 * 1024);
        assert_eq!(config.preload_sections, 2);
        assert_eq!(config.max_loaded_sections, 10);
    }

    #[test]
    fn test_lazy_load_config_low_memory() {
        let config = LazyLoadConfig::low_memory();
        assert_eq!(config.threshold_bytes, 512 * 1024);
        assert_eq!(config.max_loaded_sections, 5);
    }

    #[test]
    fn test_lazy_load_config_high_performance() {
        let config = LazyLoadConfig::high_performance();
        assert_eq!(config.threshold_bytes, 5 * 1024 * 1024);
        assert_eq!(config.preload_sections, 4);
    }

    #[test]
    fn test_section_state_is_loaded() {
        let unloaded = SectionState::Unloaded {
            estimated_size: 1000,
        };
        assert!(!unloaded.is_loaded());

        let loading = SectionState::Loading;
        assert!(!loading.is_loaded());
        assert!(loading.is_loading());

        let loaded = SectionState::Loaded {
            data: vec![1, 2, 3],
        };
        assert!(loaded.is_loaded());
    }
}
