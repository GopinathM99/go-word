//! PDF Object Model
//!
//! This module defines the core PDF object types as specified in the PDF Reference.
//! PDF uses a small set of basic object types that can be combined to represent
//! all document data.

use std::collections::BTreeMap;
use std::io::{self, Write};

/// PDF object types
#[derive(Debug, Clone)]
pub enum PdfObject {
    /// Null object
    Null,
    /// Boolean value
    Boolean(bool),
    /// Integer number
    Integer(i64),
    /// Real (floating-point) number
    Real(f64),
    /// String (literal or hexadecimal)
    String(PdfString),
    /// Name object (starts with /)
    Name(String),
    /// Array of objects
    Array(Vec<PdfObject>),
    /// Dictionary (key-value pairs)
    Dictionary(PdfDictionary),
    /// Stream (dictionary + byte data)
    Stream(PdfStream),
    /// Indirect reference (object number, generation number)
    Reference(u32, u16),
}

/// PDF string encoding
#[derive(Debug, Clone)]
pub enum PdfString {
    /// Literal string enclosed in parentheses
    Literal(Vec<u8>),
    /// Hexadecimal string enclosed in angle brackets
    Hex(Vec<u8>),
}

impl PdfString {
    /// Create a literal string from bytes
    pub fn literal(data: impl Into<Vec<u8>>) -> Self {
        PdfString::Literal(data.into())
    }

    /// Create a literal string from a str
    pub fn from_str(s: &str) -> Self {
        PdfString::Literal(s.as_bytes().to_vec())
    }

    /// Create a hex string from bytes
    pub fn hex(data: impl Into<Vec<u8>>) -> Self {
        PdfString::Hex(data.into())
    }
}

/// PDF dictionary (ordered key-value pairs)
#[derive(Debug, Clone, Default)]
pub struct PdfDictionary {
    entries: BTreeMap<String, PdfObject>,
}

impl PdfDictionary {
    /// Create an empty dictionary
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a key-value pair
    pub fn insert(&mut self, key: impl Into<String>, value: PdfObject) {
        self.entries.insert(key.into(), value);
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<&PdfObject> {
        self.entries.get(key)
    }

    /// Check if dictionary contains a key
    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if dictionary is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over entries
    pub fn iter(&self) -> impl Iterator<Item = (&String, &PdfObject)> {
        self.entries.iter()
    }

    /// Set the Type entry (common for PDF objects)
    pub fn with_type(mut self, type_name: &str) -> Self {
        self.insert("Type", PdfObject::Name(type_name.to_string()));
        self
    }
}

/// PDF stream (dictionary + data)
#[derive(Debug, Clone)]
pub struct PdfStream {
    /// Stream dictionary
    pub dict: PdfDictionary,
    /// Stream data (uncompressed or compressed)
    pub data: Vec<u8>,
    /// Whether the data is already compressed
    pub compressed: bool,
}

impl PdfStream {
    /// Create a new stream with data
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            dict: PdfDictionary::new(),
            data,
            compressed: false,
        }
    }

    /// Create a stream with a dictionary
    pub fn with_dict(mut self, dict: PdfDictionary) -> Self {
        // Merge entries
        for (key, value) in dict.entries {
            self.dict.insert(key, value);
        }
        self
    }

    /// Mark this stream as compressed
    pub fn mark_compressed(mut self) -> Self {
        self.compressed = true;
        self
    }

    /// Get the stream length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if stream is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Serializer for PDF objects
pub struct PdfSerializer<W: Write> {
    writer: W,
}

impl<W: Write> PdfSerializer<W> {
    /// Create a new serializer
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Write a PDF object
    pub fn write_object(&mut self, obj: &PdfObject) -> io::Result<()> {
        match obj {
            PdfObject::Null => write!(self.writer, "null"),
            PdfObject::Boolean(b) => {
                write!(self.writer, "{}", if *b { "true" } else { "false" })
            }
            PdfObject::Integer(n) => write!(self.writer, "{}", n),
            PdfObject::Real(n) => {
                // Format real numbers with appropriate precision
                if n.fract() == 0.0 {
                    write!(self.writer, "{:.1}", n)
                } else {
                    // Remove trailing zeros
                    let s = format!("{:.6}", n);
                    let s = s.trim_end_matches('0');
                    let s = s.trim_end_matches('.');
                    write!(self.writer, "{}", s)
                }
            }
            PdfObject::String(s) => self.write_string(s),
            PdfObject::Name(name) => self.write_name(name),
            PdfObject::Array(arr) => self.write_array(arr),
            PdfObject::Dictionary(dict) => self.write_dictionary(dict),
            PdfObject::Stream(stream) => self.write_stream(stream),
            PdfObject::Reference(obj_num, gen_num) => {
                write!(self.writer, "{} {} R", obj_num, gen_num)
            }
        }
    }

    /// Write a PDF string
    fn write_string(&mut self, s: &PdfString) -> io::Result<()> {
        match s {
            PdfString::Literal(data) => {
                write!(self.writer, "(")?;
                for &byte in data {
                    match byte {
                        b'(' | b')' | b'\\' => {
                            write!(self.writer, "\\{}", byte as char)?;
                        }
                        0x0A => write!(self.writer, "\\n")?,
                        0x0D => write!(self.writer, "\\r")?,
                        0x09 => write!(self.writer, "\\t")?,
                        0x08 => write!(self.writer, "\\b")?,
                        0x0C => write!(self.writer, "\\f")?,
                        0x20..=0x7E => write!(self.writer, "{}", byte as char)?,
                        _ => write!(self.writer, "\\{:03o}", byte)?,
                    }
                }
                write!(self.writer, ")")
            }
            PdfString::Hex(data) => {
                write!(self.writer, "<")?;
                for byte in data {
                    write!(self.writer, "{:02X}", byte)?;
                }
                write!(self.writer, ">")
            }
        }
    }

    /// Write a PDF name
    fn write_name(&mut self, name: &str) -> io::Result<()> {
        write!(self.writer, "/")?;
        for byte in name.bytes() {
            match byte {
                // Regular characters
                0x21..=0x7E
                    if byte != b'#'
                        && byte != b'('
                        && byte != b')'
                        && byte != b'<'
                        && byte != b'>'
                        && byte != b'['
                        && byte != b']'
                        && byte != b'{'
                        && byte != b'}'
                        && byte != b'/'
                        && byte != b'%' =>
                {
                    write!(self.writer, "{}", byte as char)?;
                }
                // Escape special characters
                _ => write!(self.writer, "#{:02X}", byte)?,
            }
        }
        Ok(())
    }

    /// Write a PDF array
    fn write_array(&mut self, arr: &[PdfObject]) -> io::Result<()> {
        write!(self.writer, "[")?;
        for (i, obj) in arr.iter().enumerate() {
            if i > 0 {
                write!(self.writer, " ")?;
            }
            self.write_object(obj)?;
        }
        write!(self.writer, "]")
    }

    /// Write a PDF dictionary
    fn write_dictionary(&mut self, dict: &PdfDictionary) -> io::Result<()> {
        write!(self.writer, "<<")?;
        for (key, value) in dict.iter() {
            write!(self.writer, " ")?;
            self.write_name(key)?;
            write!(self.writer, " ")?;
            self.write_object(value)?;
        }
        write!(self.writer, " >>")
    }

    /// Write a PDF stream
    fn write_stream(&mut self, stream: &PdfStream) -> io::Result<()> {
        // Write stream dictionary
        self.write_dictionary(&stream.dict)?;
        write!(self.writer, "\nstream\n")?;
        self.writer.write_all(&stream.data)?;
        write!(self.writer, "\nendstream")
    }

    /// Consume the serializer and return the writer
    pub fn into_inner(self) -> W {
        self.writer
    }
}

// Convenience constructors for PdfObject
impl PdfObject {
    /// Create an integer object
    pub fn int(n: i64) -> Self {
        PdfObject::Integer(n)
    }

    /// Create a real number object
    pub fn real(n: f64) -> Self {
        PdfObject::Real(n)
    }

    /// Create a name object
    pub fn name(s: impl Into<String>) -> Self {
        PdfObject::Name(s.into())
    }

    /// Create a string object
    pub fn string(s: impl Into<Vec<u8>>) -> Self {
        PdfObject::String(PdfString::literal(s))
    }

    /// Create a reference object
    pub fn reference(obj_num: u32, gen_num: u16) -> Self {
        PdfObject::Reference(obj_num, gen_num)
    }

    /// Create an array from a vector of objects
    pub fn array(objects: Vec<PdfObject>) -> Self {
        PdfObject::Array(objects)
    }

    /// Create a dictionary object
    pub fn dict(dict: PdfDictionary) -> Self {
        PdfObject::Dictionary(dict)
    }
}

impl From<bool> for PdfObject {
    fn from(b: bool) -> Self {
        PdfObject::Boolean(b)
    }
}

impl From<i32> for PdfObject {
    fn from(n: i32) -> Self {
        PdfObject::Integer(n as i64)
    }
}

impl From<i64> for PdfObject {
    fn from(n: i64) -> Self {
        PdfObject::Integer(n)
    }
}

impl From<f64> for PdfObject {
    fn from(n: f64) -> Self {
        PdfObject::Real(n)
    }
}

impl From<f32> for PdfObject {
    fn from(n: f32) -> Self {
        PdfObject::Real(n as f64)
    }
}

impl From<&str> for PdfObject {
    fn from(s: &str) -> Self {
        PdfObject::String(PdfString::from_str(s))
    }
}

impl From<String> for PdfObject {
    fn from(s: String) -> Self {
        PdfObject::String(PdfString::from_str(&s))
    }
}

impl From<PdfDictionary> for PdfObject {
    fn from(dict: PdfDictionary) -> Self {
        PdfObject::Dictionary(dict)
    }
}

impl From<PdfStream> for PdfObject {
    fn from(stream: PdfStream) -> Self {
        PdfObject::Stream(stream)
    }
}

impl From<Vec<PdfObject>> for PdfObject {
    fn from(arr: Vec<PdfObject>) -> Self {
        PdfObject::Array(arr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_null() {
        let mut buf = Vec::new();
        let mut serializer = PdfSerializer::new(&mut buf);
        serializer.write_object(&PdfObject::Null).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "null");
    }

    #[test]
    fn test_serialize_boolean() {
        let mut buf = Vec::new();
        let mut serializer = PdfSerializer::new(&mut buf);
        serializer.write_object(&PdfObject::Boolean(true)).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "true");
    }

    #[test]
    fn test_serialize_integer() {
        let mut buf = Vec::new();
        let mut serializer = PdfSerializer::new(&mut buf);
        serializer.write_object(&PdfObject::Integer(42)).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "42");
    }

    #[test]
    fn test_serialize_real() {
        let mut buf = Vec::new();
        let mut serializer = PdfSerializer::new(&mut buf);
        serializer.write_object(&PdfObject::Real(3.14159)).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "3.14159");
    }

    #[test]
    fn test_serialize_string() {
        let mut buf = Vec::new();
        let mut serializer = PdfSerializer::new(&mut buf);
        serializer
            .write_object(&PdfObject::String(PdfString::from_str("Hello")))
            .unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "(Hello)");
    }

    #[test]
    fn test_serialize_name() {
        let mut buf = Vec::new();
        let mut serializer = PdfSerializer::new(&mut buf);
        serializer
            .write_object(&PdfObject::Name("Type".to_string()))
            .unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "/Type");
    }

    #[test]
    fn test_serialize_array() {
        let mut buf = Vec::new();
        let mut serializer = PdfSerializer::new(&mut buf);
        serializer
            .write_object(&PdfObject::Array(vec![
                PdfObject::Integer(1),
                PdfObject::Integer(2),
                PdfObject::Integer(3),
            ]))
            .unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "[1 2 3]");
    }

    #[test]
    fn test_serialize_dictionary() {
        let mut dict = PdfDictionary::new();
        dict.insert("Type", PdfObject::Name("Page".to_string()));

        let mut buf = Vec::new();
        let mut serializer = PdfSerializer::new(&mut buf);
        serializer.write_object(&PdfObject::Dictionary(dict)).unwrap();

        let result = String::from_utf8(buf).unwrap();
        assert!(result.contains("/Type"));
        assert!(result.contains("/Page"));
    }

    #[test]
    fn test_serialize_reference() {
        let mut buf = Vec::new();
        let mut serializer = PdfSerializer::new(&mut buf);
        serializer.write_object(&PdfObject::Reference(1, 0)).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "1 0 R");
    }
}
