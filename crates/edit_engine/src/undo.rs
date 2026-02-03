//! Undo/redo manager with command batching

use crate::{Command, EditError, Result};
use std::time::{Duration, Instant};

/// An entry in the undo stack
struct UndoEntry {
    /// The original command
    command: Box<dyn Command>,
    /// The inverse command (for undo)
    inverse: Box<dyn Command>,
    /// When this entry was created
    timestamp: Instant,
}

/// Manages undo and redo stacks
pub struct UndoManager {
    /// Stack of commands that can be undone
    undo_stack: Vec<UndoEntry>,
    /// Stack of commands that can be redone
    redo_stack: Vec<Box<dyn Command>>,
    /// Maximum number of undo entries
    max_entries: usize,
    /// Time threshold for batching (commands within this time are merged)
    batch_threshold: Duration,
    /// Whether we're currently in an IME composition
    in_composition: bool,
}

impl UndoManager {
    /// Create a new undo manager
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_entries: 100,
            batch_threshold: Duration::from_millis(500),
            in_composition: false,
        }
    }

    /// Create with custom limits
    pub fn with_limits(max_entries: usize, batch_threshold: Duration) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_entries,
            batch_threshold,
            in_composition: false,
        }
    }

    /// Push a command onto the undo stack
    pub fn push(&mut self, command: Box<dyn Command>, inverse: Box<dyn Command>) {
        // Clear redo stack on new command
        self.redo_stack.clear();

        let now = Instant::now();

        // Try to merge with previous command if within batch threshold
        if let Some(last) = self.undo_stack.last_mut() {
            if !self.in_composition && now.duration_since(last.timestamp) < self.batch_threshold {
                if let Some(merged) = last.command.merge_with(command.as_ref()) {
                    last.command = merged;
                    last.inverse = inverse;
                    last.timestamp = now;
                    return;
                }
            }
        }

        // Add new entry
        self.undo_stack.push(UndoEntry {
            command,
            inverse,
            timestamp: now,
        });

        // Enforce max entries
        while self.undo_stack.len() > self.max_entries {
            self.undo_stack.remove(0);
        }
    }

    /// Pop the last command for undo
    pub fn pop_undo(&mut self) -> Result<Box<dyn Command>> {
        let entry = self.undo_stack.pop()
            .ok_or(EditError::UndoStackEmpty)?;

        // Push to redo stack
        self.redo_stack.push(entry.command);

        Ok(entry.inverse)
    }

    /// Pop a command for redo
    pub fn pop_redo(&mut self) -> Result<Box<dyn Command>> {
        self.redo_stack.pop()
            .ok_or(EditError::RedoStackEmpty)
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Start IME composition (disables batching)
    pub fn begin_composition(&mut self) {
        self.in_composition = true;
    }

    /// End IME composition
    pub fn end_composition(&mut self) {
        self.in_composition = false;
    }

    /// Check if in composition
    pub fn in_composition(&self) -> bool {
        self.in_composition
    }

    /// Clear all undo/redo history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new()
    }
}
