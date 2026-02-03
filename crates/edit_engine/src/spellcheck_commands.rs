//! Spellcheck commands for correcting, ignoring, and managing spelling errors
//!
//! This module provides commands for:
//! - Correcting misspelled words
//! - Ignoring words (once or for the document)
//! - Adding words to the custom dictionary
//! - Running spellcheck on the entire document

use crate::{Command, CommandResult, EditError, Result};
use doc_model::{DocumentTree, Node, NodeId, Position, Selection};
use serde::{Deserialize, Serialize};

/// Information about a spelling error in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSpellingError {
    /// The paragraph containing the error
    pub para_id: NodeId,
    /// Start offset in the paragraph (character index)
    pub start_offset: usize,
    /// End offset in the paragraph (character index)
    pub end_offset: usize,
    /// The misspelled word
    pub word: String,
    /// Suggested corrections
    pub suggestions: Vec<String>,
}

impl DocumentSpellingError {
    /// Create a new document spelling error
    pub fn new(
        para_id: NodeId,
        start_offset: usize,
        end_offset: usize,
        word: String,
        suggestions: Vec<String>,
    ) -> Self {
        Self {
            para_id,
            start_offset,
            end_offset,
            word,
            suggestions,
        }
    }

    /// Create a selection that highlights this error
    pub fn to_selection(&self) -> Selection {
        Selection::new(
            Position::new(self.para_id, self.start_offset),
            Position::new(self.para_id, self.end_offset),
        )
    }
}

/// Ignore a single occurrence of a misspelled word
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreWordCommand {
    /// The word to ignore
    pub word: String,
    /// The position of this specific occurrence
    pub para_id: NodeId,
    pub start_offset: usize,
    pub end_offset: usize,
}

impl IgnoreWordCommand {
    /// Create a new ignore word command
    pub fn new(word: impl Into<String>, para_id: NodeId, start_offset: usize, end_offset: usize) -> Self {
        Self {
            word: word.into(),
            para_id,
            start_offset,
            end_offset,
        }
    }

    /// Create from a spelling error
    pub fn from_error(error: &DocumentSpellingError) -> Self {
        Self {
            word: error.word.clone(),
            para_id: error.para_id,
            start_offset: error.start_offset,
            end_offset: error.end_offset,
        }
    }
}

impl Command for IgnoreWordCommand {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        // Ignoring a word doesn't modify the document tree
        // The actual ignore tracking is handled by the SpellChecker (session ignore)
        // This command is mainly for undo/redo tracking

        // Move selection to after the ignored word
        let new_selection = Selection::collapsed(Position::new(self.para_id, self.end_offset));

        Ok(CommandResult {
            tree: tree.clone(),
            selection: new_selection,
            inverse: Box::new(UnignoreWordCommand {
                word: self.word.clone(),
                para_id: self.para_id,
                start_offset: self.start_offset,
                end_offset: self.end_offset,
            }),
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(UnignoreWordCommand {
            word: self.word.clone(),
            para_id: self.para_id,
            start_offset: self.start_offset,
            end_offset: self.end_offset,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Ignore Spelling"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Unignore a previously ignored word (for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UnignoreWordCommand {
    word: String,
    para_id: NodeId,
    start_offset: usize,
    end_offset: usize,
}

impl Command for UnignoreWordCommand {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        Ok(CommandResult {
            tree: tree.clone(),
            selection: *selection,
            inverse: Box::new(IgnoreWordCommand {
                word: self.word.clone(),
                para_id: self.para_id,
                start_offset: self.start_offset,
                end_offset: self.end_offset,
            }),
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(IgnoreWordCommand {
            word: self.word.clone(),
            para_id: self.para_id,
            start_offset: self.start_offset,
            end_offset: self.end_offset,
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Unignore Spelling"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Ignore all occurrences of a word in the document (session)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreAllCommand {
    /// The word to ignore everywhere
    pub word: String,
}

impl IgnoreAllCommand {
    /// Create a new ignore all command
    pub fn new(word: impl Into<String>) -> Self {
        Self {
            word: word.into(),
        }
    }
}

impl Command for IgnoreAllCommand {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        // Like IgnoreWordCommand, the actual tracking is handled by SpellChecker
        Ok(CommandResult {
            tree: tree.clone(),
            selection: *selection,
            inverse: Box::new(UnignoreAllCommand {
                word: self.word.clone(),
            }),
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(UnignoreAllCommand {
            word: self.word.clone(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Ignore All"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Unignore all occurrences of a word (for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UnignoreAllCommand {
    word: String,
}

impl Command for UnignoreAllCommand {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        Ok(CommandResult {
            tree: tree.clone(),
            selection: *selection,
            inverse: Box::new(IgnoreAllCommand {
                word: self.word.clone(),
            }),
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(IgnoreAllCommand {
            word: self.word.clone(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Unignore All"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Add a word to the custom dictionary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddToDictionaryCommand {
    /// The word to add
    pub word: String,
    /// The language for the dictionary
    pub language: String,
}

impl AddToDictionaryCommand {
    /// Create a new add to dictionary command
    pub fn new(word: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            word: word.into(),
            language: language.into(),
        }
    }

    /// Create with default language (en-US)
    pub fn new_default_language(word: impl Into<String>) -> Self {
        Self {
            word: word.into(),
            language: "en-US".to_string(),
        }
    }
}

impl Command for AddToDictionaryCommand {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        // The actual dictionary modification is handled by the SpellChecker
        Ok(CommandResult {
            tree: tree.clone(),
            selection: *selection,
            inverse: Box::new(RemoveFromDictionaryCommand {
                word: self.word.clone(),
                language: self.language.clone(),
            }),
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(RemoveFromDictionaryCommand {
            word: self.word.clone(),
            language: self.language.clone(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Add to Dictionary"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Remove a word from the custom dictionary (for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveFromDictionaryCommand {
    /// The word to remove
    pub word: String,
    /// The language for the dictionary
    pub language: String,
}

impl RemoveFromDictionaryCommand {
    /// Create a new remove from dictionary command
    pub fn new(word: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            word: word.into(),
            language: language.into(),
        }
    }
}

impl Command for RemoveFromDictionaryCommand {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        Ok(CommandResult {
            tree: tree.clone(),
            selection: *selection,
            inverse: Box::new(AddToDictionaryCommand {
                word: self.word.clone(),
                language: self.language.clone(),
            }),
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(AddToDictionaryCommand {
            word: self.word.clone(),
            language: self.language.clone(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Remove from Dictionary"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Correct a misspelled word with a suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectSpellingCommand {
    /// The paragraph containing the error
    pub para_id: NodeId,
    /// Start offset of the misspelled word
    pub start_offset: usize,
    /// End offset of the misspelled word
    pub end_offset: usize,
    /// The original misspelled word
    pub original_word: String,
    /// The correction to apply
    pub correction: String,
}

impl CorrectSpellingCommand {
    /// Create a new correct spelling command
    pub fn new(
        para_id: NodeId,
        start_offset: usize,
        end_offset: usize,
        original_word: impl Into<String>,
        correction: impl Into<String>,
    ) -> Self {
        Self {
            para_id,
            start_offset,
            end_offset,
            original_word: original_word.into(),
            correction: correction.into(),
        }
    }

    /// Create from a spelling error and selected suggestion
    pub fn from_error(error: &DocumentSpellingError, correction: impl Into<String>) -> Self {
        Self {
            para_id: error.para_id,
            start_offset: error.start_offset,
            end_offset: error.end_offset,
            original_word: error.word.clone(),
            correction: correction.into(),
        }
    }
}

impl Command for CorrectSpellingCommand {
    fn apply(&self, tree: &DocumentTree, _selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the paragraph
        let para = new_tree.get_paragraph(self.para_id)
            .ok_or_else(|| EditError::InvalidCommand("Paragraph not found".to_string()))?;

        // Find which run contains the misspelled word and replace it
        let run_ids: Vec<NodeId> = para.children().to_vec();
        let mut current_offset = 0;

        for run_id in run_ids {
            let run = new_tree.get_run_mut(run_id)
                .ok_or_else(|| EditError::InvalidCommand("Run not found".to_string()))?;

            let run_len = run.text.chars().count();
            let run_end = current_offset + run_len;

            // Check if this run contains the misspelled word
            if run_end > self.start_offset && current_offset < self.end_offset {
                let chars: Vec<char> = run.text.chars().collect();

                let start_in_run = if self.start_offset > current_offset {
                    self.start_offset - current_offset
                } else {
                    0
                };

                let end_in_run = if self.end_offset < run_end {
                    self.end_offset - current_offset
                } else {
                    run_len
                };

                // Build the corrected text
                let mut new_text = String::new();
                for &c in &chars[..start_in_run] {
                    new_text.push(c);
                }
                new_text.push_str(&self.correction);
                for &c in &chars[end_in_run..] {
                    new_text.push(c);
                }

                run.text = new_text;
                break; // Assuming the word is contained in a single run
            }

            current_offset = run_end;
        }

        // Move cursor to end of corrected word
        let new_position = Position::new(self.para_id, self.start_offset + self.correction.chars().count());
        let new_selection = Selection::collapsed(new_position);

        // Create inverse command to undo the correction
        let inverse = Box::new(CorrectSpellingCommand {
            para_id: self.para_id,
            start_offset: self.start_offset,
            end_offset: self.start_offset + self.correction.chars().count(),
            original_word: self.correction.clone(),
            correction: self.original_word.clone(),
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(CorrectSpellingCommand {
            para_id: self.para_id,
            start_offset: self.start_offset,
            end_offset: self.start_offset + self.correction.chars().count(),
            original_word: self.correction.clone(),
            correction: self.original_word.clone(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        // Adjust selection if it's in the same paragraph after the correction point
        if selection.anchor.node_id == self.para_id {
            let old_len = self.end_offset - self.start_offset;
            let new_len = self.correction.chars().count();
            let diff = new_len as isize - old_len as isize;

            let mut new_anchor = selection.anchor;
            let mut new_focus = selection.focus;

            if selection.anchor.offset > self.end_offset {
                new_anchor.offset = (selection.anchor.offset as isize + diff) as usize;
            } else if selection.anchor.offset > self.start_offset {
                new_anchor.offset = self.start_offset + new_len;
            }

            if selection.focus.offset > self.end_offset {
                new_focus.offset = (selection.focus.offset as isize + diff) as usize;
            } else if selection.focus.offset > self.start_offset {
                new_focus.offset = self.start_offset + new_len;
            }

            Selection::new(new_anchor, new_focus)
        } else {
            *selection
        }
    }

    fn display_name(&self) -> &str {
        "Correct Spelling"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Check spelling in the entire document and return all errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellcheckAllCommand {
    /// Language to use for checking
    pub language: String,
}

impl SpellcheckAllCommand {
    /// Create a new spellcheck all command
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            language: language.into(),
        }
    }

    /// Create with default language (en-US)
    pub fn default_language() -> Self {
        Self {
            language: "en-US".to_string(),
        }
    }
}

impl Command for SpellcheckAllCommand {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        // This command doesn't modify the document
        // The actual spellchecking is done externally using SpellChecker
        // This command is mainly for triggering a UI update
        Ok(CommandResult {
            tree: tree.clone(),
            selection: *selection,
            inverse: Box::new(NoOpCommand),
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(NoOpCommand)
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Spellcheck Document"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Navigate to the next spelling error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextSpellingErrorCommand {
    /// The error to navigate to (provided by the spell checker)
    pub error: Option<DocumentSpellingError>,
}

impl NextSpellingErrorCommand {
    /// Create without a specific error (will be determined at execution)
    pub fn new() -> Self {
        Self { error: None }
    }

    /// Create with a specific error to navigate to
    pub fn to_error(error: DocumentSpellingError) -> Self {
        Self { error: Some(error) }
    }
}

impl Default for NextSpellingErrorCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for NextSpellingErrorCommand {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        match &self.error {
            Some(error) => {
                let new_selection = error.to_selection();
                Ok(CommandResult {
                    tree: tree.clone(),
                    selection: new_selection,
                    inverse: Box::new(SetSelectionCommand {
                        selection: *selection,
                    }),
                })
            }
            None => {
                // No error provided, keep current selection
                Ok(CommandResult {
                    tree: tree.clone(),
                    selection: *selection,
                    inverse: Box::new(NoOpCommand),
                })
            }
        }
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(NoOpCommand)
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Next Spelling Error"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Navigate to the previous spelling error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviousSpellingErrorCommand {
    /// The error to navigate to
    pub error: Option<DocumentSpellingError>,
}

impl PreviousSpellingErrorCommand {
    /// Create without a specific error
    pub fn new() -> Self {
        Self { error: None }
    }

    /// Create with a specific error to navigate to
    pub fn to_error(error: DocumentSpellingError) -> Self {
        Self { error: Some(error) }
    }
}

impl Default for PreviousSpellingErrorCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for PreviousSpellingErrorCommand {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        match &self.error {
            Some(error) => {
                let new_selection = error.to_selection();
                Ok(CommandResult {
                    tree: tree.clone(),
                    selection: new_selection,
                    inverse: Box::new(SetSelectionCommand {
                        selection: *selection,
                    }),
                })
            }
            None => {
                Ok(CommandResult {
                    tree: tree.clone(),
                    selection: *selection,
                    inverse: Box::new(NoOpCommand),
                })
            }
        }
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(NoOpCommand)
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Previous Spelling Error"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// No-op command for inverse of read-only commands
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NoOpCommand;

impl Command for NoOpCommand {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        Ok(CommandResult {
            tree: tree.clone(),
            selection: *selection,
            inverse: Box::new(NoOpCommand),
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(NoOpCommand)
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

/// Internal command to restore selection
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SetSelectionCommand {
    selection: Selection,
}

impl Command for SetSelectionCommand {
    fn apply(&self, tree: &DocumentTree, _selection: &Selection) -> Result<CommandResult> {
        Ok(CommandResult {
            tree: tree.clone(),
            selection: self.selection,
            inverse: Box::new(NoOpCommand),
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        Box::new(NoOpCommand)
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

/// Results from running spellcheck on a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellcheckResults {
    /// All spelling errors found
    pub errors: Vec<DocumentSpellingError>,
    /// Current error index (0-based)
    pub current_index: Option<usize>,
    /// Total word count checked
    pub words_checked: usize,
}

impl SpellcheckResults {
    /// Create new empty results
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            current_index: None,
            words_checked: 0,
        }
    }

    /// Create from a list of errors
    pub fn from_errors(errors: Vec<DocumentSpellingError>, words_checked: usize) -> Self {
        let current_index = if errors.is_empty() { None } else { Some(0) };
        Self {
            errors,
            current_index,
            words_checked,
        }
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the current error
    pub fn current(&self) -> Option<&DocumentSpellingError> {
        self.current_index.and_then(|i| self.errors.get(i))
    }

    /// Move to the next error
    pub fn next(&mut self) -> Option<&DocumentSpellingError> {
        if self.errors.is_empty() {
            return None;
        }

        self.current_index = Some(match self.current_index {
            Some(i) => (i + 1) % self.errors.len(),
            None => 0,
        });

        self.current()
    }

    /// Move to the previous error
    pub fn previous(&mut self) -> Option<&DocumentSpellingError> {
        if self.errors.is_empty() {
            return None;
        }

        self.current_index = Some(match self.current_index {
            Some(0) => self.errors.len() - 1,
            Some(i) => i - 1,
            None => self.errors.len() - 1,
        });

        self.current()
    }

    /// Remove the current error (e.g., after ignoring or correcting)
    pub fn remove_current(&mut self) {
        if let Some(i) = self.current_index {
            if i < self.errors.len() {
                self.errors.remove(i);
                if self.errors.is_empty() {
                    self.current_index = None;
                } else if i >= self.errors.len() {
                    self.current_index = Some(self.errors.len() - 1);
                }
            }
        }
    }

    /// Remove all errors for a specific word (e.g., after "Ignore All")
    pub fn remove_word(&mut self, word: &str) {
        let word_lower = word.to_lowercase();
        self.errors.retain(|e| e.word.to_lowercase() != word_lower);

        if self.errors.is_empty() {
            self.current_index = None;
        } else if let Some(i) = self.current_index {
            if i >= self.errors.len() {
                self.current_index = Some(self.errors.len() - 1);
            }
        }
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get current position string (e.g., "3 of 10")
    pub fn position_string(&self) -> String {
        match self.current_index {
            Some(i) => format!("{} of {}", i + 1, self.errors.len()),
            None => "0 of 0".to_string(),
        }
    }
}

impl Default for SpellcheckResults {
    fn default() -> Self {
        Self::new()
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
    fn test_document_spelling_error() {
        let para_id = NodeId::new();
        let error = DocumentSpellingError::new(
            para_id,
            5,
            10,
            "tset".to_string(),
            vec!["test".to_string(), "set".to_string()],
        );

        assert_eq!(error.word, "tset");
        assert_eq!(error.suggestions.len(), 2);

        let selection = error.to_selection();
        assert_eq!(selection.start().offset, 5);
        assert_eq!(selection.end().offset, 10);
    }

    #[test]
    fn test_correct_spelling_command() {
        let (tree, para_id) = create_test_tree_with_text("This is a tset.");

        let cmd = CorrectSpellingCommand::new(para_id, 10, 14, "tset", "test");
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let result = cmd.apply(&tree, &selection);
        assert!(result.is_ok());

        let cmd_result = result.unwrap();

        // Check that the text was corrected
        let para = cmd_result.tree.get_paragraph(para_id).unwrap();
        let run_id = para.children()[0];
        let run = cmd_result.tree.get_run(run_id).unwrap();
        assert_eq!(run.text, "This is a test.");
    }

    #[test]
    fn test_spellcheck_results_navigation() {
        let para_id = NodeId::new();
        let errors = vec![
            DocumentSpellingError::new(para_id, 0, 3, "err1".to_string(), vec![]),
            DocumentSpellingError::new(para_id, 10, 14, "err2".to_string(), vec![]),
            DocumentSpellingError::new(para_id, 20, 24, "err3".to_string(), vec![]),
        ];

        let mut results = SpellcheckResults::from_errors(errors, 100);

        assert_eq!(results.current_index, Some(0));
        assert_eq!(results.current().unwrap().word, "err1");

        results.next();
        assert_eq!(results.current().unwrap().word, "err2");

        results.next();
        assert_eq!(results.current().unwrap().word, "err3");

        results.next(); // Wraps around
        assert_eq!(results.current().unwrap().word, "err1");

        results.previous(); // Goes back
        assert_eq!(results.current().unwrap().word, "err3");
    }

    #[test]
    fn test_spellcheck_results_remove_current() {
        let para_id = NodeId::new();
        let errors = vec![
            DocumentSpellingError::new(para_id, 0, 3, "err1".to_string(), vec![]),
            DocumentSpellingError::new(para_id, 10, 14, "err2".to_string(), vec![]),
        ];

        let mut results = SpellcheckResults::from_errors(errors, 100);

        results.remove_current();
        assert_eq!(results.error_count(), 1);
        assert_eq!(results.current().unwrap().word, "err2");
    }

    #[test]
    fn test_spellcheck_results_remove_word() {
        let para_id = NodeId::new();
        let errors = vec![
            DocumentSpellingError::new(para_id, 0, 3, "err".to_string(), vec![]),
            DocumentSpellingError::new(para_id, 10, 14, "other".to_string(), vec![]),
            DocumentSpellingError::new(para_id, 20, 23, "err".to_string(), vec![]),
        ];

        let mut results = SpellcheckResults::from_errors(errors, 100);

        results.remove_word("err");
        assert_eq!(results.error_count(), 1);
        assert_eq!(results.current().unwrap().word, "other");
    }

    #[test]
    fn test_ignore_word_command() {
        let (tree, para_id) = create_test_tree_with_text("Hello xyzzy world");

        let cmd = IgnoreWordCommand::new("xyzzy", para_id, 6, 11);
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let result = cmd.apply(&tree, &selection);
        assert!(result.is_ok());

        // Tree should be unchanged
        let cmd_result = result.unwrap();
        let para = cmd_result.tree.get_paragraph(para_id).unwrap();
        let run_id = para.children()[0];
        let run = cmd_result.tree.get_run(run_id).unwrap();
        assert_eq!(run.text, "Hello xyzzy world");
    }

    #[test]
    fn test_add_to_dictionary_command() {
        let (tree, para_id) = create_test_tree_with_text("Hello world");

        let cmd = AddToDictionaryCommand::new_default_language("xyzzy");
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let result = cmd.apply(&tree, &selection);
        assert!(result.is_ok());

        // Tree should be unchanged (dictionary is managed externally)
    }

    #[test]
    fn test_position_string() {
        let results = SpellcheckResults {
            errors: vec![
                DocumentSpellingError::new(NodeId::new(), 0, 3, "a".to_string(), vec![]),
                DocumentSpellingError::new(NodeId::new(), 5, 8, "b".to_string(), vec![]),
            ],
            current_index: Some(0),
            words_checked: 100,
        };

        assert_eq!(results.position_string(), "1 of 2");
    }

    #[test]
    fn test_empty_results() {
        let results = SpellcheckResults::new();

        assert!(!results.has_errors());
        assert!(results.current().is_none());
        assert_eq!(results.position_string(), "0 of 0");
    }
}
