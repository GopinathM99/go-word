//! ODT Import Module (Read-Only)
//!
//! This module provides functionality to read OpenDocument Text (ODT) files.
//! ODT is the native format for LibreOffice Writer and other OpenDocument-compatible
//! applications. It is an open standard defined by OASIS.
//!
//! ## ODT Structure
//!
//! An ODT file is a ZIP archive containing:
//! - `content.xml` - Document content
//! - `styles.xml` - Style definitions
//! - `meta.xml` - Metadata (title, author, etc.)
//! - `settings.xml` - Application settings
//! - `Pictures/` - Embedded images
//! - `META-INF/manifest.xml` - Package manifest
//!
//! ## Note
//!
//! This module provides read-only support for ODT files.
//! Export to ODT is not supported.

mod error;
mod reader;
mod api;

pub use error::{OdtError, OdtResult};
pub use api::{import_odt, import_odt_bytes, OdtImportResult, OdtWarning, OdtWarningKind};

/// ODF XML namespaces
pub mod namespaces {
    /// Office namespace
    pub const OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
    /// Text namespace
    pub const TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
    /// Style namespace
    pub const STYLE: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
    /// Table namespace
    pub const TABLE: &str = "urn:oasis:names:tc:opendocument:xmlns:table:1.0";
    /// Drawing namespace
    pub const DRAW: &str = "urn:oasis:names:tc:opendocument:xmlns:drawing:1.0";
    /// FO (Formatting Objects) namespace
    pub const FO: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";
    /// SVG namespace
    pub const SVG: &str = "urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0";
    /// XLink namespace
    pub const XLINK: &str = "http://www.w3.org/1999/xlink";
    /// Dublin Core namespace (for metadata)
    pub const DC: &str = "http://purl.org/dc/elements/1.1/";
    /// Meta namespace
    pub const META: &str = "urn:oasis:names:tc:opendocument:xmlns:meta:1.0";
}

/// Common element names in ODF
pub mod elements {
    // Document structure
    pub const DOCUMENT_CONTENT: &str = "document-content";
    pub const DOCUMENT_STYLES: &str = "document-styles";
    pub const DOCUMENT_META: &str = "document-meta";
    pub const BODY: &str = "body";
    pub const TEXT_ELEM: &str = "text";

    // Text elements
    pub const P: &str = "p";
    pub const H: &str = "h";
    pub const SPAN: &str = "span";
    pub const S: &str = "s";  // Spaces
    pub const TAB: &str = "tab";
    pub const LINE_BREAK: &str = "line-break";
    pub const SOFT_PAGE_BREAK: &str = "soft-page-break";

    // List elements
    pub const LIST: &str = "list";
    pub const LIST_ITEM: &str = "list-item";

    // Table elements
    pub const TABLE: &str = "table";
    pub const TABLE_ROW: &str = "table-row";
    pub const TABLE_CELL: &str = "table-cell";
    pub const TABLE_COLUMN: &str = "table-column";
    pub const TABLE_HEADER_ROWS: &str = "table-header-rows";

    // Drawing elements
    pub const FRAME: &str = "frame";
    pub const IMAGE: &str = "image";

    // Style elements
    pub const STYLES: &str = "styles";
    pub const AUTOMATIC_STYLES: &str = "automatic-styles";
    pub const MASTER_STYLES: &str = "master-styles";
    pub const STYLE: &str = "style";
    pub const TEXT_PROPERTIES: &str = "text-properties";
    pub const PARAGRAPH_PROPERTIES: &str = "paragraph-properties";
    pub const TABLE_PROPERTIES: &str = "table-properties";
    pub const TABLE_COLUMN_PROPERTIES: &str = "table-column-properties";
    pub const TABLE_ROW_PROPERTIES: &str = "table-row-properties";
    pub const TABLE_CELL_PROPERTIES: &str = "table-cell-properties";

    // Meta elements
    pub const TITLE: &str = "title";
    pub const CREATOR: &str = "creator";
    pub const DATE: &str = "date";
    pub const CREATION_DATE: &str = "creation-date";
}

/// Common attribute names in ODF
pub mod attributes {
    // Style attributes
    pub const STYLE_NAME: &str = "style-name";
    pub const PARENT_STYLE_NAME: &str = "parent-style-name";
    pub const FAMILY: &str = "family";
    pub const NAME: &str = "name";

    // Text formatting attributes (fo: namespace)
    pub const FONT_SIZE: &str = "font-size";
    pub const FONT_WEIGHT: &str = "font-weight";
    pub const FONT_STYLE: &str = "font-style";
    pub const TEXT_DECORATION: &str = "text-decoration";
    pub const COLOR: &str = "color";
    pub const BACKGROUND_COLOR: &str = "background-color";

    // Paragraph attributes
    pub const TEXT_ALIGN: &str = "text-align";
    pub const MARGIN_LEFT: &str = "margin-left";
    pub const MARGIN_RIGHT: &str = "margin-right";
    pub const TEXT_INDENT: &str = "text-indent";
    pub const MARGIN_TOP: &str = "margin-top";
    pub const MARGIN_BOTTOM: &str = "margin-bottom";
    pub const LINE_HEIGHT: &str = "line-height";

    // Table attributes
    pub const NUMBER_COLUMNS_SPANNED: &str = "number-columns-spanned";
    pub const NUMBER_ROWS_SPANNED: &str = "number-rows-spanned";
    pub const NUMBER_COLUMNS_REPEATED: &str = "number-columns-repeated";
    pub const COLUMN_WIDTH: &str = "column-width";

    // Drawing attributes
    pub const WIDTH: &str = "width";
    pub const HEIGHT: &str = "height";
    pub const HREF: &str = "href";

    // Space count
    pub const C: &str = "c";

    // Outline level (for headings)
    pub const OUTLINE_LEVEL: &str = "outline-level";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Basic test to verify module compiles
        assert!(namespaces::OFFICE.contains("oasis"));
        assert_eq!(elements::P, "p");
        assert_eq!(attributes::FONT_SIZE, "font-size");
    }
}
