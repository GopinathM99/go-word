//! Template package format (.wdt)
//!
//! A .wdt file is a ZIP archive containing:
//! - template.json: Template metadata
//! - document.wdj: Base document in the internal format
//! - thumbnail.png: Preview image (optional)
//! - resources/: Directory for embedded images, fonts, etc.

use super::{TemplateError, TemplateMetadata, TemplateResult};
use doc_model::DocumentTree;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// File names within the template package
pub const METADATA_FILE: &str = "template.json";
pub const DOCUMENT_FILE: &str = "document.wdj";
pub const THUMBNAIL_FILE: &str = "thumbnail.png";
pub const RESOURCES_DIR: &str = "resources/";

/// Template package extension
pub const TEMPLATE_EXTENSION: &str = "wdt";

/// A complete template package
#[derive(Debug, Clone)]
pub struct TemplatePackage {
    /// Template metadata
    pub metadata: TemplateMetadata,
    /// Base document
    pub document: DocumentTree,
    /// Thumbnail image data (PNG format)
    pub thumbnail: Option<Vec<u8>>,
    /// Embedded resources (filename -> data)
    pub resources: std::collections::HashMap<String, Vec<u8>>,
}

impl TemplatePackage {
    /// Create a new template package from a document
    pub fn new(document: DocumentTree, metadata: TemplateMetadata) -> Self {
        Self {
            metadata,
            document,
            thumbnail: None,
            resources: std::collections::HashMap::new(),
        }
    }

    /// Add a thumbnail image
    pub fn with_thumbnail(mut self, thumbnail: Vec<u8>) -> Self {
        self.thumbnail = Some(thumbnail);
        self.metadata.has_thumbnail = true;
        self
    }

    /// Add a resource file
    pub fn with_resource(mut self, name: impl Into<String>, data: Vec<u8>) -> Self {
        self.resources.insert(name.into(), data);
        self
    }

    /// Read a template package from a file
    pub fn read_from_file(path: impl AsRef<Path>) -> TemplateResult<Self> {
        let file = File::open(path)?;
        Self::read_from_reader(file)
    }

    /// Read a template package from a reader
    pub fn read_from_reader<R: Read + std::io::Seek>(reader: R) -> TemplateResult<Self> {
        let mut archive = ZipArchive::new(reader)?;

        // Read metadata
        let metadata = {
            let mut file = archive
                .by_name(METADATA_FILE)
                .map_err(|_| TemplateError::MissingFile(METADATA_FILE.to_string()))?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            serde_json::from_str::<TemplateMetadata>(&contents)?
        };

        // Read document
        let document = {
            let mut file = archive
                .by_name(DOCUMENT_FILE)
                .map_err(|_| TemplateError::MissingFile(DOCUMENT_FILE.to_string()))?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            crate::deserialize(&contents)?
        };

        // Read thumbnail (optional)
        let thumbnail = match archive.by_name(THUMBNAIL_FILE) {
            Ok(mut file) => {
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;
                Some(data)
            }
            Err(_) => None,
        };

        // Read resources
        let mut resources = std::collections::HashMap::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();
            if name.starts_with(RESOURCES_DIR) && !name.ends_with('/') {
                let resource_name = name.strip_prefix(RESOURCES_DIR).unwrap_or(&name);
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;
                resources.insert(resource_name.to_string(), data);
            }
        }

        Ok(Self {
            metadata,
            document,
            thumbnail,
            resources,
        })
    }

    /// Write the template package to a file
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> TemplateResult<()> {
        let file = File::create(path)?;
        self.write_to_writer(file)
    }

    /// Write the template package to a writer
    pub fn write_to_writer<W: Write + std::io::Seek>(&self, writer: W) -> TemplateResult<()> {
        let mut zip = ZipWriter::new(writer);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(6));

        // Write metadata
        let metadata_json = serde_json::to_string_pretty(&self.metadata)?;
        zip.start_file(METADATA_FILE, options)?;
        zip.write_all(metadata_json.as_bytes())?;

        // Write document
        let document_json = crate::serialize(&self.document)?;
        zip.start_file(DOCUMENT_FILE, options)?;
        zip.write_all(document_json.as_bytes())?;

        // Write thumbnail if present
        if let Some(ref thumbnail) = self.thumbnail {
            zip.start_file(THUMBNAIL_FILE, options)?;
            zip.write_all(thumbnail)?;
        }

        // Write resources
        for (name, data) in &self.resources {
            let resource_path = format!("{}{}", RESOURCES_DIR, name);
            zip.start_file(&resource_path, options)?;
            zip.write_all(data)?;
        }

        zip.finish()?;
        Ok(())
    }

    /// Extract the document from the template (creates a new document with new IDs)
    pub fn create_document(&self) -> DocumentTree {
        // Clone the document - in a real implementation, we would regenerate node IDs
        // to ensure each document created from the template has unique IDs
        self.document.clone()
    }

    /// Get the template ID
    pub fn id(&self) -> &str {
        &self.metadata.id
    }

    /// Get the template name
    pub fn name(&self) -> &str {
        &self.metadata.name
    }
}

/// Read only the metadata from a template package (faster than reading the whole package)
pub fn read_metadata(path: impl AsRef<Path>) -> TemplateResult<TemplateMetadata> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut file = archive
        .by_name(METADATA_FILE)
        .map_err(|_| TemplateError::MissingFile(METADATA_FILE.to_string()))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(serde_json::from_str(&contents)?)
}

/// Read only the thumbnail from a template package
pub fn read_thumbnail(path: impl AsRef<Path>) -> TemplateResult<Option<Vec<u8>>> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    let result = match archive.by_name(THUMBNAIL_FILE) {
        Ok(mut file) => {
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            Ok(Some(data))
        }
        Err(_) => Ok(None),
    };
    result
}

/// Validate that a file is a valid template package
pub fn validate_package(path: impl AsRef<Path>) -> TemplateResult<()> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    // Check for required files
    archive
        .by_name(METADATA_FILE)
        .map_err(|_| TemplateError::MissingFile(METADATA_FILE.to_string()))?;

    archive
        .by_name(DOCUMENT_FILE)
        .map_err(|_| TemplateError::MissingFile(DOCUMENT_FILE.to_string()))?;

    // Try to parse metadata to validate format
    let mut metadata_file = archive
        .by_name(METADATA_FILE)
        .map_err(|_| TemplateError::MissingFile(METADATA_FILE.to_string()))?;

    let mut contents = String::new();
    metadata_file.read_to_string(&mut contents)?;
    let _: TemplateMetadata = serde_json::from_str(&contents)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_package_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.wdt");

        // Create a package
        let doc = DocumentTree::with_empty_paragraph();
        let metadata = TemplateMetadata::new("test-id", "Test Template")
            .with_description("A test template")
            .with_author("Test Author");

        let package = TemplatePackage::new(doc.clone(), metadata);
        package.write_to_file(&path).unwrap();

        // Read it back
        let loaded = TemplatePackage::read_from_file(&path).unwrap();

        assert_eq!(loaded.metadata.id, "test-id");
        assert_eq!(loaded.metadata.name, "Test Template");
        assert_eq!(loaded.metadata.description, "A test template");
        assert_eq!(loaded.metadata.author, "Test Author");
    }

    #[test]
    fn test_package_with_thumbnail() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test-thumb.wdt");

        // Create a package with thumbnail
        let doc = DocumentTree::with_empty_paragraph();
        let metadata = TemplateMetadata::new("test-id", "Test Template").with_thumbnail();
        let thumbnail = vec![0x89, 0x50, 0x4E, 0x47]; // PNG magic bytes

        let package = TemplatePackage::new(doc, metadata).with_thumbnail(thumbnail.clone());
        package.write_to_file(&path).unwrap();

        // Read thumbnail
        let loaded_thumb = read_thumbnail(&path).unwrap();
        assert_eq!(loaded_thumb, Some(thumbnail));
    }

    #[test]
    fn test_package_with_resources() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test-resources.wdt");

        // Create a package with resources
        let doc = DocumentTree::with_empty_paragraph();
        let metadata = TemplateMetadata::new("test-id", "Test Template");
        let resource_data = b"test resource content".to_vec();

        let package = TemplatePackage::new(doc, metadata)
            .with_resource("test.txt", resource_data.clone());
        package.write_to_file(&path).unwrap();

        // Read it back
        let loaded = TemplatePackage::read_from_file(&path).unwrap();
        assert_eq!(loaded.resources.get("test.txt"), Some(&resource_data));
    }

    #[test]
    fn test_read_metadata_only() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test-meta.wdt");

        // Create a package
        let doc = DocumentTree::with_empty_paragraph();
        let metadata = TemplateMetadata::new("meta-test", "Metadata Test")
            .with_author("Test Author");

        let package = TemplatePackage::new(doc, metadata);
        package.write_to_file(&path).unwrap();

        // Read only metadata
        let loaded_meta = read_metadata(&path).unwrap();
        assert_eq!(loaded_meta.id, "meta-test");
        assert_eq!(loaded_meta.name, "Metadata Test");
    }

    #[test]
    fn test_validate_package() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test-validate.wdt");

        // Create a valid package
        let doc = DocumentTree::with_empty_paragraph();
        let metadata = TemplateMetadata::new("test-id", "Test Template");
        let package = TemplatePackage::new(doc, metadata);
        package.write_to_file(&path).unwrap();

        // Should validate successfully
        assert!(validate_package(&path).is_ok());
    }
}
