//! Paragraph formatting commands for setting alignment, indentation, spacing, and borders

use crate::{Command, CommandResult, EditError, Result};
use doc_model::{
    Alignment, DocumentTree, LineSpacing, Node, NodeId, NodeType,
    ParagraphBorders, ParagraphProperties, Position, Selection,
    style::{BorderStyle, BorderStyleType},
};
use serde::{Deserialize, Serialize};

// =============================================================================
// Helper Functions
// =============================================================================

/// Get the paragraph ID for a position
fn get_paragraph_for_position(tree: &DocumentTree, position: &Position) -> Result<NodeId> {
    let node_type = tree.node_type(position.node_id)
        .ok_or_else(|| EditError::InvalidCommand(
            format!("Node not found: {:?}", position.node_id)
        ))?;

    match node_type {
        NodeType::Paragraph => Ok(position.node_id),
        NodeType::Run => {
            let run = tree.get_run(position.node_id)
                .ok_or_else(|| EditError::InvalidCommand(
                    format!("Run not found: {:?}", position.node_id)
                ))?;

            // Parent could be a paragraph or hyperlink
            let parent_id = run.parent()
                .ok_or_else(|| EditError::InvalidCommand(
                    "Run has no parent".to_string()
                ))?;

            if tree.get_paragraph(parent_id).is_some() {
                return Ok(parent_id);
            }

            // Parent is a hyperlink
            if let Some(hyperlink) = tree.get_hyperlink(parent_id) {
                return hyperlink.parent()
                    .ok_or_else(|| EditError::InvalidCommand(
                        "Hyperlink has no parent".to_string()
                    ));
            }

            Err(EditError::InvalidCommand("Cannot determine paragraph".to_string()))
        }
        NodeType::Hyperlink => {
            let hyperlink = tree.get_hyperlink(position.node_id)
                .ok_or_else(|| EditError::InvalidCommand(
                    format!("Hyperlink not found: {:?}", position.node_id)
                ))?;

            hyperlink.parent()
                .ok_or_else(|| EditError::InvalidCommand(
                    "Hyperlink has no parent".to_string()
                ))
        }
        _ => Err(EditError::InvalidCommand(
            format!("Invalid node type for position: {:?}", node_type)
        )),
    }
}

/// Get all paragraph IDs in a selection range
fn get_paragraphs_in_selection(tree: &DocumentTree, selection: &Selection) -> Result<Vec<NodeId>> {
    let start_para = get_paragraph_for_position(tree, &selection.start())?;
    let end_para = get_paragraph_for_position(tree, &selection.end())?;

    if start_para == end_para {
        return Ok(vec![start_para]);
    }

    // Get all paragraphs between start and end
    let mut paragraphs = Vec::new();
    let mut found_start = false;

    for para in tree.paragraphs() {
        let para_id = para.id();
        if para_id == start_para {
            found_start = true;
        }
        if found_start {
            paragraphs.push(para_id);
        }
        if para_id == end_para {
            break;
        }
    }

    Ok(paragraphs)
}

// =============================================================================
// Set Paragraph Alignment Command
// =============================================================================

/// Set paragraph alignment (left, center, right, justify)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParagraphAlignment {
    /// The new alignment
    pub alignment: Alignment,
}

impl SetParagraphAlignment {
    pub fn new(alignment: Alignment) -> Self {
        Self { alignment }
    }

    pub fn left() -> Self {
        Self::new(Alignment::Left)
    }

    pub fn center() -> Self {
        Self::new(Alignment::Center)
    }

    pub fn right() -> Self {
        Self::new(Alignment::Right)
    }

    pub fn justify() -> Self {
        Self::new(Alignment::Justify)
    }
}

impl Command for SetParagraphAlignment {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        let paragraphs = get_paragraphs_in_selection(&new_tree, selection)?;

        // Store old alignments for undo
        let old_alignments: Vec<(NodeId, Option<Alignment>)> = paragraphs
            .iter()
            .filter_map(|&para_id| {
                new_tree.get_paragraph(para_id)
                    .map(|p| (para_id, p.direct_formatting.alignment))
            })
            .collect();

        // Apply new alignment to all paragraphs
        for &para_id in &paragraphs {
            if let Some(para) = new_tree.get_paragraph_mut(para_id) {
                para.direct_formatting.alignment = Some(self.alignment);
                // Also update legacy style for compatibility
                para.style.alignment = Some(self.alignment);
            }
        }

        let inverse = Box::new(RestoreParagraphAlignments {
            alignments: old_alignments,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        // Proper inverse created in apply()
        Box::new(SetParagraphAlignment::new(Alignment::Left))
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        match self.alignment {
            Alignment::Left => "Align Left",
            Alignment::Center => "Center",
            Alignment::Right => "Align Right",
            Alignment::Justify => "Justify",
        }
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Restore paragraph alignments (for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestoreParagraphAlignments {
    alignments: Vec<(NodeId, Option<Alignment>)>,
}

impl Command for RestoreParagraphAlignments {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Store current alignments for redo
        let current_alignments: Vec<(NodeId, Option<Alignment>)> = self.alignments
            .iter()
            .filter_map(|(para_id, _)| {
                new_tree.get_paragraph(*para_id)
                    .map(|p| (*para_id, p.direct_formatting.alignment))
            })
            .collect();

        // Restore old alignments
        for (para_id, alignment) in &self.alignments {
            if let Some(para) = new_tree.get_paragraph_mut(*para_id) {
                para.direct_formatting.alignment = *alignment;
                para.style.alignment = *alignment;
            }
        }

        let inverse = Box::new(RestoreParagraphAlignments {
            alignments: current_alignments,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(RestoreParagraphAlignments {
            alignments: self.alignments.clone(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Restore Alignment"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Set Paragraph Indent Command
// =============================================================================

/// Set paragraph indentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParagraphIndent {
    /// Left indent in points (None = don't change)
    pub left: Option<f32>,
    /// Right indent in points (None = don't change)
    pub right: Option<f32>,
    /// First line indent in points (None = don't change, negative = hanging)
    pub first_line: Option<f32>,
}

impl SetParagraphIndent {
    pub fn new(left: Option<f32>, right: Option<f32>, first_line: Option<f32>) -> Self {
        Self { left, right, first_line }
    }

    /// Increase left indent by a standard amount (36 points = 0.5 inch)
    pub fn increase_indent() -> Self {
        Self {
            left: Some(36.0),
            right: None,
            first_line: None,
        }
    }

    /// Decrease left indent by a standard amount
    pub fn decrease_indent() -> Self {
        Self {
            left: Some(-36.0),
            right: None,
            first_line: None,
        }
    }

    /// Set first line indent
    pub fn first_line_indent(indent: f32) -> Self {
        Self {
            left: None,
            right: None,
            first_line: Some(indent),
        }
    }

    /// Set hanging indent (negative first line)
    pub fn hanging_indent(indent: f32) -> Self {
        Self {
            left: None,
            right: None,
            first_line: Some(-indent.abs()),
        }
    }
}

impl Command for SetParagraphIndent {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        let paragraphs = get_paragraphs_in_selection(&new_tree, selection)?;

        // Store old indents for undo
        let old_indents: Vec<(NodeId, Option<f32>, Option<f32>, Option<f32>)> = paragraphs
            .iter()
            .filter_map(|&para_id| {
                new_tree.get_paragraph(para_id).map(|p| (
                    para_id,
                    p.direct_formatting.indent_left,
                    p.direct_formatting.indent_right,
                    p.direct_formatting.indent_first_line,
                ))
            })
            .collect();

        // Apply new indentation to all paragraphs
        for &para_id in &paragraphs {
            if let Some(para) = new_tree.get_paragraph_mut(para_id) {
                if let Some(left) = self.left {
                    // Handle relative increase/decrease
                    let current = para.direct_formatting.indent_left.unwrap_or(0.0);
                    let new_value = (current + left).max(0.0);
                    para.direct_formatting.indent_left = Some(new_value);
                    para.style.indent_left = Some(new_value);
                }
                if let Some(right) = self.right {
                    let current = para.direct_formatting.indent_right.unwrap_or(0.0);
                    let new_value = (current + right).max(0.0);
                    para.direct_formatting.indent_right = Some(new_value);
                    para.style.indent_right = Some(new_value);
                }
                if let Some(first_line) = self.first_line {
                    para.direct_formatting.indent_first_line = Some(first_line);
                    para.style.indent_first_line = Some(first_line);
                }
            }
        }

        let inverse = Box::new(RestoreParagraphIndents {
            indents: old_indents,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SetParagraphIndent::new(None, None, None))
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Paragraph Indent"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Restore paragraph indents (for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestoreParagraphIndents {
    indents: Vec<(NodeId, Option<f32>, Option<f32>, Option<f32>)>,
}

impl Command for RestoreParagraphIndents {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Store current indents for redo
        let current_indents: Vec<(NodeId, Option<f32>, Option<f32>, Option<f32>)> = self.indents
            .iter()
            .filter_map(|(para_id, _, _, _)| {
                new_tree.get_paragraph(*para_id).map(|p| (
                    *para_id,
                    p.direct_formatting.indent_left,
                    p.direct_formatting.indent_right,
                    p.direct_formatting.indent_first_line,
                ))
            })
            .collect();

        // Restore old indents
        for (para_id, left, right, first_line) in &self.indents {
            if let Some(para) = new_tree.get_paragraph_mut(*para_id) {
                para.direct_formatting.indent_left = *left;
                para.direct_formatting.indent_right = *right;
                para.direct_formatting.indent_first_line = *first_line;
                para.style.indent_left = *left;
                para.style.indent_right = *right;
                para.style.indent_first_line = *first_line;
            }
        }

        let inverse = Box::new(RestoreParagraphIndents {
            indents: current_indents,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(RestoreParagraphIndents {
            indents: self.indents.clone(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Restore Indent"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Set Paragraph Spacing Command
// =============================================================================

/// Set paragraph spacing (before, after, line spacing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParagraphSpacing {
    /// Space before paragraph in points (None = don't change)
    pub before: Option<f32>,
    /// Space after paragraph in points (None = don't change)
    pub after: Option<f32>,
    /// Line spacing (None = don't change)
    pub line_spacing: Option<LineSpacing>,
}

impl SetParagraphSpacing {
    pub fn new(before: Option<f32>, after: Option<f32>, line_spacing: Option<LineSpacing>) -> Self {
        Self { before, after, line_spacing }
    }

    /// Set line spacing to single
    pub fn single_spacing() -> Self {
        Self {
            before: None,
            after: None,
            line_spacing: Some(LineSpacing::Multiple(1.0)),
        }
    }

    /// Set line spacing to 1.15
    pub fn spacing_1_15() -> Self {
        Self {
            before: None,
            after: None,
            line_spacing: Some(LineSpacing::Multiple(1.15)),
        }
    }

    /// Set line spacing to 1.5
    pub fn spacing_1_5() -> Self {
        Self {
            before: None,
            after: None,
            line_spacing: Some(LineSpacing::Multiple(1.5)),
        }
    }

    /// Set line spacing to double
    pub fn double_spacing() -> Self {
        Self {
            before: None,
            after: None,
            line_spacing: Some(LineSpacing::Multiple(2.0)),
        }
    }

    /// Set exact line spacing in points
    pub fn exact_spacing(points: f32) -> Self {
        Self {
            before: None,
            after: None,
            line_spacing: Some(LineSpacing::Exact(points)),
        }
    }

    /// Set minimum line spacing in points
    pub fn at_least_spacing(points: f32) -> Self {
        Self {
            before: None,
            after: None,
            line_spacing: Some(LineSpacing::AtLeast(points)),
        }
    }
}

impl Command for SetParagraphSpacing {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        let paragraphs = get_paragraphs_in_selection(&new_tree, selection)?;

        // Store old spacing for undo
        let old_spacing: Vec<(NodeId, Option<f32>, Option<f32>, Option<LineSpacing>)> = paragraphs
            .iter()
            .filter_map(|&para_id| {
                new_tree.get_paragraph(para_id).map(|p| (
                    para_id,
                    p.direct_formatting.space_before,
                    p.direct_formatting.space_after,
                    p.direct_formatting.line_spacing,
                ))
            })
            .collect();

        // Apply new spacing to all paragraphs
        for &para_id in &paragraphs {
            if let Some(para) = new_tree.get_paragraph_mut(para_id) {
                if let Some(before) = self.before {
                    para.direct_formatting.space_before = Some(before);
                    para.style.space_before = Some(before);
                }
                if let Some(after) = self.after {
                    para.direct_formatting.space_after = Some(after);
                    para.style.space_after = Some(after);
                }
                if let Some(line_spacing) = self.line_spacing {
                    para.direct_formatting.line_spacing = Some(line_spacing);
                    para.style.line_spacing = Some(line_spacing);
                }
            }
        }

        let inverse = Box::new(RestoreParagraphSpacing {
            spacing: old_spacing,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SetParagraphSpacing::new(None, None, None))
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Paragraph Spacing"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Restore paragraph spacing (for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestoreParagraphSpacing {
    spacing: Vec<(NodeId, Option<f32>, Option<f32>, Option<LineSpacing>)>,
}

impl Command for RestoreParagraphSpacing {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Store current spacing for redo
        let current_spacing: Vec<(NodeId, Option<f32>, Option<f32>, Option<LineSpacing>)> = self.spacing
            .iter()
            .filter_map(|(para_id, _, _, _)| {
                new_tree.get_paragraph(*para_id).map(|p| (
                    *para_id,
                    p.direct_formatting.space_before,
                    p.direct_formatting.space_after,
                    p.direct_formatting.line_spacing,
                ))
            })
            .collect();

        // Restore old spacing
        for (para_id, before, after, line_spacing) in &self.spacing {
            if let Some(para) = new_tree.get_paragraph_mut(*para_id) {
                para.direct_formatting.space_before = *before;
                para.direct_formatting.space_after = *after;
                para.direct_formatting.line_spacing = *line_spacing;
                para.style.space_before = *before;
                para.style.space_after = *after;
                para.style.line_spacing = *line_spacing;
            }
        }

        let inverse = Box::new(RestoreParagraphSpacing {
            spacing: current_spacing,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(RestoreParagraphSpacing {
            spacing: self.spacing.clone(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Restore Spacing"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Set Paragraph Pagination Options Command
// =============================================================================

/// Set paragraph pagination options (keep with next, keep together, page break before)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParagraphPagination {
    /// Keep with next paragraph (None = don't change)
    pub keep_with_next: Option<bool>,
    /// Keep lines together (None = don't change)
    pub keep_together: Option<bool>,
    /// Page break before (None = don't change)
    pub page_break_before: Option<bool>,
    /// Widow/orphan control (None = don't change)
    pub widow_control: Option<bool>,
}

impl SetParagraphPagination {
    pub fn new(
        keep_with_next: Option<bool>,
        keep_together: Option<bool>,
        page_break_before: Option<bool>,
        widow_control: Option<bool>,
    ) -> Self {
        Self { keep_with_next, keep_together, page_break_before, widow_control }
    }

    pub fn keep_with_next(value: bool) -> Self {
        Self {
            keep_with_next: Some(value),
            keep_together: None,
            page_break_before: None,
            widow_control: None,
        }
    }

    pub fn keep_together(value: bool) -> Self {
        Self {
            keep_with_next: None,
            keep_together: Some(value),
            page_break_before: None,
            widow_control: None,
        }
    }

    pub fn page_break_before(value: bool) -> Self {
        Self {
            keep_with_next: None,
            keep_together: None,
            page_break_before: Some(value),
            widow_control: None,
        }
    }
}

impl Command for SetParagraphPagination {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        let paragraphs = get_paragraphs_in_selection(&new_tree, selection)?;

        // Store old pagination options for undo
        let old_options: Vec<(NodeId, Option<bool>, Option<bool>, Option<bool>, Option<bool>)> = paragraphs
            .iter()
            .filter_map(|&para_id| {
                new_tree.get_paragraph(para_id).map(|p| (
                    para_id,
                    p.direct_formatting.keep_with_next,
                    p.direct_formatting.keep_together,
                    p.direct_formatting.page_break_before,
                    p.direct_formatting.widow_control,
                ))
            })
            .collect();

        // Apply new options to all paragraphs
        for &para_id in &paragraphs {
            if let Some(para) = new_tree.get_paragraph_mut(para_id) {
                if let Some(v) = self.keep_with_next {
                    para.direct_formatting.keep_with_next = Some(v);
                }
                if let Some(v) = self.keep_together {
                    para.direct_formatting.keep_together = Some(v);
                }
                if let Some(v) = self.page_break_before {
                    para.direct_formatting.page_break_before = Some(v);
                }
                if let Some(v) = self.widow_control {
                    para.direct_formatting.widow_control = Some(v);
                }
            }
        }

        let inverse = Box::new(RestoreParagraphPagination {
            options: old_options,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SetParagraphPagination::new(None, None, None, None))
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Paragraph Pagination"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Restore paragraph pagination options (for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestoreParagraphPagination {
    options: Vec<(NodeId, Option<bool>, Option<bool>, Option<bool>, Option<bool>)>,
}

impl Command for RestoreParagraphPagination {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Store current options for redo
        let current_options: Vec<(NodeId, Option<bool>, Option<bool>, Option<bool>, Option<bool>)> = self.options
            .iter()
            .filter_map(|(para_id, _, _, _, _)| {
                new_tree.get_paragraph(*para_id).map(|p| (
                    *para_id,
                    p.direct_formatting.keep_with_next,
                    p.direct_formatting.keep_together,
                    p.direct_formatting.page_break_before,
                    p.direct_formatting.widow_control,
                ))
            })
            .collect();

        // Restore old options
        for (para_id, keep_with_next, keep_together, page_break_before, widow_control) in &self.options {
            if let Some(para) = new_tree.get_paragraph_mut(*para_id) {
                para.direct_formatting.keep_with_next = *keep_with_next;
                para.direct_formatting.keep_together = *keep_together;
                para.direct_formatting.page_break_before = *page_break_before;
                para.direct_formatting.widow_control = *widow_control;
            }
        }

        let inverse = Box::new(RestoreParagraphPagination {
            options: current_options,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(RestoreParagraphPagination {
            options: self.options.clone(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Restore Pagination"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Set Paragraph Borders Command
// =============================================================================

/// Set paragraph borders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParagraphBorders {
    /// New borders configuration (None = remove borders)
    pub borders: Option<ParagraphBorders>,
}

impl SetParagraphBorders {
    pub fn new(borders: Option<ParagraphBorders>) -> Self {
        Self { borders }
    }

    /// Remove all borders
    pub fn remove() -> Self {
        Self { borders: None }
    }

    /// Set a box border (all sides)
    pub fn box_border(width: f32, style: BorderStyleType, color: String) -> Self {
        let border = BorderStyle { width, style, color };
        Self {
            borders: Some(ParagraphBorders {
                top: Some(border.clone()),
                bottom: Some(border.clone()),
                left: Some(border.clone()),
                right: Some(border),
            }),
        }
    }

    /// Set top and bottom borders only
    pub fn horizontal_borders(width: f32, style: BorderStyleType, color: String) -> Self {
        let border = BorderStyle { width, style, color };
        Self {
            borders: Some(ParagraphBorders {
                top: Some(border.clone()),
                bottom: Some(border),
                left: None,
                right: None,
            }),
        }
    }
}

impl Command for SetParagraphBorders {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();
        let paragraphs = get_paragraphs_in_selection(&new_tree, selection)?;

        // Store old borders for undo
        let old_borders: Vec<(NodeId, Option<ParagraphBorders>)> = paragraphs
            .iter()
            .filter_map(|&para_id| {
                new_tree.get_paragraph(para_id)
                    .map(|p| (para_id, p.direct_formatting.borders.clone()))
            })
            .collect();

        // Apply new borders to all paragraphs
        for &para_id in &paragraphs {
            if let Some(para) = new_tree.get_paragraph_mut(para_id) {
                para.direct_formatting.borders = self.borders.clone();
            }
        }

        let inverse = Box::new(RestoreParagraphBorders {
            borders: old_borders,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(SetParagraphBorders::remove())
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Paragraph Borders"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Restore paragraph borders (for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestoreParagraphBorders {
    borders: Vec<(NodeId, Option<ParagraphBorders>)>,
}

impl Command for RestoreParagraphBorders {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Store current borders for redo
        let current_borders: Vec<(NodeId, Option<ParagraphBorders>)> = self.borders
            .iter()
            .filter_map(|(para_id, _)| {
                new_tree.get_paragraph(*para_id)
                    .map(|p| (*para_id, p.direct_formatting.borders.clone()))
            })
            .collect();

        // Restore old borders
        for (para_id, borders) in &self.borders {
            if let Some(para) = new_tree.get_paragraph_mut(*para_id) {
                para.direct_formatting.borders = borders.clone();
            }
        }

        let inverse = Box::new(RestoreParagraphBorders {
            borders: current_borders,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(RestoreParagraphBorders {
            borders: self.borders.clone(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Restore Borders"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::{DocumentTree, Paragraph, Position, Run, Selection};

    fn create_test_tree_with_paragraph() -> (DocumentTree, NodeId) {
        let mut tree = DocumentTree::new();
        let para = Paragraph::new();
        let para_id = para.id();
        tree.insert_paragraph(para, tree.root_id(), None).unwrap();

        let run = Run::new("Test paragraph content");
        tree.insert_run(run, para_id, None).unwrap();

        (tree, para_id)
    }

    #[test]
    fn test_set_alignment_center() {
        let (tree, para_id) = create_test_tree_with_paragraph();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let cmd = SetParagraphAlignment::center();
        let result = cmd.apply(&tree, &selection).unwrap();

        let para = result.tree.get_paragraph(para_id).unwrap();
        assert_eq!(para.direct_formatting.alignment, Some(Alignment::Center));
    }

    #[test]
    fn test_set_alignment_undo() {
        let (tree, para_id) = create_test_tree_with_paragraph();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Set to center
        let cmd = SetParagraphAlignment::center();
        let result = cmd.apply(&tree, &selection).unwrap();

        // Undo
        let undo_result = result.inverse.apply(&result.tree, &selection).unwrap();

        let para = undo_result.tree.get_paragraph(para_id).unwrap();
        assert_eq!(para.direct_formatting.alignment, None);
    }

    #[test]
    fn test_set_indent() {
        let (tree, para_id) = create_test_tree_with_paragraph();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let cmd = SetParagraphIndent::new(Some(36.0), Some(18.0), Some(0.0));
        let result = cmd.apply(&tree, &selection).unwrap();

        let para = result.tree.get_paragraph(para_id).unwrap();
        assert_eq!(para.direct_formatting.indent_left, Some(36.0));
        assert_eq!(para.direct_formatting.indent_right, Some(18.0));
    }

    #[test]
    fn test_increase_indent() {
        let (tree, para_id) = create_test_tree_with_paragraph();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        // First increase
        let cmd = SetParagraphIndent::increase_indent();
        let result = cmd.apply(&tree, &selection).unwrap();

        let para = result.tree.get_paragraph(para_id).unwrap();
        assert_eq!(para.direct_formatting.indent_left, Some(36.0));

        // Second increase
        let result2 = cmd.apply(&result.tree, &selection).unwrap();
        let para2 = result2.tree.get_paragraph(para_id).unwrap();
        assert_eq!(para2.direct_formatting.indent_left, Some(72.0));
    }

    #[test]
    fn test_set_line_spacing() {
        let (tree, para_id) = create_test_tree_with_paragraph();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let cmd = SetParagraphSpacing::double_spacing();
        let result = cmd.apply(&tree, &selection).unwrap();

        let para = result.tree.get_paragraph(para_id).unwrap();
        assert_eq!(para.direct_formatting.line_spacing, Some(LineSpacing::Multiple(2.0)));
    }

    #[test]
    fn test_set_pagination_options() {
        let (tree, para_id) = create_test_tree_with_paragraph();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let cmd = SetParagraphPagination::keep_with_next(true);
        let result = cmd.apply(&tree, &selection).unwrap();

        let para = result.tree.get_paragraph(para_id).unwrap();
        assert_eq!(para.direct_formatting.keep_with_next, Some(true));
    }
}
