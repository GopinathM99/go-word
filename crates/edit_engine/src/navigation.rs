//! Navigation module for cursor/caret movement in the document
//!
//! This module provides character, word, line, and paragraph navigation
//! with proper Unicode support using grapheme clusters and word boundaries.

use doc_model::{DocumentTree, Node, NodeId, NodeType, Position, Selection};
use unicode_segmentation::UnicodeSegmentation;

/// Direction for navigation operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Move backward (left in LTR text, up for vertical)
    Backward,
    /// Move forward (right in LTR text, down for vertical)
    Forward,
}

/// Unit of movement for navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementUnit {
    /// Move by grapheme cluster (single character)
    Character,
    /// Move by word
    Word,
    /// Move by line (requires layout information)
    Line,
    /// Move by paragraph
    Paragraph,
    /// Move to line boundary (home/end)
    LineBoundary,
    /// Move to document boundary
    DocumentBoundary,
}

/// Navigation options
#[derive(Debug, Clone, Copy, Default)]
pub struct NavigationOptions {
    /// Whether to extend the selection (Shift key)
    pub extend_selection: bool,
    /// Visual X position for vertical navigation (in pixels/points)
    /// Used to maintain horizontal position when moving up/down
    pub visual_x: Option<f32>,
}

/// Result of a navigation operation
#[derive(Debug, Clone)]
pub struct NavigationResult {
    /// The new selection after navigation
    pub selection: Selection,
    /// The visual X position for subsequent vertical navigation
    pub visual_x: Option<f32>,
}

/// Trait for providing layout information needed for line navigation
pub trait LayoutProvider {
    /// Get the visual position (x, y) for a document position
    fn position_to_visual(&self, pos: &Position, tree: &DocumentTree) -> Option<(f32, f32)>;

    /// Get the document position from a visual position
    fn visual_to_position(&self, x: f32, y: f32, tree: &DocumentTree) -> Option<Position>;

    /// Get the line boundaries (start and end positions) for a given position
    fn line_boundaries(&self, pos: &Position, tree: &DocumentTree) -> Option<(Position, Position)>;

    /// Get the position on the next/previous line at the given X coordinate
    fn position_on_adjacent_line(
        &self,
        pos: &Position,
        direction: Direction,
        visual_x: f32,
        tree: &DocumentTree,
    ) -> Option<Position>;
}

/// Navigator for cursor movement in the document
pub struct Navigator<'a> {
    tree: &'a DocumentTree,
    layout: Option<&'a dyn LayoutProvider>,
}

impl<'a> Navigator<'a> {
    /// Create a new navigator for the given document tree
    pub fn new(tree: &'a DocumentTree) -> Self {
        Self { tree, layout: None }
    }

    /// Create a navigator with layout information for line navigation
    pub fn with_layout(tree: &'a DocumentTree, layout: &'a dyn LayoutProvider) -> Self {
        Self {
            tree,
            layout: Some(layout),
        }
    }

    /// Navigate in the specified direction by the specified unit
    pub fn navigate(
        &self,
        selection: &Selection,
        direction: Direction,
        unit: MovementUnit,
        options: NavigationOptions,
    ) -> NavigationResult {
        let new_position = match unit {
            MovementUnit::Character => self.move_by_character(&selection.focus, direction),
            MovementUnit::Word => self.move_by_word(&selection.focus, direction),
            MovementUnit::Paragraph => self.move_by_paragraph(&selection.focus, direction),
            MovementUnit::LineBoundary => self.move_to_line_boundary(&selection.focus, direction),
            MovementUnit::DocumentBoundary => self.move_to_document_boundary(direction),
            MovementUnit::Line => {
                // Line navigation requires layout information
                if let Some(layout) = self.layout {
                    let visual_x = options.visual_x.unwrap_or_else(|| {
                        layout
                            .position_to_visual(&selection.focus, self.tree)
                            .map(|(x, _)| x)
                            .unwrap_or(0.0)
                    });
                    layout
                        .position_on_adjacent_line(&selection.focus, direction, visual_x, self.tree)
                        .unwrap_or(selection.focus)
                } else {
                    // Without layout, fall back to paragraph navigation
                    self.move_by_paragraph(&selection.focus, direction)
                }
            }
        };

        let new_selection = if options.extend_selection {
            selection.extend_to(new_position)
        } else {
            // When not extending, collapse any existing selection first
            if !selection.is_collapsed() {
                // When collapsing a selection, move to the appropriate end
                let collapse_pos = match direction {
                    Direction::Backward => selection.start(),
                    Direction::Forward => selection.end(),
                };
                // For character moves, collapse to the edge; for other moves, apply the move
                if unit == MovementUnit::Character {
                    Selection::collapsed(collapse_pos)
                } else {
                    Selection::collapsed(new_position)
                }
            } else {
                Selection::collapsed(new_position)
            }
        };

        // Calculate visual X for vertical navigation memory
        let visual_x = if unit == MovementUnit::Line {
            options.visual_x.or_else(|| {
                self.layout
                    .and_then(|l| l.position_to_visual(&new_position, self.tree))
                    .map(|(x, _)| x)
            })
        } else {
            None
        };

        NavigationResult {
            selection: new_selection,
            visual_x,
        }
    }

    // ========================================================================
    // Character Navigation
    // ========================================================================

    /// Move by one grapheme cluster in the specified direction
    fn move_by_character(&self, pos: &Position, direction: Direction) -> Position {
        match direction {
            Direction::Forward => self.next_grapheme_position(pos),
            Direction::Backward => self.prev_grapheme_position(pos),
        }
    }

    /// Get the position after the next grapheme cluster
    fn next_grapheme_position(&self, pos: &Position) -> Position {
        // First, try to get text content at current position
        if let Some(text) = self.get_text_at_position(pos) {
            let graphemes: Vec<&str> = text.graphemes(true).collect();

            // If we can move within this run
            if pos.offset < graphemes.len() {
                return Position::new(pos.node_id, pos.offset + 1);
            }
        }

        // Otherwise, try to move to the next run/paragraph
        if let Some(next_pos) = self.next_text_position(pos) {
            return next_pos;
        }

        // Can't move further, stay at current position
        *pos
    }

    /// Get the position before the previous grapheme cluster
    fn prev_grapheme_position(&self, pos: &Position) -> Position {
        // If we can move within this run
        if pos.offset > 0 {
            return Position::new(pos.node_id, pos.offset - 1);
        }

        // Otherwise, try to move to the previous run/paragraph
        if let Some(prev_pos) = self.prev_text_position(pos) {
            return prev_pos;
        }

        // Can't move further, stay at current position
        *pos
    }

    /// Get the text content at a position (the run's text)
    fn get_text_at_position(&self, pos: &Position) -> Option<&str> {
        // Check if position is in a run
        if let Some(run) = self.tree.get_run(pos.node_id) {
            return Some(&run.text);
        }

        // If position is in a paragraph, get the first run
        if let Some(para) = self.tree.get_paragraph(pos.node_id) {
            if let Some(&first_run_id) = para.children().first() {
                if let Some(run) = self.tree.get_run(first_run_id) {
                    return Some(&run.text);
                }
            }
        }

        None
    }

    /// Get the next position after the current run (crossing run/paragraph boundaries)
    fn next_text_position(&self, pos: &Position) -> Option<Position> {
        let node_type = self.tree.node_type(pos.node_id)?;

        match node_type {
            NodeType::Run => {
                let run = self.tree.get_run(pos.node_id)?;
                let para_id = run.parent()?;
                let para = self.tree.get_paragraph(para_id)?;

                // Find the current run's index
                let run_idx = para
                    .children()
                    .iter()
                    .position(|&id| id == pos.node_id)?;

                // Try the next run in the same paragraph
                if run_idx + 1 < para.children().len() {
                    let next_run_id = para.children()[run_idx + 1];
                    return Some(Position::new(next_run_id, 0));
                }

                // Move to the next paragraph
                self.next_paragraph_start(para_id)
            }
            NodeType::Paragraph => {
                let para = self.tree.get_paragraph(pos.node_id)?;

                // Try the first run in this paragraph
                if let Some(&first_run_id) = para.children().first() {
                    return Some(Position::new(first_run_id, 0));
                }

                // Empty paragraph, move to next paragraph
                self.next_paragraph_start(pos.node_id)
            }
            _ => None,
        }
    }

    /// Get the previous position before the current run (crossing run/paragraph boundaries)
    fn prev_text_position(&self, pos: &Position) -> Option<Position> {
        let node_type = self.tree.node_type(pos.node_id)?;

        match node_type {
            NodeType::Run => {
                let run = self.tree.get_run(pos.node_id)?;
                let para_id = run.parent()?;
                let para = self.tree.get_paragraph(para_id)?;

                // Find the current run's index
                let run_idx = para
                    .children()
                    .iter()
                    .position(|&id| id == pos.node_id)?;

                // Try the previous run in the same paragraph
                if run_idx > 0 {
                    let prev_run_id = para.children()[run_idx - 1];
                    let prev_run = self.tree.get_run(prev_run_id)?;
                    let grapheme_count = prev_run.text.graphemes(true).count();
                    return Some(Position::new(prev_run_id, grapheme_count));
                }

                // Move to the previous paragraph's end
                self.prev_paragraph_end(para_id)
            }
            NodeType::Paragraph => {
                // Move to previous paragraph's end
                self.prev_paragraph_end(pos.node_id)
            }
            _ => None,
        }
    }

    /// Get the start position of the next paragraph
    fn next_paragraph_start(&self, current_para_id: NodeId) -> Option<Position> {
        let doc_children = self.tree.document.children();
        let para_idx = doc_children
            .iter()
            .position(|&id| id == current_para_id)?;

        if para_idx + 1 < doc_children.len() {
            let next_para_id = doc_children[para_idx + 1];
            let next_para = self.tree.get_paragraph(next_para_id)?;

            // Return position in first run, or paragraph itself if empty
            if let Some(&first_run_id) = next_para.children().first() {
                Some(Position::new(first_run_id, 0))
            } else {
                Some(Position::new(next_para_id, 0))
            }
        } else {
            None
        }
    }

    /// Get the end position of the previous paragraph
    fn prev_paragraph_end(&self, current_para_id: NodeId) -> Option<Position> {
        let doc_children = self.tree.document.children();
        let para_idx = doc_children
            .iter()
            .position(|&id| id == current_para_id)?;

        if para_idx > 0 {
            let prev_para_id = doc_children[para_idx - 1];
            let prev_para = self.tree.get_paragraph(prev_para_id)?;

            // Return position at end of last run, or paragraph itself if empty
            if let Some(&last_run_id) = prev_para.children().last() {
                let last_run = self.tree.get_run(last_run_id)?;
                let grapheme_count = last_run.text.graphemes(true).count();
                Some(Position::new(last_run_id, grapheme_count))
            } else {
                Some(Position::new(prev_para_id, 0))
            }
        } else {
            None
        }
    }

    // ========================================================================
    // Word Navigation
    // ========================================================================

    /// Move by one word in the specified direction
    fn move_by_word(&self, pos: &Position, direction: Direction) -> Position {
        match direction {
            Direction::Forward => self.next_word_boundary(pos),
            Direction::Backward => self.prev_word_boundary(pos),
        }
    }

    /// Find the next word boundary
    fn next_word_boundary(&self, pos: &Position) -> Position {
        // Get the current run's text
        if let Some(run) = self.tree.get_run(pos.node_id) {
            let text = &run.text;
            let graphemes: Vec<&str> = text.graphemes(true).collect();

            // Convert grapheme offset to byte offset
            let byte_offset: usize = graphemes.iter().take(pos.offset).map(|g| g.len()).sum();

            // Get word boundaries
            let word_indices: Vec<usize> = text.split_word_bound_indices().map(|(i, _)| i).collect();

            // Find the next word boundary after current position
            for &word_idx in &word_indices {
                if word_idx > byte_offset {
                    // Convert byte offset back to grapheme offset
                    let new_grapheme_offset = self.byte_to_grapheme_offset(text, word_idx);
                    return Position::new(pos.node_id, new_grapheme_offset);
                }
            }

            // No more word boundaries in this run, go to end of run
            let end_of_run = Position::new(pos.node_id, graphemes.len());

            // Then try the next run
            if let Some(next_pos) = self.next_text_position(&end_of_run) {
                // Skip whitespace at the beginning of the next word
                return self.skip_whitespace_forward(&next_pos);
            }

            return end_of_run;
        }

        // If in a paragraph, try to navigate through runs
        if let Some(para) = self.tree.get_paragraph(pos.node_id) {
            if let Some(&first_run_id) = para.children().first() {
                let run_pos = Position::new(first_run_id, 0);
                return self.next_word_boundary(&run_pos);
            }
        }

        *pos
    }

    /// Find the previous word boundary
    fn prev_word_boundary(&self, pos: &Position) -> Position {
        // Get the current run's text
        if let Some(run) = self.tree.get_run(pos.node_id) {
            let text = &run.text;
            let graphemes: Vec<&str> = text.graphemes(true).collect();

            // Handle position at start of run
            if pos.offset == 0 {
                if let Some(prev_pos) = self.prev_text_position(pos) {
                    return self.prev_word_boundary(&prev_pos);
                }
                return *pos;
            }

            // Convert grapheme offset to byte offset
            let byte_offset: usize = graphemes.iter().take(pos.offset).map(|g| g.len()).sum();

            // Get word boundaries
            let word_indices: Vec<usize> = text.split_word_bound_indices().map(|(i, _)| i).collect();

            // Find the previous word boundary before current position
            let mut prev_boundary: Option<usize> = None;
            for &word_idx in &word_indices {
                if word_idx >= byte_offset {
                    break;
                }
                // Skip whitespace boundaries
                let segment = &text[word_idx..];
                if let Some(first_char) = segment.chars().next() {
                    if !first_char.is_whitespace() || prev_boundary.is_none() {
                        prev_boundary = Some(word_idx);
                    }
                }
            }

            if let Some(boundary) = prev_boundary {
                if boundary > 0 {
                    // Convert byte offset back to grapheme offset
                    let new_grapheme_offset = self.byte_to_grapheme_offset(text, boundary);
                    return Position::new(pos.node_id, new_grapheme_offset);
                }
            }

            // Go to start of run
            let start_of_run = Position::new(pos.node_id, 0);

            // If at start, go to previous run
            if let Some(prev_pos) = self.prev_text_position(&start_of_run) {
                return self.skip_whitespace_backward(&prev_pos);
            }

            return start_of_run;
        }

        *pos
    }

    /// Convert a byte offset to a grapheme offset
    fn byte_to_grapheme_offset(&self, text: &str, byte_offset: usize) -> usize {
        let mut grapheme_count = 0;
        let mut current_byte = 0;

        for grapheme in text.graphemes(true) {
            if current_byte >= byte_offset {
                break;
            }
            current_byte += grapheme.len();
            grapheme_count += 1;
        }

        grapheme_count
    }

    /// Skip whitespace forward
    fn skip_whitespace_forward(&self, pos: &Position) -> Position {
        if let Some(run) = self.tree.get_run(pos.node_id) {
            let text = &run.text;
            let graphemes: Vec<&str> = text.graphemes(true).collect();

            let mut offset = pos.offset;
            while offset < graphemes.len() {
                let grapheme = graphemes[offset];
                if !grapheme.chars().all(char::is_whitespace) {
                    break;
                }
                offset += 1;
            }

            if offset < graphemes.len() {
                return Position::new(pos.node_id, offset);
            }

            // End of run, try next
            if let Some(next_pos) = self.next_text_position(&Position::new(pos.node_id, graphemes.len())) {
                return self.skip_whitespace_forward(&next_pos);
            }
        }

        *pos
    }

    /// Skip whitespace backward
    fn skip_whitespace_backward(&self, pos: &Position) -> Position {
        if let Some(run) = self.tree.get_run(pos.node_id) {
            let text = &run.text;
            let graphemes: Vec<&str> = text.graphemes(true).collect();

            let mut offset = pos.offset;
            while offset > 0 {
                let grapheme = graphemes[offset - 1];
                if !grapheme.chars().all(char::is_whitespace) {
                    break;
                }
                offset -= 1;
            }

            if offset > 0 {
                return Position::new(pos.node_id, offset);
            }

            // Start of run, try previous
            if let Some(prev_pos) = self.prev_text_position(&Position::new(pos.node_id, 0)) {
                return self.skip_whitespace_backward(&prev_pos);
            }
        }

        *pos
    }

    // ========================================================================
    // Paragraph Navigation
    // ========================================================================

    /// Move by one paragraph in the specified direction
    fn move_by_paragraph(&self, pos: &Position, direction: Direction) -> Position {
        let para_id = self.get_containing_paragraph(pos);

        match direction {
            Direction::Forward => {
                if let Some(para_id) = para_id {
                    if let Some(next_pos) = self.next_paragraph_start(para_id) {
                        return next_pos;
                    }
                    // At last paragraph, go to end
                    return self.get_paragraph_end(para_id).unwrap_or(*pos);
                }
            }
            Direction::Backward => {
                if let Some(para_id) = para_id {
                    // First, go to start of current paragraph
                    let para_start = self.get_paragraph_start(para_id);
                    if let Some(start_pos) = para_start {
                        // If not at start, go to start
                        if !self.positions_equal(pos, &start_pos) {
                            return start_pos;
                        }
                        // If at start, go to previous paragraph
                        if let Some(prev_end) = self.prev_paragraph_end(para_id) {
                            let prev_para_id = self.get_containing_paragraph(&prev_end);
                            if let Some(prev_id) = prev_para_id {
                                return self.get_paragraph_start(prev_id).unwrap_or(prev_end);
                            }
                        }
                    }
                }
            }
        }

        *pos
    }

    /// Get the paragraph containing a position
    fn get_containing_paragraph(&self, pos: &Position) -> Option<NodeId> {
        let node_type = self.tree.node_type(pos.node_id)?;

        match node_type {
            NodeType::Paragraph => Some(pos.node_id),
            NodeType::Run => {
                let run = self.tree.get_run(pos.node_id)?;
                run.parent()
            }
            _ => None,
        }
    }

    /// Get the start position of a paragraph
    fn get_paragraph_start(&self, para_id: NodeId) -> Option<Position> {
        let para = self.tree.get_paragraph(para_id)?;
        if let Some(&first_run_id) = para.children().first() {
            Some(Position::new(first_run_id, 0))
        } else {
            Some(Position::new(para_id, 0))
        }
    }

    /// Get the end position of a paragraph
    fn get_paragraph_end(&self, para_id: NodeId) -> Option<Position> {
        let para = self.tree.get_paragraph(para_id)?;
        if let Some(&last_run_id) = para.children().last() {
            let run = self.tree.get_run(last_run_id)?;
            let grapheme_count = run.text.graphemes(true).count();
            Some(Position::new(last_run_id, grapheme_count))
        } else {
            Some(Position::new(para_id, 0))
        }
    }

    /// Check if two positions are equal (considering run boundaries)
    fn positions_equal(&self, a: &Position, b: &Position) -> bool {
        if a.node_id == b.node_id && a.offset == b.offset {
            return true;
        }

        // Also consider equivalent positions at run boundaries
        // e.g., end of run N == start of run N+1
        false // Simplified for now
    }

    // ========================================================================
    // Line Boundary Navigation
    // ========================================================================

    /// Move to line boundary (Home/End keys)
    fn move_to_line_boundary(&self, pos: &Position, direction: Direction) -> Position {
        // If we have layout info, use visual line boundaries
        if let Some(layout) = self.layout {
            if let Some((start, end)) = layout.line_boundaries(pos, self.tree) {
                return match direction {
                    Direction::Backward => start,
                    Direction::Forward => end,
                };
            }
        }

        // Without layout info, use paragraph boundaries
        if let Some(para_id) = self.get_containing_paragraph(pos) {
            match direction {
                Direction::Backward => self.get_paragraph_start(para_id).unwrap_or(*pos),
                Direction::Forward => self.get_paragraph_end(para_id).unwrap_or(*pos),
            }
        } else {
            *pos
        }
    }

    // ========================================================================
    // Document Boundary Navigation
    // ========================================================================

    /// Move to document boundary (Ctrl+Home/End)
    fn move_to_document_boundary(&self, direction: Direction) -> Position {
        let doc_children = self.tree.document.children();

        match direction {
            Direction::Backward => {
                // Go to start of first paragraph
                if let Some(&first_para_id) = doc_children.first() {
                    self.get_paragraph_start(first_para_id)
                        .unwrap_or(Position::new(first_para_id, 0))
                } else {
                    Position::new(self.tree.root_id(), 0)
                }
            }
            Direction::Forward => {
                // Go to end of last paragraph
                if let Some(&last_para_id) = doc_children.last() {
                    self.get_paragraph_end(last_para_id)
                        .unwrap_or(Position::new(last_para_id, 0))
                } else {
                    Position::new(self.tree.root_id(), 0)
                }
            }
        }
    }
}

// ============================================================================
// Selection Extension Helpers
// ============================================================================

/// Extension methods for Selection related to navigation
pub trait SelectionNavigation {
    /// Move the selection in the given direction
    fn navigate(
        &self,
        tree: &DocumentTree,
        direction: Direction,
        unit: MovementUnit,
        extend: bool,
    ) -> Selection;

    /// Move the selection with layout information
    fn navigate_with_layout(
        &self,
        tree: &DocumentTree,
        layout: &dyn LayoutProvider,
        direction: Direction,
        unit: MovementUnit,
        extend: bool,
        visual_x: Option<f32>,
    ) -> NavigationResult;
}

impl SelectionNavigation for Selection {
    fn navigate(
        &self,
        tree: &DocumentTree,
        direction: Direction,
        unit: MovementUnit,
        extend: bool,
    ) -> Selection {
        let navigator = Navigator::new(tree);
        let result = navigator.navigate(
            self,
            direction,
            unit,
            NavigationOptions {
                extend_selection: extend,
                visual_x: None,
            },
        );
        result.selection
    }

    fn navigate_with_layout(
        &self,
        tree: &DocumentTree,
        layout: &dyn LayoutProvider,
        direction: Direction,
        unit: MovementUnit,
        extend: bool,
        visual_x: Option<f32>,
    ) -> NavigationResult {
        let navigator = Navigator::with_layout(tree, layout);
        navigator.navigate(
            self,
            direction,
            unit,
            NavigationOptions {
                extend_selection: extend,
                visual_x,
            },
        )
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Get the linear offset of a position in the document
/// Useful for comparing positions across different nodes
pub fn position_to_linear_offset(tree: &DocumentTree, pos: &Position) -> Option<usize> {
    let mut offset = 0;

    for para in tree.paragraphs() {
        for &run_id in para.children() {
            if run_id == pos.node_id {
                return Some(offset + pos.offset);
            }
            if let Some(run) = tree.get_run(run_id) {
                offset += run.text.graphemes(true).count();
            }
        }
        // Account for paragraph break
        offset += 1;

        if para.id() == pos.node_id {
            return Some(offset);
        }
    }

    None
}

/// Compare two positions in document order
/// Returns Ordering::Less if a comes before b, etc.
pub fn compare_positions(
    tree: &DocumentTree,
    a: &Position,
    b: &Position,
) -> std::cmp::Ordering {
    if a.node_id == b.node_id {
        return a.offset.cmp(&b.offset);
    }

    let a_offset = position_to_linear_offset(tree, a);
    let b_offset = position_to_linear_offset(tree, b);

    match (a_offset, b_offset) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

/// Check if a position is within a selection
pub fn position_in_selection(tree: &DocumentTree, pos: &Position, sel: &Selection) -> bool {
    let pos_offset = match position_to_linear_offset(tree, pos) {
        Some(o) => o,
        None => return false,
    };

    let start_offset = position_to_linear_offset(tree, &sel.start());
    let end_offset = position_to_linear_offset(tree, &sel.end());

    match (start_offset, end_offset) {
        (Some(start), Some(end)) => pos_offset >= start && pos_offset <= end,
        _ => false,
    }
}

// ============================================================================
// Visual Cursor Movement for BiDi Text
// ============================================================================

/// Cursor movement mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorMovement {
    /// Follow logical text order (for Ctrl+Arrow)
    #[default]
    Logical,
    /// Follow visual display order (for Arrow keys in BiDi text)
    Visual,
}

/// Cursor affinity - which side of a BiDi boundary the cursor prefers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorAffinity {
    /// Cursor prefers the leading edge (start of the character)
    #[default]
    Leading,
    /// Cursor prefers the trailing edge (end of the character)
    Trailing,
}

/// Information about a visual run in a line
#[derive(Debug, Clone)]
pub struct VisualRunInfo {
    /// Start position (logical) in the paragraph
    pub logical_start: usize,
    /// End position (logical) in the paragraph
    pub logical_end: usize,
    /// Start position (visual X coordinate)
    pub visual_start: f32,
    /// End position (visual X coordinate)
    pub visual_end: f32,
    /// BiDi level (even = LTR, odd = RTL)
    pub bidi_level: u8,
    /// Run index in visual order
    pub visual_index: usize,
}

impl VisualRunInfo {
    /// Check if this run is RTL
    pub fn is_rtl(&self) -> bool {
        self.bidi_level % 2 == 1
    }

    /// Check if this run is LTR
    pub fn is_ltr(&self) -> bool {
        self.bidi_level % 2 == 0
    }
}

/// A line with visual run information for BiDi navigation
#[derive(Debug, Clone)]
pub struct VisualLine {
    /// Visual runs in visual (display) order
    pub runs: Vec<VisualRunInfo>,
    /// Line bounds
    pub bounds: (f32, f32, f32, f32), // x, y, width, height
    /// Base direction of the line
    pub base_direction_rtl: bool,
}

impl VisualLine {
    /// Find the run containing a logical position
    pub fn run_for_logical_position(&self, logical_pos: usize) -> Option<&VisualRunInfo> {
        self.runs.iter().find(|r| logical_pos >= r.logical_start && logical_pos <= r.logical_end)
    }

    /// Find the run at a visual X position
    pub fn run_at_visual_x(&self, x: f32) -> Option<&VisualRunInfo> {
        self.runs.iter().find(|r| {
            let left = r.visual_start.min(r.visual_end);
            let right = r.visual_start.max(r.visual_end);
            x >= left && x <= right
        })
    }

    /// Get the leftmost run
    pub fn leftmost_run(&self) -> Option<&VisualRunInfo> {
        self.runs.iter().min_by(|a, b| {
            let a_left = a.visual_start.min(a.visual_end);
            let b_left = b.visual_start.min(b.visual_end);
            a_left.partial_cmp(&b_left).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Get the rightmost run
    pub fn rightmost_run(&self) -> Option<&VisualRunInfo> {
        self.runs.iter().max_by(|a, b| {
            let a_right = a.visual_start.max(a.visual_end);
            let b_right = b.visual_start.max(b.visual_end);
            a_right.partial_cmp(&b_right).unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

/// Trait for BiDi-aware layout information
pub trait BiDiLayoutProvider: LayoutProvider {
    /// Get visual run information for a line at the given position
    fn get_visual_line(&self, pos: &Position, tree: &DocumentTree) -> Option<VisualLine>;

    /// Convert a logical position to a visual X coordinate
    fn logical_to_visual_x(&self, pos: &Position, tree: &DocumentTree) -> Option<f32>;

    /// Convert a visual X coordinate to a logical position
    /// Returns the position and the affinity (which side of the character)
    fn visual_x_to_logical(
        &self,
        x: f32,
        y: f32,
        tree: &DocumentTree,
    ) -> Option<(Position, CursorAffinity)>;

    /// Get the base direction of a paragraph
    fn paragraph_base_direction(&self, para_id: NodeId, tree: &DocumentTree) -> bool;
}

/// Navigator with BiDi support for visual cursor movement
pub struct BiDiNavigator<'a> {
    tree: &'a DocumentTree,
    layout: Option<&'a dyn BiDiLayoutProvider>,
}

impl<'a> BiDiNavigator<'a> {
    /// Create a new BiDi navigator
    pub fn new(tree: &'a DocumentTree) -> Self {
        Self { tree, layout: None }
    }

    /// Create a BiDi navigator with layout information
    pub fn with_layout(tree: &'a DocumentTree, layout: &'a dyn BiDiLayoutProvider) -> Self {
        Self {
            tree,
            layout: Some(layout),
        }
    }

    /// Navigate visually left (moves cursor visually leftward, may cross BiDi boundaries)
    pub fn move_visual_left(
        &self,
        pos: &Position,
        affinity: CursorAffinity,
    ) -> (Position, CursorAffinity) {
        let layout = match self.layout {
            Some(l) => l,
            None => return (*pos, affinity),
        };

        let visual_line = match layout.get_visual_line(pos, self.tree) {
            Some(l) => l,
            None => return (*pos, affinity),
        };

        let current_run = match visual_line.run_for_logical_position(pos.offset) {
            Some(r) => r,
            None => return (*pos, affinity),
        };

        // If current run is LTR, moving left means moving to earlier logical position
        // If current run is RTL, moving left means moving to later logical position
        if current_run.is_ltr() {
            // LTR: visual left = logical backward
            if pos.offset > current_run.logical_start {
                // Move within the run
                return (Position::new(pos.node_id, pos.offset - 1), CursorAffinity::Trailing);
            } else {
                // At start of LTR run, need to jump to the run visually to the left
                return self.jump_to_adjacent_run(&visual_line, current_run, true);
            }
        } else {
            // RTL: visual left = logical forward
            if pos.offset < current_run.logical_end {
                return (Position::new(pos.node_id, pos.offset + 1), CursorAffinity::Leading);
            } else {
                // At end of RTL run, need to jump to the run visually to the left
                return self.jump_to_adjacent_run(&visual_line, current_run, true);
            }
        }
    }

    /// Navigate visually right (moves cursor visually rightward, may cross BiDi boundaries)
    pub fn move_visual_right(
        &self,
        pos: &Position,
        affinity: CursorAffinity,
    ) -> (Position, CursorAffinity) {
        let layout = match self.layout {
            Some(l) => l,
            None => return (*pos, affinity),
        };

        let visual_line = match layout.get_visual_line(pos, self.tree) {
            Some(l) => l,
            None => return (*pos, affinity),
        };

        let current_run = match visual_line.run_for_logical_position(pos.offset) {
            Some(r) => r,
            None => return (*pos, affinity),
        };

        // If current run is LTR, moving right means moving to later logical position
        // If current run is RTL, moving right means moving to earlier logical position
        if current_run.is_ltr() {
            // LTR: visual right = logical forward
            if pos.offset < current_run.logical_end {
                return (Position::new(pos.node_id, pos.offset + 1), CursorAffinity::Leading);
            } else {
                // At end of LTR run, need to jump to the run visually to the right
                return self.jump_to_adjacent_run(&visual_line, current_run, false);
            }
        } else {
            // RTL: visual right = logical backward
            if pos.offset > current_run.logical_start {
                return (Position::new(pos.node_id, pos.offset - 1), CursorAffinity::Trailing);
            } else {
                // At start of RTL run, need to jump to the run visually to the right
                return self.jump_to_adjacent_run(&visual_line, current_run, false);
            }
        }
    }

    /// Jump to the adjacent run in visual order
    fn jump_to_adjacent_run(
        &self,
        line: &VisualLine,
        current_run: &VisualRunInfo,
        go_left: bool,
    ) -> (Position, CursorAffinity) {
        let current_visual_idx = current_run.visual_index;

        let target_idx = if go_left {
            if current_visual_idx == 0 {
                // At leftmost run, can't go further left on this line
                // Could move to previous line here in a full implementation
                return (
                    Position::new(NodeId::new(), current_run.logical_start),
                    CursorAffinity::Leading,
                );
            }
            current_visual_idx - 1
        } else {
            if current_visual_idx >= line.runs.len() - 1 {
                // At rightmost run, can't go further right on this line
                return (
                    Position::new(NodeId::new(), current_run.logical_end),
                    CursorAffinity::Trailing,
                );
            }
            current_visual_idx + 1
        };

        if let Some(target_run) = line.runs.iter().find(|r| r.visual_index == target_idx) {
            // When entering a run from left, we land at the visually leftmost position
            // When entering a run from right, we land at the visually rightmost position
            let (pos_offset, affinity) = if go_left {
                // Entering from the right
                if target_run.is_ltr() {
                    (target_run.logical_end, CursorAffinity::Trailing)
                } else {
                    (target_run.logical_start, CursorAffinity::Leading)
                }
            } else {
                // Entering from the left
                if target_run.is_ltr() {
                    (target_run.logical_start, CursorAffinity::Leading)
                } else {
                    (target_run.logical_end, CursorAffinity::Trailing)
                }
            };

            return (Position::new(NodeId::new(), pos_offset), affinity);
        }

        (
            Position::new(NodeId::new(), current_run.logical_start),
            CursorAffinity::default(),
        )
    }

    /// Move to visual line start (Home key behavior)
    pub fn move_to_visual_line_start(&self, pos: &Position) -> Position {
        let layout = match self.layout {
            Some(l) => l,
            None => return *pos,
        };

        let visual_line = match layout.get_visual_line(pos, self.tree) {
            Some(l) => l,
            None => return *pos,
        };

        // Find the leftmost run and return its visual start position
        if let Some(leftmost) = visual_line.leftmost_run() {
            if leftmost.is_ltr() {
                return Position::new(pos.node_id, leftmost.logical_start);
            } else {
                return Position::new(pos.node_id, leftmost.logical_end);
            }
        }

        *pos
    }

    /// Move to visual line end (End key behavior)
    pub fn move_to_visual_line_end(&self, pos: &Position) -> Position {
        let layout = match self.layout {
            Some(l) => l,
            None => return *pos,
        };

        let visual_line = match layout.get_visual_line(pos, self.tree) {
            Some(l) => l,
            None => return *pos,
        };

        // Find the rightmost run and return its visual end position
        if let Some(rightmost) = visual_line.rightmost_run() {
            if rightmost.is_ltr() {
                return Position::new(pos.node_id, rightmost.logical_end);
            } else {
                return Position::new(pos.node_id, rightmost.logical_start);
            }
        }

        *pos
    }
}

/// Visual direction for arrow keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualDirection {
    /// Move visually left
    Left,
    /// Move visually right
    Right,
}

/// Extended navigation options with BiDi support
#[derive(Debug, Clone, Copy, Default)]
pub struct BiDiNavigationOptions {
    /// Whether to extend the selection
    pub extend_selection: bool,
    /// Visual X position for vertical navigation
    pub visual_x: Option<f32>,
    /// Cursor movement mode (logical vs visual)
    pub movement_mode: CursorMovement,
    /// Current cursor affinity
    pub affinity: CursorAffinity,
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::{Paragraph, Run};

    fn create_test_document() -> DocumentTree {
        let mut tree = DocumentTree::new();

        // Create first paragraph with two runs
        let para1 = Paragraph::new();
        let para1_id = para1.id();
        tree.nodes.paragraphs.insert(para1_id, para1);
        tree.document.add_body_child(para1_id);

        let run1 = Run::new("Hello ");
        tree.insert_run(run1, para1_id, None).unwrap();

        let run2 = Run::new("world!");
        tree.insert_run(run2, para1_id, None).unwrap();

        // Create second paragraph
        let para2 = Paragraph::new();
        let para2_id = para2.id();
        tree.nodes.paragraphs.insert(para2_id, para2);
        tree.document.add_body_child(para2_id);

        let run3 = Run::new("Second paragraph.");
        tree.insert_run(run3, para2_id, None).unwrap();

        tree
    }

    #[test]
    fn test_character_forward() {
        let tree = create_test_document();
        let navigator = Navigator::new(&tree);

        // Get first run
        let first_para_id = tree.document.children()[0];
        let first_para = tree.get_paragraph(first_para_id).unwrap();
        let first_run_id = first_para.children()[0];

        let pos = Position::new(first_run_id, 0);
        let sel = Selection::collapsed(pos);

        let result = navigator.navigate(
            &sel,
            Direction::Forward,
            MovementUnit::Character,
            NavigationOptions::default(),
        );

        assert_eq!(result.selection.focus.offset, 1);
    }

    #[test]
    fn test_character_backward() {
        let tree = create_test_document();
        let navigator = Navigator::new(&tree);

        let first_para_id = tree.document.children()[0];
        let first_para = tree.get_paragraph(first_para_id).unwrap();
        let first_run_id = first_para.children()[0];

        let pos = Position::new(first_run_id, 3);
        let sel = Selection::collapsed(pos);

        let result = navigator.navigate(
            &sel,
            Direction::Backward,
            MovementUnit::Character,
            NavigationOptions::default(),
        );

        assert_eq!(result.selection.focus.offset, 2);
    }

    #[test]
    fn test_word_forward() {
        let tree = create_test_document();
        let navigator = Navigator::new(&tree);

        let first_para_id = tree.document.children()[0];
        let first_para = tree.get_paragraph(first_para_id).unwrap();
        let first_run_id = first_para.children()[0];

        let pos = Position::new(first_run_id, 0);
        let sel = Selection::collapsed(pos);

        let result = navigator.navigate(
            &sel,
            Direction::Forward,
            MovementUnit::Word,
            NavigationOptions::default(),
        );

        // Should move past "Hello " to next word
        assert!(result.selection.focus.offset > 0);
    }

    #[test]
    fn test_selection_extend() {
        let tree = create_test_document();
        let navigator = Navigator::new(&tree);

        let first_para_id = tree.document.children()[0];
        let first_para = tree.get_paragraph(first_para_id).unwrap();
        let first_run_id = first_para.children()[0];

        let pos = Position::new(first_run_id, 0);
        let sel = Selection::collapsed(pos);

        let result = navigator.navigate(
            &sel,
            Direction::Forward,
            MovementUnit::Character,
            NavigationOptions {
                extend_selection: true,
                visual_x: None,
            },
        );

        // Anchor should stay, focus should move
        assert_eq!(result.selection.anchor.offset, 0);
        assert_eq!(result.selection.focus.offset, 1);
        assert!(!result.selection.is_collapsed());
    }

    #[test]
    fn test_paragraph_navigation() {
        let tree = create_test_document();
        let navigator = Navigator::new(&tree);

        let first_para_id = tree.document.children()[0];
        let first_para = tree.get_paragraph(first_para_id).unwrap();
        let first_run_id = first_para.children()[0];

        let pos = Position::new(first_run_id, 2);
        let sel = Selection::collapsed(pos);

        let result = navigator.navigate(
            &sel,
            Direction::Forward,
            MovementUnit::Paragraph,
            NavigationOptions::default(),
        );

        // Should be in second paragraph
        let second_para_id = tree.document.children()[1];
        let second_para = tree.get_paragraph(second_para_id).unwrap();
        let second_run_id = second_para.children()[0];

        assert_eq!(result.selection.focus.node_id, second_run_id);
    }

    #[test]
    fn test_document_boundary() {
        let tree = create_test_document();
        let navigator = Navigator::new(&tree);

        let first_para_id = tree.document.children()[0];
        let first_para = tree.get_paragraph(first_para_id).unwrap();
        let first_run_id = first_para.children()[0];

        let pos = Position::new(first_run_id, 3);
        let sel = Selection::collapsed(pos);

        // Go to end of document
        let result = navigator.navigate(
            &sel,
            Direction::Forward,
            MovementUnit::DocumentBoundary,
            NavigationOptions::default(),
        );

        // Should be at end of last paragraph
        let last_para_id = *tree.document.children().last().unwrap();
        let last_para = tree.get_paragraph(last_para_id).unwrap();
        let last_run_id = *last_para.children().last().unwrap();
        let last_run = tree.get_run(last_run_id).unwrap();

        assert_eq!(result.selection.focus.node_id, last_run_id);
        assert_eq!(
            result.selection.focus.offset,
            last_run.text.graphemes(true).count()
        );
    }

    #[test]
    fn test_unicode_grapheme_navigation() {
        let mut tree = DocumentTree::new();

        // Create paragraph with emoji (multi-codepoint grapheme)
        let para = Paragraph::new();
        let para_id = para.id();
        tree.nodes.paragraphs.insert(para_id, para);
        tree.document.add_body_child(para_id);

        // Family emoji is a single grapheme but multiple codepoints
        let run = Run::new("Hello \u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467} world");
        tree.insert_run(run, para_id, None).unwrap();

        let navigator = Navigator::new(&tree);

        let para = tree.get_paragraph(para_id).unwrap();
        let run_id = para.children()[0];

        // Start after "Hello "
        let pos = Position::new(run_id, 6);
        let sel = Selection::collapsed(pos);

        // Move one character forward (should skip the entire family emoji)
        let result = navigator.navigate(
            &sel,
            Direction::Forward,
            MovementUnit::Character,
            NavigationOptions::default(),
        );

        // Should be at position 7 (after the emoji grapheme)
        assert_eq!(result.selection.focus.offset, 7);
    }

    // =========================================================================
    // BiDi Navigation Tests
    // =========================================================================

    #[test]
    fn test_visual_run_info_direction() {
        let ltr_run = VisualRunInfo {
            logical_start: 0,
            logical_end: 10,
            visual_start: 0.0,
            visual_end: 100.0,
            bidi_level: 0,
            visual_index: 0,
        };
        assert!(ltr_run.is_ltr());
        assert!(!ltr_run.is_rtl());

        let rtl_run = VisualRunInfo {
            logical_start: 0,
            logical_end: 10,
            visual_start: 100.0,
            visual_end: 0.0,
            bidi_level: 1,
            visual_index: 0,
        };
        assert!(rtl_run.is_rtl());
        assert!(!rtl_run.is_ltr());
    }

    #[test]
    fn test_visual_line_leftmost_rightmost() {
        let runs = vec![
            VisualRunInfo {
                logical_start: 10,
                logical_end: 20,
                visual_start: 0.0,
                visual_end: 50.0,
                bidi_level: 0,
                visual_index: 0,
            },
            VisualRunInfo {
                logical_start: 0,
                logical_end: 10,
                visual_start: 100.0,
                visual_end: 50.0,
                bidi_level: 1,
                visual_index: 1,
            },
        ];

        let line = VisualLine {
            runs,
            bounds: (0.0, 0.0, 100.0, 14.0),
            base_direction_rtl: false,
        };

        let leftmost = line.leftmost_run().unwrap();
        assert_eq!(leftmost.logical_start, 10);
        assert_eq!(leftmost.visual_index, 0);

        let rightmost = line.rightmost_run().unwrap();
        assert_eq!(rightmost.logical_start, 0);
        assert_eq!(rightmost.visual_index, 1);
    }

    #[test]
    fn test_cursor_affinity() {
        assert_eq!(CursorAffinity::default(), CursorAffinity::Leading);
    }

    #[test]
    fn test_cursor_movement_mode() {
        assert_eq!(CursorMovement::default(), CursorMovement::Logical);
    }

    #[test]
    fn test_bidi_navigation_options_default() {
        let opts = BiDiNavigationOptions::default();
        assert!(!opts.extend_selection);
        assert!(opts.visual_x.is_none());
        assert_eq!(opts.movement_mode, CursorMovement::Logical);
        assert_eq!(opts.affinity, CursorAffinity::Leading);
    }
}
