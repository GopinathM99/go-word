//! ZIP archive reading and XML parsing utilities

use crate::docx::error::{DocxError, DocxResult};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::{Read, Seek};
use zip::ZipArchive;

/// A wrapper around a ZIP archive for reading DOCX files
pub struct DocxReader<R: Read + Seek> {
    archive: ZipArchive<R>,
}

impl<R: Read + Seek> DocxReader<R> {
    /// Create a new DOCX reader from a source that implements Read + Seek
    pub fn new(reader: R) -> DocxResult<Self> {
        let archive = ZipArchive::new(reader)?;
        Ok(Self { archive })
    }

    /// Read a file from the archive as a string
    pub fn read_file_as_string(&mut self, path: &str) -> DocxResult<String> {
        let mut file = self.archive.by_name(path).map_err(|e| {
            if matches!(e, zip::result::ZipError::FileNotFound) {
                DocxError::MissingPart(path.to_string())
            } else {
                DocxError::from(e)
            }
        })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    /// Read a file from the archive as bytes
    pub fn read_file_as_bytes(&mut self, path: &str) -> DocxResult<Vec<u8>> {
        let mut file = self.archive.by_name(path).map_err(|e| {
            if matches!(e, zip::result::ZipError::FileNotFound) {
                DocxError::MissingPart(path.to_string())
            } else {
                DocxError::from(e)
            }
        })?;

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        Ok(contents)
    }

    /// Check if a file exists in the archive
    pub fn file_exists(&self, path: &str) -> bool {
        self.archive.file_names().any(|name| name == path)
    }

    /// Get a list of all files in the archive
    pub fn file_names(&self) -> Vec<&str> {
        self.archive.file_names().collect()
    }

    /// Check if this is a valid DOCX file
    pub fn is_valid_docx(&self) -> bool {
        // Must have [Content_Types].xml and word/document.xml
        self.file_exists("[Content_Types].xml") &&
        self.file_exists("word/document.xml")
    }
}

/// XML reader utilities for parsing DOCX XML content
pub struct XmlParser;

impl XmlParser {
    /// Create a new XML reader from a string
    pub fn from_string(content: &str) -> Reader<&[u8]> {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);
        reader
    }

    /// Get an attribute value from an event
    pub fn get_attribute(event: &quick_xml::events::BytesStart, name: &[u8]) -> Option<String> {
        event.attributes()
            .filter_map(|a| a.ok())
            .find(|a| a.key.as_ref() == name)
            .map(|a| String::from_utf8_lossy(&a.value).to_string())
    }

    /// Get an attribute value with a namespace prefix
    pub fn get_prefixed_attribute(
        event: &quick_xml::events::BytesStart,
        prefix: &str,
        local: &str
    ) -> Option<String> {
        let key = format!("{}:{}", prefix, local);
        Self::get_attribute(event, key.as_bytes())
    }

    /// Get a w: namespaced attribute (most common in DOCX)
    pub fn get_w_attribute(event: &quick_xml::events::BytesStart, name: &str) -> Option<String> {
        Self::get_prefixed_attribute(event, "w", name)
            .or_else(|| Self::get_attribute(event, name.as_bytes()))
    }

    /// Get a r: namespaced attribute
    pub fn get_r_attribute(event: &quick_xml::events::BytesStart, name: &str) -> Option<String> {
        Self::get_prefixed_attribute(event, "r", name)
    }

    /// Parse a dimension value (twips to points conversion)
    /// DOCX uses twips (1/20 of a point) for many measurements
    pub fn parse_twips(value: &str) -> Option<f32> {
        value.parse::<f32>().ok().map(|v| v / 20.0)
    }

    /// Parse a half-point value to points
    /// DOCX uses half-points for font sizes
    pub fn parse_half_points(value: &str) -> Option<f32> {
        value.parse::<f32>().ok().map(|v| v / 2.0)
    }

    /// Parse a boolean value (0/1, true/false, on/off)
    pub fn parse_bool(value: &str) -> bool {
        matches!(value.to_lowercase().as_str(), "1" | "true" | "on" | "yes")
    }

    /// Parse a percentage value (0-100 to 0.0-1.0)
    pub fn parse_percentage(value: &str) -> Option<f32> {
        value.parse::<f32>().ok().map(|v| v / 100.0)
    }

    /// Parse EMUs (English Metric Units) to points
    /// 1 inch = 914400 EMUs, 1 point = 12700 EMUs
    pub fn parse_emu(value: &str) -> Option<f32> {
        value.parse::<f64>().ok().map(|v| (v / 12700.0) as f32)
    }

    /// Check if an element name matches with optional namespace prefix
    pub fn matches_element(name: &[u8], expected: &str) -> bool {
        let name_str = std::str::from_utf8(name).unwrap_or("");
        name_str == expected || name_str.ends_with(&format!(":{}", expected))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_twips() {
        assert_eq!(XmlParser::parse_twips("1440"), Some(72.0)); // 1 inch
        assert_eq!(XmlParser::parse_twips("720"), Some(36.0)); // 0.5 inch
    }

    #[test]
    fn test_parse_half_points() {
        assert_eq!(XmlParser::parse_half_points("24"), Some(12.0)); // 12pt
        assert_eq!(XmlParser::parse_half_points("22"), Some(11.0)); // 11pt
    }

    #[test]
    fn test_parse_bool() {
        assert!(XmlParser::parse_bool("1"));
        assert!(XmlParser::parse_bool("true"));
        assert!(XmlParser::parse_bool("on"));
        assert!(!XmlParser::parse_bool("0"));
        assert!(!XmlParser::parse_bool("false"));
    }

    #[test]
    fn test_parse_emu() {
        // 914400 EMUs = 72 points (1 inch)
        assert!((XmlParser::parse_emu("914400").unwrap() - 72.0).abs() < 0.1);
    }

    #[test]
    fn test_matches_element() {
        assert!(XmlParser::matches_element(b"p", "p"));
        assert!(XmlParser::matches_element(b"w:p", "p"));
        assert!(!XmlParser::matches_element(b"w:r", "p"));
    }
}
