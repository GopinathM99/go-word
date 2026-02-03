//! Footnote and Endnote Model
//!
//! This module implements footnotes and endnotes for the word processor.
//!
//! ## Features
//!
//! - Footnotes appear at the bottom of the page containing the reference
//! - Endnotes are collected at the end of a section or document
//! - Configurable numbering schemes (Arabic, Roman, letters, symbols)
//! - Automatic or custom note marks
//! - Restart numbering per page, section, or continuous
//! - Note continuation across pages with separator lines

use crate::{Node, NodeId, NodeType, Position};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// =============================================================================
// Note ID
// =============================================================================

/// Unique identifier for a footnote or endnote
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NoteId(Uuid);

impl NoteId {
    /// Create a new random NoteId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a NoteId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Create a NoteId from a string representation
    pub fn from_string(s: &str) -> Option<Self> {
        Uuid::parse_str(s).ok().map(Self)
    }
}

impl Default for NoteId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NoteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for NoteId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<NoteId> for Uuid {
    fn from(id: NoteId) -> Self {
        id.0
    }
}

// =============================================================================
// Note Type
// =============================================================================

/// Type of note (footnote vs endnote)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NoteType {
    /// Footnote - appears at bottom of page
    Footnote,
    /// Endnote - appears at end of section or document
    Endnote,
}

impl Default for NoteType {
    fn default() -> Self {
        NoteType::Footnote
    }
}

// =============================================================================
// Numbering Scheme
// =============================================================================

/// Numbering scheme for footnotes/endnotes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberingScheme {
    /// Arabic numerals: 1, 2, 3, ...
    Arabic,
    /// Lowercase Roman numerals: i, ii, iii, iv, ...
    LowerRoman,
    /// Uppercase Roman numerals: I, II, III, IV, ...
    UpperRoman,
    /// Lowercase letters: a, b, c, ...
    LowerLetter,
    /// Uppercase letters: A, B, C, ...
    UpperLetter,
    /// Symbols: *, dagger, double dagger, section, etc.
    Symbols,
}

impl Default for NumberingScheme {
    fn default() -> Self {
        NumberingScheme::Arabic
    }
}

impl NumberingScheme {
    /// Format a number according to the scheme
    pub fn format(&self, number: u32) -> String {
        match self {
            NumberingScheme::Arabic => number.to_string(),
            NumberingScheme::LowerRoman => Self::to_roman(number, false),
            NumberingScheme::UpperRoman => Self::to_roman(number, true),
            NumberingScheme::LowerLetter => Self::to_letter(number, false),
            NumberingScheme::UpperLetter => Self::to_letter(number, true),
            NumberingScheme::Symbols => Self::to_symbol(number),
        }
    }

    /// Convert a number to Roman numerals
    fn to_roman(mut n: u32, uppercase: bool) -> String {
        if n == 0 || n > 3999 {
            return n.to_string(); // Fallback for out of range
        }

        let values = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
        let numerals_upper = [
            "M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I",
        ];
        let numerals_lower = [
            "m", "cm", "d", "cd", "c", "xc", "l", "xl", "x", "ix", "v", "iv", "i",
        ];

        let numerals = if uppercase {
            numerals_upper
        } else {
            numerals_lower
        };

        let mut result = String::new();
        for (i, &value) in values.iter().enumerate() {
            while n >= value {
                result.push_str(numerals[i]);
                n -= value;
            }
        }

        result
    }

    /// Convert a number to letter (a=1, b=2, ..., z=26, aa=27, ...)
    fn to_letter(n: u32, uppercase: bool) -> String {
        if n == 0 {
            return String::new();
        }

        let mut n = n;
        let mut result = String::new();
        let base = if uppercase { b'A' } else { b'a' };

        while n > 0 {
            n -= 1;
            let c = base + (n % 26) as u8;
            result.insert(0, c as char);
            n /= 26;
        }

        result
    }

    /// Convert a number to a symbol
    /// Uses cycle: * dagger double-dagger section paragraph double-bar
    /// After 6, repeats with double: ** dagger-dagger etc.
    fn to_symbol(n: u32) -> String {
        if n == 0 {
            return String::new();
        }

        const SYMBOLS: [char; 6] = [
            '*',      // asterisk
            '\u{2020}', // dagger
            '\u{2021}', // double dagger
            '\u{00A7}', // section sign
            '\u{00B6}', // pilcrow (paragraph)
            '\u{2016}', // double vertical bar
        ];

        let cycle = ((n - 1) / 6) + 1;
        let index = ((n - 1) % 6) as usize;
        let symbol = SYMBOLS[index];

        std::iter::repeat(symbol).take(cycle as usize).collect()
    }
}

// =============================================================================
// Restart Numbering
// =============================================================================

/// When to restart note numbering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestartNumbering {
    /// Continuous numbering throughout the document
    Continuous,
    /// Restart at each section
    PerSection,
    /// Restart at each page (footnotes only)
    PerPage,
}

impl Default for RestartNumbering {
    fn default() -> Self {
        RestartNumbering::Continuous
    }
}

// =============================================================================
// Footnote Position
// =============================================================================

/// Position of footnotes on the page
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FootnotePosition {
    /// At the very bottom of the page
    PageBottom,
    /// Directly beneath the text content
    BeneathText,
}

impl Default for FootnotePosition {
    fn default() -> Self {
        FootnotePosition::PageBottom
    }
}

// =============================================================================
// Endnote Position
// =============================================================================

/// Position of endnotes in the document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndnotePosition {
    /// At the end of each section
    EndOfSection,
    /// At the end of the entire document
    EndOfDocument,
}

impl Default for EndnotePosition {
    fn default() -> Self {
        EndnotePosition::EndOfDocument
    }
}

// =============================================================================
// Footnote Properties
// =============================================================================

/// Properties for footnote formatting in a section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteProperties {
    /// Numbering scheme
    pub numbering: NumberingScheme,
    /// When to restart numbering
    pub restart: RestartNumbering,
    /// Starting number
    pub start_at: u32,
    /// Position on page
    pub position: FootnotePosition,
    /// Space between text and footnote area (in points)
    pub space_before: f32,
    /// Whether to show separator line
    pub show_separator: bool,
    /// Separator line length as fraction of page width (0.0 to 1.0)
    pub separator_length: f32,
    /// Separator line weight (in points)
    pub separator_weight: f32,
}

impl Default for FootnoteProperties {
    fn default() -> Self {
        Self {
            numbering: NumberingScheme::Arabic,
            restart: RestartNumbering::Continuous,
            start_at: 1,
            position: FootnotePosition::PageBottom,
            space_before: 12.0, // 12pt space before footnote area
            show_separator: true,
            separator_length: 0.33, // 1/3 of page width
            separator_weight: 0.5,  // 0.5pt line
        }
    }
}

// =============================================================================
// Endnote Properties
// =============================================================================

/// Properties for endnote formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndnoteProperties {
    /// Numbering scheme
    pub numbering: NumberingScheme,
    /// When to restart numbering
    pub restart: RestartNumbering,
    /// Starting number
    pub start_at: u32,
    /// Position of endnotes
    pub position: EndnotePosition,
}

impl Default for EndnoteProperties {
    fn default() -> Self {
        Self {
            numbering: NumberingScheme::LowerRoman,
            restart: RestartNumbering::Continuous,
            start_at: 1,
            position: EndnotePosition::EndOfDocument,
        }
    }
}

// =============================================================================
// Note Reference
// =============================================================================

/// A reference to a footnote or endnote in the document text
/// This is an inline element that appears in the paragraph where the note is referenced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteRef {
    /// Unique ID for this reference node
    id: NodeId,
    /// Parent paragraph/run ID
    parent: Option<NodeId>,
    /// Type of note being referenced
    pub note_type: NoteType,
    /// ID of the note this reference points to
    pub note_id: NoteId,
    /// Custom mark (overrides auto-numbering if set)
    pub custom_mark: Option<String>,
}

impl NoteRef {
    /// Create a new footnote reference
    pub fn footnote(note_id: NoteId) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            note_type: NoteType::Footnote,
            note_id,
            custom_mark: None,
        }
    }

    /// Create a new endnote reference
    pub fn endnote(note_id: NoteId) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            note_type: NoteType::Endnote,
            note_id,
            custom_mark: None,
        }
    }

    /// Create a reference with a custom mark
    pub fn with_custom_mark(mut self, mark: impl Into<String>) -> Self {
        self.custom_mark = Some(mark.into());
        self
    }

    /// Get the note ID this reference points to
    pub fn note_id(&self) -> NoteId {
        self.note_id
    }

    /// Check if this reference has a custom mark
    pub fn has_custom_mark(&self) -> bool {
        self.custom_mark.is_some()
    }

    /// Get the custom mark if set
    pub fn custom_mark(&self) -> Option<&str> {
        self.custom_mark.as_deref()
    }

    /// Set a custom mark
    pub fn set_custom_mark(&mut self, mark: Option<String>) {
        self.custom_mark = mark;
    }
}

impl Node for NoteRef {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Field // Using Field type for inline reference
    }

    fn children(&self) -> &[NodeId] {
        &[] // Note references have no children
    }

    fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    fn set_parent(&mut self, parent: Option<NodeId>) {
        self.parent = parent;
    }

    fn can_have_children(&self) -> bool {
        false
    }
}

// =============================================================================
// Note
// =============================================================================

/// A footnote or endnote containing content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Unique ID for this note
    id: NoteId,
    /// Type of note
    pub note_type: NoteType,
    /// The formatted mark (e.g., "1", "i", "*")
    /// This is computed based on numbering scheme and position
    pub mark: String,
    /// Content paragraphs inside the note
    children: Vec<NodeId>,
    /// Position of the reference in the document
    /// This is used for layout ordering
    pub reference_position: Option<Position>,
    /// Section ID where this note's reference appears
    pub section_id: Option<NodeId>,
    /// Page index where the reference appears (set during layout)
    pub reference_page: Option<usize>,
}

impl Note {
    /// Create a new footnote
    pub fn footnote() -> Self {
        Self {
            id: NoteId::new(),
            note_type: NoteType::Footnote,
            mark: String::new(),
            children: Vec::new(),
            reference_position: None,
            section_id: None,
            reference_page: None,
        }
    }

    /// Create a new endnote
    pub fn endnote() -> Self {
        Self {
            id: NoteId::new(),
            note_type: NoteType::Endnote,
            mark: String::new(),
            children: Vec::new(),
            reference_position: None,
            section_id: None,
            reference_page: None,
        }
    }

    /// Get the note ID
    pub fn id(&self) -> NoteId {
        self.id
    }

    /// Get the mark
    pub fn mark(&self) -> &str {
        &self.mark
    }

    /// Set the mark
    pub fn set_mark(&mut self, mark: impl Into<String>) {
        self.mark = mark.into();
    }

    /// Get the content paragraph IDs
    pub fn content(&self) -> &[NodeId] {
        &self.children
    }

    /// Add a content paragraph
    pub fn add_content(&mut self, para_id: NodeId) {
        self.children.push(para_id);
    }

    /// Insert a content paragraph at a specific index
    pub fn insert_content(&mut self, index: usize, para_id: NodeId) {
        self.children.insert(index, para_id);
    }

    /// Remove a content paragraph
    pub fn remove_content(&mut self, para_id: NodeId) -> bool {
        if let Some(pos) = self.children.iter().position(|&id| id == para_id) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }

    /// Clear all content
    pub fn clear_content(&mut self) {
        self.children.clear();
    }

    /// Check if the note has any content
    pub fn has_content(&self) -> bool {
        !self.children.is_empty()
    }

    /// Check if this is a footnote
    pub fn is_footnote(&self) -> bool {
        self.note_type == NoteType::Footnote
    }

    /// Check if this is an endnote
    pub fn is_endnote(&self) -> bool {
        self.note_type == NoteType::Endnote
    }

    /// Set the reference position
    pub fn set_reference_position(&mut self, position: Position) {
        self.reference_position = Some(position);
    }

    /// Set the section ID
    pub fn set_section(&mut self, section_id: NodeId) {
        self.section_id = Some(section_id);
    }

    /// Set the reference page (during layout)
    pub fn set_reference_page(&mut self, page: usize) {
        self.reference_page = Some(page);
    }

    /// Convert this note to the other type
    pub fn convert_type(&mut self) {
        self.note_type = match self.note_type {
            NoteType::Footnote => NoteType::Endnote,
            NoteType::Endnote => NoteType::Footnote,
        };
    }
}

// =============================================================================
// Note Store
// =============================================================================

/// Storage for all footnotes and endnotes in a document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoteStore {
    /// All footnotes indexed by ID
    footnotes: HashMap<NoteId, Note>,
    /// All endnotes indexed by ID
    endnotes: HashMap<NoteId, Note>,
    /// All note references indexed by node ID
    references: HashMap<NodeId, NoteRef>,
    /// Footnote properties for the document
    pub footnote_props: FootnoteProperties,
    /// Endnote properties for the document
    pub endnote_props: EndnoteProperties,
    /// Section-specific footnote properties
    section_footnote_props: HashMap<NodeId, FootnoteProperties>,
    /// Section-specific endnote properties
    section_endnote_props: HashMap<NodeId, EndnoteProperties>,
}

impl NoteStore {
    /// Create a new empty note store
    pub fn new() -> Self {
        Self::default()
    }

    // -------------------------------------------------------------------------
    // Footnote operations
    // -------------------------------------------------------------------------

    /// Insert a footnote
    pub fn insert_footnote(&mut self, note: Note) -> NoteId {
        let id = note.id();
        self.footnotes.insert(id, note);
        id
    }

    /// Get a footnote by ID
    pub fn get_footnote(&self, id: NoteId) -> Option<&Note> {
        self.footnotes.get(&id)
    }

    /// Get a mutable footnote by ID
    pub fn get_footnote_mut(&mut self, id: NoteId) -> Option<&mut Note> {
        self.footnotes.get_mut(&id)
    }

    /// Remove a footnote
    pub fn remove_footnote(&mut self, id: NoteId) -> Option<Note> {
        self.footnotes.remove(&id)
    }

    /// Get all footnotes
    pub fn footnotes(&self) -> impl Iterator<Item = &Note> {
        self.footnotes.values()
    }

    /// Get footnotes for a specific page
    pub fn footnotes_on_page(&self, page: usize) -> Vec<&Note> {
        self.footnotes
            .values()
            .filter(|n| n.reference_page == Some(page))
            .collect()
    }

    /// Get footnotes for a specific section
    pub fn footnotes_in_section(&self, section_id: NodeId) -> Vec<&Note> {
        self.footnotes
            .values()
            .filter(|n| n.section_id == Some(section_id))
            .collect()
    }

    /// Get footnote count
    pub fn footnote_count(&self) -> usize {
        self.footnotes.len()
    }

    // -------------------------------------------------------------------------
    // Endnote operations
    // -------------------------------------------------------------------------

    /// Insert an endnote
    pub fn insert_endnote(&mut self, note: Note) -> NoteId {
        let id = note.id();
        self.endnotes.insert(id, note);
        id
    }

    /// Get an endnote by ID
    pub fn get_endnote(&self, id: NoteId) -> Option<&Note> {
        self.endnotes.get(&id)
    }

    /// Get a mutable endnote by ID
    pub fn get_endnote_mut(&mut self, id: NoteId) -> Option<&mut Note> {
        self.endnotes.get_mut(&id)
    }

    /// Remove an endnote
    pub fn remove_endnote(&mut self, id: NoteId) -> Option<Note> {
        self.endnotes.remove(&id)
    }

    /// Get all endnotes
    pub fn endnotes(&self) -> impl Iterator<Item = &Note> {
        self.endnotes.values()
    }

    /// Get endnotes for a specific section
    pub fn endnotes_in_section(&self, section_id: NodeId) -> Vec<&Note> {
        self.endnotes
            .values()
            .filter(|n| n.section_id == Some(section_id))
            .collect()
    }

    /// Get endnote count
    pub fn endnote_count(&self) -> usize {
        self.endnotes.len()
    }

    // -------------------------------------------------------------------------
    // Reference operations
    // -------------------------------------------------------------------------

    /// Insert a note reference
    pub fn insert_reference(&mut self, reference: NoteRef) -> NodeId {
        let id = reference.id();
        self.references.insert(id, reference);
        id
    }

    /// Get a reference by node ID
    pub fn get_reference(&self, id: NodeId) -> Option<&NoteRef> {
        self.references.get(&id)
    }

    /// Get a mutable reference by node ID
    pub fn get_reference_mut(&mut self, id: NodeId) -> Option<&mut NoteRef> {
        self.references.get_mut(&id)
    }

    /// Remove a reference
    pub fn remove_reference(&mut self, id: NodeId) -> Option<NoteRef> {
        self.references.remove(&id)
    }

    /// Get all references
    pub fn references(&self) -> impl Iterator<Item = &NoteRef> {
        self.references.values()
    }

    /// Find the reference for a note
    pub fn find_reference_for_note(&self, note_id: NoteId) -> Option<&NoteRef> {
        self.references.values().find(|r| r.note_id == note_id)
    }

    /// Find the reference node ID for a note
    pub fn find_reference_id_for_note(&self, note_id: NoteId) -> Option<NodeId> {
        self.references
            .iter()
            .find(|(_, r)| r.note_id == note_id)
            .map(|(id, _)| *id)
    }

    // -------------------------------------------------------------------------
    // Generic note operations
    // -------------------------------------------------------------------------

    /// Get a note (footnote or endnote) by ID
    pub fn get_note(&self, id: NoteId, note_type: NoteType) -> Option<&Note> {
        match note_type {
            NoteType::Footnote => self.get_footnote(id),
            NoteType::Endnote => self.get_endnote(id),
        }
    }

    /// Get a mutable note by ID
    pub fn get_note_mut(&mut self, id: NoteId, note_type: NoteType) -> Option<&mut Note> {
        match note_type {
            NoteType::Footnote => self.get_footnote_mut(id),
            NoteType::Endnote => self.get_endnote_mut(id),
        }
    }

    /// Remove a note by ID
    pub fn remove_note(&mut self, id: NoteId, note_type: NoteType) -> Option<Note> {
        match note_type {
            NoteType::Footnote => self.remove_footnote(id),
            NoteType::Endnote => self.remove_endnote(id),
        }
    }

    /// Convert a footnote to endnote (or vice versa)
    pub fn convert_note(&mut self, id: NoteId) -> Option<NoteId> {
        // Try footnote first
        if let Some(mut note) = self.footnotes.remove(&id) {
            note.convert_type();
            let new_id = note.id();
            self.endnotes.insert(new_id, note);

            // Update the reference
            if let Some(ref_node_id) = self.find_reference_id_for_note(id) {
                if let Some(reference) = self.references.get_mut(&ref_node_id) {
                    reference.note_type = NoteType::Endnote;
                    reference.note_id = new_id;
                }
            }

            return Some(new_id);
        }

        // Try endnote
        if let Some(mut note) = self.endnotes.remove(&id) {
            note.convert_type();
            let new_id = note.id();
            self.footnotes.insert(new_id, note);

            // Update the reference
            if let Some(ref_node_id) = self.find_reference_id_for_note(id) {
                if let Some(reference) = self.references.get_mut(&ref_node_id) {
                    reference.note_type = NoteType::Footnote;
                    reference.note_id = new_id;
                }
            }

            return Some(new_id);
        }

        None
    }

    // -------------------------------------------------------------------------
    // Properties operations
    // -------------------------------------------------------------------------

    /// Get footnote properties for a section (or default)
    pub fn get_footnote_props(&self, section_id: Option<NodeId>) -> &FootnoteProperties {
        section_id
            .and_then(|id| self.section_footnote_props.get(&id))
            .unwrap_or(&self.footnote_props)
    }

    /// Get mutable footnote properties for a section
    pub fn get_footnote_props_mut(
        &mut self,
        section_id: Option<NodeId>,
    ) -> &mut FootnoteProperties {
        match section_id {
            Some(id) => self
                .section_footnote_props
                .entry(id)
                .or_insert_with(|| self.footnote_props.clone()),
            None => &mut self.footnote_props,
        }
    }

    /// Set footnote properties for a section
    pub fn set_footnote_props(&mut self, section_id: Option<NodeId>, props: FootnoteProperties) {
        match section_id {
            Some(id) => {
                self.section_footnote_props.insert(id, props);
            }
            None => {
                self.footnote_props = props;
            }
        }
    }

    /// Get endnote properties for a section (or default)
    pub fn get_endnote_props(&self, section_id: Option<NodeId>) -> &EndnoteProperties {
        section_id
            .and_then(|id| self.section_endnote_props.get(&id))
            .unwrap_or(&self.endnote_props)
    }

    /// Get mutable endnote properties for a section
    pub fn get_endnote_props_mut(&mut self, section_id: Option<NodeId>) -> &mut EndnoteProperties {
        match section_id {
            Some(id) => self
                .section_endnote_props
                .entry(id)
                .or_insert_with(|| self.endnote_props.clone()),
            None => &mut self.endnote_props,
        }
    }

    /// Set endnote properties for a section
    pub fn set_endnote_props(&mut self, section_id: Option<NodeId>, props: EndnoteProperties) {
        match section_id {
            Some(id) => {
                self.section_endnote_props.insert(id, props);
            }
            None => {
                self.endnote_props = props;
            }
        }
    }

    // -------------------------------------------------------------------------
    // Numbering operations
    // -------------------------------------------------------------------------

    /// Renumber all footnotes according to properties
    pub fn renumber_footnotes(&mut self) {
        let props = self.footnote_props.clone();
        self.renumber_notes_internal(&props, NoteType::Footnote);
    }

    /// Renumber all endnotes according to properties
    pub fn renumber_endnotes(&mut self) {
        let props = self.endnote_props.clone();
        self.renumber_notes_internal_endnote(&props);
    }

    /// Internal helper to renumber footnotes
    fn renumber_notes_internal(&mut self, props: &FootnoteProperties, _note_type: NoteType) {
        // Collect footnotes sorted by reference position
        let mut sorted_ids: Vec<NoteId> = self.footnotes.keys().copied().collect();

        // Sort by reference position (if available)
        sorted_ids.sort_by(|a, b| {
            let note_a = self.footnotes.get(a);
            let note_b = self.footnotes.get(b);

            match (note_a, note_b) {
                (Some(na), Some(nb)) => {
                    match (&na.reference_position, &nb.reference_position) {
                        (Some(pos_a), Some(pos_b)) => {
                            // Compare by node ID first, then offset
                            match pos_a.node_id.as_uuid().cmp(&pos_b.node_id.as_uuid()) {
                                std::cmp::Ordering::Equal => pos_a.offset.cmp(&pos_b.offset),
                                other => other,
                            }
                        }
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                }
                _ => std::cmp::Ordering::Equal,
            }
        });

        // Assign numbers
        let mut counter = props.start_at;
        let mut last_page: Option<usize> = None;
        let mut last_section: Option<NodeId> = None;

        for id in sorted_ids {
            if let Some(note) = self.footnotes.get_mut(&id) {
                // Check for restart conditions
                let should_restart = match props.restart {
                    RestartNumbering::Continuous => false,
                    RestartNumbering::PerPage => {
                        let restart = note.reference_page != last_page && last_page.is_some();
                        last_page = note.reference_page;
                        restart
                    }
                    RestartNumbering::PerSection => {
                        let restart = note.section_id != last_section && last_section.is_some();
                        last_section = note.section_id;
                        restart
                    }
                };

                if should_restart {
                    counter = props.start_at;
                }

                // Format the mark
                note.mark = props.numbering.format(counter);
                counter += 1;
            }
        }
    }

    /// Internal helper to renumber endnotes
    fn renumber_notes_internal_endnote(&mut self, props: &EndnoteProperties) {
        // Collect endnotes sorted by reference position
        let mut sorted_ids: Vec<NoteId> = self.endnotes.keys().copied().collect();

        // Sort by reference position
        sorted_ids.sort_by(|a, b| {
            let note_a = self.endnotes.get(a);
            let note_b = self.endnotes.get(b);

            match (note_a, note_b) {
                (Some(na), Some(nb)) => {
                    match (&na.reference_position, &nb.reference_position) {
                        (Some(pos_a), Some(pos_b)) => {
                            match pos_a.node_id.as_uuid().cmp(&pos_b.node_id.as_uuid()) {
                                std::cmp::Ordering::Equal => pos_a.offset.cmp(&pos_b.offset),
                                other => other,
                            }
                        }
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                }
                _ => std::cmp::Ordering::Equal,
            }
        });

        // Assign numbers
        let mut counter = props.start_at;
        let mut last_section: Option<NodeId> = None;

        for id in sorted_ids {
            if let Some(note) = self.endnotes.get_mut(&id) {
                // Check for restart at section boundary
                let should_restart = match props.restart {
                    RestartNumbering::Continuous => false,
                    RestartNumbering::PerSection => {
                        let restart = note.section_id != last_section && last_section.is_some();
                        last_section = note.section_id;
                        restart
                    }
                    RestartNumbering::PerPage => {
                        // Per-page doesn't apply to endnotes, treat as continuous
                        false
                    }
                };

                if should_restart {
                    counter = props.start_at;
                }

                // Format the mark
                note.mark = props.numbering.format(counter);
                counter += 1;
            }
        }
    }

    /// Get all footnotes sorted by document order (for a page)
    pub fn get_footnotes_sorted(&self, page: usize) -> Vec<&Note> {
        let mut notes: Vec<&Note> = self
            .footnotes
            .values()
            .filter(|n| n.reference_page == Some(page))
            .collect();

        notes.sort_by(|a, b| {
            match (&a.reference_position, &b.reference_position) {
                (Some(pos_a), Some(pos_b)) => {
                    match pos_a.node_id.as_uuid().cmp(&pos_b.node_id.as_uuid()) {
                        std::cmp::Ordering::Equal => pos_a.offset.cmp(&pos_b.offset),
                        other => other,
                    }
                }
                _ => std::cmp::Ordering::Equal,
            }
        });

        notes
    }

    /// Get all endnotes sorted by document order (optionally for a section)
    pub fn get_endnotes_sorted(&self, section_id: Option<NodeId>) -> Vec<&Note> {
        let mut notes: Vec<&Note> = if let Some(sid) = section_id {
            self.endnotes
                .values()
                .filter(|n| n.section_id == Some(sid))
                .collect()
        } else {
            self.endnotes.values().collect()
        };

        notes.sort_by(|a, b| {
            match (&a.reference_position, &b.reference_position) {
                (Some(pos_a), Some(pos_b)) => {
                    match pos_a.node_id.as_uuid().cmp(&pos_b.node_id.as_uuid()) {
                        std::cmp::Ordering::Equal => pos_a.offset.cmp(&pos_b.offset),
                        other => other,
                    }
                }
                _ => std::cmp::Ordering::Equal,
            }
        });

        notes
    }

    /// Check if there are any footnotes
    pub fn has_footnotes(&self) -> bool {
        !self.footnotes.is_empty()
    }

    /// Check if there are any endnotes
    pub fn has_endnotes(&self) -> bool {
        !self.endnotes.is_empty()
    }

    /// Clear all notes and references
    pub fn clear(&mut self) {
        self.footnotes.clear();
        self.endnotes.clear();
        self.references.clear();
    }
}

// =============================================================================
// Continuation Notice
// =============================================================================

/// Information for footnote continuation across pages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteContinuation {
    /// Continuation notice text (e.g., "continued...")
    pub continuation_notice: String,
    /// Separator text for continued footnotes (e.g., "continued from previous page")
    pub continuation_separator: String,
}

impl Default for FootnoteContinuation {
    fn default() -> Self {
        Self {
            continuation_notice: "continued on next page".to_string(),
            continuation_separator: "continued from previous page".to_string(),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_id_creation() {
        let id1 = NoteId::new();
        let id2 = NoteId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_note_id_from_string() {
        let id = NoteId::new();
        let s = id.to_string();
        let parsed = NoteId::from_string(&s).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_numbering_scheme_arabic() {
        let scheme = NumberingScheme::Arabic;
        assert_eq!(scheme.format(1), "1");
        assert_eq!(scheme.format(10), "10");
        assert_eq!(scheme.format(100), "100");
    }

    #[test]
    fn test_numbering_scheme_roman() {
        let lower = NumberingScheme::LowerRoman;
        let upper = NumberingScheme::UpperRoman;

        assert_eq!(lower.format(1), "i");
        assert_eq!(lower.format(4), "iv");
        assert_eq!(lower.format(9), "ix");
        assert_eq!(lower.format(10), "x");
        assert_eq!(lower.format(50), "l");
        assert_eq!(lower.format(100), "c");

        assert_eq!(upper.format(1), "I");
        assert_eq!(upper.format(4), "IV");
        assert_eq!(upper.format(9), "IX");
        assert_eq!(upper.format(1999), "MCMXCIX");
    }

    #[test]
    fn test_numbering_scheme_letter() {
        let lower = NumberingScheme::LowerLetter;
        let upper = NumberingScheme::UpperLetter;

        assert_eq!(lower.format(1), "a");
        assert_eq!(lower.format(26), "z");
        assert_eq!(lower.format(27), "aa");
        assert_eq!(lower.format(52), "az");

        assert_eq!(upper.format(1), "A");
        assert_eq!(upper.format(26), "Z");
        assert_eq!(upper.format(27), "AA");
    }

    #[test]
    fn test_numbering_scheme_symbols() {
        let scheme = NumberingScheme::Symbols;

        assert_eq!(scheme.format(1), "*");
        assert_eq!(scheme.format(2), "\u{2020}"); // dagger
        assert_eq!(scheme.format(7), "**");       // second cycle
        assert_eq!(scheme.format(13), "***");     // third cycle
    }

    #[test]
    fn test_note_creation() {
        let footnote = Note::footnote();
        assert!(footnote.is_footnote());
        assert!(!footnote.is_endnote());
        assert!(!footnote.has_content());

        let endnote = Note::endnote();
        assert!(endnote.is_endnote());
        assert!(!endnote.is_footnote());
    }

    #[test]
    fn test_note_content() {
        let mut note = Note::footnote();
        let para_id = NodeId::new();

        note.add_content(para_id);
        assert!(note.has_content());
        assert_eq!(note.content().len(), 1);

        let removed = note.remove_content(para_id);
        assert!(removed);
        assert!(!note.has_content());
    }

    #[test]
    fn test_note_conversion() {
        let mut note = Note::footnote();
        assert!(note.is_footnote());

        note.convert_type();
        assert!(note.is_endnote());

        note.convert_type();
        assert!(note.is_footnote());
    }

    #[test]
    fn test_note_ref_creation() {
        let note_id = NoteId::new();

        let footnote_ref = NoteRef::footnote(note_id);
        assert_eq!(footnote_ref.note_type, NoteType::Footnote);
        assert_eq!(footnote_ref.note_id(), note_id);
        assert!(!footnote_ref.has_custom_mark());

        let custom_ref = NoteRef::footnote(note_id).with_custom_mark("*");
        assert!(custom_ref.has_custom_mark());
        assert_eq!(custom_ref.custom_mark(), Some("*"));
    }

    #[test]
    fn test_note_store_basic_operations() {
        let mut store = NoteStore::new();

        // Insert footnote
        let footnote = Note::footnote();
        let fn_id = footnote.id();
        store.insert_footnote(footnote);

        assert_eq!(store.footnote_count(), 1);
        assert!(store.get_footnote(fn_id).is_some());

        // Insert endnote
        let endnote = Note::endnote();
        let en_id = endnote.id();
        store.insert_endnote(endnote);

        assert_eq!(store.endnote_count(), 1);
        assert!(store.get_endnote(en_id).is_some());

        // Remove
        store.remove_footnote(fn_id);
        assert_eq!(store.footnote_count(), 0);
    }

    #[test]
    fn test_note_store_references() {
        let mut store = NoteStore::new();

        let footnote = Note::footnote();
        let note_id = footnote.id();
        store.insert_footnote(footnote);

        let reference = NoteRef::footnote(note_id);
        let ref_id = store.insert_reference(reference);

        assert!(store.get_reference(ref_id).is_some());
        assert!(store.find_reference_for_note(note_id).is_some());
    }

    #[test]
    fn test_note_store_conversion() {
        let mut store = NoteStore::new();

        let footnote = Note::footnote();
        let fn_id = footnote.id();
        store.insert_footnote(footnote);

        let reference = NoteRef::footnote(fn_id);
        store.insert_reference(reference);

        assert_eq!(store.footnote_count(), 1);
        assert_eq!(store.endnote_count(), 0);

        // Convert footnote to endnote
        let new_id = store.convert_note(fn_id);
        assert!(new_id.is_some());

        assert_eq!(store.footnote_count(), 0);
        assert_eq!(store.endnote_count(), 1);
    }

    #[test]
    fn test_footnote_properties_default() {
        let props = FootnoteProperties::default();

        assert_eq!(props.numbering, NumberingScheme::Arabic);
        assert_eq!(props.restart, RestartNumbering::Continuous);
        assert_eq!(props.start_at, 1);
        assert_eq!(props.position, FootnotePosition::PageBottom);
        assert!(props.show_separator);
    }

    #[test]
    fn test_endnote_properties_default() {
        let props = EndnoteProperties::default();

        assert_eq!(props.numbering, NumberingScheme::LowerRoman);
        assert_eq!(props.restart, RestartNumbering::Continuous);
        assert_eq!(props.start_at, 1);
        assert_eq!(props.position, EndnotePosition::EndOfDocument);
    }

    #[test]
    fn test_renumber_footnotes() {
        let mut store = NoteStore::new();

        // Create footnotes
        for i in 0..5 {
            let mut note = Note::footnote();
            let pos = Position::new(NodeId::new(), i * 10);
            note.set_reference_position(pos);
            store.insert_footnote(note);
        }

        store.renumber_footnotes();

        // Check that all footnotes have marks
        for note in store.footnotes() {
            assert!(!note.mark.is_empty());
        }
    }

    #[test]
    fn test_section_properties() {
        let mut store = NoteStore::new();
        let section_id = NodeId::new();

        // Default props
        let default_props = store.get_footnote_props(None);
        assert_eq!(default_props.numbering, NumberingScheme::Arabic);

        // Set section-specific props
        let section_props = FootnoteProperties {
            numbering: NumberingScheme::LowerRoman,
            ..Default::default()
        };
        store.set_footnote_props(Some(section_id), section_props);

        // Check section props
        let retrieved = store.get_footnote_props(Some(section_id));
        assert_eq!(retrieved.numbering, NumberingScheme::LowerRoman);

        // Default should be unchanged
        let default_again = store.get_footnote_props(None);
        assert_eq!(default_again.numbering, NumberingScheme::Arabic);
    }
}
