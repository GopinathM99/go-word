//! Content Control Module - Structured Document Tags (SDT)
//!
//! Content controls are placeholders in a document that can contain specific types
//! of content. They are used for form fields, document templates, and structured
//! data entry. In OOXML, these are represented as w:sdt elements.
//!
//! ## Control Types
//!
//! - **RichText**: Multi-paragraph rich formatted content
//! - **PlainText**: Single-line or multiline plain text
//! - **Checkbox**: Boolean toggle with customizable symbols
//! - **DropdownList**: Select from predefined options
//! - **ComboBox**: Select from options or enter custom text
//! - **DatePicker**: Date selection with format options
//! - **Picture**: Image placeholder
//! - **RepeatingSection**: Repeatable content blocks for lists/tables

use crate::{Node, NodeId, NodeType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Content Control Types
// =============================================================================

/// Types of content controls from the OOXML specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentControlType {
    /// Multi-paragraph rich content with formatting
    RichText,
    /// Single-line or multiline plain text
    PlainText,
    /// Boolean toggle checkbox
    Checkbox,
    /// Select from predefined options only
    DropdownList,
    /// Select from options or enter custom value
    ComboBox,
    /// Date selection with calendar
    DatePicker,
    /// Image placeholder
    Picture,
    /// Repeatable content block
    RepeatingSection,
    /// Item within a repeating section
    RepeatingSectionItem,
    /// Building block gallery
    BuildingBlockGallery,
    /// Citation control
    Citation,
    /// Bibliography control
    Bibliography,
    /// Equation control
    Equation,
    /// Group of other content controls
    Group,
}

impl Default for ContentControlType {
    fn default() -> Self {
        ContentControlType::RichText
    }
}

impl ContentControlType {
    /// Get the OOXML element name for this control type
    pub fn ooxml_element(&self) -> &'static str {
        match self {
            ContentControlType::RichText => "w:richText",
            ContentControlType::PlainText => "w:text",
            ContentControlType::Checkbox => "w14:checkbox",
            ContentControlType::DropdownList => "w:dropDownList",
            ContentControlType::ComboBox => "w:comboBox",
            ContentControlType::DatePicker => "w:date",
            ContentControlType::Picture => "w:picture",
            ContentControlType::RepeatingSection => "w15:repeatingSection",
            ContentControlType::RepeatingSectionItem => "w15:repeatingSectionItem",
            ContentControlType::BuildingBlockGallery => "w:docPartList",
            ContentControlType::Citation => "w:citation",
            ContentControlType::Bibliography => "w:bibliography",
            ContentControlType::Equation => "w:equation",
            ContentControlType::Group => "w:group",
        }
    }

    /// Check if this control type can contain other content controls
    pub fn can_contain_controls(&self) -> bool {
        matches!(
            self,
            ContentControlType::RichText
                | ContentControlType::RepeatingSection
                | ContentControlType::RepeatingSectionItem
                | ContentControlType::Group
        )
    }

    /// Check if this control type supports data binding
    pub fn supports_data_binding(&self) -> bool {
        !matches!(
            self,
            ContentControlType::BuildingBlockGallery
                | ContentControlType::Citation
                | ContentControlType::Bibliography
        )
    }
}

// =============================================================================
// Data Binding
// =============================================================================

/// Data binding configuration for connecting a content control to XML data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataBinding {
    /// XPath expression to locate the data in the custom XML part
    pub xpath: String,
    /// XML namespace prefix mappings (prefix -> URI)
    pub prefix_mappings: HashMap<String, String>,
    /// ID of the custom XML data store
    pub store_id: String,
}

impl DataBinding {
    /// Create a new data binding with an XPath expression
    pub fn new(xpath: impl Into<String>) -> Self {
        Self {
            xpath: xpath.into(),
            prefix_mappings: HashMap::new(),
            store_id: String::new(),
        }
    }

    /// Create a data binding with a store ID
    pub fn with_store(xpath: impl Into<String>, store_id: impl Into<String>) -> Self {
        Self {
            xpath: xpath.into(),
            prefix_mappings: HashMap::new(),
            store_id: store_id.into(),
        }
    }

    /// Add a namespace prefix mapping
    pub fn add_prefix(&mut self, prefix: impl Into<String>, uri: impl Into<String>) {
        self.prefix_mappings.insert(prefix.into(), uri.into());
    }

    /// Get the prefix mappings as an OOXML-formatted string
    pub fn prefix_mappings_string(&self) -> String {
        self.prefix_mappings
            .iter()
            .map(|(prefix, uri)| format!("xmlns:{}='{}'", prefix, uri))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Default for DataBinding {
    fn default() -> Self {
        Self::new("")
    }
}

// =============================================================================
// Validation Rules
// =============================================================================

/// Validation rules for content control input
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Whether the control must have content
    pub required: bool,
    /// Regular expression pattern for validation
    pub regex: Option<String>,
    /// Minimum length of text content
    pub min_length: Option<u32>,
    /// Maximum length of text content
    pub max_length: Option<u32>,
    /// Minimum value (for numeric/date content)
    pub min_value: Option<String>,
    /// Maximum value (for numeric/date content)
    pub max_value: Option<String>,
    /// Custom error message when validation fails
    pub custom_error: Option<String>,
}

impl ValidationRule {
    /// Create an empty validation rule
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a required field validation rule
    pub fn required() -> Self {
        Self {
            required: true,
            ..Default::default()
        }
    }

    /// Create a validation rule with length constraints
    pub fn with_length(min: Option<u32>, max: Option<u32>) -> Self {
        Self {
            min_length: min,
            max_length: max,
            ..Default::default()
        }
    }

    /// Create a validation rule with a regex pattern
    pub fn with_pattern(pattern: impl Into<String>) -> Self {
        Self {
            regex: Some(pattern.into()),
            ..Default::default()
        }
    }

    /// Set the required flag
    pub fn set_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set the regex pattern
    pub fn set_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.regex = Some(pattern.into());
        self
    }

    /// Set length constraints
    pub fn set_length(mut self, min: Option<u32>, max: Option<u32>) -> Self {
        self.min_length = min;
        self.max_length = max;
        self
    }

    /// Set value constraints
    pub fn set_value_range(mut self, min: Option<String>, max: Option<String>) -> Self {
        self.min_value = min;
        self.max_value = max;
        self
    }

    /// Set custom error message
    pub fn set_error(mut self, message: impl Into<String>) -> Self {
        self.custom_error = Some(message.into());
        self
    }

    /// Validate a text value against this rule
    pub fn validate(&self, value: &str) -> ValidationResult {
        let mut errors = Vec::new();

        // Check required
        if self.required && value.is_empty() {
            errors.push(ValidationError::Required);
        }

        // Check length
        if let Some(min) = self.min_length {
            if value.len() < min as usize {
                errors.push(ValidationError::TooShort {
                    min,
                    actual: value.len() as u32,
                });
            }
        }

        if let Some(max) = self.max_length {
            if value.len() > max as usize {
                errors.push(ValidationError::TooLong {
                    max,
                    actual: value.len() as u32,
                });
            }
        }

        // Check regex pattern
        if let Some(ref pattern) = self.regex {
            if let Ok(re) = regex_lite::Regex::new(pattern) {
                if !re.is_match(value) {
                    errors.push(ValidationError::PatternMismatch {
                        pattern: pattern.clone(),
                    });
                }
            }
        }

        if errors.is_empty() {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid(errors)
        }
    }
}

impl Default for ValidationRule {
    fn default() -> Self {
        Self {
            required: false,
            regex: None,
            min_length: None,
            max_length: None,
            min_value: None,
            max_value: None,
            custom_error: None,
        }
    }
}

/// Result of validating content control input
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// Content is valid
    Valid,
    /// Content is invalid with a list of errors
    Invalid(Vec<ValidationError>),
}

impl ValidationResult {
    /// Check if the result is valid
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }

    /// Get validation errors, if any
    pub fn errors(&self) -> &[ValidationError] {
        match self {
            ValidationResult::Valid => &[],
            ValidationResult::Invalid(errors) => errors,
        }
    }
}

/// Types of validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Required field is empty
    Required,
    /// Content is too short
    TooShort { min: u32, actual: u32 },
    /// Content is too long
    TooLong { max: u32, actual: u32 },
    /// Content does not match pattern
    PatternMismatch { pattern: String },
    /// Value is below minimum
    BelowMinimum { min: String, actual: String },
    /// Value is above maximum
    AboveMaximum { max: String, actual: String },
    /// Custom validation error
    Custom(String),
}

// =============================================================================
// Control-Specific Properties
// =============================================================================

/// Type-specific properties for different content control types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlProperties {
    /// Properties for rich text controls
    RichText {
        /// Whether carriage returns are allowed (multiline)
        allow_carriage_return: bool,
    },

    /// Properties for plain text controls
    PlainText {
        /// Whether the control allows multiple lines
        multiline: bool,
        /// Maximum number of characters allowed
        max_chars: Option<u32>,
    },

    /// Properties for checkbox controls
    Checkbox {
        /// Current checked state
        checked: bool,
        /// Symbol to display when checked (default: ballot box with check)
        checked_symbol: CheckboxSymbol,
        /// Symbol to display when unchecked (default: empty ballot box)
        unchecked_symbol: CheckboxSymbol,
    },

    /// Properties for dropdown list controls
    DropdownList {
        /// List of available items
        items: Vec<ListItem>,
        /// Index of the currently selected item
        selected_index: Option<usize>,
    },

    /// Properties for combo box controls
    ComboBox {
        /// List of available items
        items: Vec<ListItem>,
        /// Current text value (may not match any item)
        text: String,
    },

    /// Properties for date picker controls
    DatePicker {
        /// Currently selected date
        date: Option<chrono::NaiveDate>,
        /// Date display format string
        format: String,
        /// Type of calendar to use
        calendar_type: CalendarType,
        /// Storage format for the date
        storage_format: Option<String>,
        /// Language/locale for date display
        locale: Option<String>,
    },

    /// Properties for picture controls
    Picture {
        /// Whether aspect ratio should be preserved when resizing
        aspect_ratio_locked: bool,
    },

    /// Properties for repeating section controls
    RepeatingSection {
        /// Title for the section
        section_title: String,
        /// Minimum number of items
        min_count: Option<u32>,
        /// Maximum number of items
        max_count: Option<u32>,
        /// Whether to allow inserting items
        allow_insert: bool,
        /// Whether to allow deleting items
        allow_delete: bool,
    },

    /// Properties for repeating section item
    RepeatingSectionItem,

    /// Properties for building block gallery
    BuildingBlockGallery {
        /// Gallery category
        category: String,
        /// Building block type
        gallery: String,
    },

    /// Properties for citation controls
    Citation {
        /// Citation sources
        sources: Vec<String>,
    },

    /// Properties for bibliography controls
    Bibliography {
        /// Bibliography style
        style: String,
    },

    /// Properties for equation controls
    Equation,

    /// Properties for group controls
    Group,
}

impl Default for ControlProperties {
    fn default() -> Self {
        ControlProperties::RichText {
            allow_carriage_return: true,
        }
    }
}

impl ControlProperties {
    /// Create default properties for a control type
    pub fn for_type(control_type: ContentControlType) -> Self {
        match control_type {
            ContentControlType::RichText => ControlProperties::RichText {
                allow_carriage_return: true,
            },
            ContentControlType::PlainText => ControlProperties::PlainText {
                multiline: false,
                max_chars: None,
            },
            ContentControlType::Checkbox => ControlProperties::Checkbox {
                checked: false,
                checked_symbol: CheckboxSymbol::default_checked(),
                unchecked_symbol: CheckboxSymbol::default_unchecked(),
            },
            ContentControlType::DropdownList => ControlProperties::DropdownList {
                items: Vec::new(),
                selected_index: None,
            },
            ContentControlType::ComboBox => ControlProperties::ComboBox {
                items: Vec::new(),
                text: String::new(),
            },
            ContentControlType::DatePicker => ControlProperties::DatePicker {
                date: None,
                format: "M/d/yyyy".to_string(),
                calendar_type: CalendarType::Gregorian,
                storage_format: None,
                locale: None,
            },
            ContentControlType::Picture => ControlProperties::Picture {
                aspect_ratio_locked: true,
            },
            ContentControlType::RepeatingSection => ControlProperties::RepeatingSection {
                section_title: String::new(),
                min_count: None,
                max_count: None,
                allow_insert: true,
                allow_delete: true,
            },
            ContentControlType::RepeatingSectionItem => ControlProperties::RepeatingSectionItem,
            ContentControlType::BuildingBlockGallery => ControlProperties::BuildingBlockGallery {
                category: String::new(),
                gallery: String::new(),
            },
            ContentControlType::Citation => ControlProperties::Citation {
                sources: Vec::new(),
            },
            ContentControlType::Bibliography => ControlProperties::Bibliography {
                style: String::new(),
            },
            ContentControlType::Equation => ControlProperties::Equation,
            ContentControlType::Group => ControlProperties::Group,
        }
    }

    /// Get the control type this properties object is for
    pub fn control_type(&self) -> ContentControlType {
        match self {
            ControlProperties::RichText { .. } => ContentControlType::RichText,
            ControlProperties::PlainText { .. } => ContentControlType::PlainText,
            ControlProperties::Checkbox { .. } => ContentControlType::Checkbox,
            ControlProperties::DropdownList { .. } => ContentControlType::DropdownList,
            ControlProperties::ComboBox { .. } => ContentControlType::ComboBox,
            ControlProperties::DatePicker { .. } => ContentControlType::DatePicker,
            ControlProperties::Picture { .. } => ContentControlType::Picture,
            ControlProperties::RepeatingSection { .. } => ContentControlType::RepeatingSection,
            ControlProperties::RepeatingSectionItem => ContentControlType::RepeatingSectionItem,
            ControlProperties::BuildingBlockGallery { .. } => {
                ContentControlType::BuildingBlockGallery
            }
            ControlProperties::Citation { .. } => ContentControlType::Citation,
            ControlProperties::Bibliography { .. } => ContentControlType::Bibliography,
            ControlProperties::Equation => ContentControlType::Equation,
            ControlProperties::Group => ContentControlType::Group,
        }
    }
}

// =============================================================================
// Checkbox Symbol
// =============================================================================

/// Symbol configuration for checkbox controls
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckboxSymbol {
    /// Unicode character to display
    pub character: char,
    /// Font to use for the symbol
    pub font: Option<String>,
}

impl CheckboxSymbol {
    /// Create a new checkbox symbol
    pub fn new(character: char) -> Self {
        Self {
            character,
            font: None,
        }
    }

    /// Create a checkbox symbol with a specific font
    pub fn with_font(character: char, font: impl Into<String>) -> Self {
        Self {
            character,
            font: Some(font.into()),
        }
    }

    /// Default checked symbol (ballot box with check)
    pub fn default_checked() -> Self {
        Self::with_font('\u{2612}', "MS Gothic")
    }

    /// Default unchecked symbol (empty ballot box)
    pub fn default_unchecked() -> Self {
        Self::with_font('\u{2610}', "MS Gothic")
    }
}

// =============================================================================
// List Item
// =============================================================================

/// An item in a dropdown list or combo box
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListItem {
    /// Text displayed to the user
    pub display_text: String,
    /// Underlying value (may differ from display text)
    pub value: String,
}

impl ListItem {
    /// Create a new list item with the same display text and value
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            display_text: text.clone(),
            value: text,
        }
    }

    /// Create a new list item with different display text and value
    pub fn with_value(display_text: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            display_text: display_text.into(),
            value: value.into(),
        }
    }
}

// =============================================================================
// Calendar Type
// =============================================================================

/// Types of calendars supported by date picker controls
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CalendarType {
    /// Gregorian (Western) calendar
    Gregorian,
    /// Hebrew calendar
    Hebrew,
    /// Hijri (Islamic) calendar
    Hijri,
    /// Japanese Emperor Era calendar
    Japan,
    /// Taiwan calendar
    Taiwan,
    /// Korean Tangun Era calendar
    Korea,
    /// Thai Buddhist calendar
    Thai,
    /// Saka Era (Indian) calendar
    Saka,
}

impl Default for CalendarType {
    fn default() -> Self {
        CalendarType::Gregorian
    }
}

impl CalendarType {
    /// Get the OOXML calendar type string
    pub fn ooxml_value(&self) -> &'static str {
        match self {
            CalendarType::Gregorian => "gregorian",
            CalendarType::Hebrew => "hebrew",
            CalendarType::Hijri => "hijri",
            CalendarType::Japan => "japan",
            CalendarType::Taiwan => "taiwan",
            CalendarType::Korea => "korea",
            CalendarType::Thai => "thai",
            CalendarType::Saka => "saka",
        }
    }

    /// Parse from OOXML calendar type string
    pub fn from_ooxml(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "gregorian" => CalendarType::Gregorian,
            "hebrew" => CalendarType::Hebrew,
            "hijri" => CalendarType::Hijri,
            "japan" => CalendarType::Japan,
            "taiwan" => CalendarType::Taiwan,
            "korea" => CalendarType::Korea,
            "thai" => CalendarType::Thai,
            "saka" => CalendarType::Saka,
            _ => CalendarType::Gregorian,
        }
    }
}

// =============================================================================
// Content Control
// =============================================================================

/// A content control (Structured Document Tag) in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentControl {
    /// Unique identifier for this control
    id: NodeId,
    /// Parent node ID
    parent: Option<NodeId>,
    /// Child node IDs (content within the control)
    children: Vec<NodeId>,
    /// Type of content control
    pub control_type: ContentControlType,
    /// Developer-defined identifier tag
    pub tag: String,
    /// User-visible title/label
    pub title: String,
    /// Placeholder text shown when empty
    pub placeholder: String,
    /// Alias for the control (alternative name)
    pub alias: Option<String>,
    /// Whether the control can be deleted
    pub locked: bool,
    /// Whether the contents can be edited
    pub contents_locked: bool,
    /// Data binding configuration
    pub data_binding: Option<DataBinding>,
    /// Validation rules
    pub validation: Option<ValidationRule>,
    /// Type-specific properties
    pub properties: ControlProperties,
    /// Whether to show the control as temporary (disappears after edit)
    pub temporary: bool,
    /// Color for the control (when displayed in edit mode)
    pub color: Option<String>,
    /// Appearance style
    pub appearance: ContentControlAppearance,
    /// OOXML SDT ID (w:id attribute)
    pub sdt_id: Option<i64>,
    /// Unknown/preserved XML elements for round-trip fidelity
    pub unknown_elements: Vec<String>,
}

impl ContentControl {
    /// Create a new content control with the specified type
    pub fn new(control_type: ContentControlType) -> Self {
        Self {
            id: NodeId::new(),
            parent: None,
            children: Vec::new(),
            control_type,
            tag: String::new(),
            title: String::new(),
            placeholder: String::new(),
            alias: None,
            locked: false,
            contents_locked: false,
            data_binding: None,
            validation: None,
            properties: ControlProperties::for_type(control_type),
            temporary: false,
            color: None,
            appearance: ContentControlAppearance::default(),
            sdt_id: None,
            unknown_elements: Vec::new(),
        }
    }

    /// Create a rich text content control
    pub fn rich_text() -> Self {
        Self::new(ContentControlType::RichText)
    }

    /// Create a plain text content control
    pub fn plain_text() -> Self {
        Self::new(ContentControlType::PlainText)
    }

    /// Create a checkbox content control
    pub fn checkbox() -> Self {
        Self::new(ContentControlType::Checkbox)
    }

    /// Create a dropdown list content control
    pub fn dropdown_list() -> Self {
        Self::new(ContentControlType::DropdownList)
    }

    /// Create a combo box content control
    pub fn combo_box() -> Self {
        Self::new(ContentControlType::ComboBox)
    }

    /// Create a date picker content control
    pub fn date_picker() -> Self {
        Self::new(ContentControlType::DatePicker)
    }

    /// Create a picture content control
    pub fn picture() -> Self {
        Self::new(ContentControlType::Picture)
    }

    /// Create a repeating section content control
    pub fn repeating_section() -> Self {
        Self::new(ContentControlType::RepeatingSection)
    }

    /// Set the tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    /// Set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the placeholder text
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the locked state
    pub fn with_locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }

    /// Set the contents locked state
    pub fn with_contents_locked(mut self, contents_locked: bool) -> Self {
        self.contents_locked = contents_locked;
        self
    }

    /// Set data binding
    pub fn with_data_binding(mut self, binding: DataBinding) -> Self {
        self.data_binding = Some(binding);
        self
    }

    /// Set validation rules
    pub fn with_validation(mut self, validation: ValidationRule) -> Self {
        self.validation = Some(validation);
        self
    }

    /// Set control properties
    pub fn with_properties(mut self, properties: ControlProperties) -> Self {
        self.properties = properties;
        self
    }

    /// Add a child node
    pub fn add_child(&mut self, child_id: NodeId) {
        self.children.push(child_id);
    }

    /// Remove a child node
    pub fn remove_child(&mut self, child_id: NodeId) -> bool {
        if let Some(pos) = self.children.iter().position(|&id| id == child_id) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if this control has content
    pub fn has_content(&self) -> bool {
        !self.children.is_empty()
    }

    /// Check if this control is currently showing placeholder
    pub fn is_showing_placeholder(&self) -> bool {
        self.children.is_empty() && !self.placeholder.is_empty()
    }

    /// Get checkbox checked state (if this is a checkbox)
    pub fn is_checked(&self) -> Option<bool> {
        match &self.properties {
            ControlProperties::Checkbox { checked, .. } => Some(*checked),
            _ => None,
        }
    }

    /// Set checkbox checked state (if this is a checkbox)
    pub fn set_checked(&mut self, value: bool) -> bool {
        match &mut self.properties {
            ControlProperties::Checkbox { checked, .. } => {
                *checked = value;
                true
            }
            _ => false,
        }
    }

    /// Toggle checkbox state (if this is a checkbox)
    pub fn toggle_checked(&mut self) -> bool {
        match &mut self.properties {
            ControlProperties::Checkbox { checked, .. } => {
                *checked = !*checked;
                true
            }
            _ => false,
        }
    }

    /// Get selected item index (if this is a dropdown or combo box)
    pub fn selected_index(&self) -> Option<usize> {
        match &self.properties {
            ControlProperties::DropdownList { selected_index, .. } => *selected_index,
            _ => None,
        }
    }

    /// Set selected item index (if this is a dropdown)
    pub fn set_selected_index(&mut self, index: Option<usize>) -> bool {
        match &mut self.properties {
            ControlProperties::DropdownList {
                selected_index,
                items,
            } => {
                if let Some(idx) = index {
                    if idx < items.len() {
                        *selected_index = Some(idx);
                        true
                    } else {
                        false
                    }
                } else {
                    *selected_index = None;
                    true
                }
            }
            _ => false,
        }
    }

    /// Get the selected date (if this is a date picker)
    pub fn selected_date(&self) -> Option<chrono::NaiveDate> {
        match &self.properties {
            ControlProperties::DatePicker { date, .. } => *date,
            _ => None,
        }
    }

    /// Set the selected date (if this is a date picker)
    pub fn set_selected_date(&mut self, value: Option<chrono::NaiveDate>) -> bool {
        match &mut self.properties {
            ControlProperties::DatePicker { date, .. } => {
                *date = value;
                true
            }
            _ => false,
        }
    }

    /// Add an item to a dropdown list or combo box
    pub fn add_list_item(&mut self, item: ListItem) -> bool {
        match &mut self.properties {
            ControlProperties::DropdownList { items, .. }
            | ControlProperties::ComboBox { items, .. } => {
                items.push(item);
                true
            }
            _ => false,
        }
    }

    /// Remove an item from a dropdown list or combo box by index
    pub fn remove_list_item(&mut self, index: usize) -> Option<ListItem> {
        match &mut self.properties {
            ControlProperties::DropdownList {
                items,
                selected_index,
            } => {
                if index < items.len() {
                    // Adjust selected index if needed
                    if let Some(sel) = selected_index {
                        if *sel == index {
                            *selected_index = None;
                        } else if *sel > index {
                            *selected_index = Some(*sel - 1);
                        }
                    }
                    Some(items.remove(index))
                } else {
                    None
                }
            }
            ControlProperties::ComboBox { items, .. } => {
                if index < items.len() {
                    Some(items.remove(index))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get list items (if this is a dropdown or combo box)
    pub fn list_items(&self) -> Option<&[ListItem]> {
        match &self.properties {
            ControlProperties::DropdownList { items, .. }
            | ControlProperties::ComboBox { items, .. } => Some(items),
            _ => None,
        }
    }
}

impl Node for ContentControl {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::ContentControl
    }

    fn children(&self) -> &[NodeId] {
        &self.children
    }

    fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    fn set_parent(&mut self, parent: Option<NodeId>) {
        self.parent = parent;
    }

    fn can_have_children(&self) -> bool {
        true
    }

    fn text_content(&self) -> Option<&str> {
        None // Content controls don't directly contain text
    }
}

// =============================================================================
// Content Control Appearance
// =============================================================================

/// Visual appearance style for content controls
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentControlAppearance {
    /// Bounded box (default)
    BoundingBox,
    /// No visible boundary
    Hidden,
    /// Show start and end tags
    Tags,
}

impl Default for ContentControlAppearance {
    fn default() -> Self {
        ContentControlAppearance::BoundingBox
    }
}

impl ContentControlAppearance {
    /// Get the OOXML appearance value
    pub fn ooxml_value(&self) -> &'static str {
        match self {
            ContentControlAppearance::BoundingBox => "boundingBox",
            ContentControlAppearance::Hidden => "hidden",
            ContentControlAppearance::Tags => "tags",
        }
    }

    /// Parse from OOXML appearance value
    pub fn from_ooxml(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "boundingbox" => ContentControlAppearance::BoundingBox,
            "hidden" => ContentControlAppearance::Hidden,
            "tags" => ContentControlAppearance::Tags,
            _ => ContentControlAppearance::BoundingBox,
        }
    }
}

// =============================================================================
// Content Control Registry
// =============================================================================

/// Registry for tracking all content controls in a document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentControlRegistry {
    /// All content controls indexed by ID
    controls: HashMap<NodeId, ContentControl>,
    /// Controls indexed by tag
    by_tag: HashMap<String, Vec<NodeId>>,
    /// Controls indexed by OOXML SDT ID
    by_sdt_id: HashMap<i64, NodeId>,
}

impl ContentControlRegistry {
    /// Create a new content control registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a content control into the registry
    pub fn insert(&mut self, control: ContentControl) -> NodeId {
        let id = control.id;
        let tag = control.tag.clone();
        let sdt_id = control.sdt_id;

        // Index by tag
        if !tag.is_empty() {
            self.by_tag.entry(tag).or_insert_with(Vec::new).push(id);
        }

        // Index by SDT ID
        if let Some(sid) = sdt_id {
            self.by_sdt_id.insert(sid, id);
        }

        self.controls.insert(id, control);
        id
    }

    /// Remove a content control from the registry
    pub fn remove(&mut self, id: NodeId) -> Option<ContentControl> {
        if let Some(control) = self.controls.remove(&id) {
            // Remove from tag index
            if !control.tag.is_empty() {
                if let Some(ids) = self.by_tag.get_mut(&control.tag) {
                    ids.retain(|&cid| cid != id);
                    if ids.is_empty() {
                        self.by_tag.remove(&control.tag);
                    }
                }
            }

            // Remove from SDT ID index
            if let Some(sdt_id) = control.sdt_id {
                self.by_sdt_id.remove(&sdt_id);
            }

            Some(control)
        } else {
            None
        }
    }

    /// Get a content control by ID
    pub fn get(&self, id: NodeId) -> Option<&ContentControl> {
        self.controls.get(&id)
    }

    /// Get a mutable content control by ID
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut ContentControl> {
        self.controls.get_mut(&id)
    }

    /// Get content controls by tag
    pub fn get_by_tag(&self, tag: &str) -> Vec<&ContentControl> {
        self.by_tag
            .get(tag)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.controls.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get a content control by OOXML SDT ID
    pub fn get_by_sdt_id(&self, sdt_id: i64) -> Option<&ContentControl> {
        self.by_sdt_id
            .get(&sdt_id)
            .and_then(|id| self.controls.get(id))
    }

    /// Get all content controls
    pub fn all(&self) -> impl Iterator<Item = &ContentControl> {
        self.controls.values()
    }

    /// Get all content control IDs
    pub fn all_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.controls.keys().copied()
    }

    /// Get controls of a specific type
    pub fn of_type(&self, control_type: ContentControlType) -> Vec<&ContentControl> {
        self.controls
            .values()
            .filter(|c| c.control_type == control_type)
            .collect()
    }

    /// Get all checkbox controls
    pub fn checkboxes(&self) -> Vec<&ContentControl> {
        self.of_type(ContentControlType::Checkbox)
    }

    /// Get all dropdown list controls
    pub fn dropdowns(&self) -> Vec<&ContentControl> {
        self.of_type(ContentControlType::DropdownList)
    }

    /// Get all date picker controls
    pub fn date_pickers(&self) -> Vec<&ContentControl> {
        self.of_type(ContentControlType::DatePicker)
    }

    /// Number of content controls
    pub fn len(&self) -> usize {
        self.controls.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.controls.is_empty()
    }

    /// Get all unique tags
    pub fn tags(&self) -> impl Iterator<Item = &String> {
        self.by_tag.keys()
    }

    /// Update the tag of a content control
    pub fn update_tag(&mut self, id: NodeId, new_tag: String) -> bool {
        if let Some(control) = self.controls.get_mut(&id) {
            let old_tag = std::mem::replace(&mut control.tag, new_tag.clone());

            // Update tag index
            if !old_tag.is_empty() {
                if let Some(ids) = self.by_tag.get_mut(&old_tag) {
                    ids.retain(|&cid| cid != id);
                    if ids.is_empty() {
                        self.by_tag.remove(&old_tag);
                    }
                }
            }

            if !new_tag.is_empty() {
                self.by_tag.entry(new_tag).or_insert_with(Vec::new).push(id);
            }

            true
        } else {
            false
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // ContentControlType Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_content_control_type_default() {
        let default = ContentControlType::default();
        assert_eq!(default, ContentControlType::RichText);
    }

    #[test]
    fn test_content_control_type_ooxml_element() {
        assert_eq!(ContentControlType::RichText.ooxml_element(), "w:richText");
        assert_eq!(ContentControlType::PlainText.ooxml_element(), "w:text");
        assert_eq!(ContentControlType::Checkbox.ooxml_element(), "w14:checkbox");
        assert_eq!(
            ContentControlType::DropdownList.ooxml_element(),
            "w:dropDownList"
        );
        assert_eq!(ContentControlType::ComboBox.ooxml_element(), "w:comboBox");
        assert_eq!(ContentControlType::DatePicker.ooxml_element(), "w:date");
        assert_eq!(ContentControlType::Picture.ooxml_element(), "w:picture");
    }

    #[test]
    fn test_content_control_type_can_contain_controls() {
        assert!(ContentControlType::RichText.can_contain_controls());
        assert!(ContentControlType::RepeatingSection.can_contain_controls());
        assert!(ContentControlType::Group.can_contain_controls());
        assert!(!ContentControlType::Checkbox.can_contain_controls());
        assert!(!ContentControlType::PlainText.can_contain_controls());
    }

    #[test]
    fn test_content_control_type_supports_data_binding() {
        assert!(ContentControlType::RichText.supports_data_binding());
        assert!(ContentControlType::PlainText.supports_data_binding());
        assert!(ContentControlType::DatePicker.supports_data_binding());
        assert!(!ContentControlType::Citation.supports_data_binding());
        assert!(!ContentControlType::Bibliography.supports_data_binding());
    }

    // -------------------------------------------------------------------------
    // DataBinding Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_data_binding_new() {
        let binding = DataBinding::new("/root/element");
        assert_eq!(binding.xpath, "/root/element");
        assert!(binding.prefix_mappings.is_empty());
        assert!(binding.store_id.is_empty());
    }

    #[test]
    fn test_data_binding_with_store() {
        let binding = DataBinding::with_store("/root/element", "{12345}");
        assert_eq!(binding.xpath, "/root/element");
        assert_eq!(binding.store_id, "{12345}");
    }

    #[test]
    fn test_data_binding_add_prefix() {
        let mut binding = DataBinding::new("/ns:root/ns:element");
        binding.add_prefix("ns", "http://example.com/namespace");
        assert_eq!(
            binding.prefix_mappings.get("ns"),
            Some(&"http://example.com/namespace".to_string())
        );
    }

    #[test]
    fn test_data_binding_prefix_mappings_string() {
        let mut binding = DataBinding::new("/ns:root");
        binding.add_prefix("ns", "http://example.com");
        let mapping_str = binding.prefix_mappings_string();
        assert!(mapping_str.contains("xmlns:ns='http://example.com'"));
    }

    // -------------------------------------------------------------------------
    // ValidationRule Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validation_rule_new() {
        let rule = ValidationRule::new();
        assert!(!rule.required);
        assert!(rule.regex.is_none());
        assert!(rule.min_length.is_none());
        assert!(rule.max_length.is_none());
    }

    #[test]
    fn test_validation_rule_required() {
        let rule = ValidationRule::required();
        assert!(rule.required);
    }

    #[test]
    fn test_validation_rule_with_length() {
        let rule = ValidationRule::with_length(Some(5), Some(100));
        assert_eq!(rule.min_length, Some(5));
        assert_eq!(rule.max_length, Some(100));
    }

    #[test]
    fn test_validation_rule_validate_required() {
        let rule = ValidationRule::required();
        assert!(rule.validate("some text").is_valid());
        assert!(!rule.validate("").is_valid());
    }

    #[test]
    fn test_validation_rule_validate_length() {
        let rule = ValidationRule::with_length(Some(5), Some(10));
        assert!(rule.validate("hello").is_valid());
        assert!(!rule.validate("hi").is_valid());
        assert!(!rule.validate("hello world!").is_valid());
    }

    #[test]
    fn test_validation_rule_builder() {
        let rule = ValidationRule::new()
            .set_required(true)
            .set_length(Some(1), Some(50))
            .set_error("Custom error message");

        assert!(rule.required);
        assert_eq!(rule.min_length, Some(1));
        assert_eq!(rule.max_length, Some(50));
        assert_eq!(rule.custom_error, Some("Custom error message".to_string()));
    }

    #[test]
    fn test_validation_result_errors() {
        let result = ValidationResult::Invalid(vec![ValidationError::Required]);
        assert!(!result.is_valid());
        assert_eq!(result.errors().len(), 1);

        let valid = ValidationResult::Valid;
        assert!(valid.is_valid());
        assert!(valid.errors().is_empty());
    }

    // -------------------------------------------------------------------------
    // ControlProperties Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_control_properties_for_type() {
        let props = ControlProperties::for_type(ContentControlType::Checkbox);
        assert!(matches!(props, ControlProperties::Checkbox { .. }));

        let props = ControlProperties::for_type(ContentControlType::DatePicker);
        assert!(matches!(props, ControlProperties::DatePicker { .. }));
    }

    #[test]
    fn test_control_properties_control_type() {
        let props = ControlProperties::Checkbox {
            checked: true,
            checked_symbol: CheckboxSymbol::default_checked(),
            unchecked_symbol: CheckboxSymbol::default_unchecked(),
        };
        assert_eq!(props.control_type(), ContentControlType::Checkbox);
    }

    // -------------------------------------------------------------------------
    // CheckboxSymbol Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_checkbox_symbol_new() {
        let symbol = CheckboxSymbol::new('X');
        assert_eq!(symbol.character, 'X');
        assert!(symbol.font.is_none());
    }

    #[test]
    fn test_checkbox_symbol_with_font() {
        let symbol = CheckboxSymbol::with_font('X', "Arial");
        assert_eq!(symbol.character, 'X');
        assert_eq!(symbol.font, Some("Arial".to_string()));
    }

    #[test]
    fn test_checkbox_symbol_defaults() {
        let checked = CheckboxSymbol::default_checked();
        let unchecked = CheckboxSymbol::default_unchecked();
        assert_ne!(checked.character, unchecked.character);
    }

    // -------------------------------------------------------------------------
    // ListItem Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_list_item_new() {
        let item = ListItem::new("Option 1");
        assert_eq!(item.display_text, "Option 1");
        assert_eq!(item.value, "Option 1");
    }

    #[test]
    fn test_list_item_with_value() {
        let item = ListItem::with_value("Display Text", "underlying_value");
        assert_eq!(item.display_text, "Display Text");
        assert_eq!(item.value, "underlying_value");
    }

    // -------------------------------------------------------------------------
    // CalendarType Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_calendar_type_default() {
        assert_eq!(CalendarType::default(), CalendarType::Gregorian);
    }

    #[test]
    fn test_calendar_type_ooxml_value() {
        assert_eq!(CalendarType::Gregorian.ooxml_value(), "gregorian");
        assert_eq!(CalendarType::Hebrew.ooxml_value(), "hebrew");
        assert_eq!(CalendarType::Hijri.ooxml_value(), "hijri");
    }

    #[test]
    fn test_calendar_type_from_ooxml() {
        assert_eq!(CalendarType::from_ooxml("gregorian"), CalendarType::Gregorian);
        assert_eq!(CalendarType::from_ooxml("HEBREW"), CalendarType::Hebrew);
        assert_eq!(CalendarType::from_ooxml("unknown"), CalendarType::Gregorian);
    }

    // -------------------------------------------------------------------------
    // ContentControl Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_content_control_new() {
        let control = ContentControl::new(ContentControlType::PlainText);
        assert_eq!(control.control_type, ContentControlType::PlainText);
        assert!(control.tag.is_empty());
        assert!(control.title.is_empty());
        assert!(!control.locked);
        assert!(!control.contents_locked);
    }

    #[test]
    fn test_content_control_factory_methods() {
        assert_eq!(
            ContentControl::rich_text().control_type,
            ContentControlType::RichText
        );
        assert_eq!(
            ContentControl::plain_text().control_type,
            ContentControlType::PlainText
        );
        assert_eq!(
            ContentControl::checkbox().control_type,
            ContentControlType::Checkbox
        );
        assert_eq!(
            ContentControl::dropdown_list().control_type,
            ContentControlType::DropdownList
        );
        assert_eq!(
            ContentControl::combo_box().control_type,
            ContentControlType::ComboBox
        );
        assert_eq!(
            ContentControl::date_picker().control_type,
            ContentControlType::DatePicker
        );
        assert_eq!(
            ContentControl::picture().control_type,
            ContentControlType::Picture
        );
    }

    #[test]
    fn test_content_control_builder() {
        let control = ContentControl::plain_text()
            .with_tag("username")
            .with_title("User Name")
            .with_placeholder("Enter your name")
            .with_locked(true)
            .with_contents_locked(false);

        assert_eq!(control.tag, "username");
        assert_eq!(control.title, "User Name");
        assert_eq!(control.placeholder, "Enter your name");
        assert!(control.locked);
        assert!(!control.contents_locked);
    }

    #[test]
    fn test_content_control_add_remove_child() {
        let mut control = ContentControl::rich_text();
        let child_id = NodeId::new();

        control.add_child(child_id);
        assert!(control.has_content());
        assert_eq!(control.children().len(), 1);

        assert!(control.remove_child(child_id));
        assert!(!control.has_content());
    }

    #[test]
    fn test_content_control_checkbox_operations() {
        let mut control = ContentControl::checkbox();

        assert_eq!(control.is_checked(), Some(false));
        assert!(control.set_checked(true));
        assert_eq!(control.is_checked(), Some(true));
        assert!(control.toggle_checked());
        assert_eq!(control.is_checked(), Some(false));

        // Operations on non-checkbox should fail
        let mut text_control = ContentControl::plain_text();
        assert_eq!(text_control.is_checked(), None);
        assert!(!text_control.set_checked(true));
    }

    #[test]
    fn test_content_control_dropdown_operations() {
        let mut control = ContentControl::dropdown_list();

        control.add_list_item(ListItem::new("Option 1"));
        control.add_list_item(ListItem::new("Option 2"));
        control.add_list_item(ListItem::new("Option 3"));

        assert_eq!(control.list_items().map(|i| i.len()), Some(3));
        assert_eq!(control.selected_index(), None);

        assert!(control.set_selected_index(Some(1)));
        assert_eq!(control.selected_index(), Some(1));

        // Cannot set index beyond items
        assert!(!control.set_selected_index(Some(10)));

        // Remove item and check index adjustment
        assert!(control.remove_list_item(0).is_some());
        assert_eq!(control.selected_index(), Some(0));
    }

    #[test]
    fn test_content_control_date_picker_operations() {
        let mut control = ContentControl::date_picker();

        assert_eq!(control.selected_date(), None);

        let date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        assert!(control.set_selected_date(Some(date)));
        assert_eq!(control.selected_date(), Some(date));

        assert!(control.set_selected_date(None));
        assert_eq!(control.selected_date(), None);
    }

    #[test]
    fn test_content_control_node_trait() {
        let control = ContentControl::plain_text();
        assert_eq!(control.node_type(), NodeType::ContentControl);
        assert!(control.can_have_children());
        assert!(control.parent().is_none());
    }

    // -------------------------------------------------------------------------
    // ContentControlAppearance Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_content_control_appearance_default() {
        assert_eq!(
            ContentControlAppearance::default(),
            ContentControlAppearance::BoundingBox
        );
    }

    #[test]
    fn test_content_control_appearance_ooxml() {
        assert_eq!(
            ContentControlAppearance::BoundingBox.ooxml_value(),
            "boundingBox"
        );
        assert_eq!(ContentControlAppearance::Hidden.ooxml_value(), "hidden");
        assert_eq!(ContentControlAppearance::Tags.ooxml_value(), "tags");
    }

    #[test]
    fn test_content_control_appearance_from_ooxml() {
        assert_eq!(
            ContentControlAppearance::from_ooxml("boundingBox"),
            ContentControlAppearance::BoundingBox
        );
        assert_eq!(
            ContentControlAppearance::from_ooxml("HIDDEN"),
            ContentControlAppearance::Hidden
        );
        assert_eq!(
            ContentControlAppearance::from_ooxml("unknown"),
            ContentControlAppearance::BoundingBox
        );
    }

    // -------------------------------------------------------------------------
    // ContentControlRegistry Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_content_control_registry_new() {
        let registry = ContentControlRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_content_control_registry_insert_and_get() {
        let mut registry = ContentControlRegistry::new();

        let control = ContentControl::plain_text().with_tag("test_tag");
        let id = registry.insert(control);

        assert_eq!(registry.len(), 1);
        assert!(registry.get(id).is_some());
        assert_eq!(registry.get(id).unwrap().tag, "test_tag");
    }

    #[test]
    fn test_content_control_registry_get_by_tag() {
        let mut registry = ContentControlRegistry::new();

        registry.insert(ContentControl::plain_text().with_tag("name"));
        registry.insert(ContentControl::plain_text().with_tag("name"));
        registry.insert(ContentControl::plain_text().with_tag("email"));

        let by_name = registry.get_by_tag("name");
        assert_eq!(by_name.len(), 2);

        let by_email = registry.get_by_tag("email");
        assert_eq!(by_email.len(), 1);

        let by_unknown = registry.get_by_tag("unknown");
        assert!(by_unknown.is_empty());
    }

    #[test]
    fn test_content_control_registry_get_by_sdt_id() {
        let mut registry = ContentControlRegistry::new();

        let mut control = ContentControl::plain_text();
        control.sdt_id = Some(12345);
        let id = registry.insert(control);

        assert!(registry.get_by_sdt_id(12345).is_some());
        assert_eq!(registry.get_by_sdt_id(12345).unwrap().id(), id);
        assert!(registry.get_by_sdt_id(99999).is_none());
    }

    #[test]
    fn test_content_control_registry_remove() {
        let mut registry = ContentControlRegistry::new();

        let mut control = ContentControl::plain_text().with_tag("test");
        control.sdt_id = Some(123);
        let id = registry.insert(control);

        assert_eq!(registry.len(), 1);

        let removed = registry.remove(id);
        assert!(removed.is_some());
        assert_eq!(registry.len(), 0);
        assert!(registry.get(id).is_none());
        assert!(registry.get_by_tag("test").is_empty());
        assert!(registry.get_by_sdt_id(123).is_none());
    }

    #[test]
    fn test_content_control_registry_of_type() {
        let mut registry = ContentControlRegistry::new();

        registry.insert(ContentControl::checkbox());
        registry.insert(ContentControl::checkbox());
        registry.insert(ContentControl::plain_text());
        registry.insert(ContentControl::date_picker());

        assert_eq!(registry.checkboxes().len(), 2);
        assert_eq!(registry.dropdowns().len(), 0);
        assert_eq!(registry.date_pickers().len(), 1);
        assert_eq!(registry.of_type(ContentControlType::PlainText).len(), 1);
    }

    #[test]
    fn test_content_control_registry_update_tag() {
        let mut registry = ContentControlRegistry::new();

        let control = ContentControl::plain_text().with_tag("old_tag");
        let id = registry.insert(control);

        assert_eq!(registry.get_by_tag("old_tag").len(), 1);
        assert!(registry.get_by_tag("new_tag").is_empty());

        assert!(registry.update_tag(id, "new_tag".to_string()));

        assert!(registry.get_by_tag("old_tag").is_empty());
        assert_eq!(registry.get_by_tag("new_tag").len(), 1);
    }

    #[test]
    fn test_content_control_registry_tags() {
        let mut registry = ContentControlRegistry::new();

        registry.insert(ContentControl::plain_text().with_tag("tag_a"));
        registry.insert(ContentControl::plain_text().with_tag("tag_b"));
        registry.insert(ContentControl::plain_text().with_tag("tag_a"));

        let tags: Vec<_> = registry.tags().collect();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&&"tag_a".to_string()));
        assert!(tags.contains(&&"tag_b".to_string()));
    }

    #[test]
    fn test_content_control_registry_all() {
        let mut registry = ContentControlRegistry::new();

        registry.insert(ContentControl::plain_text());
        registry.insert(ContentControl::checkbox());
        registry.insert(ContentControl::date_picker());

        let all: Vec<_> = registry.all().collect();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_content_control_registry_all_ids() {
        let mut registry = ContentControlRegistry::new();

        let id1 = registry.insert(ContentControl::plain_text());
        let id2 = registry.insert(ContentControl::checkbox());

        let ids: Vec<_> = registry.all_ids().collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }
}
