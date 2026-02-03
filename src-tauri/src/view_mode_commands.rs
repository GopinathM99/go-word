//! Tauri IPC commands for view mode operations

use crate::state::ViewModeState;
use layout_engine::{DraftViewOptions, OutlineViewOptions, ViewMode, ViewModeConfig};
use serde::{Deserialize, Serialize};
use tauri::State;

// =============================================================================
// DTO Types for Frontend Communication
// =============================================================================

/// View mode DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewModeDto {
    pub mode: String,
    pub display_name: String,
    pub shortcut: String,
    pub shows_page_breaks: bool,
    pub is_continuous: bool,
}

impl From<ViewMode> for ViewModeDto {
    fn from(mode: ViewMode) -> Self {
        Self {
            mode: match mode {
                ViewMode::PrintLayout => "print_layout".to_string(),
                ViewMode::Draft => "draft".to_string(),
                ViewMode::Outline => "outline".to_string(),
                ViewMode::WebLayout => "web_layout".to_string(),
            },
            display_name: mode.display_name().to_string(),
            shortcut: mode.shortcut_hint().to_string(),
            shows_page_breaks: mode.shows_page_breaks(),
            is_continuous: mode.is_continuous(),
        }
    }
}

/// Draft view options DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftViewOptionsDto {
    pub show_style_names: bool,
    pub show_images: bool,
    pub wrap_to_window: bool,
    pub show_paragraph_marks: bool,
    pub line_spacing_multiplier: f32,
    pub style_name_margin: f32,
}

impl From<DraftViewOptions> for DraftViewOptionsDto {
    fn from(options: DraftViewOptions) -> Self {
        Self {
            show_style_names: options.show_style_names,
            show_images: options.show_images,
            wrap_to_window: options.wrap_to_window,
            show_paragraph_marks: options.show_paragraph_marks,
            line_spacing_multiplier: options.line_spacing_multiplier,
            style_name_margin: options.style_name_margin,
        }
    }
}

impl From<DraftViewOptionsDto> for DraftViewOptions {
    fn from(dto: DraftViewOptionsDto) -> Self {
        Self {
            show_style_names: dto.show_style_names,
            show_images: dto.show_images,
            wrap_to_window: dto.wrap_to_window,
            show_paragraph_marks: dto.show_paragraph_marks,
            line_spacing_multiplier: dto.line_spacing_multiplier,
            style_name_margin: dto.style_name_margin,
        }
    }
}

/// Outline view options DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineViewOptionsDto {
    pub show_levels_start: u8,
    pub show_levels_end: u8,
    pub show_body_text: bool,
    pub show_first_line_only: bool,
    pub show_level_indicators: bool,
    pub enable_drag_drop: bool,
    pub indent_per_level: f32,
}

impl From<OutlineViewOptions> for OutlineViewOptionsDto {
    fn from(options: OutlineViewOptions) -> Self {
        Self {
            show_levels_start: options.show_levels_start,
            show_levels_end: options.show_levels_end,
            show_body_text: options.show_body_text,
            show_first_line_only: options.show_first_line_only,
            show_level_indicators: options.show_level_indicators,
            enable_drag_drop: options.enable_drag_drop,
            indent_per_level: options.indent_per_level,
        }
    }
}

impl From<OutlineViewOptionsDto> for OutlineViewOptions {
    fn from(dto: OutlineViewOptionsDto) -> Self {
        Self {
            show_levels_start: dto.show_levels_start,
            show_levels_end: dto.show_levels_end,
            show_body_text: dto.show_body_text,
            show_first_line_only: dto.show_first_line_only,
            show_level_indicators: dto.show_level_indicators,
            enable_drag_drop: dto.enable_drag_drop,
            indent_per_level: dto.indent_per_level,
        }
    }
}

/// View mode config DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewModeConfigDto {
    pub mode: ViewModeDto,
    pub draft_options: DraftViewOptionsDto,
    pub outline_options: OutlineViewOptionsDto,
}

impl From<ViewModeConfig> for ViewModeConfigDto {
    fn from(config: ViewModeConfig) -> Self {
        Self {
            mode: config.mode.into(),
            draft_options: config.draft_options.into(),
            outline_options: config.outline_options.into(),
        }
    }
}

// =============================================================================
// Commands
// =============================================================================

/// Get the current view mode for a document
#[tauri::command]
pub fn get_view_mode(doc_id: String, state: State<'_, ViewModeState>) -> Result<ViewModeDto, String> {
    let mode = state.get_mode(&doc_id);
    Ok(mode.into())
}

/// Set the view mode for a document
#[tauri::command]
pub fn set_view_mode(doc_id: String, mode: String, state: State<'_, ViewModeState>) -> Result<ViewModeDto, String> {
    let view_mode = match mode.as_str() {
        "print_layout" | "printLayout" => ViewMode::PrintLayout,
        "draft" => ViewMode::Draft,
        "outline" => ViewMode::Outline,
        "web_layout" | "webLayout" => ViewMode::WebLayout,
        _ => return Err(format!("Invalid view mode: {}", mode)),
    };

    state.set_mode(&doc_id, view_mode);
    tracing::info!("Set view mode for doc {} to {:?}", doc_id, view_mode);

    Ok(view_mode.into())
}

/// Get the full view mode configuration for a document
#[tauri::command]
pub fn get_view_mode_config(doc_id: String, state: State<'_, ViewModeState>) -> Result<ViewModeConfigDto, String> {
    let config = state.get_or_create(&doc_id);
    Ok(config.into())
}

/// Get draft view options for a document
#[tauri::command]
pub fn get_draft_options(doc_id: String, state: State<'_, ViewModeState>) -> Result<DraftViewOptionsDto, String> {
    let options = state.get_draft_options(&doc_id);
    Ok(options.into())
}

/// Set draft view options for a document
#[tauri::command]
pub fn set_draft_options(
    doc_id: String,
    options: DraftViewOptionsDto,
    state: State<'_, ViewModeState>,
) -> Result<(), String> {
    state.set_draft_options(&doc_id, options.into());
    tracing::info!("Updated draft options for doc {}", doc_id);
    Ok(())
}

/// Get outline view options for a document
#[tauri::command]
pub fn get_outline_options(doc_id: String, state: State<'_, ViewModeState>) -> Result<OutlineViewOptionsDto, String> {
    let options = state.get_outline_options(&doc_id);
    Ok(options.into())
}

/// Set outline view options for a document
#[tauri::command]
pub fn set_outline_options(
    doc_id: String,
    options: OutlineViewOptionsDto,
    state: State<'_, ViewModeState>,
) -> Result<(), String> {
    state.set_outline_options(&doc_id, options.into());
    tracing::info!("Updated outline options for doc {}", doc_id);
    Ok(())
}

/// Get available view modes
#[tauri::command]
pub fn get_available_view_modes() -> Result<Vec<ViewModeDto>, String> {
    Ok(vec![
        ViewMode::PrintLayout.into(),
        ViewMode::Draft.into(),
        ViewMode::Outline.into(),
        ViewMode::WebLayout.into(),
    ])
}

/// Get simplified draft layout for a document
/// This converts the paginated layout to a continuous flow without page breaks
#[tauri::command]
pub fn get_draft_layout(doc_id: String, _state: State<'_, ViewModeState>) -> Result<DraftLayoutDto, String> {
    // TODO: In a real implementation, this would:
    // 1. Get the document's current layout tree from AppState
    // 2. Get draft options from ViewModeState
    // 3. Create DraftLayout from the layout tree
    // 4. Enrich with metadata if needed

    // For now, return a placeholder
    Ok(DraftLayoutDto {
        blocks: vec![],
        total_height: 0.0,
        content_width: 468.0,
    })
}

/// DTO for draft layout
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftLayoutDto {
    pub blocks: Vec<DraftBlockDto>,
    pub total_height: f32,
    pub content_width: f32,
}

/// DTO for draft block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftBlockDto {
    pub node_id: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub style_name: Option<String>,
    pub is_heading: bool,
    pub heading_level: Option<u8>,
}

/// Promote a heading (e.g., H2 -> H1)
#[tauri::command]
pub fn promote_heading(doc_id: String, heading_id: String) -> Result<(), String> {
    // TODO: Implement with edit_engine
    // This would:
    // 1. Find the heading paragraph
    // 2. Get current heading level from style
    // 3. Apply new style with level - 1 (e.g., Heading 2 -> Heading 1)
    tracing::info!("Promote heading {} in doc {}", heading_id, doc_id);
    Ok(())
}

/// Demote a heading (e.g., H1 -> H2)
#[tauri::command]
pub fn demote_heading(doc_id: String, heading_id: String) -> Result<(), String> {
    // TODO: Implement with edit_engine
    // This would:
    // 1. Find the heading paragraph
    // 2. Get current heading level from style
    // 3. Apply new style with level + 1 (e.g., Heading 1 -> Heading 2)
    tracing::info!("Demote heading {} in doc {}", heading_id, doc_id);
    Ok(())
}

/// Move a section (heading and its content) to a new position
#[tauri::command]
pub fn move_section(
    doc_id: String,
    heading_id: String,
    before_id: Option<String>,
) -> Result<(), String> {
    // TODO: Implement with edit_engine
    // This would:
    // 1. Find the heading paragraph and all content until next same-or-higher level heading
    // 2. Cut the section
    // 3. Insert before the specified location (or at end if before_id is None)
    tracing::info!(
        "Move section {} in doc {} before {:?}",
        heading_id,
        doc_id,
        before_id
    );
    Ok(())
}

/// Expand a heading in outline view
#[tauri::command]
pub fn expand_outline_heading(doc_id: String, heading_id: String) -> Result<(), String> {
    // This is tracked in frontend state, but we log it for debugging
    tracing::debug!("Expand outline heading {} in doc {}", heading_id, doc_id);
    Ok(())
}

/// Collapse a heading in outline view
#[tauri::command]
pub fn collapse_outline_heading(doc_id: String, heading_id: String) -> Result<(), String> {
    // This is tracked in frontend state, but we log it for debugging
    tracing::debug!("Collapse outline heading {} in doc {}", heading_id, doc_id);
    Ok(())
}

/// Set heading level filter for outline view
#[tauri::command]
pub fn set_outline_level_filter(
    doc_id: String,
    start_level: u8,
    end_level: u8,
    state: State<'_, ViewModeState>,
) -> Result<(), String> {
    let mut options = state.get_outline_options(&doc_id);
    options.show_levels_start = start_level.clamp(1, 6);
    options.show_levels_end = end_level.clamp(2, 7);
    state.set_outline_options(&doc_id, options);
    tracing::info!(
        "Set outline level filter for doc {} to {}-{}",
        doc_id,
        start_level,
        end_level
    );
    Ok(())
}
