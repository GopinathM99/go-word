//! Field Module - Dynamic content fields
//!
//! Fields are placeholders for dynamic content that gets calculated at render time.
//! Common fields include page numbers, dates, file names, table of contents, etc.

use crate::{Node, NodeId, NodeType, Run};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Range;

// =============================================================================
// Field Instruction Types
// =============================================================================

/// Number format for sequence and page numbers
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberFormat {
    /// Arabic numerals (1, 2, 3...)
    #[default]
    Arabic,
    /// Lowercase letters (a, b, c...)
    LowercaseLetter,
    /// Uppercase letters (A, B, C...)
    UppercaseLetter,
    /// Lowercase Roman numerals (i, ii, iii...)
    LowercaseRoman,
    /// Uppercase Roman numerals (I, II, III...)
    UppercaseRoman,
    /// Ordinal (1st, 2nd, 3rd...)
    Ordinal,
    /// Cardinal text (one, two, three...)
    CardinalText,
    /// Ordinal text (first, second, third...)
    OrdinalText,
}

impl NumberFormat {
    /// Format a number according to this format
    pub fn format(&self, number: u32) -> String {
        match self {
            NumberFormat::Arabic => number.to_string(),
            NumberFormat::LowercaseLetter => Self::to_letter(number, false),
            NumberFormat::UppercaseLetter => Self::to_letter(number, true),
            NumberFormat::LowercaseRoman => Self::to_roman(number, false),
            NumberFormat::UppercaseRoman => Self::to_roman(number, true),
            NumberFormat::Ordinal => Self::to_ordinal(number),
            NumberFormat::CardinalText => Self::to_cardinal_text(number),
            NumberFormat::OrdinalText => Self::to_ordinal_text(number),
        }
    }

    fn to_letter(number: u32, uppercase: bool) -> String {
        if number == 0 {
            return String::new();
        }

        let mut result = String::new();
        let mut n = number;

        while n > 0 {
            n -= 1;
            let letter = ((n % 26) as u8 + if uppercase { b'A' } else { b'a' }) as char;
            result.insert(0, letter);
            n /= 26;
        }

        result
    }

    fn to_roman(number: u32, uppercase: bool) -> String {
        if number == 0 || number > 3999 {
            return number.to_string();
        }

        let numerals = if uppercase {
            [
                ("M", 1000), ("CM", 900), ("D", 500), ("CD", 400),
                ("C", 100), ("XC", 90), ("L", 50), ("XL", 40),
                ("X", 10), ("IX", 9), ("V", 5), ("IV", 4), ("I", 1),
            ]
        } else {
            [
                ("m", 1000), ("cm", 900), ("d", 500), ("cd", 400),
                ("c", 100), ("xc", 90), ("l", 50), ("xl", 40),
                ("x", 10), ("ix", 9), ("v", 5), ("iv", 4), ("i", 1),
            ]
        };

        let mut result = String::new();
        let mut n = number;

        for (numeral, value) in &numerals {
            while n >= *value {
                result.push_str(numeral);
                n -= value;
            }
        }

        result
    }

    fn to_ordinal(number: u32) -> String {
        let suffix = match number % 100 {
            11 | 12 | 13 => "th",
            _ => match number % 10 {
                1 => "st",
                2 => "nd",
                3 => "rd",
                _ => "th",
            },
        };
        format!("{}{}", number, suffix)
    }

    fn to_cardinal_text(number: u32) -> String {
        const ONES: [&str; 20] = [
            "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
            "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen",
            "eighteen", "nineteen",
        ];
        const TENS: [&str; 10] = [
            "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
        ];

        if number < 20 {
            return ONES[number as usize].to_string();
        }
        if number < 100 {
            let tens = TENS[(number / 10) as usize];
            let ones = number % 10;
            if ones == 0 {
                return tens.to_string();
            }
            return format!("{}-{}", tens, ONES[ones as usize]);
        }
        // For larger numbers, just use Arabic
        number.to_string()
    }

    fn to_ordinal_text(number: u32) -> String {
        const ORDINALS: [&str; 20] = [
            "zeroth", "first", "second", "third", "fourth", "fifth", "sixth", "seventh",
            "eighth", "ninth", "tenth", "eleventh", "twelfth", "thirteenth", "fourteenth",
            "fifteenth", "sixteenth", "seventeenth", "eighteenth", "nineteenth",
        ];

        if number < 20 {
            return ORDINALS[number as usize].to_string();
        }
        // For larger numbers, just use ordinal
        Self::to_ordinal(number)
    }
}

// =============================================================================
// TOC (Table of Contents) Switches
// =============================================================================

/// Switches/options for Table of Contents field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TocSwitches {
    /// Range of heading levels to include (e.g., 1..4 means Heading 1-3)
    pub heading_levels: Range<u8>,
    /// Include page numbers
    pub include_page_numbers: bool,
    /// Right-align page numbers
    pub right_align_page_numbers: bool,
    /// Create hyperlinks to headings
    pub hyperlinks: bool,
    /// Tab leader style between title and page number
    pub tab_leader: TocTabLeader,
    /// Include entries marked with TC fields
    pub include_tc_fields: bool,
    /// Custom styles to include (style name -> TOC level)
    pub custom_styles: HashMap<String, u8>,
}

impl Default for TocSwitches {
    fn default() -> Self {
        Self {
            heading_levels: 1..4, // Heading 1-3 by default
            include_page_numbers: true,
            right_align_page_numbers: true,
            hyperlinks: true,
            tab_leader: TocTabLeader::Dots,
            include_tc_fields: false,
            custom_styles: HashMap::new(),
        }
    }
}

/// Tab leader style for TOC entries
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TocTabLeader {
    /// No leader
    None,
    /// Dots (...)
    #[default]
    Dots,
    /// Dashes (---)
    Dashes,
    /// Underline (___)
    Underline,
}

// =============================================================================
// SEQ (Sequence) Options
// =============================================================================

/// Options for sequence numbering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeqOptions {
    /// Sequence identifier (e.g., "Figure", "Table", "Equation")
    pub identifier: String,
    /// Number format
    pub format: NumberFormat,
    /// Reset sequence at specific level (None = no reset)
    pub reset_at_heading_level: Option<u8>,
    /// Hidden - don't advance counter, just show current value
    pub current_only: bool,
    /// Reset to specific value
    pub reset_to: Option<u32>,
    /// Repeat previous value
    pub repeat_previous: bool,
}

impl Default for SeqOptions {
    fn default() -> Self {
        Self {
            identifier: String::new(),
            format: NumberFormat::Arabic,
            reset_at_heading_level: None,
            current_only: false,
            reset_to: None,
            repeat_previous: false,
        }
    }
}

// =============================================================================
// REF (Reference) Options
// =============================================================================

/// What to display for a REF field
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefDisplayType {
    /// Display the bookmark's content
    #[default]
    Content,
    /// Display the page number where the bookmark is
    PageNumber,
    /// Display the paragraph number (for numbered paragraphs)
    ParagraphNumber,
    /// Display the paragraph number with context
    ParagraphNumberFullContext,
    /// Display "above" or "below" relative to current position
    RelativePosition,
}

/// Options for REF field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefOptions {
    /// Name of the bookmark to reference
    pub bookmark: String,
    /// What to display
    pub display: RefDisplayType,
    /// Create hyperlink to the bookmark
    pub hyperlink: bool,
    /// Include position relative text ("above"/"below")
    pub include_position: bool,
}

impl Default for RefOptions {
    fn default() -> Self {
        Self {
            bookmark: String::new(),
            display: RefDisplayType::Content,
            hyperlink: true,
            include_position: false,
        }
    }
}

// =============================================================================
// Field Instruction Enum
// =============================================================================

/// Types of field instructions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldInstruction {
    /// Current page number (PAGE)
    Page {
        format: NumberFormat,
    },
    /// Total number of pages (NUMPAGES)
    NumPages {
        format: NumberFormat,
    },
    /// Current date (DATE)
    Date {
        format: String,
    },
    /// Current time (TIME)
    Time {
        format: String,
    },
    /// Table of Contents (TOC)
    Toc {
        switches: TocSwitches,
    },
    /// Cross-reference to bookmark (REF)
    Ref {
        options: RefOptions,
    },
    /// Sequence numbering (SEQ)
    Seq {
        options: SeqOptions,
    },
    /// Document author (AUTHOR)
    Author,
    /// Document title (TITLE)
    Title,
    /// Document subject (SUBJECT)
    Subject,
    /// Document file name (FILENAME)
    FileName {
        include_path: bool,
    },
    /// Section number (SECTION)
    Section,
    /// Section page count (SECTIONPAGES)
    SectionPages,
    /// Hyperlink (HYPERLINK)
    Hyperlink {
        url: String,
        display_text: Option<String>,
    },
    /// Include text from another file (INCLUDETEXT)
    IncludeText {
        file_path: String,
    },
    /// IF conditional field
    If {
        condition: String,
        true_text: String,
        false_text: String,
    },
    /// Print date (PRINTDATE)
    PrintDate {
        format: String,
    },
    /// Save date (SAVEDATE)
    SaveDate {
        format: String,
    },
    /// Create date (CREATEDATE)
    CreateDate {
        format: String,
    },
    /// Edit time in minutes (EDITTIME)
    EditTime,
    /// Number of words (NUMWORDS)
    NumWords,
    /// Number of characters (NUMCHARS)
    NumChars,
    /// Custom field with arbitrary code
    Custom {
        code: String,
    },
}

impl FieldInstruction {
    /// Get the field code name (e.g., "PAGE", "TOC", "REF")
    pub fn code_name(&self) -> &str {
        match self {
            FieldInstruction::Page { .. } => "PAGE",
            FieldInstruction::NumPages { .. } => "NUMPAGES",
            FieldInstruction::Date { .. } => "DATE",
            FieldInstruction::Time { .. } => "TIME",
            FieldInstruction::Toc { .. } => "TOC",
            FieldInstruction::Ref { .. } => "REF",
            FieldInstruction::Seq { .. } => "SEQ",
            FieldInstruction::Author => "AUTHOR",
            FieldInstruction::Title => "TITLE",
            FieldInstruction::Subject => "SUBJECT",
            FieldInstruction::FileName { .. } => "FILENAME",
            FieldInstruction::Section => "SECTION",
            FieldInstruction::SectionPages => "SECTIONPAGES",
            FieldInstruction::Hyperlink { .. } => "HYPERLINK",
            FieldInstruction::IncludeText { .. } => "INCLUDETEXT",
            FieldInstruction::If { .. } => "IF",
            FieldInstruction::PrintDate { .. } => "PRINTDATE",
            FieldInstruction::SaveDate { .. } => "SAVEDATE",
            FieldInstruction::CreateDate { .. } => "CREATEDATE",
            FieldInstruction::EditTime => "EDITTIME",
            FieldInstruction::NumWords => "NUMWORDS",
            FieldInstruction::NumChars => "NUMCHARS",
            FieldInstruction::Custom { .. } => "CUSTOM",
        }
    }

    /// Get a display string for the field instruction
    pub fn display_string(&self) -> String {
        match self {
            FieldInstruction::Page { .. } => "PAGE".to_string(),
            FieldInstruction::NumPages { .. } => "NUMPAGES".to_string(),
            FieldInstruction::Date { format } => format!("DATE \\@ \"{}\"", format),
            FieldInstruction::Time { format } => format!("TIME \\@ \"{}\"", format),
            FieldInstruction::Toc { switches } => {
                let mut s = "TOC".to_string();
                if switches.heading_levels.start > 1 || switches.heading_levels.end != 4 {
                    s.push_str(&format!(" \\o \"{}-{}\"",
                        switches.heading_levels.start,
                        switches.heading_levels.end - 1
                    ));
                }
                if switches.hyperlinks {
                    s.push_str(" \\h");
                }
                s
            }
            FieldInstruction::Ref { options } => {
                let mut s = format!("REF {}", options.bookmark);
                if options.hyperlink {
                    s.push_str(" \\h");
                }
                match options.display {
                    RefDisplayType::PageNumber => s.push_str(" \\p"),
                    RefDisplayType::ParagraphNumber => s.push_str(" \\n"),
                    RefDisplayType::ParagraphNumberFullContext => s.push_str(" \\w"),
                    RefDisplayType::RelativePosition => s.push_str(" \\r"),
                    RefDisplayType::Content => {}
                }
                s
            }
            FieldInstruction::Seq { options } => {
                let mut s = format!("SEQ {}", options.identifier);
                if options.format != NumberFormat::Arabic {
                    s.push_str(&format!(" \\* {:?}", options.format));
                }
                s
            }
            FieldInstruction::Author => "AUTHOR".to_string(),
            FieldInstruction::Title => "TITLE".to_string(),
            FieldInstruction::Subject => "SUBJECT".to_string(),
            FieldInstruction::FileName { include_path } => {
                if *include_path {
                    "FILENAME \\p".to_string()
                } else {
                    "FILENAME".to_string()
                }
            }
            FieldInstruction::Section => "SECTION".to_string(),
            FieldInstruction::SectionPages => "SECTIONPAGES".to_string(),
            FieldInstruction::Hyperlink { url, .. } => format!("HYPERLINK \"{}\"", url),
            FieldInstruction::IncludeText { file_path } => format!("INCLUDETEXT \"{}\"", file_path),
            FieldInstruction::If { condition, .. } => format!("IF {}", condition),
            FieldInstruction::PrintDate { format } => format!("PRINTDATE \\@ \"{}\"", format),
            FieldInstruction::SaveDate { format } => format!("SAVEDATE \\@ \"{}\"", format),
            FieldInstruction::CreateDate { format } => format!("CREATEDATE \\@ \"{}\"", format),
            FieldInstruction::EditTime => "EDITTIME".to_string(),
            FieldInstruction::NumWords => "NUMWORDS".to_string(),
            FieldInstruction::NumChars => "NUMCHARS".to_string(),
            FieldInstruction::Custom { code } => code.clone(),
        }
    }

    /// Check if this field type auto-updates during layout (e.g., PAGE)
    pub fn auto_updates_on_layout(&self) -> bool {
        matches!(
            self,
            FieldInstruction::Page { .. }
                | FieldInstruction::NumPages { .. }
                | FieldInstruction::Section
                | FieldInstruction::SectionPages
        )
    }

    /// Check if this field needs the full document context to evaluate
    pub fn needs_document_context(&self) -> bool {
        matches!(
            self,
            FieldInstruction::Toc { .. }
                | FieldInstruction::Ref { .. }
                | FieldInstruction::Seq { .. }
                | FieldInstruction::NumWords
                | FieldInstruction::NumChars
        )
    }
}

// =============================================================================
// Field Node
// =============================================================================

/// A field node representing dynamic content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Unique identifier
    id: NodeId,
    /// Parent node ID
    parent: Option<NodeId>,
    /// The field instruction
    pub instruction: FieldInstruction,
    /// Cached result as runs (with formatting preserved)
    pub result: Vec<Run>,
    /// Cached result as plain text
    pub cached_text: Option<String>,
    /// Whether the field is locked (won't auto-update)
    pub locked: bool,
    /// Whether to show field code instead of result
    pub show_code: bool,
    /// Whether the field result is dirty and needs updating
    pub dirty: bool,
}

impl Field {
    /// Create a new field with the given instruction
    pub fn new(instruction: FieldInstruction) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            instruction,
            result: Vec::new(),
            cached_text: None,
            locked: false,
            show_code: false,
            dirty: true,
        }
    }

    /// Create a page number field
    pub fn page() -> Self {
        Self::new(FieldInstruction::Page {
            format: NumberFormat::Arabic,
        })
    }

    /// Create a page number field with format
    pub fn page_with_format(format: NumberFormat) -> Self {
        Self::new(FieldInstruction::Page { format })
    }

    /// Create a total pages field
    pub fn num_pages() -> Self {
        Self::new(FieldInstruction::NumPages {
            format: NumberFormat::Arabic,
        })
    }

    /// Create a date field
    pub fn date(format: impl Into<String>) -> Self {
        Self::new(FieldInstruction::Date {
            format: format.into(),
        })
    }

    /// Create a time field
    pub fn time(format: impl Into<String>) -> Self {
        Self::new(FieldInstruction::Time {
            format: format.into(),
        })
    }

    /// Create a TOC field with default switches
    pub fn toc() -> Self {
        Self::new(FieldInstruction::Toc {
            switches: TocSwitches::default(),
        })
    }

    /// Create a TOC field with custom switches
    pub fn toc_with_switches(switches: TocSwitches) -> Self {
        Self::new(FieldInstruction::Toc { switches })
    }

    /// Create a REF field
    pub fn reference(bookmark: impl Into<String>) -> Self {
        Self::new(FieldInstruction::Ref {
            options: RefOptions {
                bookmark: bookmark.into(),
                ..Default::default()
            },
        })
    }

    /// Create a REF field that displays page number
    pub fn reference_page(bookmark: impl Into<String>) -> Self {
        Self::new(FieldInstruction::Ref {
            options: RefOptions {
                bookmark: bookmark.into(),
                display: RefDisplayType::PageNumber,
                ..Default::default()
            },
        })
    }

    /// Create a SEQ field
    pub fn seq(identifier: impl Into<String>) -> Self {
        Self::new(FieldInstruction::Seq {
            options: SeqOptions {
                identifier: identifier.into(),
                ..Default::default()
            },
        })
    }

    /// Create an AUTHOR field
    pub fn author() -> Self {
        Self::new(FieldInstruction::Author)
    }

    /// Create a TITLE field
    pub fn title() -> Self {
        Self::new(FieldInstruction::Title)
    }

    /// Create a FILENAME field
    pub fn filename(include_path: bool) -> Self {
        Self::new(FieldInstruction::FileName { include_path })
    }

    /// Lock the field to prevent auto-updates
    pub fn lock(&mut self) {
        self.locked = true;
    }

    /// Unlock the field for auto-updates
    pub fn unlock(&mut self) {
        self.locked = false;
    }

    /// Toggle showing field code vs result
    pub fn toggle_code(&mut self) {
        self.show_code = !self.show_code;
    }

    /// Mark the field as dirty (needs update)
    pub fn mark_dirty(&mut self) {
        if !self.locked {
            self.dirty = true;
        }
    }

    /// Update the field result
    pub fn set_result(&mut self, text: String) {
        self.cached_text = Some(text.clone());
        self.result = vec![Run::new(text)];
        self.dirty = false;
    }

    /// Update the field result with formatted runs
    pub fn set_result_runs(&mut self, runs: Vec<Run>) {
        self.cached_text = Some(
            runs.iter()
                .map(|r| r.text.as_str())
                .collect::<Vec<_>>()
                .join(""),
        );
        self.result = runs;
        self.dirty = false;
    }

    /// Get the display text (result or field code)
    pub fn display_text(&self) -> String {
        if self.show_code {
            format!("{{ {} }}", self.instruction.display_string())
        } else if let Some(ref text) = self.cached_text {
            text.clone()
        } else {
            format!("{{ {} }}", self.instruction.code_name())
        }
    }

    /// Check if this field auto-updates on layout
    pub fn auto_updates_on_layout(&self) -> bool {
        !self.locked && self.instruction.auto_updates_on_layout()
    }
}

impl Default for Field {
    fn default() -> Self {
        Self::page()
    }
}

impl Node for Field {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Field
    }

    fn children(&self) -> &[NodeId] {
        &[] // Fields have no children
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

    fn text_content(&self) -> Option<&str> {
        self.cached_text.as_deref()
    }
}

// =============================================================================
// Field Registry
// =============================================================================

/// Registry for tracking all fields in a document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldRegistry {
    /// All fields indexed by ID
    fields: HashMap<NodeId, Field>,
    /// Sequence counters (identifier -> current value)
    sequence_counters: HashMap<String, u32>,
    /// Fields marked as dirty
    dirty_fields: Vec<NodeId>,
}

impl FieldRegistry {
    /// Create a new field registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a field into the registry
    pub fn insert(&mut self, field: Field) -> NodeId {
        let id = field.id;
        if field.dirty {
            self.dirty_fields.push(id);
        }
        self.fields.insert(id, field);
        id
    }

    /// Remove a field from the registry
    pub fn remove(&mut self, id: NodeId) -> Option<Field> {
        self.dirty_fields.retain(|&fid| fid != id);
        self.fields.remove(&id)
    }

    /// Get a field by ID
    pub fn get(&self, id: NodeId) -> Option<&Field> {
        self.fields.get(&id)
    }

    /// Get a mutable field by ID
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Field> {
        self.fields.get_mut(&id)
    }

    /// Get all fields
    pub fn all(&self) -> impl Iterator<Item = &Field> {
        self.fields.values()
    }

    /// Get all field IDs
    pub fn all_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.fields.keys().copied()
    }

    /// Get dirty field IDs
    pub fn dirty_fields(&self) -> &[NodeId] {
        &self.dirty_fields
    }

    /// Clear dirty fields list
    pub fn clear_dirty(&mut self) {
        self.dirty_fields.clear();
    }

    /// Mark a field as dirty
    pub fn mark_dirty(&mut self, id: NodeId) {
        if let Some(field) = self.fields.get_mut(&id) {
            field.mark_dirty();
            if !self.dirty_fields.contains(&id) {
                self.dirty_fields.push(id);
            }
        }
    }

    /// Mark all fields as dirty
    pub fn mark_all_dirty(&mut self) {
        self.dirty_fields.clear();
        for (id, field) in &mut self.fields {
            field.mark_dirty();
            self.dirty_fields.push(*id);
        }
    }

    /// Get the next sequence number for an identifier
    pub fn next_seq(&mut self, identifier: &str) -> u32 {
        let counter = self.sequence_counters.entry(identifier.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Get the current sequence number for an identifier (without incrementing)
    pub fn current_seq(&self, identifier: &str) -> u32 {
        self.sequence_counters.get(identifier).copied().unwrap_or(0)
    }

    /// Reset a sequence counter
    pub fn reset_seq(&mut self, identifier: &str, value: u32) {
        self.sequence_counters.insert(identifier.to_string(), value);
    }

    /// Reset all sequence counters
    pub fn reset_all_seq(&mut self) {
        self.sequence_counters.clear();
    }

    /// Get fields by type
    pub fn fields_of_type(&self, code_name: &str) -> Vec<&Field> {
        self.fields
            .values()
            .filter(|f| f.instruction.code_name() == code_name)
            .collect()
    }

    /// Get all TOC fields
    pub fn toc_fields(&self) -> Vec<&Field> {
        self.fields_of_type("TOC")
    }

    /// Get all REF fields
    pub fn ref_fields(&self) -> Vec<&Field> {
        self.fields_of_type("REF")
    }

    /// Get all SEQ fields
    pub fn seq_fields(&self) -> Vec<&Field> {
        self.fields_of_type("SEQ")
    }

    /// Number of fields
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

// =============================================================================
// TOC Entry (generated during TOC field evaluation)
// =============================================================================

/// A single entry in a table of contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    /// The heading text
    pub text: String,
    /// The heading level (1-9)
    pub level: u8,
    /// The page number
    pub page_number: u32,
    /// Bookmark name for linking
    pub bookmark: Option<String>,
    /// Node ID of the heading paragraph
    pub paragraph_id: NodeId,
}

// =============================================================================
// Field Context (for evaluation)
// =============================================================================

/// Context information needed to evaluate fields
#[derive(Debug, Clone, Default)]
pub struct FieldContext {
    /// Current page number (1-indexed)
    pub current_page: u32,
    /// Total number of pages
    pub total_pages: u32,
    /// Current section number (1-indexed)
    pub current_section: u32,
    /// Pages in current section
    pub section_pages: u32,
    /// Document file name
    pub file_name: Option<String>,
    /// Document file path
    pub file_path: Option<String>,
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Document subject
    pub subject: Option<String>,
    /// Current date/time
    pub now: Option<chrono::DateTime<chrono::Local>>,
    /// Create date
    pub create_date: Option<chrono::DateTime<chrono::Local>>,
    /// Last save date
    pub save_date: Option<chrono::DateTime<chrono::Local>>,
    /// Last print date
    pub print_date: Option<chrono::DateTime<chrono::Local>>,
    /// Edit time in minutes
    pub edit_time_minutes: u32,
    /// Word count
    pub word_count: u32,
    /// Character count
    pub char_count: u32,
    /// TOC entries (for TOC fields)
    pub toc_entries: Vec<TocEntry>,
    /// Bookmark page numbers (bookmark name -> page number)
    pub bookmark_pages: HashMap<String, u32>,
    /// Bookmark content (bookmark name -> text content)
    pub bookmark_content: HashMap<String, String>,
}

impl FieldContext {
    /// Create a new field context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set page information
    pub fn with_page_info(mut self, current: u32, total: u32) -> Self {
        self.current_page = current;
        self.total_pages = total;
        self
    }

    /// Set section information
    pub fn with_section_info(mut self, current: u32, pages: u32) -> Self {
        self.current_section = current;
        self.section_pages = pages;
        self
    }

    /// Set document metadata
    pub fn with_metadata(
        mut self,
        file_name: Option<String>,
        title: Option<String>,
        author: Option<String>,
    ) -> Self {
        self.file_name = file_name;
        self.title = title;
        self.author = author;
        self
    }

    /// Set current time
    pub fn with_now(mut self) -> Self {
        self.now = Some(chrono::Local::now());
        self
    }
}

// =============================================================================
// Field Evaluator
// =============================================================================

/// Evaluates fields based on context
pub struct FieldEvaluator;

impl FieldEvaluator {
    /// Evaluate a field and return its string value
    pub fn evaluate(field: &Field, context: &FieldContext) -> String {
        Self::evaluate_instruction(&field.instruction, context)
    }

    /// Evaluate a field instruction
    pub fn evaluate_instruction(instruction: &FieldInstruction, context: &FieldContext) -> String {
        match instruction {
            FieldInstruction::Page { format } => {
                format.format(context.current_page)
            }
            FieldInstruction::NumPages { format } => {
                format.format(context.total_pages)
            }
            FieldInstruction::Date { format } => {
                if let Some(now) = &context.now {
                    Self::format_datetime(now, format)
                } else {
                    "DATE".to_string()
                }
            }
            FieldInstruction::Time { format } => {
                if let Some(now) = &context.now {
                    Self::format_datetime(now, format)
                } else {
                    "TIME".to_string()
                }
            }
            FieldInstruction::Author => {
                context.author.clone().unwrap_or_default()
            }
            FieldInstruction::Title => {
                context.title.clone().unwrap_or_default()
            }
            FieldInstruction::Subject => {
                context.subject.clone().unwrap_or_default()
            }
            FieldInstruction::FileName { include_path } => {
                if *include_path {
                    context.file_path.clone().unwrap_or_default()
                } else {
                    context.file_name.clone().unwrap_or_default()
                }
            }
            FieldInstruction::Section => {
                context.current_section.to_string()
            }
            FieldInstruction::SectionPages => {
                context.section_pages.to_string()
            }
            FieldInstruction::Ref { options } => {
                Self::evaluate_ref(options, context)
            }
            FieldInstruction::Toc { switches } => {
                Self::evaluate_toc(switches, context)
            }
            FieldInstruction::Seq { options } => {
                // Note: SEQ evaluation requires mutable access to the registry
                // This returns a placeholder; actual evaluation happens in the update engine
                format!("[SEQ:{}]", options.identifier)
            }
            FieldInstruction::Hyperlink { url, display_text } => {
                display_text.as_ref().unwrap_or(url).clone()
            }
            FieldInstruction::If { true_text, false_text, .. } => {
                // Simplified: always return true_text
                // Real implementation would parse and evaluate the condition
                true_text.clone()
            }
            FieldInstruction::PrintDate { format } => {
                if let Some(dt) = &context.print_date {
                    Self::format_datetime(dt, format)
                } else {
                    String::new()
                }
            }
            FieldInstruction::SaveDate { format } => {
                if let Some(dt) = &context.save_date {
                    Self::format_datetime(dt, format)
                } else {
                    String::new()
                }
            }
            FieldInstruction::CreateDate { format } => {
                if let Some(dt) = &context.create_date {
                    Self::format_datetime(dt, format)
                } else {
                    String::new()
                }
            }
            FieldInstruction::EditTime => {
                context.edit_time_minutes.to_string()
            }
            FieldInstruction::NumWords => {
                context.word_count.to_string()
            }
            FieldInstruction::NumChars => {
                context.char_count.to_string()
            }
            FieldInstruction::IncludeText { .. } => {
                // Would need file system access
                "[INCLUDETEXT]".to_string()
            }
            FieldInstruction::Custom { code } => {
                format!("{{ {} }}", code)
            }
        }
    }

    fn evaluate_ref(options: &RefOptions, context: &FieldContext) -> String {
        match options.display {
            RefDisplayType::Content => {
                context
                    .bookmark_content
                    .get(&options.bookmark)
                    .cloned()
                    .unwrap_or_else(|| format!("[REF:{}]", options.bookmark))
            }
            RefDisplayType::PageNumber => {
                context
                    .bookmark_pages
                    .get(&options.bookmark)
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "?".to_string())
            }
            RefDisplayType::ParagraphNumber => {
                // Would need paragraph numbering info
                "[#]".to_string()
            }
            RefDisplayType::ParagraphNumberFullContext => {
                "[#.#]".to_string()
            }
            RefDisplayType::RelativePosition => {
                // Would need to compare positions
                "above".to_string()
            }
        }
    }

    fn evaluate_toc(switches: &TocSwitches, context: &FieldContext) -> String {
        let mut lines = Vec::new();

        for entry in &context.toc_entries {
            if entry.level >= switches.heading_levels.start
                && entry.level < switches.heading_levels.end
            {
                let indent = "  ".repeat((entry.level - 1) as usize);
                let page_str = if switches.include_page_numbers {
                    let leader = match switches.tab_leader {
                        TocTabLeader::None => " ",
                        TocTabLeader::Dots => "...",
                        TocTabLeader::Dashes => "---",
                        TocTabLeader::Underline => "___",
                    };
                    format!("{}{}", leader, entry.page_number)
                } else {
                    String::new()
                };
                lines.push(format!("{}{}{}", indent, entry.text, page_str));
            }
        }

        lines.join("\n")
    }

    fn format_datetime(dt: &chrono::DateTime<chrono::Local>, format: &str) -> String {
        // Support common format codes
        let format = format
            .replace("MMMM", "%B")    // Full month name
            .replace("MMM", "%b")     // Abbreviated month name
            .replace("MM", "%m")      // Month number with leading zero
            .replace("M", "%-m")      // Month number without leading zero
            .replace("dddd", "%A")    // Full day name
            .replace("ddd", "%a")     // Abbreviated day name
            .replace("dd", "%d")      // Day with leading zero
            .replace("d", "%-d")      // Day without leading zero
            .replace("yyyy", "%Y")    // 4-digit year
            .replace("yy", "%y")      // 2-digit year
            .replace("HH", "%H")      // 24-hour with leading zero
            .replace("H", "%-H")      // 24-hour without leading zero
            .replace("hh", "%I")      // 12-hour with leading zero
            .replace("h", "%-I")      // 12-hour without leading zero
            .replace("mm", "%M")      // Minutes with leading zero
            .replace("ss", "%S")      // Seconds with leading zero
            .replace("AM/PM", "%p")   // AM/PM
            .replace("am/pm", "%P");  // am/pm

        dt.format(&format).to_string()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_format_arabic() {
        let fmt = NumberFormat::Arabic;
        assert_eq!(fmt.format(1), "1");
        assert_eq!(fmt.format(42), "42");
        assert_eq!(fmt.format(100), "100");
    }

    #[test]
    fn test_number_format_roman() {
        let fmt = NumberFormat::UppercaseRoman;
        assert_eq!(fmt.format(1), "I");
        assert_eq!(fmt.format(4), "IV");
        assert_eq!(fmt.format(9), "IX");
        assert_eq!(fmt.format(10), "X");
        assert_eq!(fmt.format(50), "L");
        assert_eq!(fmt.format(100), "C");
        assert_eq!(fmt.format(2024), "MMXXIV");
    }

    #[test]
    fn test_number_format_letter() {
        let fmt = NumberFormat::LowercaseLetter;
        assert_eq!(fmt.format(1), "a");
        assert_eq!(fmt.format(26), "z");
        assert_eq!(fmt.format(27), "aa");
        assert_eq!(fmt.format(28), "ab");
    }

    #[test]
    fn test_number_format_ordinal() {
        let fmt = NumberFormat::Ordinal;
        assert_eq!(fmt.format(1), "1st");
        assert_eq!(fmt.format(2), "2nd");
        assert_eq!(fmt.format(3), "3rd");
        assert_eq!(fmt.format(4), "4th");
        assert_eq!(fmt.format(11), "11th");
        assert_eq!(fmt.format(12), "12th");
        assert_eq!(fmt.format(13), "13th");
        assert_eq!(fmt.format(21), "21st");
        assert_eq!(fmt.format(22), "22nd");
    }

    #[test]
    fn test_field_page() {
        let field = Field::page();
        assert!(matches!(field.instruction, FieldInstruction::Page { .. }));
        assert!(field.dirty);
        assert!(!field.locked);
    }

    #[test]
    fn test_field_evaluation_page() {
        let field = Field::page();
        let context = FieldContext::new().with_page_info(5, 10);
        let result = FieldEvaluator::evaluate(&field, &context);
        assert_eq!(result, "5");
    }

    #[test]
    fn test_field_evaluation_numpages() {
        let field = Field::num_pages();
        let context = FieldContext::new().with_page_info(5, 10);
        let result = FieldEvaluator::evaluate(&field, &context);
        assert_eq!(result, "10");
    }

    #[test]
    fn test_field_lock() {
        let mut field = Field::page();
        assert!(!field.locked);
        field.lock();
        assert!(field.locked);
        field.unlock();
        assert!(!field.locked);
    }

    #[test]
    fn test_field_toggle_code() {
        let mut field = Field::page();
        assert!(!field.show_code);
        field.toggle_code();
        assert!(field.show_code);
        assert_eq!(field.display_text(), "{ PAGE }");
    }

    #[test]
    fn test_field_registry() {
        let mut registry = FieldRegistry::new();

        let field1 = Field::page();
        let id1 = field1.id;
        registry.insert(field1);

        let field2 = Field::num_pages();
        let id2 = field2.id;
        registry.insert(field2);

        assert_eq!(registry.len(), 2);
        assert!(registry.get(id1).is_some());
        assert!(registry.get(id2).is_some());
    }

    #[test]
    fn test_sequence_counters() {
        let mut registry = FieldRegistry::new();

        assert_eq!(registry.current_seq("Figure"), 0);
        assert_eq!(registry.next_seq("Figure"), 1);
        assert_eq!(registry.next_seq("Figure"), 2);
        assert_eq!(registry.current_seq("Figure"), 2);

        assert_eq!(registry.next_seq("Table"), 1);
        assert_eq!(registry.current_seq("Table"), 1);

        registry.reset_seq("Figure", 0);
        assert_eq!(registry.current_seq("Figure"), 0);
    }

    #[test]
    fn test_toc_field() {
        let field = Field::toc();
        if let FieldInstruction::Toc { switches } = &field.instruction {
            assert_eq!(switches.heading_levels, 1..4);
            assert!(switches.include_page_numbers);
            assert!(switches.hyperlinks);
        } else {
            panic!("Expected TOC instruction");
        }
    }

    #[test]
    fn test_ref_field() {
        let field = Field::reference("bookmark1");
        if let FieldInstruction::Ref { options } = &field.instruction {
            assert_eq!(options.bookmark, "bookmark1");
            assert!(options.hyperlink);
        } else {
            panic!("Expected REF instruction");
        }
    }

    #[test]
    fn test_seq_field() {
        let field = Field::seq("Figure");
        if let FieldInstruction::Seq { options } = &field.instruction {
            assert_eq!(options.identifier, "Figure");
            assert_eq!(options.format, NumberFormat::Arabic);
        } else {
            panic!("Expected SEQ instruction");
        }
    }

    #[test]
    fn test_toc_evaluation() {
        let switches = TocSwitches::default();
        let mut context = FieldContext::new();
        context.toc_entries = vec![
            TocEntry {
                text: "Chapter 1".to_string(),
                level: 1,
                page_number: 1,
                bookmark: None,
                paragraph_id: NodeId::new(),
            },
            TocEntry {
                text: "Section 1.1".to_string(),
                level: 2,
                page_number: 5,
                bookmark: None,
                paragraph_id: NodeId::new(),
            },
            TocEntry {
                text: "Chapter 2".to_string(),
                level: 1,
                page_number: 10,
                bookmark: None,
                paragraph_id: NodeId::new(),
            },
        ];

        let result = FieldEvaluator::evaluate_toc(&switches, &context);
        assert!(result.contains("Chapter 1"));
        assert!(result.contains("Section 1.1"));
        assert!(result.contains("Chapter 2"));
    }

    #[test]
    fn test_ref_evaluation() {
        let mut context = FieldContext::new();
        context.bookmark_content.insert("intro".to_string(), "Introduction".to_string());
        context.bookmark_pages.insert("intro".to_string(), 5);

        let options_content = RefOptions {
            bookmark: "intro".to_string(),
            display: RefDisplayType::Content,
            ..Default::default()
        };
        assert_eq!(FieldEvaluator::evaluate_ref(&options_content, &context), "Introduction");

        let options_page = RefOptions {
            bookmark: "intro".to_string(),
            display: RefDisplayType::PageNumber,
            ..Default::default()
        };
        assert_eq!(FieldEvaluator::evaluate_ref(&options_page, &context), "5");
    }

    #[test]
    fn test_field_instruction_code_name() {
        assert_eq!(FieldInstruction::Page { format: NumberFormat::Arabic }.code_name(), "PAGE");
        assert_eq!(FieldInstruction::Toc { switches: TocSwitches::default() }.code_name(), "TOC");
        assert_eq!(FieldInstruction::Author.code_name(), "AUTHOR");
    }

    #[test]
    fn test_field_auto_updates() {
        let page_field = Field::page();
        assert!(page_field.auto_updates_on_layout());

        let author_field = Field::author();
        assert!(!author_field.auto_updates_on_layout());

        let mut locked_page = Field::page();
        locked_page.lock();
        assert!(!locked_page.auto_updates_on_layout());
    }
}
