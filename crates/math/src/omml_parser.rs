//! OMML Parser - Parse Office Math Markup Language from DOCX files
//!
//! This module parses OMML (Office Math Markup Language) XML into MathNode trees.
//! OMML is Microsoft's XML format for math in Office documents.

use crate::error::{MathError, MathResult};
use crate::model::*;
use quick_xml::events::Event;
use quick_xml::Reader;

/// Parse OMML XML from a string
pub fn parse_omml(xml: &str) -> MathResult<Vec<MathNode>> {
    let mut parser = OmmlParser::new(xml);
    parser.parse()
}

/// Parser for OMML XML content
pub struct OmmlParser<'a> {
    reader: Reader<&'a [u8]>,
}

impl<'a> OmmlParser<'a> {
    /// Create a new parser from XML string
    pub fn new(xml: &'a str) -> Self {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        Self { reader }
    }

    /// Parse the entire content and return MathNode trees
    pub fn parse(&mut self) -> MathResult<Vec<MathNode>> {
        let mut results = Vec::new();
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "oMathPara" => {
                            let node = self.parse_omath_para()?;
                            results.push(node);
                        }
                        "oMath" => {
                            let node = self.parse_omath()?;
                            results.push(node);
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(results)
    }

    /// Parse an oMathPara element
    fn parse_omath_para(&mut self) -> MathResult<MathNode> {
        let mut children = Vec::new();
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    if local_name == "oMath" {
                        children.push(self.parse_omath()?);
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "oMathPara" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::OMathPara(children))
    }

    /// Parse an oMath element
    fn parse_omath(&mut self) -> MathResult<MathNode> {
        let children = self.parse_math_children("oMath")?;
        Ok(MathNode::OMath(children))
    }

    /// Parse children of a math element until end tag
    fn parse_math_children(&mut self, end_tag: &str) -> MathResult<Vec<MathNode>> {
        let mut children = Vec::new();
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    if let Some(node) = self.parse_math_element(&local_name)? {
                        children.push(node);
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let _local_name = local_name_from_bytes(e.name().as_ref());
                    // Most empty elements are properties, not content
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == end_tag {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(children)
    }

    /// Parse a math element based on its tag name
    fn parse_math_element(&mut self, local_name: &str) -> MathResult<Option<MathNode>> {
        match local_name {
            "f" => Ok(Some(self.parse_fraction()?)),
            "rad" => Ok(Some(self.parse_radical()?)),
            "sSub" => Ok(Some(self.parse_subscript()?)),
            "sSup" => Ok(Some(self.parse_superscript()?)),
            "sSubSup" => Ok(Some(self.parse_sub_superscript()?)),
            "nary" => Ok(Some(self.parse_nary()?)),
            "d" => Ok(Some(self.parse_delimiter()?)),
            "m" => Ok(Some(self.parse_matrix()?)),
            "eqArr" => Ok(Some(self.parse_eq_array()?)),
            "box" => Ok(Some(self.parse_box()?)),
            "bar" => Ok(Some(self.parse_bar()?)),
            "acc" => Ok(Some(self.parse_accent()?)),
            "limLow" => Ok(Some(self.parse_limit_low()?)),
            "limUpp" => Ok(Some(self.parse_limit_upper()?)),
            "func" => Ok(Some(self.parse_function()?)),
            "groupChr" => Ok(Some(self.parse_group_char()?)),
            "borderBox" => Ok(Some(self.parse_border_box()?)),
            "phant" => Ok(Some(self.parse_phantom()?)),
            "r" => Ok(Some(self.parse_run()?)),
            _ => {
                // Skip unknown elements
                self.skip_element(local_name)?;
                Ok(None)
            }
        }
    }

    /// Parse a fraction (m:f)
    fn parse_fraction(&mut self) -> MathResult<MathNode> {
        let mut num = None;
        let mut den = None;
        let mut bar_visible = true;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "fPr" => {
                            bar_visible = self.parse_fraction_props()?;
                        }
                        "num" => {
                            let children = self.parse_math_children("num")?;
                            num = Some(wrap_children(children));
                        }
                        "den" => {
                            let children = self.parse_math_children("den")?;
                            den = Some(wrap_children(children));
                        }
                        _ => {
                            self.skip_element(&local_name)?;
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "f" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::Fraction {
            num: Box::new(num.unwrap_or_else(|| MathNode::run(""))),
            den: Box::new(den.unwrap_or_else(|| MathNode::run(""))),
            bar_visible,
        })
    }

    /// Parse fraction properties
    fn parse_fraction_props(&mut self) -> MathResult<bool> {
        let mut bar_visible = true;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    if local_name == "type" {
                        if let Some(val) = get_val_attr(e) {
                            bar_visible = val != "noBar";
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "fPr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(bar_visible)
    }

    /// Parse a radical (m:rad)
    fn parse_radical(&mut self) -> MathResult<MathNode> {
        let mut degree = None;
        let mut base = None;
        let mut hide_degree = false;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "radPr" => {
                            hide_degree = self.parse_radical_props()?;
                        }
                        "deg" => {
                            let children = self.parse_math_children("deg")?;
                            if !children.is_empty() {
                                degree = Some(wrap_children(children));
                            }
                        }
                        "e" => {
                            let children = self.parse_math_children("e")?;
                            base = Some(wrap_children(children));
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "rad" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::Radical {
            degree: if hide_degree { None } else { degree.map(Box::new) },
            base: Box::new(base.unwrap_or_else(|| MathNode::run(""))),
        })
    }

    /// Parse radical properties
    fn parse_radical_props(&mut self) -> MathResult<bool> {
        let mut hide_degree = false;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    if local_name == "degHide" {
                        if let Some(val) = get_val_attr(e) {
                            hide_degree = val == "1" || val == "true";
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "radPr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(hide_degree)
    }

    /// Parse a subscript (m:sSub)
    fn parse_subscript(&mut self) -> MathResult<MathNode> {
        let (base, sub, _) = self.parse_base_sub_sup("sSub")?;
        Ok(MathNode::Subscript {
            base: Box::new(base),
            sub: Box::new(sub.unwrap_or_else(|| MathNode::run(""))),
        })
    }

    /// Parse a superscript (m:sSup)
    fn parse_superscript(&mut self) -> MathResult<MathNode> {
        let (base, _, sup) = self.parse_base_sub_sup("sSup")?;
        Ok(MathNode::Superscript {
            base: Box::new(base),
            sup: Box::new(sup.unwrap_or_else(|| MathNode::run(""))),
        })
    }

    /// Parse combined sub/superscript (m:sSubSup)
    fn parse_sub_superscript(&mut self) -> MathResult<MathNode> {
        let (base, sub, sup) = self.parse_base_sub_sup("sSubSup")?;
        Ok(MathNode::SubSuperscript {
            base: Box::new(base),
            sub: Box::new(sub.unwrap_or_else(|| MathNode::run(""))),
            sup: Box::new(sup.unwrap_or_else(|| MathNode::run(""))),
        })
    }

    /// Parse base, sub, and sup elements common to subscript/superscript elements
    fn parse_base_sub_sup(
        &mut self,
        end_tag: &str,
    ) -> MathResult<(MathNode, Option<MathNode>, Option<MathNode>)> {
        let mut base = None;
        let mut sub = None;
        let mut sup = None;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "e" => {
                            let children = self.parse_math_children("e")?;
                            base = Some(wrap_children(children));
                        }
                        "sub" => {
                            let children = self.parse_math_children("sub")?;
                            sub = Some(wrap_children(children));
                        }
                        "sup" => {
                            let children = self.parse_math_children("sup")?;
                            sup = Some(wrap_children(children));
                        }
                        _ => {
                            self.skip_element(&local_name)?;
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == end_tag {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok((base.unwrap_or_else(|| MathNode::run("")), sub, sup))
    }

    /// Parse an n-ary operator (m:nary)
    fn parse_nary(&mut self) -> MathResult<MathNode> {
        let mut op = '\u{2211}'; // Default to sum
        let mut sub_sup_placement = SubSupPlacement::AboveBelow;
        let mut sub = None;
        let mut sup = None;
        let mut base = None;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "naryPr" => {
                            let (o, p) = self.parse_nary_props()?;
                            op = o;
                            sub_sup_placement = p;
                        }
                        "sub" => {
                            let children = self.parse_math_children("sub")?;
                            if !children.is_empty() {
                                sub = Some(wrap_children(children));
                            }
                        }
                        "sup" => {
                            let children = self.parse_math_children("sup")?;
                            if !children.is_empty() {
                                sup = Some(wrap_children(children));
                            }
                        }
                        "e" => {
                            let children = self.parse_math_children("e")?;
                            base = Some(wrap_children(children));
                        }
                        _ => {
                            self.skip_element(&local_name)?;
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "nary" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::Nary {
            op,
            sub_sup_placement,
            sub: sub.map(Box::new),
            sup: sup.map(Box::new),
            base: Box::new(base.unwrap_or_else(|| MathNode::run(""))),
        })
    }

    /// Parse n-ary properties
    fn parse_nary_props(&mut self) -> MathResult<(char, SubSupPlacement)> {
        let mut op = '\u{2211}';
        let mut placement = SubSupPlacement::AboveBelow;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "chr" => {
                            if let Some(val) = get_val_attr(e) {
                                if let Some(c) = val.chars().next() {
                                    op = c;
                                }
                            }
                        }
                        "limLoc" => {
                            if let Some(val) = get_val_attr(e) {
                                placement = if val == "subSup" {
                                    SubSupPlacement::Inline
                                } else {
                                    SubSupPlacement::AboveBelow
                                };
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "naryPr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok((op, placement))
    }

    /// Parse a delimiter (m:d)
    fn parse_delimiter(&mut self) -> MathResult<MathNode> {
        let mut open = '(';
        let mut close = ')';
        let mut separators = Vec::new();
        let mut content = Vec::new();
        let mut grow = true;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "dPr" => {
                            let (o, c, s, g) = self.parse_delimiter_props()?;
                            open = o;
                            close = c;
                            separators = s;
                            grow = g;
                        }
                        "e" => {
                            let children = self.parse_math_children("e")?;
                            content.push(wrap_children(children));
                        }
                        _ => {
                            self.skip_element(&local_name)?;
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "d" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::Delimiter {
            open,
            close,
            separators,
            content,
            grow,
        })
    }

    /// Parse delimiter properties
    fn parse_delimiter_props(&mut self) -> MathResult<(char, char, Vec<char>, bool)> {
        let mut open = '(';
        let mut close = ')';
        let mut separators = Vec::new();
        let mut grow = true;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "begChr" => {
                            if let Some(val) = get_val_attr(e) {
                                if let Some(c) = val.chars().next() {
                                    open = c;
                                }
                            }
                        }
                        "endChr" => {
                            if let Some(val) = get_val_attr(e) {
                                if let Some(c) = val.chars().next() {
                                    close = c;
                                }
                            }
                        }
                        "sepChr" => {
                            if let Some(val) = get_val_attr(e) {
                                separators = val.chars().collect();
                            }
                        }
                        "grow" => {
                            if let Some(val) = get_val_attr(e) {
                                grow = val != "0" && val != "false";
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "dPr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok((open, close, separators, grow))
    }

    /// Parse a matrix (m:m)
    fn parse_matrix(&mut self) -> MathResult<MathNode> {
        let mut rows = Vec::new();
        let mut row_spacing = 1.0f32;
        let mut col_spacing = 1.0f32;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "mPr" => {
                            let (rs, cs) = self.parse_matrix_props()?;
                            row_spacing = rs;
                            col_spacing = cs;
                        }
                        "mr" => {
                            let row = self.parse_matrix_row()?;
                            rows.push(row);
                        }
                        _ => {
                            self.skip_element(&local_name)?;
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "m" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::Matrix {
            rows,
            row_spacing,
            col_spacing,
        })
    }

    /// Parse matrix properties
    fn parse_matrix_props(&mut self) -> MathResult<(f32, f32)> {
        let mut row_spacing = 1.0f32;
        let mut col_spacing = 1.0f32;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "rSp" => {
                            if let Some(val) = get_val_attr(e) {
                                row_spacing = val.parse().unwrap_or(1.0);
                            }
                        }
                        "cSp" => {
                            if let Some(val) = get_val_attr(e) {
                                col_spacing = val.parse().unwrap_or(1.0);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "mPr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok((row_spacing, col_spacing))
    }

    /// Parse a matrix row
    fn parse_matrix_row(&mut self) -> MathResult<Vec<MathNode>> {
        let mut cells = Vec::new();
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    if local_name == "e" {
                        let children = self.parse_math_children("e")?;
                        cells.push(wrap_children(children));
                    } else {
                        self.skip_element(&local_name)?;
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "mr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(cells)
    }

    /// Parse an equation array (m:eqArr)
    fn parse_eq_array(&mut self) -> MathResult<MathNode> {
        let mut rows = Vec::new();
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    if local_name == "e" {
                        let children = self.parse_math_children("e")?;
                        rows.push(children);
                    } else {
                        self.skip_element(&local_name)?;
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "eqArr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::EqArray(rows))
    }

    /// Parse a box (m:box)
    fn parse_box(&mut self) -> MathResult<MathNode> {
        let mut base = None;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    if local_name == "e" {
                        let children = self.parse_math_children("e")?;
                        base = Some(wrap_children(children));
                    } else {
                        self.skip_element(&local_name)?;
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "box" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::Box(Box::new(base.unwrap_or_else(|| MathNode::run("")))))
    }

    /// Parse a bar (m:bar)
    fn parse_bar(&mut self) -> MathResult<MathNode> {
        let mut base = None;
        let mut position = BarPosition::Top;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "barPr" => {
                            position = self.parse_bar_props()?;
                        }
                        "e" => {
                            let children = self.parse_math_children("e")?;
                            base = Some(wrap_children(children));
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "bar" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::Bar {
            base: Box::new(base.unwrap_or_else(|| MathNode::run(""))),
            position,
        })
    }

    /// Parse bar properties
    fn parse_bar_props(&mut self) -> MathResult<BarPosition> {
        let mut position = BarPosition::Top;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    if local_name == "pos" {
                        if let Some(val) = get_val_attr(e) {
                            position = if val == "bot" {
                                BarPosition::Bottom
                            } else {
                                BarPosition::Top
                            };
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "barPr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(position)
    }

    /// Parse an accent (m:acc)
    fn parse_accent(&mut self) -> MathResult<MathNode> {
        let mut base = None;
        let mut accent_char = '\u{0302}'; // Default combining circumflex
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "accPr" => {
                            accent_char = self.parse_accent_props()?;
                        }
                        "e" => {
                            let children = self.parse_math_children("e")?;
                            base = Some(wrap_children(children));
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "acc" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::Accent {
            base: Box::new(base.unwrap_or_else(|| MathNode::run(""))),
            accent_char,
        })
    }

    /// Parse accent properties
    fn parse_accent_props(&mut self) -> MathResult<char> {
        let mut accent_char = '\u{0302}';
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    if local_name == "chr" {
                        if let Some(val) = get_val_attr(e) {
                            if let Some(c) = val.chars().next() {
                                accent_char = c;
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "accPr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(accent_char)
    }

    /// Parse lower limit (m:limLow)
    fn parse_limit_low(&mut self) -> MathResult<MathNode> {
        let mut func = None;
        let mut limit = None;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "e" => {
                            let children = self.parse_math_children("e")?;
                            func = Some(wrap_children(children));
                        }
                        "lim" => {
                            let children = self.parse_math_children("lim")?;
                            limit = Some(wrap_children(children));
                        }
                        _ => {
                            self.skip_element(&local_name)?;
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "limLow" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::Limit {
            func: Box::new(func.unwrap_or_else(|| MathNode::run(""))),
            limit: Box::new(limit.unwrap_or_else(|| MathNode::run(""))),
            position: LimitPosition::Lower,
        })
    }

    /// Parse upper limit (m:limUpp)
    fn parse_limit_upper(&mut self) -> MathResult<MathNode> {
        let mut func = None;
        let mut limit = None;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "e" => {
                            let children = self.parse_math_children("e")?;
                            func = Some(wrap_children(children));
                        }
                        "lim" => {
                            let children = self.parse_math_children("lim")?;
                            limit = Some(wrap_children(children));
                        }
                        _ => {
                            self.skip_element(&local_name)?;
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "limUpp" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::Limit {
            func: Box::new(func.unwrap_or_else(|| MathNode::run(""))),
            limit: Box::new(limit.unwrap_or_else(|| MathNode::run(""))),
            position: LimitPosition::Upper,
        })
    }

    /// Parse a function (m:func)
    fn parse_function(&mut self) -> MathResult<MathNode> {
        let mut name = String::new();
        let mut base = None;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "fName" => {
                            name = self.parse_function_name()?;
                        }
                        "e" => {
                            let children = self.parse_math_children("e")?;
                            base = Some(wrap_children(children));
                        }
                        _ => {
                            self.skip_element(&local_name)?;
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "func" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::Function {
            name,
            base: Box::new(base.unwrap_or_else(|| MathNode::run(""))),
        })
    }

    /// Parse function name
    fn parse_function_name(&mut self) -> MathResult<String> {
        let mut name = String::new();
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    if local_name == "r" {
                        if let MathNode::Run { text, .. } = self.parse_run()? {
                            name.push_str(&text);
                        }
                    } else {
                        self.skip_element(&local_name)?;
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "fName" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(name)
    }

    /// Parse a group character (m:groupChr)
    fn parse_group_char(&mut self) -> MathResult<MathNode> {
        let mut base = None;
        let mut chr = '\u{23DF}'; // Bottom curly bracket
        let mut position = BarPosition::Bottom;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "groupChrPr" => {
                            let (c, p) = self.parse_group_char_props()?;
                            chr = c;
                            position = p;
                        }
                        "e" => {
                            let children = self.parse_math_children("e")?;
                            base = Some(wrap_children(children));
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "groupChr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::GroupChar {
            base: Box::new(base.unwrap_or_else(|| MathNode::run(""))),
            chr,
            position,
        })
    }

    /// Parse group character properties
    fn parse_group_char_props(&mut self) -> MathResult<(char, BarPosition)> {
        let mut chr = '\u{23DF}';
        let mut position = BarPosition::Bottom;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "chr" => {
                            if let Some(val) = get_val_attr(e) {
                                if let Some(c) = val.chars().next() {
                                    chr = c;
                                }
                            }
                        }
                        "pos" => {
                            if let Some(val) = get_val_attr(e) {
                                position = if val == "top" {
                                    BarPosition::Top
                                } else {
                                    BarPosition::Bottom
                                };
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "groupChrPr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok((chr, position))
    }

    /// Parse a border box (m:borderBox)
    fn parse_border_box(&mut self) -> MathResult<MathNode> {
        let mut base = None;
        let mut hide_top = false;
        let mut hide_bottom = false;
        let mut hide_left = false;
        let mut hide_right = false;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "borderBoxPr" => {
                            let (t, b, l, r) = self.parse_border_box_props()?;
                            hide_top = t;
                            hide_bottom = b;
                            hide_left = l;
                            hide_right = r;
                        }
                        "e" => {
                            let children = self.parse_math_children("e")?;
                            base = Some(wrap_children(children));
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "borderBox" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::BorderBox {
            base: Box::new(base.unwrap_or_else(|| MathNode::run(""))),
            hide_top,
            hide_bottom,
            hide_left,
            hide_right,
        })
    }

    /// Parse border box properties
    fn parse_border_box_props(&mut self) -> MathResult<(bool, bool, bool, bool)> {
        let mut hide_top = false;
        let mut hide_bottom = false;
        let mut hide_left = false;
        let mut hide_right = false;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    let is_true = |e: &quick_xml::events::BytesStart<'_>| {
                        get_val_attr(e).map_or(false, |v| v == "1" || v == "true")
                    };
                    match local_name.as_str() {
                        "hideTop" => hide_top = is_true(e),
                        "hideBot" => hide_bottom = is_true(e),
                        "hideLeft" => hide_left = is_true(e),
                        "hideRight" => hide_right = is_true(e),
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "borderBoxPr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok((hide_top, hide_bottom, hide_left, hide_right))
    }

    /// Parse a phantom (m:phant)
    fn parse_phantom(&mut self) -> MathResult<MathNode> {
        let mut base = None;
        let mut zero_width = false;
        let mut zero_height = false;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "phantPr" => {
                            let (w, h) = self.parse_phantom_props()?;
                            zero_width = w;
                            zero_height = h;
                        }
                        "e" => {
                            let children = self.parse_math_children("e")?;
                            base = Some(wrap_children(children));
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "phant" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::Phantom {
            base: Box::new(base.unwrap_or_else(|| MathNode::run(""))),
            zero_width,
            zero_height,
        })
    }

    /// Parse phantom properties
    fn parse_phantom_props(&mut self) -> MathResult<(bool, bool)> {
        let mut zero_width = false;
        let mut zero_height = false;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    let is_true = |e: &quick_xml::events::BytesStart<'_>| {
                        get_val_attr(e).map_or(false, |v| v == "1" || v == "true")
                    };
                    match local_name.as_str() {
                        "zeroWid" => zero_width = is_true(e),
                        "zeroAsc" | "zeroDesc" => {
                            if is_true(e) {
                                zero_height = true;
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "phantPr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok((zero_width, zero_height))
    }

    /// Parse a run (m:r)
    fn parse_run(&mut self) -> MathResult<MathNode> {
        let mut text = String::new();
        let mut style = MathStyle::default();
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "rPr" => {
                            style = self.parse_run_props()?;
                        }
                        "t" => {
                            text = self.parse_text()?;
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "r" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(MathNode::Run { text, style })
    }

    /// Parse run properties
    fn parse_run_props(&mut self) -> MathResult<MathStyle> {
        let mut style = MathStyle::default();
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let local_name = local_name_from_bytes(e.name().as_ref());
                    match local_name.as_str() {
                        "sty" => {
                            if let Some(val) = get_val_attr(e) {
                                style.font_style = match val.as_str() {
                                    "p" => MathFontStyle::Normal,
                                    "b" => MathFontStyle::Bold,
                                    "i" => MathFontStyle::Italic,
                                    "bi" => MathFontStyle::BoldItalic,
                                    _ => MathFontStyle::Italic,
                                };
                            }
                        }
                        "lit" => {
                            if let Some(val) = get_val_attr(e) {
                                style.literal = val == "1" || val == "true";
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "rPr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(style)
    }

    /// Parse text content
    fn parse_text(&mut self) -> MathResult<String> {
        let mut text = String::new();
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Text(ref e)) => {
                    text.push_str(&e.unescape().map_err(|e| {
                        MathError::OmmlParse(format!("Failed to unescape text: {}", e))
                    })?);
                }
                Ok(Event::End(ref e)) => {
                    if local_name_from_bytes(e.name().as_ref()) == "t" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(text)
    }

    /// Skip an unknown element and all its children
    fn skip_element(&mut self, tag_name: &str) -> MathResult<()> {
        let mut depth = 1;
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match self.reader.read_event_into(&mut buf) {
                Ok(Event::Start(_)) => depth += 1,
                Ok(Event::End(ref e)) => {
                    depth -= 1;
                    if depth == 0 && local_name_from_bytes(e.name().as_ref()) == tag_name {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(MathError::Xml(e)),
                _ => {}
            }
        }

        Ok(())
    }
}

/// Get local name from bytes (without namespace prefix)
fn local_name_from_bytes(name: &[u8]) -> String {
    let name_str = String::from_utf8_lossy(name);
    if let Some(pos) = name_str.find(':') {
        name_str[pos + 1..].to_string()
    } else {
        name_str.to_string()
    }
}

/// Get the val attribute from an element
fn get_val_attr(e: &quick_xml::events::BytesStart<'_>) -> Option<String> {
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref());
        if key == "val" || key.ends_with(":val") || key == "m:val" {
            return Some(String::from_utf8_lossy(&attr.value).to_string());
        }
    }
    None
}

/// Wrap children in a single node if needed
fn wrap_children(children: Vec<MathNode>) -> MathNode {
    if children.len() == 1 {
        children.into_iter().next().unwrap()
    } else {
        MathNode::OMath(children)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_omath() {
        let xml = r#"<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
            <m:r><m:t>x</m:t></m:r>
        </m:oMath>"#;

        let result = parse_omml(xml).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], MathNode::OMath(_)));
    }

    #[test]
    fn test_parse_omath_para() {
        let xml = r#"<m:oMathPara xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
            <m:oMath>
                <m:r><m:t>y</m:t></m:r>
            </m:oMath>
        </m:oMathPara>"#;

        let result = parse_omml(xml).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], MathNode::OMathPara(_)));
    }

    #[test]
    fn test_parse_fraction() {
        let xml = r#"<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
            <m:f>
                <m:num><m:r><m:t>a</m:t></m:r></m:num>
                <m:den><m:r><m:t>b</m:t></m:r></m:den>
            </m:f>
        </m:oMath>"#;

        let result = parse_omml(xml).unwrap();
        if let MathNode::OMath(children) = &result[0] {
            assert!(matches!(children[0], MathNode::Fraction { .. }));
        } else {
            panic!("Expected OMath");
        }
    }

    #[test]
    fn test_parse_radical() {
        let xml = r#"<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
            <m:rad>
                <m:radPr><m:degHide m:val="1"/></m:radPr>
                <m:e><m:r><m:t>x</m:t></m:r></m:e>
            </m:rad>
        </m:oMath>"#;

        let result = parse_omml(xml).unwrap();
        if let MathNode::OMath(children) = &result[0] {
            if let MathNode::Radical { degree, .. } = &children[0] {
                assert!(degree.is_none());
            } else {
                panic!("Expected Radical");
            }
        } else {
            panic!("Expected OMath");
        }
    }

    #[test]
    fn test_parse_subscript() {
        let xml = r#"<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
            <m:sSub>
                <m:e><m:r><m:t>x</m:t></m:r></m:e>
                <m:sub><m:r><m:t>i</m:t></m:r></m:sub>
            </m:sSub>
        </m:oMath>"#;

        let result = parse_omml(xml).unwrap();
        if let MathNode::OMath(children) = &result[0] {
            assert!(matches!(children[0], MathNode::Subscript { .. }));
        } else {
            panic!("Expected OMath");
        }
    }

    #[test]
    fn test_parse_superscript() {
        let xml = r#"<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
            <m:sSup>
                <m:e><m:r><m:t>x</m:t></m:r></m:e>
                <m:sup><m:r><m:t>2</m:t></m:r></m:sup>
            </m:sSup>
        </m:oMath>"#;

        let result = parse_omml(xml).unwrap();
        if let MathNode::OMath(children) = &result[0] {
            assert!(matches!(children[0], MathNode::Superscript { .. }));
        } else {
            panic!("Expected OMath");
        }
    }

    #[test]
    fn test_parse_nary() {
        let xml = r#"<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
            <m:nary>
                <m:naryPr>
                    <m:chr m:val="∑"/>
                    <m:limLoc m:val="undOvr"/>
                </m:naryPr>
                <m:sub><m:r><m:t>i=0</m:t></m:r></m:sub>
                <m:sup><m:r><m:t>n</m:t></m:r></m:sup>
                <m:e><m:r><m:t>i</m:t></m:r></m:e>
            </m:nary>
        </m:oMath>"#;

        let result = parse_omml(xml).unwrap();
        if let MathNode::OMath(children) = &result[0] {
            if let MathNode::Nary { op, .. } = &children[0] {
                assert_eq!(*op, '∑');
            } else {
                panic!("Expected Nary");
            }
        } else {
            panic!("Expected OMath");
        }
    }

    #[test]
    fn test_parse_delimiter() {
        let xml = r#"<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
            <m:d>
                <m:dPr>
                    <m:begChr m:val="["/>
                    <m:endChr m:val="]"/>
                </m:dPr>
                <m:e><m:r><m:t>a</m:t></m:r></m:e>
            </m:d>
        </m:oMath>"#;

        let result = parse_omml(xml).unwrap();
        if let MathNode::OMath(children) = &result[0] {
            if let MathNode::Delimiter { open, close, .. } = &children[0] {
                assert_eq!(*open, '[');
                assert_eq!(*close, ']');
            } else {
                panic!("Expected Delimiter");
            }
        } else {
            panic!("Expected OMath");
        }
    }

    #[test]
    fn test_parse_matrix() {
        let xml = r#"<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
            <m:m>
                <m:mr>
                    <m:e><m:r><m:t>1</m:t></m:r></m:e>
                    <m:e><m:r><m:t>2</m:t></m:r></m:e>
                </m:mr>
                <m:mr>
                    <m:e><m:r><m:t>3</m:t></m:r></m:e>
                    <m:e><m:r><m:t>4</m:t></m:r></m:e>
                </m:mr>
            </m:m>
        </m:oMath>"#;

        let result = parse_omml(xml).unwrap();
        if let MathNode::OMath(children) = &result[0] {
            if let MathNode::Matrix { rows, .. } = &children[0] {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].len(), 2);
            } else {
                panic!("Expected Matrix");
            }
        } else {
            panic!("Expected OMath");
        }
    }

    #[test]
    fn test_parse_bar() {
        let xml = r#"<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
            <m:bar>
                <m:barPr><m:pos m:val="top"/></m:barPr>
                <m:e><m:r><m:t>x</m:t></m:r></m:e>
            </m:bar>
        </m:oMath>"#;

        let result = parse_omml(xml).unwrap();
        if let MathNode::OMath(children) = &result[0] {
            if let MathNode::Bar { position, .. } = &children[0] {
                assert_eq!(*position, BarPosition::Top);
            } else {
                panic!("Expected Bar");
            }
        } else {
            panic!("Expected OMath");
        }
    }

    #[test]
    fn test_parse_run_style() {
        let xml = r#"<m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
            <m:r>
                <m:rPr><m:sty m:val="b"/></m:rPr>
                <m:t>x</m:t>
            </m:r>
        </m:oMath>"#;

        let result = parse_omml(xml).unwrap();
        if let MathNode::OMath(children) = &result[0] {
            if let MathNode::Run { style, .. } = &children[0] {
                assert_eq!(style.font_style, MathFontStyle::Bold);
            } else {
                panic!("Expected Run");
            }
        } else {
            panic!("Expected OMath");
        }
    }
}
