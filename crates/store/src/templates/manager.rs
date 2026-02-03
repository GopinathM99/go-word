//! Template manager for CRUD operations on templates

use super::{
    read_metadata, read_thumbnail, validate_package, LockedRegion, TemplateError, TemplateMetadata,
    TemplatePackage, TemplateResult, TemplateSummary, TEMPLATE_EXTENSION,
};
use doc_model::DocumentTree;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Manages templates in a directory
#[derive(Debug)]
pub struct TemplateManager {
    /// Directory where templates are stored
    templates_dir: PathBuf,
    /// Cache of template metadata
    metadata_cache: HashMap<String, TemplateMetadata>,
}

impl TemplateManager {
    /// Create a new template manager
    pub fn new(templates_dir: impl Into<PathBuf>) -> Self {
        Self {
            templates_dir: templates_dir.into(),
            metadata_cache: HashMap::new(),
        }
    }

    /// Ensure the templates directory exists
    pub fn ensure_directory(&self) -> TemplateResult<()> {
        if !self.templates_dir.exists() {
            fs::create_dir_all(&self.templates_dir)?;
        }
        Ok(())
    }

    /// Get the templates directory path
    pub fn templates_dir(&self) -> &Path {
        &self.templates_dir
    }

    /// Get the path for a template file
    fn template_path(&self, template_id: &str) -> PathBuf {
        self.templates_dir
            .join(format!("{}.{}", template_id, TEMPLATE_EXTENSION))
    }

    /// List all available templates
    pub fn list_templates(&mut self) -> TemplateResult<Vec<TemplateSummary>> {
        self.ensure_directory()?;
        self.refresh_cache()?;

        let summaries = self
            .metadata_cache
            .values()
            .map(TemplateSummary::from)
            .collect();

        Ok(summaries)
    }

    /// Refresh the metadata cache by scanning the templates directory
    pub fn refresh_cache(&mut self) -> TemplateResult<()> {
        self.ensure_directory()?;
        self.metadata_cache.clear();

        let entries = fs::read_dir(&self.templates_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == TEMPLATE_EXTENSION).unwrap_or(false) {
                if let Ok(metadata) = read_metadata(&path) {
                    self.metadata_cache.insert(metadata.id.clone(), metadata);
                }
            }
        }

        Ok(())
    }

    /// Get template metadata by ID
    pub fn get_metadata(&self, template_id: &str) -> TemplateResult<&TemplateMetadata> {
        self.metadata_cache
            .get(template_id)
            .ok_or_else(|| TemplateError::NotFound(template_id.to_string()))
    }

    /// Get template metadata by ID (refreshing cache first)
    pub fn get_metadata_fresh(&mut self, template_id: &str) -> TemplateResult<TemplateMetadata> {
        let path = self.template_path(template_id);
        if !path.exists() {
            return Err(TemplateError::NotFound(template_id.to_string()));
        }

        let metadata = read_metadata(&path)?;
        self.metadata_cache
            .insert(template_id.to_string(), metadata.clone());
        Ok(metadata)
    }

    /// Load a complete template package
    pub fn load_template(&self, template_id: &str) -> TemplateResult<TemplatePackage> {
        let path = self.template_path(template_id);
        if !path.exists() {
            return Err(TemplateError::NotFound(template_id.to_string()));
        }

        TemplatePackage::read_from_file(&path)
    }

    /// Get template thumbnail
    pub fn get_thumbnail(&self, template_id: &str) -> TemplateResult<Option<Vec<u8>>> {
        let path = self.template_path(template_id);
        if !path.exists() {
            return Err(TemplateError::NotFound(template_id.to_string()));
        }

        read_thumbnail(&path)
    }

    /// Create a new document from a template
    pub fn create_from_template(&self, template_id: &str) -> TemplateResult<DocumentTree> {
        let package = self.load_template(template_id)?;
        Ok(package.create_document())
    }

    /// Save a document as a template
    pub fn save_as_template(
        &mut self,
        document: &DocumentTree,
        metadata: TemplateMetadata,
        thumbnail: Option<Vec<u8>>,
    ) -> TemplateResult<String> {
        self.ensure_directory()?;

        let template_id = metadata.id.clone();
        let path = self.template_path(&template_id);

        // Check if template already exists
        if path.exists() {
            return Err(TemplateError::AlreadyExists(template_id));
        }

        // Create the package
        let mut package = TemplatePackage::new(document.clone(), metadata.clone());
        if let Some(thumb) = thumbnail {
            package = package.with_thumbnail(thumb);
        }

        // Write to file
        package.write_to_file(&path)?;

        // Update cache
        self.metadata_cache.insert(template_id.clone(), metadata);

        Ok(template_id)
    }

    /// Update an existing template
    pub fn update_template(
        &mut self,
        template_id: &str,
        document: Option<&DocumentTree>,
        metadata: Option<TemplateMetadata>,
        thumbnail: Option<Vec<u8>>,
    ) -> TemplateResult<()> {
        let path = self.template_path(template_id);
        if !path.exists() {
            return Err(TemplateError::NotFound(template_id.to_string()));
        }

        // Load existing package
        let mut package = TemplatePackage::read_from_file(&path)?;

        // Update document if provided
        if let Some(doc) = document {
            package.document = doc.clone();
        }

        // Update metadata if provided
        if let Some(meta) = metadata {
            package.metadata = meta;
        }

        // Update thumbnail if provided
        if let Some(thumb) = thumbnail {
            package.thumbnail = Some(thumb);
            package.metadata.has_thumbnail = true;
        }

        // Update modified timestamp
        package.metadata.touch();

        // Write back
        package.write_to_file(&path)?;

        // Update cache
        self.metadata_cache
            .insert(template_id.to_string(), package.metadata);

        Ok(())
    }

    /// Delete a template
    pub fn delete_template(&mut self, template_id: &str) -> TemplateResult<()> {
        let path = self.template_path(template_id);
        if !path.exists() {
            return Err(TemplateError::NotFound(template_id.to_string()));
        }

        fs::remove_file(&path)?;
        self.metadata_cache.remove(template_id);

        Ok(())
    }

    /// Check if a template exists
    pub fn template_exists(&self, template_id: &str) -> bool {
        self.template_path(template_id).exists()
    }

    /// Validate a template file
    pub fn validate_template(&self, template_id: &str) -> TemplateResult<()> {
        let path = self.template_path(template_id);
        if !path.exists() {
            return Err(TemplateError::NotFound(template_id.to_string()));
        }

        validate_package(&path)
    }

    /// Import a template from an external file
    pub fn import_template(&mut self, source_path: impl AsRef<Path>) -> TemplateResult<String> {
        self.ensure_directory()?;

        // Validate and read the source template
        let package = TemplatePackage::read_from_file(source_path)?;
        let template_id = package.metadata.id.clone();

        let target_path = self.template_path(&template_id);
        if target_path.exists() {
            return Err(TemplateError::AlreadyExists(template_id));
        }

        // Write to templates directory
        package.write_to_file(&target_path)?;

        // Update cache
        self.metadata_cache
            .insert(template_id.clone(), package.metadata);

        Ok(template_id)
    }

    /// Export a template to an external file
    pub fn export_template(
        &self,
        template_id: &str,
        target_path: impl AsRef<Path>,
    ) -> TemplateResult<()> {
        let package = self.load_template(template_id)?;
        package.write_to_file(target_path)
    }

    /// Search templates by name or tags
    pub fn search_templates(&self, query: &str) -> Vec<&TemplateMetadata> {
        let query_lower = query.to_lowercase();

        self.metadata_cache
            .values()
            .filter(|meta| {
                meta.name.to_lowercase().contains(&query_lower)
                    || meta.description.to_lowercase().contains(&query_lower)
                    || meta.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Filter templates by category
    pub fn filter_by_category(&self, category: &str) -> Vec<&TemplateMetadata> {
        let category_lower = category.to_lowercase();

        self.metadata_cache
            .values()
            .filter(|meta| meta.category.to_string().to_lowercase() == category_lower)
            .collect()
    }

    /// Get all unique categories
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<_> = self
            .metadata_cache
            .values()
            .map(|meta| meta.category.to_string())
            .collect();

        categories.sort();
        categories.dedup();
        categories
    }
}

/// Locked region manager for tracking and validating edits
#[derive(Debug, Clone, Default)]
pub struct LockedRegionManager {
    /// Locked regions for the current document
    regions: Vec<LockedRegion>,
}

impl LockedRegionManager {
    /// Create a new locked region manager
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    /// Create from template metadata
    pub fn from_metadata(metadata: &TemplateMetadata) -> Self {
        Self {
            regions: metadata.locked_regions.clone(),
        }
    }

    /// Add a locked region
    pub fn add_region(&mut self, region: LockedRegion) {
        self.regions.push(region);
    }

    /// Remove a locked region by index
    pub fn remove_region(&mut self, index: usize) -> Option<LockedRegion> {
        if index < self.regions.len() {
            Some(self.regions.remove(index))
        } else {
            None
        }
    }

    /// Remove a locked region by ID
    pub fn remove_region_by_id(&mut self, id: &str) -> Option<LockedRegion> {
        if let Some(pos) = self.regions.iter().position(|r| r.id.as_deref() == Some(id)) {
            Some(self.regions.remove(pos))
        } else {
            None
        }
    }

    /// Clear all locked regions
    pub fn clear(&mut self) {
        self.regions.clear();
    }

    /// Get all locked regions
    pub fn regions(&self) -> &[LockedRegion] {
        &self.regions
    }

    /// Set locked regions
    pub fn set_regions(&mut self, regions: Vec<LockedRegion>) {
        self.regions = regions;
    }

    /// Check if a position is in a locked region
    pub fn is_locked(&self, position: &doc_model::Position) -> bool {
        self.regions.iter().any(|r| r.contains(position))
    }

    /// Check if an edit would affect a locked region
    pub fn would_affect_locked(
        &self,
        start: &doc_model::Position,
        end: &doc_model::Position,
    ) -> Option<&LockedRegion> {
        self.regions.iter().find(|r| r.overlaps(start, end))
    }

    /// Validate that an edit doesn't affect locked regions
    pub fn validate_edit(
        &self,
        start: &doc_model::Position,
        end: &doc_model::Position,
    ) -> TemplateResult<()> {
        if let Some(region) = self.would_affect_locked(start, end) {
            Err(TemplateError::LockedRegion(region.reason.clone()))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::{NodeId, Position};
    use tempfile::tempdir;

    #[test]
    fn test_template_manager_creation() {
        let dir = tempdir().unwrap();
        let manager = TemplateManager::new(dir.path());
        assert_eq!(manager.templates_dir(), dir.path());
    }

    #[test]
    fn test_save_and_load_template() {
        let dir = tempdir().unwrap();
        let mut manager = TemplateManager::new(dir.path());

        // Create and save a template
        let doc = DocumentTree::with_empty_paragraph();
        let metadata = TemplateMetadata::new("test-template", "Test Template")
            .with_description("A test template")
            .with_author("Test Author");

        let id = manager
            .save_as_template(&doc, metadata, None)
            .unwrap();

        assert_eq!(id, "test-template");
        assert!(manager.template_exists("test-template"));

        // Load it back
        let loaded = manager.load_template("test-template").unwrap();
        assert_eq!(loaded.metadata.name, "Test Template");
    }

    #[test]
    fn test_list_templates() {
        let dir = tempdir().unwrap();
        let mut manager = TemplateManager::new(dir.path());

        // Save a few templates
        for i in 1..=3 {
            let doc = DocumentTree::with_empty_paragraph();
            let metadata = TemplateMetadata::new(format!("template-{}", i), format!("Template {}", i));
            manager.save_as_template(&doc, metadata, None).unwrap();
        }

        // List them
        let templates = manager.list_templates().unwrap();
        assert_eq!(templates.len(), 3);
    }

    #[test]
    fn test_delete_template() {
        let dir = tempdir().unwrap();
        let mut manager = TemplateManager::new(dir.path());

        // Create a template
        let doc = DocumentTree::with_empty_paragraph();
        let metadata = TemplateMetadata::new("to-delete", "Delete Me");
        manager.save_as_template(&doc, metadata, None).unwrap();

        assert!(manager.template_exists("to-delete"));

        // Delete it
        manager.delete_template("to-delete").unwrap();

        assert!(!manager.template_exists("to-delete"));
    }

    #[test]
    fn test_search_templates() {
        let dir = tempdir().unwrap();
        let mut manager = TemplateManager::new(dir.path());

        // Create templates with various names and tags
        let templates = vec![
            ("business-letter", "Business Letter", vec!["business", "formal"]),
            ("personal-letter", "Personal Letter", vec!["personal", "casual"]),
            ("invoice", "Invoice Template", vec!["business", "financial"]),
        ];

        for (id, name, tags) in templates {
            let doc = DocumentTree::with_empty_paragraph();
            let metadata = TemplateMetadata::new(id, name).with_tags(tags.iter().map(|s| s.to_string()).collect());
            manager.save_as_template(&doc, metadata, None).unwrap();
        }

        // Search by name
        let results = manager.search_templates("letter");
        assert_eq!(results.len(), 2);

        // Search by tag
        let results = manager.search_templates("business");
        assert_eq!(results.len(), 2);

        // Search by name that matches only one
        let results = manager.search_templates("invoice");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_locked_region_manager() {
        let node_id = NodeId::new();
        let mut manager = LockedRegionManager::new();

        manager.add_region(LockedRegion::new(
            Position::new(node_id, 0),
            Position::new(node_id, 10),
            "Header region",
        ));

        // Check if positions are locked
        assert!(manager.is_locked(&Position::new(node_id, 5)));
        assert!(!manager.is_locked(&Position::new(node_id, 15)));

        // Validate edits
        let result = manager.validate_edit(
            &Position::new(node_id, 3),
            &Position::new(node_id, 7),
        );
        assert!(result.is_err());

        let result = manager.validate_edit(
            &Position::new(node_id, 15),
            &Position::new(node_id, 20),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_duplicate_template_error() {
        let dir = tempdir().unwrap();
        let mut manager = TemplateManager::new(dir.path());

        let doc = DocumentTree::with_empty_paragraph();
        let metadata = TemplateMetadata::new("duplicate", "First Template");
        manager.save_as_template(&doc, metadata, None).unwrap();

        // Try to save with the same ID
        let metadata2 = TemplateMetadata::new("duplicate", "Second Template");
        let result = manager.save_as_template(&doc, metadata2, None);

        assert!(matches!(result, Err(TemplateError::AlreadyExists(_))));
    }

    #[test]
    fn test_not_found_error() {
        let dir = tempdir().unwrap();
        let manager = TemplateManager::new(dir.path());

        let result = manager.load_template("nonexistent");
        assert!(matches!(result, Err(TemplateError::NotFound(_))));
    }
}
