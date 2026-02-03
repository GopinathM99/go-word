//! Tauri IPC commands for document operations

use crate::state::{AppState, FontManagerState, SettingsState};
use doc_model::{
    Alignment, CharacterProperties, LineSpacing, Node, ParagraphProperties, PropertySource, StyleId,
    StyleRegistry, StyleType,
};
use edit_engine::Command;
use serde::{Deserialize, Serialize};
use store::{AppSettings, EditingSettings, GeneralSettings, PrivacySettings, Theme};
use tauri::{Manager, State};
use text_engine::{FontStyle, FontWeight, SubstitutionReason};

/// Create a new empty document
#[tauri::command]
pub fn create_document() -> Result<String, String> {
    // TODO: Implement with doc_model
    Ok("doc_id".to_string())
}

/// Apply an editing command to the document
#[tauri::command]
pub fn apply_command(doc_id: String, command: String) -> Result<DocumentChange, String> {
    // TODO: Implement with edit_engine
    Ok(DocumentChange::default())
}

/// Get the layout/render model for the current viewport
#[tauri::command]
pub fn get_layout(doc_id: String, viewport: Viewport) -> Result<RenderModel, String> {
    // TODO: Implement with layout_engine and render_model
    Ok(RenderModel::default())
}

/// Save document to file
#[tauri::command]
pub fn save_document(doc_id: String, path: String) -> Result<(), String> {
    // TODO: Implement with store
    Ok(())
}

/// Load document from file
#[tauri::command]
pub fn load_document(path: String) -> Result<String, String> {
    // TODO: Implement with store
    Ok("doc_id".to_string())
}

/// Undo the last operation
#[tauri::command]
pub fn undo(doc_id: String) -> Result<DocumentChange, String> {
    // TODO: Implement with edit_engine
    Ok(DocumentChange::default())
}

/// Redo the last undone operation
#[tauri::command]
pub fn redo(doc_id: String) -> Result<DocumentChange, String> {
    // TODO: Implement with edit_engine
    Ok(DocumentChange::default())
}

// IPC Types

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentChange {
    pub changed_nodes: Vec<String>,
    pub dirty_pages: Vec<u32>,
    pub selection: Option<Selection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub anchor: Position,
    pub focus: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub node_id: String,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenderModel {
    pub pages: Vec<PageRender>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRender {
    pub page_index: u32,
    pub width: f64,
    pub height: f64,
    pub items: Vec<RenderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderItem {
    pub item_type: String,
    pub bounds: Rect,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

// =============================================================================
// Settings Commands
// =============================================================================

/// Settings data transfer object matching the frontend structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsDto {
    pub general: GeneralSettingsDto,
    pub editing: EditingSettingsDto,
    pub privacy: PrivacySettingsDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettingsDto {
    pub language: String,
    pub theme: String,
    pub recent_files_count: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditingSettingsDto {
    pub autosave_enabled: bool,
    pub autosave_interval_seconds: u32,
    pub default_font_family: String,
    pub default_font_size: f32,
    pub show_spelling_errors: bool,
    pub show_grammar_errors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettingsDto {
    pub telemetry_enabled: bool,
    pub crash_reports_enabled: bool,
}

impl From<&AppSettings> for SettingsDto {
    fn from(settings: &AppSettings) -> Self {
        Self {
            general: GeneralSettingsDto {
                language: settings.general.language.clone(),
                theme: match settings.general.theme {
                    Theme::Light => "light".to_string(),
                    Theme::Dark => "dark".to_string(),
                    Theme::System => "system".to_string(),
                },
                recent_files_count: settings.general.recent_files_count,
            },
            editing: EditingSettingsDto {
                autosave_enabled: settings.editing.autosave_enabled,
                autosave_interval_seconds: settings.editing.autosave_interval_seconds,
                default_font_family: settings.editing.default_font_family.clone(),
                default_font_size: settings.editing.default_font_size,
                show_spelling_errors: settings.editing.show_spelling_errors,
                show_grammar_errors: settings.editing.show_grammar_errors,
            },
            privacy: PrivacySettingsDto {
                telemetry_enabled: settings.privacy.telemetry_enabled,
                crash_reports_enabled: settings.privacy.crash_reports_enabled,
            },
        }
    }
}

impl From<SettingsDto> for AppSettings {
    fn from(dto: SettingsDto) -> Self {
        Self {
            general: GeneralSettings {
                language: dto.general.language,
                theme: match dto.general.theme.as_str() {
                    "light" => Theme::Light,
                    "dark" => Theme::Dark,
                    _ => Theme::System,
                },
                recent_files_count: dto.general.recent_files_count,
            },
            editing: EditingSettings {
                autosave_enabled: dto.editing.autosave_enabled,
                autosave_interval_seconds: dto.editing.autosave_interval_seconds,
                default_font_family: dto.editing.default_font_family,
                default_font_size: dto.editing.default_font_size,
                show_spelling_errors: dto.editing.show_spelling_errors,
                show_grammar_errors: dto.editing.show_grammar_errors,
            },
            privacy: PrivacySettings {
                telemetry_enabled: dto.privacy.telemetry_enabled,
                crash_reports_enabled: dto.privacy.crash_reports_enabled,
            },
        }
    }
}

/// Get current application settings
#[tauri::command]
pub fn get_settings(state: State<'_, SettingsState>) -> Result<SettingsDto, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    Ok(SettingsDto::from(manager.get()))
}

/// Update application settings
#[tauri::command]
pub fn update_settings(
    settings: SettingsDto,
    state: State<'_, SettingsState>,
) -> Result<(), String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    let app_settings: AppSettings = settings.into();
    manager.update_sync(app_settings).map_err(|e| e.to_string())
}

/// Reset settings to defaults
#[tauri::command]
pub fn reset_settings(state: State<'_, SettingsState>) -> Result<SettingsDto, String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    let settings = manager.reset_sync().map_err(|e| e.to_string())?;
    Ok(SettingsDto::from(settings))
}

// =============================================================================
// Style System Commands
// =============================================================================

/// Style DTO for frontend communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleDto {
    pub id: String,
    pub name: String,
    pub style_type: String, // "paragraph", "character", "table", "numbering"
    pub based_on: Option<String>,
    pub next_style: Option<String>,
    pub built_in: bool,
    pub hidden: bool,
    pub priority: u32,
    pub paragraph_props: ParagraphPropertiesDto,
    pub character_props: CharacterPropertiesDto,
}

/// Paragraph properties DTO
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphPropertiesDto {
    pub alignment: Option<String>, // "left", "center", "right", "justify"
    pub indent_left: Option<f32>,
    pub indent_right: Option<f32>,
    pub indent_first_line: Option<f32>,
    pub space_before: Option<f32>,
    pub space_after: Option<f32>,
    pub line_spacing: Option<LineSpacingDto>,
}

/// Line spacing DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum LineSpacingDto {
    Multiple { value: f32 },
    Exact { value: f32 },
    AtLeast { value: f32 },
}

/// Character properties DTO
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterPropertiesDto {
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strikethrough: Option<bool>,
    pub color: Option<String>,
    pub highlight: Option<String>,
}

/// Property source DTO for inspector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PropertySourceDto {
    DirectFormatting,
    Style { style_id: String },
    Default,
}

/// Computed property with source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputedPropertyDto<T> {
    pub value: T,
    pub source: PropertySourceDto,
}

/// Computed character properties with sources for inspector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputedCharacterPropertiesDto {
    pub font_family: ComputedPropertyDto<String>,
    pub font_size: ComputedPropertyDto<f32>,
    pub bold: ComputedPropertyDto<bool>,
    pub italic: ComputedPropertyDto<bool>,
    pub underline: ComputedPropertyDto<bool>,
    pub color: ComputedPropertyDto<String>,
}

/// Computed paragraph properties with sources for inspector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputedParagraphPropertiesDto {
    pub alignment: ComputedPropertyDto<String>,
    pub indent_left: ComputedPropertyDto<f32>,
    pub indent_right: ComputedPropertyDto<f32>,
    pub indent_first_line: ComputedPropertyDto<f32>,
    pub space_before: ComputedPropertyDto<f32>,
    pub space_after: ComputedPropertyDto<f32>,
    pub line_spacing: ComputedPropertyDto<LineSpacingDto>,
}

/// Style inspector data for the current selection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleInspectorData {
    pub paragraph_style_id: Option<String>,
    pub character_style_id: Option<String>,
    pub paragraph_props: ComputedParagraphPropertiesDto,
    pub character_props: ComputedCharacterPropertiesDto,
    pub has_direct_paragraph_formatting: bool,
    pub has_direct_character_formatting: bool,
}

// Conversion functions
impl From<PropertySource> for PropertySourceDto {
    fn from(source: PropertySource) -> Self {
        match source {
            PropertySource::DirectFormatting => PropertySourceDto::DirectFormatting,
            PropertySource::Style(id) => PropertySourceDto::Style {
                style_id: id.to_string(),
            },
            PropertySource::Default => PropertySourceDto::Default,
        }
    }
}

fn alignment_to_string(alignment: Alignment) -> String {
    match alignment {
        Alignment::Left => "left".to_string(),
        Alignment::Center => "center".to_string(),
        Alignment::Right => "right".to_string(),
        Alignment::Justify => "justify".to_string(),
    }
}

impl From<LineSpacing> for LineSpacingDto {
    fn from(spacing: LineSpacing) -> Self {
        match spacing {
            LineSpacing::Multiple(v) => LineSpacingDto::Multiple { value: v },
            LineSpacing::Exact(v) => LineSpacingDto::Exact { value: v },
            LineSpacing::AtLeast(v) => LineSpacingDto::AtLeast { value: v },
        }
    }
}

impl From<&doc_model::Style> for StyleDto {
    fn from(style: &doc_model::Style) -> Self {
        Self {
            id: style.id.to_string(),
            name: style.name.clone(),
            style_type: match style.style_type {
                StyleType::Paragraph => "paragraph".to_string(),
                StyleType::Character => "character".to_string(),
                StyleType::Table => "table".to_string(),
                StyleType::Numbering => "numbering".to_string(),
            },
            based_on: style.based_on.as_ref().map(|s| s.to_string()),
            next_style: style.next_style.as_ref().map(|s| s.to_string()),
            built_in: style.built_in,
            hidden: style.hidden,
            priority: style.priority,
            paragraph_props: ParagraphPropertiesDto::from(&style.paragraph_props),
            character_props: CharacterPropertiesDto::from(&style.character_props),
        }
    }
}

impl From<&ParagraphProperties> for ParagraphPropertiesDto {
    fn from(props: &ParagraphProperties) -> Self {
        Self {
            alignment: props.alignment.map(alignment_to_string),
            indent_left: props.indent_left,
            indent_right: props.indent_right,
            indent_first_line: props.indent_first_line,
            space_before: props.space_before,
            space_after: props.space_after,
            line_spacing: props.line_spacing.map(|ls| ls.into()),
        }
    }
}

impl From<&CharacterProperties> for CharacterPropertiesDto {
    fn from(props: &CharacterProperties) -> Self {
        Self {
            font_family: props.font_family.clone(),
            font_size: props.font_size,
            bold: props.bold,
            italic: props.italic,
            underline: props.underline,
            strikethrough: props.strikethrough,
            color: props.color.clone(),
            highlight: props.highlight.clone(),
        }
    }
}

fn parse_alignment(s: &str) -> Option<Alignment> {
    match s {
        "left" => Some(Alignment::Left),
        "center" => Some(Alignment::Center),
        "right" => Some(Alignment::Right),
        "justify" => Some(Alignment::Justify),
        _ => None,
    }
}

fn parse_line_spacing(dto: &LineSpacingDto) -> LineSpacing {
    match dto {
        LineSpacingDto::Multiple { value } => LineSpacing::Multiple(*value),
        LineSpacingDto::Exact { value } => LineSpacing::Exact(*value),
        LineSpacingDto::AtLeast { value } => LineSpacing::AtLeast(*value),
    }
}

impl From<&ParagraphPropertiesDto> for ParagraphProperties {
    fn from(dto: &ParagraphPropertiesDto) -> Self {
        Self {
            alignment: dto.alignment.as_ref().and_then(|s| parse_alignment(s)),
            indent_left: dto.indent_left,
            indent_right: dto.indent_right,
            indent_first_line: dto.indent_first_line,
            space_before: dto.space_before,
            space_after: dto.space_after,
            line_spacing: dto.line_spacing.as_ref().map(parse_line_spacing),
            ..Default::default()
        }
    }
}

impl From<&CharacterPropertiesDto> for CharacterProperties {
    fn from(dto: &CharacterPropertiesDto) -> Self {
        Self {
            font_family: dto.font_family.clone(),
            font_size: dto.font_size,
            bold: dto.bold,
            italic: dto.italic,
            underline: dto.underline,
            strikethrough: dto.strikethrough,
            color: dto.color.clone(),
            highlight: dto.highlight.clone(),
            ..Default::default()
        }
    }
}

/// Get all available styles for the gallery
#[tauri::command]
pub fn get_styles(_doc_id: String) -> Result<Vec<StyleDto>, String> {
    // For now, return the default built-in styles
    let registry = StyleRegistry::new();
    let styles: Vec<StyleDto> = registry
        .gallery_styles()
        .iter()
        .map(|s| StyleDto::from(*s))
        .collect();
    Ok(styles)
}

/// Get a specific style by ID
#[tauri::command]
pub fn get_style(_doc_id: String, style_id: String) -> Result<Option<StyleDto>, String> {
    let registry = StyleRegistry::new();
    let style = registry.get(&StyleId::new(&style_id));
    Ok(style.map(StyleDto::from))
}

/// Get resolved style properties (after inheritance chain resolution)
#[tauri::command]
pub fn get_resolved_style(
    _doc_id: String,
    style_id: String,
) -> Result<Option<ResolvedStyleDto>, String> {
    let registry = StyleRegistry::new();
    let resolved = registry.resolve(&StyleId::new(&style_id));
    Ok(resolved.map(|r| ResolvedStyleDto {
        style_id: r.style_id.to_string(),
        paragraph_props: ParagraphPropertiesDto::from(&r.paragraph_props),
        character_props: CharacterPropertiesDto::from(&r.character_props),
        inheritance_chain: r.inheritance_chain.iter().map(|id| id.to_string()).collect(),
    }))
}

/// Resolved style DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedStyleDto {
    pub style_id: String,
    pub paragraph_props: ParagraphPropertiesDto,
    pub character_props: CharacterPropertiesDto,
    pub inheritance_chain: Vec<String>,
}

/// Apply a paragraph style to the current selection
#[tauri::command]
pub fn apply_paragraph_style(
    _doc_id: String,
    _style_id: String,
) -> Result<DocumentChange, String> {
    // TODO: Implement with actual document state
    Ok(DocumentChange::default())
}

/// Apply a character style to the current selection
#[tauri::command]
pub fn apply_character_style(
    _doc_id: String,
    _style_id: String,
) -> Result<DocumentChange, String> {
    // TODO: Implement with actual document state
    Ok(DocumentChange::default())
}

/// Apply direct formatting to the current selection
#[tauri::command]
pub fn apply_direct_formatting(
    _doc_id: String,
    paragraph_props: Option<ParagraphPropertiesDto>,
    character_props: Option<CharacterPropertiesDto>,
) -> Result<DocumentChange, String> {
    // TODO: Implement with actual document state
    let _para_props = paragraph_props.as_ref().map(ParagraphProperties::from);
    let _char_props = character_props.as_ref().map(CharacterProperties::from);
    Ok(DocumentChange::default())
}

/// Clear direct formatting from the current selection
#[tauri::command]
pub fn clear_direct_formatting(
    _doc_id: String,
    _clear_paragraph: bool,
    _clear_character: bool,
) -> Result<DocumentChange, String> {
    // TODO: Implement with actual document state
    Ok(DocumentChange::default())
}

/// Get style inspector data for the current selection
#[tauri::command]
pub fn get_style_inspector(
    _doc_id: String,
) -> Result<StyleInspectorData, String> {
    // Return default inspector data for now
    // In a real implementation, this would look at the current selection
    Ok(StyleInspectorData {
        paragraph_style_id: Some("Normal".to_string()),
        character_style_id: None,
        paragraph_props: ComputedParagraphPropertiesDto {
            alignment: ComputedPropertyDto {
                value: "left".to_string(),
                source: PropertySourceDto::Style {
                    style_id: "Normal".to_string(),
                },
            },
            indent_left: ComputedPropertyDto {
                value: 0.0,
                source: PropertySourceDto::Default,
            },
            indent_right: ComputedPropertyDto {
                value: 0.0,
                source: PropertySourceDto::Default,
            },
            indent_first_line: ComputedPropertyDto {
                value: 0.0,
                source: PropertySourceDto::Default,
            },
            space_before: ComputedPropertyDto {
                value: 0.0,
                source: PropertySourceDto::Default,
            },
            space_after: ComputedPropertyDto {
                value: 8.0,
                source: PropertySourceDto::Style {
                    style_id: "Normal".to_string(),
                },
            },
            line_spacing: ComputedPropertyDto {
                value: LineSpacingDto::Multiple { value: 1.08 },
                source: PropertySourceDto::Style {
                    style_id: "Normal".to_string(),
                },
            },
        },
        character_props: ComputedCharacterPropertiesDto {
            font_family: ComputedPropertyDto {
                value: "Calibri".to_string(),
                source: PropertySourceDto::Style {
                    style_id: "Normal".to_string(),
                },
            },
            font_size: ComputedPropertyDto {
                value: 11.0,
                source: PropertySourceDto::Style {
                    style_id: "Normal".to_string(),
                },
            },
            bold: ComputedPropertyDto {
                value: false,
                source: PropertySourceDto::Default,
            },
            italic: ComputedPropertyDto {
                value: false,
                source: PropertySourceDto::Default,
            },
            underline: ComputedPropertyDto {
                value: false,
                source: PropertySourceDto::Default,
            },
            color: ComputedPropertyDto {
                value: "#000000".to_string(),
                source: PropertySourceDto::Style {
                    style_id: "Normal".to_string(),
                },
            },
        },
        has_direct_paragraph_formatting: false,
        has_direct_character_formatting: false,
    })
}

/// Create a new custom style
#[tauri::command]
pub fn create_style(
    _doc_id: String,
    name: String,
    style_type: String,
    based_on: Option<String>,
    paragraph_props: Option<ParagraphPropertiesDto>,
    character_props: Option<CharacterPropertiesDto>,
) -> Result<StyleDto, String> {
    // Validate style type
    let st = match style_type.as_str() {
        "paragraph" => StyleType::Paragraph,
        "character" => StyleType::Character,
        _ => return Err("Invalid style type".to_string()),
    };

    // Create the style
    let mut style = match st {
        StyleType::Paragraph => doc_model::Style::paragraph(name.clone(), name.clone()),
        StyleType::Character => doc_model::Style::character(name.clone(), name.clone()),
        _ => return Err("Unsupported style type".to_string()),
    };

    if let Some(base) = based_on {
        style = style.with_based_on(base);
    }

    if let Some(props) = paragraph_props {
        style = style.with_paragraph_props(ParagraphProperties::from(&props));
    }

    if let Some(props) = character_props {
        style = style.with_character_props(CharacterProperties::from(&props));
    }

    Ok(StyleDto::from(&style))
}

/// Modify an existing style
#[tauri::command]
pub fn modify_style(
    _doc_id: String,
    style_id: String,
    name: Option<String>,
    based_on: Option<String>,
    paragraph_props: Option<ParagraphPropertiesDto>,
    character_props: Option<CharacterPropertiesDto>,
) -> Result<StyleDto, String> {
    let registry = StyleRegistry::new();
    let style = registry
        .get(&StyleId::new(&style_id))
        .ok_or_else(|| format!("Style not found: {}", style_id))?;

    // Create modified copy
    let mut modified = style.clone();
    if let Some(n) = name {
        modified.name = n;
    }
    if let Some(base) = based_on {
        modified.based_on = Some(StyleId::new(base));
    }
    if let Some(props) = paragraph_props {
        modified.paragraph_props = ParagraphProperties::from(&props);
    }
    if let Some(props) = character_props {
        modified.character_props = CharacterProperties::from(&props);
    }

    Ok(StyleDto::from(&modified))
}

// =============================================================================
// Font Substitution Commands
// =============================================================================

/// Font substitution record DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FontSubstitutionRecordDto {
    pub requested_font: String,
    pub actual_font: String,
    pub requested_weight: String,
    pub requested_style: String,
    pub reason: String,
    pub occurrence_count: usize,
}

/// Font substitution summary DTO
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FontSubstitutionSummaryDto {
    pub substitutions: Vec<FontSubstitutionRecordDto>,
    pub total_substituted: usize,
    pub total_found: usize,
}

fn weight_to_string(weight: FontWeight) -> String {
    match weight {
        FontWeight::Normal => "Normal".to_string(),
        FontWeight::Bold => "Bold".to_string(),
    }
}

fn style_to_string(style: FontStyle) -> String {
    match style {
        FontStyle::Normal => "Normal".to_string(),
        FontStyle::Italic => "Italic".to_string(),
    }
}

fn reason_to_string(reason: SubstitutionReason) -> String {
    match reason {
        SubstitutionReason::NotInstalled => "NotInstalled".to_string(),
        SubstitutionReason::VariantNotAvailable => "VariantNotAvailable".to_string(),
        SubstitutionReason::ScriptNotSupported => "ScriptNotSupported".to_string(),
        SubstitutionReason::FallbackToDefault => "FallbackToDefault".to_string(),
    }
}

impl From<&text_engine::FontSubstitutionSummary> for FontSubstitutionSummaryDto {
    fn from(summary: &text_engine::FontSubstitutionSummary) -> Self {
        Self {
            substitutions: summary
                .substitutions
                .iter()
                .map(|r| FontSubstitutionRecordDto {
                    requested_font: r.requested_font.clone(),
                    actual_font: r.actual_font.clone(),
                    requested_weight: weight_to_string(r.requested_weight),
                    requested_style: style_to_string(r.requested_style),
                    reason: reason_to_string(r.reason),
                    occurrence_count: r.occurrence_count,
                })
                .collect(),
            total_substituted: summary.total_substituted,
            total_found: summary.total_found,
        }
    }
}

/// Get the current font substitution summary
#[tauri::command]
pub fn get_font_substitutions(
    state: State<'_, FontManagerState>,
) -> Result<FontSubstitutionSummaryDto, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let summary = manager.get_substitution_summary();
    Ok(FontSubstitutionSummaryDto::from(&summary))
}

/// Clear the font substitution summary
#[tauri::command]
pub fn clear_font_substitutions(
    state: State<'_, FontManagerState>,
) -> Result<(), String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    manager.clear_substitution_summary();
    Ok(())
}

/// Get list of available font families
#[tauri::command]
pub fn get_available_fonts(
    state: State<'_, FontManagerState>,
) -> Result<Vec<String>, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    manager.list_families().map_err(|e| format!("{:?}", e))
}

/// Check if a font is available on the system
#[tauri::command]
pub fn is_font_available(
    family: String,
    state: State<'_, FontManagerState>,
) -> Result<bool, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.is_font_available(&family))
}

/// Font resolution result DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FontResolutionDto {
    pub family: String,
    pub weight: String,
    pub style: String,
    pub was_substituted: bool,
    pub warning: Option<SubstitutionWarningDto>,
}

/// Substitution warning DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SubstitutionWarningDto {
    pub requested: String,
    pub substituted: String,
    pub reason: String,
}

/// Resolve a font, performing substitution if necessary
#[tauri::command]
pub fn resolve_font(
    family: String,
    weight: Option<String>,
    style: Option<String>,
    state: State<'_, FontManagerState>,
) -> Result<FontResolutionDto, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;

    let weight = match weight.as_deref() {
        Some("Bold") | Some("bold") => FontWeight::Bold,
        _ => FontWeight::Normal,
    };

    let style = match style.as_deref() {
        Some("Italic") | Some("italic") => FontStyle::Italic,
        _ => FontStyle::Normal,
    };

    let resolution = manager
        .resolve_font(&family, weight, style)
        .map_err(|e| format!("{:?}", e))?;

    Ok(FontResolutionDto {
        family: resolution.family.clone(),
        weight: weight_to_string(resolution.weight),
        style: style_to_string(resolution.style),
        was_substituted: resolution.was_substituted(),
        warning: resolution.warning.as_ref().map(|w| SubstitutionWarningDto {
            requested: w.requested.clone(),
            substituted: w.substituted.clone(),
            reason: reason_to_string(w.reason),
        }),
    })
}

// =============================================================================
// Bookmark Commands
// =============================================================================

/// Bookmark information DTO for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkDto {
    /// Bookmark ID
    pub id: String,
    /// Bookmark name
    pub name: String,
    /// Whether this is a point bookmark (vs range)
    pub is_point: bool,
    /// Preview text near the bookmark
    pub preview: Option<String>,
    /// Paragraph ID containing the bookmark start
    pub paragraph_id: String,
    /// Character offset in the paragraph
    pub offset: usize,
}

/// Insert a bookmark at the current selection
#[tauri::command]
pub fn insert_bookmark(
    _doc_id: String,
    name: String,
) -> Result<BookmarkDto, String> {
    // Validate the bookmark name
    doc_model::validate_bookmark_name(&name)
        .map_err(|e| format!("Invalid bookmark name: {}", e))?;

    // TODO: Implement with actual document state and EditingEngine
    // For now, return a placeholder
    Ok(BookmarkDto {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        is_point: true,
        preview: None,
        paragraph_id: "placeholder".to_string(),
        offset: 0,
    })
}

/// Delete a bookmark by name
#[tauri::command]
pub fn delete_bookmark(
    _doc_id: String,
    name: String,
) -> Result<(), String> {
    // TODO: Implement with actual document state
    if name.is_empty() {
        return Err("Bookmark name cannot be empty".to_string());
    }
    Ok(())
}

/// Rename a bookmark
#[tauri::command]
pub fn rename_bookmark(
    _doc_id: String,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    // Validate the new name
    doc_model::validate_bookmark_name(&new_name)
        .map_err(|e| format!("Invalid bookmark name: {}", e))?;

    if old_name.is_empty() {
        return Err("Current bookmark name cannot be empty".to_string());
    }

    // TODO: Implement with actual document state
    Ok(())
}

/// Navigate to a bookmark and return the new selection
#[tauri::command]
pub fn go_to_bookmark(
    _doc_id: String,
    name: String,
) -> Result<Selection, String> {
    if name.is_empty() {
        return Err("Bookmark name cannot be empty".to_string());
    }

    // TODO: Implement with actual document state
    // For now, return a placeholder selection
    Ok(Selection {
        anchor: Position {
            node_id: "placeholder".to_string(),
            offset: 0,
        },
        focus: Position {
            node_id: "placeholder".to_string(),
            offset: 0,
        },
    })
}

/// Get a list of all bookmarks in the document
#[tauri::command]
pub fn list_bookmarks(
    _doc_id: String,
) -> Result<Vec<BookmarkDto>, String> {
    // TODO: Implement with actual document state
    // For now, return an empty list
    Ok(Vec::new())
}

/// Check if a bookmark name is valid
#[tauri::command]
pub fn validate_bookmark_name(
    name: String,
) -> Result<bool, String> {
    match doc_model::validate_bookmark_name(&name) {
        Ok(()) => Ok(true),
        Err(e) => Err(format!("{}", e)),
    }
}

/// Check if a bookmark with the given name exists
#[tauri::command]
pub fn bookmark_exists(
    _doc_id: String,
    name: String,
) -> Result<bool, String> {
    // TODO: Implement with actual document state
    // For now, always return false
    Ok(false)
}

// =============================================================================
// Autosave and Recovery Commands
// =============================================================================

use store::{
    AutosaveConfig, AutosaveStatus, RecoveryConfig, RecoveryFile, RecoveryManager,
};

/// Recovery file DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryFileDto {
    /// Unique identifier for this recovery file
    pub id: String,
    /// Document ID from the original document
    pub document_id: String,
    /// Timestamp when the recovery file was created (Unix timestamp in ms)
    pub timestamp: u64,
    /// Original file path (if known)
    pub original_path: Option<String>,
    /// Human-readable description of when the file was created
    pub time_description: String,
    /// Size of the recovery file in bytes
    pub file_size: u64,
}

impl From<RecoveryFile> for RecoveryFileDto {
    fn from(file: RecoveryFile) -> Self {
        Self {
            id: file.id,
            document_id: file.document_id,
            timestamp: file.timestamp,
            original_path: file.original_path.map(|p| p.display().to_string()),
            time_description: file.time_description,
            file_size: file.file_size,
        }
    }
}

/// Autosave configuration DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutosaveConfigDto {
    /// Whether autosave is enabled
    pub enabled: bool,
    /// Interval between autosaves in seconds
    pub interval_secs: u64,
    /// Maximum number of autosave versions to keep
    pub max_versions: usize,
}

impl From<AutosaveConfigDto> for AutosaveConfig {
    fn from(dto: AutosaveConfigDto) -> Self {
        Self {
            enabled: dto.enabled,
            interval_secs: dto.interval_secs,
            max_versions: dto.max_versions,
            ..Default::default()
        }
    }
}

impl From<&AutosaveConfig> for AutosaveConfigDto {
    fn from(config: &AutosaveConfig) -> Self {
        Self {
            enabled: config.enabled,
            interval_secs: config.interval_secs,
            max_versions: config.max_versions,
        }
    }
}

/// Autosave status DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutosaveStatusDto {
    /// Whether autosave is enabled
    pub enabled: bool,
    /// Whether there are unsaved changes
    pub has_unsaved_changes: bool,
    /// Whether a save is currently in progress
    pub is_saving: bool,
    /// Timestamp of last successful save (Unix timestamp in ms)
    pub last_save_time: Option<u64>,
    /// Error message from last save attempt (if any)
    pub last_error: Option<String>,
    /// Time until next scheduled autosave (in seconds)
    pub next_save_in_secs: Option<u64>,
}

impl From<AutosaveStatus> for AutosaveStatusDto {
    fn from(status: AutosaveStatus) -> Self {
        Self {
            enabled: status.enabled,
            has_unsaved_changes: status.has_unsaved_changes,
            is_saving: status.is_saving,
            last_save_time: status.last_save_time,
            last_error: status.last_error,
            next_save_in_secs: status.next_save_in_secs,
        }
    }
}

/// Get list of available recovery files
#[tauri::command]
pub async fn get_recovery_files(
    app: tauri::AppHandle,
) -> Result<Vec<RecoveryFileDto>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let recovery_dir = app_data_dir.join("recovery");
    let config = RecoveryConfig::default().with_recovery_dir(recovery_dir);
    let manager = RecoveryManager::new(config);

    let files = manager
        .list_recovery_files()
        .await
        .map_err(|e| e.to_string())?;

    Ok(files.into_iter().map(RecoveryFileDto::from).collect())
}

/// Check if there are any recovery files (crash detection)
#[tauri::command]
pub async fn has_recovery_files(
    app: tauri::AppHandle,
) -> Result<bool, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let recovery_dir = app_data_dir.join("recovery");
    let config = RecoveryConfig::default().with_recovery_dir(recovery_dir);
    let manager = RecoveryManager::new(config);

    Ok(manager.has_recovery_files().await)
}

/// Recover a document from a recovery file
#[tauri::command]
pub async fn recover_document(
    app: tauri::AppHandle,
    recovery_id: String,
) -> Result<String, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let recovery_dir = app_data_dir.join("recovery");
    let config = RecoveryConfig::default().with_recovery_dir(recovery_dir);
    let manager = RecoveryManager::new(config);

    // Load the recovered document
    let tree = manager
        .recover_document(&recovery_id)
        .await
        .map_err(|e| e.to_string())?;

    // Return the document ID
    Ok(tree.root_id().to_string())
}

/// Discard a recovery file
#[tauri::command]
pub async fn discard_recovery(
    app: tauri::AppHandle,
    recovery_id: String,
) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let recovery_dir = app_data_dir.join("recovery");
    let config = RecoveryConfig::default().with_recovery_dir(recovery_dir);
    let manager = RecoveryManager::new(config);

    manager
        .discard_recovery(&recovery_id)
        .await
        .map_err(|e| e.to_string())
}

/// Discard all recovery files
#[tauri::command]
pub async fn discard_all_recovery(
    app: tauri::AppHandle,
) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let recovery_dir = app_data_dir.join("recovery");
    let config = RecoveryConfig::default().with_recovery_dir(recovery_dir);
    let manager = RecoveryManager::new(config);

    manager
        .discard_all_recovery()
        .await
        .map_err(|e| e.to_string())
}

/// Get the current autosave status
#[tauri::command]
pub fn get_autosave_status(
    state: State<'_, SettingsState>,
) -> Result<AutosaveStatusDto, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let settings = manager.get();

    // Return status based on settings
    // In a real implementation, this would use the actual autosave manager state
    Ok(AutosaveStatusDto {
        enabled: settings.editing.autosave_enabled,
        has_unsaved_changes: false,
        is_saving: false,
        last_save_time: None,
        last_error: None,
        next_save_in_secs: if settings.editing.autosave_enabled {
            Some(settings.editing.autosave_interval_seconds as u64)
        } else {
            None
        },
    })
}

/// Get the current autosave configuration
#[tauri::command]
pub fn get_autosave_config(
    state: State<'_, SettingsState>,
) -> Result<AutosaveConfigDto, String> {
    let manager = state.manager.lock().map_err(|e| e.to_string())?;
    let settings = manager.get();

    Ok(AutosaveConfigDto {
        enabled: settings.editing.autosave_enabled,
        interval_secs: settings.editing.autosave_interval_seconds as u64,
        max_versions: 5, // Default
    })
}

/// Update the autosave configuration
#[tauri::command]
pub fn set_autosave_config(
    config: AutosaveConfigDto,
    state: State<'_, SettingsState>,
) -> Result<(), String> {
    let mut manager = state.manager.lock().map_err(|e| e.to_string())?;
    let mut settings = manager.get().clone();

    settings.editing.autosave_enabled = config.enabled;
    settings.editing.autosave_interval_seconds = config.interval_secs as u32;

    manager.update_sync(settings).map_err(|e| e.to_string())
}

// =============================================================================
// PDF Export Commands
// =============================================================================

use store::pdf::{
    PdfExportOptions, PageRange, PdfVersionOption, PdfAConformance,
    ComplianceReport, ComplianceIssue, IssueCategory, IssueSeverity,
};

/// PDF export options DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfExportOptionsDto {
    /// Document title
    #[serde(default)]
    pub title: Option<String>,
    /// Document author
    #[serde(default)]
    pub author: Option<String>,
    /// Document subject
    #[serde(default)]
    pub subject: Option<String>,
    /// Document keywords
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Whether to compress content streams
    #[serde(default = "default_compress")]
    pub compress: bool,
    /// Whether to embed fonts
    #[serde(default)]
    pub embed_fonts: bool,
    /// PDF version ("v14", "v15", "v17")
    #[serde(default)]
    pub pdf_version: String,
    /// Page range to export (None = all pages)
    #[serde(default)]
    pub page_range: Option<PageRangeDto>,
    /// Image quality (0-100)
    #[serde(default = "default_image_quality")]
    pub image_quality: u8,
    /// PDF/A conformance level ("none", "1b", "2b")
    #[serde(default)]
    pub pdfa_conformance: String,
}

/// PDF/A compliance issue DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplianceIssueDto {
    /// Issue severity ("error", "warning", "info")
    pub severity: String,
    /// Issue category
    pub category: String,
    /// Human-readable description
    pub description: String,
    /// Suggestion for fixing the issue
    pub suggestion: Option<String>,
}

impl From<ComplianceIssue> for ComplianceIssueDto {
    fn from(issue: ComplianceIssue) -> Self {
        Self {
            severity: match issue.severity {
                IssueSeverity::Error => "error".to_string(),
                IssueSeverity::Warning => "warning".to_string(),
                IssueSeverity::Info => "info".to_string(),
            },
            category: match issue.category {
                IssueCategory::Font => "font".to_string(),
                IssueCategory::Metadata => "metadata".to_string(),
                IssueCategory::ColorSpace => "colorSpace".to_string(),
                IssueCategory::Transparency => "transparency".to_string(),
                IssueCategory::ImageCompression => "imageCompression".to_string(),
                IssueCategory::Security => "security".to_string(),
                IssueCategory::ExternalReference => "externalReference".to_string(),
                IssueCategory::Structure => "structure".to_string(),
            },
            description: issue.description,
            suggestion: issue.suggestion,
        }
    }
}

/// PDF/A compliance report DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplianceReportDto {
    /// Target conformance level
    pub conformance: String,
    /// Whether the document is compliant
    pub is_compliant: bool,
    /// List of issues found
    pub issues: Vec<ComplianceIssueDto>,
    /// Fonts that need to be embedded
    pub fonts_to_embed: Vec<String>,
    /// Whether transparency was detected
    pub has_transparency: bool,
    /// Color spaces used
    pub color_spaces: Vec<String>,
    /// Number of errors
    pub error_count: usize,
    /// Number of warnings
    pub warning_count: usize,
}

impl From<ComplianceReport> for ComplianceReportDto {
    fn from(report: ComplianceReport) -> Self {
        // Get counts before moving fields to avoid borrow-after-move
        let error_count = report.error_count();
        let warning_count = report.warning_count();
        Self {
            conformance: report.conformance.to_string(),
            is_compliant: report.is_compliant,
            issues: report.issues.into_iter().map(ComplianceIssueDto::from).collect(),
            fonts_to_embed: report.fonts_to_embed,
            has_transparency: report.has_transparency,
            color_spaces: report.color_spaces,
            error_count,
            warning_count,
        }
    }
}

fn default_compress() -> bool {
    true
}

fn default_image_quality() -> u8 {
    85
}

/// Page range DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageRangeDto {
    pub start: usize,
    pub end: usize,
}

impl From<PdfExportOptionsDto> for PdfExportOptions {
    fn from(dto: PdfExportOptionsDto) -> Self {
        let mut opts = PdfExportOptions::new();

        if let Some(title) = dto.title {
            opts = opts.with_title(title);
        }
        if let Some(author) = dto.author {
            opts = opts.with_author(author);
        }
        if let Some(subject) = dto.subject {
            opts = opts.with_subject(subject);
        }
        for keyword in dto.keywords {
            opts = opts.with_keyword(keyword);
        }

        opts = opts.with_compression(dto.compress);
        opts = opts.with_font_embedding(dto.embed_fonts);
        opts = opts.with_image_quality(dto.image_quality);

        // Parse PDF version
        let version = match dto.pdf_version.as_str() {
            "v15" | "1.5" => PdfVersionOption::V15,
            "v17" | "1.7" => PdfVersionOption::V17,
            _ => PdfVersionOption::V14,
        };
        opts = opts.with_version(version);

        // Page range
        if let Some(range) = dto.page_range {
            opts = opts.with_page_range(PageRange::new(range.start, range.end));
        }

        // PDF/A conformance
        let conformance = match dto.pdfa_conformance.to_lowercase().as_str() {
            "1b" | "a-1b" | "pdfa-1b" | "pdf/a-1b" => PdfAConformance::PdfA1b,
            "2b" | "a-2b" | "pdfa-2b" | "pdf/a-2b" => PdfAConformance::PdfA2b,
            _ => PdfAConformance::None,
        };
        opts = opts.with_pdfa_conformance(conformance);

        opts
    }
}

impl From<&PdfExportOptions> for PdfExportOptionsDto {
    fn from(opts: &PdfExportOptions) -> Self {
        Self {
            title: opts.title.clone(),
            author: opts.author.clone(),
            subject: opts.subject.clone(),
            keywords: opts.keywords.clone(),
            compress: opts.compress,
            embed_fonts: opts.embed_fonts,
            pdf_version: match opts.pdf_version {
                PdfVersionOption::V14 => "v14".to_string(),
                PdfVersionOption::V15 => "v15".to_string(),
                PdfVersionOption::V17 => "v17".to_string(),
            },
            page_range: opts.page_range.as_ref().map(|r| PageRangeDto {
                start: r.start,
                end: r.end,
            }),
            image_quality: opts.image_quality,
            pdfa_conformance: match opts.pdfa_conformance {
                PdfAConformance::None => "none".to_string(),
                PdfAConformance::PdfA1b => "1b".to_string(),
                PdfAConformance::PdfA2b => "2b".to_string(),
            },
        }
    }
}

/// Export the current document to PDF
#[tauri::command]
pub fn export_pdf(
    _doc_id: String,
    path: String,
    options: PdfExportOptionsDto,
) -> Result<(), String> {
    // TODO: Get actual render pages from the document
    // For now, create a placeholder page
    let pages = vec![render_model::PageRender {
        page_index: 0,
        width: 612.0,
        height: 792.0,
        items: vec![],
    }];

    let pdf_options: PdfExportOptions = options.into();

    store::pdf::export_pdf(&pages, &path, pdf_options)
        .map_err(|e| format!("PDF export failed: {}", e))
}

/// Export the current document to PDF bytes (for preview or in-memory use)
#[tauri::command]
pub fn export_pdf_bytes(
    _doc_id: String,
    options: PdfExportOptionsDto,
) -> Result<Vec<u8>, String> {
    // TODO: Get actual render pages from the document
    let pages = vec![render_model::PageRender {
        page_index: 0,
        width: 612.0,
        height: 792.0,
        items: vec![],
    }];

    let pdf_options: PdfExportOptions = options.into();

    store::pdf::export_pdf_bytes(&pages, pdf_options)
        .map_err(|e| format!("PDF export failed: {}", e))
}

/// Get default PDF export options
#[tauri::command]
pub fn get_pdf_export_options() -> PdfExportOptionsDto {
    PdfExportOptionsDto::from(&PdfExportOptions::default())
}

// =============================================================================
// PDF/A Export Commands
// =============================================================================

/// Export the current document to PDF/A format
///
/// # Arguments
///
/// * `doc_id` - The document ID
/// * `path` - The file path to save to
/// * `conformance_level` - The PDF/A conformance level ("1b" or "2b")
/// * `options` - Additional export options
#[tauri::command]
pub fn export_pdf_a(
    _doc_id: String,
    path: String,
    conformance_level: String,
    mut options: PdfExportOptionsDto,
) -> Result<(), String> {
    // Set the PDF/A conformance level
    options.pdfa_conformance = conformance_level;

    // TODO: Get actual render pages from the document
    let pages = vec![render_model::PageRender {
        page_index: 0,
        width: 612.0,
        height: 792.0,
        items: vec![],
    }];

    let pdf_options: PdfExportOptions = options.into();

    store::pdf::export_pdf(&pages, &path, pdf_options)
        .map_err(|e| format!("PDF/A export failed: {}", e))
}

/// Export the current document to PDF/A bytes
#[tauri::command]
pub fn export_pdf_a_bytes(
    _doc_id: String,
    conformance_level: String,
    mut options: PdfExportOptionsDto,
) -> Result<Vec<u8>, String> {
    // Set the PDF/A conformance level
    options.pdfa_conformance = conformance_level;

    // TODO: Get actual render pages from the document
    let pages = vec![render_model::PageRender {
        page_index: 0,
        width: 612.0,
        height: 792.0,
        items: vec![],
    }];

    let pdf_options: PdfExportOptions = options.into();

    store::pdf::export_pdf_bytes(&pages, pdf_options)
        .map_err(|e| format!("PDF/A export failed: {}", e))
}

/// Validate document for PDF/A compliance
///
/// Checks if the document can be exported as PDF/A compliant
/// and returns a detailed compliance report.
///
/// # Arguments
///
/// * `doc_id` - The document ID
/// * `conformance_level` - The target PDF/A conformance level ("1b" or "2b")
#[tauri::command]
pub fn validate_pdf_a_compliance(
    _doc_id: String,
    conformance_level: String,
) -> Result<ComplianceReportDto, String> {
    // Parse conformance level
    let conformance = match conformance_level.to_lowercase().as_str() {
        "1b" | "a-1b" | "pdfa-1b" | "pdf/a-1b" => PdfAConformance::PdfA1b,
        "2b" | "a-2b" | "pdfa-2b" | "pdf/a-2b" => PdfAConformance::PdfA2b,
        _ => return Err(format!("Invalid conformance level: {}. Use '1b' or '2b'", conformance_level)),
    };

    // TODO: Get actual render pages from the document
    let pages = vec![render_model::PageRender {
        page_index: 0,
        width: 612.0,
        height: 792.0,
        items: vec![],
    }];

    let report = store::pdf::validate_pdf_a_compliance(&pages, conformance);
    Ok(ComplianceReportDto::from(report))
}

/// Get supported PDF/A conformance levels
#[tauri::command]
pub fn get_pdfa_conformance_levels() -> Vec<PdfAConformanceLevelDto> {
    vec![
        PdfAConformanceLevelDto {
            id: "none".to_string(),
            name: "None (Standard PDF)".to_string(),
            description: "Standard PDF without PDF/A compliance".to_string(),
            pdf_version: "1.4+".to_string(),
            allows_transparency: true,
            requires_font_embedding: false,
        },
        PdfAConformanceLevelDto {
            id: "1b".to_string(),
            name: "PDF/A-1b".to_string(),
            description: "ISO 19005-1, Level B - Basic visual appearance for archival".to_string(),
            pdf_version: "1.4".to_string(),
            allows_transparency: false,
            requires_font_embedding: true,
        },
        PdfAConformanceLevelDto {
            id: "2b".to_string(),
            name: "PDF/A-2b".to_string(),
            description: "ISO 19005-2, Level B - Allows JPEG2000, transparency, and layers".to_string(),
            pdf_version: "1.7".to_string(),
            allows_transparency: true,
            requires_font_embedding: true,
        },
    ]
}

/// PDF/A conformance level information DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfAConformanceLevelDto {
    /// Level identifier ("none", "1b", "2b")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of the conformance level
    pub description: String,
    /// Required PDF version
    pub pdf_version: String,
    /// Whether transparency is allowed
    pub allows_transparency: bool,
    /// Whether font embedding is required
    pub requires_font_embedding: bool,
}

// =============================================================================
// DOCX Import/Export Commands
// =============================================================================

use store::docx;

/// File format DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileFormatDto {
    /// Format identifier
    pub id: String,
    /// File extension
    pub extension: String,
    /// MIME type
    pub mime_type: String,
    /// Human-readable name
    pub display_name: String,
    /// Whether import is supported
    pub supports_import: bool,
    /// Whether export is supported
    pub supports_export: bool,
}

impl From<docx::FileFormat> for FileFormatDto {
    fn from(format: docx::FileFormat) -> Self {
        Self {
            id: format.extension().to_string(),
            extension: format.extension().to_string(),
            mime_type: format.mime_type().to_string(),
            display_name: format.display_name().to_string(),
            supports_import: format.supports_import(),
            supports_export: format.supports_export(),
        }
    }
}

/// Document data DTO returned after opening a DOCX
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDataDto {
    /// Document ID
    pub id: String,
    /// Document title (from metadata)
    pub title: Option<String>,
    /// Number of paragraphs
    pub paragraph_count: usize,
    /// Approximate word count
    pub word_count: usize,
}

/// Open a DOCX file and return document data
#[tauri::command]
pub fn open_docx(path: String) -> Result<DocumentDataDto, String> {
    use std::path::Path;

    let path = Path::new(&path);
    let tree = store::import_docx(path)
        .map_err(|e| format!("Failed to open DOCX: {}", e))?;

    // Count paragraphs and estimate word count
    let paragraph_count = tree.paragraphs().count();
    let text_content = tree.text_content();
    let word_count = text_content.split_whitespace().count();

    // Get title from metadata
    let title = tree.document.metadata.title.clone();

    Ok(DocumentDataDto {
        id: tree.root_id().to_string(),
        title,
        paragraph_count,
        word_count,
    })
}

/// Save a document as DOCX
#[tauri::command]
pub fn save_as_docx(_doc_id: String, path: String) -> Result<(), String> {
    use std::path::Path;

    // TODO: Get actual document from document state
    // For now, create an empty document
    let tree = doc_model::DocumentTree::new();

    store::export_docx(&tree, Path::new(&path))
        .map_err(|e| format!("Failed to save DOCX: {}", e))
}

/// Get a list of all supported file formats
#[tauri::command]
pub fn get_supported_formats() -> Vec<FileFormatDto> {
    docx::get_supported_formats()
        .into_iter()
        .map(FileFormatDto::from)
        .collect()
}

/// Get formats that support import
#[tauri::command]
pub fn get_import_formats() -> Vec<FileFormatDto> {
    docx::get_import_formats()
        .into_iter()
        .map(FileFormatDto::from)
        .collect()
}

/// Get formats that support export
#[tauri::command]
pub fn get_export_formats() -> Vec<FileFormatDto> {
    docx::get_export_formats()
        .into_iter()
        .map(FileFormatDto::from)
        .collect()
}

// =============================================================================
// Enhanced DOCX Import/Export Commands (Phase 2)
// =============================================================================

/// Import options for DOCX files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportOptionsDto {
    /// Whether to import track changes
    pub import_track_changes: bool,
    /// Whether to import comments
    pub import_comments: bool,
    /// Whether to import fields
    pub import_fields: bool,
    /// Whether to import shapes/drawings
    pub import_drawings: bool,
    /// Whether to preserve unknown elements
    pub preserve_unknown: bool,
}

impl Default for ImportOptionsDto {
    fn default() -> Self {
        Self {
            import_track_changes: true,
            import_comments: true,
            import_fields: true,
            import_drawings: true,
            preserve_unknown: true,
        }
    }
}

/// Export options for DOCX files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOptionsDto {
    /// Whether to export track changes
    pub export_track_changes: bool,
    /// Whether to export comments
    pub export_comments: bool,
    /// Whether to export fields (or just their values)
    pub export_fields: bool,
    /// Whether to compress images
    pub compress_images: bool,
    /// Whether to include document properties
    pub include_properties: bool,
}

impl Default for ExportOptionsDto {
    fn default() -> Self {
        Self {
            export_track_changes: true,
            export_comments: true,
            export_fields: true,
            compress_images: true,
            include_properties: true,
        }
    }
}

/// A single DOCX fidelity warning
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocxFidelityWarningDto {
    /// Warning code
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Severity: "info", "minor", "moderate", "major", "critical"
    pub severity: String,
    /// Feature category
    pub category: String,
    /// Suggested action
    pub suggestion: Option<String>,
    /// Number of occurrences
    pub count: usize,
}

/// Fidelity report for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FidelityReportDto {
    /// Overall fidelity score (0-100)
    pub score: f32,
    /// Whether the document meets the 95% fidelity target
    pub meets_target: bool,
    /// All warnings
    pub warnings: Vec<DocxFidelityWarningDto>,
    /// Recommendations for improving fidelity
    pub recommendations: Vec<String>,
}

/// Open a DOCX file with import options
#[tauri::command]
pub fn import_docx_with_options(
    path: String,
    options: Option<ImportOptionsDto>,
) -> Result<DocumentDataDto, String> {
    use std::path::Path;

    let _opts = options.unwrap_or_default();

    // TODO: Apply import options to the parser
    let path = Path::new(&path);
    let tree = store::import_docx(path)
        .map_err(|e| format!("Failed to open DOCX: {}", e))?;

    let paragraph_count = tree.paragraphs().count();
    let text_content = tree.text_content();
    let word_count = text_content.split_whitespace().count();
    let title = tree.document.metadata.title.clone();

    Ok(DocumentDataDto {
        id: tree.root_id().to_string(),
        title,
        paragraph_count,
        word_count,
    })
}

/// Export a document to DOCX with options
#[tauri::command]
pub fn export_docx_with_options(
    doc_id: String,
    path: String,
    options: Option<ExportOptionsDto>,
) -> Result<(), String> {
    use std::path::Path;

    let _opts = options.unwrap_or_default();

    // TODO: Get actual document from state and apply export options
    let _ = doc_id;
    let tree = doc_model::DocumentTree::new();

    store::export_docx(&tree, Path::new(&path))
        .map_err(|e| format!("Failed to save DOCX: {}", e))
}

/// Get import warnings for a document
#[tauri::command]
pub fn get_docx_import_warnings(_doc_id: String) -> Result<Vec<DocxFidelityWarningDto>, String> {
    // TODO: Retrieve warnings from document state
    // For now, return empty list
    Ok(Vec::new())
}

/// Validate DOCX fidelity for a document
#[tauri::command]
pub fn validate_docx_fidelity(_doc_id: String) -> Result<FidelityReportDto, String> {
    // TODO: Actually validate the document
    // For now, return a perfect report
    Ok(FidelityReportDto {
        score: 100.0,
        meets_target: true,
        warnings: Vec::new(),
        recommendations: Vec::new(),
    })
}

// =============================================================================
// Print Commands
// =============================================================================

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use tauri::AppHandle;

/// Print capabilities information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintCapabilities {
    /// Available printers on the system
    pub printers: Vec<PrinterInfo>,
    /// Name of the default printer (if any)
    pub default_printer: Option<String>,
}

/// Information about a single printer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterInfo {
    /// Printer name
    pub name: String,
    /// Whether this is the default printer
    pub is_default: bool,
    /// Whether the printer supports color printing
    pub supports_color: bool,
    /// Whether the printer supports duplex (double-sided) printing
    pub supports_duplex: bool,
}

/// Print options for sending a document to the printer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintOptions {
    /// Specific printer to use (None = default printer)
    pub printer: Option<String>,
    /// Page range to print
    pub page_range: PrintPageRange,
    /// Number of copies
    pub copies: u32,
    /// Whether to collate when printing multiple copies
    pub collate: bool,
}

/// Page range specification for printing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PrintPageRange {
    /// Print all pages
    All,
    /// Print the current page
    Current { page: usize },
    /// Print a range of pages (inclusive)
    Range { from: usize, to: usize },
    /// Print specific pages
    Pages { pages: Vec<usize> },
}

/// Query available printers and their capabilities
///
/// Returns information about printers available on the system.
/// Note: On macOS/Linux, this uses lpstat to enumerate printers.
/// On Windows, returns a placeholder for the default printer.
#[tauri::command]
pub async fn get_print_capabilities() -> Result<PrintCapabilities, String> {
    // On macOS/Linux, use lpstat to enumerate printers
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let output = std::process::Command::new("lpstat")
            .args(["-p", "-d"])
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut printers = Vec::new();
                let mut default_printer = None;

                for line in stdout.lines() {
                    if line.starts_with("printer ") {
                        // Format: "printer <name> is idle. ..."
                        if let Some(name) = line.strip_prefix("printer ") {
                            if let Some(name) = name.split_whitespace().next() {
                                printers.push(PrinterInfo {
                                    name: name.to_string(),
                                    is_default: false,
                                    supports_color: true, // Assume color support
                                    supports_duplex: true, // Assume duplex support
                                });
                            }
                        }
                    } else if line.starts_with("system default destination: ") {
                        default_printer = line
                            .strip_prefix("system default destination: ")
                            .map(|s| s.trim().to_string());
                    }
                }

                // Mark the default printer
                if let Some(ref default_name) = default_printer {
                    for printer in &mut printers {
                        if &printer.name == default_name {
                            printer.is_default = true;
                            break;
                        }
                    }
                }

                // If no printers found but we have a default, add it
                if printers.is_empty() && default_printer.is_some() {
                    printers.push(PrinterInfo {
                        name: default_printer.clone().unwrap(),
                        is_default: true,
                        supports_color: true,
                        supports_duplex: true,
                    });
                }

                Ok(PrintCapabilities {
                    printers,
                    default_printer,
                })
            }
            Err(_) => {
                // If lpstat fails, return empty but valid capabilities
                Ok(PrintCapabilities {
                    printers: vec![],
                    default_printer: None,
                })
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, we would use the Windows API for printer enumeration
        // For now, return a placeholder
        Ok(PrintCapabilities {
            printers: vec![PrinterInfo {
                name: "Default Printer".to_string(),
                is_default: true,
                supports_color: true,
                supports_duplex: true,
            }],
            default_printer: Some("Default Printer".to_string()),
        })
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Ok(PrintCapabilities {
            printers: vec![],
            default_printer: None,
        })
    }
}

/// Send a document to the printer
///
/// This generates a PDF internally and opens the system print dialog,
/// or sends directly to the specified printer.
#[tauri::command]
pub async fn print_document(
    doc_id: String,
    options: PrintOptions,
    app: AppHandle,
) -> Result<(), String> {
    // Get app cache directory for temporary PDF
    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to get cache directory: {}", e))?;

    // Ensure cache directory exists
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;

    // Generate a temporary PDF file
    let temp_pdf_path = cache_dir.join(format!("print_{}.pdf", doc_id));

    // TODO: Get actual render pages from the document state
    // For now, create a placeholder page
    let pages = vec![render_model::PageRender {
        page_index: 0,
        width: 612.0,
        height: 792.0,
        items: vec![],
    }];

    // Filter pages based on print options
    let pages_to_print: Vec<_> = match &options.page_range {
        PrintPageRange::All => pages,
        PrintPageRange::Current { page } => pages
            .into_iter()
            .filter(|p| p.page_index == *page as u32)
            .collect(),
        PrintPageRange::Range { from, to } => pages
            .into_iter()
            .filter(|p| {
                let idx = p.page_index as usize;
                idx >= *from && idx <= *to
            })
            .collect(),
        PrintPageRange::Pages { pages: page_nums } => pages
            .into_iter()
            .filter(|p| page_nums.contains(&(p.page_index as usize)))
            .collect(),
    };

    if pages_to_print.is_empty() {
        return Err("No pages to print".to_string());
    }

    // Export to PDF
    let pdf_options = store::pdf::PdfExportOptions::new();
    store::pdf::export_pdf(&pages_to_print, &temp_pdf_path, pdf_options)
        .map_err(|e| format!("Failed to generate PDF for printing: {}", e))?;

    // Open the PDF with the system's default handler or print dialog
    #[cfg(target_os = "macos")]
    {
        // On macOS, use 'lpr' for direct printing or 'open' to open in Preview
        if let Some(printer_name) = &options.printer {
            // Direct print to specified printer
            let mut cmd = std::process::Command::new("lpr");
            cmd.arg("-P").arg(printer_name);

            if options.copies > 1 {
                cmd.arg("-#").arg(options.copies.to_string());
            }

            cmd.arg(&temp_pdf_path);

            let status = cmd
                .status()
                .map_err(|e| format!("Failed to execute lpr: {}", e))?;

            if !status.success() {
                return Err("Print command failed".to_string());
            }
        } else {
            // Open in system viewer which allows user to use print dialog
            std::process::Command::new("open")
                .arg(&temp_pdf_path)
                .spawn()
                .map_err(|e| format!("Failed to open PDF: {}", e))?;
        }
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, use the shell to open the print dialog
        if let Some(printer_name) = &options.printer {
            // Direct print using PowerShell
            let script = format!(
                "Start-Process -FilePath '{}' -Verb Print -ArgumentList '/d:{}'",
                temp_pdf_path.display(),
                printer_name
            );
            std::process::Command::new("powershell")
                .args(["-Command", &script])
                .spawn()
                .map_err(|e| format!("Failed to print: {}", e))?;
        } else {
            // Open with default handler
            std::process::Command::new("cmd")
                .args(["/C", "start", "", temp_pdf_path.to_str().unwrap_or("")])
                .spawn()
                .map_err(|e| format!("Failed to open PDF: {}", e))?;
        }
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, use lpr for printing or xdg-open for viewing
        if let Some(printer_name) = &options.printer {
            let mut cmd = std::process::Command::new("lpr");
            cmd.arg("-P").arg(printer_name);

            if options.copies > 1 {
                cmd.arg("-#").arg(options.copies.to_string());
            }

            cmd.arg(&temp_pdf_path);

            let status = cmd
                .status()
                .map_err(|e| format!("Failed to execute lpr: {}", e))?;

            if !status.success() {
                return Err("Print command failed".to_string());
            }
        } else {
            // Open with default handler
            std::process::Command::new("xdg-open")
                .arg(&temp_pdf_path)
                .spawn()
                .map_err(|e| format!("Failed to open PDF: {}", e))?;
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        return Err("Printing not supported on this platform".to_string());
    }

    Ok(())
}

/// Render a single page for print preview
///
/// Returns a base64-encoded PNG image of the page at the specified scale.
#[tauri::command]
pub async fn render_preview_page(
    _doc_id: String,
    page_number: usize,
    scale: f64,
) -> Result<String, String> {
    // TODO: Get actual render page from document state
    // For now, create a placeholder page
    let page = render_model::PageRender {
        page_index: page_number as u32,
        width: 612.0,  // Letter size width in points
        height: 792.0, // Letter size height in points
        items: vec![],
    };

    // Calculate dimensions based on scale
    let width = (page.width * scale) as u32;
    let height = (page.height * scale) as u32;

    // Create a placeholder image
    // In a real implementation, this would render the actual page content
    let png_data = create_placeholder_png(width, height, page_number)?;

    // Encode as base64
    let base64_encoded = BASE64_STANDARD.encode(&png_data);

    Ok(format!("data:image/png;base64,{}", base64_encoded))
}

/// Render multiple thumbnail images for print preview
///
/// Returns an array of base64-encoded PNG thumbnails.
#[tauri::command]
pub async fn render_preview_thumbnails(
    doc_id: String,
    start_page: usize,
    count: usize,
) -> Result<Vec<String>, String> {
    let mut thumbnails = Vec::with_capacity(count);

    // Thumbnail scale (smaller than full preview)
    let thumbnail_scale = 0.2;

    for i in 0..count {
        let page_number = start_page + i;

        // TODO: Check if page exists in document
        // For now, generate placeholder thumbnails

        let thumbnail = render_preview_page(doc_id.clone(), page_number, thumbnail_scale).await?;
        thumbnails.push(thumbnail);
    }

    Ok(thumbnails)
}

/// Create a simple placeholder PNG image
///
/// This creates a minimal valid PNG with a white background and a border.
/// In production, this would be replaced with actual page rendering.
fn create_placeholder_png(width: u32, height: u32, _page_number: usize) -> Result<Vec<u8>, String> {
    let mut png_data = Vec::new();

    // PNG signature
    png_data.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

    // IHDR chunk
    let mut ihdr_data = Vec::new();
    ihdr_data.extend_from_slice(&width.to_be_bytes());
    ihdr_data.extend_from_slice(&height.to_be_bytes());
    ihdr_data.push(8);  // Bit depth
    ihdr_data.push(2);  // Color type (RGB)
    ihdr_data.push(0);  // Compression method
    ihdr_data.push(0);  // Filter method
    ihdr_data.push(0);  // Interlace method

    write_png_chunk(&mut png_data, b"IHDR", &ihdr_data)?;

    // IDAT chunk (image data)
    // Create a simple white image with a border
    let mut raw_data = Vec::new();
    for y in 0..height {
        raw_data.push(0); // Filter type: None
        for x in 0..width {
            // Create a light gray background with a darker border
            let is_border = x < 2 || x >= width - 2 || y < 2 || y >= height - 2;
            if is_border {
                raw_data.extend_from_slice(&[200, 200, 200]); // Gray border
            } else {
                raw_data.extend_from_slice(&[255, 255, 255]); // White background
            }
        }
    }

    // Compress the image data using deflate
    let compressed = compress_deflate(&raw_data)?;
    write_png_chunk(&mut png_data, b"IDAT", &compressed)?;

    // IEND chunk
    write_png_chunk(&mut png_data, b"IEND", &[])?;

    Ok(png_data)
}

/// Write a PNG chunk with CRC
fn write_png_chunk(output: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) -> Result<(), String> {
    // Length (4 bytes, big-endian)
    output.extend_from_slice(&(data.len() as u32).to_be_bytes());

    // Chunk type
    output.extend_from_slice(chunk_type);

    // Chunk data
    output.extend_from_slice(data);

    // CRC32 of chunk type + data
    let crc = calculate_crc32(chunk_type, data);
    output.extend_from_slice(&crc.to_be_bytes());

    Ok(())
}

/// Calculate CRC32 for PNG chunk
fn calculate_crc32(chunk_type: &[u8], data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;

    // CRC table generation
    let mut crc_table = [0u32; 256];
    for n in 0..256 {
        let mut c = n as u32;
        for _ in 0..8 {
            if c & 1 != 0 {
                c = 0xEDB88320 ^ (c >> 1);
            } else {
                c >>= 1;
            }
        }
        crc_table[n] = c;
    }

    // Update CRC with chunk type
    for &byte in chunk_type {
        crc = crc_table[((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }

    // Update CRC with data
    for &byte in data {
        crc = crc_table[((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }

    crc ^ 0xFFFFFFFF
}

/// Simple deflate compression for PNG
fn compress_deflate(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();

    // zlib header (CMF + FLG)
    output.push(0x78); // CMF: compression method 8 (deflate), window size 32K
    output.push(0x01); // FLG: no dict, compression level 0

    // Split data into blocks of max 65535 bytes
    let chunks: Vec<&[u8]> = data.chunks(65535).collect();
    let num_chunks = chunks.len();

    for (i, chunk) in chunks.iter().enumerate() {
        let is_final = i == num_chunks - 1;

        // Block header: BFINAL (1 bit) + BTYPE (2 bits) = 00 for stored
        output.push(if is_final { 0x01 } else { 0x00 });

        // LEN (2 bytes, little-endian)
        let len = chunk.len() as u16;
        output.push((len & 0xFF) as u8);
        output.push((len >> 8) as u8);

        // NLEN (one's complement of LEN)
        let nlen = !len;
        output.push((nlen & 0xFF) as u8);
        output.push((nlen >> 8) as u8);

        // Data
        output.extend_from_slice(chunk);
    }

    // Adler-32 checksum
    let adler = calculate_adler32(data);
    output.extend_from_slice(&adler.to_be_bytes());

    Ok(output)
}

/// Calculate Adler-32 checksum
fn calculate_adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;

    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }

    (b << 16) | a
}

// =============================================================================
// Performance Telemetry Commands
// =============================================================================

use crate::state::PerfMetricsState;
use perf::{BudgetReport, BudgetViolation, PerfBudget, PerfSummary, TimingStats, ViolationSeverity};

/// Performance summary DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerfSummaryDto {
    /// Statistics for each command type
    pub command_stats: std::collections::HashMap<String, TimingStatsDto>,
    /// Layout timing statistics
    pub layout_stats: TimingStatsDto,
    /// Render timing statistics
    pub render_stats: TimingStatsDto,
    /// Input latency statistics
    pub input_latency_stats: TimingStatsDto,
    /// General timing statistics
    pub general_stats: std::collections::HashMap<String, TimingStatsDto>,
    /// Total number of commands recorded
    pub total_commands: usize,
    /// Total number of layout operations recorded
    pub total_layouts: usize,
    /// Total number of render operations recorded
    pub total_renders: usize,
    /// Total number of input events recorded
    pub total_inputs: usize,
}

/// Timing statistics DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimingStatsDto {
    /// Number of samples
    pub count: usize,
    /// Minimum time in milliseconds
    pub min_ms: f64,
    /// Maximum time in milliseconds
    pub max_ms: f64,
    /// Mean time in milliseconds
    pub mean_ms: f64,
    /// Median time in milliseconds
    pub median_ms: f64,
    /// 95th percentile in milliseconds
    pub p95_ms: f64,
    /// 99th percentile in milliseconds
    pub p99_ms: f64,
    /// Standard deviation in milliseconds
    pub std_dev_ms: f64,
    /// Total time in milliseconds
    pub total_ms: f64,
}

impl From<TimingStats> for TimingStatsDto {
    fn from(stats: TimingStats) -> Self {
        Self {
            count: stats.count,
            min_ms: stats.min_ms,
            max_ms: stats.max_ms,
            mean_ms: stats.mean_ms,
            median_ms: stats.median_ms,
            p95_ms: stats.p95_ms,
            p99_ms: stats.p99_ms,
            std_dev_ms: stats.std_dev_ms,
            total_ms: stats.total_ms,
        }
    }
}

impl From<PerfSummary> for PerfSummaryDto {
    fn from(summary: PerfSummary) -> Self {
        Self {
            command_stats: summary
                .command_stats
                .into_iter()
                .map(|(k, v)| (k, TimingStatsDto::from(v)))
                .collect(),
            layout_stats: TimingStatsDto::from(summary.layout_stats),
            render_stats: TimingStatsDto::from(summary.render_stats),
            input_latency_stats: TimingStatsDto::from(summary.input_latency_stats),
            general_stats: summary
                .general_stats
                .into_iter()
                .map(|(k, v)| (k, TimingStatsDto::from(v)))
                .collect(),
            total_commands: summary.total_commands,
            total_layouts: summary.total_layouts,
            total_renders: summary.total_renders,
            total_inputs: summary.total_inputs,
        }
    }
}

/// Performance budget DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerfBudgetDto {
    /// Maximum acceptable input latency in milliseconds
    pub max_input_latency_ms: f64,
    /// Maximum acceptable layout time per paragraph in milliseconds
    pub max_layout_time_ms: f64,
    /// Maximum acceptable render time per frame in milliseconds
    pub max_render_time_ms: f64,
    /// Maximum acceptable command execution time in milliseconds
    pub max_command_time_ms: f64,
}

impl From<&PerfBudget> for PerfBudgetDto {
    fn from(budget: &PerfBudget) -> Self {
        Self {
            max_input_latency_ms: budget.max_input_latency_ms,
            max_layout_time_ms: budget.max_layout_time_ms,
            max_render_time_ms: budget.max_render_time_ms,
            max_command_time_ms: budget.max_command_time_ms,
        }
    }
}

impl From<PerfBudgetDto> for PerfBudget {
    fn from(dto: PerfBudgetDto) -> Self {
        PerfBudget::new(
            dto.max_input_latency_ms,
            dto.max_layout_time_ms,
            dto.max_render_time_ms,
            dto.max_command_time_ms,
        )
    }
}

/// Budget violation DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetViolationDto {
    /// Category of the violation
    pub category: String,
    /// Actual measured time in milliseconds
    pub actual_ms: f64,
    /// Budget threshold in milliseconds
    pub budget_ms: f64,
    /// Severity of the violation
    pub severity: String,
}

impl From<BudgetViolation> for BudgetViolationDto {
    fn from(v: BudgetViolation) -> Self {
        Self {
            category: v.category.to_string(),
            actual_ms: v.actual_ms,
            budget_ms: v.budget_ms,
            severity: match v.severity {
                ViolationSeverity::Low => "low".to_string(),
                ViolationSeverity::Medium => "medium".to_string(),
                ViolationSeverity::High => "high".to_string(),
                ViolationSeverity::Critical => "critical".to_string(),
            },
        }
    }
}

/// Budget report DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetReportDto {
    /// All violations in this report
    pub violations: Vec<BudgetViolationDto>,
    /// Number of critical violations
    pub critical_count: usize,
    /// Number of high severity violations
    pub high_count: usize,
    /// Number of medium severity violations
    pub medium_count: usize,
    /// Number of low severity violations
    pub low_count: usize,
    /// Whether the report passes (no high or critical violations)
    pub passes: bool,
}

impl From<BudgetReport> for BudgetReportDto {
    fn from(report: BudgetReport) -> Self {
        let passes = report.passes();
        Self {
            violations: report.violations.into_iter().map(BudgetViolationDto::from).collect(),
            critical_count: report.critical_count,
            high_count: report.high_count,
            medium_count: report.medium_count,
            low_count: report.low_count,
            passes,
        }
    }
}

/// Get the current performance metrics summary
#[tauri::command]
pub fn get_performance_metrics(
    state: State<'_, PerfMetricsState>,
) -> Result<PerfSummaryDto, String> {
    let metrics = state.metrics.lock().map_err(|e| e.to_string())?;
    Ok(PerfSummaryDto::from(metrics.summary()))
}

/// Reset all performance metrics
#[tauri::command]
pub fn reset_performance_metrics(
    state: State<'_, PerfMetricsState>,
) -> Result<(), String> {
    let mut metrics = state.metrics.lock().map_err(|e| e.to_string())?;
    metrics.reset();
    Ok(())
}

/// Get the current performance budget
#[tauri::command]
pub fn get_performance_budget(
    state: State<'_, PerfMetricsState>,
) -> Result<PerfBudgetDto, String> {
    let metrics = state.metrics.lock().map_err(|e| e.to_string())?;
    Ok(PerfBudgetDto::from(metrics.budget()))
}

/// Set the performance budget
#[tauri::command]
pub fn set_performance_budget(
    budget: PerfBudgetDto,
    state: State<'_, PerfMetricsState>,
) -> Result<(), String> {
    let mut metrics = state.metrics.lock().map_err(|e| e.to_string())?;
    metrics.set_budget(PerfBudget::from(budget));
    Ok(())
}

/// Check the performance budget and get any violations
#[tauri::command]
pub fn check_performance_budget(
    state: State<'_, PerfMetricsState>,
) -> Result<Vec<BudgetViolationDto>, String> {
    let metrics = state.metrics.lock().map_err(|e| e.to_string())?;
    let violations = metrics.check_budget();
    Ok(violations.into_iter().map(BudgetViolationDto::from).collect())
}

/// Enable or disable performance metrics collection
#[tauri::command]
pub fn set_performance_enabled(
    enabled: bool,
    state: State<'_, PerfMetricsState>,
) -> Result<(), String> {
    let mut metrics = state.metrics.lock().map_err(|e| e.to_string())?;
    metrics.set_enabled(enabled);
    Ok(())
}

/// Check if performance metrics collection is enabled
#[tauri::command]
pub fn is_performance_enabled(
    state: State<'_, PerfMetricsState>,
) -> Result<bool, String> {
    let metrics = state.metrics.lock().map_err(|e| e.to_string())?;
    Ok(metrics.is_enabled())
}

/// Record a command timing measurement
#[tauri::command]
pub fn record_command_timing(
    name: String,
    duration_ms: f64,
    state: State<'_, PerfMetricsState>,
) -> Result<(), String> {
    let mut metrics = state.metrics.lock().map_err(|e| e.to_string())?;
    metrics.record_command(&name, duration_ms);
    Ok(())
}

/// Record a layout timing measurement
#[tauri::command]
pub fn record_layout_timing(
    duration_ms: f64,
    state: State<'_, PerfMetricsState>,
) -> Result<(), String> {
    let mut metrics = state.metrics.lock().map_err(|e| e.to_string())?;
    metrics.record_layout(duration_ms);
    Ok(())
}

/// Record a render timing measurement
#[tauri::command]
pub fn record_render_timing(
    duration_ms: f64,
    state: State<'_, PerfMetricsState>,
) -> Result<(), String> {
    let mut metrics = state.metrics.lock().map_err(|e| e.to_string())?;
    metrics.record_render(duration_ms);
    Ok(())
}

/// Record an input latency measurement
#[tauri::command]
pub fn record_input_latency(
    duration_ms: f64,
    state: State<'_, PerfMetricsState>,
) -> Result<(), String> {
    let mut metrics = state.metrics.lock().map_err(|e| e.to_string())?;
    metrics.record_input_latency(duration_ms);
    Ok(())
}

// =============================================================================
// Field Commands
// =============================================================================

use doc_model::field::{
    Field, FieldContext, FieldEvaluator, FieldInstruction, FieldRegistry, NumberFormat,
    RefDisplayType, RefOptions, SeqOptions, TocEntry, TocSwitches, TocTabLeader,
};
#[allow(unused_imports)]
use edit_engine::{FieldInfo, FieldUpdateEngine};

/// Field instruction DTO for frontend communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum FieldInstructionDto {
    /// Current page number (PAGE)
    Page { format: Option<String> },
    /// Total number of pages (NUMPAGES)
    NumPages { format: Option<String> },
    /// Current date (DATE)
    Date { format: String },
    /// Current time (TIME)
    Time { format: String },
    /// Table of Contents (TOC)
    Toc {
        heading_start: Option<u8>,
        heading_end: Option<u8>,
        include_page_numbers: Option<bool>,
        hyperlinks: Option<bool>,
    },
    /// Cross-reference to bookmark (REF)
    Ref {
        bookmark: String,
        display: Option<String>,
        hyperlink: Option<bool>,
    },
    /// Sequence numbering (SEQ)
    Seq {
        identifier: String,
        format: Option<String>,
    },
    /// Document author (AUTHOR)
    Author,
    /// Document title (TITLE)
    Title,
    /// Document subject (SUBJECT)
    Subject,
    /// Document file name (FILENAME)
    FileName { include_path: Option<bool> },
    /// Section number (SECTION)
    Section,
    /// Section page count (SECTIONPAGES)
    SectionPages,
    /// Hyperlink (HYPERLINK)
    Hyperlink {
        url: String,
        display_text: Option<String>,
    },
    /// Number of words (NUMWORDS)
    NumWords,
    /// Number of characters (NUMCHARS)
    NumChars,
    /// Custom field with arbitrary code
    Custom { code: String },
}

fn parse_number_format(format: &str) -> NumberFormat {
    match format.to_lowercase().as_str() {
        "arabic" | "1" => NumberFormat::Arabic,
        "lowercaseletter" | "a" => NumberFormat::LowercaseLetter,
        "uppercaseletter" | "A" => NumberFormat::UppercaseLetter,
        "lowercaseroman" | "i" => NumberFormat::LowercaseRoman,
        "uppercaseroman" | "I" => NumberFormat::UppercaseRoman,
        "ordinal" => NumberFormat::Ordinal,
        "cardinaltext" => NumberFormat::CardinalText,
        "ordinaltext" => NumberFormat::OrdinalText,
        _ => NumberFormat::Arabic,
    }
}

fn number_format_to_string(format: &NumberFormat) -> String {
    match format {
        NumberFormat::Arabic => "arabic".to_string(),
        NumberFormat::LowercaseLetter => "lowercaseLetter".to_string(),
        NumberFormat::UppercaseLetter => "uppercaseLetter".to_string(),
        NumberFormat::LowercaseRoman => "lowercaseRoman".to_string(),
        NumberFormat::UppercaseRoman => "uppercaseRoman".to_string(),
        NumberFormat::Ordinal => "ordinal".to_string(),
        NumberFormat::CardinalText => "cardinalText".to_string(),
        NumberFormat::OrdinalText => "ordinalText".to_string(),
    }
}

fn parse_ref_display(display: &str) -> RefDisplayType {
    match display.to_lowercase().as_str() {
        "content" => RefDisplayType::Content,
        "pagenumber" | "page" => RefDisplayType::PageNumber,
        "paragraphnumber" => RefDisplayType::ParagraphNumber,
        "paragraphnumberfullcontext" => RefDisplayType::ParagraphNumberFullContext,
        "relativeposition" | "above" | "below" => RefDisplayType::RelativePosition,
        _ => RefDisplayType::Content,
    }
}

impl From<FieldInstructionDto> for FieldInstruction {
    fn from(dto: FieldInstructionDto) -> Self {
        match dto {
            FieldInstructionDto::Page { format } => FieldInstruction::Page {
                format: format.map(|f| parse_number_format(&f)).unwrap_or_default(),
            },
            FieldInstructionDto::NumPages { format } => FieldInstruction::NumPages {
                format: format.map(|f| parse_number_format(&f)).unwrap_or_default(),
            },
            FieldInstructionDto::Date { format } => FieldInstruction::Date { format },
            FieldInstructionDto::Time { format } => FieldInstruction::Time { format },
            FieldInstructionDto::Toc {
                heading_start,
                heading_end,
                include_page_numbers,
                hyperlinks,
            } => FieldInstruction::Toc {
                switches: TocSwitches {
                    heading_levels: heading_start.unwrap_or(1)..heading_end.unwrap_or(4),
                    include_page_numbers: include_page_numbers.unwrap_or(true),
                    hyperlinks: hyperlinks.unwrap_or(true),
                    ..Default::default()
                },
            },
            FieldInstructionDto::Ref {
                bookmark,
                display,
                hyperlink,
            } => FieldInstruction::Ref {
                options: RefOptions {
                    bookmark,
                    display: display.map(|d| parse_ref_display(&d)).unwrap_or_default(),
                    hyperlink: hyperlink.unwrap_or(true),
                    include_position: false,
                },
            },
            FieldInstructionDto::Seq { identifier, format } => FieldInstruction::Seq {
                options: SeqOptions {
                    identifier,
                    format: format.map(|f| parse_number_format(&f)).unwrap_or_default(),
                    ..Default::default()
                },
            },
            FieldInstructionDto::Author => FieldInstruction::Author,
            FieldInstructionDto::Title => FieldInstruction::Title,
            FieldInstructionDto::Subject => FieldInstruction::Subject,
            FieldInstructionDto::FileName { include_path } => FieldInstruction::FileName {
                include_path: include_path.unwrap_or(false),
            },
            FieldInstructionDto::Section => FieldInstruction::Section,
            FieldInstructionDto::SectionPages => FieldInstruction::SectionPages,
            FieldInstructionDto::Hyperlink { url, display_text } => FieldInstruction::Hyperlink {
                url,
                display_text,
            },
            FieldInstructionDto::NumWords => FieldInstruction::NumWords,
            FieldInstructionDto::NumChars => FieldInstruction::NumChars,
            FieldInstructionDto::Custom { code } => FieldInstruction::Custom { code },
        }
    }
}

impl From<&FieldInstruction> for FieldInstructionDto {
    fn from(instruction: &FieldInstruction) -> Self {
        match instruction {
            FieldInstruction::Page { format } => FieldInstructionDto::Page {
                format: Some(number_format_to_string(format)),
            },
            FieldInstruction::NumPages { format } => FieldInstructionDto::NumPages {
                format: Some(number_format_to_string(format)),
            },
            FieldInstruction::Date { format } => FieldInstructionDto::Date {
                format: format.clone(),
            },
            FieldInstruction::Time { format } => FieldInstructionDto::Time {
                format: format.clone(),
            },
            FieldInstruction::Toc { switches } => FieldInstructionDto::Toc {
                heading_start: Some(switches.heading_levels.start),
                heading_end: Some(switches.heading_levels.end),
                include_page_numbers: Some(switches.include_page_numbers),
                hyperlinks: Some(switches.hyperlinks),
            },
            FieldInstruction::Ref { options } => FieldInstructionDto::Ref {
                bookmark: options.bookmark.clone(),
                display: Some(match options.display {
                    RefDisplayType::Content => "content".to_string(),
                    RefDisplayType::PageNumber => "pageNumber".to_string(),
                    RefDisplayType::ParagraphNumber => "paragraphNumber".to_string(),
                    RefDisplayType::ParagraphNumberFullContext => "paragraphNumberFullContext".to_string(),
                    RefDisplayType::RelativePosition => "relativePosition".to_string(),
                }),
                hyperlink: Some(options.hyperlink),
            },
            FieldInstruction::Seq { options } => FieldInstructionDto::Seq {
                identifier: options.identifier.clone(),
                format: Some(number_format_to_string(&options.format)),
            },
            FieldInstruction::Author => FieldInstructionDto::Author,
            FieldInstruction::Title => FieldInstructionDto::Title,
            FieldInstruction::Subject => FieldInstructionDto::Subject,
            FieldInstruction::FileName { include_path } => FieldInstructionDto::FileName {
                include_path: Some(*include_path),
            },
            FieldInstruction::Section => FieldInstructionDto::Section,
            FieldInstruction::SectionPages => FieldInstructionDto::SectionPages,
            FieldInstruction::Hyperlink { url, display_text } => FieldInstructionDto::Hyperlink {
                url: url.clone(),
                display_text: display_text.clone(),
            },
            FieldInstruction::NumWords => FieldInstructionDto::NumWords,
            FieldInstruction::NumChars => FieldInstructionDto::NumChars,
            FieldInstruction::Custom { code } => FieldInstructionDto::Custom { code: code.clone() },
            // Handle other variants
            _ => FieldInstructionDto::Custom {
                code: instruction.display_string(),
            },
        }
    }
}

/// Field information DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldDto {
    /// Field ID
    pub id: String,
    /// Field code name (e.g., "PAGE", "TOC")
    pub code_name: String,
    /// Full field code string
    pub code_string: String,
    /// Current result text
    pub result: String,
    /// Whether the field is locked
    pub locked: bool,
    /// Whether showing field code
    pub show_code: bool,
    /// Whether the field is dirty
    pub dirty: bool,
    /// The field instruction
    pub instruction: FieldInstructionDto,
}

impl From<&Field> for FieldDto {
    fn from(field: &Field) -> Self {
        Self {
            id: field.id().to_string(),
            code_name: field.instruction.code_name().to_string(),
            code_string: field.instruction.display_string(),
            result: field.display_text(),
            locked: field.locked,
            show_code: field.show_code,
            dirty: field.dirty,
            instruction: FieldInstructionDto::from(&field.instruction),
        }
    }
}

/// Insert a field at the current position
#[tauri::command]
pub fn insert_field(
    _doc_id: String,
    instruction: FieldInstructionDto,
) -> Result<FieldDto, String> {
    let field_instruction: FieldInstruction = instruction.into();
    let field = Field::new(field_instruction);

    // TODO: Actually insert the field into the document tree
    // For now, return the field data

    Ok(FieldDto::from(&field))
}

/// Update a specific field's result
#[tauri::command]
pub fn update_field(
    _doc_id: String,
    field_id: String,
) -> Result<FieldDto, String> {
    // TODO: Get the field from the document's field registry
    // Update it with the current context
    // For now, create a placeholder

    let mut field = Field::page();
    field.set_result("1".to_string());

    Ok(FieldDto::from(&field))
}

/// Update all fields in the document
#[tauri::command]
pub fn update_all_fields(
    _doc_id: String,
) -> Result<Vec<FieldDto>, String> {
    // TODO: Get the document's field registry
    // Build context from document and layout
    // Update all fields
    // For now, return empty list

    Ok(Vec::new())
}

/// Toggle field codes display (show instruction vs result)
#[tauri::command]
pub fn toggle_field_codes(
    _doc_id: String,
    field_id: Option<String>,
) -> Result<(), String> {
    // TODO: Toggle show_code flag on the specified field or all fields
    // If field_id is None, toggle all fields

    Ok(())
}

/// Lock or unlock a field
#[tauri::command]
pub fn lock_field(
    _doc_id: String,
    field_id: String,
    locked: bool,
) -> Result<(), String> {
    // TODO: Set the locked status on the field

    Ok(())
}

/// Get a field's result
#[tauri::command]
pub fn get_field_result(
    _doc_id: String,
    field_id: String,
) -> Result<String, String> {
    // TODO: Get the field from the registry and return its result

    Ok("".to_string())
}

/// Get all fields in the document
#[tauri::command]
pub fn list_fields(
    _doc_id: String,
) -> Result<Vec<FieldDto>, String> {
    // TODO: Get all fields from the document's field registry

    Ok(Vec::new())
}

/// Delete a field from the document
#[tauri::command]
pub fn delete_field(
    _doc_id: String,
    field_id: String,
) -> Result<(), String> {
    // TODO: Remove the field from the document tree and registry

    Ok(())
}

/// Get the text content of a field for editing
#[tauri::command]
pub fn get_field_code(
    _doc_id: String,
    field_id: String,
) -> Result<String, String> {
    // TODO: Get the field instruction display string

    Ok("PAGE".to_string())
}

/// Evaluate a field instruction without inserting it
#[tauri::command]
pub fn evaluate_field(
    instruction: FieldInstructionDto,
    page_number: Option<u32>,
    total_pages: Option<u32>,
) -> Result<String, String> {
    let field_instruction: FieldInstruction = instruction.into();
    let field = Field::new(field_instruction);

    let context = FieldContext::new()
        .with_page_info(page_number.unwrap_or(1), total_pages.unwrap_or(1))
        .with_now();

    let result = FieldEvaluator::evaluate(&field, &context);
    Ok(result)
}

// =============================================================================
// Section and Column Layout Commands
// =============================================================================

/// Insert a section break at the current position
#[tauri::command]
pub fn insert_section_break(
    doc_id: String,
    break_type: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use doc_model::SectionBreakType;

    let mut documents = state.documents.lock().map_err(|e| e.to_string())?;
    let doc_state = documents
        .get_mut(&doc_id)
        .ok_or_else(|| format!("Document not found: {}", doc_id))?;

    let break_type = match break_type.as_str() {
        "next_page" | "NextPage" => SectionBreakType::NextPage,
        "continuous" | "Continuous" => SectionBreakType::Continuous,
        "even_page" | "EvenPage" => SectionBreakType::EvenPage,
        "odd_page" | "OddPage" => SectionBreakType::OddPage,
        _ => SectionBreakType::NextPage,
    };

    let position = doc_state.selection.anchor;

    let cmd = edit_engine::InsertSectionBreak::new(position, break_type);
    let result = cmd.apply(&doc_state.tree, &doc_state.selection)
        .map_err(|e| e.to_string())?;

    doc_state.tree = result.tree;
    doc_state.selection = result.selection;

    Ok(())
}

/// Set the column layout for the current section
#[tauri::command]
pub fn set_column_layout(
    doc_id: String,
    column_count: u32,
    column_spacing: Option<f32>,
    draw_separator: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut documents = state.documents.lock().map_err(|e| e.to_string())?;
    let doc_state = documents
        .get_mut(&doc_id)
        .ok_or_else(|| format!("Document not found: {}", doc_id))?;

    let cmd = edit_engine::SetColumnLayout {
        section_id: None,
        column_count,
        column_spacing,
        equal_width: true,
        custom_columns: None,
        draw_separator,
    };

    let result = cmd.apply(&doc_state.tree, &doc_state.selection)
        .map_err(|e| e.to_string())?;

    doc_state.tree = result.tree;
    doc_state.selection = result.selection;

    Ok(())
}

/// Set a custom column layout with specific widths
#[tauri::command]
pub fn set_custom_column_layout(
    doc_id: String,
    columns: Vec<edit_engine::ColumnDefDto>,
    column_spacing: f32,
    draw_separator: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut documents = state.documents.lock().map_err(|e| e.to_string())?;
    let doc_state = documents
        .get_mut(&doc_id)
        .ok_or_else(|| format!("Document not found: {}", doc_id))?;

    let cmd = edit_engine::SetColumnLayout {
        section_id: None,
        column_count: columns.len() as u32,
        column_spacing: Some(column_spacing),
        equal_width: false,
        custom_columns: Some(columns),
        draw_separator,
    };

    let result = cmd.apply(&doc_state.tree, &doc_state.selection)
        .map_err(|e| e.to_string())?;

    doc_state.tree = result.tree;
    doc_state.selection = result.selection;

    Ok(())
}

/// Insert a column break at the current position
#[tauri::command]
pub fn insert_column_break(
    doc_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut documents = state.documents.lock().map_err(|e| e.to_string())?;
    let doc_state = documents
        .get_mut(&doc_id)
        .ok_or_else(|| format!("Document not found: {}", doc_id))?;

    let position = doc_state.selection.anchor;
    let cmd = edit_engine::InsertColumnBreak::new(position);

    let result = cmd.apply(&doc_state.tree, &doc_state.selection)
        .map_err(|e| e.to_string())?;

    doc_state.tree = result.tree;
    doc_state.selection = result.selection;

    Ok(())
}

/// Get the properties of the current section
#[tauri::command]
pub fn get_section_properties(
    _state: State<'_, AppState>,
) -> Result<edit_engine::SectionProperties, String> {
    // For now, return default section properties
    // In a full implementation, this would look up the section containing the cursor
    Ok(edit_engine::SectionProperties::default_properties())
}

/// Set section page setup (margins, size, orientation)
#[tauri::command]
pub fn set_section_page_setup(
    doc_id: String,
    page_size: Option<String>,
    orientation: Option<String>,
    margin_top: Option<f32>,
    margin_bottom: Option<f32>,
    margin_left: Option<f32>,
    margin_right: Option<f32>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use doc_model::{Orientation, PageSizePreset};

    let mut documents = state.documents.lock().map_err(|e| e.to_string())?;
    let doc_state = documents
        .get_mut(&doc_id)
        .ok_or_else(|| format!("Document not found: {}", doc_id))?;

    let page_size_preset = page_size.and_then(|s| match s.as_str() {
        "Letter" | "letter" => Some(PageSizePreset::Letter),
        "A4" | "a4" => Some(PageSizePreset::A4),
        "Legal" | "legal" => Some(PageSizePreset::Legal),
        "Executive" | "executive" => Some(PageSizePreset::Executive),
        "A5" | "a5" => Some(PageSizePreset::A5),
        _ => None,
    });

    let orientation = orientation.and_then(|s| match s.as_str() {
        "Portrait" | "portrait" => Some(Orientation::Portrait),
        "Landscape" | "landscape" => Some(Orientation::Landscape),
        _ => None,
    });

    let cmd = edit_engine::SetPageSetup {
        section_id: None,
        page_size_preset,
        custom_width: None,
        custom_height: None,
        orientation,
        margin_top,
        margin_bottom,
        margin_left,
        margin_right,
        margin_header: None,
        margin_footer: None,
        gutter: None,
        gutter_position: None,
    };

    let result = cmd.apply(&doc_state.tree, &doc_state.selection)
        .map_err(|e| e.to_string())?;

    doc_state.tree = result.tree;
    doc_state.selection = result.selection;

    Ok(())
}

/// Preset column layouts
#[tauri::command]
pub fn set_column_preset(
    doc_id: String,
    preset: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    match preset.as_str() {
        "one" | "single" => set_column_layout(doc_id, 1, None, None, state),
        "two" => set_column_layout(doc_id, 2, Some(36.0), None, state),
        "three" => set_column_layout(doc_id, 3, Some(36.0), None, state),
        "left" => {
            // Left column narrower than right
            let columns = vec![
                edit_engine::ColumnDefDto { width: 150.0, space_after: 36.0 },
                edit_engine::ColumnDefDto { width: 282.0, space_after: 0.0 },
            ];
            set_custom_column_layout(doc_id, columns, 36.0, None, state)
        }
        "right" => {
            // Right column narrower than left
            let columns = vec![
                edit_engine::ColumnDefDto { width: 282.0, space_after: 36.0 },
                edit_engine::ColumnDefDto { width: 150.0, space_after: 0.0 },
            ];
            set_custom_column_layout(doc_id, columns, 36.0, None, state)
        }
        _ => Err(format!("Unknown column preset: {}", preset)),
    }
}

// =============================================================================
// Comment Commands
// =============================================================================

use edit_engine::{CommentInfo, ReplyInfo};

/// Comment DTO for frontend communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentDto {
    /// The comment ID
    pub id: String,
    /// Author of the comment
    pub author: String,
    /// Date created (ISO 8601 format)
    pub date: String,
    /// Content of the comment
    pub content: String,
    /// Number of replies
    pub reply_count: usize,
    /// Whether the comment is resolved
    pub resolved: bool,
    /// Who resolved the comment (if resolved)
    pub resolved_by: Option<String>,
    /// When the comment was resolved (if resolved)
    pub resolved_date: Option<String>,
    /// Preview of the commented text
    pub text_preview: Option<String>,
    /// Anchor start node ID
    pub anchor_start_node: String,
    /// Anchor start offset
    pub anchor_start_offset: usize,
    /// Anchor end node ID
    pub anchor_end_node: String,
    /// Anchor end offset
    pub anchor_end_offset: usize,
}

impl From<CommentInfo> for CommentDto {
    fn from(info: CommentInfo) -> Self {
        Self {
            id: info.id,
            author: info.author,
            date: info.date,
            content: info.content,
            reply_count: info.reply_count,
            resolved: info.resolved,
            resolved_by: info.resolved_by,
            resolved_date: info.resolved_date,
            text_preview: info.text_preview,
            anchor_start_node: info.anchor_start_node,
            anchor_start_offset: info.anchor_start_offset,
            anchor_end_node: info.anchor_end_node,
            anchor_end_offset: info.anchor_end_offset,
        }
    }
}

/// Reply DTO for frontend communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentReplyDto {
    /// The reply ID
    pub id: String,
    /// Author of the reply
    pub author: String,
    /// Date created (ISO 8601 format)
    pub date: String,
    /// Content of the reply
    pub content: String,
}

impl From<ReplyInfo> for CommentReplyDto {
    fn from(info: ReplyInfo) -> Self {
        Self {
            id: info.id,
            author: info.author,
            date: info.date,
            content: info.content,
        }
    }
}

/// Range DTO for specifying comment anchor
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentRangeDto {
    /// Start node ID
    pub start_node: String,
    /// Start offset
    pub start_offset: usize,
    /// End node ID
    pub end_node: String,
    /// End offset
    pub end_offset: usize,
}

/// Add a comment to a selection in the document
#[tauri::command]
pub fn add_comment(
    _doc_id: String,
    range: CommentRangeDto,
    author: String,
    content: String,
) -> Result<CommentDto, String> {
    if author.trim().is_empty() {
        return Err("Author cannot be empty".to_string());
    }
    if content.trim().is_empty() {
        return Err("Content cannot be empty".to_string());
    }

    let now = chrono::Utc::now();
    Ok(CommentDto {
        id: uuid::Uuid::new_v4().to_string(),
        author,
        date: now.to_rfc3339(),
        content,
        reply_count: 0,
        resolved: false,
        resolved_by: None,
        resolved_date: None,
        text_preview: None,
        anchor_start_node: range.start_node,
        anchor_start_offset: range.start_offset,
        anchor_end_node: range.end_node,
        anchor_end_offset: range.end_offset,
    })
}

/// Edit a comment's content
#[tauri::command]
pub fn edit_comment(_doc_id: String, comment_id: String, content: String) -> Result<(), String> {
    if content.trim().is_empty() {
        return Err("Content cannot be empty".to_string());
    }
    if comment_id.is_empty() {
        return Err("Comment ID cannot be empty".to_string());
    }
    Ok(())
}

/// Delete a comment
#[tauri::command]
pub fn delete_comment(_doc_id: String, comment_id: String) -> Result<(), String> {
    if comment_id.is_empty() {
        return Err("Comment ID cannot be empty".to_string());
    }
    Ok(())
}

/// Reply to a comment
#[tauri::command]
pub fn reply_to_comment(
    _doc_id: String,
    comment_id: String,
    author: String,
    content: String,
) -> Result<CommentReplyDto, String> {
    if author.trim().is_empty() {
        return Err("Author cannot be empty".to_string());
    }
    if content.trim().is_empty() {
        return Err("Content cannot be empty".to_string());
    }
    if comment_id.is_empty() {
        return Err("Comment ID cannot be empty".to_string());
    }

    let now = chrono::Utc::now();
    Ok(CommentReplyDto {
        id: uuid::Uuid::new_v4().to_string(),
        author,
        date: now.to_rfc3339(),
        content,
    })
}

/// Edit a reply's content
#[tauri::command]
pub fn edit_comment_reply(
    _doc_id: String,
    comment_id: String,
    reply_id: String,
    content: String,
) -> Result<(), String> {
    if content.trim().is_empty() {
        return Err("Content cannot be empty".to_string());
    }
    if comment_id.is_empty() {
        return Err("Comment ID cannot be empty".to_string());
    }
    if reply_id.is_empty() {
        return Err("Reply ID cannot be empty".to_string());
    }
    Ok(())
}

/// Delete a reply
#[tauri::command]
pub fn delete_comment_reply(
    _doc_id: String,
    comment_id: String,
    reply_id: String,
) -> Result<(), String> {
    if comment_id.is_empty() {
        return Err("Comment ID cannot be empty".to_string());
    }
    if reply_id.is_empty() {
        return Err("Reply ID cannot be empty".to_string());
    }
    Ok(())
}

/// Resolve a comment
#[tauri::command]
pub fn resolve_comment(
    _doc_id: String,
    comment_id: String,
    resolved_by: String,
) -> Result<(), String> {
    if comment_id.is_empty() {
        return Err("Comment ID cannot be empty".to_string());
    }
    if resolved_by.trim().is_empty() {
        return Err("Resolved by cannot be empty".to_string());
    }
    Ok(())
}

/// Reopen a resolved comment
#[tauri::command]
pub fn reopen_comment(_doc_id: String, comment_id: String) -> Result<(), String> {
    if comment_id.is_empty() {
        return Err("Comment ID cannot be empty".to_string());
    }
    Ok(())
}

/// Get all comments in a document
#[tauri::command]
pub fn get_comments(_doc_id: String) -> Result<Vec<CommentDto>, String> {
    Ok(Vec::new())
}

/// Get comments filtered by various criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentFilterDto {
    pub author: Option<String>,
    pub resolved: Option<bool>,
    pub sort_by: Option<String>,
}

/// Get comments with optional filtering
#[tauri::command]
pub fn get_comments_filtered(
    _doc_id: String,
    _filter: CommentFilterDto,
) -> Result<Vec<CommentDto>, String> {
    Ok(Vec::new())
}

/// Get comments that overlap with a specific range
#[tauri::command]
pub fn get_comments_for_range(
    _doc_id: String,
    _range: CommentRangeDto,
) -> Result<Vec<CommentDto>, String> {
    Ok(Vec::new())
}

/// Get a single comment by ID
#[tauri::command]
pub fn get_comment(_doc_id: String, comment_id: String) -> Result<Option<CommentDto>, String> {
    if comment_id.is_empty() {
        return Err("Comment ID cannot be empty".to_string());
    }
    Ok(None)
}

/// Get replies for a comment
#[tauri::command]
pub fn get_comment_replies(
    _doc_id: String,
    comment_id: String,
) -> Result<Vec<CommentReplyDto>, String> {
    if comment_id.is_empty() {
        return Err("Comment ID cannot be empty".to_string());
    }
    Ok(Vec::new())
}

/// Navigate to a comment (returns the selection to set)
#[tauri::command]
pub fn navigate_to_comment(_doc_id: String, comment_id: String) -> Result<Selection, String> {
    if comment_id.is_empty() {
        return Err("Comment ID cannot be empty".to_string());
    }
    Ok(Selection {
        anchor: Position {
            node_id: "placeholder".to_string(),
            offset: 0,
        },
        focus: Position {
            node_id: "placeholder".to_string(),
            offset: 0,
        },
    })
}

/// Comment count DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentCountDto {
    pub total: usize,
    pub resolved: usize,
    pub unresolved: usize,
}

/// Get comment count in the document
#[tauri::command]
pub fn get_comment_count(_doc_id: String) -> Result<CommentCountDto, String> {
    Ok(CommentCountDto {
        total: 0,
        resolved: 0,
        unresolved: 0,
    })
}

// =============================================================================
// RTF Import/Export Commands
// =============================================================================

/// Import warning DTO for RTF/ODT import
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportWarningDto {
    /// Kind of warning
    pub kind: String,
    /// Warning message
    pub message: String,
}

/// RTF import result DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RtfImportResultDto {
    /// Document ID of the imported document
    pub doc_id: String,
    /// Document title (if available)
    pub title: Option<String>,
    /// Number of paragraphs
    pub paragraph_count: usize,
    /// Approximate word count
    pub word_count: usize,
    /// Warnings encountered during import
    pub warnings: Vec<ImportWarningDto>,
}

/// Import an RTF file and return document data
///
/// # Arguments
///
/// * `path` - Path to the RTF file
///
/// # Returns
///
/// Document information including ID, metadata, and any import warnings
#[tauri::command]
pub fn import_rtf(path: String) -> Result<RtfImportResultDto, String> {
    use std::path::Path;

    let path = Path::new(&path);
    let result = store::import_rtf(path)
        .map_err(|e| format!("Failed to import RTF: {}", e))?;

    let tree = result.tree;

    // Count paragraphs and estimate word count
    let paragraph_count = tree.paragraphs().count();
    let text_content = tree.text_content();
    let word_count = text_content.split_whitespace().count();

    // Get title from metadata
    let title = tree.document.metadata.title.clone();

    // Convert warnings
    let warnings: Vec<ImportWarningDto> = result.warnings.iter().map(|w| {
        ImportWarningDto {
            kind: format!("{:?}", w.kind),
            message: w.message.clone(),
        }
    }).collect();

    Ok(RtfImportResultDto {
        doc_id: tree.root_id().to_string(),
        title,
        paragraph_count,
        word_count,
        warnings,
    })
}

/// Export a document to RTF format
///
/// # Arguments
///
/// * `doc_id` - The document ID
/// * `path` - Path where the RTF file will be saved
#[tauri::command]
pub fn export_rtf(_doc_id: String, path: String) -> Result<(), String> {
    use std::path::Path;

    // TODO: Get actual document from document state
    // For now, create an empty document
    let tree = doc_model::DocumentTree::new();

    store::export_rtf(&tree, Path::new(&path))
        .map_err(|e| format!("Failed to export RTF: {}", e))
}

/// Import RTF from bytes (for clipboard or drag-drop operations)
#[tauri::command]
pub fn import_rtf_bytes(data: Vec<u8>) -> Result<RtfImportResultDto, String> {
    let result = store::import_rtf_bytes(&data)
        .map_err(|e| format!("Failed to import RTF: {}", e))?;

    let tree = result.tree;

    let paragraph_count = tree.paragraphs().count();
    let text_content = tree.text_content();
    let word_count = text_content.split_whitespace().count();
    let title = tree.document.metadata.title.clone();

    let warnings: Vec<ImportWarningDto> = result.warnings.iter().map(|w| {
        ImportWarningDto {
            kind: format!("{:?}", w.kind),
            message: w.message.clone(),
        }
    }).collect();

    Ok(RtfImportResultDto {
        doc_id: tree.root_id().to_string(),
        title,
        paragraph_count,
        word_count,
        warnings,
    })
}

/// Export a document to RTF bytes (for clipboard operations)
#[tauri::command]
pub fn export_rtf_bytes(_doc_id: String) -> Result<Vec<u8>, String> {
    // TODO: Get actual document from document state
    let tree = doc_model::DocumentTree::new();

    store::export_rtf_bytes(&tree)
        .map_err(|e| format!("Failed to export RTF: {}", e))
}

// =============================================================================
// ODT Import Commands (Read-Only)
// =============================================================================

/// ODT import result DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OdtImportResultDto {
    /// Document ID of the imported document
    pub doc_id: String,
    /// Document title (if available)
    pub title: Option<String>,
    /// Document author (if available)
    pub author: Option<String>,
    /// Number of paragraphs
    pub paragraph_count: usize,
    /// Approximate word count
    pub word_count: usize,
    /// Warnings encountered during import
    pub warnings: Vec<ImportWarningDto>,
}

/// Import an ODT file and return document data
///
/// # Arguments
///
/// * `path` - Path to the ODT file
///
/// # Returns
///
/// Document information including ID, metadata, and any import warnings
///
/// # Note
///
/// ODT import is read-only. Export to ODT is not supported.
#[tauri::command]
pub fn import_odt(path: String) -> Result<OdtImportResultDto, String> {
    use std::path::Path;

    let path = Path::new(&path);
    let result = store::import_odt(path)
        .map_err(|e| format!("Failed to import ODT: {}", e))?;

    let tree = result.tree;

    // Count paragraphs and estimate word count
    let paragraph_count = tree.paragraphs().count();
    let text_content = tree.text_content();
    let word_count = text_content.split_whitespace().count();

    // Get metadata
    let title = tree.document.metadata.title.clone();
    let author = tree.document.metadata.author.clone();

    // Convert warnings
    let warnings: Vec<ImportWarningDto> = result.warnings.iter().map(|w| {
        ImportWarningDto {
            kind: format!("{:?}", w.kind),
            message: w.message.clone(),
        }
    }).collect();

    Ok(OdtImportResultDto {
        doc_id: tree.root_id().to_string(),
        title,
        author,
        paragraph_count,
        word_count,
        warnings,
    })
}

/// Import ODT from bytes (for drag-drop operations)
///
/// # Note
///
/// ODT import is read-only. Export to ODT is not supported.
#[tauri::command]
pub fn import_odt_bytes(data: Vec<u8>) -> Result<OdtImportResultDto, String> {
    let result = store::import_odt_bytes(&data)
        .map_err(|e| format!("Failed to import ODT: {}", e))?;

    let tree = result.tree;

    let paragraph_count = tree.paragraphs().count();
    let text_content = tree.text_content();
    let word_count = text_content.split_whitespace().count();
    let title = tree.document.metadata.title.clone();
    let author = tree.document.metadata.author.clone();

    let warnings: Vec<ImportWarningDto> = result.warnings.iter().map(|w| {
        ImportWarningDto {
            kind: format!("{:?}", w.kind),
            message: w.message.clone(),
        }
    }).collect();

    Ok(OdtImportResultDto {
        doc_id: tree.root_id().to_string(),
        title,
        author,
        paragraph_count,
        word_count,
        warnings,
    })
}

// =============================================================================
// Extended File Format Support
// =============================================================================

/// Get all supported import formats including RTF and ODT
#[tauri::command]
pub fn get_all_import_formats() -> Vec<FileFormatDto> {
    let mut formats = docx::get_import_formats()
        .into_iter()
        .map(FileFormatDto::from)
        .collect::<Vec<_>>();

    // Add RTF format
    formats.push(FileFormatDto {
        id: "rtf".to_string(),
        extension: "rtf".to_string(),
        mime_type: "application/rtf".to_string(),
        display_name: "Rich Text Format".to_string(),
        supports_import: true,
        supports_export: true,
    });

    // Add ODT format (read-only)
    formats.push(FileFormatDto {
        id: "odt".to_string(),
        extension: "odt".to_string(),
        mime_type: "application/vnd.oasis.opendocument.text".to_string(),
        display_name: "OpenDocument Text".to_string(),
        supports_import: true,
        supports_export: false, // ODT export not supported
    });

    formats
}

/// Get all supported export formats including RTF
#[tauri::command]
pub fn get_all_export_formats() -> Vec<FileFormatDto> {
    let mut formats = docx::get_export_formats()
        .into_iter()
        .map(FileFormatDto::from)
        .collect::<Vec<_>>();

    // Add RTF format
    formats.push(FileFormatDto {
        id: "rtf".to_string(),
        extension: "rtf".to_string(),
        mime_type: "application/rtf".to_string(),
        display_name: "Rich Text Format".to_string(),
        supports_import: true,
        supports_export: true,
    });

    // Note: ODT is not included as it doesn't support export

    formats
}

// =============================================================================
// Table Commands
// =============================================================================

use doc_model::{
    CellVerticalAlign, HorizontalMerge, TableAutoFitMode, VerticalMerge,
    table::CellTextDirection as TableTextDirection,
};

/// Table cell info DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableCellInfoDto {
    pub cell_id: String,
    pub row_index: usize,
    pub col_index: usize,
    pub grid_span: u32,
    pub row_span: u32,
    pub h_merge: String,
    pub v_merge: String,
    pub is_covered: bool,
    pub vertical_align: String,
    pub text_direction: String,
    pub shading: Option<String>,
}

/// Table row info DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRowInfoDto {
    pub row_id: String,
    pub row_index: usize,
    pub is_header: bool,
    pub can_split: bool,
    pub cant_split: bool,
    pub height: Option<f32>,
    pub cells: Vec<TableCellInfoDto>,
}

/// Table info DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableInfoDto {
    pub table_id: String,
    pub row_count: usize,
    pub column_count: usize,
    pub auto_fit_mode: String,
    pub nesting_depth: usize,
    pub rows: Vec<TableRowInfoDto>,
    pub header_row_indices: Vec<usize>,
}

/// Merged cell region DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergedCellRegionDto {
    pub anchor_cell_id: String,
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
    pub row_span: usize,
    pub col_span: usize,
}

fn h_merge_to_string(merge: HorizontalMerge) -> String {
    match merge {
        HorizontalMerge::None => "none".to_string(),
        HorizontalMerge::Start => "start".to_string(),
        HorizontalMerge::Continue => "continue".to_string(),
    }
}

fn v_merge_to_string(merge: VerticalMerge) -> String {
    match merge {
        VerticalMerge::None => "none".to_string(),
        VerticalMerge::Start => "start".to_string(),
        VerticalMerge::Continue => "continue".to_string(),
    }
}

fn cell_v_align_to_string(align: CellVerticalAlign) -> String {
    match align {
        CellVerticalAlign::Top => "top".to_string(),
        CellVerticalAlign::Center => "center".to_string(),
        CellVerticalAlign::Bottom => "bottom".to_string(),
    }
}

fn text_direction_to_string(direction: TableTextDirection) -> String {
    match direction {
        TableTextDirection::Ltr => "ltr".to_string(),
        TableTextDirection::Rtl => "rtl".to_string(),
        TableTextDirection::TbLr => "tb-lr".to_string(),
        TableTextDirection::TbRl => "tb-rl".to_string(),
    }
}

fn auto_fit_mode_to_string(mode: TableAutoFitMode) -> String {
    match mode {
        TableAutoFitMode::AutoFitContent => "autoFitContent".to_string(),
        TableAutoFitMode::AutoFitWindow => "autoFitWindow".to_string(),
        TableAutoFitMode::FixedWidth => "fixedWidth".to_string(),
    }
}

/// Insert a new table at the current position
#[tauri::command]
pub fn table_insert(
    _doc_id: String,
    rows: usize,
    cols: usize,
    _width: Option<f32>,
) -> Result<TableInfoDto, String> {
    // TODO: Implement with actual document state
    let table_id = uuid::Uuid::new_v4().to_string();

    let row_infos: Vec<TableRowInfoDto> = (0..rows)
        .map(|row_idx| {
            let cells: Vec<TableCellInfoDto> = (0..cols)
                .map(|col_idx| TableCellInfoDto {
                    cell_id: uuid::Uuid::new_v4().to_string(),
                    row_index: row_idx,
                    col_index: col_idx,
                    grid_span: 1,
                    row_span: 1,
                    h_merge: "none".to_string(),
                    v_merge: "none".to_string(),
                    is_covered: false,
                    vertical_align: "top".to_string(),
                    text_direction: "ltr".to_string(),
                    shading: None,
                })
                .collect();

            TableRowInfoDto {
                row_id: uuid::Uuid::new_v4().to_string(),
                row_index: row_idx,
                is_header: false,
                can_split: true,
                cant_split: false,
                height: None,
                cells,
            }
        })
        .collect();

    Ok(TableInfoDto {
        table_id,
        row_count: rows,
        column_count: cols,
        auto_fit_mode: "autoFitContent".to_string(),
        nesting_depth: 0,
        rows: row_infos,
        header_row_indices: Vec::new(),
    })
}

/// Merge cells horizontally (colspan)
#[tauri::command]
pub fn table_merge_cells_horizontal(
    _doc_id: String,
    _table_id: String,
    _row_index: usize,
    start_col: usize,
    end_col: usize,
) -> Result<DocumentChange, String> {
    if start_col > end_col {
        return Err("Start column must be less than or equal to end column".to_string());
    }
    Ok(DocumentChange::default())
}

/// Merge cells vertically (rowspan)
#[tauri::command]
pub fn table_merge_cells_vertical(
    _doc_id: String,
    _table_id: String,
    start_row: usize,
    end_row: usize,
    _col_index: usize,
) -> Result<DocumentChange, String> {
    if start_row > end_row {
        return Err("Start row must be less than or equal to end row".to_string());
    }
    Ok(DocumentChange::default())
}

/// Split a merged cell horizontally
#[tauri::command]
pub fn table_split_cell_horizontal(
    _doc_id: String,
    _table_id: String,
    _row_index: usize,
    _col_index: usize,
    split_count: usize,
) -> Result<DocumentChange, String> {
    if split_count < 2 {
        return Err("Split count must be at least 2".to_string());
    }
    Ok(DocumentChange::default())
}

/// Split a merged cell vertically
#[tauri::command]
pub fn table_split_cell_vertical(
    _doc_id: String,
    _table_id: String,
    _start_row: usize,
    _col_index: usize,
    split_count: usize,
) -> Result<DocumentChange, String> {
    if split_count < 2 {
        return Err("Split count must be at least 2".to_string());
    }
    Ok(DocumentChange::default())
}

/// Set row as header row (repeats on page breaks)
#[tauri::command]
pub fn table_set_header_row(
    _doc_id: String,
    _table_id: String,
    _row_index: usize,
    _is_header: bool,
) -> Result<DocumentChange, String> {
    Ok(DocumentChange::default())
}

/// Set row break behavior
#[tauri::command]
pub fn table_set_row_can_split(
    _doc_id: String,
    _table_id: String,
    _row_index: usize,
    _can_split: bool,
) -> Result<DocumentChange, String> {
    Ok(DocumentChange::default())
}

/// Set cell vertical alignment
#[tauri::command]
pub fn table_set_cell_vertical_align(
    _doc_id: String,
    _cell_ids: Vec<String>,
    alignment: String,
) -> Result<DocumentChange, String> {
    match alignment.as_str() {
        "top" | "center" | "bottom" => {}
        _ => return Err(format!("Invalid alignment: {}", alignment)),
    }
    Ok(DocumentChange::default())
}

/// Set cell text direction
#[tauri::command]
pub fn table_set_cell_text_direction(
    _doc_id: String,
    _cell_ids: Vec<String>,
    direction: String,
) -> Result<DocumentChange, String> {
    match direction.as_str() {
        "ltr" | "rtl" | "tb-lr" | "tb-rl" => {}
        _ => return Err(format!("Invalid text direction: {}", direction)),
    }
    Ok(DocumentChange::default())
}

/// Set cell padding
#[tauri::command]
pub fn table_set_cell_padding(
    _doc_id: String,
    _cell_ids: Vec<String>,
    _top: f32,
    _right: f32,
    _bottom: f32,
    _left: f32,
) -> Result<DocumentChange, String> {
    Ok(DocumentChange::default())
}

/// Set table auto-fit mode
#[tauri::command]
pub fn table_set_auto_fit(
    _doc_id: String,
    _table_id: String,
    mode: String,
) -> Result<DocumentChange, String> {
    match mode.as_str() {
        "autoFitContent" | "autoFitWindow" | "fixedWidth" => {}
        _ => return Err(format!("Invalid auto-fit mode: {}", mode)),
    }
    Ok(DocumentChange::default())
}

/// Insert a nested table inside a cell
#[tauri::command]
pub fn table_insert_nested(
    _doc_id: String,
    _cell_id: String,
    rows: usize,
    cols: usize,
    _width: Option<f32>,
) -> Result<TableInfoDto, String> {
    let table_id = uuid::Uuid::new_v4().to_string();

    let row_infos: Vec<TableRowInfoDto> = (0..rows)
        .map(|row_idx| {
            let cells: Vec<TableCellInfoDto> = (0..cols)
                .map(|col_idx| TableCellInfoDto {
                    cell_id: uuid::Uuid::new_v4().to_string(),
                    row_index: row_idx,
                    col_index: col_idx,
                    grid_span: 1,
                    row_span: 1,
                    h_merge: "none".to_string(),
                    v_merge: "none".to_string(),
                    is_covered: false,
                    vertical_align: "top".to_string(),
                    text_direction: "ltr".to_string(),
                    shading: None,
                })
                .collect();

            TableRowInfoDto {
                row_id: uuid::Uuid::new_v4().to_string(),
                row_index: row_idx,
                is_header: false,
                can_split: true,
                cant_split: false,
                height: None,
                cells,
            }
        })
        .collect();

    Ok(TableInfoDto {
        table_id,
        row_count: rows,
        column_count: cols,
        auto_fit_mode: "autoFitContent".to_string(),
        nesting_depth: 1,
        rows: row_infos,
        header_row_indices: Vec::new(),
    })
}

/// Get table info
#[tauri::command]
pub fn table_get_info(
    _doc_id: String,
    _table_id: String,
) -> Result<Option<TableInfoDto>, String> {
    Ok(None)
}

/// Get merged cell regions in a table
#[tauri::command]
pub fn table_get_merged_regions(
    _doc_id: String,
    _table_id: String,
) -> Result<Vec<MergedCellRegionDto>, String> {
    Ok(Vec::new())
}

/// Check if cells can be merged
#[tauri::command]
pub fn table_can_merge_cells(
    _doc_id: String,
    _table_id: String,
    start_row: usize,
    start_col: usize,
    end_row: usize,
    end_col: usize,
) -> Result<bool, String> {
    if start_row > end_row || start_col > end_col {
        return Ok(false);
    }
    Ok(true)
}

/// Insert a row above the specified row
#[tauri::command]
pub fn table_insert_row_above(
    _doc_id: String,
    _table_id: String,
    _row_index: usize,
) -> Result<DocumentChange, String> {
    Ok(DocumentChange::default())
}

/// Insert a row below the specified row
#[tauri::command]
pub fn table_insert_row_below(
    _doc_id: String,
    _table_id: String,
    _row_index: usize,
) -> Result<DocumentChange, String> {
    Ok(DocumentChange::default())
}

/// Insert a column to the left
#[tauri::command]
pub fn table_insert_column_left(
    _doc_id: String,
    _table_id: String,
    _col_index: usize,
) -> Result<DocumentChange, String> {
    Ok(DocumentChange::default())
}

/// Insert a column to the right
#[tauri::command]
pub fn table_insert_column_right(
    _doc_id: String,
    _table_id: String,
    _col_index: usize,
) -> Result<DocumentChange, String> {
    Ok(DocumentChange::default())
}

/// Delete a row from the table
#[tauri::command]
pub fn table_delete_row(
    _doc_id: String,
    _table_id: String,
    _row_index: usize,
) -> Result<DocumentChange, String> {
    Ok(DocumentChange::default())
}

/// Delete a column from the table
#[tauri::command]
pub fn table_delete_column(
    _doc_id: String,
    _table_id: String,
    _col_index: usize,
) -> Result<DocumentChange, String> {
    Ok(DocumentChange::default())
}

/// Delete the entire table
#[tauri::command]
pub fn table_delete(
    _doc_id: String,
    _table_id: String,
) -> Result<DocumentChange, String> {
    Ok(DocumentChange::default())
}

/// Set cell shading/background color
#[tauri::command]
pub fn table_set_cell_shading(
    _doc_id: String,
    _cell_ids: Vec<String>,
    _color: Option<String>,
) -> Result<DocumentChange, String> {
    Ok(DocumentChange::default())
}

// =============================================================================
// Caption Commands
// =============================================================================

use doc_model::caption::{
    Caption, CaptionBuilder, CaptionFormat, CaptionLabel, CaptionPosition, CaptionRefDisplayType,
    CaptionRegistry,
};
// NumberFormat is already imported from doc_model::field earlier in this file

/// Caption label types for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum CaptionLabelDto {
    Figure,
    Table,
    Equation,
    Custom { name: String },
}

impl From<CaptionLabelDto> for CaptionLabel {
    fn from(dto: CaptionLabelDto) -> Self {
        match dto {
            CaptionLabelDto::Figure => CaptionLabel::Figure,
            CaptionLabelDto::Table => CaptionLabel::Table,
            CaptionLabelDto::Equation => CaptionLabel::Equation,
            CaptionLabelDto::Custom { name } => CaptionLabel::Custom(name),
        }
    }
}

impl From<&CaptionLabel> for CaptionLabelDto {
    fn from(label: &CaptionLabel) -> Self {
        match label {
            CaptionLabel::Figure => CaptionLabelDto::Figure,
            CaptionLabel::Table => CaptionLabelDto::Table,
            CaptionLabel::Equation => CaptionLabelDto::Equation,
            CaptionLabel::Custom(name) => CaptionLabelDto::Custom { name: name.clone() },
        }
    }
}

/// Caption position for frontend
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptionPositionDto {
    Above,
    Below,
}

impl From<CaptionPositionDto> for CaptionPosition {
    fn from(dto: CaptionPositionDto) -> Self {
        match dto {
            CaptionPositionDto::Above => CaptionPosition::Above,
            CaptionPositionDto::Below => CaptionPosition::Below,
        }
    }
}

impl From<CaptionPosition> for CaptionPositionDto {
    fn from(pos: CaptionPosition) -> Self {
        match pos {
            CaptionPosition::Above => CaptionPositionDto::Above,
            CaptionPosition::Below => CaptionPositionDto::Below,
        }
    }
}

/// Number format for caption numbering
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NumberFormatDto {
    Arabic,
    LowercaseLetter,
    UppercaseLetter,
    LowercaseRoman,
    UppercaseRoman,
    Ordinal,
    CardinalText,
    OrdinalText,
}

impl From<NumberFormatDto> for NumberFormat {
    fn from(dto: NumberFormatDto) -> Self {
        match dto {
            NumberFormatDto::Arabic => NumberFormat::Arabic,
            NumberFormatDto::LowercaseLetter => NumberFormat::LowercaseLetter,
            NumberFormatDto::UppercaseLetter => NumberFormat::UppercaseLetter,
            NumberFormatDto::LowercaseRoman => NumberFormat::LowercaseRoman,
            NumberFormatDto::UppercaseRoman => NumberFormat::UppercaseRoman,
            NumberFormatDto::Ordinal => NumberFormat::Ordinal,
            NumberFormatDto::CardinalText => NumberFormat::CardinalText,
            NumberFormatDto::OrdinalText => NumberFormat::OrdinalText,
        }
    }
}

impl From<NumberFormat> for NumberFormatDto {
    fn from(fmt: NumberFormat) -> Self {
        match fmt {
            NumberFormat::Arabic => NumberFormatDto::Arabic,
            NumberFormat::LowercaseLetter => NumberFormatDto::LowercaseLetter,
            NumberFormat::UppercaseLetter => NumberFormatDto::UppercaseLetter,
            NumberFormat::LowercaseRoman => NumberFormatDto::LowercaseRoman,
            NumberFormat::UppercaseRoman => NumberFormatDto::UppercaseRoman,
            NumberFormat::Ordinal => NumberFormatDto::Ordinal,
            NumberFormat::CardinalText => NumberFormatDto::CardinalText,
            NumberFormat::OrdinalText => NumberFormatDto::OrdinalText,
        }
    }
}

/// Caption format DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptionFormatDto {
    /// The label type this format applies to
    pub label: CaptionLabelDto,
    /// Separator between number and text (e.g., ": " or " - ")
    pub separator: String,
    /// Number format for the sequence number
    pub number_format: NumberFormatDto,
    /// Whether to include chapter numbers (e.g., "Figure 2-1")
    pub include_chapter: bool,
    /// Chapter separator (e.g., "-" for "Figure 2-1")
    pub chapter_separator: String,
    /// Heading style used for chapter numbers
    pub chapter_style: Option<String>,
    /// Default position for this caption type
    pub default_position: CaptionPositionDto,
    /// Paragraph style ID for caption paragraphs
    pub paragraph_style: String,
}

impl From<&CaptionFormat> for CaptionFormatDto {
    fn from(format: &CaptionFormat) -> Self {
        Self {
            label: CaptionLabelDto::from(&format.label),
            separator: format.separator.clone(),
            number_format: NumberFormatDto::from(format.number_format),
            include_chapter: format.include_chapter,
            chapter_separator: format.chapter_separator.clone(),
            chapter_style: format.chapter_style.as_ref().map(|s| s.to_string()),
            default_position: CaptionPositionDto::from(format.default_position),
            paragraph_style: format.paragraph_style.to_string(),
        }
    }
}

impl From<CaptionFormatDto> for CaptionFormat {
    fn from(dto: CaptionFormatDto) -> Self {
        let mut format = CaptionFormat::new(CaptionLabel::from(dto.label));
        format.separator = dto.separator;
        format.number_format = NumberFormat::from(dto.number_format);
        format.include_chapter = dto.include_chapter;
        format.chapter_separator = dto.chapter_separator;
        format.chapter_style = dto.chapter_style.map(StyleId::new);
        format.default_position = CaptionPosition::from(dto.default_position);
        format.paragraph_style = StyleId::new(dto.paragraph_style);
        format
    }
}

/// Caption DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptionDto {
    /// Caption ID
    pub id: String,
    /// Label type
    pub label: CaptionLabelDto,
    /// Caption number (sequence number)
    pub number: u32,
    /// Formatted caption text (e.g., "Figure 1: Description")
    pub formatted_text: String,
    /// User's caption text (without label and number)
    pub text: String,
    /// Position relative to target
    pub position: CaptionPositionDto,
    /// Target object ID (if any)
    pub target_id: Option<String>,
    /// Paragraph ID containing the caption
    pub paragraph_id: String,
    /// Bookmark name for cross-referencing
    pub bookmark_name: String,
    /// Whether included in table of figures
    pub include_in_list: bool,
}

impl CaptionDto {
    fn from_caption(caption: &Caption, number: u32, format: &CaptionFormat) -> Self {
        let formatted_text = format!(
            "{} {}{}{}",
            caption.label.display_text(),
            format.format_number(number),
            format.separator,
            caption.text()
        );

        Self {
            id: caption.id().to_string(),
            label: CaptionLabelDto::from(&caption.label),
            number,
            formatted_text,
            text: caption.text().to_string(),
            position: CaptionPositionDto::from(caption.position),
            target_id: caption.target_id.map(|id| id.to_string()),
            paragraph_id: caption.paragraph_id.to_string(),
            bookmark_name: caption.bookmark_name().to_string(),
            include_in_list: caption.include_in_list,
        }
    }
}

/// Caption cross-reference display type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptionRefDisplayTypeDto {
    FullCaption,
    LabelAndNumber,
    LabelOnly,
    NumberOnly,
    PageNumber,
    RelativePosition,
    CaptionTextOnly,
}

impl From<CaptionRefDisplayTypeDto> for CaptionRefDisplayType {
    fn from(dto: CaptionRefDisplayTypeDto) -> Self {
        match dto {
            CaptionRefDisplayTypeDto::FullCaption => CaptionRefDisplayType::FullCaption,
            CaptionRefDisplayTypeDto::LabelAndNumber => CaptionRefDisplayType::LabelAndNumber,
            CaptionRefDisplayTypeDto::LabelOnly => CaptionRefDisplayType::LabelOnly,
            CaptionRefDisplayTypeDto::NumberOnly => CaptionRefDisplayType::NumberOnly,
            CaptionRefDisplayTypeDto::PageNumber => CaptionRefDisplayType::PageNumber,
            CaptionRefDisplayTypeDto::RelativePosition => CaptionRefDisplayType::RelativePosition,
            CaptionRefDisplayTypeDto::CaptionTextOnly => CaptionRefDisplayType::CaptionTextOnly,
        }
    }
}

/// Insert a caption for an object (image, table, equation)
///
/// Creates a caption paragraph with auto-numbering using SEQ fields.
#[tauri::command]
pub fn insert_caption(
    _doc_id: String,
    target_id: Option<String>,
    label: CaptionLabelDto,
    text: String,
    position: CaptionPositionDto,
) -> Result<CaptionDto, String> {
    // Convert DTOs to domain types
    let label = CaptionLabel::from(label);
    let position = CaptionPosition::from(position);
    let target_node_id = target_id
        .as_ref()
        .and_then(|s| doc_model::NodeId::from_string(s));

    // Build the caption
    let mut builder = CaptionBuilder::new(label.clone())
        .with_text(&text)
        .with_position(position);

    if let Some(tid) = target_node_id {
        builder = builder.with_target(tid);
    }

    let components = builder.build();
    let caption_id = components.caption.id();

    // Create a registry and format for generating the response
    let registry = CaptionRegistry::new();
    let format = registry
        .get_format(&label)
        .cloned()
        .unwrap_or_else(|| CaptionFormat::new(label.clone()));

    // For now, return with number 1 (in real implementation, would get from registry)
    let caption_dto = CaptionDto::from_caption(&components.caption, 1, &format);

    // TODO: Add caption to document state, insert paragraph at correct position
    // This would involve:
    // 1. Finding the target object's position in the document
    // 2. Inserting the caption paragraph above or below
    // 3. Registering the caption in CaptionRegistry
    // 4. Creating a bookmark for cross-referencing

    Ok(caption_dto)
}

/// Delete a caption
#[tauri::command]
pub fn delete_caption(_doc_id: String, caption_id: String) -> Result<(), String> {
    // Validate caption ID
    let _id = doc_model::NodeId::from_string(&caption_id)
        .ok_or_else(|| format!("Invalid caption ID: {}", caption_id))?;

    // TODO: Remove caption from registry, delete paragraph from document
    Ok(())
}

/// Edit caption text
#[tauri::command]
pub fn edit_caption_text(
    _doc_id: String,
    caption_id: String,
    text: String,
) -> Result<CaptionDto, String> {
    let _id = doc_model::NodeId::from_string(&caption_id)
        .ok_or_else(|| format!("Invalid caption ID: {}", caption_id))?;

    // TODO: Get caption from registry, update text, update paragraph content
    // For now, return a placeholder
    let label = CaptionLabel::Figure;
    let format = CaptionFormat::new(label.clone());

    Ok(CaptionDto {
        id: caption_id,
        label: CaptionLabelDto::Figure,
        number: 1,
        formatted_text: format!("Figure 1{}{}", format.separator, text),
        text,
        position: CaptionPositionDto::Below,
        target_id: None,
        paragraph_id: "placeholder".to_string(),
        bookmark_name: "placeholder".to_string(),
        include_in_list: true,
    })
}

/// Get a caption by ID
#[tauri::command]
pub fn get_caption(_doc_id: String, caption_id: String) -> Result<Option<CaptionDto>, String> {
    let _id = doc_model::NodeId::from_string(&caption_id)
        .ok_or_else(|| format!("Invalid caption ID: {}", caption_id))?;

    // TODO: Look up caption in document state
    Ok(None)
}

/// List all captions in the document
#[tauri::command]
pub fn list_captions(_doc_id: String) -> Result<Vec<CaptionDto>, String> {
    // TODO: Get all captions from document state's CaptionRegistry
    Ok(Vec::new())
}

/// List captions by label type (e.g., all figures)
#[tauri::command]
pub fn list_captions_by_label(
    _doc_id: String,
    label: CaptionLabelDto,
) -> Result<Vec<CaptionDto>, String> {
    let _label = CaptionLabel::from(label);

    // TODO: Filter captions by label from document state
    Ok(Vec::new())
}

/// Get the caption format for a label type
#[tauri::command]
pub fn get_caption_format(
    _doc_id: String,
    label: CaptionLabelDto,
) -> Result<CaptionFormatDto, String> {
    let label = CaptionLabel::from(label);

    // Return default format for the label type
    let format = CaptionFormat::new(label);
    Ok(CaptionFormatDto::from(&format))
}

/// Set the caption format for a label type
#[tauri::command]
pub fn set_caption_format(
    _doc_id: String,
    label: CaptionLabelDto,
    format: CaptionFormatDto,
) -> Result<(), String> {
    let _label = CaptionLabel::from(label);
    let _format = CaptionFormat::from(format);

    // TODO: Update format in document state's CaptionRegistry
    Ok(())
}

/// Update all caption numbers (refresh SEQ fields)
///
/// This should be called when captions are reordered or inserted/deleted.
#[tauri::command]
pub fn update_caption_numbers(_doc_id: String) -> Result<Vec<CaptionDto>, String> {
    // TODO: Iterate through all captions in document order,
    // update their SEQ field values, return updated captions
    Ok(Vec::new())
}

/// Get the caption for a specific target object (image, table)
#[tauri::command]
pub fn get_caption_for_target(
    _doc_id: String,
    target_id: String,
) -> Result<Option<CaptionDto>, String> {
    let _id = doc_model::NodeId::from_string(&target_id)
        .ok_or_else(|| format!("Invalid target ID: {}", target_id))?;

    // TODO: Look up caption by target ID in CaptionRegistry
    Ok(None)
}

/// Get available caption labels (built-in + custom)
#[tauri::command]
pub fn get_caption_labels(_doc_id: String) -> Result<Vec<CaptionLabelDto>, String> {
    // Return built-in labels
    // TODO: Add custom labels from document state
    Ok(vec![
        CaptionLabelDto::Figure,
        CaptionLabelDto::Table,
        CaptionLabelDto::Equation,
    ])
}

/// Add a custom caption label
#[tauri::command]
pub fn add_custom_caption_label(_doc_id: String, name: String) -> Result<CaptionLabelDto, String> {
    if name.is_empty() {
        return Err("Caption label name cannot be empty".to_string());
    }

    // TODO: Add custom label to document state
    Ok(CaptionLabelDto::Custom { name })
}

/// Get default caption formats for all label types
#[tauri::command]
pub fn get_default_caption_formats() -> Vec<CaptionFormatDto> {
    vec![
        CaptionFormatDto::from(&CaptionFormat::new(CaptionLabel::Figure)),
        CaptionFormatDto::from(&CaptionFormat::new(CaptionLabel::Table)),
        CaptionFormatDto::from(&CaptionFormat::new(CaptionLabel::Equation)),
    ]
}

/// Get supported number formats for captions
#[tauri::command]
pub fn get_caption_number_formats() -> Vec<NumberFormatInfoDto> {
    vec![
        NumberFormatInfoDto {
            id: NumberFormatDto::Arabic,
            name: "1, 2, 3".to_string(),
            example: "Figure 1".to_string(),
        },
        NumberFormatInfoDto {
            id: NumberFormatDto::LowercaseLetter,
            name: "a, b, c".to_string(),
            example: "Figure a".to_string(),
        },
        NumberFormatInfoDto {
            id: NumberFormatDto::UppercaseLetter,
            name: "A, B, C".to_string(),
            example: "Figure A".to_string(),
        },
        NumberFormatInfoDto {
            id: NumberFormatDto::LowercaseRoman,
            name: "i, ii, iii".to_string(),
            example: "Figure i".to_string(),
        },
        NumberFormatInfoDto {
            id: NumberFormatDto::UppercaseRoman,
            name: "I, II, III".to_string(),
            example: "Figure I".to_string(),
        },
    ]
}

/// Number format info for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumberFormatInfoDto {
    pub id: NumberFormatDto,
    pub name: String,
    pub example: String,
}

// =============================================================================
// Pagination Control Commands
// =============================================================================

/// Paragraph keep rules DTO for frontend communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphKeepRulesDto {
    /// Don't break between this paragraph and the next paragraph
    pub keep_with_next: bool,
    /// Don't break within this paragraph
    pub keep_together: bool,
    /// Always start this paragraph on a new page
    pub page_break_before: bool,
}

impl Default for ParagraphKeepRulesDto {
    fn default() -> Self {
        Self {
            keep_with_next: false,
            keep_together: false,
            page_break_before: false,
        }
    }
}

impl From<&doc_model::ParagraphKeepRules> for ParagraphKeepRulesDto {
    fn from(rules: &doc_model::ParagraphKeepRules) -> Self {
        Self {
            keep_with_next: rules.keep_with_next,
            keep_together: rules.keep_together,
            page_break_before: rules.page_break_before,
        }
    }
}

impl From<&ParagraphKeepRulesDto> for doc_model::ParagraphKeepRules {
    fn from(dto: &ParagraphKeepRulesDto) -> Self {
        Self {
            keep_with_next: dto.keep_with_next,
            keep_together: dto.keep_together,
            page_break_before: dto.page_break_before,
        }
    }
}

/// Widow/orphan control DTO for frontend communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidowOrphanControlDto {
    /// Whether widow/orphan control is enabled
    pub enabled: bool,
    /// Minimum number of lines that must appear at the top of a page (widow control)
    pub min_lines_top: u8,
    /// Minimum number of lines that must appear at the bottom of a page (orphan control)
    pub min_lines_bottom: u8,
}

impl Default for WidowOrphanControlDto {
    fn default() -> Self {
        Self {
            enabled: true,
            min_lines_top: 2,
            min_lines_bottom: 2,
        }
    }
}

impl From<&doc_model::WidowOrphanControl> for WidowOrphanControlDto {
    fn from(control: &doc_model::WidowOrphanControl) -> Self {
        Self {
            enabled: control.enabled,
            min_lines_top: control.min_lines_top,
            min_lines_bottom: control.min_lines_bottom,
        }
    }
}

impl From<&WidowOrphanControlDto> for doc_model::WidowOrphanControl {
    fn from(dto: &WidowOrphanControlDto) -> Self {
        Self {
            enabled: dto.enabled,
            min_lines_top: dto.min_lines_top.max(1),
            min_lines_bottom: dto.min_lines_bottom.max(1),
        }
    }
}

/// Line number restart mode DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LineNumberRestartDto {
    /// Restart numbering on each new page
    PerPage,
    /// Restart numbering at each new section
    PerSection,
    /// Continuous numbering throughout the document
    Continuous,
}

impl Default for LineNumberRestartDto {
    fn default() -> Self {
        Self::PerPage
    }
}

impl From<&doc_model::LineNumberRestart> for LineNumberRestartDto {
    fn from(restart: &doc_model::LineNumberRestart) -> Self {
        match restart {
            doc_model::LineNumberRestart::PerPage => Self::PerPage,
            doc_model::LineNumberRestart::PerSection => Self::PerSection,
            doc_model::LineNumberRestart::Continuous => Self::Continuous,
        }
    }
}

impl From<&LineNumberRestartDto> for doc_model::LineNumberRestart {
    fn from(dto: &LineNumberRestartDto) -> Self {
        match dto {
            LineNumberRestartDto::PerPage => Self::PerPage,
            LineNumberRestartDto::PerSection => Self::PerSection,
            LineNumberRestartDto::Continuous => Self::Continuous,
        }
    }
}

/// Line numbering DTO for frontend communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineNumberingDto {
    /// Whether line numbering is enabled
    pub enabled: bool,
    /// The starting line number
    pub start_at: u32,
    /// Show line number every Nth line (1 = every line)
    pub count_by: u32,
    /// When to restart line numbering
    pub restart: LineNumberRestartDto,
    /// Distance from the text to the line number in points
    pub distance_from_text: f32,
}

impl Default for LineNumberingDto {
    fn default() -> Self {
        Self {
            enabled: false,
            start_at: 1,
            count_by: 1,
            restart: LineNumberRestartDto::default(),
            distance_from_text: 18.0,
        }
    }
}

impl From<&doc_model::LineNumbering> for LineNumberingDto {
    fn from(ln: &doc_model::LineNumbering) -> Self {
        Self {
            enabled: ln.enabled,
            start_at: ln.start_at,
            count_by: ln.count_by,
            restart: LineNumberRestartDto::from(&ln.restart),
            distance_from_text: ln.distance_from_text,
        }
    }
}

impl From<&LineNumberingDto> for doc_model::LineNumbering {
    fn from(dto: &LineNumberingDto) -> Self {
        Self {
            enabled: dto.enabled,
            start_at: dto.start_at.max(1),
            count_by: dto.count_by.max(1),
            restart: doc_model::LineNumberRestart::from(&dto.restart),
            distance_from_text: dto.distance_from_text,
        }
    }
}

/// Set paragraph keep rules for a specific paragraph
///
/// Updates the paragraph's pagination properties using the edit engine's
/// SetParagraphPagination command. This affects how the paragraph behaves
/// during page breaks.
#[tauri::command]
pub fn set_paragraph_keep_rules(
    _doc_id: String,
    para_id: String,
    rules: ParagraphKeepRulesDto,
    state: State<'_, crate::state::AppState>,
) -> Result<DocumentChange, String> {
    let id = doc_model::NodeId::from_string(&para_id)
        .ok_or_else(|| format!("Invalid paragraph ID: {}", para_id))?;

    // Lock the document state
    let mut documents = state.documents.lock().map_err(|e| e.to_string())?;
    let doc_state = documents
        .get_mut(&_doc_id)
        .ok_or_else(|| format!("Document not found: {}", _doc_id))?;

    // Get the paragraph to verify it exists
    let para = doc_state.tree.get_paragraph(id)
        .ok_or_else(|| format!("Paragraph not found: {}", para_id))?;

    // Create a selection at the paragraph position for the command
    let selection = doc_model::Selection::collapsed(doc_model::Position::new(para.id(), 0));

    // Use the edit engine's SetParagraphPagination command
    let cmd = edit_engine::SetParagraphPagination::new(
        Some(rules.keep_with_next),
        Some(rules.keep_together),
        Some(rules.page_break_before),
        None, // widow_control is handled separately at document level
    );

    let result = cmd.apply(&doc_state.tree, &selection)
        .map_err(|e| e.to_string())?;

    // Update the document state with the result
    doc_state.tree = result.tree;
    doc_state.dirty = true;

    Ok(DocumentChange {
        changed_nodes: vec![para_id],
        dirty_pages: vec![],
        selection: None,
    })
}

/// Get paragraph keep rules for a specific paragraph
///
/// Retrieves the current pagination/keep rules from the paragraph's
/// direct formatting properties.
#[tauri::command]
pub fn get_paragraph_keep_rules(
    _doc_id: String,
    para_id: String,
    state: State<'_, crate::state::AppState>,
) -> Result<ParagraphKeepRulesDto, String> {
    let id = doc_model::NodeId::from_string(&para_id)
        .ok_or_else(|| format!("Invalid paragraph ID: {}", para_id))?;

    // Lock the document state
    let documents = state.documents.lock().map_err(|e| e.to_string())?;
    let doc_state = documents
        .get(&_doc_id)
        .ok_or_else(|| format!("Document not found: {}", _doc_id))?;

    // Get the paragraph
    let para = doc_state.tree.get_paragraph(id)
        .ok_or_else(|| format!("Paragraph not found: {}", para_id))?;

    // Extract keep rules from direct formatting
    Ok(ParagraphKeepRulesDto {
        keep_with_next: para.direct_formatting.keep_with_next.unwrap_or(false),
        keep_together: para.direct_formatting.keep_together.unwrap_or(false),
        page_break_before: para.direct_formatting.page_break_before.unwrap_or(false),
    })
}

/// Set widow/orphan control for the document
///
/// Updates the document-level widow/orphan control settings. These settings
/// determine how the layout engine handles paragraph breaks to prevent
/// single lines from appearing alone at the top (widows) or bottom (orphans)
/// of pages.
#[tauri::command]
pub fn set_widow_orphan_control(
    _doc_id: String,
    settings: WidowOrphanControlDto,
    state: State<'_, crate::state::AppState>,
) -> Result<DocumentChange, String> {
    let control = doc_model::WidowOrphanControl::from(&settings);

    // Lock the document state
    let mut documents = state.documents.lock().map_err(|e| e.to_string())?;
    let doc_state = documents
        .get_mut(&_doc_id)
        .ok_or_else(|| format!("Document not found: {}", _doc_id))?;

    // Update the document's pagination settings
    doc_state.pagination_settings.widow_orphan_control = control;
    doc_state.dirty = true;

    Ok(DocumentChange {
        changed_nodes: vec![],
        dirty_pages: vec![], // All pages may need re-layout
        selection: None,
    })
}

/// Get widow/orphan control settings for the document
///
/// Retrieves the current document-level widow/orphan control settings.
#[tauri::command]
pub fn get_widow_orphan_control(
    _doc_id: String,
    state: State<'_, crate::state::AppState>,
) -> Result<WidowOrphanControlDto, String> {
    // Lock the document state
    let documents = state.documents.lock().map_err(|e| e.to_string())?;
    let doc_state = documents
        .get(&_doc_id)
        .ok_or_else(|| format!("Document not found: {}", _doc_id))?;

    // Return the current widow/orphan control settings
    Ok(WidowOrphanControlDto::from(&doc_state.pagination_settings.widow_orphan_control))
}

/// Set line numbering for a specific section
///
/// Updates the line numbering settings for a section. Line numbering displays
/// numbers in the margin for each line of text, useful for legal documents,
/// code listings, or reference materials.
#[tauri::command]
pub fn set_line_numbering(
    _doc_id: String,
    section_id: String,
    settings: LineNumberingDto,
    state: State<'_, crate::state::AppState>,
) -> Result<DocumentChange, String> {
    let id = doc_model::NodeId::from_string(&section_id)
        .ok_or_else(|| format!("Invalid section ID: {}", section_id))?;

    let line_numbering = doc_model::LineNumbering::from(&settings);

    // Lock the document state
    let mut documents = state.documents.lock().map_err(|e| e.to_string())?;
    let doc_state = documents
        .get_mut(&_doc_id)
        .ok_or_else(|| format!("Document not found: {}", _doc_id))?;

    // Get the section and update its line numbering settings
    let section = doc_state.sections.get_mut(&id)
        .ok_or_else(|| format!("Section not found: {}", section_id))?;

    section.page_setup.line_numbering = line_numbering;
    doc_state.dirty = true;

    Ok(DocumentChange {
        changed_nodes: vec![section_id],
        dirty_pages: vec![], // Pages in this section need re-layout
        selection: None,
    })
}

/// Get line numbering settings for a specific section
///
/// Retrieves the current line numbering configuration for a section.
#[tauri::command]
pub fn get_line_numbering(
    _doc_id: String,
    section_id: String,
    state: State<'_, crate::state::AppState>,
) -> Result<LineNumberingDto, String> {
    let id = doc_model::NodeId::from_string(&section_id)
        .ok_or_else(|| format!("Invalid section ID: {}", section_id))?;

    // Lock the document state
    let documents = state.documents.lock().map_err(|e| e.to_string())?;
    let doc_state = documents
        .get(&_doc_id)
        .ok_or_else(|| format!("Document not found: {}", _doc_id))?;

    // Get the section's line numbering settings
    let section = doc_state.sections.get(&id)
        .ok_or_else(|| format!("Section not found: {}", section_id))?;

    Ok(LineNumberingDto::from(&section.page_setup.line_numbering))
}

/// Get all pagination settings for a paragraph (combined view)
#[tauri::command]
pub fn get_paragraph_pagination_info(
    _doc_id: String,
    para_id: String,
) -> Result<ParagraphPaginationInfoDto, String> {
    let _id = doc_model::NodeId::from_string(&para_id)
        .ok_or_else(|| format!("Invalid paragraph ID: {}", para_id))?;

    // TODO: Get all pagination info from document state
    Ok(ParagraphPaginationInfoDto {
        keep_rules: ParagraphKeepRulesDto::default(),
        widow_control_enabled: true,
    })
}

/// Combined pagination info for a paragraph
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphPaginationInfoDto {
    /// Paragraph keep rules
    pub keep_rules: ParagraphKeepRulesDto,
    /// Whether widow control applies to this paragraph
    pub widow_control_enabled: bool,
}

// =============================================================================
// Text Box Commands
// =============================================================================

/// Anchor type for text box positioning
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnchorTypeDto {
    Inline,
    Character,
    Paragraph,
    Page,
}

impl Default for AnchorTypeDto {
    fn default() -> Self {
        Self::Paragraph
    }
}

/// Horizontal position specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum HorizontalPositionDto {
    Absolute { offset: f32 },
    Relative { position: String }, // "left", "center", "right", "inside", "outside"
}

impl Default for HorizontalPositionDto {
    fn default() -> Self {
        Self::Absolute { offset: 72.0 }
    }
}

/// Vertical position specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum VerticalPositionDto {
    Absolute { offset: f32 },
    Relative { position: String }, // "top", "center", "bottom", "inside", "outside"
}

impl Default for VerticalPositionDto {
    fn default() -> Self {
        Self::Absolute { offset: 0.0 }
    }
}

/// Wrap mode for text around the text box
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WrapModeDto {
    None,
    Square,
    Tight,
    Through,
    TopAndBottom,
}

impl Default for WrapModeDto {
    fn default() -> Self {
        Self::Square
    }
}

/// Anchor configuration DTO
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnchorDto {
    pub anchor_type: AnchorTypeDto,
    pub horizontal: HorizontalPositionDto,
    pub vertical: VerticalPositionDto,
    pub wrap_mode: WrapModeDto,
    pub allow_overlap: bool,
}

/// Size specification DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SizeDto {
    /// Width in points (null for auto)
    pub width: Option<f32>,
    /// Height in points (null for auto)
    pub height: Option<f32>,
}

impl Default for SizeDto {
    fn default() -> Self {
        Self {
            width: Some(200.0),
            height: Some(100.0),
        }
    }
}

/// Border edge style DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BorderEdgeDto {
    pub width: f32,
    pub color: String,
    pub style: String, // "none", "solid", "dashed", "dotted", "double"
}

impl Default for BorderEdgeDto {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: "#000000".to_string(),
            style: "solid".to_string(),
        }
    }
}

/// Border style DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextBoxBorderStyleDto {
    pub top: BorderEdgeDto,
    pub right: BorderEdgeDto,
    pub bottom: BorderEdgeDto,
    pub left: BorderEdgeDto,
}

impl Default for TextBoxBorderStyleDto {
    fn default() -> Self {
        let edge = BorderEdgeDto::default();
        Self {
            top: edge.clone(),
            right: edge.clone(),
            bottom: edge.clone(),
            left: edge,
        }
    }
}

/// Fill style DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum FillStyleDto {
    None,
    Solid { color: String },
    Gradient { colors: Vec<(String, f32)>, angle: f32 },
}

impl Default for FillStyleDto {
    fn default() -> Self {
        Self::Solid { color: "#FFFFFF".to_string() }
    }
}

/// Internal margins DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginsDto {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Default for MarginsDto {
    fn default() -> Self {
        Self {
            top: 7.2,
            right: 7.2,
            bottom: 7.2,
            left: 7.2,
        }
    }
}

/// Vertical alignment DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VerticalAlignDto {
    Top,
    Center,
    Bottom,
}

impl Default for VerticalAlignDto {
    fn default() -> Self {
        Self::Top
    }
}

/// Text box style DTO
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextBoxStyleDto {
    pub border: Option<TextBoxBorderStyleDto>,
    pub fill: Option<FillStyleDto>,
    pub internal_margins: MarginsDto,
    pub vertical_align: VerticalAlignDto,
    pub rotation: f32,
    pub opacity: f32,
}

/// Text box DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextBoxDto {
    pub id: String,
    pub anchor: AnchorDto,
    pub size: SizeDto,
    pub style: TextBoxStyleDto,
    pub content: String,
    pub name: Option<String>,
    pub alt_text: Option<String>,
}

/// Text box edit mode state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextBoxEditModeDto {
    /// The text box currently being edited (null if not in edit mode)
    pub active_textbox_id: Option<String>,
    /// Whether we're in text box edit mode
    pub is_editing: bool,
}

/// Insert a text box at the current selection
#[tauri::command]
pub fn insert_text_box(
    _doc_id: String,
    anchor: AnchorDto,
    size: SizeDto,
    style: TextBoxStyleDto,
) -> Result<TextBoxDto, String> {
    // TODO: Implement with edit_engine
    // This would:
    // 1. Get the document from state
    // 2. Create an InsertTextBox command
    // 3. Apply it through the executor
    // 4. Return the created text box info

    Ok(TextBoxDto {
        id: "textbox_id".to_string(),
        anchor,
        size,
        style,
        content: String::new(),
        name: None,
        alt_text: None,
    })
}

/// Delete a text box
#[tauri::command]
pub fn delete_text_box(
    _doc_id: String,
    textbox_id: String,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&textbox_id)
        .ok_or_else(|| format!("Invalid text box ID: {}", textbox_id))?;

    // TODO: Implement with edit_engine
    Ok(DocumentChange::default())
}

/// Set text box content
#[tauri::command]
pub fn set_text_box_content(
    _doc_id: String,
    textbox_id: String,
    content: String,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&textbox_id)
        .ok_or_else(|| format!("Invalid text box ID: {}", textbox_id))?;

    // TODO: Implement with edit_engine
    let _ = content; // Will be used when implemented
    Ok(DocumentChange::default())
}

/// Set text box style
#[tauri::command]
pub fn set_text_box_style(
    _doc_id: String,
    textbox_id: String,
    style: TextBoxStyleDto,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&textbox_id)
        .ok_or_else(|| format!("Invalid text box ID: {}", textbox_id))?;

    // TODO: Implement with edit_engine
    let _ = style; // Will be used when implemented
    Ok(DocumentChange::default())
}

/// Set text box anchor/position
#[tauri::command]
pub fn set_text_box_anchor(
    _doc_id: String,
    textbox_id: String,
    anchor: AnchorDto,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&textbox_id)
        .ok_or_else(|| format!("Invalid text box ID: {}", textbox_id))?;

    // TODO: Implement with edit_engine
    let _ = anchor; // Will be used when implemented
    Ok(DocumentChange::default())
}

/// Resize a text box
#[tauri::command]
pub fn resize_text_box(
    _doc_id: String,
    textbox_id: String,
    size: SizeDto,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&textbox_id)
        .ok_or_else(|| format!("Invalid text box ID: {}", textbox_id))?;

    // TODO: Implement with edit_engine
    let _ = size; // Will be used when implemented
    Ok(DocumentChange::default())
}

/// Get a text box by ID
#[tauri::command]
pub fn get_text_box(
    _doc_id: String,
    textbox_id: String,
) -> Result<Option<TextBoxDto>, String> {
    let _id = doc_model::NodeId::from_string(&textbox_id)
        .ok_or_else(|| format!("Invalid text box ID: {}", textbox_id))?;

    // TODO: Get text box from document state
    Ok(None)
}

/// List all text boxes in the document
#[tauri::command]
pub fn list_text_boxes(
    _doc_id: String,
) -> Result<Vec<TextBoxDto>, String> {
    // TODO: Implement with document state
    Ok(Vec::new())
}

/// Enter text box edit mode (for editing content inside)
#[tauri::command]
pub fn enter_text_box_edit_mode(
    _doc_id: String,
    textbox_id: String,
) -> Result<TextBoxEditModeDto, String> {
    let _id = doc_model::NodeId::from_string(&textbox_id)
        .ok_or_else(|| format!("Invalid text box ID: {}", textbox_id))?;

    // TODO: Update document state to track active text box editing
    Ok(TextBoxEditModeDto {
        active_textbox_id: Some(textbox_id),
        is_editing: true,
    })
}

/// Exit text box edit mode
#[tauri::command]
pub fn exit_text_box_edit_mode(
    _doc_id: String,
) -> Result<TextBoxEditModeDto, String> {
    // TODO: Update document state to clear active text box editing
    Ok(TextBoxEditModeDto {
        active_textbox_id: None,
        is_editing: false,
    })
}

/// Get current text box edit mode state
#[tauri::command]
pub fn get_text_box_edit_mode(
    _doc_id: String,
) -> Result<TextBoxEditModeDto, String> {
    // TODO: Get from document state
    Ok(TextBoxEditModeDto::default())
}

// =============================================================================
// Footnote and Endnote Commands
// =============================================================================

use doc_model::footnote::{
    EndnotePosition, EndnoteProperties, FootnotePosition, FootnoteProperties, Note, NoteId,
    NoteStore, NoteType, NumberingScheme, RestartNumbering,
};

/// DTO for footnote/endnote information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteInfoDto {
    /// Note ID
    pub id: String,
    /// Note type ("footnote" or "endnote")
    pub note_type: String,
    /// Formatted mark (e.g., "1", "i", "*")
    pub mark: String,
    /// Preview of note content
    pub preview: Option<String>,
    /// Page number where reference appears
    pub reference_page: Option<usize>,
    /// Section ID if applicable
    pub section_id: Option<String>,
}

/// DTO for footnote properties
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FootnotePropertiesDto {
    /// Numbering scheme: "arabic", "lowerRoman", "upperRoman", "lowerLetter", "upperLetter", "symbols"
    pub numbering: String,
    /// Restart mode: "continuous", "perSection", "perPage"
    pub restart: String,
    /// Starting number
    pub start_at: u32,
    /// Position: "pageBottom" or "beneathText"
    pub position: String,
    /// Space before footnote area (points)
    pub space_before: f32,
    /// Show separator line
    pub show_separator: bool,
    /// Separator length as fraction (0.0 to 1.0)
    pub separator_length: f32,
    /// Separator weight (points)
    pub separator_weight: f32,
}

impl From<&FootnoteProperties> for FootnotePropertiesDto {
    fn from(props: &FootnoteProperties) -> Self {
        Self {
            numbering: match props.numbering {
                NumberingScheme::Arabic => "arabic",
                NumberingScheme::LowerRoman => "lowerRoman",
                NumberingScheme::UpperRoman => "upperRoman",
                NumberingScheme::LowerLetter => "lowerLetter",
                NumberingScheme::UpperLetter => "upperLetter",
                NumberingScheme::Symbols => "symbols",
            }
            .to_string(),
            restart: match props.restart {
                RestartNumbering::Continuous => "continuous",
                RestartNumbering::PerSection => "perSection",
                RestartNumbering::PerPage => "perPage",
            }
            .to_string(),
            start_at: props.start_at,
            position: match props.position {
                FootnotePosition::PageBottom => "pageBottom",
                FootnotePosition::BeneathText => "beneathText",
            }
            .to_string(),
            space_before: props.space_before,
            show_separator: props.show_separator,
            separator_length: props.separator_length,
            separator_weight: props.separator_weight,
        }
    }
}

impl From<FootnotePropertiesDto> for FootnoteProperties {
    fn from(dto: FootnotePropertiesDto) -> Self {
        Self {
            numbering: match dto.numbering.to_lowercase().as_str() {
                "lowerroman" => NumberingScheme::LowerRoman,
                "upperroman" => NumberingScheme::UpperRoman,
                "lowerletter" => NumberingScheme::LowerLetter,
                "upperletter" => NumberingScheme::UpperLetter,
                "symbols" => NumberingScheme::Symbols,
                _ => NumberingScheme::Arabic,
            },
            restart: match dto.restart.to_lowercase().as_str() {
                "persection" => RestartNumbering::PerSection,
                "perpage" => RestartNumbering::PerPage,
                _ => RestartNumbering::Continuous,
            },
            start_at: dto.start_at,
            position: match dto.position.to_lowercase().as_str() {
                "beneathtext" => FootnotePosition::BeneathText,
                _ => FootnotePosition::PageBottom,
            },
            space_before: dto.space_before,
            show_separator: dto.show_separator,
            separator_length: dto.separator_length,
            separator_weight: dto.separator_weight,
        }
    }
}

/// DTO for endnote properties
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndnotePropertiesDto {
    /// Numbering scheme: "arabic", "lowerRoman", "upperRoman", "lowerLetter", "upperLetter", "symbols"
    pub numbering: String,
    /// Restart mode: "continuous", "perSection"
    pub restart: String,
    /// Starting number
    pub start_at: u32,
    /// Position: "endOfSection" or "endOfDocument"
    pub position: String,
}

impl From<&EndnoteProperties> for EndnotePropertiesDto {
    fn from(props: &EndnoteProperties) -> Self {
        Self {
            numbering: match props.numbering {
                NumberingScheme::Arabic => "arabic",
                NumberingScheme::LowerRoman => "lowerRoman",
                NumberingScheme::UpperRoman => "upperRoman",
                NumberingScheme::LowerLetter => "lowerLetter",
                NumberingScheme::UpperLetter => "upperLetter",
                NumberingScheme::Symbols => "symbols",
            }
            .to_string(),
            restart: match props.restart {
                RestartNumbering::Continuous => "continuous",
                RestartNumbering::PerSection => "perSection",
                RestartNumbering::PerPage => "perSection", // PerPage not applicable to endnotes
            }
            .to_string(),
            start_at: props.start_at,
            position: match props.position {
                EndnotePosition::EndOfSection => "endOfSection",
                EndnotePosition::EndOfDocument => "endOfDocument",
            }
            .to_string(),
        }
    }
}

impl From<EndnotePropertiesDto> for EndnoteProperties {
    fn from(dto: EndnotePropertiesDto) -> Self {
        Self {
            numbering: match dto.numbering.to_lowercase().as_str() {
                "lowerroman" => NumberingScheme::LowerRoman,
                "upperroman" => NumberingScheme::UpperRoman,
                "lowerletter" => NumberingScheme::LowerLetter,
                "upperletter" => NumberingScheme::UpperLetter,
                "symbols" => NumberingScheme::Symbols,
                _ => NumberingScheme::Arabic,
            },
            restart: match dto.restart.to_lowercase().as_str() {
                "persection" => RestartNumbering::PerSection,
                _ => RestartNumbering::Continuous,
            },
            start_at: dto.start_at,
            position: match dto.position.to_lowercase().as_str() {
                "endofsection" => EndnotePosition::EndOfSection,
                _ => EndnotePosition::EndOfDocument,
            },
        }
    }
}

/// Insert position DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertPositionDto {
    /// Node ID where to insert
    pub node_id: String,
    /// Character offset within the node
    pub offset: usize,
}

/// Insert a footnote at the specified position
#[tauri::command]
pub fn insert_footnote(
    _doc_id: String,
    position: InsertPositionDto,
    content: Option<String>,
    section_id: Option<String>,
) -> Result<NoteInfoDto, String> {
    let _node_id = doc_model::NodeId::from_string(&position.node_id)
        .ok_or_else(|| format!("Invalid node ID: {}", position.node_id))?;

    // TODO: Implement with edit_engine InsertFootnote command
    // For now, return a placeholder response
    let note_id = NoteId::new();
    Ok(NoteInfoDto {
        id: note_id.to_string(),
        note_type: "footnote".to_string(),
        mark: "1".to_string(),
        preview: content,
        reference_page: Some(1),
        section_id,
    })
}

/// Insert an endnote at the specified position
#[tauri::command]
pub fn insert_endnote(
    _doc_id: String,
    position: InsertPositionDto,
    content: Option<String>,
    section_id: Option<String>,
) -> Result<NoteInfoDto, String> {
    let _node_id = doc_model::NodeId::from_string(&position.node_id)
        .ok_or_else(|| format!("Invalid node ID: {}", position.node_id))?;

    // TODO: Implement with edit_engine InsertEndnote command
    let note_id = NoteId::new();
    Ok(NoteInfoDto {
        id: note_id.to_string(),
        note_type: "endnote".to_string(),
        mark: "i".to_string(),
        preview: content,
        reference_page: Some(1),
        section_id,
    })
}

/// Delete a footnote or endnote
#[tauri::command]
pub fn delete_note(
    _doc_id: String,
    note_id: String,
    note_type: String,
) -> Result<DocumentChange, String> {
    let _id = NoteId::from_string(&note_id)
        .ok_or_else(|| format!("Invalid note ID: {}", note_id))?;

    let _note_type = match note_type.to_lowercase().as_str() {
        "endnote" => NoteType::Endnote,
        _ => NoteType::Footnote,
    };

    // TODO: Implement with edit_engine DeleteNote command
    Ok(DocumentChange::default())
}

/// Edit a note's content
#[tauri::command]
pub fn edit_note(
    _doc_id: String,
    note_id: String,
    note_type: String,
    content: String,
) -> Result<DocumentChange, String> {
    let _id = NoteId::from_string(&note_id)
        .ok_or_else(|| format!("Invalid note ID: {}", note_id))?;

    let _note_type = match note_type.to_lowercase().as_str() {
        "endnote" => NoteType::Endnote,
        _ => NoteType::Footnote,
    };

    // TODO: Implement with edit_engine EditNoteContent command
    let _ = content; // Will be used when implemented
    Ok(DocumentChange::default())
}

/// Get footnote properties for a section (or document default)
#[tauri::command]
pub fn get_footnote_properties(
    _doc_id: String,
    section_id: Option<String>,
) -> Result<FootnotePropertiesDto, String> {
    let _section_id = section_id.as_ref().and_then(|s| doc_model::NodeId::from_string(s));

    // TODO: Get from document state
    Ok(FootnotePropertiesDto::from(&FootnoteProperties::default()))
}

/// Set footnote properties for a section (or document default)
#[tauri::command]
pub fn set_footnote_properties(
    _doc_id: String,
    section_id: Option<String>,
    properties: FootnotePropertiesDto,
) -> Result<DocumentChange, String> {
    let _section_id = section_id.as_ref().and_then(|s| doc_model::NodeId::from_string(s));
    let _props = FootnoteProperties::from(properties);

    // TODO: Implement with edit_engine SetFootnoteProperties command
    Ok(DocumentChange::default())
}

/// Get endnote properties for a section (or document default)
#[tauri::command]
pub fn get_endnote_properties(
    _doc_id: String,
    section_id: Option<String>,
) -> Result<EndnotePropertiesDto, String> {
    let _section_id = section_id.as_ref().and_then(|s| doc_model::NodeId::from_string(s));

    // TODO: Get from document state
    Ok(EndnotePropertiesDto::from(&EndnoteProperties::default()))
}

/// Set endnote properties for a section (or document default)
#[tauri::command]
pub fn set_endnote_properties(
    _doc_id: String,
    section_id: Option<String>,
    properties: EndnotePropertiesDto,
) -> Result<DocumentChange, String> {
    let _section_id = section_id.as_ref().and_then(|s| doc_model::NodeId::from_string(s));
    let _props = EndnoteProperties::from(properties);

    // TODO: Implement with edit_engine SetEndnoteProperties command
    Ok(DocumentChange::default())
}

/// Navigate to a note (from reference to note content)
#[tauri::command]
pub fn navigate_to_note(
    _doc_id: String,
    note_id: String,
    note_type: String,
) -> Result<Selection, String> {
    let _id = NoteId::from_string(&note_id)
        .ok_or_else(|| format!("Invalid note ID: {}", note_id))?;

    let _note_type = match note_type.to_lowercase().as_str() {
        "endnote" => NoteType::Endnote,
        _ => NoteType::Footnote,
    };

    // TODO: Implement with edit_engine NavigateToNote command
    // Return a placeholder selection
    Ok(Selection {
        anchor: Position {
            node_id: "placeholder".to_string(),
            offset: 0,
        },
        focus: Position {
            node_id: "placeholder".to_string(),
            offset: 0,
        },
    })
}

/// Navigate to a note reference (from note content back to reference)
#[tauri::command]
pub fn navigate_to_note_ref(
    _doc_id: String,
    note_id: String,
    note_type: String,
) -> Result<Selection, String> {
    let _id = NoteId::from_string(&note_id)
        .ok_or_else(|| format!("Invalid note ID: {}", note_id))?;

    let _note_type = match note_type.to_lowercase().as_str() {
        "endnote" => NoteType::Endnote,
        _ => NoteType::Footnote,
    };

    // TODO: Implement with edit_engine NavigateToNoteRef command
    Ok(Selection {
        anchor: Position {
            node_id: "placeholder".to_string(),
            offset: 0,
        },
        focus: Position {
            node_id: "placeholder".to_string(),
            offset: 0,
        },
    })
}

/// Convert a footnote to an endnote (or vice versa)
#[tauri::command]
pub fn convert_footnote_to_endnote(
    _doc_id: String,
    note_id: String,
) -> Result<NoteInfoDto, String> {
    let _id = NoteId::from_string(&note_id)
        .ok_or_else(|| format!("Invalid note ID: {}", note_id))?;

    // TODO: Implement with edit_engine ConvertNote command
    let new_id = NoteId::new();
    Ok(NoteInfoDto {
        id: new_id.to_string(),
        note_type: "endnote".to_string(),
        mark: "i".to_string(),
        preview: None,
        reference_page: Some(1),
        section_id: None,
    })
}

/// Convert an endnote to a footnote
#[tauri::command]
pub fn convert_endnote_to_footnote(
    _doc_id: String,
    note_id: String,
) -> Result<NoteInfoDto, String> {
    let _id = NoteId::from_string(&note_id)
        .ok_or_else(|| format!("Invalid note ID: {}", note_id))?;

    // TODO: Implement with edit_engine ConvertNote command
    let new_id = NoteId::new();
    Ok(NoteInfoDto {
        id: new_id.to_string(),
        note_type: "footnote".to_string(),
        mark: "1".to_string(),
        preview: None,
        reference_page: Some(1),
        section_id: None,
    })
}

/// List all footnotes in the document
#[tauri::command]
pub fn list_footnotes(
    _doc_id: String,
) -> Result<Vec<NoteInfoDto>, String> {
    // TODO: Get from document state
    Ok(Vec::new())
}

/// List all endnotes in the document
#[tauri::command]
pub fn list_endnotes(
    _doc_id: String,
) -> Result<Vec<NoteInfoDto>, String> {
    // TODO: Get from document state
    Ok(Vec::new())
}

/// Get footnotes on a specific page
#[tauri::command]
pub fn get_footnotes_on_page(
    _doc_id: String,
    page_index: usize,
) -> Result<Vec<NoteInfoDto>, String> {
    // TODO: Get from document state/layout
    let _ = page_index; // Will be used when implemented
    Ok(Vec::new())
}

/// Get endnotes for a specific section (or all if section_id is None)
#[tauri::command]
pub fn get_endnotes_for_section(
    _doc_id: String,
    section_id: Option<String>,
) -> Result<Vec<NoteInfoDto>, String> {
    let _section_id = section_id.as_ref().and_then(|s| doc_model::NodeId::from_string(s));

    // TODO: Get from document state
    Ok(Vec::new())
}

/// Get footnote/endnote statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteStatisticsDto {
    /// Total number of footnotes
    pub footnote_count: usize,
    /// Total number of endnotes
    pub endnote_count: usize,
    /// Pages with footnotes
    pub pages_with_footnotes: Vec<usize>,
}

#[tauri::command]
pub fn get_note_statistics(
    _doc_id: String,
) -> Result<NoteStatisticsDto, String> {
    // TODO: Get from document state
    Ok(NoteStatisticsDto {
        footnote_count: 0,
        endnote_count: 0,
        pages_with_footnotes: Vec::new(),
    })
}

// =============================================================================
// Cross-Reference Commands
// =============================================================================

use doc_model::crossref::{
    AvailableTarget, BrokenReference, CrossRefDisplay, CrossRefRegistry, CrossRefType,
    CrossRefUpdater, CrossRefValidator, CrossReference, TargetDiscovery,
};

/// Cross-reference type DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CrossRefTypeDto {
    Heading,
    Bookmark,
    Footnote,
    Endnote,
    Equation,
    Figure,
    Table,
    CustomCaption,
}

impl From<CrossRefTypeDto> for CrossRefType {
    fn from(dto: CrossRefTypeDto) -> Self {
        match dto {
            CrossRefTypeDto::Heading => CrossRefType::Heading,
            CrossRefTypeDto::Bookmark => CrossRefType::Bookmark,
            CrossRefTypeDto::Footnote => CrossRefType::Footnote,
            CrossRefTypeDto::Endnote => CrossRefType::Endnote,
            CrossRefTypeDto::Equation => CrossRefType::Equation,
            CrossRefTypeDto::Figure => CrossRefType::Figure,
            CrossRefTypeDto::Table => CrossRefType::Table,
            CrossRefTypeDto::CustomCaption => CrossRefType::CustomCaption,
        }
    }
}

impl From<CrossRefType> for CrossRefTypeDto {
    fn from(ref_type: CrossRefType) -> Self {
        match ref_type {
            CrossRefType::Heading => CrossRefTypeDto::Heading,
            CrossRefType::Bookmark => CrossRefTypeDto::Bookmark,
            CrossRefType::Footnote => CrossRefTypeDto::Footnote,
            CrossRefType::Endnote => CrossRefTypeDto::Endnote,
            CrossRefType::Equation => CrossRefTypeDto::Equation,
            CrossRefType::Figure => CrossRefTypeDto::Figure,
            CrossRefType::Table => CrossRefTypeDto::Table,
            CrossRefType::CustomCaption => CrossRefTypeDto::CustomCaption,
        }
    }
}

/// Cross-reference display type DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CrossRefDisplayDto {
    Text,
    Number,
    PageNumber,
    AboveBelow,
    FullCaption,
    LabelAndNumber,
    ParagraphNumber,
    ParagraphNumberNoContext,
}

impl From<CrossRefDisplayDto> for CrossRefDisplay {
    fn from(dto: CrossRefDisplayDto) -> Self {
        match dto {
            CrossRefDisplayDto::Text => CrossRefDisplay::Text,
            CrossRefDisplayDto::Number => CrossRefDisplay::Number,
            CrossRefDisplayDto::PageNumber => CrossRefDisplay::PageNumber,
            CrossRefDisplayDto::AboveBelow => CrossRefDisplay::AboveBelow,
            CrossRefDisplayDto::FullCaption => CrossRefDisplay::FullCaption,
            CrossRefDisplayDto::LabelAndNumber => CrossRefDisplay::LabelAndNumber,
            CrossRefDisplayDto::ParagraphNumber => CrossRefDisplay::ParagraphNumber,
            CrossRefDisplayDto::ParagraphNumberNoContext => CrossRefDisplay::ParagraphNumberNoContext,
        }
    }
}

impl From<CrossRefDisplay> for CrossRefDisplayDto {
    fn from(display: CrossRefDisplay) -> Self {
        match display {
            CrossRefDisplay::Text => CrossRefDisplayDto::Text,
            CrossRefDisplay::Number => CrossRefDisplayDto::Number,
            CrossRefDisplay::PageNumber => CrossRefDisplayDto::PageNumber,
            CrossRefDisplay::AboveBelow => CrossRefDisplayDto::AboveBelow,
            CrossRefDisplay::FullCaption => CrossRefDisplayDto::FullCaption,
            CrossRefDisplay::LabelAndNumber => CrossRefDisplayDto::LabelAndNumber,
            CrossRefDisplay::ParagraphNumber => CrossRefDisplayDto::ParagraphNumber,
            CrossRefDisplay::ParagraphNumberNoContext => CrossRefDisplayDto::ParagraphNumberNoContext,
        }
    }
}

/// Cross-reference DTO for frontend communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossReferenceDto {
    /// Unique ID
    pub id: String,
    /// Type of reference
    pub ref_type: CrossRefTypeDto,
    /// Target identifier (bookmark name, heading ID, etc.)
    pub target_id: String,
    /// Display format
    pub display: CrossRefDisplayDto,
    /// Whether to include a hyperlink
    pub include_hyperlink: bool,
    /// Display text (cached)
    pub display_text: String,
    /// Whether the reference is broken
    pub is_broken: bool,
    /// Error message if broken
    pub error_message: Option<String>,
    /// Custom caption label (for CustomCaption type)
    pub custom_label: Option<String>,
}

impl CrossReferenceDto {
    /// Create a DTO from a CrossReference
    pub fn from_crossref(crossref: &CrossReference) -> Self {
        Self {
            id: crossref.id().to_string(),
            ref_type: CrossRefTypeDto::from(crossref.ref_type),
            target_id: crossref.target_id.clone(),
            display: CrossRefDisplayDto::from(crossref.display),
            include_hyperlink: crossref.include_hyperlink,
            display_text: crossref.display_text().to_string(),
            is_broken: crossref.is_broken,
            error_message: crossref.error_message.clone(),
            custom_label: crossref.custom_label.clone(),
        }
    }
}

/// Available target DTO for the cross-reference dialog
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableTargetDto {
    /// Target ID
    pub id: String,
    /// Display text
    pub display_text: String,
    /// Target type
    pub target_type: CrossRefTypeDto,
    /// Page number (if known)
    pub page_number: Option<u32>,
    /// Preview of what the reference will show
    pub preview: String,
}

impl From<&AvailableTarget> for AvailableTargetDto {
    fn from(target: &AvailableTarget) -> Self {
        Self {
            id: target.id.clone(),
            display_text: target.display_text.clone(),
            target_type: CrossRefTypeDto::from(target.target_type),
            page_number: target.page_number,
            preview: target.preview.clone(),
        }
    }
}

/// Broken reference DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokenReferenceDto {
    /// Reference ID
    pub ref_id: String,
    /// Reference type
    pub ref_type: CrossRefTypeDto,
    /// Target ID that wasn't found
    pub target_id: String,
    /// Error message
    pub error_message: String,
    /// Suggested alternative targets
    pub suggested_targets: Vec<String>,
}

impl From<&BrokenReference> for BrokenReferenceDto {
    fn from(broken: &BrokenReference) -> Self {
        Self {
            ref_id: broken.ref_id.to_string(),
            ref_type: CrossRefTypeDto::from(broken.ref_type),
            target_id: broken.target_id.clone(),
            error_message: broken.error_message.clone(),
            suggested_targets: broken.suggested_targets.clone(),
        }
    }
}

/// Display option info for the dialog
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayOptionDto {
    /// Display type
    pub display_type: CrossRefDisplayDto,
    /// Human-readable name
    pub name: String,
}

/// Insert a cross-reference at the current position
#[tauri::command]
pub fn insert_cross_reference(
    _doc_id: String,
    ref_type: CrossRefTypeDto,
    target_id: String,
    display: CrossRefDisplayDto,
    include_hyperlink: bool,
    custom_label: Option<String>,
) -> Result<CrossReferenceDto, String> {
    let ref_type = CrossRefType::from(ref_type);
    let display = CrossRefDisplay::from(display);

    // Create the cross-reference
    let mut crossref = match ref_type {
        CrossRefType::CustomCaption => {
            if let Some(label) = custom_label.clone() {
                CrossReference::custom_caption(label, &target_id)
            } else {
                return Err("Custom caption requires a label".to_string());
            }
        }
        _ => CrossReference::new(ref_type, &target_id),
    };

    crossref.display = display;
    crossref.include_hyperlink = include_hyperlink;

    // TODO: Add to document state and registry
    // - Insert the cross-reference field at the current selection
    // - Register in CrossRefRegistry
    // - Evaluate the field to get display text
    // - Create bookmark link if include_hyperlink is true

    // For now, set a placeholder display text
    let placeholder_text = match ref_type {
        CrossRefType::Heading => "[Heading]".to_string(),
        CrossRefType::Bookmark => format!("[{}]", target_id),
        CrossRefType::Footnote | CrossRefType::Endnote => "1".to_string(),
        CrossRefType::Figure => "Figure 1".to_string(),
        CrossRefType::Table => "Table 1".to_string(),
        CrossRefType::Equation => "Equation 1".to_string(),
        CrossRefType::CustomCaption => {
            format!("{} 1", custom_label.as_deref().unwrap_or("Item"))
        }
    };
    crossref.set_cached_text(&placeholder_text);

    Ok(CrossReferenceDto::from_crossref(&crossref))
}

/// Delete a cross-reference
#[tauri::command]
pub fn delete_cross_reference(
    _doc_id: String,
    ref_id: String,
) -> Result<(), String> {
    let _id = doc_model::NodeId::from_string(&ref_id)
        .ok_or_else(|| format!("Invalid cross-reference ID: {}", ref_id))?;

    // TODO: Remove from document state and registry
    // - Remove the field node from the document
    // - Remove from CrossRefRegistry

    Ok(())
}

/// Update a cross-reference's display format
#[tauri::command]
pub fn update_cross_reference(
    _doc_id: String,
    ref_id: String,
    display: CrossRefDisplayDto,
) -> Result<CrossReferenceDto, String> {
    let _id = doc_model::NodeId::from_string(&ref_id)
        .ok_or_else(|| format!("Invalid cross-reference ID: {}", ref_id))?;

    let display = CrossRefDisplay::from(display);

    // TODO: Update in document state
    // - Get the cross-reference from registry
    // - Update display format
    // - Re-evaluate to get new display text

    // For now, return a placeholder
    let mut crossref = CrossReference::heading("placeholder");
    crossref.display = display;
    crossref.set_cached_text("[Updated Reference]");

    Ok(CrossReferenceDto::from_crossref(&crossref))
}

/// Get a cross-reference by ID
#[tauri::command]
pub fn get_cross_reference(
    _doc_id: String,
    ref_id: String,
) -> Result<Option<CrossReferenceDto>, String> {
    let _id = doc_model::NodeId::from_string(&ref_id)
        .ok_or_else(|| format!("Invalid cross-reference ID: {}", ref_id))?;

    // TODO: Get from document state/registry
    Ok(None)
}

/// List all cross-references in the document
#[tauri::command]
pub fn list_cross_references(
    _doc_id: String,
) -> Result<Vec<CrossReferenceDto>, String> {
    // TODO: Get all cross-references from registry
    Ok(Vec::new())
}

/// Get available targets for a specific reference type
#[tauri::command]
pub fn get_available_targets(
    _doc_id: String,
    ref_type: CrossRefTypeDto,
) -> Result<Vec<AvailableTargetDto>, String> {
    let ref_type = CrossRefType::from(ref_type);

    // TODO: Get targets from document state using TargetDiscovery
    // For now, return sample data based on type

    let targets = match ref_type {
        CrossRefType::Heading => vec![
            AvailableTarget::new("_Heading_1", "Chapter 1: Introduction", CrossRefType::Heading)
                .with_page(1),
            AvailableTarget::new("_Heading_2", "Chapter 2: Background", CrossRefType::Heading)
                .with_page(5),
            AvailableTarget::new("_Heading_3", "Chapter 3: Methods", CrossRefType::Heading)
                .with_page(10),
        ],
        CrossRefType::Bookmark => vec![
            AvailableTarget::new("introduction", "introduction", CrossRefType::Bookmark).with_page(1),
            AvailableTarget::new("conclusion", "conclusion", CrossRefType::Bookmark).with_page(20),
        ],
        CrossRefType::Figure => vec![
            AvailableTarget::new("_RefFigure_1", "Figure 1: Sample Image", CrossRefType::Figure)
                .with_page(3)
                .with_preview("Figure 1"),
            AvailableTarget::new("_RefFigure_2", "Figure 2: Results Chart", CrossRefType::Figure)
                .with_page(12)
                .with_preview("Figure 2"),
        ],
        CrossRefType::Table => vec![
            AvailableTarget::new("_RefTable_1", "Table 1: Data Summary", CrossRefType::Table)
                .with_page(8)
                .with_preview("Table 1"),
        ],
        CrossRefType::Equation => vec![
            AvailableTarget::new("_RefEquation_1", "Equation 1", CrossRefType::Equation)
                .with_page(7)
                .with_preview("Equation 1"),
        ],
        CrossRefType::Footnote => vec![
            AvailableTarget::new("footnote_1", "Footnote 1", CrossRefType::Footnote)
                .with_preview("1"),
        ],
        CrossRefType::Endnote => vec![
            AvailableTarget::new("endnote_1", "Endnote i", CrossRefType::Endnote)
                .with_preview("i"),
        ],
        CrossRefType::CustomCaption => Vec::new(),
    };

    Ok(targets.iter().map(AvailableTargetDto::from).collect())
}

/// Navigate to a cross-reference target
#[tauri::command]
pub fn navigate_to_target(
    _doc_id: String,
    ref_id: String,
) -> Result<NavigationResultDto, String> {
    let _id = doc_model::NodeId::from_string(&ref_id)
        .ok_or_else(|| format!("Invalid cross-reference ID: {}", ref_id))?;

    // TODO: Get the cross-reference, find its target, and scroll to it
    // Return the target position for the frontend to scroll to

    Ok(NavigationResultDto {
        success: true,
        target_node_id: Some("target_para_id".to_string()),
        page_number: Some(1),
        scroll_position: Some(ScrollPositionDto { x: 0.0, y: 0.0 }),
        error_message: None,
    })
}

/// Navigation result DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationResultDto {
    /// Whether navigation was successful
    pub success: bool,
    /// Target node ID (if found)
    pub target_node_id: Option<String>,
    /// Page number of target
    pub page_number: Option<u32>,
    /// Scroll position
    pub scroll_position: Option<ScrollPositionDto>,
    /// Error message if navigation failed
    pub error_message: Option<String>,
}

/// Scroll position DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrollPositionDto {
    pub x: f64,
    pub y: f64,
}

/// Validate all cross-references in the document
#[tauri::command]
pub fn validate_cross_references(
    _doc_id: String,
) -> Result<CrossRefValidationResultDto, String> {
    // TODO: Use CrossRefValidator to validate all references
    // Return validation results

    Ok(CrossRefValidationResultDto {
        total_references: 0,
        valid_count: 0,
        broken_count: 0,
        broken_references: Vec::new(),
    })
}

/// Cross-reference validation result DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossRefValidationResultDto {
    /// Total number of cross-references
    pub total_references: usize,
    /// Number of valid references
    pub valid_count: usize,
    /// Number of broken references
    pub broken_count: usize,
    /// Details of broken references
    pub broken_references: Vec<BrokenReferenceDto>,
}

/// Update all cross-references in the document
#[tauri::command]
pub fn update_all_cross_references(
    _doc_id: String,
) -> Result<CrossRefUpdateResultDto, String> {
    // TODO: Use CrossRefUpdater to refresh all reference text

    Ok(CrossRefUpdateResultDto {
        updated_count: 0,
        failed_count: 0,
        failed_refs: Vec::new(),
    })
}

/// Cross-reference update result DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossRefUpdateResultDto {
    /// Number of successfully updated references
    pub updated_count: usize,
    /// Number of failed updates
    pub failed_count: usize,
    /// IDs of references that failed to update
    pub failed_refs: Vec<String>,
}

/// Get all broken cross-references
#[tauri::command]
pub fn get_broken_references(
    _doc_id: String,
) -> Result<Vec<BrokenReferenceDto>, String> {
    // TODO: Get broken references from registry
    Ok(Vec::new())
}

/// Get available display options for a reference type
#[tauri::command]
pub fn get_display_options_for_type(
    ref_type: CrossRefTypeDto,
) -> Result<Vec<DisplayOptionDto>, String> {
    let ref_type = CrossRefType::from(ref_type);
    let options = CrossRefDisplay::available_for_type(ref_type);

    Ok(options
        .into_iter()
        .map(|d| DisplayOptionDto {
            display_type: CrossRefDisplayDto::from(d),
            name: d.display_name().to_string(),
        })
        .collect())
}

/// Preview what a cross-reference will display
#[tauri::command]
pub fn preview_cross_reference(
    _doc_id: String,
    ref_type: CrossRefTypeDto,
    target_id: String,
    display: CrossRefDisplayDto,
) -> Result<String, String> {
    let ref_type = CrossRefType::from(ref_type);
    let display = CrossRefDisplay::from(display);

    // TODO: Generate preview text based on actual document content
    // For now, return placeholder text

    let preview = match (ref_type, display) {
        (CrossRefType::Heading, CrossRefDisplay::Text) => "[Heading Text]".to_string(),
        (CrossRefType::Heading, CrossRefDisplay::PageNumber) => "5".to_string(),
        (CrossRefType::Figure, CrossRefDisplay::FullCaption) => "Figure 1: Sample caption".to_string(),
        (CrossRefType::Figure, CrossRefDisplay::LabelAndNumber) => "Figure 1".to_string(),
        (CrossRefType::Figure, CrossRefDisplay::Number) => "1".to_string(),
        (CrossRefType::Table, CrossRefDisplay::FullCaption) => "Table 1: Data summary".to_string(),
        (CrossRefType::Table, CrossRefDisplay::LabelAndNumber) => "Table 1".to_string(),
        (CrossRefType::Footnote, CrossRefDisplay::Number) => "1".to_string(),
        (CrossRefType::Endnote, CrossRefDisplay::Number) => "i".to_string(),
        (_, CrossRefDisplay::AboveBelow) => "above".to_string(),
        (_, CrossRefDisplay::PageNumber) => "1".to_string(),
        _ => format!("[{}]", target_id),
    };

    Ok(preview)
}

/// Get cross-references by type
#[tauri::command]
pub fn list_cross_references_by_type(
    _doc_id: String,
    ref_type: CrossRefTypeDto,
) -> Result<Vec<CrossReferenceDto>, String> {
    let _ref_type = CrossRefType::from(ref_type);

    // TODO: Get from registry filtered by type
    Ok(Vec::new())
}

/// Get cross-references that target a specific element
#[tauri::command]
pub fn get_references_to_target(
    _doc_id: String,
    target_id: String,
) -> Result<Vec<CrossReferenceDto>, String> {
    // TODO: Get from registry filtered by target
    let _ = target_id;
    Ok(Vec::new())
}

/// Cross-reference statistics DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossRefStatisticsDto {
    /// Total number of cross-references
    pub total_count: usize,
    /// Number of heading references
    pub heading_count: usize,
    /// Number of bookmark references
    pub bookmark_count: usize,
    /// Number of figure references
    pub figure_count: usize,
    /// Number of table references
    pub table_count: usize,
    /// Number of equation references
    pub equation_count: usize,
    /// Number of footnote references
    pub footnote_count: usize,
    /// Number of endnote references
    pub endnote_count: usize,
    /// Number of broken references
    pub broken_count: usize,
}

/// Get cross-reference statistics
#[tauri::command]
pub fn get_cross_reference_statistics(
    _doc_id: String,
) -> Result<CrossRefStatisticsDto, String> {
    // TODO: Calculate from registry
    Ok(CrossRefStatisticsDto {
        total_count: 0,
        heading_count: 0,
        bookmark_count: 0,
        figure_count: 0,
        table_count: 0,
        equation_count: 0,
        footnote_count: 0,
        endnote_count: 0,
        broken_count: 0,
    })
}

// =============================================================================
// Advanced Shapes Commands
// =============================================================================

use doc_model::shape::{
    ArrowConfig, ArrowHead, ArrowSize, BevelEffect, BevelType, CalloutLineType,
    CalloutTextPosition, ConnectionPoint, Connector, ConnectorEndpoint, ConnectorRouting,
    DashStyle, DistributeDirection, Effect3D, GlowEffect, GradientStop, HorizontalAlignment,
    LightDirection, LightingType, LineCap, LineJoin, MaterialType, PatternType,
    PictureStretchMode, Point, Rect as ShapeRect, ReflectionEffect, ShapeCategory, ShapeColor,
    ShapeEffects, ShapeFill, ShapeGroup, ShapeNode, ShapeProperties, ShapeStroke, ShapeText,
    ShapeTextDirection, ShapeTextMargins, ShapeTextVerticalAlign, ShapeType, ShadowEffect,
    ShadowPreset, ShadowType, SoftEdgeEffect, TextAnchor, TextAutoFit,
    VerticalAlignment as ShapeVerticalAlignment, ZOrderOperation,
};

// -----------------------------------------------------------------------------
// Shape Type DTOs
// -----------------------------------------------------------------------------

/// Shape type DTO for frontend communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ShapeTypeDto {
    // Basic shapes
    Rectangle,
    RoundedRectangle { corner_radius: f32 },
    Oval,
    Line,
    Arrow,
    DoubleArrow,
    Triangle,
    Diamond,
    Pentagon,
    Hexagon,
    Star { points: u8, inner_radius_ratio: f32 },
    TextBox,

    // Block arrows
    RightArrow { head_width: f32, head_length: f32 },
    LeftArrow { head_width: f32, head_length: f32 },
    UpArrow { head_width: f32, head_length: f32 },
    DownArrow { head_width: f32, head_length: f32 },
    LeftRightArrow { head_width: f32, head_length: f32 },
    UpDownArrow { head_width: f32, head_length: f32 },
    QuadArrow { head_width: f32, head_length: f32 },
    BentArrow { bend_position: f32, head_width: f32, head_length: f32 },
    UTurnArrow { head_width: f32, head_length: f32 },
    ChevronArrow { thickness: f32 },

    // Flowchart shapes
    FlowchartProcess,
    FlowchartDecision,
    FlowchartData,
    FlowchartTerminator,
    FlowchartDocument,
    FlowchartPredefined,
    FlowchartManualInput,
    FlowchartPreparation,
    FlowchartConnector,
    FlowchartOffPageConnector,
    FlowchartDelay,

    // Callouts
    RectangularCallout { tail_anchor: (f32, f32), tail_tip: (f32, f32), tail_width: f32 },
    RoundedCallout { corner_radius: f32, tail_anchor: (f32, f32), tail_tip: (f32, f32), tail_width: f32 },
    OvalCallout { tail_anchor: (f32, f32), tail_tip: (f32, f32), tail_width: f32 },
    CloudCallout { tail_tip: (f32, f32), bubble_count: u8 },
    LineCallout { accent_bar: bool },

    // Stars and banners
    Star4,
    Star5,
    Star6,
    Star8,
    Star10,
    Star12,
    Ribbon { tail_length: f32, tails_up: bool },
    Wave { amplitude: f32, periods: f32 },
    DoubleWave { amplitude: f32, periods: f32 },
    HorizontalScroll { roll_size: f32 },
    VerticalScroll { roll_size: f32 },

    // Additional shapes
    Parallelogram { slant: f32 },
    Trapezoid { top_ratio: f32 },
    Octagon,
    Cross { thickness: f32 },
    Donut { inner_radius: f32 },
    Heart,
    LightningBolt,
    Cloud,

    // Equation shapes
    MathPlus,
    MathMinus,
    MathMultiply,
    MathDivide,
    MathEqual,

    // Custom
    CustomPath { path_data: String },
    Freeform { points: Vec<(f32, f32)>, closed: bool },
}

impl Default for ShapeTypeDto {
    fn default() -> Self {
        Self::Rectangle
    }
}

// -----------------------------------------------------------------------------
// Color and Fill DTOs
// -----------------------------------------------------------------------------

/// Shape color DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeColorDto {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for ShapeColorDto {
    fn default() -> Self {
        Self { r: 0, g: 0, b: 0, a: 255 }
    }
}

impl From<ShapeColor> for ShapeColorDto {
    fn from(c: ShapeColor) -> Self {
        Self { r: c.r, g: c.g, b: c.b, a: c.a }
    }
}

impl From<ShapeColorDto> for ShapeColor {
    fn from(dto: ShapeColorDto) -> Self {
        ShapeColor::rgba(dto.r, dto.g, dto.b, dto.a)
    }
}

/// Gradient stop DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GradientStopDto {
    pub color: ShapeColorDto,
    pub position: f32,
    pub transparency: f32,
}

/// Shape fill DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ShapeFillDto {
    None,
    Solid { color: ShapeColorDto },
    LinearGradient { angle: f32, stops: Vec<GradientStopDto>, rotate_with_shape: bool },
    RadialGradient { center_x: f32, center_y: f32, stops: Vec<GradientStopDto> },
    Pattern { pattern: String, foreground: ShapeColorDto, background: ShapeColorDto },
    Picture { image_id: String, stretch_mode: String, transparency: f32 },
}

impl Default for ShapeFillDto {
    fn default() -> Self {
        Self::Solid { color: ShapeColorDto { r: 68, g: 114, b: 196, a: 255 } }
    }
}

// -----------------------------------------------------------------------------
// Stroke DTO
// -----------------------------------------------------------------------------

/// Shape stroke DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeStrokeDto {
    pub color: ShapeColorDto,
    pub width: f32,
    pub dash_style: String, // "solid", "dash", "dot", "dashDot", "dashDotDot"
    pub cap: String, // "flat", "round", "square"
    pub join: String, // "miter", "round", "bevel"
}

impl Default for ShapeStrokeDto {
    fn default() -> Self {
        Self {
            color: ShapeColorDto::default(),
            width: 1.0,
            dash_style: "solid".to_string(),
            cap: "flat".to_string(),
            join: "round".to_string(),
        }
    }
}

// -----------------------------------------------------------------------------
// Effect DTOs
// -----------------------------------------------------------------------------

/// Shadow effect DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShadowEffectDto {
    pub enabled: bool,
    pub color: ShapeColorDto,
    pub shadow_type: String, // "outer", "inner", "perspective"
    pub blur: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub opacity: f32,
    pub distance: f32,
    pub angle: f32,
    pub preset: String,
}

impl Default for ShadowEffectDto {
    fn default() -> Self {
        Self {
            enabled: true,
            color: ShapeColorDto { r: 0, g: 0, b: 0, a: 128 },
            shadow_type: "outer".to_string(),
            blur: 4.0,
            offset_x: 2.0,
            offset_y: 2.0,
            opacity: 0.5,
            distance: 3.0,
            angle: 45.0,
            preset: "offsetDiagonal".to_string(),
        }
    }
}

/// 3D effect DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Effect3DDto {
    pub enabled: bool,
    pub depth: f32,
    pub extrusion_color: ShapeColorDto,
    pub lighting: String,
    pub light_direction: String,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    pub bevel_type: String,
    pub bevel_width: f32,
    pub bevel_height: f32,
}

impl Default for Effect3DDto {
    fn default() -> Self {
        Self {
            enabled: false,
            depth: 0.0,
            extrusion_color: ShapeColorDto { r: 128, g: 128, b: 128, a: 255 },
            lighting: "none".to_string(),
            light_direction: "topLeft".to_string(),
            rotation_x: 0.0,
            rotation_y: 0.0,
            rotation_z: 0.0,
            bevel_type: "none".to_string(),
            bevel_width: 6.0,
            bevel_height: 6.0,
        }
    }
}

/// Glow effect DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlowEffectDto {
    pub enabled: bool,
    pub color: ShapeColorDto,
    pub radius: f32,
    pub opacity: f32,
}

/// Reflection effect DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReflectionEffectDto {
    pub enabled: bool,
    pub blur: f32,
    pub start_opacity: f32,
    pub end_opacity: f32,
    pub offset: f32,
}

/// Shape effects DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeEffectsDto {
    pub shadow: Option<ShadowEffectDto>,
    pub effect_3d: Option<Effect3DDto>,
    pub glow: Option<GlowEffectDto>,
    pub reflection: Option<ReflectionEffectDto>,
    pub opacity: f32,
}

impl Default for ShapeEffectsDto {
    fn default() -> Self {
        Self {
            shadow: None,
            effect_3d: None,
            glow: None,
            reflection: None,
            opacity: 1.0,
        }
    }
}

// -----------------------------------------------------------------------------
// Shape Text DTO
// -----------------------------------------------------------------------------

/// Shape text margins DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeTextMarginsDto {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Default for ShapeTextMarginsDto {
    fn default() -> Self {
        Self { top: 3.6, right: 7.2, bottom: 3.6, left: 7.2 }
    }
}

/// Shape text DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeTextDto {
    pub content: Vec<String>, // Paragraph IDs
    pub anchor: String, // "top", "middle", "bottom"
    pub auto_fit: String, // "none", "shrinkText", "resizeShape"
    pub margins: ShapeTextMarginsDto,
    pub vertical_align: String, // "top", "center", "bottom", "justify"
    pub direction: String, // "horizontal", "vertical", etc.
    pub rotation: f32,
    pub columns: u8,
    pub column_spacing: f32,
}

impl Default for ShapeTextDto {
    fn default() -> Self {
        Self {
            content: Vec::new(),
            anchor: "middle".to_string(),
            auto_fit: "none".to_string(),
            margins: ShapeTextMarginsDto::default(),
            vertical_align: "center".to_string(),
            direction: "horizontal".to_string(),
            rotation: 0.0,
            columns: 1,
            column_spacing: 0.0,
        }
    }
}

// -----------------------------------------------------------------------------
// Shape Properties DTO
// -----------------------------------------------------------------------------

/// Shape properties DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapePropertiesDto {
    pub width: f32,
    pub height: f32,
    pub wrap_type: String, // "inline", "square", "tight", "behind", "inFront"
    pub rotation: f32,
    pub fill: Option<ShapeFillDto>,
    pub stroke: Option<ShapeStrokeDto>,
    pub effects: ShapeEffectsDto,
    pub lock_aspect_ratio: bool,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

impl Default for ShapePropertiesDto {
    fn default() -> Self {
        Self {
            width: 100.0,
            height: 100.0,
            wrap_type: "inFront".to_string(),
            rotation: 0.0,
            fill: Some(ShapeFillDto::default()),
            stroke: Some(ShapeStrokeDto::default()),
            effects: ShapeEffectsDto::default(),
            lock_aspect_ratio: false,
            flip_horizontal: false,
            flip_vertical: false,
        }
    }
}

// -----------------------------------------------------------------------------
// Shape DTO
// -----------------------------------------------------------------------------

/// Complete shape DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeDto {
    pub id: String,
    pub shape_type: ShapeTypeDto,
    pub properties: ShapePropertiesDto,
    pub shape_text: Option<ShapeTextDto>,
    pub name: Option<String>,
    pub alt_text: Option<String>,
    pub group_id: Option<String>,
    pub z_order: i32,
    pub locked: bool,
    pub hidden: bool,
}

// -----------------------------------------------------------------------------
// Shape Group DTO
// -----------------------------------------------------------------------------

/// Shape group DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeGroupDto {
    pub id: String,
    pub shapes: Vec<String>, // Shape IDs
    pub bounds: RectDto,
    pub name: Option<String>,
    pub locked: bool,
}

/// Rectangle DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RectDto {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for RectDto {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, width: 100.0, height: 100.0 }
    }
}

// -----------------------------------------------------------------------------
// Connector DTOs
// -----------------------------------------------------------------------------

/// Connection point DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionPointDto {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
    Custom { x: f32, y: f32 },
}

impl Default for ConnectionPointDto {
    fn default() -> Self {
        Self::Right
    }
}

/// Connector endpoint DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ConnectorEndpointDto {
    ShapeConnection { shape_id: String, point: ConnectionPointDto },
    ShapeCustom { shape_id: String, position_x: f32, position_y: f32 },
    Floating { x: f32, y: f32 },
}

impl Default for ConnectorEndpointDto {
    fn default() -> Self {
        Self::Floating { x: 0.0, y: 0.0 }
    }
}

/// Arrow config DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArrowConfigDto {
    pub start_type: String, // "none", "triangle", "stealth", "diamond", "oval", "open"
    pub start_size: String, // "small", "medium", "large"
    pub end_type: String,
    pub end_size: String,
}

impl Default for ArrowConfigDto {
    fn default() -> Self {
        Self {
            start_type: "none".to_string(),
            start_size: "medium".to_string(),
            end_type: "triangle".to_string(),
            end_size: "medium".to_string(),
        }
    }
}

/// Connector DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorDto {
    pub id: String,
    pub start: ConnectorEndpointDto,
    pub end: ConnectorEndpointDto,
    pub routing: String, // "straight", "elbow", "curved"
    pub line_style: ShapeStrokeDto,
    pub arrows: ArrowConfigDto,
    pub adjustments: Vec<f32>,
    pub name: Option<String>,
}

// -----------------------------------------------------------------------------
// Alignment/Distribution DTOs
// -----------------------------------------------------------------------------

/// Alignment reference DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AlignmentReferenceDto {
    Selection,
    Page,
    Margin,
}

impl Default for AlignmentReferenceDto {
    fn default() -> Self {
        Self::Selection
    }
}

// =============================================================================
// Shape CRUD Commands
// =============================================================================

/// Insert a new shape at the specified position
#[tauri::command]
pub fn insert_shape(
    _doc_id: String,
    shape_type: ShapeTypeDto,
    properties: ShapePropertiesDto,
    position_x: f32,
    position_y: f32,
) -> Result<ShapeDto, String> {
    // TODO: Implement with edit_engine
    // This would:
    // 1. Create a ShapeNode from the DTO
    // 2. Apply InsertShape command
    // 3. Return the created shape info

    let _x = position_x;
    let _y = position_y;

    Ok(ShapeDto {
        id: "shape_id".to_string(),
        shape_type,
        properties,
        shape_text: None,
        name: None,
        alt_text: None,
        group_id: None,
        z_order: 0,
        locked: false,
        hidden: false,
    })
}

/// Delete a shape
#[tauri::command]
pub fn delete_shape(
    _doc_id: String,
    shape_id: String,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Implement with edit_engine
    Ok(DocumentChange::default())
}

/// Get a shape by ID
#[tauri::command]
pub fn get_shape(
    _doc_id: String,
    shape_id: String,
) -> Result<Option<ShapeDto>, String> {
    let _id = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Get shape from document state
    Ok(None)
}

/// List all shapes in the document
#[tauri::command]
pub fn list_shapes(
    _doc_id: String,
) -> Result<Vec<ShapeDto>, String> {
    // TODO: Implement with document state
    Ok(Vec::new())
}

/// Update shape properties
#[tauri::command]
pub fn update_shape_properties(
    _doc_id: String,
    shape_id: String,
    properties: ShapePropertiesDto,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Implement with edit_engine
    let _ = properties;
    Ok(DocumentChange::default())
}

// =============================================================================
// Shape Text Commands
// =============================================================================

/// Set text content for a shape
#[tauri::command]
pub fn set_shape_text(
    _doc_id: String,
    shape_id: String,
    text: String,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Implement with edit_engine
    let _ = text;
    Ok(DocumentChange::default())
}

/// Get text content from a shape
#[tauri::command]
pub fn get_shape_text(
    _doc_id: String,
    shape_id: String,
) -> Result<Option<ShapeTextDto>, String> {
    let _id = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Get from document state
    Ok(None)
}

/// Set text auto-fit mode for a shape
#[tauri::command]
pub fn set_text_auto_fit(
    _doc_id: String,
    shape_id: String,
    auto_fit: String, // "none", "shrinkText", "resizeShape"
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Implement with edit_engine
    let _ = auto_fit;
    Ok(DocumentChange::default())
}

// =============================================================================
// Shape Styling Commands
// =============================================================================

/// Set shape fill
#[tauri::command]
pub fn set_shape_fill(
    _doc_id: String,
    shape_id: String,
    fill: Option<ShapeFillDto>,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Implement with edit_engine
    let _ = fill;
    Ok(DocumentChange::default())
}

/// Set shape line/stroke
#[tauri::command]
pub fn set_shape_line(
    _doc_id: String,
    shape_id: String,
    stroke: Option<ShapeStrokeDto>,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Implement with edit_engine
    let _ = stroke;
    Ok(DocumentChange::default())
}

/// Set shape effects
#[tauri::command]
pub fn set_shape_effects(
    _doc_id: String,
    shape_id: String,
    effects: ShapeEffectsDto,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Implement with edit_engine
    let _ = effects;
    Ok(DocumentChange::default())
}

// =============================================================================
// Shape Group Commands
// =============================================================================

/// Group selected shapes
#[tauri::command]
pub fn group_shapes(
    _doc_id: String,
    shape_ids: Vec<String>,
) -> Result<ShapeGroupDto, String> {
    // Validate all shape IDs
    for id in &shape_ids {
        doc_model::NodeId::from_string(id)
            .ok_or_else(|| format!("Invalid shape ID: {}", id))?;
    }

    // TODO: Implement with edit_engine
    Ok(ShapeGroupDto {
        id: "group_id".to_string(),
        shapes: shape_ids,
        bounds: RectDto::default(),
        name: None,
        locked: false,
    })
}

/// Ungroup a shape group
#[tauri::command]
pub fn ungroup_shapes(
    _doc_id: String,
    group_id: String,
) -> Result<Vec<String>, String> {
    let _id = doc_model::NodeId::from_string(&group_id)
        .ok_or_else(|| format!("Invalid group ID: {}", group_id))?;

    // TODO: Implement with edit_engine
    // Returns the IDs of the shapes that were in the group
    Ok(Vec::new())
}

/// Add a shape to an existing group
#[tauri::command]
pub fn add_to_group(
    _doc_id: String,
    group_id: String,
    shape_id: String,
) -> Result<DocumentChange, String> {
    let _gid = doc_model::NodeId::from_string(&group_id)
        .ok_or_else(|| format!("Invalid group ID: {}", group_id))?;
    let _sid = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Implement with edit_engine
    Ok(DocumentChange::default())
}

/// Remove a shape from a group
#[tauri::command]
pub fn remove_from_group(
    _doc_id: String,
    group_id: String,
    shape_id: String,
) -> Result<DocumentChange, String> {
    let _gid = doc_model::NodeId::from_string(&group_id)
        .ok_or_else(|| format!("Invalid group ID: {}", group_id))?;
    let _sid = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Implement with edit_engine
    Ok(DocumentChange::default())
}

/// Get a shape group by ID
#[tauri::command]
pub fn get_shape_group(
    _doc_id: String,
    group_id: String,
) -> Result<Option<ShapeGroupDto>, String> {
    let _id = doc_model::NodeId::from_string(&group_id)
        .ok_or_else(|| format!("Invalid group ID: {}", group_id))?;

    // TODO: Get from document state
    Ok(None)
}

/// List all shape groups in the document
#[tauri::command]
pub fn list_shape_groups(
    _doc_id: String,
) -> Result<Vec<ShapeGroupDto>, String> {
    // TODO: Implement with document state
    Ok(Vec::new())
}

// =============================================================================
// Shape Alignment/Distribution Commands
// =============================================================================

/// Align shapes
#[tauri::command]
pub fn align_shapes(
    _doc_id: String,
    shape_ids: Vec<String>,
    horizontal: Option<String>, // "left", "center", "right"
    vertical: Option<String>, // "top", "middle", "bottom"
    reference: AlignmentReferenceDto,
) -> Result<DocumentChange, String> {
    // Validate all shape IDs
    for id in &shape_ids {
        doc_model::NodeId::from_string(id)
            .ok_or_else(|| format!("Invalid shape ID: {}", id))?;
    }

    // TODO: Implement with edit_engine
    let _ = horizontal;
    let _ = vertical;
    let _ = reference;
    Ok(DocumentChange::default())
}

/// Distribute shapes evenly
#[tauri::command]
pub fn distribute_shapes(
    _doc_id: String,
    shape_ids: Vec<String>,
    direction: String, // "horizontal", "vertical"
    reference: AlignmentReferenceDto,
) -> Result<DocumentChange, String> {
    // Validate all shape IDs
    for id in &shape_ids {
        doc_model::NodeId::from_string(id)
            .ok_or_else(|| format!("Invalid shape ID: {}", id))?;
    }

    // TODO: Implement with edit_engine
    let _ = direction;
    let _ = reference;
    Ok(DocumentChange::default())
}

// =============================================================================
// Connector Commands
// =============================================================================

/// Insert a connector between shapes
#[tauri::command]
pub fn insert_connector(
    _doc_id: String,
    start: ConnectorEndpointDto,
    end: ConnectorEndpointDto,
    routing: String, // "straight", "elbow", "curved"
    arrows: ArrowConfigDto,
) -> Result<ConnectorDto, String> {
    // TODO: Implement with edit_engine
    Ok(ConnectorDto {
        id: "connector_id".to_string(),
        start,
        end,
        routing,
        line_style: ShapeStrokeDto::default(),
        arrows,
        adjustments: Vec::new(),
        name: None,
    })
}

/// Update a connector
#[tauri::command]
pub fn update_connector(
    _doc_id: String,
    connector_id: String,
    start: Option<ConnectorEndpointDto>,
    end: Option<ConnectorEndpointDto>,
    routing: Option<String>,
    line_style: Option<ShapeStrokeDto>,
    arrows: Option<ArrowConfigDto>,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&connector_id)
        .ok_or_else(|| format!("Invalid connector ID: {}", connector_id))?;

    // TODO: Implement with edit_engine
    let _ = start;
    let _ = end;
    let _ = routing;
    let _ = line_style;
    let _ = arrows;
    Ok(DocumentChange::default())
}

/// Delete a connector
#[tauri::command]
pub fn delete_connector(
    _doc_id: String,
    connector_id: String,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&connector_id)
        .ok_or_else(|| format!("Invalid connector ID: {}", connector_id))?;

    // TODO: Implement with edit_engine
    Ok(DocumentChange::default())
}

/// Get a connector by ID
#[tauri::command]
pub fn get_connector(
    _doc_id: String,
    connector_id: String,
) -> Result<Option<ConnectorDto>, String> {
    let _id = doc_model::NodeId::from_string(&connector_id)
        .ok_or_else(|| format!("Invalid connector ID: {}", connector_id))?;

    // TODO: Get from document state
    Ok(None)
}

/// List all connectors in the document
#[tauri::command]
pub fn list_connectors(
    _doc_id: String,
) -> Result<Vec<ConnectorDto>, String> {
    // TODO: Implement with document state
    Ok(Vec::new())
}

/// Get connectors connected to a specific shape
#[tauri::command]
pub fn get_connectors_for_shape(
    _doc_id: String,
    shape_id: String,
) -> Result<Vec<ConnectorDto>, String> {
    let _id = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Get from document state
    Ok(Vec::new())
}

// =============================================================================
// Z-Order Commands
// =============================================================================

/// Bring shape to front
#[tauri::command]
pub fn bring_to_front(
    _doc_id: String,
    shape_id: String,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Implement with edit_engine
    Ok(DocumentChange::default())
}

/// Send shape to back
#[tauri::command]
pub fn send_to_back(
    _doc_id: String,
    shape_id: String,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Implement with edit_engine
    Ok(DocumentChange::default())
}

/// Bring shape forward one level
#[tauri::command]
pub fn bring_forward(
    _doc_id: String,
    shape_id: String,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Implement with edit_engine
    Ok(DocumentChange::default())
}

/// Send shape backward one level
#[tauri::command]
pub fn send_backward(
    _doc_id: String,
    shape_id: String,
) -> Result<DocumentChange, String> {
    let _id = doc_model::NodeId::from_string(&shape_id)
        .ok_or_else(|| format!("Invalid shape ID: {}", shape_id))?;

    // TODO: Implement with edit_engine
    Ok(DocumentChange::default())
}

// =============================================================================
// Shape Library/Presets Commands
// =============================================================================

/// Get available shape types by category
#[tauri::command]
pub fn get_shape_types_by_category(
    category: String, // "basic", "blockArrows", "flowchart", "callouts", "starsAndBanners", "equation", "actionButtons"
) -> Result<Vec<ShapeTypeDto>, String> {
    // Return preset shape types for the given category
    let shapes = match category.as_str() {
        "basic" => vec![
            ShapeTypeDto::Rectangle,
            ShapeTypeDto::RoundedRectangle { corner_radius: 10.0 },
            ShapeTypeDto::Oval,
            ShapeTypeDto::Triangle,
            ShapeTypeDto::Diamond,
            ShapeTypeDto::Pentagon,
            ShapeTypeDto::Hexagon,
            ShapeTypeDto::Octagon,
            ShapeTypeDto::Heart,
            ShapeTypeDto::Cloud,
        ],
        "blockArrows" => vec![
            ShapeTypeDto::RightArrow { head_width: 0.6, head_length: 0.35 },
            ShapeTypeDto::LeftArrow { head_width: 0.6, head_length: 0.35 },
            ShapeTypeDto::UpArrow { head_width: 0.6, head_length: 0.35 },
            ShapeTypeDto::DownArrow { head_width: 0.6, head_length: 0.35 },
            ShapeTypeDto::LeftRightArrow { head_width: 0.6, head_length: 0.35 },
            ShapeTypeDto::UpDownArrow { head_width: 0.6, head_length: 0.35 },
            ShapeTypeDto::QuadArrow { head_width: 0.6, head_length: 0.35 },
            ShapeTypeDto::ChevronArrow { thickness: 0.3 },
        ],
        "flowchart" => vec![
            ShapeTypeDto::FlowchartProcess,
            ShapeTypeDto::FlowchartDecision,
            ShapeTypeDto::FlowchartData,
            ShapeTypeDto::FlowchartTerminator,
            ShapeTypeDto::FlowchartDocument,
            ShapeTypeDto::FlowchartPredefined,
            ShapeTypeDto::FlowchartManualInput,
            ShapeTypeDto::FlowchartPreparation,
            ShapeTypeDto::FlowchartConnector,
            ShapeTypeDto::FlowchartOffPageConnector,
            ShapeTypeDto::FlowchartDelay,
        ],
        "callouts" => vec![
            ShapeTypeDto::RectangularCallout {
                tail_anchor: (0.5, 1.0),
                tail_tip: (0.0, 30.0),
                tail_width: 20.0,
            },
            ShapeTypeDto::RoundedCallout {
                corner_radius: 10.0,
                tail_anchor: (0.5, 1.0),
                tail_tip: (0.0, 30.0),
                tail_width: 20.0,
            },
            ShapeTypeDto::OvalCallout {
                tail_anchor: (0.5, 1.0),
                tail_tip: (0.0, 30.0),
                tail_width: 20.0,
            },
            ShapeTypeDto::CloudCallout {
                tail_tip: (0.0, 30.0),
                bubble_count: 3,
            },
            ShapeTypeDto::LineCallout { accent_bar: false },
        ],
        "starsAndBanners" => vec![
            ShapeTypeDto::Star4,
            ShapeTypeDto::Star5,
            ShapeTypeDto::Star6,
            ShapeTypeDto::Star8,
            ShapeTypeDto::Star10,
            ShapeTypeDto::Star12,
            ShapeTypeDto::Ribbon { tail_length: 0.3, tails_up: false },
            ShapeTypeDto::Wave { amplitude: 0.2, periods: 1.0 },
            ShapeTypeDto::DoubleWave { amplitude: 0.2, periods: 1.0 },
            ShapeTypeDto::HorizontalScroll { roll_size: 0.2 },
            ShapeTypeDto::VerticalScroll { roll_size: 0.2 },
        ],
        "equation" => vec![
            ShapeTypeDto::MathPlus,
            ShapeTypeDto::MathMinus,
            ShapeTypeDto::MathMultiply,
            ShapeTypeDto::MathDivide,
            ShapeTypeDto::MathEqual,
        ],
        _ => Vec::new(),
    };

    Ok(shapes)
}

/// Get all shape categories
#[tauri::command]
pub fn get_shape_categories() -> Vec<String> {
    vec![
        "basic".to_string(),
        "blockArrows".to_string(),
        "flowchart".to_string(),
        "callouts".to_string(),
        "starsAndBanners".to_string(),
        "equation".to_string(),
        "actionButtons".to_string(),
    ]
}
