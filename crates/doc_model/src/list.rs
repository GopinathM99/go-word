//! List and numbering system for document paragraphs
//!
//! This module implements OOXML-compatible list definitions with:
//! - Abstract numbering definitions (templates)
//! - Numbering instances (concrete uses of templates)
//! - Multi-level lists (up to 9 levels, 0-8)
//! - Various number formats (decimal, roman, letters, bullets)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// ID Types
// =============================================================================

/// Unique identifier for an abstract numbering definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AbstractNumId(pub u32);

impl AbstractNumId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl Default for AbstractNumId {
    fn default() -> Self {
        Self(0)
    }
}

/// Unique identifier for a numbering instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NumId(pub u32);

impl NumId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl Default for NumId {
    fn default() -> Self {
        Self(0)
    }
}

// =============================================================================
// Number Format
// =============================================================================

/// Number format types for list items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NumberFormat {
    /// Decimal numbers: 1, 2, 3, ...
    #[default]
    Decimal,
    /// Decimal with leading zeros: 01, 02, 03, ...
    DecimalZero,
    /// Lowercase letters: a, b, c, ...
    LowerLetter,
    /// Uppercase letters: A, B, C, ...
    UpperLetter,
    /// Lowercase roman numerals: i, ii, iii, ...
    LowerRoman,
    /// Uppercase roman numerals: I, II, III, ...
    UpperRoman,
    /// Bullet character (uses font glyph)
    Bullet,
    /// No number displayed
    None,
    /// Ordinal: 1st, 2nd, 3rd, ...
    Ordinal,
    /// Cardinal text: One, Two, Three, ...
    CardinalText,
    /// Ordinal text: First, Second, Third, ...
    OrdinalText,
}

impl NumberFormat {
    /// Check if this format is a bullet (non-numbering) format
    pub fn is_bullet(&self) -> bool {
        matches!(self, NumberFormat::Bullet | NumberFormat::None)
    }

    /// Format a number according to this format
    pub fn format(&self, value: u32) -> String {
        match self {
            NumberFormat::Decimal => value.to_string(),
            NumberFormat::DecimalZero => format!("{:02}", value),
            NumberFormat::LowerLetter => format_letter(value, false),
            NumberFormat::UpperLetter => format_letter(value, true),
            NumberFormat::LowerRoman => format_roman(value, false),
            NumberFormat::UpperRoman => format_roman(value, true),
            NumberFormat::Bullet => String::new(), // Bullet character handled separately
            NumberFormat::None => String::new(),
            NumberFormat::Ordinal => format_ordinal(value),
            NumberFormat::CardinalText => format_cardinal_text(value),
            NumberFormat::OrdinalText => format_ordinal_text(value),
        }
    }
}

/// Format a number as a letter (a, b, c, ... z, aa, ab, ...)
fn format_letter(value: u32, uppercase: bool) -> String {
    if value == 0 {
        return String::new();
    }

    let mut result = String::new();
    let mut n = value;

    while n > 0 {
        n -= 1;
        let c = ((n % 26) as u8 + if uppercase { b'A' } else { b'a' }) as char;
        result.insert(0, c);
        n /= 26;
    }

    result
}

/// Format a number as roman numerals
fn format_roman(value: u32, uppercase: bool) -> String {
    if value == 0 || value > 3999 {
        return value.to_string(); // Fallback for out-of-range
    }

    let numerals = [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];

    let mut result = String::new();
    let mut n = value;

    for (num, roman) in numerals {
        while n >= num {
            result.push_str(roman);
            n -= num;
        }
    }

    if uppercase {
        result.to_uppercase()
    } else {
        result
    }
}

/// Format a number as an ordinal (1st, 2nd, 3rd, ...)
fn format_ordinal(value: u32) -> String {
    let suffix = match value % 100 {
        11 | 12 | 13 => "th",
        _ => match value % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    format!("{}{}", value, suffix)
}

/// Format a number as cardinal text (One, Two, Three, ...)
fn format_cardinal_text(value: u32) -> String {
    match value {
        0 => "Zero".to_string(),
        1 => "One".to_string(),
        2 => "Two".to_string(),
        3 => "Three".to_string(),
        4 => "Four".to_string(),
        5 => "Five".to_string(),
        6 => "Six".to_string(),
        7 => "Seven".to_string(),
        8 => "Eight".to_string(),
        9 => "Nine".to_string(),
        10 => "Ten".to_string(),
        11 => "Eleven".to_string(),
        12 => "Twelve".to_string(),
        13 => "Thirteen".to_string(),
        14 => "Fourteen".to_string(),
        15 => "Fifteen".to_string(),
        16 => "Sixteen".to_string(),
        17 => "Seventeen".to_string(),
        18 => "Eighteen".to_string(),
        19 => "Nineteen".to_string(),
        20 => "Twenty".to_string(),
        _ => value.to_string(), // Fallback for larger numbers
    }
}

/// Format a number as ordinal text (First, Second, Third, ...)
fn format_ordinal_text(value: u32) -> String {
    match value {
        1 => "First".to_string(),
        2 => "Second".to_string(),
        3 => "Third".to_string(),
        4 => "Fourth".to_string(),
        5 => "Fifth".to_string(),
        6 => "Sixth".to_string(),
        7 => "Seventh".to_string(),
        8 => "Eighth".to_string(),
        9 => "Ninth".to_string(),
        10 => "Tenth".to_string(),
        _ => format_ordinal(value), // Fallback to numeric ordinal
    }
}

// =============================================================================
// List Level Definition
// =============================================================================

/// Definition for a single level of a list (0-8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListLevel {
    /// Level index (0-8)
    pub level: u8,
    /// Number format for this level
    pub format: NumberFormat,
    /// Text pattern for the number (e.g., "%1." or "%1.%2.")
    /// %1 = level 0 number, %2 = level 1 number, etc.
    pub text: String,
    /// Starting number for this level
    pub start: u32,
    /// Left indent in points for this level
    pub indent: f32,
    /// Hanging indent in points (space for number)
    pub hanging: f32,
    /// Font family for bullet character (if bullet format)
    pub font: Option<String>,
    /// Bullet character (if bullet format)
    pub bullet_char: Option<char>,
    /// Tab stop position after number (None = use hanging)
    pub tab_stop: Option<f32>,
    /// Restart numbering when parent level increments
    pub restart_after_level: Option<u8>,
    /// Text alignment for the number
    pub alignment: ListLevelAlignment,
    /// Suffix after the number
    pub suffix: ListLevelSuffix,
}

/// Alignment of the list number
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ListLevelAlignment {
    #[default]
    Left,
    Center,
    Right,
}

/// Suffix after the list number
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ListLevelSuffix {
    #[default]
    Tab,
    Space,
    Nothing,
}

impl Default for ListLevel {
    fn default() -> Self {
        Self {
            level: 0,
            format: NumberFormat::Decimal,
            text: "%1.".to_string(),
            start: 1,
            indent: 36.0,  // 0.5 inch
            hanging: 18.0, // 0.25 inch
            font: None,
            bullet_char: None,
            tab_stop: None,
            restart_after_level: None,
            alignment: ListLevelAlignment::Left,
            suffix: ListLevelSuffix::Tab,
        }
    }
}

impl ListLevel {
    /// Create a new list level with default settings
    pub fn new(level: u8) -> Self {
        let indent = 36.0 * (level as f32 + 1.0); // 0.5 inch per level
        Self {
            level,
            text: format!("%{}.", level + 1),
            indent,
            ..Default::default()
        }
    }

    /// Create a bullet list level
    pub fn bullet(level: u8, bullet_char: char) -> Self {
        let indent = 36.0 * (level as f32 + 1.0);
        Self {
            level,
            format: NumberFormat::Bullet,
            text: String::new(),
            indent,
            bullet_char: Some(bullet_char),
            font: Some("Symbol".to_string()),
            ..Default::default()
        }
    }

    /// Create a numbered list level
    pub fn numbered(level: u8, format: NumberFormat) -> Self {
        let indent = 36.0 * (level as f32 + 1.0);
        Self {
            level,
            format,
            text: format!("%{}.", level + 1),
            indent,
            ..Default::default()
        }
    }

    /// Format the number text for this level given the current counts
    pub fn format_number(&self, counts: &[u32]) -> String {
        if self.format == NumberFormat::Bullet {
            return self.bullet_char.map(String::from).unwrap_or_default();
        }

        if self.format == NumberFormat::None {
            return String::new();
        }

        let mut result = self.text.clone();

        // Replace %1, %2, etc. with formatted numbers
        for (i, &count) in counts.iter().enumerate() {
            let placeholder = format!("%{}", i + 1);
            if result.contains(&placeholder) {
                let level_format = if i == self.level as usize {
                    self.format
                } else {
                    NumberFormat::Decimal // Parent levels default to decimal
                };
                let formatted = level_format.format(count);
                result = result.replace(&placeholder, &formatted);
            }
        }

        result
    }
}

// =============================================================================
// Abstract Numbering Definition
// =============================================================================

/// Abstract numbering definition - a template for list styles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractNum {
    /// Unique identifier
    pub id: AbstractNumId,
    /// Human-readable name
    pub name: Option<String>,
    /// Levels (0-8)
    pub levels: Vec<ListLevel>,
    /// Multi-level type
    pub multi_level_type: MultiLevelType,
}

/// Type of multi-level list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MultiLevelType {
    /// Single level only
    #[default]
    SingleLevel,
    /// Multiple levels, each restarts independently
    MultiLevel,
    /// Multiple levels that don't restart (e.g., legal style 1.1.1)
    HybridMultiLevel,
}

impl AbstractNum {
    /// Create a new abstract numbering definition
    pub fn new(id: AbstractNumId) -> Self {
        Self {
            id,
            name: None,
            levels: vec![ListLevel::default()],
            multi_level_type: MultiLevelType::SingleLevel,
        }
    }

    /// Create a simple bullet list definition
    pub fn simple_bullet(id: AbstractNumId) -> Self {
        let bullets = ['\u{2022}', '\u{25E6}', '\u{25AA}']; // bullet, white bullet, small square
        let levels: Vec<ListLevel> = (0..9)
            .map(|i| {
                let bullet = bullets[i as usize % bullets.len()];
                ListLevel::bullet(i, bullet)
            })
            .collect();

        Self {
            id,
            name: Some("Simple Bullet".to_string()),
            levels,
            multi_level_type: MultiLevelType::MultiLevel,
        }
    }

    /// Create a simple numbered list definition
    pub fn simple_numbered(id: AbstractNumId) -> Self {
        let levels: Vec<ListLevel> = (0..9)
            .map(|i| {
                let format = match i % 3 {
                    0 => NumberFormat::Decimal,
                    1 => NumberFormat::LowerLetter,
                    _ => NumberFormat::LowerRoman,
                };
                ListLevel::numbered(i, format)
            })
            .collect();

        Self {
            id,
            name: Some("Simple Numbered".to_string()),
            levels,
            multi_level_type: MultiLevelType::MultiLevel,
        }
    }

    /// Create a legal-style numbering (1, 1.1, 1.1.1, etc.)
    pub fn legal_style(id: AbstractNumId) -> Self {
        let levels: Vec<ListLevel> = (0..9)
            .map(|i| {
                let text = (0..=i).map(|j| format!("%{}", j + 1)).collect::<Vec<_>>().join(".");
                let text = format!("{}.", text);
                ListLevel {
                    level: i,
                    format: NumberFormat::Decimal,
                    text,
                    indent: 36.0 * (i as f32 + 1.0),
                    hanging: 36.0,
                    ..Default::default()
                }
            })
            .collect();

        Self {
            id,
            name: Some("Legal Style".to_string()),
            levels,
            multi_level_type: MultiLevelType::HybridMultiLevel,
        }
    }

    /// Get a level definition
    pub fn get_level(&self, level: u8) -> Option<&ListLevel> {
        self.levels.iter().find(|l| l.level == level)
    }

    /// Get a mutable level definition
    pub fn get_level_mut(&mut self, level: u8) -> Option<&mut ListLevel> {
        self.levels.iter_mut().find(|l| l.level == level)
    }
}

// =============================================================================
// Numbering Instance
// =============================================================================

/// Level override for a numbering instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelOverride {
    /// Override the starting number
    pub start_override: Option<u32>,
    /// Override the level definition
    pub level_override: Option<ListLevel>,
}

/// Concrete numbering instance - a use of an abstract numbering definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberingInstance {
    /// Unique identifier
    pub id: NumId,
    /// Reference to abstract numbering definition
    pub abstract_num_id: AbstractNumId,
    /// Per-level overrides
    pub level_overrides: HashMap<u8, LevelOverride>,
}

impl NumberingInstance {
    /// Create a new numbering instance
    pub fn new(id: NumId, abstract_num_id: AbstractNumId) -> Self {
        Self {
            id,
            abstract_num_id,
            level_overrides: HashMap::new(),
        }
    }

    /// Add a start override for a level
    pub fn set_start_override(&mut self, level: u8, start: u32) {
        self.level_overrides
            .entry(level)
            .or_insert_with(|| LevelOverride {
                start_override: None,
                level_override: None,
            })
            .start_override = Some(start);
    }

    /// Get the start override for a level
    pub fn get_start_override(&self, level: u8) -> Option<u32> {
        self.level_overrides
            .get(&level)
            .and_then(|o| o.start_override)
    }
}

// =============================================================================
// List Paragraph Properties
// =============================================================================

/// List properties for a paragraph
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ListProperties {
    /// Numbering instance ID
    pub num_id: Option<NumId>,
    /// Indent level (0-8)
    pub ilvl: Option<u8>,
}

impl ListProperties {
    /// Create new list properties
    pub fn new(num_id: NumId, ilvl: u8) -> Self {
        Self {
            num_id: Some(num_id),
            ilvl: Some(ilvl),
        }
    }

    /// Check if this paragraph is in a list
    pub fn is_in_list(&self) -> bool {
        self.num_id.is_some()
    }

    /// Get the effective indent level (defaults to 0)
    pub fn effective_level(&self) -> u8 {
        self.ilvl.unwrap_or(0)
    }
}

// =============================================================================
// Numbering Registry
// =============================================================================

/// Registry for all list definitions in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberingRegistry {
    /// Abstract numbering definitions
    pub abstract_nums: HashMap<AbstractNumId, AbstractNum>,
    /// Numbering instances
    pub instances: HashMap<NumId, NumberingInstance>,
    /// Counter tracking: (NumId, level) -> current count
    counters: HashMap<(NumId, u8), u32>,
    /// Next available abstract num ID
    next_abstract_id: u32,
    /// Next available num ID
    next_num_id: u32,
}

impl Default for NumberingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NumberingRegistry {
    /// Create a new registry with built-in list definitions
    pub fn new() -> Self {
        let mut registry = Self {
            abstract_nums: HashMap::new(),
            instances: HashMap::new(),
            counters: HashMap::new(),
            next_abstract_id: 0,
            next_num_id: 0,
        };
        registry.register_built_in_lists();
        registry
    }

    /// Register built-in list definitions
    fn register_built_in_lists(&mut self) {
        // Simple bullet list (ID 1)
        let bullet = AbstractNum::simple_bullet(AbstractNumId::new(1));
        self.abstract_nums.insert(bullet.id, bullet);
        let bullet_instance = NumberingInstance::new(NumId::new(1), AbstractNumId::new(1));
        self.instances.insert(bullet_instance.id, bullet_instance);

        // Simple numbered list (ID 2)
        let numbered = AbstractNum::simple_numbered(AbstractNumId::new(2));
        self.abstract_nums.insert(numbered.id, numbered);
        let numbered_instance = NumberingInstance::new(NumId::new(2), AbstractNumId::new(2));
        self.instances.insert(numbered_instance.id, numbered_instance);

        // Legal style list (ID 3)
        let legal = AbstractNum::legal_style(AbstractNumId::new(3));
        self.abstract_nums.insert(legal.id, legal);
        let legal_instance = NumberingInstance::new(NumId::new(3), AbstractNumId::new(3));
        self.instances.insert(legal_instance.id, legal_instance);

        // Update next IDs
        self.next_abstract_id = 4;
        self.next_num_id = 4;
    }

    /// Get the built-in bullet list NumId
    pub fn bullet_list_id() -> NumId {
        NumId::new(1)
    }

    /// Get the built-in numbered list NumId
    pub fn numbered_list_id() -> NumId {
        NumId::new(2)
    }

    /// Get the built-in legal style list NumId
    pub fn legal_list_id() -> NumId {
        NumId::new(3)
    }

    /// Create a new abstract numbering definition
    pub fn create_abstract_num(&mut self, abstract_num: AbstractNum) -> AbstractNumId {
        let id = abstract_num.id;
        self.abstract_nums.insert(id, abstract_num);
        if id.0 >= self.next_abstract_id {
            self.next_abstract_id = id.0 + 1;
        }
        id
    }

    /// Create a new numbering instance
    pub fn create_instance(&mut self, instance: NumberingInstance) -> NumId {
        let id = instance.id;
        self.instances.insert(id, instance);
        if id.0 >= self.next_num_id {
            self.next_num_id = id.0 + 1;
        }
        id
    }

    /// Allocate a new NumId for a new instance
    pub fn next_num_id(&mut self) -> NumId {
        let id = NumId::new(self.next_num_id);
        self.next_num_id += 1;
        id
    }

    /// Allocate a new AbstractNumId
    pub fn next_abstract_num_id(&mut self) -> AbstractNumId {
        let id = AbstractNumId::new(self.next_abstract_id);
        self.next_abstract_id += 1;
        id
    }

    /// Get an abstract numbering definition
    pub fn get_abstract_num(&self, id: AbstractNumId) -> Option<&AbstractNum> {
        self.abstract_nums.get(&id)
    }

    /// Get a numbering instance
    pub fn get_instance(&self, id: NumId) -> Option<&NumberingInstance> {
        self.instances.get(&id)
    }

    /// Get the effective level definition for a numbering instance
    pub fn get_effective_level(&self, num_id: NumId, level: u8) -> Option<ListLevel> {
        let instance = self.instances.get(&num_id)?;
        let abstract_num = self.abstract_nums.get(&instance.abstract_num_id)?;
        let base_level = abstract_num.get_level(level)?;

        // Check for level override
        if let Some(override_info) = instance.level_overrides.get(&level) {
            if let Some(level_override) = &override_info.level_override {
                return Some(level_override.clone());
            }
        }

        Some(base_level.clone())
    }

    /// Get the current counter value for a list/level
    pub fn get_counter(&self, num_id: NumId, level: u8) -> u32 {
        self.counters.get(&(num_id, level)).copied().unwrap_or(0)
    }

    /// Set the counter value for a list/level
    pub fn set_counter(&mut self, num_id: NumId, level: u8, value: u32) {
        self.counters.insert((num_id, level), value);
    }

    /// Increment the counter for a list/level and return the new value
    pub fn increment_counter(&mut self, num_id: NumId, level: u8) -> u32 {
        let entry = self.counters.entry((num_id, level)).or_insert(0);
        *entry += 1;
        *entry
    }

    /// Reset the counter for a list/level
    pub fn reset_counter(&mut self, num_id: NumId, level: u8) {
        let start = self.get_start_value(num_id, level);
        self.counters.insert((num_id, level), start - 1); // -1 because increment happens before use
    }

    /// Reset counters for all levels after the given level
    pub fn reset_counters_after_level(&mut self, num_id: NumId, level: u8) {
        for l in (level + 1)..9 {
            self.reset_counter(num_id, l);
        }
    }

    /// Get the start value for a level
    fn get_start_value(&self, num_id: NumId, level: u8) -> u32 {
        // Check instance override first
        if let Some(instance) = self.instances.get(&num_id) {
            if let Some(start) = instance.get_start_override(level) {
                return start;
            }

            // Check abstract num
            if let Some(abstract_num) = self.abstract_nums.get(&instance.abstract_num_id) {
                if let Some(level_def) = abstract_num.get_level(level) {
                    return level_def.start;
                }
            }
        }

        1 // Default start value
    }

    /// Reset all counters
    pub fn reset_all_counters(&mut self) {
        self.counters.clear();
    }

    /// Format the number for a specific list/level
    pub fn format_number(&self, num_id: NumId, level: u8, counts: &[u32]) -> Option<String> {
        let level_def = self.get_effective_level(num_id, level)?;
        Some(level_def.format_number(counts))
    }

    /// Get all numbering instances
    pub fn all_instances(&self) -> impl Iterator<Item = &NumberingInstance> {
        self.instances.values()
    }

    /// Get all abstract numbering definitions
    pub fn all_abstract_nums(&self) -> impl Iterator<Item = &AbstractNum> {
        self.abstract_nums.values()
    }

    /// Check if a NumId is a bullet list
    pub fn is_bullet_list(&self, num_id: NumId) -> bool {
        if let Some(instance) = self.instances.get(&num_id) {
            if let Some(abstract_num) = self.abstract_nums.get(&instance.abstract_num_id) {
                if let Some(level) = abstract_num.get_level(0) {
                    return level.format == NumberFormat::Bullet;
                }
            }
        }
        false
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_letter() {
        assert_eq!(format_letter(1, false), "a");
        assert_eq!(format_letter(26, false), "z");
        assert_eq!(format_letter(27, false), "aa");
        assert_eq!(format_letter(52, false), "az");
        assert_eq!(format_letter(1, true), "A");
        assert_eq!(format_letter(26, true), "Z");
    }

    #[test]
    fn test_format_roman() {
        assert_eq!(format_roman(1, false), "i");
        assert_eq!(format_roman(4, false), "iv");
        assert_eq!(format_roman(9, false), "ix");
        assert_eq!(format_roman(10, false), "x");
        assert_eq!(format_roman(50, false), "l");
        assert_eq!(format_roman(100, false), "c");
        assert_eq!(format_roman(1999, true), "MCMXCIX");
    }

    #[test]
    fn test_format_ordinal() {
        assert_eq!(format_ordinal(1), "1st");
        assert_eq!(format_ordinal(2), "2nd");
        assert_eq!(format_ordinal(3), "3rd");
        assert_eq!(format_ordinal(4), "4th");
        assert_eq!(format_ordinal(11), "11th");
        assert_eq!(format_ordinal(12), "12th");
        assert_eq!(format_ordinal(13), "13th");
        assert_eq!(format_ordinal(21), "21st");
    }

    #[test]
    fn test_list_level_format() {
        let level = ListLevel {
            level: 0,
            format: NumberFormat::Decimal,
            text: "%1.".to_string(),
            ..Default::default()
        };

        assert_eq!(level.format_number(&[1]), "1.");
        assert_eq!(level.format_number(&[10]), "10.");
    }

    #[test]
    fn test_multi_level_format() {
        let level = ListLevel {
            level: 2,
            format: NumberFormat::Decimal,
            text: "%1.%2.%3.".to_string(),
            ..Default::default()
        };

        assert_eq!(level.format_number(&[1, 2, 3]), "1.2.3.");
        assert_eq!(level.format_number(&[1, 1, 1]), "1.1.1.");
    }

    #[test]
    fn test_bullet_level() {
        let level = ListLevel::bullet(0, '\u{2022}');
        assert!(level.format.is_bullet());
        assert_eq!(level.format_number(&[1]), "\u{2022}");
    }

    #[test]
    fn test_registry_built_in_lists() {
        let registry = NumberingRegistry::new();

        // Check bullet list exists
        let bullet_instance = registry.get_instance(NumberingRegistry::bullet_list_id());
        assert!(bullet_instance.is_some());

        // Check numbered list exists
        let numbered_instance = registry.get_instance(NumberingRegistry::numbered_list_id());
        assert!(numbered_instance.is_some());

        // Check legal style exists
        let legal_instance = registry.get_instance(NumberingRegistry::legal_list_id());
        assert!(legal_instance.is_some());
    }

    #[test]
    fn test_counter_tracking() {
        let mut registry = NumberingRegistry::new();
        let num_id = NumberingRegistry::numbered_list_id();

        // Increment counter
        assert_eq!(registry.increment_counter(num_id, 0), 1);
        assert_eq!(registry.increment_counter(num_id, 0), 2);
        assert_eq!(registry.increment_counter(num_id, 0), 3);

        // Reset counter
        registry.reset_counter(num_id, 0);
        assert_eq!(registry.increment_counter(num_id, 0), 1);
    }

    #[test]
    fn test_is_bullet_list() {
        let registry = NumberingRegistry::new();
        assert!(registry.is_bullet_list(NumberingRegistry::bullet_list_id()));
        assert!(!registry.is_bullet_list(NumberingRegistry::numbered_list_id()));
    }
}
