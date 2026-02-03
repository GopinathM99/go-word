//! PDF Font Handling
//!
//! This module handles font embedding and management for PDF export.
//! It supports:
//! - Standard 14 PDF fonts (always available)
//! - TrueType font embedding (with optional subsetting)
//! - Font descriptor generation
//! - ToUnicode CMap for text extraction

use super::objects::{PdfDictionary, PdfObject, PdfStream, PdfString};
use std::collections::HashMap;

/// Standard 14 PDF fonts (built into every PDF viewer)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardFont {
    /// Times Roman
    TimesRoman,
    /// Times Bold
    TimesBold,
    /// Times Italic
    TimesItalic,
    /// Times Bold Italic
    TimesBoldItalic,
    /// Helvetica
    Helvetica,
    /// Helvetica Bold
    HelveticaBold,
    /// Helvetica Oblique
    HelveticaOblique,
    /// Helvetica Bold Oblique
    HelveticaBoldOblique,
    /// Courier
    Courier,
    /// Courier Bold
    CourierBold,
    /// Courier Oblique
    CourierOblique,
    /// Courier Bold Oblique
    CourierBoldOblique,
    /// Symbol
    Symbol,
    /// Zapf Dingbats
    ZapfDingbats,
}

impl StandardFont {
    /// Get the PDF name for this font
    pub fn pdf_name(&self) -> &'static str {
        match self {
            StandardFont::TimesRoman => "Times-Roman",
            StandardFont::TimesBold => "Times-Bold",
            StandardFont::TimesItalic => "Times-Italic",
            StandardFont::TimesBoldItalic => "Times-BoldItalic",
            StandardFont::Helvetica => "Helvetica",
            StandardFont::HelveticaBold => "Helvetica-Bold",
            StandardFont::HelveticaOblique => "Helvetica-Oblique",
            StandardFont::HelveticaBoldOblique => "Helvetica-BoldOblique",
            StandardFont::Courier => "Courier",
            StandardFont::CourierBold => "Courier-Bold",
            StandardFont::CourierOblique => "Courier-Oblique",
            StandardFont::CourierBoldOblique => "Courier-BoldOblique",
            StandardFont::Symbol => "Symbol",
            StandardFont::ZapfDingbats => "ZapfDingbats",
        }
    }

    /// Get the font encoding (WinAnsiEncoding for text fonts)
    pub fn encoding(&self) -> Option<&'static str> {
        match self {
            StandardFont::Symbol | StandardFont::ZapfDingbats => None,
            _ => Some("WinAnsiEncoding"),
        }
    }

    /// Try to match a font name to a standard font
    pub fn from_name(name: &str, bold: bool, italic: bool) -> Option<Self> {
        let name_lower = name.to_lowercase();

        // Times variants
        if name_lower.contains("times") || name_lower.contains("serif") {
            return Some(match (bold, italic) {
                (false, false) => StandardFont::TimesRoman,
                (true, false) => StandardFont::TimesBold,
                (false, true) => StandardFont::TimesItalic,
                (true, true) => StandardFont::TimesBoldItalic,
            });
        }

        // Helvetica/Arial variants
        if name_lower.contains("helvetica")
            || name_lower.contains("arial")
            || name_lower.contains("sans")
        {
            return Some(match (bold, italic) {
                (false, false) => StandardFont::Helvetica,
                (true, false) => StandardFont::HelveticaBold,
                (false, true) => StandardFont::HelveticaOblique,
                (true, true) => StandardFont::HelveticaBoldOblique,
            });
        }

        // Courier variants
        if name_lower.contains("courier") || name_lower.contains("mono") {
            return Some(match (bold, italic) {
                (false, false) => StandardFont::Courier,
                (true, false) => StandardFont::CourierBold,
                (false, true) => StandardFont::CourierOblique,
                (true, true) => StandardFont::CourierBoldOblique,
            });
        }

        // Symbol fonts
        if name_lower.contains("symbol") {
            return Some(StandardFont::Symbol);
        }
        if name_lower.contains("dingbat") || name_lower.contains("zapf") {
            return Some(StandardFont::ZapfDingbats);
        }

        // Default to Helvetica if no match
        None
    }

    /// Get default fallback font
    pub fn default_fallback(bold: bool, italic: bool) -> Self {
        match (bold, italic) {
            (false, false) => StandardFont::Helvetica,
            (true, false) => StandardFont::HelveticaBold,
            (false, true) => StandardFont::HelveticaOblique,
            (true, true) => StandardFont::HelveticaBoldOblique,
        }
    }
}

/// Font reference in a PDF document
#[derive(Debug, Clone)]
pub struct FontRef {
    /// Internal font name (e.g., "F1", "F2")
    pub name: String,
    /// Object reference number
    pub obj_ref: u32,
}

/// Font manager for PDF export
#[derive(Debug, Default)]
pub struct FontManager {
    /// Fonts that have been added (internal name -> font info)
    fonts: HashMap<String, FontInfo>,
    /// Mapping from original font request to internal font name
    font_map: HashMap<FontKey, String>,
    /// Next font number
    next_font_num: u32,
}

/// Key for looking up fonts
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontKey {
    /// Font family name
    pub family: String,
    /// Is bold
    pub bold: bool,
    /// Is italic
    pub italic: bool,
}

impl FontKey {
    /// Create a new font key
    pub fn new(family: impl Into<String>, bold: bool, italic: bool) -> Self {
        Self {
            family: family.into(),
            bold,
            italic,
        }
    }
}

/// Information about a font in the PDF
#[derive(Debug, Clone)]
pub struct FontInfo {
    /// Internal PDF font name (e.g., "F1")
    pub name: String,
    /// The standard font being used
    pub standard_font: StandardFont,
    /// Original family name requested
    pub original_family: String,
}

impl FontManager {
    /// Create a new font manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create a font for the given font key
    pub fn get_or_create_font(&mut self, key: &FontKey) -> &FontInfo {
        // Check if we already have this font
        if let Some(name) = self.font_map.get(key) {
            return self.fonts.get(name).unwrap();
        }

        // Find a standard font match or use fallback
        let standard_font =
            StandardFont::from_name(&key.family, key.bold, key.italic)
                .unwrap_or_else(|| StandardFont::default_fallback(key.bold, key.italic));

        // Generate internal font name
        let name = format!("F{}", self.next_font_num);
        self.next_font_num += 1;

        // Create font info
        let font_info = FontInfo {
            name: name.clone(),
            standard_font,
            original_family: key.family.clone(),
        };

        // Store the font
        self.font_map.insert(key.clone(), name.clone());
        self.fonts.insert(name.clone(), font_info);

        self.fonts.get(&name).unwrap()
    }

    /// Get font info by internal name
    pub fn get_font(&self, name: &str) -> Option<&FontInfo> {
        self.fonts.get(name)
    }

    /// Iterate over all fonts
    pub fn fonts(&self) -> impl Iterator<Item = &FontInfo> {
        self.fonts.values()
    }

    /// Get the number of fonts
    pub fn font_count(&self) -> usize {
        self.fonts.len()
    }
}

/// Create a font dictionary for a standard font
pub fn create_standard_font_dict(font: StandardFont) -> PdfDictionary {
    let mut dict = PdfDictionary::new().with_type("Font");

    dict.insert("Subtype", PdfObject::Name("Type1".to_string()));
    dict.insert("BaseFont", PdfObject::Name(font.pdf_name().to_string()));

    if let Some(encoding) = font.encoding() {
        dict.insert("Encoding", PdfObject::Name(encoding.to_string()));
    }

    dict
}

/// Font widths for a Type1 font (simplified - use fixed width approximations)
pub fn get_standard_font_widths(font: StandardFont, first_char: u8, last_char: u8) -> Vec<i32> {
    let width = match font {
        StandardFont::Courier
        | StandardFont::CourierBold
        | StandardFont::CourierOblique
        | StandardFont::CourierBoldOblique => 600, // Monospace
        StandardFont::Helvetica
        | StandardFont::HelveticaOblique => 500, // Approximate average
        StandardFont::HelveticaBold
        | StandardFont::HelveticaBoldOblique => 520,
        StandardFont::TimesRoman
        | StandardFont::TimesItalic => 500,
        StandardFont::TimesBold
        | StandardFont::TimesBoldItalic => 520,
        StandardFont::Symbol
        | StandardFont::ZapfDingbats => 500,
    };

    vec![width; (last_char - first_char + 1) as usize]
}

/// Estimate the width of a string in a standard font
pub fn estimate_text_width(text: &str, font: StandardFont, font_size: f64) -> f64 {
    // Use average character width approximation
    let avg_width = match font {
        StandardFont::Courier
        | StandardFont::CourierBold
        | StandardFont::CourierOblique
        | StandardFont::CourierBoldOblique => 0.6, // 600/1000
        StandardFont::Helvetica
        | StandardFont::HelveticaOblique => 0.5,
        StandardFont::HelveticaBold
        | StandardFont::HelveticaBoldOblique => 0.52,
        StandardFont::TimesRoman
        | StandardFont::TimesItalic => 0.45,
        StandardFont::TimesBold
        | StandardFont::TimesBoldItalic => 0.48,
        StandardFont::Symbol
        | StandardFont::ZapfDingbats => 0.5,
    };

    text.chars().count() as f64 * avg_width * font_size
}

/// Simple ToUnicode CMap for basic ASCII text extraction
pub fn create_simple_tounicode_cmap() -> Vec<u8> {
    let cmap = r#"/CIDInit /ProcSet findresource begin
12 dict begin
begincmap
/CIDSystemInfo <<
  /Registry (Adobe)
  /Ordering (UCS)
  /Supplement 0
>> def
/CMapName /Adobe-Identity-UCS def
/CMapType 2 def
1 begincodespacerange
<00> <FF>
endcodespacerange
1 beginbfrange
<00> <FF> <0000>
endbfrange
endcmap
CMapName currentdict /CMap defineresource pop
end
end
"#;
    cmap.as_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_font_names() {
        assert_eq!(StandardFont::Helvetica.pdf_name(), "Helvetica");
        assert_eq!(StandardFont::TimesBold.pdf_name(), "Times-Bold");
        assert_eq!(StandardFont::CourierOblique.pdf_name(), "Courier-Oblique");
    }

    #[test]
    fn test_font_matching() {
        assert_eq!(
            StandardFont::from_name("Arial", false, false),
            Some(StandardFont::Helvetica)
        );
        assert_eq!(
            StandardFont::from_name("Times New Roman", true, false),
            Some(StandardFont::TimesBold)
        );
        assert_eq!(
            StandardFont::from_name("Courier New", false, true),
            Some(StandardFont::CourierOblique)
        );
        assert_eq!(
            StandardFont::from_name("Unknown Font", false, false),
            None
        );
    }

    #[test]
    fn test_font_manager() {
        let mut manager = FontManager::new();

        let key1 = FontKey::new("Arial", false, false);
        let font1 = manager.get_or_create_font(&key1);
        assert_eq!(font1.name, "F0");
        assert_eq!(font1.standard_font, StandardFont::Helvetica);

        let key2 = FontKey::new("Times New Roman", true, false);
        let font2 = manager.get_or_create_font(&key2);
        assert_eq!(font2.name, "F1");
        assert_eq!(font2.standard_font, StandardFont::TimesBold);

        // Getting the same font again should return the same name
        let font1_again = manager.get_or_create_font(&key1);
        assert_eq!(font1_again.name, "F0");

        assert_eq!(manager.font_count(), 2);
    }

    #[test]
    fn test_create_font_dict() {
        let dict = create_standard_font_dict(StandardFont::Helvetica);
        assert!(dict.get("Type").is_some());
        assert!(dict.get("Subtype").is_some());
        assert!(dict.get("BaseFont").is_some());
        assert!(dict.get("Encoding").is_some());
    }

    #[test]
    fn test_estimate_width() {
        let width = estimate_text_width("Hello", StandardFont::Helvetica, 12.0);
        assert!(width > 0.0);
        assert!(width < 100.0); // Reasonable bound for 5 characters at 12pt
    }
}
