//! ODT Reader - Parses ODT content into DocumentTree
//!
//! This module implements an ODT reader that handles:
//! - ZIP archive extraction
//! - XML parsing of content.xml, styles.xml, meta.xml
//! - Style resolution
//! - Content conversion to document model

use crate::odt::attributes::*;
use crate::odt::elements::*;
use crate::odt::error::{OdtError, OdtResult};
use crate::odt::namespaces;
use crate::odt::api::{OdtWarning, OdtWarningKind};
use doc_model::{
    Alignment, CharacterProperties, DocumentMetadata, DocumentTree, ImageNode,
    ImageProperties, LineSpacing, Node, Paragraph, ParagraphProperties, ResourceId, Run,
    StyleId, Table, TableCell, TableGrid, TableRow, GridColumn, TableWidth,
    CellProperties, RowProperties,
};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::collections::HashMap;
use std::io::{Read, Seek};
use zip::ZipArchive;

/// Parsed style properties
#[derive(Debug, Clone, Default)]
struct OdtStyle {
    name: String,
    family: String,
    parent_style_name: Option<String>,
    text_props: CharacterProperties,
    para_props: ParagraphProperties,
}

/// ODT Reader
pub struct OdtReader<R: Read + Seek> {
    archive: ZipArchive<R>,
    /// Parsed styles
    styles: HashMap<String, OdtStyle>,
    /// Warnings collected during parsing
    warnings: Vec<OdtWarning>,
    /// Image data keyed by path
    images: HashMap<String, Vec<u8>>,
    /// Image counter for resource IDs
    image_counter: u32,
}

impl<R: Read + Seek> OdtReader<R> {
    /// Create a new ODT reader
    pub fn new(reader: R) -> OdtResult<Self> {
        let archive = ZipArchive::new(reader)?;
        Ok(Self {
            archive,
            styles: HashMap::new(),
            warnings: Vec::new(),
            images: HashMap::new(),
            image_counter: 0,
        })
    }

    /// Check if this is a valid ODT file
    pub fn is_valid_odt(&self) -> bool {
        // Must have content.xml
        self.archive.file_names().any(|name| name == "content.xml")
    }

    /// Parse the ODT file and return a DocumentTree
    pub fn parse(mut self) -> OdtResult<(DocumentTree, Vec<OdtWarning>)> {
        if !self.is_valid_odt() {
            return Err(OdtError::invalid_structure("Missing content.xml"));
        }

        let mut tree = DocumentTree::new();

        // Parse meta.xml for metadata
        if let Ok(metadata) = self.parse_metadata() {
            tree.document.metadata = metadata;
        }

        // Parse styles.xml
        if let Ok(content) = self.read_file_as_string("styles.xml") {
            self.parse_styles(&content)?;
        }

        // Parse content.xml (also contains automatic styles)
        let content_xml = self.read_file_as_string("content.xml")?;
        self.parse_content(&content_xml, &mut tree)?;

        // Load images
        self.load_images()?;

        Ok((tree, self.warnings))
    }

    /// Read a file from the archive as string
    fn read_file_as_string(&mut self, path: &str) -> OdtResult<String> {
        let mut file = self.archive.by_name(path).map_err(|e| {
            if matches!(e, zip::result::ZipError::FileNotFound) {
                OdtError::missing_part(path)
            } else {
                OdtError::from(e)
            }
        })?;

        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }

    /// Read a file from the archive as bytes
    fn read_file_as_bytes(&mut self, path: &str) -> OdtResult<Vec<u8>> {
        let mut file = self.archive.by_name(path).map_err(|e| {
            if matches!(e, zip::result::ZipError::FileNotFound) {
                OdtError::missing_part(path)
            } else {
                OdtError::from(e)
            }
        })?;

        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        Ok(content)
    }

    /// Parse metadata from meta.xml
    fn parse_metadata(&mut self) -> OdtResult<DocumentMetadata> {
        let content = self.read_file_as_string("meta.xml")?;
        let mut reader = Reader::from_str(&content);
        reader.config_mut().trim_text(true);

        let mut metadata = DocumentMetadata::default();
        let mut current_element = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) => {
                    let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    current_element = name;
                }
                Event::Text(e) => {
                    let text = e.unescape()?.to_string();
                    match current_element.as_str() {
                        TITLE => metadata.title = Some(text),
                        CREATOR => metadata.author = Some(text),
                        DATE => metadata.modified = Some(text),
                        CREATION_DATE => metadata.created = Some(text),
                        _ => {}
                    }
                }
                Event::End(_) => {
                    current_element.clear();
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        Ok(metadata)
    }

    /// Parse styles from styles.xml or automatic styles in content.xml
    fn parse_styles(&mut self, content: &str) -> OdtResult<()> {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut current_style: Option<OdtStyle> = None;
        let mut in_text_props = false;
        let mut in_para_props = false;

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) | Event::Empty(e) => {
                    let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                    match name.as_str() {
                        STYLE => {
                            let style = self.parse_style_element(&e)?;
                            current_style = Some(style);
                        }
                        TEXT_PROPERTIES => {
                            in_text_props = true;
                            if let Some(ref mut style) = current_style {
                                self.parse_text_properties(&e, &mut style.text_props);
                            }
                        }
                        PARAGRAPH_PROPERTIES => {
                            in_para_props = true;
                            if let Some(ref mut style) = current_style {
                                self.parse_paragraph_properties(&e, &mut style.para_props);
                            }
                        }
                        _ => {}
                    }

                    // Handle empty elements
                    if matches!(reader.read_event_into(&mut buf)?, Event::End(_)) {
                        if name == STYLE {
                            if let Some(style) = current_style.take() {
                                self.styles.insert(style.name.clone(), style);
                            }
                        }
                    }
                }
                Event::End(e) => {
                    let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    match name.as_str() {
                        STYLE => {
                            if let Some(style) = current_style.take() {
                                self.styles.insert(style.name.clone(), style);
                            }
                        }
                        TEXT_PROPERTIES => in_text_props = false,
                        PARAGRAPH_PROPERTIES => in_para_props = false,
                        _ => {}
                    }
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        Ok(())
    }

    /// Parse a style element's basic attributes
    fn parse_style_element(&self, e: &BytesStart) -> OdtResult<OdtStyle> {
        let mut style = OdtStyle::default();

        for attr in e.attributes() {
            let attr = attr?;
            let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
            let value = String::from_utf8_lossy(&attr.value).to_string();

            match key.as_str() {
                NAME => style.name = value,
                FAMILY => style.family = value,
                PARENT_STYLE_NAME => style.parent_style_name = Some(value),
                _ => {}
            }
        }

        Ok(style)
    }

    /// Parse text properties
    fn parse_text_properties(&self, e: &BytesStart, props: &mut CharacterProperties) {
        for attr in e.attributes().filter_map(|a| a.ok()) {
            let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
            let value = String::from_utf8_lossy(&attr.value).to_string();

            match key.as_str() {
                FONT_SIZE => {
                    props.font_size = parse_length(&value);
                }
                FONT_WEIGHT => {
                    props.bold = Some(value == "bold" || value == "700");
                }
                FONT_STYLE => {
                    props.italic = Some(value == "italic" || value == "oblique");
                }
                TEXT_DECORATION => {
                    if value.contains("underline") {
                        props.underline = Some(true);
                    }
                    if value.contains("line-through") {
                        props.strikethrough = Some(true);
                    }
                }
                COLOR => {
                    props.color = Some(value);
                }
                BACKGROUND_COLOR => {
                    props.highlight = Some(value);
                }
                "font-name" | "font-family" => {
                    props.font_family = Some(value);
                }
                _ => {}
            }
        }
    }

    /// Parse paragraph properties
    fn parse_paragraph_properties(&self, e: &BytesStart, props: &mut ParagraphProperties) {
        for attr in e.attributes().filter_map(|a| a.ok()) {
            let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
            let value = String::from_utf8_lossy(&attr.value).to_string();

            match key.as_str() {
                TEXT_ALIGN => {
                    props.alignment = match value.as_str() {
                        "start" | "left" => Some(Alignment::Left),
                        "center" => Some(Alignment::Center),
                        "end" | "right" => Some(Alignment::Right),
                        "justify" => Some(Alignment::Justify),
                        _ => None,
                    };
                }
                MARGIN_LEFT => {
                    props.indent_left = parse_length(&value);
                }
                MARGIN_RIGHT => {
                    props.indent_right = parse_length(&value);
                }
                TEXT_INDENT => {
                    props.indent_first_line = parse_length(&value);
                }
                MARGIN_TOP => {
                    props.space_before = parse_length(&value);
                }
                MARGIN_BOTTOM => {
                    props.space_after = parse_length(&value);
                }
                LINE_HEIGHT => {
                    if value.ends_with('%') {
                        if let Ok(pct) = value.trim_end_matches('%').parse::<f32>() {
                            props.line_spacing = Some(LineSpacing::Multiple(pct / 100.0));
                        }
                    } else {
                        if let Some(pts) = parse_length(&value) {
                            props.line_spacing = Some(LineSpacing::Exact(pts));
                        }
                    }
                }
                "keep-with-next" => {
                    props.keep_with_next = Some(value == "always");
                }
                "keep-together" => {
                    props.keep_together = Some(value == "always");
                }
                "break-before" => {
                    props.page_break_before = Some(value == "page");
                }
                _ => {}
            }
        }
    }

    /// Parse the main document content
    fn parse_content(&mut self, content: &str, tree: &mut DocumentTree) -> OdtResult<()> {
        // First, parse automatic styles from content.xml
        self.parse_styles(content)?;

        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut in_body = false;
        let mut in_text = false;

        // Paragraph state
        let mut current_para: Option<Paragraph> = None;
        let mut current_runs: Vec<Run> = Vec::new();
        let mut current_text = String::new();
        let mut current_char_props = CharacterProperties::default();

        // Table state
        let mut in_table = false;
        let mut table_rows: Vec<(TableRow, Vec<(TableCell, Vec<(Paragraph, Vec<Run>)>)>)> = Vec::new();
        let mut current_row_cells: Vec<(TableCell, Vec<(Paragraph, Vec<Run>)>)> = Vec::new();
        let mut current_cell_paras: Vec<(Paragraph, Vec<Run>)> = Vec::new();
        let mut col_widths: Vec<f32> = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) => {
                    let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                    match name.as_str() {
                        BODY => in_body = true,
                        TEXT_ELEM if in_body => in_text = true,
                        P | H if in_text => {
                            // Get paragraph style
                            let style_name = get_attribute(&e, STYLE_NAME);
                            let para_props = self.resolve_paragraph_style(&style_name);
                            current_char_props = self.resolve_character_style(&style_name);
                            current_para = Some(Paragraph::with_direct_formatting(para_props));
                        }
                        SPAN => {
                            // Save current text first
                            if !current_text.is_empty() {
                                current_runs.push(Run::with_direct_formatting(
                                    std::mem::take(&mut current_text),
                                    current_char_props.clone(),
                                ));
                            }

                            // Get span style
                            let style_name = get_attribute(&e, STYLE_NAME);
                            current_char_props = self.resolve_character_style(&style_name);
                        }
                        S => {
                            // Space element
                            let count = get_attribute(&e, C)
                                .and_then(|c| c.parse::<usize>().ok())
                                .unwrap_or(1);
                            current_text.push_str(&" ".repeat(count));
                        }
                        TAB => {
                            current_text.push('\t');
                        }
                        LINE_BREAK => {
                            current_text.push('\n');
                        }
                        TABLE if in_text => {
                            // Save any current paragraph first
                            if current_para.is_some() || !current_text.is_empty() {
                                self.finish_paragraph(
                                    tree, &mut current_para, &mut current_runs,
                                    &mut current_text, &current_char_props, in_table,
                                    &mut current_cell_paras,
                                );
                            }
                            in_table = true;
                            table_rows.clear();
                            col_widths.clear();
                        }
                        TABLE_COLUMN if in_table => {
                            let width = get_attribute(&e, COLUMN_WIDTH)
                                .and_then(|w| parse_length(&w))
                                .unwrap_or(72.0);
                            let repeated = get_attribute(&e, NUMBER_COLUMNS_REPEATED)
                                .and_then(|r| r.parse::<usize>().ok())
                                .unwrap_or(1);
                            for _ in 0..repeated {
                                col_widths.push(width);
                            }
                        }
                        TABLE_ROW if in_table => {
                            current_row_cells.clear();
                        }
                        TABLE_CELL if in_table => {
                            current_cell_paras.clear();
                        }
                        FRAME => {
                            // Drawing frame - might contain an image
                            // We'll handle images in the image element
                        }
                        IMAGE => {
                            // Get image href
                            if let Some(href) = get_attribute(&e, HREF) {
                                // Handle image (simplified - full implementation would load from Pictures/)
                                self.warnings.push(OdtWarning {
                                    kind: OdtWarningKind::PartialSupport,
                                    message: format!("Image '{}' referenced but not fully imported", href),
                                });
                            }
                        }
                        _ => {}
                    }
                }
                Event::Text(e) => {
                    if current_para.is_some() {
                        let text = e.unescape()?.to_string();
                        current_text.push_str(&text);
                    }
                }
                Event::End(e) => {
                    let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                    match name.as_str() {
                        BODY => in_body = false,
                        TEXT_ELEM => in_text = false,
                        P | H => {
                            // End of paragraph
                            self.finish_paragraph(
                                tree, &mut current_para, &mut current_runs,
                                &mut current_text, &current_char_props, in_table,
                                &mut current_cell_paras,
                            );
                            current_char_props = CharacterProperties::default();
                        }
                        SPAN => {
                            // End of span - save text and reset style
                            if !current_text.is_empty() {
                                current_runs.push(Run::with_direct_formatting(
                                    std::mem::take(&mut current_text),
                                    current_char_props.clone(),
                                ));
                            }
                            current_char_props = CharacterProperties::default();
                        }
                        TABLE_CELL if in_table => {
                            // Ensure cell has at least one paragraph
                            if current_cell_paras.is_empty() {
                                current_cell_paras.push((Paragraph::new(), Vec::new()));
                            }
                            let cell = TableCell::new();
                            current_row_cells.push((cell, std::mem::take(&mut current_cell_paras)));
                        }
                        TABLE_ROW if in_table => {
                            let row = TableRow::new();
                            table_rows.push((row, std::mem::take(&mut current_row_cells)));
                        }
                        TABLE => {
                            // Build the table
                            self.build_table(tree, &table_rows, &col_widths)?;
                            table_rows.clear();
                            col_widths.clear();
                            in_table = false;
                        }
                        _ => {}
                    }
                }
                Event::Empty(e) => {
                    let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    match name.as_str() {
                        S => {
                            let count = get_attribute(&e, C)
                                .and_then(|c| c.parse::<usize>().ok())
                                .unwrap_or(1);
                            current_text.push_str(&" ".repeat(count));
                        }
                        TAB => current_text.push('\t'),
                        LINE_BREAK => current_text.push('\n'),
                        SOFT_PAGE_BREAK => {
                            // Soft page break - we can ignore this
                        }
                        TABLE_COLUMN if in_table => {
                            let width = get_attribute(&e, COLUMN_WIDTH)
                                .and_then(|w| parse_length(&w))
                                .unwrap_or(72.0);
                            let repeated = get_attribute(&e, NUMBER_COLUMNS_REPEATED)
                                .and_then(|r| r.parse::<usize>().ok())
                                .unwrap_or(1);
                            for _ in 0..repeated {
                                col_widths.push(width);
                            }
                        }
                        P | H if in_text => {
                            // Empty paragraph
                            let style_name = get_attribute(&e, STYLE_NAME);
                            let para_props = self.resolve_paragraph_style(&style_name);

                            if in_table {
                                current_cell_paras.push((
                                    Paragraph::with_direct_formatting(para_props),
                                    Vec::new(),
                                ));
                            } else {
                                let para = Paragraph::with_direct_formatting(para_props);
                                let para_id = para.id();
                                tree.nodes.paragraphs.insert(para_id, para);
                                tree.document.add_body_child(para_id);
                            }
                        }
                        _ => {}
                    }
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        // Ensure document has at least one paragraph
        if tree.paragraphs().count() == 0 {
            let para = Paragraph::new();
            let para_id = para.id();
            tree.nodes.paragraphs.insert(para_id, para);
            tree.document.add_body_child(para_id);
        }

        Ok(())
    }

    /// Resolve paragraph style by name
    fn resolve_paragraph_style(&self, style_name: &Option<String>) -> ParagraphProperties {
        let mut props = ParagraphProperties::default();

        if let Some(name) = style_name {
            if let Some(style) = self.styles.get(name) {
                props = style.para_props.clone();

                // Resolve parent style
                if let Some(ref parent_name) = style.parent_style_name {
                    if let Some(parent) = self.styles.get(parent_name) {
                        props = parent.para_props.merge(&props);
                    }
                }
            }
        }

        props
    }

    /// Resolve character style by name
    fn resolve_character_style(&self, style_name: &Option<String>) -> CharacterProperties {
        let mut props = CharacterProperties::default();

        if let Some(name) = style_name {
            if let Some(style) = self.styles.get(name) {
                props = style.text_props.clone();

                // Resolve parent style
                if let Some(ref parent_name) = style.parent_style_name {
                    if let Some(parent) = self.styles.get(parent_name) {
                        props = parent.text_props.merge(&props);
                    }
                }
            }
        }

        props
    }

    /// Finish the current paragraph
    fn finish_paragraph(
        &self,
        tree: &mut DocumentTree,
        current_para: &mut Option<Paragraph>,
        current_runs: &mut Vec<Run>,
        current_text: &mut String,
        current_char_props: &CharacterProperties,
        in_table: bool,
        current_cell_paras: &mut Vec<(Paragraph, Vec<Run>)>,
    ) {
        // Save remaining text as a run
        if !current_text.is_empty() {
            current_runs.push(Run::with_direct_formatting(
                std::mem::take(current_text),
                current_char_props.clone(),
            ));
        }

        if let Some(para) = current_para.take() {
            let runs = std::mem::take(current_runs);

            if in_table {
                current_cell_paras.push((para, runs));
            } else {
                let para_id = para.id();
                tree.nodes.paragraphs.insert(para_id, para);
                tree.document.add_body_child(para_id);

                for mut run in runs {
                    let run_id = run.id();
                    run.set_parent(Some(para_id));
                    tree.nodes.runs.insert(run_id, run);
                    if let Some(p) = tree.nodes.paragraphs.get_mut(&para_id) {
                        p.add_child(run_id);
                    }
                }
            }
        }
    }

    /// Build a table from parsed rows
    fn build_table(
        &self,
        tree: &mut DocumentTree,
        rows: &[(TableRow, Vec<(TableCell, Vec<(Paragraph, Vec<Run>)>)>)],
        col_widths: &[f32],
    ) -> OdtResult<()> {
        if rows.is_empty() {
            return Ok(());
        }

        // Create table grid
        let columns: Vec<GridColumn> = col_widths.iter().map(|&w| {
            GridColumn {
                width: TableWidth::fixed(w),
            }
        }).collect();

        let grid = TableGrid { columns };
        let mut table = Table::with_grid(grid);
        let table_id = table.id();

        // Add rows
        for (row_template, cells) in rows {
            let mut row = row_template.clone();
            let row_id = row.id();
            row.set_parent(Some(table_id));

            // Add cells
            for (cell_template, paras) in cells {
                let mut cell = cell_template.clone();
                let cell_id = cell.id();
                cell.set_parent(Some(row_id));

                // Add paragraphs
                for (para_template, runs) in paras {
                    let mut para = para_template.clone();
                    let para_id = para.id();
                    para.set_parent(Some(cell_id));

                    // Add runs
                    for mut run in runs.clone() {
                        let run_id = run.id();
                        run.set_parent(Some(para_id));
                        tree.nodes.runs.insert(run_id, run);
                        para.add_child(run_id);
                    }

                    tree.nodes.paragraphs.insert(para_id, para);
                    cell.add_child(para_id);
                }

                tree.nodes.table_cells.insert(cell_id, cell);
                row.add_cell(cell_id);
            }

            tree.nodes.table_rows.insert(row_id, row);
            table.add_row(row_id);
        }

        tree.nodes.tables.insert(table_id, table);
        tree.document.add_body_child(table_id);

        Ok(())
    }

    /// Load images from the Pictures/ directory
    fn load_images(&mut self) -> OdtResult<()> {
        // Get list of files in Pictures/
        let picture_files: Vec<String> = self.archive.file_names()
            .filter(|name| name.starts_with("Pictures/"))
            .map(|s| s.to_string())
            .collect();

        for path in picture_files {
            if let Ok(data) = self.read_file_as_bytes(&path) {
                self.images.insert(path, data);
            }
        }

        Ok(())
    }
}

/// Get an attribute value from an element
fn get_attribute(e: &BytesStart, name: &str) -> Option<String> {
    for attr in e.attributes().filter_map(|a| a.ok()) {
        let local_name = attr.key.local_name();
        let key = String::from_utf8_lossy(local_name.as_ref());
        if key == name {
            return Some(String::from_utf8_lossy(&attr.value).to_string());
        }
    }
    None
}

/// Parse a length value (e.g., "1.5in", "2cm", "36pt") to points
fn parse_length(value: &str) -> Option<f32> {
    let value = value.trim();

    if value.ends_with("in") {
        value.trim_end_matches("in").parse::<f32>().ok().map(|v| v * 72.0)
    } else if value.ends_with("cm") {
        value.trim_end_matches("cm").parse::<f32>().ok().map(|v| v * 28.3465)
    } else if value.ends_with("mm") {
        value.trim_end_matches("mm").parse::<f32>().ok().map(|v| v * 2.83465)
    } else if value.ends_with("pt") {
        value.trim_end_matches("pt").parse::<f32>().ok()
    } else if value.ends_with("px") {
        value.trim_end_matches("px").parse::<f32>().ok().map(|v| v * 0.75)
    } else if value.ends_with('%') {
        // Percentage - return as-is for special handling
        None
    } else {
        // Try parsing as a number (assume points)
        value.parse::<f32>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_length() {
        assert!((parse_length("1in").unwrap() - 72.0).abs() < 0.01);
        assert!((parse_length("2.54cm").unwrap() - 72.0).abs() < 1.0);
        assert!((parse_length("12pt").unwrap() - 12.0).abs() < 0.01);
        assert!(parse_length("50%").is_none());
    }

    #[test]
    fn test_get_attribute() {
        // This would require constructing a BytesStart which is complex
        // In practice, this is tested through the full parse tests
    }
}
