//! Text shaping using rustybuzz
//!
//! This module provides text shaping capabilities using the rustybuzz library,
//! which is a pure Rust implementation of HarfBuzz. Text shaping is the process
//! of converting a sequence of Unicode codepoints into properly positioned glyphs.

use crate::{FontId, FontMetrics, FontStyle, FontWeight, Result, TextError};
use std::collections::HashMap;
use std::sync::Arc;

/// A shaped glyph with position information
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    /// Glyph ID in the font
    pub glyph_id: u16,
    /// Horizontal advance in font units (scaled to font size)
    pub x_advance: i32,
    /// Vertical advance in font units
    pub y_advance: i32,
    /// Horizontal offset from the origin
    pub x_offset: i32,
    /// Vertical offset from the origin
    pub y_offset: i32,
    /// Cluster index (byte offset in the original text for cursor mapping)
    pub cluster: u32,
    /// Character this glyph represents (for fallback when no font)
    pub character: Option<char>,
}

impl ShapedGlyph {
    /// Get the advance width scaled to a specific font size
    pub fn advance_width(&self, font_size: f32, units_per_em: u16) -> f32 {
        self.x_advance as f32 * font_size / units_per_em as f32
    }
}

/// A run of shaped glyphs with associated metrics
#[derive(Debug, Clone)]
pub struct ShapedRun {
    /// The shaped glyphs
    pub glyphs: Vec<ShapedGlyph>,
    /// Total width of the run in points
    pub width: f32,
    /// Font size used for shaping
    pub font_size: f32,
    /// Units per em of the font used
    pub units_per_em: u16,
    /// Ascender height scaled to font size
    pub ascender: f32,
    /// Descender depth scaled to font size (positive value)
    pub descender: f32,
    /// Line gap scaled to font size
    pub line_gap: f32,
}

impl ShapedRun {
    /// Calculate the total line height for this run
    pub fn line_height(&self) -> f32 {
        self.ascender + self.descender + self.line_gap
    }

    /// Get glyph at a specific byte offset
    pub fn glyph_at_offset(&self, byte_offset: usize) -> Option<&ShapedGlyph> {
        self.glyphs.iter().find(|g| g.cluster as usize == byte_offset)
    }

    /// Get the x position at a specific byte offset
    pub fn x_at_offset(&self, byte_offset: usize) -> f32 {
        let mut x = 0.0;
        for glyph in &self.glyphs {
            if glyph.cluster as usize >= byte_offset {
                break;
            }
            x += glyph.advance_width(self.font_size, self.units_per_em);
        }
        x
    }
}

/// Cached font face for shaping
struct CachedFace {
    /// The font data (kept alive for rustybuzz)
    #[allow(dead_code)]
    data: Arc<Vec<u8>>,
    /// The rustybuzz face
    face: rustybuzz::Face<'static>,
}

/// Text shaper using rustybuzz
///
/// The TextShaper handles text shaping operations, converting Unicode text
/// into positioned glyphs. It maintains a cache of font faces for efficiency.
pub struct TextShaper {
    /// Cache of loaded font faces
    face_cache: HashMap<FontId, CachedFace>,
    /// Default metrics when no font is available
    default_metrics: FontMetrics,
}

impl TextShaper {
    /// Create a new text shaper
    pub fn new() -> Self {
        Self {
            face_cache: HashMap::new(),
            default_metrics: FontMetrics::default(),
        }
    }

    /// Load a font from data
    pub fn load_font(&mut self, font_id: FontId, data: Vec<u8>) -> Result<()> {
        let data = Arc::new(data);
        // SAFETY: We keep the Arc alive in CachedFace, so the data lives as long as the face
        let static_data: &'static [u8] = unsafe {
            std::mem::transmute::<&[u8], &'static [u8]>(data.as_slice())
        };

        let face = rustybuzz::Face::from_slice(static_data, 0)
            .ok_or_else(|| TextError::InvalidFontData("Failed to parse font".into()))?;

        self.face_cache.insert(font_id, CachedFace { data, face });
        Ok(())
    }

    /// Check if a font is loaded
    pub fn has_font(&self, font_id: &FontId) -> bool {
        self.face_cache.contains_key(font_id)
    }

    /// Shape a text string with optional font
    pub fn shape(&self, text: &str, font_size: f32) -> Result<ShapedRun> {
        self.shape_with_font(text, font_size, None)
    }

    /// Shape a text string with a specific font
    pub fn shape_with_font(
        &self,
        text: &str,
        font_size: f32,
        font_id: Option<&FontId>,
    ) -> Result<ShapedRun> {
        // Try to get the font face
        let cached = font_id.and_then(|id| self.face_cache.get(id));

        match cached {
            Some(cached_face) => self.shape_with_face(text, font_size, &cached_face.face),
            None => self.shape_fallback(text, font_size),
        }
    }

    /// Shape text using a rustybuzz face
    fn shape_with_face(
        &self,
        text: &str,
        font_size: f32,
        face: &rustybuzz::Face<'_>,
    ) -> Result<ShapedRun> {
        let units_per_em = face.units_per_em() as u16;
        let scale = font_size / units_per_em as f32;

        // Create a buffer and add the text
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(text);

        // Shape the text
        let output = rustybuzz::shape(face, &[], buffer);

        // Extract glyph information
        let glyph_infos = output.glyph_infos();
        let glyph_positions = output.glyph_positions();

        let mut glyphs = Vec::with_capacity(glyph_infos.len());
        let mut total_advance = 0i32;

        for (info, pos) in glyph_infos.iter().zip(glyph_positions.iter()) {
            glyphs.push(ShapedGlyph {
                glyph_id: info.glyph_id as u16,
                x_advance: pos.x_advance,
                y_advance: pos.y_advance,
                x_offset: pos.x_offset,
                y_offset: pos.y_offset,
                cluster: info.cluster,
                character: None,
            });
            total_advance += pos.x_advance;
        }

        // Get font metrics
        let ascender = face.ascender() as f32 * scale;
        let descender = face.descender().abs() as f32 * scale;
        let line_gap = face.line_gap() as f32 * scale;

        Ok(ShapedRun {
            glyphs,
            width: total_advance as f32 * scale,
            font_size,
            units_per_em,
            ascender,
            descender,
            line_gap,
        })
    }

    /// Fallback shaping when no font is available
    /// Uses character-width estimation based on Unicode properties
    fn shape_fallback(&self, text: &str, font_size: f32) -> Result<ShapedRun> {
        let units_per_em = self.default_metrics.units_per_em;
        let scale = font_size / units_per_em as f32;

        // Estimate advance widths based on character properties
        let mut glyphs = Vec::new();
        let mut total_advance = 0i32;
        let mut byte_offset = 0u32;

        for ch in text.chars() {
            let advance = self.estimate_char_width(ch, units_per_em);
            glyphs.push(ShapedGlyph {
                glyph_id: ch as u16, // Use codepoint as pseudo glyph ID
                x_advance: advance,
                y_advance: 0,
                x_offset: 0,
                y_offset: 0,
                cluster: byte_offset,
                character: Some(ch),
            });
            total_advance += advance;
            byte_offset += ch.len_utf8() as u32;
        }

        let ascender = self.default_metrics.ascender as f32 * scale;
        let descender = self.default_metrics.descender.abs() as f32 * scale;
        let line_gap = self.default_metrics.line_gap as f32 * scale;

        Ok(ShapedRun {
            glyphs,
            width: total_advance as f32 * scale,
            font_size,
            units_per_em,
            ascender,
            descender,
            line_gap,
        })
    }

    /// Estimate character width based on Unicode properties
    fn estimate_char_width(&self, ch: char, units_per_em: u16) -> i32 {
        let em = units_per_em as i32;

        match ch {
            // Narrow characters
            ' ' | 'i' | 'l' | 'j' | 't' | 'f' | 'r' | '!' | '|' | '\'' | '`' | '.' | ',' | ':' | ';' => {
                em * 30 / 100
            }
            // Very narrow
            'I' | '1' => em * 35 / 100,
            // Wide characters
            'm' | 'w' | 'M' | 'W' | '@' | '%' => em * 90 / 100,
            // Uppercase typically wider
            'A'..='Z' => em * 70 / 100,
            // Lowercase average
            'a'..='z' => em * 55 / 100,
            // Digits are typically monospaced
            '0'..='9' => em * 60 / 100,
            // CJK characters are full-width
            '\u{4E00}'..='\u{9FFF}' | '\u{3000}'..='\u{303F}' => em,
            // Zero-width characters
            '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' => 0,
            // Soft hyphen is zero-width normally
            '\u{00AD}' => 0,
            // No-break space same as regular space
            '\u{00A0}' => em * 30 / 100,
            // Default width
            _ => em * 60 / 100,
        }
    }

    /// Shape text for a specific run style
    pub fn shape_run(
        &self,
        text: &str,
        font_family: Option<&str>,
        font_size: f32,
        bold: bool,
        italic: bool,
    ) -> Result<ShapedRun> {
        // Try to find a matching font
        let font_id = font_family.map(|family| {
            FontId::new(family)
                .with_weight(if bold { FontWeight::Bold } else { FontWeight::Normal })
                .with_style(if italic { FontStyle::Italic } else { FontStyle::Normal })
        });

        self.shape_with_font(text, font_size, font_id.as_ref())
    }
}

impl Default for TextShaper {
    fn default() -> Self {
        Self::new()
    }
}
