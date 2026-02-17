//! Bookmark - marks a location or range in the document
//!
//! Bookmarks provide named anchors that can be referenced by hyperlinks
//! for internal document navigation.

use crate::{NodeId, Position};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The range a bookmark covers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookmarkRange {
    /// Point bookmark - marks a single position
    Point(Position),
    /// Range bookmark - marks a span of content
    Range { start: Position, end: Position },
}

/// Errors that can occur during bookmark validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookmarkValidationError {
    /// Name is empty
    EmptyName,
    /// Name contains invalid characters
    InvalidCharacters,
    /// Name doesn't start with a letter
    MustStartWithLetter,
    /// Name already exists
    DuplicateName,
    /// Name is too long (max 40 characters)
    NameTooLong,
}

impl std::fmt::Display for BookmarkValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookmarkValidationError::EmptyName => write!(f, "Bookmark name cannot be empty"),
            BookmarkValidationError::InvalidCharacters => {
                write!(f, "Bookmark name can only contain letters, numbers, and underscores")
            }
            BookmarkValidationError::MustStartWithLetter => {
                write!(f, "Bookmark name must start with a letter")
            }
            BookmarkValidationError::DuplicateName => {
                write!(f, "A bookmark with this name already exists")
            }
            BookmarkValidationError::NameTooLong => {
                write!(f, "Bookmark name cannot exceed 40 characters")
            }
        }
    }
}

impl std::error::Error for BookmarkValidationError {}

/// Maximum bookmark name length
pub const MAX_BOOKMARK_NAME_LENGTH: usize = 40;

/// Validate a bookmark name
pub fn validate_bookmark_name(name: &str) -> Result<(), BookmarkValidationError> {
    // Check empty
    if name.is_empty() {
        return Err(BookmarkValidationError::EmptyName);
    }

    // Check length
    if name.len() > MAX_BOOKMARK_NAME_LENGTH {
        return Err(BookmarkValidationError::NameTooLong);
    }

    // Check first character is a letter
    let first_char = name.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() {
        return Err(BookmarkValidationError::MustStartWithLetter);
    }

    // Check all characters are alphanumeric or underscore
    for c in name.chars() {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return Err(BookmarkValidationError::InvalidCharacters);
        }
    }

    Ok(())
}

/// A bookmark marks a location or range in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// Unique identifier for this bookmark
    id: NodeId,
    /// User-visible name for the bookmark (must be unique within document)
    name: String,
    /// The range this bookmark covers
    range: BookmarkRange,
}

impl Bookmark {
    /// Create a new point bookmark at a single position
    pub fn new_point(name: impl Into<String>, position: Position) -> Self {
        Self {
            id: NodeId::new(),
            name: name.into(),
            range: BookmarkRange::Point(position),
        }
    }

    /// Create a new range bookmark spanning from start to end
    pub fn new_range(name: impl Into<String>, start: Position, end: Position) -> Self {
        Self {
            id: NodeId::new(),
            name: name.into(),
            range: BookmarkRange::Range { start, end },
        }
    }

    /// Create a bookmark from a selection (point if collapsed, range if not)
    pub fn from_selection(name: impl Into<String>, anchor: Position, focus: Position) -> Self {
        if anchor == focus {
            Self::new_point(name, anchor)
        } else {
            // Normalize so start is before end
            // For now, simple comparison - full implementation needs document context
            let (start, end) = if anchor.node_id == focus.node_id {
                if anchor.offset <= focus.offset {
                    (anchor, focus)
                } else {
                    (focus, anchor)
                }
            } else {
                // Cross-node - use as-is (proper ordering requires document context)
                (anchor, focus)
            };
            Self::new_range(name, start, end)
        }
    }

    /// Get the bookmark ID
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Get the bookmark name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the bookmark name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Get the bookmark range
    pub fn range(&self) -> &BookmarkRange {
        &self.range
    }

    /// Set the bookmark range
    pub fn set_range(&mut self, range: BookmarkRange) {
        self.range = range;
    }

    /// Check if this is a point bookmark
    pub fn is_point(&self) -> bool {
        matches!(self.range, BookmarkRange::Point(_))
    }

    /// Check if this is a range bookmark
    pub fn is_range(&self) -> bool {
        matches!(self.range, BookmarkRange::Range { .. })
    }

    /// Get the start position of the bookmark
    pub fn start_position(&self) -> Position {
        match &self.range {
            BookmarkRange::Point(pos) => *pos,
            BookmarkRange::Range { start, .. } => *start,
        }
    }

    /// Get the end position of the bookmark (same as start for point bookmarks)
    pub fn end_position(&self) -> Position {
        match &self.range {
            BookmarkRange::Point(pos) => *pos,
            BookmarkRange::Range { end, .. } => *end,
        }
    }

    /// Check if a position is within this bookmark's range
    pub fn contains(&self, position: &Position) -> bool {
        match &self.range {
            BookmarkRange::Point(pos) => pos == position,
            BookmarkRange::Range { start, end } => {
                // Simple check for same-node ranges
                if position.node_id == start.node_id && position.node_id == end.node_id {
                    position.offset >= start.offset && position.offset <= end.offset
                } else if position.node_id == start.node_id {
                    position.offset >= start.offset
                } else if position.node_id == end.node_id {
                    position.offset <= end.offset
                } else {
                    // Position in a different node - would need document context
                    // to determine if it's between start and end nodes
                    false
                }
            }
        }
    }

    /// Validate this bookmark's name
    pub fn validate(&self) -> Result<(), BookmarkValidationError> {
        validate_bookmark_name(&self.name)
    }
}

/// Registry for managing bookmarks within a document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookmarkRegistry {
    /// Bookmarks indexed by ID
    bookmarks: HashMap<NodeId, Bookmark>,
    /// Name to ID mapping for fast lookup by name
    name_index: HashMap<String, NodeId>,
}

impl BookmarkRegistry {
    /// Create a new empty bookmark registry
    pub fn new() -> Self {
        Self {
            bookmarks: HashMap::new(),
            name_index: HashMap::new(),
        }
    }

    /// Insert a bookmark into the registry
    ///
    /// Returns an error if a bookmark with the same name already exists
    pub fn insert(&mut self, bookmark: Bookmark) -> Result<NodeId, BookmarkValidationError> {
        // Validate the name first
        bookmark.validate()?;

        // Check for duplicate name
        if self.name_index.contains_key(&bookmark.name) {
            return Err(BookmarkValidationError::DuplicateName);
        }

        let id = bookmark.id();
        let name = bookmark.name.clone();

        self.bookmarks.insert(id, bookmark);
        self.name_index.insert(name, id);

        Ok(id)
    }

    /// Remove a bookmark by ID
    pub fn remove(&mut self, id: NodeId) -> Option<Bookmark> {
        if let Some(bookmark) = self.bookmarks.remove(&id) {
            self.name_index.remove(&bookmark.name);
            Some(bookmark)
        } else {
            None
        }
    }

    /// Remove a bookmark by name
    pub fn remove_by_name(&mut self, name: &str) -> Option<Bookmark> {
        if let Some(id) = self.name_index.remove(name) {
            self.bookmarks.remove(&id)
        } else {
            None
        }
    }

    /// Get a bookmark by ID
    pub fn get(&self, id: NodeId) -> Option<&Bookmark> {
        self.bookmarks.get(&id)
    }

    /// Get a mutable bookmark by ID
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Bookmark> {
        self.bookmarks.get_mut(&id)
    }

    /// Get a bookmark by name
    pub fn get_by_name(&self, name: &str) -> Option<&Bookmark> {
        self.name_index
            .get(name)
            .and_then(|id| self.bookmarks.get(id))
    }

    /// Get a mutable bookmark by name
    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut Bookmark> {
        if let Some(id) = self.name_index.get(name).copied() {
            self.bookmarks.get_mut(&id)
        } else {
            None
        }
    }

    /// Check if a bookmark with the given name exists
    pub fn contains_name(&self, name: &str) -> bool {
        self.name_index.contains_key(name)
    }

    /// Rename a bookmark
    ///
    /// Returns an error if the new name is invalid or already in use
    pub fn rename(
        &mut self,
        id: NodeId,
        new_name: impl Into<String>,
    ) -> Result<(), BookmarkValidationError> {
        let new_name = new_name.into();

        // Validate the new name
        validate_bookmark_name(&new_name)?;

        // Get the bookmark
        let bookmark = self
            .bookmarks
            .get(&id)
            .ok_or(BookmarkValidationError::EmptyName)?;

        let old_name = bookmark.name.clone();

        // Check if new name is different
        if old_name == new_name {
            return Ok(());
        }

        // Check for duplicate name
        if self.name_index.contains_key(&new_name) {
            return Err(BookmarkValidationError::DuplicateName);
        }

        // Update the name
        self.name_index.remove(&old_name);
        self.name_index.insert(new_name.clone(), id);

        if let Some(bookmark) = self.bookmarks.get_mut(&id) {
            bookmark.set_name(new_name);
        }

        Ok(())
    }

    /// Get all bookmarks
    pub fn all(&self) -> impl Iterator<Item = &Bookmark> {
        self.bookmarks.values()
    }

    /// Get all bookmark names sorted alphabetically
    pub fn names_sorted(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.name_index.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Get the number of bookmarks
    pub fn len(&self) -> usize {
        self.bookmarks.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.bookmarks.is_empty()
    }

    /// Clear all bookmarks
    pub fn clear(&mut self) {
        self.bookmarks.clear();
        self.name_index.clear();
    }

    /// Find bookmarks at or containing a position
    pub fn find_at_position(&self, position: &Position) -> Vec<&Bookmark> {
        self.bookmarks
            .values()
            .filter(|b| b.contains(position))
            .collect()
    }

    /// Find bookmarks by paragraph node ID
    pub fn find_in_paragraph(&self, para_id: NodeId) -> Vec<&Bookmark> {
        self.bookmarks
            .values()
            .filter(|b| {
                let start = b.start_position();
                let end = b.end_position();
                start.node_id == para_id || end.node_id == para_id
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_position(node_id: NodeId, offset: usize) -> Position {
        Position::new(node_id, offset)
    }

    #[test]
    fn test_validate_bookmark_name() {
        // Valid names
        assert!(validate_bookmark_name("section1").is_ok());
        assert!(validate_bookmark_name("Chapter_1").is_ok());
        assert!(validate_bookmark_name("a").is_ok());
        assert!(validate_bookmark_name("my_bookmark_123").is_ok());

        // Invalid: empty
        assert!(matches!(
            validate_bookmark_name(""),
            Err(BookmarkValidationError::EmptyName)
        ));

        // Invalid: starts with number
        assert!(matches!(
            validate_bookmark_name("1section"),
            Err(BookmarkValidationError::MustStartWithLetter)
        ));

        // Invalid: starts with underscore
        assert!(matches!(
            validate_bookmark_name("_section"),
            Err(BookmarkValidationError::MustStartWithLetter)
        ));

        // Invalid: contains spaces
        assert!(matches!(
            validate_bookmark_name("my section"),
            Err(BookmarkValidationError::InvalidCharacters)
        ));

        // Invalid: contains special characters
        assert!(matches!(
            validate_bookmark_name("my-section"),
            Err(BookmarkValidationError::InvalidCharacters)
        ));

        // Invalid: too long
        let long_name = "a".repeat(MAX_BOOKMARK_NAME_LENGTH + 1);
        assert!(matches!(
            validate_bookmark_name(&long_name),
            Err(BookmarkValidationError::NameTooLong)
        ));
    }

    #[test]
    fn test_point_bookmark() {
        let node_id = NodeId::new();
        let pos = make_position(node_id, 5);

        let bookmark = Bookmark::new_point("chapter1", pos);

        assert!(bookmark.is_point());
        assert!(!bookmark.is_range());
        assert_eq!(bookmark.name(), "chapter1");
        assert_eq!(bookmark.start_position(), pos);
        assert_eq!(bookmark.end_position(), pos);
        assert!(bookmark.contains(&pos));
        assert!(!bookmark.contains(&make_position(node_id, 4)));
    }

    #[test]
    fn test_range_bookmark() {
        let node_id = NodeId::new();
        let start = make_position(node_id, 5);
        let end = make_position(node_id, 15);

        let bookmark = Bookmark::new_range("selection1", start, end);

        assert!(!bookmark.is_point());
        assert!(bookmark.is_range());
        assert_eq!(bookmark.name(), "selection1");
        assert_eq!(bookmark.start_position(), start);
        assert_eq!(bookmark.end_position(), end);

        // Contains checks
        assert!(bookmark.contains(&start));
        assert!(bookmark.contains(&end));
        assert!(bookmark.contains(&make_position(node_id, 10)));
        assert!(!bookmark.contains(&make_position(node_id, 4)));
        assert!(!bookmark.contains(&make_position(node_id, 16)));
    }

    #[test]
    fn test_bookmark_registry() {
        let mut registry = BookmarkRegistry::new();
        let node_id = NodeId::new();
        let pos = make_position(node_id, 5);

        // Insert bookmark
        let bookmark = Bookmark::new_point("section1", pos);
        let id = registry.insert(bookmark).unwrap();

        // Retrieve by ID
        assert!(registry.get(id).is_some());
        assert_eq!(registry.get(id).unwrap().name(), "section1");

        // Retrieve by name
        assert!(registry.get_by_name("section1").is_some());
        assert_eq!(registry.get_by_name("section1").unwrap().id(), id);

        // Contains check
        assert!(registry.contains_name("section1"));
        assert!(!registry.contains_name("nonexistent"));

        // Duplicate name should fail
        let duplicate = Bookmark::new_point("section1", pos);
        assert!(matches!(
            registry.insert(duplicate),
            Err(BookmarkValidationError::DuplicateName)
        ));

        // Count
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_bookmark_rename() {
        let mut registry = BookmarkRegistry::new();
        let node_id = NodeId::new();
        let pos = make_position(node_id, 5);

        let bookmark = Bookmark::new_point("old_name", pos);
        let id = registry.insert(bookmark).unwrap();

        // Rename
        registry.rename(id, "new_name").unwrap();
        assert!(!registry.contains_name("old_name"));
        assert!(registry.contains_name("new_name"));
        assert_eq!(registry.get(id).unwrap().name(), "new_name");

        // Rename to existing name should fail
        let another = Bookmark::new_point("another", pos);
        registry.insert(another).unwrap();
        assert!(matches!(
            registry.rename(id, "another"),
            Err(BookmarkValidationError::DuplicateName)
        ));
    }

    #[test]
    fn test_bookmark_removal() {
        let mut registry = BookmarkRegistry::new();
        let node_id = NodeId::new();
        let pos = make_position(node_id, 5);

        let bookmark = Bookmark::new_point("to_remove", pos);
        let id = registry.insert(bookmark).unwrap();

        // Remove by ID
        let removed = registry.remove(id);
        assert!(removed.is_some());
        assert!(registry.get(id).is_none());
        assert!(!registry.contains_name("to_remove"));

        // Add again and remove by name
        let bookmark2 = Bookmark::new_point("by_name", pos);
        registry.insert(bookmark2).unwrap();

        let removed2 = registry.remove_by_name("by_name");
        assert!(removed2.is_some());
        assert!(!registry.contains_name("by_name"));
    }

    #[test]
    fn test_from_selection() {
        let node_id = NodeId::new();
        let pos1 = make_position(node_id, 5);
        let pos2 = make_position(node_id, 15);

        // Collapsed selection -> point bookmark
        let point = Bookmark::from_selection("collapsed", pos1, pos1);
        assert!(point.is_point());

        // Expanded selection -> range bookmark
        let range = Bookmark::from_selection("expanded", pos1, pos2);
        assert!(range.is_range());

        // Reversed selection -> should normalize
        let reversed = Bookmark::from_selection("reversed", pos2, pos1);
        assert!(reversed.is_range());
        assert_eq!(reversed.start_position().offset, 5);
        assert_eq!(reversed.end_position().offset, 15);
    }
}
