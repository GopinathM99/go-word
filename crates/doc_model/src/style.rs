//! Style system - Style definitions, registry, and cascade resolution
//!
//! This module implements a Word-like style system with:
//! - Named styles (Paragraph, Character, Table, Numbering)
//! - Style inheritance via `based_on` chains
//! - Property merging with direct formatting overrides

use crate::{Alignment, LineSpacing, ListProperties};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Style Identifier
// =============================================================================

/// Unique identifier for a style
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StyleId(pub String);

impl StyleId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for StyleId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for StyleId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for StyleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// =============================================================================
// Style Types
// =============================================================================

/// The type of style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StyleType {
    /// Paragraph style - applied to entire paragraphs
    Paragraph,
    /// Character style - applied to text runs
    Character,
    /// Table style - applied to tables
    Table,
    /// Numbering style - for list formatting
    Numbering,
}

// =============================================================================
// Character Properties
// =============================================================================

/// Character formatting properties
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CharacterProperties {
    /// Font family name
    pub font_family: Option<String>,
    /// Font size in points
    pub font_size: Option<f32>,
    /// Bold formatting
    pub bold: Option<bool>,
    /// Italic formatting
    pub italic: Option<bool>,
    /// Underline formatting
    pub underline: Option<bool>,
    /// Strikethrough formatting
    pub strikethrough: Option<bool>,
    /// Text color (CSS color string)
    pub color: Option<String>,
    /// Highlight/background color (CSS color string)
    pub highlight: Option<String>,
    /// Vertical alignment (superscript/subscript)
    pub vertical_align: Option<VerticalAlign>,
    /// All caps
    pub all_caps: Option<bool>,
    /// Small caps
    pub small_caps: Option<bool>,
    /// Character spacing adjustment in points
    pub spacing: Option<f32>,
}

impl CharacterProperties {
    /// Create new empty character properties
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge another set of properties on top of this one
    /// Properties from `other` override properties from `self` when present
    pub fn merge(&self, other: &CharacterProperties) -> CharacterProperties {
        CharacterProperties {
            font_family: other.font_family.clone().or_else(|| self.font_family.clone()),
            font_size: other.font_size.or(self.font_size),
            bold: other.bold.or(self.bold),
            italic: other.italic.or(self.italic),
            underline: other.underline.or(self.underline),
            strikethrough: other.strikethrough.or(self.strikethrough),
            color: other.color.clone().or_else(|| self.color.clone()),
            highlight: other.highlight.clone().or_else(|| self.highlight.clone()),
            vertical_align: other.vertical_align.or(self.vertical_align),
            all_caps: other.all_caps.or(self.all_caps),
            small_caps: other.small_caps.or(self.small_caps),
            spacing: other.spacing.or(self.spacing),
        }
    }

    /// Check if all properties are None
    pub fn is_empty(&self) -> bool {
        self.font_family.is_none()
            && self.font_size.is_none()
            && self.bold.is_none()
            && self.italic.is_none()
            && self.underline.is_none()
            && self.strikethrough.is_none()
            && self.color.is_none()
            && self.highlight.is_none()
            && self.vertical_align.is_none()
            && self.all_caps.is_none()
            && self.small_caps.is_none()
            && self.spacing.is_none()
    }
}

/// Vertical text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerticalAlign {
    Baseline,
    Superscript,
    Subscript,
}

// =============================================================================
// Paragraph Properties
// =============================================================================

/// Border style for paragraph borders
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BorderStyle {
    /// Border width in points
    pub width: f32,
    /// Border style type
    pub style: BorderStyleType,
    /// Border color (CSS color string)
    pub color: String,
}

/// Border style types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BorderStyleType {
    None,
    Single,
    Double,
    Dotted,
    Dashed,
}

/// Paragraph borders
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ParagraphBorders {
    pub top: Option<BorderStyle>,
    pub bottom: Option<BorderStyle>,
    pub left: Option<BorderStyle>,
    pub right: Option<BorderStyle>,
}

/// Text direction for paragraphs (RTL/LTR support)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextDirection {
    /// Left-to-right (default for Latin, etc.)
    #[default]
    Ltr,
    /// Right-to-left (for Hebrew, Arabic, etc.)
    Rtl,
    /// Auto-detect from first strong character
    Auto,
}

/// Paragraph formatting properties
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ParagraphProperties {
    /// Text alignment
    pub alignment: Option<Alignment>,
    /// Left indent in points
    pub indent_left: Option<f32>,
    /// Right indent in points
    pub indent_right: Option<f32>,
    /// First line indent in points (negative for hanging)
    pub indent_first_line: Option<f32>,
    /// Space before paragraph in points
    pub space_before: Option<f32>,
    /// Space after paragraph in points
    pub space_after: Option<f32>,
    /// Line spacing
    pub line_spacing: Option<LineSpacing>,
    /// Keep with next paragraph
    pub keep_with_next: Option<bool>,
    /// Keep lines together (no page break within)
    pub keep_together: Option<bool>,
    /// Page break before
    pub page_break_before: Option<bool>,
    /// Widow/orphan control
    pub widow_control: Option<bool>,
    /// Paragraph borders
    pub borders: Option<ParagraphBorders>,
    /// Background/shading color
    pub background_color: Option<String>,
    /// Outline level (0 = body text, 1-9 = heading levels)
    pub outline_level: Option<u8>,
    /// List/numbering properties
    pub list_props: Option<ListProperties>,
    /// Text direction (LTR, RTL, or Auto)
    pub direction: Option<TextDirection>,
}

impl ParagraphProperties {
    /// Create new empty paragraph properties
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge another set of properties on top of this one
    pub fn merge(&self, other: &ParagraphProperties) -> ParagraphProperties {
        ParagraphProperties {
            alignment: other.alignment.or(self.alignment),
            indent_left: other.indent_left.or(self.indent_left),
            indent_right: other.indent_right.or(self.indent_right),
            indent_first_line: other.indent_first_line.or(self.indent_first_line),
            space_before: other.space_before.or(self.space_before),
            space_after: other.space_after.or(self.space_after),
            line_spacing: other.line_spacing.or(self.line_spacing),
            keep_with_next: other.keep_with_next.or(self.keep_with_next),
            keep_together: other.keep_together.or(self.keep_together),
            page_break_before: other.page_break_before.or(self.page_break_before),
            widow_control: other.widow_control.or(self.widow_control),
            borders: other.borders.clone().or_else(|| self.borders.clone()),
            background_color: other.background_color.clone().or_else(|| self.background_color.clone()),
            outline_level: other.outline_level.or(self.outline_level),
            list_props: other.list_props.clone().or_else(|| self.list_props.clone()),
            direction: other.direction.or(self.direction),
        }
    }

    /// Check if all properties are None
    pub fn is_empty(&self) -> bool {
        self.alignment.is_none()
            && self.indent_left.is_none()
            && self.indent_right.is_none()
            && self.indent_first_line.is_none()
            && self.space_before.is_none()
            && self.space_after.is_none()
            && self.line_spacing.is_none()
            && self.keep_with_next.is_none()
            && self.keep_together.is_none()
            && self.page_break_before.is_none()
            && self.widow_control.is_none()
            && self.borders.is_none()
            && self.background_color.is_none()
            && self.outline_level.is_none()
            && self.list_props.is_none()
            && self.direction.is_none()
    }
}

// =============================================================================
// Style Definition
// =============================================================================

/// A complete style definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    /// Unique identifier for this style
    pub id: StyleId,
    /// Display name
    pub name: String,
    /// Style type
    pub style_type: StyleType,
    /// Base style this style inherits from
    pub based_on: Option<StyleId>,
    /// Style to use for following paragraph (only for paragraph styles)
    pub next_style: Option<StyleId>,
    /// Whether this is a built-in style
    pub built_in: bool,
    /// Whether to hide from style gallery
    pub hidden: bool,
    /// Priority for sorting in style gallery (lower = higher priority)
    pub priority: u32,
    /// Paragraph properties (for paragraph styles)
    pub paragraph_props: ParagraphProperties,
    /// Character properties (for paragraph and character styles)
    pub character_props: CharacterProperties,
}

impl Style {
    /// Create a new paragraph style
    pub fn paragraph(id: impl Into<StyleId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            style_type: StyleType::Paragraph,
            based_on: None,
            next_style: None,
            built_in: false,
            hidden: false,
            priority: 99,
            paragraph_props: ParagraphProperties::default(),
            character_props: CharacterProperties::default(),
        }
    }

    /// Create a new character style
    pub fn character(id: impl Into<StyleId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            style_type: StyleType::Character,
            based_on: None,
            next_style: None,
            built_in: false,
            hidden: false,
            priority: 99,
            paragraph_props: ParagraphProperties::default(),
            character_props: CharacterProperties::default(),
        }
    }

    /// Set the base style
    pub fn with_based_on(mut self, base: impl Into<StyleId>) -> Self {
        self.based_on = Some(base.into());
        self
    }

    /// Set as built-in
    pub fn as_built_in(mut self) -> Self {
        self.built_in = true;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Set paragraph properties
    pub fn with_paragraph_props(mut self, props: ParagraphProperties) -> Self {
        self.paragraph_props = props;
        self
    }

    /// Set character properties
    pub fn with_character_props(mut self, props: CharacterProperties) -> Self {
        self.character_props = props;
        self
    }

    /// Set next style
    pub fn with_next_style(mut self, next: impl Into<StyleId>) -> Self {
        self.next_style = Some(next.into());
        self
    }
}

// =============================================================================
// Resolved Style (Computed Properties)
// =============================================================================

/// A fully resolved style with all inherited properties merged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedStyle {
    /// The style ID this was resolved from
    pub style_id: StyleId,
    /// Fully resolved paragraph properties
    pub paragraph_props: ParagraphProperties,
    /// Fully resolved character properties
    pub character_props: CharacterProperties,
    /// Chain of style IDs that contributed to this resolution
    pub inheritance_chain: Vec<StyleId>,
}

// =============================================================================
// Property Source Tracking
// =============================================================================

/// Tracks where a property value came from
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PropertySource {
    /// From direct formatting on the element
    DirectFormatting,
    /// From a named style
    Style(StyleId),
    /// Default value (not explicitly set)
    Default,
}

/// Computed properties with source tracking for the inspector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedCharacterProperty<T: Clone> {
    /// The computed value
    pub value: T,
    /// Where this value came from
    pub source: PropertySource,
}

/// Full computed character properties with sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedCharacterProperties {
    pub font_family: ComputedCharacterProperty<String>,
    pub font_size: ComputedCharacterProperty<f32>,
    pub bold: ComputedCharacterProperty<bool>,
    pub italic: ComputedCharacterProperty<bool>,
    pub underline: ComputedCharacterProperty<bool>,
    pub color: ComputedCharacterProperty<String>,
}

impl Default for ComputedCharacterProperties {
    fn default() -> Self {
        Self {
            font_family: ComputedCharacterProperty {
                value: "Calibri".to_string(),
                source: PropertySource::Default,
            },
            font_size: ComputedCharacterProperty {
                value: 11.0,
                source: PropertySource::Default,
            },
            bold: ComputedCharacterProperty {
                value: false,
                source: PropertySource::Default,
            },
            italic: ComputedCharacterProperty {
                value: false,
                source: PropertySource::Default,
            },
            underline: ComputedCharacterProperty {
                value: false,
                source: PropertySource::Default,
            },
            color: ComputedCharacterProperty {
                value: "#000000".to_string(),
                source: PropertySource::Default,
            },
        }
    }
}

/// Computed paragraph property with source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedParagraphProperty<T: Clone> {
    pub value: T,
    pub source: PropertySource,
}

/// Full computed paragraph properties with sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedParagraphProperties {
    pub alignment: ComputedParagraphProperty<Alignment>,
    pub indent_left: ComputedParagraphProperty<f32>,
    pub indent_right: ComputedParagraphProperty<f32>,
    pub indent_first_line: ComputedParagraphProperty<f32>,
    pub space_before: ComputedParagraphProperty<f32>,
    pub space_after: ComputedParagraphProperty<f32>,
    pub line_spacing: ComputedParagraphProperty<LineSpacing>,
}

impl Default for ComputedParagraphProperties {
    fn default() -> Self {
        Self {
            alignment: ComputedParagraphProperty {
                value: Alignment::Left,
                source: PropertySource::Default,
            },
            indent_left: ComputedParagraphProperty {
                value: 0.0,
                source: PropertySource::Default,
            },
            indent_right: ComputedParagraphProperty {
                value: 0.0,
                source: PropertySource::Default,
            },
            indent_first_line: ComputedParagraphProperty {
                value: 0.0,
                source: PropertySource::Default,
            },
            space_before: ComputedParagraphProperty {
                value: 0.0,
                source: PropertySource::Default,
            },
            space_after: ComputedParagraphProperty {
                value: 8.0,
                source: PropertySource::Default,
            },
            line_spacing: ComputedParagraphProperty {
                value: LineSpacing::Multiple(1.08),
                source: PropertySource::Default,
            },
        }
    }
}

// =============================================================================
// Style Registry
// =============================================================================

/// Registry for storing and looking up styles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleRegistry {
    /// All styles indexed by ID
    styles: HashMap<StyleId, Style>,
    /// Default paragraph style ID
    default_paragraph_style: StyleId,
    /// Default character style ID
    default_character_style: StyleId,
}

impl StyleRegistry {
    /// Create a new style registry with built-in styles
    pub fn new() -> Self {
        let mut registry = Self {
            styles: HashMap::new(),
            default_paragraph_style: StyleId::new("Normal"),
            default_character_style: StyleId::new("DefaultParagraphFont"),
        };
        registry.register_built_in_styles();
        registry
    }

    /// Register all built-in styles
    fn register_built_in_styles(&mut self) {
        // Normal style - the base paragraph style
        let normal = Style::paragraph("Normal", "Normal")
            .as_built_in()
            .with_priority(0)
            .with_paragraph_props(ParagraphProperties {
                alignment: Some(Alignment::Left),
                space_after: Some(8.0),
                line_spacing: Some(LineSpacing::Multiple(1.08)),
                ..Default::default()
            })
            .with_character_props(CharacterProperties {
                font_family: Some("Calibri".to_string()),
                font_size: Some(11.0),
                color: Some("#000000".to_string()),
                ..Default::default()
            })
            .with_next_style("Normal");
        self.register(normal);

        // Default Paragraph Font - base character style
        let default_font = Style::character("DefaultParagraphFont", "Default Paragraph Font")
            .as_built_in()
            .with_priority(1);
        self.register(default_font);

        // Heading styles
        self.register_heading_styles();

        // Title and Subtitle
        self.register_title_styles();

        // Other common styles
        self.register_common_styles();
    }

    fn register_heading_styles(&mut self) {
        // Heading 1
        let heading1 = Style::paragraph("Heading1", "Heading 1")
            .as_built_in()
            .with_priority(9)
            .with_based_on("Normal")
            .with_next_style("Normal")
            .with_paragraph_props(ParagraphProperties {
                space_before: Some(12.0),
                space_after: Some(0.0),
                keep_with_next: Some(true),
                keep_together: Some(true),
                outline_level: Some(1),
                ..Default::default()
            })
            .with_character_props(CharacterProperties {
                font_family: Some("Calibri Light".to_string()),
                font_size: Some(16.0),
                color: Some("#2F5496".to_string()),
                ..Default::default()
            });
        self.register(heading1);

        // Heading 2
        let heading2 = Style::paragraph("Heading2", "Heading 2")
            .as_built_in()
            .with_priority(9)
            .with_based_on("Normal")
            .with_next_style("Normal")
            .with_paragraph_props(ParagraphProperties {
                space_before: Some(2.0),
                space_after: Some(0.0),
                keep_with_next: Some(true),
                keep_together: Some(true),
                outline_level: Some(2),
                ..Default::default()
            })
            .with_character_props(CharacterProperties {
                font_family: Some("Calibri Light".to_string()),
                font_size: Some(13.0),
                color: Some("#2F5496".to_string()),
                ..Default::default()
            });
        self.register(heading2);

        // Heading 3
        let heading3 = Style::paragraph("Heading3", "Heading 3")
            .as_built_in()
            .with_priority(9)
            .with_based_on("Normal")
            .with_next_style("Normal")
            .with_paragraph_props(ParagraphProperties {
                space_before: Some(2.0),
                space_after: Some(0.0),
                keep_with_next: Some(true),
                keep_together: Some(true),
                outline_level: Some(3),
                ..Default::default()
            })
            .with_character_props(CharacterProperties {
                font_family: Some("Calibri Light".to_string()),
                font_size: Some(12.0),
                color: Some("#1F3763".to_string()),
                ..Default::default()
            });
        self.register(heading3);

        // Heading 4
        let heading4 = Style::paragraph("Heading4", "Heading 4")
            .as_built_in()
            .with_priority(9)
            .with_based_on("Normal")
            .with_next_style("Normal")
            .with_paragraph_props(ParagraphProperties {
                space_before: Some(2.0),
                space_after: Some(0.0),
                keep_with_next: Some(true),
                keep_together: Some(true),
                outline_level: Some(4),
                ..Default::default()
            })
            .with_character_props(CharacterProperties {
                italic: Some(true),
                color: Some("#2F5496".to_string()),
                ..Default::default()
            });
        self.register(heading4);

        // Heading 5
        let heading5 = Style::paragraph("Heading5", "Heading 5")
            .as_built_in()
            .with_priority(9)
            .with_based_on("Normal")
            .with_next_style("Normal")
            .with_paragraph_props(ParagraphProperties {
                space_before: Some(2.0),
                space_after: Some(0.0),
                keep_with_next: Some(true),
                keep_together: Some(true),
                outline_level: Some(5),
                ..Default::default()
            })
            .with_character_props(CharacterProperties {
                color: Some("#2F5496".to_string()),
                ..Default::default()
            });
        self.register(heading5);

        // Heading 6
        let heading6 = Style::paragraph("Heading6", "Heading 6")
            .as_built_in()
            .with_priority(9)
            .with_based_on("Normal")
            .with_next_style("Normal")
            .with_paragraph_props(ParagraphProperties {
                space_before: Some(2.0),
                space_after: Some(0.0),
                keep_with_next: Some(true),
                keep_together: Some(true),
                outline_level: Some(6),
                ..Default::default()
            })
            .with_character_props(CharacterProperties {
                color: Some("#1F3763".to_string()),
                ..Default::default()
            });
        self.register(heading6);
    }

    fn register_title_styles(&mut self) {
        // Title
        let title = Style::paragraph("Title", "Title")
            .as_built_in()
            .with_priority(10)
            .with_based_on("Normal")
            .with_next_style("Normal")
            .with_paragraph_props(ParagraphProperties {
                space_after: Some(0.0),
                ..Default::default()
            })
            .with_character_props(CharacterProperties {
                font_family: Some("Calibri Light".to_string()),
                font_size: Some(28.0),
                spacing: Some(-0.5),
                ..Default::default()
            });
        self.register(title);

        // Subtitle
        let subtitle = Style::paragraph("Subtitle", "Subtitle")
            .as_built_in()
            .with_priority(11)
            .with_based_on("Normal")
            .with_next_style("Normal")
            .with_paragraph_props(ParagraphProperties {
                space_after: Some(8.0),
                ..Default::default()
            })
            .with_character_props(CharacterProperties {
                font_size: Some(11.0),
                color: Some("#5A5A5A".to_string()),
                spacing: Some(0.75),
                ..Default::default()
            });
        self.register(subtitle);
    }

    fn register_common_styles(&mut self) {
        // Quote
        let quote = Style::paragraph("Quote", "Quote")
            .as_built_in()
            .with_priority(29)
            .with_based_on("Normal")
            .with_next_style("Normal")
            .with_paragraph_props(ParagraphProperties {
                indent_left: Some(43.2), // 0.6 inches
                indent_right: Some(43.2),
                space_before: Some(10.0),
                space_after: Some(10.0),
                ..Default::default()
            })
            .with_character_props(CharacterProperties {
                italic: Some(true),
                color: Some("#404040".to_string()),
                ..Default::default()
            });
        self.register(quote);

        // Intense Quote
        let intense_quote = Style::paragraph("IntenseQuote", "Intense Quote")
            .as_built_in()
            .with_priority(30)
            .with_based_on("Normal")
            .with_next_style("Normal")
            .with_paragraph_props(ParagraphProperties {
                indent_left: Some(43.2),
                indent_right: Some(43.2),
                space_before: Some(18.0),
                space_after: Some(18.0),
                alignment: Some(Alignment::Center),
                ..Default::default()
            })
            .with_character_props(CharacterProperties {
                italic: Some(true),
                color: Some("#2F5496".to_string()),
                ..Default::default()
            });
        self.register(intense_quote);

        // No Spacing
        let no_spacing = Style::paragraph("NoSpacing", "No Spacing")
            .as_built_in()
            .with_priority(1)
            .with_paragraph_props(ParagraphProperties {
                space_after: Some(0.0),
                line_spacing: Some(LineSpacing::Multiple(1.0)),
                ..Default::default()
            })
            .with_character_props(CharacterProperties {
                font_family: Some("Calibri".to_string()),
                font_size: Some(11.0),
                ..Default::default()
            });
        self.register(no_spacing);

        // Strong (character style)
        let strong = Style::character("Strong", "Strong")
            .as_built_in()
            .with_priority(22)
            .with_character_props(CharacterProperties {
                bold: Some(true),
                ..Default::default()
            });
        self.register(strong);

        // Emphasis (character style)
        let emphasis = Style::character("Emphasis", "Emphasis")
            .as_built_in()
            .with_priority(20)
            .with_character_props(CharacterProperties {
                italic: Some(true),
                ..Default::default()
            });
        self.register(emphasis);

        // Intense Emphasis (character style)
        let intense_emphasis = Style::character("IntenseEmphasis", "Intense Emphasis")
            .as_built_in()
            .with_priority(21)
            .with_character_props(CharacterProperties {
                italic: Some(true),
                color: Some("#2F5496".to_string()),
                ..Default::default()
            });
        self.register(intense_emphasis);

        // Subtle Reference (character style)
        let subtle_ref = Style::character("SubtleReference", "Subtle Reference")
            .as_built_in()
            .with_priority(31)
            .with_character_props(CharacterProperties {
                small_caps: Some(true),
                color: Some("#5A5A5A".to_string()),
                ..Default::default()
            });
        self.register(subtle_ref);

        // Book Title (character style)
        let book_title = Style::character("BookTitle", "Book Title")
            .as_built_in()
            .with_priority(33)
            .with_character_props(CharacterProperties {
                bold: Some(true),
                small_caps: Some(true),
                spacing: Some(0.25),
                ..Default::default()
            });
        self.register(book_title);

        // List Paragraph
        let list_para = Style::paragraph("ListParagraph", "List Paragraph")
            .as_built_in()
            .with_priority(34)
            .with_based_on("Normal")
            .with_paragraph_props(ParagraphProperties {
                indent_left: Some(36.0), // 0.5 inches
                ..Default::default()
            });
        self.register(list_para);

        // Caption style for figure/table/equation captions
        let caption = Style::paragraph("Caption", "Caption")
            .as_built_in()
            .with_priority(35)
            .with_based_on("Normal")
            .with_next_style("Normal")
            .with_paragraph_props(ParagraphProperties {
                space_before: Some(0.0),
                space_after: Some(10.0),
                ..Default::default()
            })
            .with_character_props(CharacterProperties {
                font_size: Some(9.0),
                italic: Some(true),
                color: Some("#44546A".to_string()),
                ..Default::default()
            });
        self.register(caption);

        // Table of Figures style (for list of figures/tables)
        let tof = Style::paragraph("TableOfFigures", "Table of Figures")
            .as_built_in()
            .with_priority(99)
            .with_based_on("Normal")
            .with_paragraph_props(ParagraphProperties {
                space_after: Some(0.0),
                ..Default::default()
            });
        self.register(tof);
    }

    /// Register a style in the registry
    pub fn register(&mut self, style: Style) {
        self.styles.insert(style.id.clone(), style);
    }

    /// Get a style by ID
    pub fn get(&self, id: &StyleId) -> Option<&Style> {
        self.styles.get(id)
    }

    /// Get a mutable style by ID
    pub fn get_mut(&mut self, id: &StyleId) -> Option<&mut Style> {
        self.styles.get_mut(id)
    }

    /// Check if a style exists
    pub fn contains(&self, id: &StyleId) -> bool {
        self.styles.contains_key(id)
    }

    /// Remove a style (only non-built-in styles)
    pub fn remove(&mut self, id: &StyleId) -> Option<Style> {
        if let Some(style) = self.styles.get(id) {
            if style.built_in {
                return None; // Cannot remove built-in styles
            }
        }
        self.styles.remove(id)
    }

    /// Get the default paragraph style ID
    pub fn default_paragraph_style(&self) -> &StyleId {
        &self.default_paragraph_style
    }

    /// Get the default character style ID
    pub fn default_character_style(&self) -> &StyleId {
        &self.default_character_style
    }

    /// Get all styles
    pub fn all_styles(&self) -> impl Iterator<Item = &Style> {
        self.styles.values()
    }

    /// Get all paragraph styles
    pub fn paragraph_styles(&self) -> impl Iterator<Item = &Style> {
        self.styles
            .values()
            .filter(|s| s.style_type == StyleType::Paragraph)
    }

    /// Get all character styles
    pub fn character_styles(&self) -> impl Iterator<Item = &Style> {
        self.styles
            .values()
            .filter(|s| s.style_type == StyleType::Character)
    }

    /// Get styles for the gallery (sorted by priority, non-hidden)
    pub fn gallery_styles(&self) -> Vec<&Style> {
        let mut styles: Vec<_> = self
            .styles
            .values()
            .filter(|s| !s.hidden && s.style_type == StyleType::Paragraph)
            .collect();
        styles.sort_by_key(|s| s.priority);
        styles
    }

    /// Resolve a style by walking the inheritance chain and merging properties
    pub fn resolve(&self, id: &StyleId) -> Option<ResolvedStyle> {
        let _style = self.styles.get(id)?;

        // Build the inheritance chain (from base to derived)
        let mut chain = Vec::new();
        let mut current_id = Some(id.clone());
        let mut visited = std::collections::HashSet::new();

        while let Some(ref cid) = current_id {
            if visited.contains(cid) {
                break; // Circular reference protection
            }
            visited.insert(cid.clone());

            if let Some(s) = self.styles.get(cid) {
                chain.push(cid.clone());
                current_id = s.based_on.clone();
            } else {
                break;
            }
        }

        // Reverse to go from base to derived
        chain.reverse();

        // Merge properties from base to derived
        let mut paragraph_props = ParagraphProperties::default();
        let mut character_props = CharacterProperties::default();

        for style_id in &chain {
            if let Some(s) = self.styles.get(style_id) {
                paragraph_props = paragraph_props.merge(&s.paragraph_props);
                character_props = character_props.merge(&s.character_props);
            }
        }

        Some(ResolvedStyle {
            style_id: id.clone(),
            paragraph_props,
            character_props,
            inheritance_chain: chain,
        })
    }

    /// Resolve character properties with direct formatting override
    pub fn resolve_character_props(
        &self,
        style_id: Option<&StyleId>,
        direct_formatting: &CharacterProperties,
    ) -> CharacterProperties {
        // Start with defaults
        let mut props = CharacterProperties {
            font_family: Some("Calibri".to_string()),
            font_size: Some(11.0),
            bold: Some(false),
            italic: Some(false),
            underline: Some(false),
            strikethrough: Some(false),
            color: Some("#000000".to_string()),
            ..Default::default()
        };

        // Apply style properties
        if let Some(id) = style_id {
            if let Some(resolved) = self.resolve(id) {
                props = props.merge(&resolved.character_props);
            }
        }

        // Apply direct formatting (always wins)
        props.merge(direct_formatting)
    }

    /// Resolve paragraph properties with direct formatting override
    pub fn resolve_paragraph_props(
        &self,
        style_id: Option<&StyleId>,
        direct_formatting: &ParagraphProperties,
    ) -> ParagraphProperties {
        // Start with defaults
        let mut props = ParagraphProperties {
            alignment: Some(Alignment::Left),
            indent_left: Some(0.0),
            indent_right: Some(0.0),
            indent_first_line: Some(0.0),
            space_before: Some(0.0),
            space_after: Some(8.0),
            line_spacing: Some(LineSpacing::Multiple(1.08)),
            widow_control: Some(true),
            ..Default::default()
        };

        // Apply style properties
        if let Some(id) = style_id {
            if let Some(resolved) = self.resolve(id) {
                props = props.merge(&resolved.paragraph_props);
            }
        }

        // Apply direct formatting (always wins)
        props.merge(direct_formatting)
    }

    /// Compute character properties with source tracking
    pub fn compute_character_props_with_sources(
        &self,
        style_id: Option<&StyleId>,
        direct_formatting: &CharacterProperties,
    ) -> ComputedCharacterProperties {
        let mut result = ComputedCharacterProperties::default();

        // Track the style source
        let style_source = style_id
            .map(|id| PropertySource::Style(id.clone()))
            .unwrap_or(PropertySource::Default);

        // Apply style properties
        if let Some(id) = style_id {
            if let Some(resolved) = self.resolve(id) {
                if let Some(v) = &resolved.character_props.font_family {
                    result.font_family.value = v.clone();
                    result.font_family.source = style_source.clone();
                }
                if let Some(v) = resolved.character_props.font_size {
                    result.font_size.value = v;
                    result.font_size.source = style_source.clone();
                }
                if let Some(v) = resolved.character_props.bold {
                    result.bold.value = v;
                    result.bold.source = style_source.clone();
                }
                if let Some(v) = resolved.character_props.italic {
                    result.italic.value = v;
                    result.italic.source = style_source.clone();
                }
                if let Some(v) = resolved.character_props.underline {
                    result.underline.value = v;
                    result.underline.source = style_source.clone();
                }
                if let Some(v) = &resolved.character_props.color {
                    result.color.value = v.clone();
                    result.color.source = style_source.clone();
                }
            }
        }

        // Apply direct formatting (always wins)
        if let Some(v) = &direct_formatting.font_family {
            result.font_family.value = v.clone();
            result.font_family.source = PropertySource::DirectFormatting;
        }
        if let Some(v) = direct_formatting.font_size {
            result.font_size.value = v;
            result.font_size.source = PropertySource::DirectFormatting;
        }
        if let Some(v) = direct_formatting.bold {
            result.bold.value = v;
            result.bold.source = PropertySource::DirectFormatting;
        }
        if let Some(v) = direct_formatting.italic {
            result.italic.value = v;
            result.italic.source = PropertySource::DirectFormatting;
        }
        if let Some(v) = direct_formatting.underline {
            result.underline.value = v;
            result.underline.source = PropertySource::DirectFormatting;
        }
        if let Some(v) = &direct_formatting.color {
            result.color.value = v.clone();
            result.color.source = PropertySource::DirectFormatting;
        }

        result
    }
}

impl Default for StyleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_registry_creation() {
        let registry = StyleRegistry::new();

        // Check built-in styles exist
        assert!(registry.contains(&StyleId::new("Normal")));
        assert!(registry.contains(&StyleId::new("Heading1")));
        assert!(registry.contains(&StyleId::new("Heading2")));
        assert!(registry.contains(&StyleId::new("Title")));
        assert!(registry.contains(&StyleId::new("Strong")));
    }

    #[test]
    fn test_style_inheritance_chain() {
        let registry = StyleRegistry::new();

        // Heading1 is based on Normal
        let heading1 = registry.get(&StyleId::new("Heading1")).unwrap();
        assert_eq!(heading1.based_on, Some(StyleId::new("Normal")));

        // Resolve should include both in the chain
        let resolved = registry.resolve(&StyleId::new("Heading1")).unwrap();
        assert!(resolved.inheritance_chain.contains(&StyleId::new("Normal")));
        assert!(resolved.inheritance_chain.contains(&StyleId::new("Heading1")));
    }

    #[test]
    fn test_style_resolution_merges_properties() {
        let registry = StyleRegistry::new();

        // Resolve Heading1 - should have merged properties from Normal
        let resolved = registry.resolve(&StyleId::new("Heading1")).unwrap();

        // Font family should come from Heading1 (overrides Normal)
        assert_eq!(resolved.character_props.font_family, Some("Calibri Light".to_string()));

        // Font size should come from Heading1
        assert_eq!(resolved.character_props.font_size, Some(16.0));
    }

    #[test]
    fn test_direct_formatting_override() {
        let registry = StyleRegistry::new();

        // Create direct formatting with bold
        let direct = CharacterProperties {
            bold: Some(true),
            ..Default::default()
        };

        // Resolve with Normal style and direct formatting
        let computed = registry.compute_character_props_with_sources(
            Some(&StyleId::new("Normal")),
            &direct,
        );

        // Bold should come from direct formatting
        assert!(computed.bold.value);
        assert_eq!(computed.bold.source, PropertySource::DirectFormatting);

        // Font family should come from style
        assert_eq!(computed.font_family.source, PropertySource::Style(StyleId::new("Normal")));
    }

    #[test]
    fn test_cannot_remove_builtin_style() {
        let mut registry = StyleRegistry::new();

        // Try to remove Normal - should fail
        let removed = registry.remove(&StyleId::new("Normal"));
        assert!(removed.is_none());
        assert!(registry.contains(&StyleId::new("Normal")));
    }

    #[test]
    fn test_custom_style_registration() {
        let mut registry = StyleRegistry::new();

        // Create a custom style
        let custom = Style::paragraph("MyStyle", "My Custom Style")
            .with_based_on("Normal")
            .with_character_props(CharacterProperties {
                font_family: Some("Arial".to_string()),
                bold: Some(true),
                ..Default::default()
            });

        registry.register(custom);

        // Should be retrievable
        assert!(registry.contains(&StyleId::new("MyStyle")));

        // Should resolve with inheritance
        let resolved = registry.resolve(&StyleId::new("MyStyle")).unwrap();
        assert_eq!(resolved.character_props.font_family, Some("Arial".to_string()));
        assert_eq!(resolved.character_props.bold, Some(true));
    }

    #[test]
    fn test_gallery_styles_sorted_by_priority() {
        let registry = StyleRegistry::new();

        let gallery = registry.gallery_styles();

        // Check that styles are sorted by priority
        let mut last_priority = 0;
        for style in &gallery {
            assert!(style.priority >= last_priority);
            last_priority = style.priority;
        }

        // Normal should be first (priority 0)
        assert_eq!(gallery[0].id, StyleId::new("Normal"));
    }

    #[test]
    fn test_circular_reference_protection() {
        let mut registry = StyleRegistry::new();

        // Create two styles that reference each other
        let style_a = Style::paragraph("StyleA", "Style A")
            .with_based_on("StyleB");
        let style_b = Style::paragraph("StyleB", "Style B")
            .with_based_on("StyleA");

        registry.register(style_a);
        registry.register(style_b);

        // Resolution should not hang
        let resolved = registry.resolve(&StyleId::new("StyleA"));
        assert!(resolved.is_some());
    }

    #[test]
    fn test_character_properties_merge() {
        let base = CharacterProperties {
            font_family: Some("Arial".to_string()),
            font_size: Some(12.0),
            bold: Some(false),
            ..Default::default()
        };

        let derived = CharacterProperties {
            bold: Some(true),
            italic: Some(true),
            ..Default::default()
        };

        let merged = base.merge(&derived);

        // base properties preserved
        assert_eq!(merged.font_family, Some("Arial".to_string()));
        assert_eq!(merged.font_size, Some(12.0));
        // derived properties override
        assert_eq!(merged.bold, Some(true));
        assert_eq!(merged.italic, Some(true));
    }

    #[test]
    fn test_paragraph_properties_merge() {
        let base = ParagraphProperties {
            alignment: Some(Alignment::Left),
            space_after: Some(8.0),
            ..Default::default()
        };

        let derived = ParagraphProperties {
            alignment: Some(Alignment::Center),
            indent_left: Some(36.0),
            ..Default::default()
        };

        let merged = base.merge(&derived);

        // derived alignment overrides base
        assert_eq!(merged.alignment, Some(Alignment::Center));
        // base space_after preserved
        assert_eq!(merged.space_after, Some(8.0));
        // derived indent added
        assert_eq!(merged.indent_left, Some(36.0));
    }
}
