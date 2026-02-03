//! View Mode Support for Document Editor
//!
//! This module provides different view modes for the document editor:
//! - PrintLayout: Shows pages as they will print (default)
//! - Draft: Continuous scroll without page breaks for fast editing
//! - Outline: Hierarchical heading view for document navigation
//! - WebLayout: Future support for HTML export preview

use crate::{BlockBox, LayoutTree, LineBox, Rect};
use doc_model::{DocumentTree, Node, NodeId};
use serde::{Deserialize, Serialize};
use std::ops::Range;

// =============================================================================
// View Mode Enum
// =============================================================================

/// Available view modes for the document editor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
    /// Default - shows pages as they will print
    #[default]
    PrintLayout,
    /// Continuous scroll, no page breaks
    Draft,
    /// Hierarchical heading view
    Outline,
    /// Future: for HTML export preview
    WebLayout,
}

impl ViewMode {
    /// Get display name for the view mode
    pub fn display_name(&self) -> &'static str {
        match self {
            ViewMode::PrintLayout => "Print Layout",
            ViewMode::Draft => "Draft",
            ViewMode::Outline => "Outline",
            ViewMode::WebLayout => "Web Layout",
        }
    }

    /// Get keyboard shortcut hint
    pub fn shortcut_hint(&self) -> &'static str {
        match self {
            ViewMode::PrintLayout => "Ctrl+Alt+P",
            ViewMode::Draft => "Ctrl+Alt+N",
            ViewMode::Outline => "Ctrl+Alt+O",
            ViewMode::WebLayout => "Ctrl+Alt+W",
        }
    }

    /// Check if this mode shows page breaks
    pub fn shows_page_breaks(&self) -> bool {
        matches!(self, ViewMode::PrintLayout)
    }

    /// Check if this mode uses continuous scroll
    pub fn is_continuous(&self) -> bool {
        matches!(self, ViewMode::Draft | ViewMode::WebLayout)
    }
}

// =============================================================================
// Draft View Options
// =============================================================================

/// Options for Draft view mode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftViewOptions {
    /// Show style names in left margin
    pub show_style_names: bool,
    /// Show images (false = show placeholders for speed)
    pub show_images: bool,
    /// Wrap text to window width
    pub wrap_to_window: bool,
    /// Show paragraph markers
    pub show_paragraph_marks: bool,
    /// Line spacing multiplier (can be reduced for compact view)
    pub line_spacing_multiplier: f32,
    /// Left margin for style names (in points)
    pub style_name_margin: f32,
}

impl Default for DraftViewOptions {
    fn default() -> Self {
        Self {
            show_style_names: false,
            show_images: false,
            wrap_to_window: true,
            show_paragraph_marks: false,
            line_spacing_multiplier: 1.0,
            style_name_margin: 100.0,
        }
    }
}

impl DraftViewOptions {
    /// Create options optimized for maximum editing speed
    pub fn fast_editing() -> Self {
        Self {
            show_style_names: false,
            show_images: false,
            wrap_to_window: true,
            show_paragraph_marks: false,
            line_spacing_multiplier: 1.0,
            style_name_margin: 0.0,
        }
    }

    /// Create options showing style names
    pub fn with_style_names() -> Self {
        Self {
            show_style_names: true,
            show_images: false,
            wrap_to_window: true,
            show_paragraph_marks: false,
            line_spacing_multiplier: 1.0,
            style_name_margin: 100.0,
        }
    }
}

// =============================================================================
// Outline View Options
// =============================================================================

/// Options for Outline view mode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineViewOptions {
    /// Range of heading levels to show (e.g., 1..4 for H1-H3)
    pub show_levels_start: u8,
    pub show_levels_end: u8,
    /// Show body text under headings
    pub show_body_text: bool,
    /// Show only first line of body text
    pub show_first_line_only: bool,
    /// Show heading level indicators
    pub show_level_indicators: bool,
    /// Allow drag-and-drop reordering
    pub enable_drag_drop: bool,
    /// Indent per level in pixels
    pub indent_per_level: f32,
}

impl Default for OutlineViewOptions {
    fn default() -> Self {
        Self {
            show_levels_start: 1,
            show_levels_end: 7, // Show all levels by default (H1-H6)
            show_body_text: false,
            show_first_line_only: true,
            show_level_indicators: true,
            enable_drag_drop: true,
            indent_per_level: 20.0,
        }
    }
}

impl OutlineViewOptions {
    /// Get the range of heading levels to show
    pub fn show_levels(&self) -> Range<u8> {
        self.show_levels_start..self.show_levels_end
    }

    /// Set the range of heading levels to show
    pub fn set_show_levels(&mut self, range: Range<u8>) {
        self.show_levels_start = range.start;
        self.show_levels_end = range.end;
    }

    /// Create options showing only top-level headings
    pub fn top_level_only() -> Self {
        Self {
            show_levels_start: 1,
            show_levels_end: 2,
            show_body_text: false,
            show_first_line_only: false,
            show_level_indicators: true,
            enable_drag_drop: true,
            indent_per_level: 20.0,
        }
    }

    /// Create options with body text preview
    pub fn with_body_preview() -> Self {
        Self {
            show_levels_start: 1,
            show_levels_end: 7,
            show_body_text: true,
            show_first_line_only: true,
            show_level_indicators: true,
            enable_drag_drop: true,
            indent_per_level: 20.0,
        }
    }
}

// =============================================================================
// View Mode Configuration
// =============================================================================

/// Complete view mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewModeConfig {
    /// Current view mode
    pub mode: ViewMode,
    /// Draft view options
    pub draft_options: DraftViewOptions,
    /// Outline view options
    pub outline_options: OutlineViewOptions,
}

impl Default for ViewModeConfig {
    fn default() -> Self {
        Self {
            mode: ViewMode::PrintLayout,
            draft_options: DraftViewOptions::default(),
            outline_options: OutlineViewOptions::default(),
        }
    }
}

impl ViewModeConfig {
    /// Create a new configuration with the specified mode
    pub fn new(mode: ViewMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    /// Set draft view options
    pub fn with_draft_options(mut self, options: DraftViewOptions) -> Self {
        self.draft_options = options;
        self
    }

    /// Set outline view options
    pub fn with_outline_options(mut self, options: OutlineViewOptions) -> Self {
        self.outline_options = options;
        self
    }
}

// =============================================================================
// Draft Layout
// =============================================================================

/// A simplified layout for draft view (continuous, no page breaks)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DraftLayout {
    /// All blocks in document order (no page separation)
    pub blocks: Vec<DraftBlock>,
    /// Total content height
    pub total_height: f32,
    /// Content width
    pub content_width: f32,
}

/// A block in draft layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftBlock {
    /// The paragraph's node ID
    pub node_id: NodeId,
    /// Bounding rectangle
    pub bounds: Rect,
    /// Lines in this block
    pub lines: Vec<LineBox>,
    /// Style name (if show_style_names is enabled)
    pub style_name: Option<String>,
    /// Whether this is a heading
    pub is_heading: bool,
    /// Heading level (1-6) if is_heading
    pub heading_level: Option<u8>,
}

impl DraftLayout {
    /// Create a draft layout from a paginated layout tree
    pub fn from_layout_tree(layout: &LayoutTree, options: &DraftViewOptions) -> Self {
        let mut blocks = Vec::new();
        let mut current_y = 0.0f32;
        let content_width = layout
            .pages
            .first()
            .and_then(|p| p.content_area_box())
            .map(|a| a.bounds.width)
            .unwrap_or(468.0);

        // Flatten all pages into a single continuous flow
        for page in &layout.pages {
            for area in &page.areas {
                for column in &area.columns {
                    for block in &column.blocks {
                        let block_height = block.bounds.height;

                        // Create draft block with updated Y position
                        let mut draft_block = DraftBlock {
                            node_id: block.node_id,
                            bounds: Rect::new(0.0, current_y, content_width, block_height),
                            lines: block.lines.clone(),
                            style_name: None,
                            is_heading: false,
                            heading_level: None,
                        };

                        // Update line positions
                        let block_y = block.bounds.y;
                        for line in &mut draft_block.lines {
                            let relative_y = line.bounds.y - block_y;
                            line.bounds.y = current_y + relative_y;
                        }

                        blocks.push(draft_block);
                        current_y += block_height;
                    }
                }
            }
        }

        Self {
            blocks,
            total_height: current_y,
            content_width,
        }
    }

    /// Enrich draft layout with document metadata (style names, heading info)
    pub fn enrich_with_metadata(&mut self, tree: &DocumentTree, options: &DraftViewOptions) {
        for block in &mut self.blocks {
            if let Some(para) = tree.get_paragraph(block.node_id) {
                // Set style name if enabled
                if options.show_style_names {
                    block.style_name = para.paragraph_style_id.as_ref().map(|id| id.to_string());
                }

                // Check if this is a heading
                if let Some(style_id) = &para.paragraph_style_id {
                    let style_name = style_id.to_string().to_lowercase();
                    if style_name.starts_with("heading") || style_name.starts_with("h") {
                        block.is_heading = true;
                        // Try to extract heading level from style name
                        block.heading_level = style_name
                            .chars()
                            .filter(|c| c.is_ascii_digit())
                            .collect::<String>()
                            .parse()
                            .ok();
                    }
                }
            }
        }
    }
}

// =============================================================================
// Outline Data
// =============================================================================

/// A heading in the outline view
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineHeading {
    /// Unique identifier
    pub id: String,
    /// Node ID in document
    pub node_id: NodeId,
    /// Heading level (1-6)
    pub level: u8,
    /// Heading text
    pub text: String,
    /// Child headings
    pub children: Vec<OutlineHeading>,
    /// First line of body text (if show_body_text is enabled)
    pub body_preview: Option<String>,
    /// Character offset in document
    pub offset: usize,
    /// Whether this heading is expanded in the view
    pub expanded: bool,
}

impl OutlineHeading {
    /// Create a new outline heading
    pub fn new(id: String, node_id: NodeId, level: u8, text: String) -> Self {
        Self {
            id,
            node_id,
            level,
            text,
            children: Vec::new(),
            body_preview: None,
            offset: 0,
            expanded: true,
        }
    }

    /// Count total headings including children
    pub fn total_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.total_count()).sum::<usize>()
    }

    /// Find a heading by ID
    pub fn find_by_id(&self, id: &str) -> Option<&OutlineHeading> {
        if self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_by_id(id) {
                return Some(found);
            }
        }
        None
    }

    /// Find a heading by ID (mutable)
    pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut OutlineHeading> {
        if self.id == id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_by_id_mut(id) {
                return Some(found);
            }
        }
        None
    }
}

/// Complete outline data for a document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineData {
    /// Root-level headings
    pub headings: Vec<OutlineHeading>,
    /// Total heading count
    pub total_count: usize,
}

impl OutlineData {
    /// Build outline from document tree
    pub fn from_document(tree: &DocumentTree, options: &OutlineViewOptions) -> Self {
        let mut headings = Vec::new();
        let mut stack: Vec<OutlineHeading> = Vec::new();
        let show_range = options.show_levels();

        for para in tree.paragraphs() {
            // Check if this is a heading
            let heading_level = Self::get_heading_level(tree, para.id());
            if let Some(level) = heading_level {
                // Filter by level range
                if !show_range.contains(&level) {
                    continue;
                }

                // Get heading text
                let text = Self::get_paragraph_text(tree, para.id());
                if text.is_empty() {
                    continue;
                }

                let heading = OutlineHeading::new(
                    para.id().to_string(),
                    para.id(),
                    level,
                    text,
                );

                // Pop stack until we find a parent with lower level
                while !stack.is_empty() && stack.last().unwrap().level >= level {
                    let completed = stack.pop().unwrap();
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(completed);
                    } else {
                        headings.push(completed);
                    }
                }

                stack.push(heading);
            }
        }

        // Pop remaining items from stack
        while let Some(completed) = stack.pop() {
            if let Some(parent) = stack.last_mut() {
                parent.children.push(completed);
            } else {
                headings.push(completed);
            }
        }

        let total_count = headings.iter().map(|h| h.total_count()).sum();

        Self {
            headings,
            total_count,
        }
    }

    /// Get heading level from paragraph style
    fn get_heading_level(tree: &DocumentTree, para_id: NodeId) -> Option<u8> {
        let para = tree.get_paragraph(para_id)?;
        let style_id = para.paragraph_style_id.as_ref()?;
        let style_name = style_id.to_string().to_lowercase();

        // Check for "heading N" or "hN" style names
        if style_name.starts_with("heading") {
            style_name
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .ok()
        } else if style_name.len() == 2 && style_name.starts_with('h') {
            style_name[1..].parse().ok()
        } else {
            None
        }
    }

    /// Get plain text from paragraph
    fn get_paragraph_text(tree: &DocumentTree, para_id: NodeId) -> String {
        let para = match tree.get_paragraph(para_id) {
            Some(p) => p,
            None => return String::new(),
        };

        let mut text = String::new();
        for &child_id in para.children() {
            if let Some(run) = tree.nodes.runs.get(&child_id) {
                text.push_str(&run.text);
            }
        }
        text
    }

    /// Find a heading by ID
    pub fn find_heading(&self, id: &str) -> Option<&OutlineHeading> {
        for heading in &self.headings {
            if let Some(found) = heading.find_by_id(id) {
                return Some(found);
            }
        }
        None
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_mode_default() {
        let mode = ViewMode::default();
        assert_eq!(mode, ViewMode::PrintLayout);
    }

    #[test]
    fn test_view_mode_properties() {
        assert!(ViewMode::PrintLayout.shows_page_breaks());
        assert!(!ViewMode::Draft.shows_page_breaks());

        assert!(ViewMode::Draft.is_continuous());
        assert!(ViewMode::WebLayout.is_continuous());
        assert!(!ViewMode::PrintLayout.is_continuous());
        assert!(!ViewMode::Outline.is_continuous());
    }

    #[test]
    fn test_draft_options_default() {
        let options = DraftViewOptions::default();
        assert!(!options.show_style_names);
        assert!(!options.show_images);
        assert!(options.wrap_to_window);
    }

    #[test]
    fn test_outline_options_levels() {
        let mut options = OutlineViewOptions::default();
        assert_eq!(options.show_levels(), 1..7);

        options.set_show_levels(1..4);
        assert_eq!(options.show_levels(), 1..4);
    }

    #[test]
    fn test_view_mode_config() {
        let config = ViewModeConfig::new(ViewMode::Draft)
            .with_draft_options(DraftViewOptions::fast_editing());

        assert_eq!(config.mode, ViewMode::Draft);
        assert!(!config.draft_options.show_style_names);
    }

    #[test]
    fn test_outline_heading_count() {
        let mut heading = OutlineHeading::new(
            "h1".to_string(),
            NodeId::new(),
            1,
            "Chapter 1".to_string(),
        );
        heading.children.push(OutlineHeading::new(
            "h2a".to_string(),
            NodeId::new(),
            2,
            "Section 1.1".to_string(),
        ));
        heading.children.push(OutlineHeading::new(
            "h2b".to_string(),
            NodeId::new(),
            2,
            "Section 1.2".to_string(),
        ));

        assert_eq!(heading.total_count(), 3);
    }
}
