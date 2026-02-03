//! Tauri IPC commands for template operations

use crate::state::TemplateState;
use doc_model::{NodeId, Position};
use serde::{Deserialize, Serialize};
use store::{
    LockedRegion, StylePack, TemplateCategory, TemplateMetadata,
};
use tauri::State;

// =============================================================================
// DTOs for Template Operations
// =============================================================================

/// Template summary DTO for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSummaryDto {
    /// Template ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Category
    pub category: String,
    /// Author
    pub author: String,
    /// Tags
    pub tags: Vec<String>,
    /// Whether thumbnail is available
    pub has_thumbnail: bool,
    /// Preview text
    pub preview_text: Option<String>,
}

impl From<store::TemplateSummary> for TemplateSummaryDto {
    fn from(summary: store::TemplateSummary) -> Self {
        Self {
            id: summary.id,
            name: summary.name,
            description: summary.description,
            category: summary.category,
            author: summary.author,
            tags: summary.tags,
            has_thumbnail: summary.has_thumbnail,
            preview_text: summary.preview_text,
        }
    }
}

/// Template metadata DTO for detailed info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateMetadataDto {
    /// Template ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Category
    pub category: String,
    /// Author
    pub author: String,
    /// Creation timestamp
    pub created: String,
    /// Modified timestamp
    pub modified: Option<String>,
    /// Version
    pub version: String,
    /// Locked regions
    pub locked_regions: Vec<LockedRegionDto>,
    /// Tags
    pub tags: Vec<String>,
    /// Has thumbnail
    pub has_thumbnail: bool,
    /// Preview text
    pub preview_text: Option<String>,
}

impl From<&TemplateMetadata> for TemplateMetadataDto {
    fn from(meta: &TemplateMetadata) -> Self {
        Self {
            id: meta.id.clone(),
            name: meta.name.clone(),
            description: meta.description.clone(),
            category: meta.category.to_string(),
            author: meta.author.clone(),
            created: meta.created.clone(),
            modified: meta.modified.clone(),
            version: meta.version.clone(),
            locked_regions: meta.locked_regions.iter().map(LockedRegionDto::from).collect(),
            tags: meta.tags.clone(),
            has_thumbnail: meta.has_thumbnail,
            preview_text: meta.preview_text.clone(),
        }
    }
}

/// Input DTO for creating/updating template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateMetadataInput {
    /// Template ID (optional for creation - will be generated if not provided)
    pub id: Option<String>,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Category
    pub category: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Tags
    pub tags: Option<Vec<String>>,
}

impl From<TemplateMetadataInput> for TemplateMetadata {
    fn from(input: TemplateMetadataInput) -> Self {
        let id = input.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let mut meta = TemplateMetadata::new(id, input.name);

        if let Some(desc) = input.description {
            meta = meta.with_description(desc);
        }
        if let Some(cat) = input.category {
            meta = meta.with_category(TemplateCategory::from(cat.as_str()));
        }
        if let Some(author) = input.author {
            meta = meta.with_author(author);
        }
        if let Some(tags) = input.tags {
            meta = meta.with_tags(tags);
        }

        meta
    }
}

/// Locked region DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockedRegionDto {
    /// Start position
    pub start: PositionDto,
    /// End position
    pub end: PositionDto,
    /// Reason for locking
    pub reason: String,
    /// Optional region ID
    pub id: Option<String>,
}

impl From<&LockedRegion> for LockedRegionDto {
    fn from(region: &LockedRegion) -> Self {
        Self {
            start: PositionDto::from(&region.start),
            end: PositionDto::from(&region.end),
            reason: region.reason.clone(),
            id: region.id.clone(),
        }
    }
}

impl From<LockedRegionDto> for LockedRegion {
    fn from(dto: LockedRegionDto) -> Self {
        let mut region = LockedRegion::new(
            Position::from(dto.start),
            Position::from(dto.end),
            dto.reason,
        );
        if let Some(id) = dto.id {
            region = region.with_id(id);
        }
        region
    }
}

/// Position DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionDto {
    /// Node ID
    pub node_id: String,
    /// Character offset
    pub offset: usize,
}

impl From<&Position> for PositionDto {
    fn from(pos: &Position) -> Self {
        Self {
            node_id: pos.node_id.to_string(),
            offset: pos.offset,
        }
    }
}

impl From<PositionDto> for Position {
    fn from(dto: PositionDto) -> Self {
        // Parse the node ID from string
        let node_id = NodeId::from_string(&dto.node_id).unwrap_or_else(NodeId::new);
        Position::new(node_id, dto.offset)
    }
}

/// Style pack DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StylePackDto {
    /// Style pack name
    pub name: String,
    /// Description
    pub description: String,
    /// Author
    pub author: String,
    /// Version
    pub version: String,
    /// Created timestamp
    pub created: String,
    /// Number of styles
    pub style_count: usize,
    /// JSON representation of the style pack
    pub data: String,
}

impl From<&StylePack> for StylePackDto {
    fn from(pack: &StylePack) -> Self {
        Self {
            name: pack.name.clone(),
            description: pack.description.clone(),
            author: pack.author.clone(),
            version: pack.version.clone(),
            created: pack.created.clone(),
            style_count: pack.styles.len(),
            data: pack.to_json().unwrap_or_default(),
        }
    }
}

// =============================================================================
// Template Commands
// =============================================================================

/// List all available templates
#[tauri::command]
pub fn list_templates(
    state: State<'_, TemplateState>,
) -> Result<Vec<TemplateSummaryDto>, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    let templates = manager.list_templates().map_err(|e| e.to_string())?;
    Ok(templates.into_iter().map(TemplateSummaryDto::from).collect())
}

/// Get template metadata by ID
#[tauri::command]
pub fn get_template_metadata(
    template_id: String,
    state: State<'_, TemplateState>,
) -> Result<TemplateMetadataDto, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    let metadata = manager
        .get_metadata_fresh(&template_id)
        .map_err(|e| e.to_string())?;
    Ok(TemplateMetadataDto::from(&metadata))
}

/// Get template thumbnail as base64 encoded string
#[tauri::command]
pub fn get_template_thumbnail(
    template_id: String,
    state: State<'_, TemplateState>,
) -> Result<Option<String>, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let thumbnail = manager
        .get_thumbnail(&template_id)
        .map_err(|e| e.to_string())?;

    Ok(thumbnail.map(|data| {
        use base64::{engine::general_purpose::STANDARD, Engine};
        format!("data:image/png;base64,{}", STANDARD.encode(&data))
    }))
}

/// Create a new document from a template
#[tauri::command]
pub fn create_from_template(
    template_id: String,
    state: State<'_, TemplateState>,
) -> Result<String, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let document = manager
        .create_from_template(&template_id)
        .map_err(|e| e.to_string())?;

    // Return the document root ID
    Ok(document.root_id().to_string())
}

/// Save a document as a template
#[tauri::command]
pub fn save_as_template(
    _doc_id: String,
    metadata: TemplateMetadataInput,
    thumbnail_base64: Option<String>,
    state: State<'_, TemplateState>,
) -> Result<String, String> {
    // TODO: Get actual document from document state by doc_id
    // For now, create a placeholder document
    let document = doc_model::DocumentTree::with_empty_paragraph();

    let template_metadata: TemplateMetadata = metadata.into();

    // Decode thumbnail if provided
    let thumbnail = thumbnail_base64.and_then(|b64| {
        use base64::{engine::general_purpose::STANDARD, Engine};
        // Remove data URL prefix if present
        let data = b64.strip_prefix("data:image/png;base64,").unwrap_or(&b64);
        STANDARD.decode(data).ok()
    });

    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    let id = manager
        .save_as_template(&document, template_metadata, thumbnail)
        .map_err(|e| e.to_string())?;

    Ok(id)
}

/// Delete a template
#[tauri::command]
pub fn delete_template(
    template_id: String,
    state: State<'_, TemplateState>,
) -> Result<(), String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    manager.delete_template(&template_id).map_err(|e| e.to_string())
}

/// Search templates by query
#[tauri::command]
pub fn search_templates(
    query: String,
    state: State<'_, TemplateState>,
) -> Result<Vec<TemplateSummaryDto>, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let results = manager.search_templates(&query);
    Ok(results
        .into_iter()
        .map(|m| TemplateSummaryDto::from(store::TemplateSummary::from(m)))
        .collect())
}

/// Filter templates by category
#[tauri::command]
pub fn filter_templates_by_category(
    category: String,
    state: State<'_, TemplateState>,
) -> Result<Vec<TemplateSummaryDto>, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let results = manager.filter_by_category(&category);
    Ok(results
        .into_iter()
        .map(|m| TemplateSummaryDto::from(store::TemplateSummary::from(m)))
        .collect())
}

/// Get all template categories
#[tauri::command]
pub fn get_template_categories(
    state: State<'_, TemplateState>,
) -> Result<Vec<String>, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_categories())
}

/// Import a template from an external file
#[tauri::command]
pub fn import_template(
    source_path: String,
    state: State<'_, TemplateState>,
) -> Result<String, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    manager.import_template(&source_path).map_err(|e| e.to_string())
}

/// Export a template to an external file
#[tauri::command]
pub fn export_template(
    template_id: String,
    target_path: String,
    state: State<'_, TemplateState>,
) -> Result<(), String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    manager
        .export_template(&template_id, &target_path)
        .map_err(|e| e.to_string())
}

// =============================================================================
// Style Pack Commands
// =============================================================================

/// Export styles from a document as a style pack
#[tauri::command]
pub fn export_style_pack(
    _doc_id: String,
    name: String,
    description: Option<String>,
    author: Option<String>,
) -> Result<StylePackDto, String> {
    // TODO: Get actual document's style registry by doc_id
    // For now, export from a default registry with no custom styles
    let registry = doc_model::StyleRegistry::new();

    let mut pack = StylePack::from_registry(&registry, name);
    if let Some(desc) = description {
        pack = pack.with_description(desc);
    }
    if let Some(auth) = author {
        pack = pack.with_author(auth);
    }

    Ok(StylePackDto::from(&pack))
}

/// Import a style pack into a document
#[tauri::command]
pub fn import_style_pack(
    _doc_id: String,
    style_pack_json: String,
) -> Result<usize, String> {
    let pack = StylePack::from_json(&style_pack_json).map_err(|e| e.to_string())?;

    // TODO: Get actual document's style registry by doc_id
    // For now, import into a default registry (changes won't persist)
    let mut registry = doc_model::StyleRegistry::new();
    let count = pack.import_into(&mut registry).map_err(|e| e.to_string())?;

    Ok(count)
}

// =============================================================================
// Locked Region Commands
// =============================================================================

/// Get locked regions for a document
#[tauri::command]
pub fn get_locked_regions(
    _doc_id: String,
    state: State<'_, TemplateState>,
) -> Result<Vec<LockedRegionDto>, String> {
    let locked_manager = state.locked_regions.lock().map_err(|e| e.to_string())?;
    Ok(locked_manager
        .regions()
        .iter()
        .map(LockedRegionDto::from)
        .collect())
}

/// Set locked regions for a document
#[tauri::command]
pub fn set_locked_regions(
    _doc_id: String,
    regions: Vec<LockedRegionDto>,
    state: State<'_, TemplateState>,
) -> Result<(), String> {
    let mut locked_manager = state.locked_regions.lock().map_err(|e| e.to_string())?;
    let converted: Vec<LockedRegion> = regions.into_iter().map(LockedRegion::from).collect();
    locked_manager.set_regions(converted);
    Ok(())
}

/// Add a locked region
#[tauri::command]
pub fn add_locked_region(
    _doc_id: String,
    region: LockedRegionDto,
    state: State<'_, TemplateState>,
) -> Result<(), String> {
    let mut locked_manager = state.locked_regions.lock().map_err(|e| e.to_string())?;
    locked_manager.add_region(LockedRegion::from(region));
    Ok(())
}

/// Remove a locked region by ID
#[tauri::command]
pub fn remove_locked_region(
    _doc_id: String,
    region_id: String,
    state: State<'_, TemplateState>,
) -> Result<bool, String> {
    let mut locked_manager = state.locked_regions.lock().map_err(|e| e.to_string())?;
    Ok(locked_manager.remove_region_by_id(&region_id).is_some())
}

/// Clear all locked regions
#[tauri::command]
pub fn clear_locked_regions(
    _doc_id: String,
    state: State<'_, TemplateState>,
) -> Result<(), String> {
    let mut locked_manager = state.locked_regions.lock().map_err(|e| e.to_string())?;
    locked_manager.clear();
    Ok(())
}

/// Check if a position is in a locked region
#[tauri::command]
pub fn is_position_locked(
    _doc_id: String,
    position: PositionDto,
    state: State<'_, TemplateState>,
) -> Result<bool, String> {
    let locked_manager = state.locked_regions.lock().map_err(|e| e.to_string())?;
    Ok(locked_manager.is_locked(&Position::from(position)))
}

/// Validate that an edit doesn't affect locked regions
#[tauri::command]
pub fn validate_edit_for_locked_regions(
    _doc_id: String,
    start: PositionDto,
    end: PositionDto,
    state: State<'_, TemplateState>,
) -> Result<Option<String>, String> {
    let locked_manager = state.locked_regions.lock().map_err(|e| e.to_string())?;
    match locked_manager.validate_edit(&Position::from(start), &Position::from(end)) {
        Ok(()) => Ok(None),
        Err(e) => Ok(Some(e.to_string())),
    }
}
