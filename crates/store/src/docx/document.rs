//! Document.xml parser
//!
//! Parses the main document content including paragraphs, runs, and text.

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::reader::XmlParser;
use crate::docx::relationships::Relationships;
use doc_model::{
    Alignment, CharacterProperties, DocumentTree, Hyperlink, HyperlinkTarget, LineSpacing,
    Node, Paragraph, ParagraphProperties, Run, StyleId,
};
use quick_xml::events::Event;
use std::collections::HashMap;

/// Parser for document.xml
pub struct DocumentParser<'a> {
    /// Document relationships (for hyperlinks, images, etc.)
    doc_rels: &'a Relationships,
    /// External hyperlink targets by relationship ID
    hyperlinks: &'a HashMap<String, String>,
}

impl<'a> DocumentParser<'a> {
    /// Create a new document parser
    pub fn new(doc_rels: &'a Relationships, hyperlinks: &'a HashMap<String, String>) -> Self {
        Self { doc_rels, hyperlinks }
    }

    /// Parse document.xml and populate the DocumentTree
    pub fn parse(&self, content: &str, tree: &mut DocumentTree) -> DocxResult<()> {
        let mut reader = XmlParser::from_string(content);
        let mut buf = Vec::new();

        // Parse state
        let mut in_body = false;
        let mut current_para: Option<ParsedParagraph> = None;
        let mut current_run: Option<ParsedRun> = None;
        let mut current_hyperlink: Option<ParsedHyperlink> = None;
        let mut in_text = false;
        let mut in_para_props = false;
        let mut in_run_props = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "body") {
                        in_body = true;
                    } else if in_body && XmlParser::matches_element(name_ref, "p") {
                        current_para = Some(ParsedParagraph::new());
                    } else if current_para.is_some() && XmlParser::matches_element(name_ref, "pPr") {
                        in_para_props = true;
                    } else if current_para.is_some() && XmlParser::matches_element(name_ref, "r") {
                        current_run = Some(ParsedRun::new());
                    } else if current_run.is_some() && XmlParser::matches_element(name_ref, "rPr") {
                        in_run_props = true;
                    } else if current_run.is_some() && XmlParser::matches_element(name_ref, "t") {
                        in_text = true;
                    } else if current_para.is_some() && XmlParser::matches_element(name_ref, "hyperlink") {
                        // Start of hyperlink
                        let rel_id = XmlParser::get_r_attribute(e, "id");
                        let anchor = XmlParser::get_w_attribute(e, "anchor");
                        current_hyperlink = Some(ParsedHyperlink::new(rel_id, anchor));
                    } else if in_para_props {
                        self.parse_para_property(e, current_para.as_mut().unwrap())?;
                    } else if in_run_props {
                        self.parse_run_property(e, current_run.as_mut().unwrap())?;
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if in_para_props && current_para.is_some() {
                        self.parse_para_property(e, current_para.as_mut().unwrap())?;
                    } else if in_run_props && current_run.is_some() {
                        self.parse_run_property(e, current_run.as_mut().unwrap())?;
                    } else if current_run.is_some() && XmlParser::matches_element(name_ref, "br") {
                        // Line break
                        if let Some(ref mut run) = current_run {
                            run.text.push('\n');
                        }
                    } else if current_run.is_some() && XmlParser::matches_element(name_ref, "tab") {
                        // Tab character
                        if let Some(ref mut run) = current_run {
                            run.text.push('\t');
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let name_ref = name.as_ref();

                    if XmlParser::matches_element(name_ref, "body") {
                        in_body = false;
                    } else if XmlParser::matches_element(name_ref, "p") {
                        // End of paragraph - commit it
                        if let Some(parsed_para) = current_para.take() {
                            self.commit_paragraph(parsed_para, tree)?;
                        }
                    } else if XmlParser::matches_element(name_ref, "pPr") {
                        in_para_props = false;
                    } else if XmlParser::matches_element(name_ref, "r") {
                        // End of run - add it to paragraph or hyperlink
                        if let Some(parsed_run) = current_run.take() {
                            if let Some(ref mut hyperlink) = current_hyperlink {
                                hyperlink.runs.push(parsed_run);
                            } else if let Some(ref mut para) = current_para {
                                para.runs.push(parsed_run);
                            }
                        }
                    } else if XmlParser::matches_element(name_ref, "rPr") {
                        in_run_props = false;
                    } else if XmlParser::matches_element(name_ref, "t") {
                        in_text = false;
                    } else if XmlParser::matches_element(name_ref, "hyperlink") {
                        // End of hyperlink - add it to paragraph
                        if let Some(parsed_hyperlink) = current_hyperlink.take() {
                            if let Some(ref mut para) = current_para {
                                para.hyperlinks.push(parsed_hyperlink);
                            }
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_text {
                        if let Some(ref mut run) = current_run {
                            let text = e.unescape().map_err(|e| DocxError::XmlParse(e.to_string()))?;
                            run.text.push_str(&text);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::from(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(())
    }

    /// Parse a paragraph property element
    fn parse_para_property(&self, e: &quick_xml::events::BytesStart, para: &mut ParsedParagraph) -> DocxResult<()> {
        let name = e.name();
        let name_ref = name.as_ref();

        if XmlParser::matches_element(name_ref, "pStyle") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                para.style_id = Some(val);
            }
        } else if XmlParser::matches_element(name_ref, "jc") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                para.props.alignment = Some(parse_alignment(&val));
            }
        } else if XmlParser::matches_element(name_ref, "ind") {
            // Indentation
            if let Some(val) = XmlParser::get_w_attribute(e, "left") {
                para.props.indent_left = XmlParser::parse_twips(&val);
            }
            if let Some(val) = XmlParser::get_w_attribute(e, "right") {
                para.props.indent_right = XmlParser::parse_twips(&val);
            }
            if let Some(val) = XmlParser::get_w_attribute(e, "firstLine") {
                para.props.indent_first_line = XmlParser::parse_twips(&val);
            }
            if let Some(val) = XmlParser::get_w_attribute(e, "hanging") {
                // Hanging indent is negative first line
                if let Some(twips) = XmlParser::parse_twips(&val) {
                    para.props.indent_first_line = Some(-twips);
                }
            }
        } else if XmlParser::matches_element(name_ref, "spacing") {
            // Spacing
            if let Some(val) = XmlParser::get_w_attribute(e, "before") {
                para.props.space_before = XmlParser::parse_twips(&val);
            }
            if let Some(val) = XmlParser::get_w_attribute(e, "after") {
                para.props.space_after = XmlParser::parse_twips(&val);
            }
            if let Some(val) = XmlParser::get_w_attribute(e, "line") {
                let line_rule = XmlParser::get_w_attribute(e, "lineRule")
                    .unwrap_or_else(|| "auto".to_string());
                para.props.line_spacing = Some(parse_line_spacing(&val, &line_rule));
            }
        } else if XmlParser::matches_element(name_ref, "keepNext") {
            para.props.keep_with_next = Some(true);
        } else if XmlParser::matches_element(name_ref, "keepLines") {
            para.props.keep_together = Some(true);
        } else if XmlParser::matches_element(name_ref, "pageBreakBefore") {
            para.props.page_break_before = Some(true);
        }

        Ok(())
    }

    /// Parse a run property element
    fn parse_run_property(&self, e: &quick_xml::events::BytesStart, run: &mut ParsedRun) -> DocxResult<()> {
        let name = e.name();
        let name_ref = name.as_ref();

        if XmlParser::matches_element(name_ref, "rStyle") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                run.style_id = Some(val);
            }
        } else if XmlParser::matches_element(name_ref, "b") {
            let val = XmlParser::get_w_attribute(e, "val");
            run.props.bold = Some(val.map(|v| XmlParser::parse_bool(&v)).unwrap_or(true));
        } else if XmlParser::matches_element(name_ref, "i") {
            let val = XmlParser::get_w_attribute(e, "val");
            run.props.italic = Some(val.map(|v| XmlParser::parse_bool(&v)).unwrap_or(true));
        } else if XmlParser::matches_element(name_ref, "u") {
            let val = XmlParser::get_w_attribute(e, "val");
            run.props.underline = Some(val.map(|v| v != "none").unwrap_or(true));
        } else if XmlParser::matches_element(name_ref, "strike") {
            let val = XmlParser::get_w_attribute(e, "val");
            run.props.strikethrough = Some(val.map(|v| XmlParser::parse_bool(&v)).unwrap_or(true));
        } else if XmlParser::matches_element(name_ref, "sz") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                run.props.font_size = XmlParser::parse_half_points(&val);
            }
        } else if XmlParser::matches_element(name_ref, "rFonts") {
            // Font family - try ascii, then hAnsi, then others
            if let Some(font) = XmlParser::get_w_attribute(e, "ascii")
                .or_else(|| XmlParser::get_w_attribute(e, "hAnsi"))
                .or_else(|| XmlParser::get_w_attribute(e, "cs"))
            {
                run.props.font_family = Some(font);
            }
        } else if XmlParser::matches_element(name_ref, "color") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                if val != "auto" {
                    run.props.color = Some(format!("#{}", val));
                }
            }
        } else if XmlParser::matches_element(name_ref, "highlight") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                run.props.highlight = Some(highlight_to_color(&val));
            }
        }

        Ok(())
    }

    /// Commit a parsed paragraph to the tree
    fn commit_paragraph(&self, parsed: ParsedParagraph, tree: &mut DocumentTree) -> DocxResult<()> {
        // Create the paragraph
        let mut para = Paragraph::new();

        // Apply style
        if let Some(style_id) = &parsed.style_id {
            para.set_paragraph_style(Some(StyleId::new(style_id)));
        }

        // Apply direct formatting
        if !parsed.props.is_empty() {
            para.apply_direct_formatting(parsed.props);
        }

        let para_id = para.id();
        tree.nodes.paragraphs.insert(para_id, para);
        tree.document.add_body_child(para_id);

        // Add runs directly to paragraph
        for parsed_run in parsed.runs {
            self.commit_run(parsed_run, para_id, tree)?;
        }

        // Add hyperlinks with their runs
        for parsed_hyperlink in parsed.hyperlinks {
            self.commit_hyperlink(parsed_hyperlink, para_id, tree)?;
        }

        Ok(())
    }

    /// Commit a parsed run to the tree
    fn commit_run(&self, parsed: ParsedRun, parent_id: doc_model::NodeId, tree: &mut DocumentTree) -> DocxResult<()> {
        // Don't create empty runs
        if parsed.text.is_empty() {
            return Ok(());
        }

        let mut run = Run::new(&parsed.text);

        // Apply style
        if let Some(style_id) = &parsed.style_id {
            run.set_character_style(Some(StyleId::new(style_id)));
        }

        // Apply direct formatting
        if !parsed.props.is_empty() {
            run.apply_direct_formatting(parsed.props);
        }

        tree.insert_run(run, parent_id, None)?;
        Ok(())
    }

    /// Commit a parsed hyperlink to the tree
    fn commit_hyperlink(&self, parsed: ParsedHyperlink, para_id: doc_model::NodeId, tree: &mut DocumentTree) -> DocxResult<()> {
        // Determine the hyperlink target
        let target = if let Some(anchor) = &parsed.anchor {
            // Internal bookmark link
            HyperlinkTarget::internal(anchor)
        } else if let Some(rel_id) = &parsed.rel_id {
            // External link via relationship
            if let Some(url) = self.hyperlinks.get(rel_id) {
                if url.starts_with("mailto:") {
                    let email = url.trim_start_matches("mailto:");
                    let (address, subject) = if let Some(pos) = email.find("?subject=") {
                        (&email[..pos], Some(email[pos + 9..].to_string()))
                    } else {
                        (email, None)
                    };
                    HyperlinkTarget::email(address, subject)
                } else {
                    HyperlinkTarget::external(url)
                }
            } else {
                // Fallback - couldn't find relationship
                return Ok(());
            }
        } else {
            // No target - skip
            return Ok(());
        };

        let hyperlink = Hyperlink::new(target);
        let hyperlink_id = tree.insert_hyperlink(hyperlink, para_id, None)?;

        // Add runs to the hyperlink
        for parsed_run in parsed.runs {
            if !parsed_run.text.is_empty() {
                let mut run = Run::new(&parsed_run.text);
                if let Some(style_id) = &parsed_run.style_id {
                    run.set_character_style(Some(StyleId::new(style_id)));
                }
                if !parsed_run.props.is_empty() {
                    run.apply_direct_formatting(parsed_run.props);
                }
                tree.insert_run_into_hyperlink(run, hyperlink_id, None)?;
            }
        }

        Ok(())
    }
}

/// Parsed paragraph data (before committing to tree)
#[derive(Debug)]
struct ParsedParagraph {
    style_id: Option<String>,
    props: ParagraphProperties,
    runs: Vec<ParsedRun>,
    hyperlinks: Vec<ParsedHyperlink>,
}

impl ParsedParagraph {
    fn new() -> Self {
        Self {
            style_id: None,
            props: ParagraphProperties::default(),
            runs: Vec::new(),
            hyperlinks: Vec::new(),
        }
    }
}

/// Parsed run data
#[derive(Debug)]
struct ParsedRun {
    style_id: Option<String>,
    props: CharacterProperties,
    text: String,
}

impl ParsedRun {
    fn new() -> Self {
        Self {
            style_id: None,
            props: CharacterProperties::default(),
            text: String::new(),
        }
    }
}

/// Parsed hyperlink data
#[derive(Debug)]
struct ParsedHyperlink {
    rel_id: Option<String>,
    anchor: Option<String>,
    runs: Vec<ParsedRun>,
}

impl ParsedHyperlink {
    fn new(rel_id: Option<String>, anchor: Option<String>) -> Self {
        Self {
            rel_id,
            anchor,
            runs: Vec::new(),
        }
    }
}

/// Parse alignment value
fn parse_alignment(value: &str) -> Alignment {
    match value {
        "center" => Alignment::Center,
        "right" => Alignment::Right,
        "both" | "justify" => Alignment::Justify,
        _ => Alignment::Left,
    }
}

/// Parse line spacing value
fn parse_line_spacing(value: &str, line_rule: &str) -> LineSpacing {
    let val: f32 = value.parse().unwrap_or(240.0);

    match line_rule {
        "exact" => LineSpacing::Exact(val / 20.0), // Twips to points
        "atLeast" => LineSpacing::AtLeast(val / 20.0),
        _ => LineSpacing::Multiple(val / 240.0), // 240 twips = single line
    }
}

/// Convert highlight color name to CSS color
fn highlight_to_color(name: &str) -> String {
    match name {
        "yellow" => "#FFFF00",
        "green" => "#00FF00",
        "cyan" => "#00FFFF",
        "magenta" => "#FF00FF",
        "blue" => "#0000FF",
        "red" => "#FF0000",
        "darkBlue" => "#000080",
        "darkCyan" => "#008080",
        "darkGreen" => "#008000",
        "darkMagenta" => "#800080",
        "darkRed" => "#800000",
        "darkYellow" => "#808000",
        "darkGray" | "darkGrey" => "#808080",
        "lightGray" | "lightGrey" => "#C0C0C0",
        "black" => "#000000",
        "white" => "#FFFFFF",
        _ => "#FFFF00", // Default to yellow
    }.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_alignment() {
        assert_eq!(parse_alignment("left"), Alignment::Left);
        assert_eq!(parse_alignment("center"), Alignment::Center);
        assert_eq!(parse_alignment("right"), Alignment::Right);
        assert_eq!(parse_alignment("both"), Alignment::Justify);
        assert_eq!(parse_alignment("justify"), Alignment::Justify);
    }

    #[test]
    fn test_parse_line_spacing() {
        // Single spacing (240 twips)
        match parse_line_spacing("240", "auto") {
            LineSpacing::Multiple(m) => assert!((m - 1.0).abs() < 0.01),
            _ => panic!("Expected Multiple"),
        }

        // Double spacing
        match parse_line_spacing("480", "auto") {
            LineSpacing::Multiple(m) => assert!((m - 2.0).abs() < 0.01),
            _ => panic!("Expected Multiple"),
        }

        // Exact 12pt
        match parse_line_spacing("240", "exact") {
            LineSpacing::Exact(v) => assert!((v - 12.0).abs() < 0.01),
            _ => panic!("Expected Exact"),
        }
    }

    #[test]
    fn test_highlight_to_color() {
        assert_eq!(highlight_to_color("yellow"), "#FFFF00");
        assert_eq!(highlight_to_color("blue"), "#0000FF");
        assert_eq!(highlight_to_color("unknown"), "#FFFF00");
    }
}
