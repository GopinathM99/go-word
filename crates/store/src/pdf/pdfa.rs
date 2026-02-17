//! PDF/A Compliance Module
//!
//! This module provides PDF/A-1b and PDF/A-2b compliant PDF generation.
//! PDF/A is an ISO-standardized version of PDF designed for long-term archiving.
//!
//! # PDF/A Requirements
//!
//! ## PDF/A-1b (ISO 19005-1, Level B)
//! - All fonts must be embedded
//! - No transparency allowed
//! - XMP metadata required
//! - Output intents required (color profile)
//! - No encryption allowed
//! - No LZW compression
//! - No external references
//!
//! ## PDF/A-2b (ISO 19005-2, Level B)
//! - Same as PDF/A-1b but allows:
//!   - JPEG2000 compression
//!   - Transparency (with restrictions)
//!   - Layers (optional content)
//!   - PDF/A-1 attachments

use super::document::{DocumentInfo, PdfVersion};
use super::objects::{PdfDictionary, PdfObject, PdfStream, PdfString};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// PDF/A conformance level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PdfAConformance {
    /// No PDF/A compliance (standard PDF)
    #[default]
    None,
    /// PDF/A-1b (ISO 19005-1, Level B - basic visual appearance)
    #[serde(rename = "1b")]
    PdfA1b,
    /// PDF/A-2b (ISO 19005-2, Level B - allows JPEG2000 and transparency)
    #[serde(rename = "2b")]
    PdfA2b,
}

impl PdfAConformance {
    /// Get the PDF/A part number (1 or 2)
    pub fn part(&self) -> Option<i32> {
        match self {
            PdfAConformance::None => None,
            PdfAConformance::PdfA1b => Some(1),
            PdfAConformance::PdfA2b => Some(2),
        }
    }

    /// Get the conformance level string (e.g., "B")
    pub fn conformance_level(&self) -> Option<&'static str> {
        match self {
            PdfAConformance::None => None,
            PdfAConformance::PdfA1b | PdfAConformance::PdfA2b => Some("B"),
        }
    }

    /// Get the required PDF version for this conformance level
    pub fn required_pdf_version(&self) -> PdfVersion {
        match self {
            PdfAConformance::None => PdfVersion::V1_4,
            PdfAConformance::PdfA1b => PdfVersion::V1_4,
            PdfAConformance::PdfA2b => PdfVersion::V1_7,
        }
    }

    /// Check if transparency is allowed
    pub fn allows_transparency(&self) -> bool {
        match self {
            PdfAConformance::None => true,
            PdfAConformance::PdfA1b => false,
            PdfAConformance::PdfA2b => true,
        }
    }

    /// Check if JPEG2000 is allowed
    pub fn allows_jpeg2000(&self) -> bool {
        match self {
            PdfAConformance::None => true,
            PdfAConformance::PdfA1b => false,
            PdfAConformance::PdfA2b => true,
        }
    }

    /// Get the XMP namespace for PDF/A identification
    pub fn xmp_namespace(&self) -> &'static str {
        "http://www.aiim.org/pdfa/ns/id/"
    }

    /// Get the XMP schema prefix
    pub fn xmp_prefix(&self) -> &'static str {
        "pdfaid"
    }

    /// Check if this conformance level requires font embedding
    pub fn requires_font_embedding(&self) -> bool {
        !matches!(self, PdfAConformance::None)
    }

    /// Check if this is a valid PDF/A conformance level
    pub fn is_pdfa(&self) -> bool {
        !matches!(self, PdfAConformance::None)
    }
}

impl std::fmt::Display for PdfAConformance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PdfAConformance::None => write!(f, "None"),
            PdfAConformance::PdfA1b => write!(f, "PDF/A-1b"),
            PdfAConformance::PdfA2b => write!(f, "PDF/A-2b"),
        }
    }
}

/// PDF/A validation error
#[derive(Debug, Error)]
pub enum PdfAError {
    /// Font is not embedded
    #[error("Font not embedded: {0}")]
    FontNotEmbedded(String),
    /// Transparency found in PDF/A-1
    #[error("Transparency not allowed in PDF/A-1b: {0}")]
    TransparencyNotAllowed(String),
    /// Invalid color space
    #[error("Invalid color space for PDF/A: {0}")]
    InvalidColorSpace(String),
    /// Missing metadata
    #[error("Missing required metadata: {0}")]
    MissingMetadata(String),
    /// Missing output intent
    #[error("Missing output intent for PDF/A")]
    MissingOutputIntent,
    /// Encryption not allowed
    #[error("Encryption is not allowed in PDF/A")]
    EncryptionNotAllowed,
    /// External reference not allowed
    #[error("External references are not allowed in PDF/A")]
    ExternalReferenceNotAllowed,
    /// LZW compression not allowed in PDF/A-1
    #[error("LZW compression is not allowed in PDF/A-1")]
    LzwCompressionNotAllowed,
    /// Unsupported feature
    #[error("Unsupported feature for PDF/A: {0}")]
    UnsupportedFeature(String),
}

/// PDF/A compliance issue (non-fatal warning)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplianceIssue {
    /// Issue severity
    pub severity: IssueSeverity,
    /// Issue category
    pub category: IssueCategory,
    /// Human-readable description
    pub description: String,
    /// Suggestion for fixing the issue
    pub suggestion: Option<String>,
}

/// Severity level for compliance issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    /// Error - must be fixed for compliance
    Error,
    /// Warning - may cause issues with some validators
    Warning,
    /// Info - informational message
    Info,
}

/// Category of compliance issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IssueCategory {
    /// Font-related issue
    Font,
    /// Metadata issue
    Metadata,
    /// Color space issue
    ColorSpace,
    /// Transparency issue
    Transparency,
    /// Image compression issue
    ImageCompression,
    /// Security/encryption issue
    Security,
    /// External reference issue
    ExternalReference,
    /// Structure issue
    Structure,
}

impl ComplianceIssue {
    /// Create a new compliance issue
    pub fn new(severity: IssueSeverity, category: IssueCategory, description: impl Into<String>) -> Self {
        Self {
            severity,
            category,
            description: description.into(),
            suggestion: None,
        }
    }

    /// Add a suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Create an error issue
    pub fn error(category: IssueCategory, description: impl Into<String>) -> Self {
        Self::new(IssueSeverity::Error, category, description)
    }

    /// Create a warning issue
    pub fn warning(category: IssueCategory, description: impl Into<String>) -> Self {
        Self::new(IssueSeverity::Warning, category, description)
    }

    /// Create an info issue
    pub fn info(category: IssueCategory, description: impl Into<String>) -> Self {
        Self::new(IssueSeverity::Info, category, description)
    }
}

/// PDF/A compliance validation result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplianceReport {
    /// Target conformance level
    pub conformance: PdfAConformance,
    /// Whether the document is compliant
    pub is_compliant: bool,
    /// List of issues found
    pub issues: Vec<ComplianceIssue>,
    /// Fonts that need to be embedded
    pub fonts_to_embed: Vec<String>,
    /// Whether transparency was detected
    pub has_transparency: bool,
    /// Color spaces used
    pub color_spaces: Vec<String>,
}

impl ComplianceReport {
    /// Create a new compliance report
    pub fn new(conformance: PdfAConformance) -> Self {
        Self {
            conformance,
            is_compliant: true,
            issues: Vec::new(),
            fonts_to_embed: Vec::new(),
            has_transparency: false,
            color_spaces: Vec::new(),
        }
    }

    /// Add an issue
    pub fn add_issue(&mut self, issue: ComplianceIssue) {
        if issue.severity == IssueSeverity::Error {
            self.is_compliant = false;
        }
        self.issues.push(issue);
    }

    /// Get only error-level issues
    pub fn errors(&self) -> Vec<&ComplianceIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Error)
            .collect()
    }

    /// Get only warning-level issues
    pub fn warnings(&self) -> Vec<&ComplianceIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Warning)
            .collect()
    }

    /// Check if document has any issues
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    /// Count errors
    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == IssueSeverity::Error).count()
    }

    /// Count warnings
    pub fn warning_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count()
    }
}

/// XMP metadata generator for PDF/A
pub struct XmpMetadata {
    /// Document title
    pub title: Option<String>,
    /// Document creator (author)
    pub creator: Option<String>,
    /// Document description
    pub description: Option<String>,
    /// Creation date (ISO 8601 format)
    pub create_date: Option<String>,
    /// Modification date (ISO 8601 format)
    pub modify_date: Option<String>,
    /// Creator tool (application name)
    pub creator_tool: Option<String>,
    /// PDF/A part number
    pub pdfa_part: Option<i32>,
    /// PDF/A conformance level
    pub pdfa_conformance: Option<String>,
    /// Document ID (UUID)
    pub document_id: Option<String>,
    /// Instance ID (UUID for this version)
    pub instance_id: Option<String>,
}

impl Default for XmpMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl XmpMetadata {
    /// Create new XMP metadata
    pub fn new() -> Self {
        Self {
            title: None,
            creator: None,
            description: None,
            create_date: None,
            modify_date: None,
            creator_tool: Some("Go Word".to_string()),
            pdfa_part: None,
            pdfa_conformance: None,
            document_id: None,
            instance_id: None,
        }
    }

    /// Create from document info and conformance level
    pub fn from_document_info(info: &DocumentInfo, conformance: PdfAConformance) -> Self {
        let mut metadata = Self::new();
        metadata.title = info.title.clone();
        metadata.creator = info.author.clone();
        metadata.description = info.subject.clone();
        metadata.creator_tool = info.creator.clone();
        metadata.create_date = info.creation_date.clone();
        metadata.modify_date = info.modification_date.clone();

        if let Some(part) = conformance.part() {
            metadata.pdfa_part = Some(part);
            metadata.pdfa_conformance = conformance.conformance_level().map(|s| s.to_string());
        }

        // Generate UUIDs for document and instance IDs
        metadata.document_id = Some(generate_uuid());
        metadata.instance_id = Some(generate_uuid());

        metadata
    }

    /// Generate XMP packet as bytes
    pub fn generate(&self) -> Vec<u8> {
        let mut xmp = String::new();

        // XMP header with packet wrapper
        xmp.push_str(r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>"#);
        xmp.push('\n');
        xmp.push_str(r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">"#);
        xmp.push('\n');
        xmp.push_str(r#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">"#);
        xmp.push('\n');

        // Dublin Core namespace (dc)
        xmp.push_str(r#"<rdf:Description rdf:about="" xmlns:dc="http://purl.org/dc/elements/1.1/">"#);
        xmp.push('\n');

        if let Some(title) = &self.title {
            xmp.push_str(&format!(
                r#"<dc:title><rdf:Alt><rdf:li xml:lang="x-default">{}</rdf:li></rdf:Alt></dc:title>"#,
                escape_xml(title)
            ));
            xmp.push('\n');
        }

        if let Some(creator) = &self.creator {
            xmp.push_str(&format!(
                r#"<dc:creator><rdf:Seq><rdf:li>{}</rdf:li></rdf:Seq></dc:creator>"#,
                escape_xml(creator)
            ));
            xmp.push('\n');
        }

        if let Some(description) = &self.description {
            xmp.push_str(&format!(
                r#"<dc:description><rdf:Alt><rdf:li xml:lang="x-default">{}</rdf:li></rdf:Alt></dc:description>"#,
                escape_xml(description)
            ));
            xmp.push('\n');
        }

        // Format element (required for PDF/A)
        xmp.push_str(r#"<dc:format>application/pdf</dc:format>"#);
        xmp.push('\n');

        xmp.push_str(r#"</rdf:Description>"#);
        xmp.push('\n');

        // XMP namespace
        xmp.push_str(r#"<rdf:Description rdf:about="" xmlns:xmp="http://ns.adobe.com/xap/1.0/">"#);
        xmp.push('\n');

        if let Some(creator_tool) = &self.creator_tool {
            xmp.push_str(&format!(r#"<xmp:CreatorTool>{}</xmp:CreatorTool>"#, escape_xml(creator_tool)));
            xmp.push('\n');
        }

        if let Some(date) = &self.create_date {
            xmp.push_str(&format!(r#"<xmp:CreateDate>{}</xmp:CreateDate>"#, date));
            xmp.push('\n');
        }

        if let Some(date) = &self.modify_date {
            xmp.push_str(&format!(r#"<xmp:ModifyDate>{}</xmp:ModifyDate>"#, date));
            xmp.push('\n');
        }

        xmp.push_str(r#"</rdf:Description>"#);
        xmp.push('\n');

        // PDF namespace
        xmp.push_str(r#"<rdf:Description rdf:about="" xmlns:pdf="http://ns.adobe.com/pdf/1.3/">"#);
        xmp.push('\n');
        xmp.push_str(r#"<pdf:Producer>Go Word PDF/A Export</pdf:Producer>"#);
        xmp.push('\n');
        xmp.push_str(r#"</rdf:Description>"#);
        xmp.push('\n');

        // XMP Media Management namespace (for document IDs)
        xmp.push_str(r#"<rdf:Description rdf:about="" xmlns:xmpMM="http://ns.adobe.com/xap/1.0/mm/">"#);
        xmp.push('\n');

        if let Some(doc_id) = &self.document_id {
            xmp.push_str(&format!(r#"<xmpMM:DocumentID>uuid:{}</xmpMM:DocumentID>"#, doc_id));
            xmp.push('\n');
        }

        if let Some(inst_id) = &self.instance_id {
            xmp.push_str(&format!(r#"<xmpMM:InstanceID>uuid:{}</xmpMM:InstanceID>"#, inst_id));
            xmp.push('\n');
        }

        xmp.push_str(r#"</rdf:Description>"#);
        xmp.push('\n');

        // PDF/A identification (if applicable)
        if let (Some(part), Some(conformance)) = (&self.pdfa_part, &self.pdfa_conformance) {
            xmp.push_str(r#"<rdf:Description rdf:about="" xmlns:pdfaid="http://www.aiim.org/pdfa/ns/id/">"#);
            xmp.push('\n');
            xmp.push_str(&format!(r#"<pdfaid:part>{}</pdfaid:part>"#, part));
            xmp.push('\n');
            xmp.push_str(&format!(r#"<pdfaid:conformance>{}</pdfaid:conformance>"#, conformance));
            xmp.push('\n');
            xmp.push_str(r#"</rdf:Description>"#);
            xmp.push('\n');
        }

        xmp.push_str(r#"</rdf:RDF>"#);
        xmp.push('\n');
        xmp.push_str(r#"</x:xmpmeta>"#);
        xmp.push('\n');

        // Padding for in-place updates (PDF/A allows this)
        for _ in 0..20 {
            xmp.push_str("                                                                                \n");
        }

        xmp.push_str(r#"<?xpacket end="w"?>"#);

        xmp.into_bytes()
    }

    /// Create a PDF stream object for the metadata
    pub fn to_stream(&self) -> PdfStream {
        let data = self.generate();
        let mut dict = PdfDictionary::new().with_type("Metadata");
        dict.insert("Subtype", PdfObject::Name("XML".to_string()));
        dict.insert("Length", PdfObject::Integer(data.len() as i64));

        PdfStream {
            dict,
            data,
            compressed: false, // XMP metadata should not be compressed for accessibility
        }
    }
}

/// sRGB color profile for output intent
pub fn create_srgb_output_intent(icc_profile_ref: u32) -> PdfDictionary {
    let mut dict = PdfDictionary::new().with_type("OutputIntent");

    // Output intent subtype
    dict.insert("S", PdfObject::Name("GTS_PDFA1".to_string()));

    // Output condition identifier (standard sRGB)
    dict.insert("OutputConditionIdentifier", PdfObject::String(PdfString::from_str("sRGB IEC61966-2.1")));

    // Human-readable output condition
    dict.insert("OutputCondition", PdfObject::String(PdfString::from_str("sRGB")));

    // Registry name
    dict.insert("RegistryName", PdfObject::String(PdfString::from_str("http://www.color.org")));

    // Info about the output condition
    dict.insert("Info", PdfObject::String(PdfString::from_str("sRGB IEC61966-2.1")));

    // Reference to ICC profile
    dict.insert("DestOutputProfile", PdfObject::Reference(icc_profile_ref, 0));

    dict
}

/// Create a minimal sRGB ICC profile
///
/// This is a simplified sRGB profile that meets PDF/A requirements.
/// For production use, consider using a full ICC profile from the ICC website.
pub fn create_srgb_icc_profile() -> PdfStream {
    // Minimal sRGB ICC profile (v2.1)
    // This is a valid but minimal ICC profile that identifies as sRGB
    let profile_data = create_minimal_srgb_profile();

    let mut dict = PdfDictionary::new();
    dict.insert("N", PdfObject::Integer(3)); // Number of components (RGB)
    dict.insert("Length", PdfObject::Integer(profile_data.len() as i64));
    dict.insert("Filter", PdfObject::Name("FlateDecode".to_string()));

    // Compress the profile
    let compressed = compress_data(&profile_data);

    PdfStream {
        dict,
        data: compressed,
        compressed: true,
    }
}

/// Create a minimal but valid sRGB ICC profile
fn create_minimal_srgb_profile() -> Vec<u8> {
    // This is a simplified sRGB profile header
    // A full implementation would include the complete ICC profile data

    let mut profile = Vec::new();

    // Profile header (128 bytes)
    // Profile size (will be updated at the end)
    profile.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

    // CMM type signature
    profile.extend_from_slice(b"appl");

    // Profile version (2.1.0)
    profile.extend_from_slice(&[0x02, 0x10, 0x00, 0x00]);

    // Profile/Device class ('mntr' for monitor)
    profile.extend_from_slice(b"mntr");

    // Color space ('RGB ')
    profile.extend_from_slice(b"RGB ");

    // Profile connection space ('XYZ ')
    profile.extend_from_slice(b"XYZ ");

    // Date/time (zeros for now)
    profile.extend_from_slice(&[0u8; 12]);

    // Profile file signature ('acsp')
    profile.extend_from_slice(b"acsp");

    // Primary platform signature
    profile.extend_from_slice(b"APPL");

    // Profile flags
    profile.extend_from_slice(&[0u8; 4]);

    // Device manufacturer
    profile.extend_from_slice(&[0u8; 4]);

    // Device model
    profile.extend_from_slice(&[0u8; 4]);

    // Device attributes
    profile.extend_from_slice(&[0u8; 8]);

    // Rendering intent (perceptual)
    profile.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

    // PCS illuminant (D50)
    // X: 0.9642 (fixed point s15Fixed16)
    profile.extend_from_slice(&[0x00, 0x00, 0xF6, 0xD6]);
    // Y: 1.0000
    profile.extend_from_slice(&[0x00, 0x01, 0x00, 0x00]);
    // Z: 0.8249
    profile.extend_from_slice(&[0x00, 0x00, 0xD3, 0x2D]);

    // Profile creator signature
    profile.extend_from_slice(&[0u8; 4]);

    // Profile ID (MD5 checksum, zeros for now)
    profile.extend_from_slice(&[0u8; 16]);

    // Reserved
    profile.extend_from_slice(&[0u8; 28]);

    // Tag table
    // Tag count: 9 tags for minimal sRGB
    profile.extend_from_slice(&[0x00, 0x00, 0x00, 0x09]);

    // Tag entries (each 12 bytes: signature, offset, size)
    let tag_data_start = 128 + 4 + (9 * 12); // header + count + tag table

    // Add minimal required tags
    let tags: &[(&[u8; 4], &[u8])] = &[
        (b"desc", b"sRGB IEC61966-2.1"),
        (b"cprt", b"Public Domain"),
        (b"wtpt", &create_xyz_type(0.9505, 1.0, 1.0890)), // D65 white point
        (b"bkpt", &create_xyz_type(0.0, 0.0, 0.0)), // Black point
        (b"rXYZ", &create_xyz_type(0.4360, 0.2225, 0.0139)), // Red primary
        (b"gXYZ", &create_xyz_type(0.3851, 0.7169, 0.0971)), // Green primary
        (b"bXYZ", &create_xyz_type(0.1431, 0.0606, 0.7139)), // Blue primary
        (b"rTRC", &create_trc_type()), // Red TRC (gamma 2.2 approx)
        (b"gTRC", &create_trc_type()), // Green TRC
    ];

    // Calculate offsets and write tag table
    let mut current_offset = tag_data_start;
    let mut tag_data_blocks: Vec<Vec<u8>> = Vec::new();

    for (sig, data) in tags {
        // Wrap data in appropriate ICC type
        let wrapped = wrap_tag_data(sig, data);

        // Tag signature
        profile.extend_from_slice(*sig);
        // Tag offset
        profile.extend_from_slice(&(current_offset as u32).to_be_bytes());
        // Tag size
        profile.extend_from_slice(&(wrapped.len() as u32).to_be_bytes());

        current_offset += wrapped.len();
        // Align to 4 bytes
        while current_offset % 4 != 0 {
            current_offset += 1;
        }

        tag_data_blocks.push(wrapped);
    }

    // bTRC points to same data as rTRC (sharing allowed in ICC)
    profile.extend_from_slice(b"bTRC");
    let rtrc_offset = tag_data_start + tag_data_blocks.iter().take(7).map(|b| {
        let mut len = b.len();
        while len % 4 != 0 { len += 1; }
        len
    }).sum::<usize>();
    profile.extend_from_slice(&(rtrc_offset as u32).to_be_bytes());
    profile.extend_from_slice(&(tag_data_blocks[7].len() as u32).to_be_bytes());

    // Write tag data
    for block in tag_data_blocks {
        profile.extend_from_slice(&block);
        // Pad to 4-byte alignment
        while profile.len() % 4 != 0 {
            profile.push(0);
        }
    }

    // Update profile size
    let size = profile.len() as u32;
    profile[0..4].copy_from_slice(&size.to_be_bytes());

    profile
}

/// Create ICC XYZ type data
fn create_xyz_type(x: f64, y: f64, z: f64) -> Vec<u8> {
    let mut data = Vec::new();
    // s15Fixed16Number format
    data.extend_from_slice(&to_s15fixed16(x).to_be_bytes());
    data.extend_from_slice(&to_s15fixed16(y).to_be_bytes());
    data.extend_from_slice(&to_s15fixed16(z).to_be_bytes());
    data
}

/// Create ICC TRC (Tone Reproduction Curve) type data for gamma 2.2
fn create_trc_type() -> Vec<u8> {
    // Simple gamma curve (gamma = 2.2)
    let gamma: u32 = (2.2 * 65536.0) as u32;
    gamma.to_be_bytes().to_vec()
}

/// Wrap tag data in appropriate ICC type structure
fn wrap_tag_data(sig: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut wrapped = Vec::new();

    match sig {
        b"desc" => {
            // textDescriptionType
            wrapped.extend_from_slice(b"desc");
            wrapped.extend_from_slice(&[0, 0, 0, 0]); // Reserved

            // ASCII description
            wrapped.extend_from_slice(&((data.len() + 1) as u32).to_be_bytes());
            wrapped.extend_from_slice(data);
            wrapped.push(0); // Null terminator

            // Unicode count (0)
            wrapped.extend_from_slice(&[0, 0, 0, 0]);
            // Unicode code (0)
            wrapped.extend_from_slice(&[0, 0, 0, 0]);
            // ScriptCode count (0)
            wrapped.extend_from_slice(&[0, 0]);
            // ScriptCode code (0)
            wrapped.push(0);
            // ScriptCode string (67 bytes, all zeros for "67 Pascal string")
            wrapped.extend_from_slice(&[0u8; 67]);
        }
        b"cprt" => {
            // textType
            wrapped.extend_from_slice(b"text");
            wrapped.extend_from_slice(&[0, 0, 0, 0]); // Reserved
            wrapped.extend_from_slice(data);
            wrapped.push(0); // Null terminator
        }
        b"wtpt" | b"bkpt" | b"rXYZ" | b"gXYZ" | b"bXYZ" => {
            // XYZType
            wrapped.extend_from_slice(b"XYZ ");
            wrapped.extend_from_slice(&[0, 0, 0, 0]); // Reserved
            wrapped.extend_from_slice(data);
        }
        b"rTRC" | b"gTRC" | b"bTRC" => {
            // curveType with gamma value
            wrapped.extend_from_slice(b"curv");
            wrapped.extend_from_slice(&[0, 0, 0, 0]); // Reserved
            wrapped.extend_from_slice(&[0, 0, 0, 1]); // Curve count = 1 (gamma)
            wrapped.extend_from_slice(&data[0..2]); // Gamma value (u8Fixed8Number)
        }
        _ => {
            wrapped.extend_from_slice(data);
        }
    }

    wrapped
}

/// Convert f64 to ICC s15Fixed16Number
fn to_s15fixed16(value: f64) -> i32 {
    (value * 65536.0) as i32
}

/// Compress data using zlib/deflate
fn compress_data(data: &[u8]) -> Vec<u8> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

/// Create MarkInfo dictionary for PDF/A
pub fn create_mark_info(marked: bool) -> PdfDictionary {
    let mut dict = PdfDictionary::new();
    dict.insert("Marked", PdfObject::Boolean(marked));
    dict
}

/// Escape special XML characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Generate a simple UUID v4
fn generate_uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    // Generate pseudo-random UUID (not cryptographically secure, but sufficient for document IDs)
    let mut bytes = [0u8; 16];
    let ts_bytes = timestamp.to_le_bytes();

    for (i, &b) in ts_bytes.iter().enumerate().take(16) {
        bytes[i] = b;
    }

    // Set version 4 (random)
    bytes[6] = (bytes[6] & 0x0F) | 0x40;
    // Set variant (RFC 4122)
    bytes[8] = (bytes[8] & 0x3F) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

/// Get current date/time in ISO 8601 format for XMP
pub fn get_iso_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();

    let secs = duration.as_secs();

    // Simple conversion (not accounting for leap seconds, but good enough for documents)
    let days_since_epoch = secs / 86400;
    let secs_today = secs % 86400;

    // Calculate year, month, day (simplified)
    let mut year = 1970;
    let mut remaining_days = days_since_epoch;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months: [u64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for days in days_in_months {
        if remaining_days < days {
            break;
        }
        remaining_days -= days;
        month += 1;
    }

    let day = remaining_days + 1;
    let hour = secs_today / 3600;
    let minute = (secs_today % 3600) / 60;
    let second = secs_today % 60;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, minute, second
    )
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Validate a document for PDF/A compliance
pub struct PdfAValidator {
    /// Target conformance level
    conformance: PdfAConformance,
    /// Fonts used in the document
    fonts_used: HashSet<String>,
    /// Whether embedded fonts were detected
    has_embedded_fonts: bool,
    /// Whether transparency was detected
    has_transparency: bool,
    /// Color spaces detected
    color_spaces: HashSet<String>,
}

impl PdfAValidator {
    /// Create a new validator for the given conformance level
    pub fn new(conformance: PdfAConformance) -> Self {
        Self {
            conformance,
            fonts_used: HashSet::new(),
            has_embedded_fonts: false,
            has_transparency: false,
            color_spaces: HashSet::new(),
        }
    }

    /// Add a font that's used in the document
    pub fn add_font(&mut self, font_name: &str, is_embedded: bool) {
        self.fonts_used.insert(font_name.to_string());
        if is_embedded {
            self.has_embedded_fonts = true;
        }
    }

    /// Mark that transparency is used
    pub fn set_has_transparency(&mut self, has_transparency: bool) {
        self.has_transparency = has_transparency;
    }

    /// Add a color space
    pub fn add_color_space(&mut self, color_space: &str) {
        self.color_spaces.insert(color_space.to_string());
    }

    /// Validate and generate a compliance report
    pub fn validate(&self) -> ComplianceReport {
        let mut report = ComplianceReport::new(self.conformance);

        if self.conformance == PdfAConformance::None {
            return report;
        }

        // Check font embedding
        for font in &self.fonts_used {
            if !self.has_embedded_fonts {
                report.add_issue(
                    ComplianceIssue::error(IssueCategory::Font, format!("Font '{}' is not embedded", font))
                        .with_suggestion("Enable font embedding in PDF/A export options")
                );
                report.fonts_to_embed.push(font.clone());
            }
        }

        // Check transparency for PDF/A-1
        if self.has_transparency && !self.conformance.allows_transparency() {
            report.add_issue(
                ComplianceIssue::error(
                    IssueCategory::Transparency,
                    "Transparency is not allowed in PDF/A-1b"
                )
                .with_suggestion("Use PDF/A-2b conformance level or flatten transparency")
            );
        }
        report.has_transparency = self.has_transparency;

        // Check color spaces
        for cs in &self.color_spaces {
            // PDF/A requires device-independent color spaces or ICC-based spaces
            let is_valid = matches!(
                cs.as_str(),
                "DeviceRGB" | "DeviceGray" | "DeviceCMYK" | "ICCBased" | "sRGB"
            );

            if !is_valid {
                report.add_issue(
                    ComplianceIssue::warning(
                        IssueCategory::ColorSpace,
                        format!("Color space '{}' may not be valid for PDF/A", cs)
                    )
                );
            }
            report.color_spaces.push(cs.clone());
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdfa_conformance_properties() {
        assert_eq!(PdfAConformance::PdfA1b.part(), Some(1));
        assert_eq!(PdfAConformance::PdfA2b.part(), Some(2));
        assert_eq!(PdfAConformance::None.part(), None);

        assert!(!PdfAConformance::PdfA1b.allows_transparency());
        assert!(PdfAConformance::PdfA2b.allows_transparency());
        assert!(PdfAConformance::None.allows_transparency());

        assert!(PdfAConformance::PdfA1b.requires_font_embedding());
        assert!(!PdfAConformance::None.requires_font_embedding());
    }

    #[test]
    fn test_xmp_metadata_generation() {
        let mut metadata = XmpMetadata::new();
        metadata.title = Some("Test Document".to_string());
        metadata.creator = Some("Test Author".to_string());
        metadata.pdfa_part = Some(1);
        metadata.pdfa_conformance = Some("B".to_string());

        let xmp = metadata.generate();
        let xmp_str = String::from_utf8_lossy(&xmp);

        assert!(xmp_str.contains("Test Document"));
        assert!(xmp_str.contains("Test Author"));
        assert!(xmp_str.contains("pdfaid:part"));
        assert!(xmp_str.contains("pdfaid:conformance"));
        assert!(xmp_str.contains("<?xpacket"));
    }

    #[test]
    fn test_compliance_report() {
        let mut report = ComplianceReport::new(PdfAConformance::PdfA1b);
        assert!(report.is_compliant);
        assert_eq!(report.error_count(), 0);

        report.add_issue(ComplianceIssue::error(
            IssueCategory::Font,
            "Font not embedded"
        ));

        assert!(!report.is_compliant);
        assert_eq!(report.error_count(), 1);
    }

    #[test]
    fn test_validator() {
        let mut validator = PdfAValidator::new(PdfAConformance::PdfA1b);
        validator.add_font("Helvetica", false);
        validator.set_has_transparency(true);
        validator.add_color_space("DeviceRGB");

        let report = validator.validate();

        assert!(!report.is_compliant);
        assert!(report.error_count() >= 2); // Font not embedded + transparency
    }

    #[test]
    fn test_iso_date_format() {
        let date = get_iso_date();
        // Check format: YYYY-MM-DDTHH:MM:SSZ
        assert!(date.contains("-"));
        assert!(date.contains("T"));
        assert!(date.ends_with("Z"));
    }

    #[test]
    fn test_xml_escaping() {
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("\"quote\""), "&quot;quote&quot;");
    }

    #[test]
    fn test_output_intent() {
        let intent = create_srgb_output_intent(10);
        assert!(intent.get("Type").is_some());
        assert!(intent.get("S").is_some());
        assert!(intent.get("OutputConditionIdentifier").is_some());
    }

    #[test]
    fn test_mark_info() {
        let mark_info = create_mark_info(true);
        assert!(mark_info.get("Marked").is_some());
    }
}
