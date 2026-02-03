//! Command execution engine

use crate::{Command, Result, UndoManager};
use doc_model::{DocumentTree, Node, Selection};

/// The main editing engine that manages document state and command execution
pub struct EditingEngine {
    /// Current document tree
    tree: DocumentTree,
    /// Current selection
    selection: Selection,
    /// Undo manager
    undo_manager: UndoManager,
}

impl EditingEngine {
    /// Create a new editing engine with an empty document
    pub fn new() -> Self {
        let tree = DocumentTree::default();
        let selection = if let Some(para) = tree.document.children().first() {
            Selection::at_start_of(*para)
        } else {
            Selection::default()
        };

        Self {
            tree,
            selection,
            undo_manager: UndoManager::new(),
        }
    }

    /// Create an editing engine with a specific document tree
    pub fn with_tree(tree: DocumentTree) -> Self {
        let selection = if let Some(para) = tree.document.children().first() {
            Selection::at_start_of(*para)
        } else {
            Selection::default()
        };

        Self {
            tree,
            selection,
            undo_manager: UndoManager::new(),
        }
    }

    /// Get the current document tree
    pub fn tree(&self) -> &DocumentTree {
        &self.tree
    }

    /// Get the current selection
    pub fn selection(&self) -> Selection {
        self.selection
    }

    /// Set the selection
    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = selection;
    }

    /// Execute a command
    pub fn execute(&mut self, command: Box<dyn Command>) -> Result<()> {
        let result = command.apply(&self.tree, &self.selection)?;

        // Record for undo
        self.undo_manager.push(command, result.inverse);

        // Update state
        self.tree = result.tree;
        self.selection = result.selection;

        Ok(())
    }

    /// Undo the last command
    pub fn undo(&mut self) -> Result<()> {
        let inverse = self.undo_manager.pop_undo()?;
        let result = inverse.apply(&self.tree, &self.selection)?;

        self.tree = result.tree;
        self.selection = result.selection;

        Ok(())
    }

    /// Redo the last undone command
    pub fn redo(&mut self) -> Result<()> {
        let command = self.undo_manager.pop_redo()?;
        let result = command.apply(&self.tree, &self.selection)?;

        self.tree = result.tree;
        self.selection = result.selection;

        Ok(())
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.undo_manager.can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.undo_manager.can_redo()
    }
}

impl Default for EditingEngine {
    fn default() -> Self {
        Self::new()
    }
}
