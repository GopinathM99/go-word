//! Chunked document format for large documents
//!
//! This module defines a chunked file format that enables lazy loading
//! by storing each section as a separate compressed chunk with an index.

use crate::{DocumentIndex, LazyLoadConfig, Result, SectionIndexEntry, StoreError};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

// =============================================================================
// Constants
// =============================================================================

/// Magic bytes for chunked format identification
pub const CHUNKED_MAGIC: &[u8; 8] = b"MSWCHUNK";

/// Current chunked format version
pub const CHUNKED_VERSION: u32 = 1;

/// Default minimum section size (in paragraphs) before splitting
pub const MIN_SECTION_SIZE: usize = 50;

/// Default maximum section size (in paragraphs) before splitting
pub const MAX_SECTION_SIZE: usize = 500;

// =============================================================================
// Chunked Section
// =============================================================================

/// A section chunk containing serialized content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkedSection {
    /// Section index
    pub index: usize,
    /// Serialized content (JSON)
    pub content: String,
    /// Number of paragraphs
    pub paragraph_count: usize,
    /// Number of nodes
    pub node_count: usize,
}

impl ChunkedSection {
    /// Create a new chunked section
    pub fn new(index: usize, content: String, paragraph_count: usize, node_count: usize) -> Self {
        Self {
            index,
            content,
            paragraph_count,
            node_count,
        }
    }

    /// Get the approximate size in bytes when serialized
    pub fn estimated_size(&self) -> usize {
        self.content.len() + 100 // Content + overhead
    }
}

// =============================================================================
// Chunked Document
// =============================================================================

/// A document split into chunks for efficient storage and lazy loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkedDocument {
    /// Document title
    pub title: String,
    /// Sections (chunks)
    pub sections: Vec<ChunkedSection>,
    /// Style registry data (serialized)
    pub styles_data: String,
    /// Numbering registry data (serialized)
    pub numbering_data: String,
}

impl ChunkedDocument {
    /// Create a new chunked document
    pub fn new(title: String) -> Self {
        Self {
            title,
            sections: Vec::new(),
            styles_data: "{}".to_string(),
            numbering_data: "{}".to_string(),
        }
    }

    /// Add a section to the document
    pub fn add_section(&mut self, section: ChunkedSection) {
        self.sections.push(section);
    }

    /// Get the total number of sections
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }
}

// =============================================================================
// Chunked File Writer
// =============================================================================

/// Writer for chunked document format
pub struct ChunkedFileWriter<W: Write + Seek> {
    writer: BufWriter<W>,
    compression_level: Compression,
}

impl<W: Write + Seek> ChunkedFileWriter<W> {
    /// Create a new chunked file writer
    pub fn new(writer: W) -> Self {
        Self {
            writer: BufWriter::new(writer),
            compression_level: Compression::default(),
        }
    }

    /// Set compression level
    pub fn with_compression(mut self, level: Compression) -> Self {
        self.compression_level = level;
        self
    }

    /// Write a chunked document to the file
    pub fn write_chunked(&mut self, doc: &ChunkedDocument) -> Result<()> {
        // Write magic bytes
        self.writer.write_all(CHUNKED_MAGIC)?;

        // Write version
        self.writer.write_all(&CHUNKED_VERSION.to_le_bytes())?;

        // Prepare index entries
        let mut index_entries: Vec<SectionIndexEntry> = Vec::with_capacity(doc.sections.len());

        // Calculate header size to reserve space
        let index_placeholder_size = self.estimate_index_size(doc.sections.len());

        // Write placeholder for index size
        let index_size_pos = self.writer.stream_position()?;
        self.writer.write_all(&[0u8; 8])?;

        // Reserve space for index
        let index_data_pos = self.writer.stream_position()?;
        let placeholder = vec![0u8; index_placeholder_size];
        self.writer.write_all(&placeholder)?;

        // Write each section
        for section in &doc.sections {
            let offset = self.writer.stream_position()?;

            // Serialize and compress section
            let section_json = serde_json::to_vec(section)?;
            let compressed = self.compress(&section_json)?;

            // Calculate checksum
            let checksum = crate::integrity::crc32(&compressed);

            // Write compressed data
            self.writer.write_all(&compressed)?;

            // Create index entry
            index_entries.push(SectionIndexEntry {
                index: section.index,
                offset,
                size: compressed.len(),
                paragraph_count: section.paragraph_count,
                node_count: section.node_count,
                checksum,
            });
        }

        // Create the index
        let index = DocumentIndex {
            title: doc.title.clone(),
            section_count: doc.sections.len(),
            total_size: self.writer.stream_position()? as usize,
            sections: index_entries,
            version: CHUNKED_VERSION,
        };

        // Serialize index
        let index_json = serde_json::to_vec(&index)?;

        // Write index at reserved position
        let current_pos = self.writer.stream_position()?;

        // Write index size
        self.writer.seek(SeekFrom::Start(index_size_pos))?;
        self.writer
            .write_all(&(index_json.len() as u64).to_le_bytes())?;

        // Write index data
        self.writer.seek(SeekFrom::Start(index_data_pos))?;
        self.writer.write_all(&index_json)?;

        // Return to end
        self.writer.seek(SeekFrom::Start(current_pos))?;

        self.writer.flush()?;
        Ok(())
    }

    /// Compress data using gzip
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), self.compression_level);
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    /// Estimate index size based on section count
    fn estimate_index_size(&self, section_count: usize) -> usize {
        1000 + section_count * 250
    }
}

// =============================================================================
// Chunked File Reader
// =============================================================================

/// Reader for chunked document format
pub struct ChunkedFileReader<R: Read + Seek> {
    reader: BufReader<R>,
}

impl<R: Read + Seek> ChunkedFileReader<R> {
    /// Create a new chunked file reader
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
        }
    }

    /// Read and verify the file header
    pub fn read_header(&mut self) -> Result<()> {
        let mut magic = [0u8; 8];
        self.reader.read_exact(&mut magic)?;

        if &magic != CHUNKED_MAGIC {
            return Err(StoreError::InvalidFormat(
                "Invalid chunked file magic bytes".to_string(),
            ));
        }

        let mut version_bytes = [0u8; 4];
        self.reader.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);

        if version > CHUNKED_VERSION {
            return Err(StoreError::InvalidFormat(format!(
                "Unsupported chunked format version: {}",
                version
            )));
        }

        Ok(())
    }

    /// Read the document index
    pub fn read_index(&mut self) -> Result<DocumentIndex> {
        // Read index size
        let mut size_buf = [0u8; 8];
        self.reader.read_exact(&mut size_buf)?;
        let index_size = u64::from_le_bytes(size_buf) as usize;

        // Read index data
        let mut index_data = vec![0u8; index_size];
        self.reader.read_exact(&mut index_data)?;

        // Deserialize
        let index: DocumentIndex = serde_json::from_slice(&index_data)?;
        Ok(index)
    }

    /// Read a specific section by index
    pub fn read_section(&mut self, index: &DocumentIndex, section_idx: usize) -> Result<ChunkedSection> {
        let entry = index.sections.get(section_idx).ok_or_else(|| {
            StoreError::InvalidFormat(format!("Section {} not found in index", section_idx))
        })?;

        // Seek to section
        self.reader.seek(SeekFrom::Start(entry.offset))?;

        // Read compressed data
        let mut data = vec![0u8; entry.size];
        self.reader.read_exact(&mut data)?;

        // Verify checksum
        let actual_checksum = crate::integrity::crc32(&data);
        if actual_checksum != entry.checksum {
            return Err(StoreError::InvalidFormat(format!(
                "Section {} checksum mismatch",
                section_idx
            )));
        }

        // Decompress
        let decompressed = self.decompress(&data)?;

        // Deserialize
        let section: ChunkedSection = serde_json::from_slice(&decompressed)?;
        Ok(section)
    }

    /// Read all sections
    pub fn read_all(&mut self) -> Result<ChunkedDocument> {
        self.reader.seek(SeekFrom::Start(0))?;
        self.read_header()?;

        let index = self.read_index()?;

        let mut sections = Vec::with_capacity(index.section_count);
        for i in 0..index.section_count {
            let section = self.read_section(&index, i)?;
            sections.push(section);
        }

        Ok(ChunkedDocument {
            title: index.title,
            sections,
            styles_data: "{}".to_string(),
            numbering_data: "{}".to_string(),
        })
    }

    /// Decompress data using gzip
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
}

// =============================================================================
// Public API Functions
// =============================================================================

/// Save a chunked document to a file
pub fn save_chunked(doc: &ChunkedDocument, path: &Path) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = ChunkedFileWriter::new(file);
    writer.write_chunked(doc)
}

/// Load a chunked document from a file
pub fn load_chunked(path: &Path) -> Result<ChunkedDocument> {
    let file = File::open(path)?;
    let mut reader = ChunkedFileReader::new(file);
    reader.read_all()
}

/// Check if a file is in chunked format
pub fn is_chunked_format(path: &Path) -> Result<bool> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 8];

    if file.read_exact(&mut magic).is_err() {
        return Ok(false);
    }

    Ok(&magic == CHUNKED_MAGIC)
}

/// Get document info without loading content
pub fn get_chunked_info(path: &Path) -> Result<DocumentIndex> {
    let file = File::open(path)?;
    let mut reader = ChunkedFileReader::new(file);
    reader.read_header()?;
    reader.read_index()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_chunked_section_creation() {
        let section = ChunkedSection::new(0, "test content".to_string(), 5, 10);
        assert_eq!(section.index, 0);
        assert_eq!(section.paragraph_count, 5);
        assert_eq!(section.node_count, 10);
    }

    #[test]
    fn test_chunked_document_creation() {
        let mut doc = ChunkedDocument::new("Test Document".to_string());
        doc.add_section(ChunkedSection::new(0, "section 0".to_string(), 3, 6));
        doc.add_section(ChunkedSection::new(1, "section 1".to_string(), 4, 8));

        assert_eq!(doc.section_count(), 2);
    }

    #[test]
    fn test_chunked_file_roundtrip() {
        let mut doc = ChunkedDocument::new("Test Document".to_string());
        doc.add_section(ChunkedSection::new(0, "section 0 content".to_string(), 3, 6));
        doc.add_section(ChunkedSection::new(1, "section 1 content".to_string(), 4, 8));

        // Write to buffer
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut writer = ChunkedFileWriter::new(&mut buffer);
            writer.write_chunked(&doc).unwrap();
        }

        // Read back
        buffer.set_position(0);
        let mut reader = ChunkedFileReader::new(buffer);
        let restored = reader.read_all().unwrap();

        assert_eq!(doc.title, restored.title);
        assert_eq!(doc.section_count(), restored.section_count());
    }
}
