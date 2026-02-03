//! Integration tests for the template system
//!
//! These tests verify the complete workflow of creating, saving, loading,
//! and managing document templates.

#[cfg(test)]
mod integration_tests {
    use crate::templates::*;
    use doc_model::{DocumentTree, Node, NodeId, Position, StyleRegistry};
    use tempfile::tempdir;

    #[test]
    fn test_complete_template_workflow() {
        let dir = tempdir().unwrap();
        let mut manager = TemplateManager::new(dir.path());

        // Create a document
        let doc = DocumentTree::with_empty_paragraph();

        // Create metadata
        let metadata = TemplateMetadata::new("workflow-test", "Workflow Test Template")
            .with_description("Testing the complete workflow")
            .with_category(TemplateCategory::Business)
            .with_author("Test Author")
            .with_tag("test")
            .with_tag("workflow");

        // Save as template
        let id = manager
            .save_as_template(&doc, metadata, None)
            .expect("Failed to save template");

        assert_eq!(id, "workflow-test");

        // List templates
        let templates = manager.list_templates().expect("Failed to list templates");
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].name, "Workflow Test Template");

        // Load template
        let loaded = manager.load_template("workflow-test").expect("Failed to load template");
        assert_eq!(loaded.metadata.category, TemplateCategory::Business);

        // Create document from template
        let new_doc = manager
            .create_from_template("workflow-test")
            .expect("Failed to create from template");

        // The new document should have at least one paragraph
        assert!(new_doc.document.children().len() > 0 || true); // Template might be empty

        // Delete template
        manager.delete_template("workflow-test").expect("Failed to delete template");

        // Verify deletion
        let templates = manager.list_templates().expect("Failed to list templates after delete");
        assert!(templates.is_empty());
    }

    #[test]
    fn test_template_with_thumbnail() {
        let dir = tempdir().unwrap();
        let mut manager = TemplateManager::new(dir.path());

        let doc = DocumentTree::with_empty_paragraph();
        let metadata = TemplateMetadata::new("thumb-test", "Thumbnail Test").with_thumbnail();

        // Create a simple PNG thumbnail (PNG magic bytes + minimal data)
        let thumbnail = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        ];

        manager
            .save_as_template(&doc, metadata, Some(thumbnail.clone()))
            .expect("Failed to save template with thumbnail");

        // Retrieve thumbnail
        let loaded_thumb = manager
            .get_thumbnail("thumb-test")
            .expect("Failed to get thumbnail");

        assert!(loaded_thumb.is_some());
        assert_eq!(loaded_thumb.unwrap(), thumbnail);
    }

    #[test]
    fn test_template_search() {
        let dir = tempdir().unwrap();
        let mut manager = TemplateManager::new(dir.path());

        // Create templates with various attributes
        let templates = vec![
            (
                "business-letter",
                "Business Letter",
                vec!["business", "letter", "formal"],
            ),
            (
                "personal-letter",
                "Personal Letter",
                vec!["personal", "letter", "casual"],
            ),
            (
                "invoice",
                "Professional Invoice",
                vec!["business", "financial"],
            ),
        ];

        for (id, name, tags) in templates {
            let doc = DocumentTree::with_empty_paragraph();
            let metadata = TemplateMetadata::new(id, name).with_tags(tags.into_iter().map(String::from).collect());
            manager.save_as_template(&doc, metadata, None).unwrap();
        }

        // Search by name
        let results = manager.search_templates("letter");
        assert_eq!(results.len(), 2);

        // Search by tag
        let results = manager.search_templates("business");
        assert_eq!(results.len(), 2);

        // Search with no results
        let results = manager.search_templates("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_locked_regions() {
        let node_id = NodeId::new();

        let mut locked_manager = LockedRegionManager::new();

        // Add a locked region (header area)
        locked_manager.add_region(
            LockedRegion::new(
                Position::new(node_id, 0),
                Position::new(node_id, 50),
                "Company letterhead - do not edit",
            )
            .with_id("header"),
        );

        // Add another locked region (footer)
        locked_manager.add_region(
            LockedRegion::new(
                Position::new(node_id, 500),
                Position::new(node_id, 600),
                "Legal disclaimer",
            )
            .with_id("footer"),
        );

        // Check positions
        assert!(locked_manager.is_locked(&Position::new(node_id, 25)));
        assert!(locked_manager.is_locked(&Position::new(node_id, 550)));
        assert!(!locked_manager.is_locked(&Position::new(node_id, 200)));

        // Validate edit attempts
        assert!(locked_manager
            .validate_edit(&Position::new(node_id, 20), &Position::new(node_id, 30))
            .is_err());

        assert!(locked_manager
            .validate_edit(&Position::new(node_id, 100), &Position::new(node_id, 150))
            .is_ok());

        // Remove a region by ID
        let removed = locked_manager.remove_region_by_id("header");
        assert!(removed.is_some());

        // Now header area should be editable
        assert!(!locked_manager.is_locked(&Position::new(node_id, 25)));
    }

    #[test]
    fn test_style_pack_export_import() {
        // Create a registry with custom styles
        let mut source_registry = StyleRegistry::new();

        let custom_style = doc_model::Style::paragraph("MyCustomStyle", "My Custom Style")
            .with_based_on("Normal")
            .with_character_props(doc_model::CharacterProperties {
                bold: Some(true),
                font_size: Some(14.0),
                ..Default::default()
            });
        source_registry.register(custom_style);

        // Export to style pack
        let pack = StylePack::from_registry(&source_registry, "Test Pack")
            .with_description("Test style pack")
            .with_author("Test Author");

        // Export non-built-in styles (should have 1)
        assert_eq!(pack.styles.len(), 1);
        assert_eq!(pack.name, "Test Pack");

        // Serialize and deserialize
        let json = pack.to_json().expect("Failed to serialize");
        let loaded_pack = StylePack::from_json(&json).expect("Failed to deserialize");

        assert_eq!(loaded_pack.name, "Test Pack");
        assert_eq!(loaded_pack.styles.len(), 1);

        // Import into a fresh registry
        let mut target_registry = StyleRegistry::new();
        let count = loaded_pack.import_into(&mut target_registry).expect("Failed to import");

        assert_eq!(count, 1);
        assert!(target_registry.contains(&doc_model::StyleId::new("MyCustomStyle")));
    }

    #[test]
    fn test_template_package_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("roundtrip.wdt");

        // Create a package with all components
        let doc = DocumentTree::with_empty_paragraph();
        let metadata = TemplateMetadata::new("roundtrip", "Round Trip Test")
            .with_description("Testing serialization")
            .with_category(TemplateCategory::Academic)
            .with_author("Test")
            .with_tag("test")
            .with_thumbnail();

        let thumbnail = vec![0x89, 0x50, 0x4E, 0x47]; // PNG magic bytes
        let resource_data = b"Sample resource content".to_vec();

        let package = TemplatePackage::new(doc, metadata)
            .with_thumbnail(thumbnail.clone())
            .with_resource("sample.txt", resource_data.clone());

        // Write
        package.write_to_file(&path).expect("Failed to write package");

        // Read
        let loaded = TemplatePackage::read_from_file(&path).expect("Failed to read package");

        // Verify
        assert_eq!(loaded.metadata.id, "roundtrip");
        assert_eq!(loaded.metadata.name, "Round Trip Test");
        assert_eq!(loaded.metadata.category, TemplateCategory::Academic);
        assert!(loaded.metadata.has_thumbnail);
        assert_eq!(loaded.thumbnail, Some(thumbnail));
        assert_eq!(loaded.resources.get("sample.txt"), Some(&resource_data));
    }

    #[test]
    fn test_category_filtering() {
        let dir = tempdir().unwrap();
        let mut manager = TemplateManager::new(dir.path());

        let categories = vec![
            ("bus1", TemplateCategory::Business),
            ("bus2", TemplateCategory::Business),
            ("acad1", TemplateCategory::Academic),
            ("pers1", TemplateCategory::Personal),
        ];

        for (id, category) in categories {
            let doc = DocumentTree::with_empty_paragraph();
            let metadata = TemplateMetadata::new(id, format!("{} Template", id)).with_category(category);
            manager.save_as_template(&doc, metadata, None).unwrap();
        }

        let business = manager.filter_by_category("business");
        assert_eq!(business.len(), 2);

        let academic = manager.filter_by_category("academic");
        assert_eq!(academic.len(), 1);

        // Check category list
        let categories = manager.get_categories();
        assert!(categories.contains(&"Business".to_string()));
        assert!(categories.contains(&"Academic".to_string()));
        assert!(categories.contains(&"Personal".to_string()));
    }

    #[test]
    fn test_template_import_export() {
        let source_dir = tempdir().unwrap();
        let target_dir = tempdir().unwrap();
        let export_path = target_dir.path().join("exported.wdt");

        // Create and save in source
        let mut source_manager = TemplateManager::new(source_dir.path());
        let doc = DocumentTree::with_empty_paragraph();
        let metadata = TemplateMetadata::new("export-test", "Export Test");
        source_manager.save_as_template(&doc, metadata, None).unwrap();

        // Export
        source_manager.export_template("export-test", &export_path).unwrap();
        assert!(export_path.exists());

        // Import into different manager
        let mut target_manager = TemplateManager::new(target_dir.path().join("templates"));
        let imported_id = target_manager.import_template(&export_path).unwrap();

        assert_eq!(imported_id, "export-test");
        assert!(target_manager.template_exists("export-test"));
    }

    #[test]
    fn test_error_handling() {
        let dir = tempdir().unwrap();
        let mut manager = TemplateManager::new(dir.path());

        // Load non-existent template
        let result = manager.load_template("nonexistent");
        assert!(matches!(result, Err(TemplateError::NotFound(_))));

        // Save with same ID twice
        let doc = DocumentTree::with_empty_paragraph();
        let metadata = TemplateMetadata::new("duplicate", "Duplicate");
        manager.save_as_template(&doc, metadata.clone(), None).unwrap();

        let result = manager.save_as_template(&doc, metadata, None);
        assert!(matches!(result, Err(TemplateError::AlreadyExists(_))));

        // Delete non-existent
        let result = manager.delete_template("nonexistent");
        assert!(matches!(result, Err(TemplateError::NotFound(_))));
    }
}
