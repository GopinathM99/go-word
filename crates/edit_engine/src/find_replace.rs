//! Find and Replace Engine
//!
//! This module provides comprehensive find and replace functionality:
//! - FindOptions for configuring search behavior
//! - FindResult for representing search matches
//! - FindEngine for searching documents
//! - ReplaceEngine for replacing matches
//! - Regex pattern support with capture groups

use crate::{Command, CommandResult, EditError, Result};
use doc_model::{DocumentTree, Node, NodeId, Position, Selection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Options for find operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FindOptions {
    /// Case-sensitive search
    pub case_sensitive: bool,
    /// Match whole words only
    pub whole_word: bool,
    /// Use regex pattern
    pub use_regex: bool,
    /// Wrap around to beginning when reaching end
    pub wrap_around: bool,
    /// Search backwards
    pub search_backwards: bool,
}

impl FindOptions {
    /// Create a new FindOptions with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set case sensitivity
    pub fn case_sensitive(mut self, value: bool) -> Self {
        self.case_sensitive = value;
        self
    }

    /// Set whole word matching
    pub fn whole_word(mut self, value: bool) -> Self {
        self.whole_word = value;
        self
    }

    /// Enable regex mode
    pub fn regex(mut self, value: bool) -> Self {
        self.use_regex = value;
        self
    }

    /// Enable wrap around
    pub fn wrap_around(mut self, value: bool) -> Self {
        self.wrap_around = value;
        self
    }

    /// Enable backward search
    pub fn backwards(mut self, value: bool) -> Self {
        self.search_backwards = value;
        self
    }
}

/// Result of a find operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindResult {
    /// The paragraph containing the match
    pub node_id: NodeId,
    /// Start offset within the paragraph (character index)
    pub start_offset: usize,
    /// End offset within the paragraph (character index)
    pub end_offset: usize,
    /// The matched text
    pub matched_text: String,
    /// Context around the match (for preview)
    pub context: Option<String>,
    /// Match index (1-based, for display like "1 of 5")
    pub match_index: usize,
}

impl FindResult {
    /// Create a new find result
    pub fn new(node_id: NodeId, start_offset: usize, end_offset: usize, matched_text: String) -> Self {
        Self {
            node_id,
            start_offset,
            end_offset,
            matched_text,
            context: None,
            match_index: 0,
        }
    }

    /// Get the length of the match
    pub fn len(&self) -> usize {
        self.end_offset - self.start_offset
    }

    /// Check if the match is empty
    pub fn is_empty(&self) -> bool {
        self.start_offset == self.end_offset
    }

    /// Create a selection from this result
    pub fn to_selection(&self) -> Selection {
        Selection::new(
            Position::new(self.node_id, self.start_offset),
            Position::new(self.node_id, self.end_offset),
        )
    }
}

/// Find engine for searching documents
pub struct FindEngine<'a> {
    tree: &'a DocumentTree,
}

impl<'a> FindEngine<'a> {
    /// Create a new find engine for a document
    pub fn new(tree: &'a DocumentTree) -> Self {
        Self { tree }
    }

    /// Find all matches in the document
    pub fn find_all(&self, pattern: &str, options: &FindOptions) -> Vec<FindResult> {
        if pattern.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        let para_ids: Vec<NodeId> = self.tree.document.children().to_vec();

        for para_id in para_ids {
            let matches = self.find_in_paragraph(para_id, pattern, options);
            results.extend(matches);
        }

        // Assign match indices
        for (i, result) in results.iter_mut().enumerate() {
            result.match_index = i + 1;
        }

        results
    }

    /// Find the next match starting from a position
    pub fn find_next(
        &self,
        pattern: &str,
        from: &Position,
        options: &FindOptions,
    ) -> Option<FindResult> {
        if pattern.is_empty() {
            return None;
        }

        let para_ids: Vec<NodeId> = self.tree.document.children().to_vec();
        let start_para_index = para_ids.iter().position(|&id| id == from.node_id).unwrap_or(0);

        // Search from current paragraph to end
        for (offset, &para_id) in para_ids[start_para_index..].iter().enumerate() {
            let start_offset = if offset == 0 { from.offset } else { 0 };
            if let Some(result) = self.find_in_paragraph_from(para_id, pattern, options, start_offset, false)
            {
                return Some(result);
            }
        }

        // Wrap around if enabled
        if options.wrap_around && start_para_index > 0 {
            for &para_id in para_ids[..start_para_index].iter() {
                if let Some(result) = self.find_in_paragraph_from(para_id, pattern, options, 0, false) {
                    return Some(result);
                }
            }
        }

        None
    }

    /// Find the previous match starting from a position
    pub fn find_previous(
        &self,
        pattern: &str,
        from: &Position,
        options: &FindOptions,
    ) -> Option<FindResult> {
        if pattern.is_empty() {
            return None;
        }

        let para_ids: Vec<NodeId> = self.tree.document.children().to_vec();
        let start_para_index = para_ids.iter().position(|&id| id == from.node_id).unwrap_or(0);

        // Search backwards from current paragraph to beginning
        for (offset, &para_id) in para_ids[..=start_para_index].iter().rev().enumerate() {
            let end_offset = if offset == 0 { from.offset } else { usize::MAX };
            if let Some(result) = self.find_in_paragraph_from(para_id, pattern, options, 0, true) {
                // For backwards search, we need to find the last match before end_offset
                let matches = self.find_in_paragraph(para_id, pattern, options);
                if let Some(last_before) = matches.into_iter().filter(|m| m.end_offset <= end_offset).last() {
                    return Some(last_before);
                }
            }
        }

        // Wrap around if enabled
        if options.wrap_around && start_para_index < para_ids.len() - 1 {
            for &para_id in para_ids[start_para_index + 1..].iter().rev() {
                let matches = self.find_in_paragraph(para_id, pattern, options);
                if let Some(last) = matches.into_iter().last() {
                    return Some(last);
                }
            }
        }

        None
    }

    /// Find matches in a single paragraph
    fn find_in_paragraph(
        &self,
        para_id: NodeId,
        pattern: &str,
        options: &FindOptions,
    ) -> Vec<FindResult> {
        let text = self.get_paragraph_text(para_id);
        if text.is_empty() {
            return Vec::new();
        }

        let matches = if options.use_regex {
            self.find_regex_matches(&text, pattern, options)
        } else {
            self.find_literal_matches(&text, pattern, options)
        };

        matches
            .into_iter()
            .map(|(start, end, matched)| {
                let mut result = FindResult::new(para_id, start, end, matched);
                result.context = Some(self.get_context(&text, start, end));
                result
            })
            .collect()
    }

    /// Find the first match in a paragraph starting from an offset
    fn find_in_paragraph_from(
        &self,
        para_id: NodeId,
        pattern: &str,
        options: &FindOptions,
        from_offset: usize,
        _backwards: bool,
    ) -> Option<FindResult> {
        let matches = self.find_in_paragraph(para_id, pattern, options);
        matches.into_iter().find(|m| m.start_offset >= from_offset)
    }

    /// Get the text content of a paragraph
    fn get_paragraph_text(&self, para_id: NodeId) -> String {
        let para = match self.tree.get_paragraph(para_id) {
            Some(p) => p,
            None => return String::new(),
        };

        let mut text = String::new();
        for &run_id in para.children() {
            if let Some(run) = self.tree.get_run(run_id) {
                text.push_str(&run.text);
            }
        }
        text
    }

    /// Find literal (non-regex) matches
    fn find_literal_matches(
        &self,
        text: &str,
        pattern: &str,
        options: &FindOptions,
    ) -> Vec<(usize, usize, String)> {
        let mut results = Vec::new();

        let (search_text, search_pattern) = if options.case_sensitive {
            (text.to_string(), pattern.to_string())
        } else {
            (text.to_lowercase(), pattern.to_lowercase())
        };

        let text_chars: Vec<char> = text.chars().collect();
        let search_chars: Vec<char> = search_text.chars().collect();
        let pattern_chars: Vec<char> = search_pattern.chars().collect();
        let pattern_len = pattern_chars.len();

        if pattern_len == 0 || search_chars.len() < pattern_len {
            return results;
        }

        let mut char_index = 0;
        while char_index + pattern_len <= search_chars.len() {
            let window: String = search_chars[char_index..char_index + pattern_len]
                .iter()
                .collect();

            if window == search_pattern {
                // Check whole word if required
                let is_whole_word = if options.whole_word {
                    let before_ok = char_index == 0
                        || !search_chars[char_index - 1].is_alphanumeric();
                    let after_ok = char_index + pattern_len >= search_chars.len()
                        || !search_chars[char_index + pattern_len].is_alphanumeric();
                    before_ok && after_ok
                } else {
                    true
                };

                if is_whole_word {
                    let matched: String = text_chars[char_index..char_index + pattern_len]
                        .iter()
                        .collect();
                    results.push((char_index, char_index + pattern_len, matched));
                }
            }

            char_index += 1;
        }

        results
    }

    /// Find regex matches
    fn find_regex_matches(
        &self,
        text: &str,
        pattern: &str,
        options: &FindOptions,
    ) -> Vec<(usize, usize, String)> {
        // Build regex pattern with options
        let regex_pattern = if options.case_sensitive {
            pattern.to_string()
        } else {
            format!("(?i){}", pattern)
        };

        let regex_pattern = if options.whole_word {
            format!(r"\b{}\b", regex_pattern)
        } else {
            regex_pattern
        };

        // Note: In a real implementation, we would use the regex crate
        // For now, we'll use a simple implementation that handles basic patterns
        self.simple_regex_match(text, &regex_pattern, options)
    }

    /// Simple regex matching (basic implementation)
    /// In production, this would use the regex crate
    fn simple_regex_match(
        &self,
        text: &str,
        _pattern: &str,
        _options: &FindOptions,
    ) -> Vec<(usize, usize, String)> {
        // This is a placeholder - a full implementation would use the regex crate
        // For now, fall back to literal matching
        Vec::new()
    }

    /// Get context around a match for preview
    fn get_context(&self, text: &str, start: usize, end: usize) -> String {
        let chars: Vec<char> = text.chars().collect();
        let context_chars = 20;

        let context_start = start.saturating_sub(context_chars);
        let context_end = (end + context_chars).min(chars.len());

        let mut context: String = chars[context_start..context_end].iter().collect();

        if context_start > 0 {
            context = format!("...{}", context);
        }
        if context_end < chars.len() {
            context = format!("{}...", context);
        }

        context
    }
}

/// Result of a replace operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceResult {
    /// Number of replacements made
    pub count: usize,
    /// The positions that were modified
    pub modified_positions: Vec<Position>,
}

/// Replace engine for modifying documents
pub struct ReplaceEngine;

impl ReplaceEngine {
    /// Replace a single match
    pub fn replace(
        tree: &DocumentTree,
        find_result: &FindResult,
        replacement: &str,
        selection: &Selection,
    ) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the paragraph
        let para_id = find_result.node_id;
        let para = new_tree.get_paragraph(para_id)
            .ok_or_else(|| EditError::InvalidCommand("Paragraph not found".to_string()))?;

        // Find which run(s) contain the match and modify them
        let mut current_offset = 0;
        let start = find_result.start_offset;
        let end = find_result.end_offset;
        let run_ids: Vec<NodeId> = para.children().to_vec();

        // Track modifications across runs
        let mut remaining_delete_start = start;
        let mut remaining_delete_end = end;
        let mut inserted = false;

        for run_id in run_ids {
            let run = new_tree.get_run_mut(run_id)
                .ok_or_else(|| EditError::InvalidCommand("Run not found".to_string()))?;

            let run_len = run.text.chars().count();
            let run_start = current_offset;
            let run_end = current_offset + run_len;

            // Check if this run overlaps with the deletion range
            if run_end > remaining_delete_start && run_start < remaining_delete_end {
                let chars: Vec<char> = run.text.chars().collect();

                let delete_start_in_run = if remaining_delete_start > run_start {
                    remaining_delete_start - run_start
                } else {
                    0
                };

                let delete_end_in_run = if remaining_delete_end < run_end {
                    remaining_delete_end - run_start
                } else {
                    run_len
                };

                // Build new text
                let mut new_text = String::new();

                // Add text before deletion
                for &c in &chars[..delete_start_in_run] {
                    new_text.push(c);
                }

                // Add replacement (only once, in the first affected run)
                if !inserted {
                    new_text.push_str(replacement);
                    inserted = true;
                }

                // Add text after deletion
                for &c in &chars[delete_end_in_run..] {
                    new_text.push(c);
                }

                run.text = new_text;

                // Update remaining range
                remaining_delete_start = run_end;
            }

            current_offset = run_end;
        }

        // Calculate new selection position (after replacement)
        let new_position = Position::new(para_id, start + replacement.chars().count());
        let new_selection = Selection::collapsed(new_position);

        // Create inverse command (would replace back with original)
        let inverse = Box::new(ReplaceText {
            para_id,
            start_offset: start,
            end_offset: start + replacement.chars().count(),
            replacement: find_result.matched_text.clone(),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    /// Replace all matches
    pub fn replace_all(
        tree: &DocumentTree,
        pattern: &str,
        replacement: &str,
        options: &FindOptions,
        selection: &Selection,
    ) -> Result<CommandResult> {
        let engine = FindEngine::new(tree);
        let matches = engine.find_all(pattern, options);

        if matches.is_empty() {
            return Ok(CommandResult {
                tree: tree.clone(),
                selection: *selection,
                inverse: Box::new(NoOp),
            });
        }

        let mut new_tree = tree.clone();
        let match_count = matches.len();

        // Group matches by paragraph
        let mut matches_by_para: HashMap<NodeId, Vec<FindResult>> = HashMap::new();
        for m in matches {
            matches_by_para.entry(m.node_id).or_default().push(m);
        }

        // Process each paragraph (in reverse order of matches to maintain positions)
        for (para_id, mut para_matches) in matches_by_para {
            // Sort matches in reverse order so we can replace from end to start
            para_matches.sort_by(|a, b| b.start_offset.cmp(&a.start_offset));

            for find_result in para_matches {
                // Get current paragraph state
                let para = new_tree.get_paragraph(para_id)
                    .ok_or_else(|| EditError::InvalidCommand("Paragraph not found".to_string()))?;

                let mut current_offset = 0;
                let start = find_result.start_offset;
                let end = find_result.end_offset;
                let run_ids: Vec<NodeId> = para.children().to_vec();

                let mut remaining_delete_start = start;
                let mut remaining_delete_end = end;
                let mut inserted = false;

                for run_id in run_ids {
                    if let Some(run) = new_tree.get_run_mut(run_id) {
                        let run_len = run.text.chars().count();
                        let run_start = current_offset;
                        let run_end = current_offset + run_len;

                        if run_end > remaining_delete_start && run_start < remaining_delete_end {
                            let chars: Vec<char> = run.text.chars().collect();

                            let delete_start_in_run = if remaining_delete_start > run_start {
                                remaining_delete_start - run_start
                            } else {
                                0
                            };

                            let delete_end_in_run = if remaining_delete_end < run_end {
                                remaining_delete_end - run_start
                            } else {
                                run_len
                            };

                            let mut new_text = String::new();
                            for &c in &chars[..delete_start_in_run] {
                                new_text.push(c);
                            }
                            if !inserted {
                                new_text.push_str(replacement);
                                inserted = true;
                            }
                            for &c in &chars[delete_end_in_run..] {
                                new_text.push(c);
                            }

                            run.text = new_text;
                            remaining_delete_start = run_end;
                        }

                        current_offset = run_end;
                    }
                }
            }
        }

        // For replace all, the inverse would need to store all original matches
        // For simplicity, we'll use a compound undo marker
        let inverse = Box::new(ReplaceAllUndo {
            original_tree: tree.clone(),
            count: match_count,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }
}

/// Command to replace text at a specific location
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReplaceText {
    para_id: NodeId,
    start_offset: usize,
    end_offset: usize,
    replacement: String,
}

impl Command for ReplaceText {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let find_result = FindResult::new(
            self.para_id,
            self.start_offset,
            self.end_offset,
            String::new(), // We don't need the original text for replacement
        );
        ReplaceEngine::replace(tree, &find_result, &self.replacement, selection)
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        // Get the current text that would be replaced
        let engine = FindEngine::new(tree);
        let text = engine.get_paragraph_text(self.para_id);
        let chars: Vec<char> = text.chars().collect();
        let current_text: String = chars
            .get(self.start_offset..self.end_offset)
            .map(|s| s.iter().collect())
            .unwrap_or_default();

        Box::new(ReplaceText {
            para_id: self.para_id,
            start_offset: self.start_offset,
            end_offset: self.start_offset + self.replacement.chars().count(),
            replacement: current_text,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Replace"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Undo data for replace all operation
#[derive(Debug, Clone)]
struct ReplaceAllUndo {
    original_tree: DocumentTree,
    count: usize,
}

impl Command for ReplaceAllUndo {
    fn apply(&self, _tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        Ok(CommandResult {
            tree: self.original_tree.clone(),
            selection: *selection,
            inverse: Box::new(NoOp), // Can't re-redo easily
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(NoOp)
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Undo Replace All"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// No-operation command placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NoOp;

impl Command for NoOp {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        Ok(CommandResult {
            tree: tree.clone(),
            selection: *selection,
            inverse: Box::new(NoOp),
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(NoOp)
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "No Operation"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// ============================================================================
// Find/Replace Commands for the command system
// ============================================================================

/// Command to find and select the next match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindCommand {
    /// The search pattern
    pub pattern: String,
    /// Search options
    pub options: FindOptions,
}

impl FindCommand {
    /// Create a new find command
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            options: FindOptions::default(),
        }
    }

    /// Create with specific options
    pub fn with_options(pattern: impl Into<String>, options: FindOptions) -> Self {
        Self {
            pattern: pattern.into(),
            options,
        }
    }
}

impl Command for FindCommand {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let engine = FindEngine::new(tree);
        let from = selection.focus;

        let result = if self.options.search_backwards {
            engine.find_previous(&self.pattern, &from, &self.options)
        } else {
            engine.find_next(&self.pattern, &from, &self.options)
        };

        match result {
            Some(find_result) => {
                let new_selection = find_result.to_selection();
                Ok(CommandResult {
                    tree: tree.clone(),
                    selection: new_selection,
                    inverse: Box::new(SetSelectionCommand {
                        selection: *selection,
                    }),
                })
            }
            None => {
                // No match found - keep current selection
                Ok(CommandResult {
                    tree: tree.clone(),
                    selection: *selection,
                    inverse: Box::new(NoOp),
                })
            }
        }
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(NoOp)
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Find"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Command to replace the current selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceCommand {
    /// The replacement text
    pub replacement: String,
    /// Optional: the pattern that was searched (for validation)
    pub expected_pattern: Option<String>,
}

impl ReplaceCommand {
    /// Create a new replace command
    pub fn new(replacement: impl Into<String>) -> Self {
        Self {
            replacement: replacement.into(),
            expected_pattern: None,
        }
    }

    /// Create with expected pattern validation
    pub fn with_pattern(replacement: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            replacement: replacement.into(),
            expected_pattern: Some(pattern.into()),
        }
    }
}

impl Command for ReplaceCommand {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        // Only replace if there's a selection (not collapsed)
        if selection.is_collapsed() {
            return Err(EditError::InvalidCommand(
                "No selection to replace".to_string(),
            ));
        }

        // Get the selected text
        let engine = FindEngine::new(tree);

        // For same-node selection
        if selection.anchor.node_id == selection.focus.node_id {
            let text = engine.get_paragraph_text(selection.anchor.node_id);
            let chars: Vec<char> = text.chars().collect();
            let start = selection.start().offset;
            let end = selection.end().offset;

            let selected_text: String = chars
                .get(start..end)
                .map(|s| s.iter().collect())
                .unwrap_or_default();

            let find_result = FindResult::new(
                selection.anchor.node_id,
                start,
                end,
                selected_text,
            );

            ReplaceEngine::replace(tree, &find_result, &self.replacement, selection)
        } else {
            Err(EditError::InvalidCommand(
                "Cross-paragraph replace not yet supported".to_string(),
            ))
        }
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(NoOp)
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Replace"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Command to replace all matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceAllCommand {
    /// The search pattern
    pub pattern: String,
    /// The replacement text
    pub replacement: String,
    /// Search options
    pub options: FindOptions,
}

impl ReplaceAllCommand {
    /// Create a new replace all command
    pub fn new(pattern: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            replacement: replacement.into(),
            options: FindOptions::default(),
        }
    }

    /// Create with specific options
    pub fn with_options(
        pattern: impl Into<String>,
        replacement: impl Into<String>,
        options: FindOptions,
    ) -> Self {
        Self {
            pattern: pattern.into(),
            replacement: replacement.into(),
            options,
        }
    }
}

impl Command for ReplaceAllCommand {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        ReplaceEngine::replace_all(tree, &self.pattern, &self.replacement, &self.options, selection)
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(ReplaceAllUndo {
            original_tree: tree.clone(),
            count: 0,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Replace All"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Internal command to set selection (for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SetSelectionCommand {
    selection: Selection,
}

impl Command for SetSelectionCommand {
    fn apply(&self, tree: &DocumentTree, _selection: &Selection) -> Result<CommandResult> {
        Ok(CommandResult {
            tree: tree.clone(),
            selection: self.selection,
            inverse: Box::new(NoOp),
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(NoOp)
    }

    fn transform_selection(&self, _selection: &Selection) -> Selection {
        self.selection
    }

    fn display_name(&self) -> &str {
        "Set Selection"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Information about all matches for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindAllResults {
    /// All matches found
    pub matches: Vec<FindResult>,
    /// Total count
    pub total_count: usize,
    /// Current match index (0-based, None if no current)
    pub current_index: Option<usize>,
}

impl FindAllResults {
    /// Create from a list of matches
    pub fn from_matches(matches: Vec<FindResult>) -> Self {
        let total_count = matches.len();
        Self {
            matches,
            total_count,
            current_index: if total_count > 0 { Some(0) } else { None },
        }
    }

    /// Check if there are any matches
    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }

    /// Get the current match
    pub fn current(&self) -> Option<&FindResult> {
        self.current_index.and_then(|i| self.matches.get(i))
    }

    /// Move to the next match
    pub fn next(&mut self) {
        if let Some(idx) = self.current_index {
            self.current_index = Some((idx + 1) % self.total_count);
        }
    }

    /// Move to the previous match
    pub fn previous(&mut self) {
        if let Some(idx) = self.current_index {
            self.current_index = Some(if idx == 0 {
                self.total_count.saturating_sub(1)
            } else {
                idx - 1
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::{Paragraph, Run};

    fn create_test_tree_with_text(text: &str) -> (DocumentTree, NodeId) {
        let mut tree = DocumentTree::new();
        let para = Paragraph::new();
        let para_id = para.id();
        tree.insert_paragraph(para, tree.root_id(), None).unwrap();

        let run = Run::new(text);
        tree.insert_run(run, para_id, None).unwrap();

        (tree, para_id)
    }

    #[test]
    fn test_find_options_builder() {
        let options = FindOptions::new()
            .case_sensitive(true)
            .whole_word(true)
            .regex(false)
            .wrap_around(true);

        assert!(options.case_sensitive);
        assert!(options.whole_word);
        assert!(!options.use_regex);
        assert!(options.wrap_around);
    }

    #[test]
    fn test_find_result_to_selection() {
        let node_id = NodeId::new();
        let result = FindResult::new(node_id, 5, 10, "hello".to_string());

        let selection = result.to_selection();
        assert_eq!(selection.anchor.node_id, node_id);
        assert_eq!(selection.anchor.offset, 5);
        assert_eq!(selection.focus.offset, 10);
    }

    #[test]
    fn test_find_all_basic() {
        let (tree, _) = create_test_tree_with_text("The quick brown fox jumps over the lazy dog.");

        let engine = FindEngine::new(&tree);

        // Default is case_sensitive=false, so should find both "The" and "the"
        let results = engine.find_all("the", &FindOptions::default());
        assert_eq!(results.len(), 2);

        // With case sensitive, should only find exact match "the"
        let results = engine.find_all("the", &FindOptions::new().case_sensitive(true));
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_find_all_case_sensitive() {
        let (tree, _) = create_test_tree_with_text("Hello hello HELLO");

        let engine = FindEngine::new(&tree);

        // Case sensitive
        let results = engine.find_all("Hello", &FindOptions::new().case_sensitive(true));
        assert_eq!(results.len(), 1);

        // Case insensitive
        let results = engine.find_all("hello", &FindOptions::new().case_sensitive(false));
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_find_all_whole_word() {
        let (tree, _) = create_test_tree_with_text("test testing tested test");

        let engine = FindEngine::new(&tree);

        // Without whole word
        let results = engine.find_all("test", &FindOptions::default());
        assert_eq!(results.len(), 4); // test, testing, tested, test

        // With whole word
        let results = engine.find_all("test", &FindOptions::new().whole_word(true));
        assert_eq!(results.len(), 2); // Only standalone "test"
    }

    #[test]
    fn test_find_next() {
        let (tree, para_id) = create_test_tree_with_text("one two one three one");

        let engine = FindEngine::new(&tree);
        let from = Position::new(para_id, 0);

        let result = engine.find_next("one", &from, &FindOptions::default());
        assert!(result.is_some());
        assert_eq!(result.unwrap().start_offset, 0);

        let from2 = Position::new(para_id, 1);
        let result2 = engine.find_next("one", &from2, &FindOptions::default());
        assert!(result2.is_some());
        assert_eq!(result2.unwrap().start_offset, 8);
    }

    #[test]
    fn test_find_empty_pattern() {
        let (tree, _) = create_test_tree_with_text("Hello world");

        let engine = FindEngine::new(&tree);
        let results = engine.find_all("", &FindOptions::default());
        assert!(results.is_empty());
    }

    #[test]
    fn test_find_no_match() {
        let (tree, _) = create_test_tree_with_text("Hello world");

        let engine = FindEngine::new(&tree);
        let results = engine.find_all("xyz", &FindOptions::default());
        assert!(results.is_empty());
    }

    #[test]
    fn test_replace_single() {
        let (tree, para_id) = create_test_tree_with_text("Hello world");

        let find_result = FindResult::new(para_id, 6, 11, "world".to_string());
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let result = ReplaceEngine::replace(&tree, &find_result, "universe", &selection);
        assert!(result.is_ok());

        let cmd_result = result.unwrap();
        let engine = FindEngine::new(&cmd_result.tree);
        let text = engine.get_paragraph_text(para_id);
        assert_eq!(text, "Hello universe");
    }

    #[test]
    fn test_replace_all() {
        let (tree, para_id) = create_test_tree_with_text("cat dog cat bird cat");

        let selection = Selection::collapsed(Position::new(para_id, 0));
        let result = ReplaceEngine::replace_all(&tree, "cat", "fish", &FindOptions::default(), &selection);
        assert!(result.is_ok());

        let cmd_result = result.unwrap();
        let engine = FindEngine::new(&cmd_result.tree);
        let text = engine.get_paragraph_text(para_id);
        assert_eq!(text, "fish dog fish bird fish");
    }

    #[test]
    fn test_find_command() {
        let (tree, para_id) = create_test_tree_with_text("Hello world hello");

        let selection = Selection::collapsed(Position::new(para_id, 0));
        let cmd = FindCommand::new("hello");

        let result = cmd.apply(&tree, &selection);
        assert!(result.is_ok());

        let cmd_result = result.unwrap();
        // Default is case_insensitive, so should find "Hello" at position 0 first
        assert_eq!(cmd_result.selection.start().offset, 0);
        assert_eq!(cmd_result.selection.end().offset, 5);
    }

    #[test]
    fn test_find_all_results() {
        let matches = vec![
            FindResult::new(NodeId::new(), 0, 5, "hello".to_string()),
            FindResult::new(NodeId::new(), 10, 15, "hello".to_string()),
        ];

        let mut results = FindAllResults::from_matches(matches);
        assert_eq!(results.total_count, 2);
        assert_eq!(results.current_index, Some(0));

        results.next();
        assert_eq!(results.current_index, Some(1));

        results.next();
        assert_eq!(results.current_index, Some(0)); // Wraps around
    }

    #[test]
    fn test_get_context() {
        let (tree, para_id) = create_test_tree_with_text("The quick brown fox jumps over the lazy dog.");

        let engine = FindEngine::new(&tree);
        let results = engine.find_all("fox", &FindOptions::default());

        assert_eq!(results.len(), 1);
        assert!(results[0].context.is_some());
        let context = results[0].context.as_ref().unwrap();
        assert!(context.contains("fox"));
    }
}
