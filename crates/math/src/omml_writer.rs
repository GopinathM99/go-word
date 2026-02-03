//! OMML Writer - Serialize MathNode to Office Math Markup Language XML
//!
//! This module converts MathNode trees back to OMML XML for writing to DOCX files.

use crate::error::{MathError, MathResult};
use crate::model::*;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::io::Write;

/// OMML namespace URI
const MATH_NS_URI: &str = "http://schemas.openxmlformats.org/officeDocument/2006/math";
/// OMML namespace prefix
const MATH_NS: &str = "m";

/// Writer for converting MathNode to OMML XML
pub struct OmmlWriter<W: Write> {
    writer: Writer<W>,
}

impl<W: Write> OmmlWriter<W> {
    /// Create a new OMML writer
    pub fn new(inner: W) -> Self {
        Self {
            writer: Writer::new(inner),
        }
    }

    /// Write a MathNode to OMML XML
    pub fn write(&mut self, node: &MathNode) -> MathResult<()> {
        self.write_node(node)
    }

    /// Write a single node
    fn write_node(&mut self, node: &MathNode) -> MathResult<()> {
        match node {
            MathNode::OMath(children) => self.write_omath(children),
            MathNode::OMathPara(children) => self.write_omath_para(children),
            MathNode::Fraction { num, den, bar_visible } => {
                self.write_fraction(num, den, *bar_visible)
            }
            MathNode::Radical { degree, base } => self.write_radical(degree.as_deref(), base),
            MathNode::Subscript { base, sub } => self.write_subscript(base, sub),
            MathNode::Superscript { base, sup } => self.write_superscript(base, sup),
            MathNode::SubSuperscript { base, sub, sup } => {
                self.write_sub_superscript(base, sub, sup)
            }
            MathNode::Nary {
                op,
                sub_sup_placement,
                sub,
                sup,
                base,
            } => self.write_nary(*op, *sub_sup_placement, sub.as_deref(), sup.as_deref(), base),
            MathNode::Delimiter {
                open,
                close,
                separators,
                content,
                grow,
            } => self.write_delimiter(*open, *close, separators, content, *grow),
            MathNode::Matrix {
                rows,
                row_spacing,
                col_spacing,
            } => self.write_matrix(rows, *row_spacing, *col_spacing),
            MathNode::EqArray(rows) => self.write_eq_array(rows),
            MathNode::Box(base) => self.write_box(base),
            MathNode::Bar { base, position } => self.write_bar(base, *position),
            MathNode::Accent { base, accent_char } => self.write_accent(base, *accent_char),
            MathNode::Limit { func, limit, position } => self.write_limit(func, limit, *position),
            MathNode::Function { name, base } => self.write_function(name, base),
            MathNode::GroupChar { base, chr, position } => {
                self.write_group_char(base, *chr, *position)
            }
            MathNode::BorderBox {
                base,
                hide_top,
                hide_bottom,
                hide_left,
                hide_right,
            } => self.write_border_box(base, *hide_top, *hide_bottom, *hide_left, *hide_right),
            MathNode::Phantom {
                base,
                zero_width,
                zero_height,
            } => self.write_phantom(base, *zero_width, *zero_height),
            MathNode::Run { text, style } => self.write_run(text, style),
            MathNode::Operator { chr, form } => self.write_operator(*chr, *form),
            MathNode::Text(text) => self.write_text(text),
            MathNode::Number(num) => self.write_number(num),
            MathNode::Unknown { tag, content } => self.write_unknown(tag, content),
        }
    }

    /// Write oMath element
    fn write_omath(&mut self, children: &[MathNode]) -> MathResult<()> {
        let mut elem = BytesStart::new(format!("{}:oMath", MATH_NS));
        elem.push_attribute(("xmlns:m", MATH_NS_URI));
        self.writer
            .write_event(Event::Start(elem))
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;

        for child in children {
            self.write_node(child)?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new(format!("{}:oMath", MATH_NS))))
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;

        Ok(())
    }

    /// Write oMathPara element
    fn write_omath_para(&mut self, children: &[MathNode]) -> MathResult<()> {
        let mut elem = BytesStart::new(format!("{}:oMathPara", MATH_NS));
        elem.push_attribute(("xmlns:m", MATH_NS_URI));
        self.writer
            .write_event(Event::Start(elem))
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;

        for child in children {
            // Each child should be an oMath
            match child {
                MathNode::OMath(inner_children) => {
                    let omath_elem = BytesStart::new(format!("{}:oMath", MATH_NS));
                    self.writer
                        .write_event(Event::Start(omath_elem))
                        .map_err(|e| MathError::OmmlWrite(e.to_string()))?;

                    for inner in inner_children {
                        self.write_node(inner)?;
                    }

                    self.writer
                        .write_event(Event::End(BytesEnd::new(format!("{}:oMath", MATH_NS))))
                        .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
                }
                _ => {
                    // Wrap non-oMath in oMath
                    let omath_elem = BytesStart::new(format!("{}:oMath", MATH_NS));
                    self.writer
                        .write_event(Event::Start(omath_elem))
                        .map_err(|e| MathError::OmmlWrite(e.to_string()))?;

                    self.write_node(child)?;

                    self.writer
                        .write_event(Event::End(BytesEnd::new(format!("{}:oMath", MATH_NS))))
                        .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
                }
            }
        }

        self.writer
            .write_event(Event::End(BytesEnd::new(format!("{}:oMathPara", MATH_NS))))
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;

        Ok(())
    }

    /// Write fraction element
    fn write_fraction(
        &mut self,
        num: &MathNode,
        den: &MathNode,
        bar_visible: bool,
    ) -> MathResult<()> {
        self.start_element("f")?;

        // Write properties if bar is hidden
        if !bar_visible {
            self.start_element("fPr")?;
            let mut type_elem = BytesStart::new(format!("{}:type", MATH_NS));
            type_elem.push_attribute((format!("{}:val", MATH_NS).as_str(), "noBar"));
            self.writer
                .write_event(Event::Empty(type_elem))
                .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
            self.end_element("fPr")?;
        }

        // Write numerator
        self.start_element("num")?;
        self.write_node(num)?;
        self.end_element("num")?;

        // Write denominator
        self.start_element("den")?;
        self.write_node(den)?;
        self.end_element("den")?;

        self.end_element("f")?;
        Ok(())
    }

    /// Write radical element
    fn write_radical(&mut self, degree: Option<&MathNode>, base: &MathNode) -> MathResult<()> {
        self.start_element("rad")?;

        // Write properties
        self.start_element("radPr")?;
        if degree.is_none() {
            let mut deg_hide = BytesStart::new(format!("{}:degHide", MATH_NS));
            deg_hide.push_attribute((format!("{}:val", MATH_NS).as_str(), "1"));
            self.writer
                .write_event(Event::Empty(deg_hide))
                .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
        }
        self.end_element("radPr")?;

        // Write degree if present
        self.start_element("deg")?;
        if let Some(d) = degree {
            self.write_node(d)?;
        }
        self.end_element("deg")?;

        // Write base
        self.start_element("e")?;
        self.write_node(base)?;
        self.end_element("e")?;

        self.end_element("rad")?;
        Ok(())
    }

    /// Write subscript element
    fn write_subscript(&mut self, base: &MathNode, sub: &MathNode) -> MathResult<()> {
        self.start_element("sSub")?;

        self.start_element("e")?;
        self.write_node(base)?;
        self.end_element("e")?;

        self.start_element("sub")?;
        self.write_node(sub)?;
        self.end_element("sub")?;

        self.end_element("sSub")?;
        Ok(())
    }

    /// Write superscript element
    fn write_superscript(&mut self, base: &MathNode, sup: &MathNode) -> MathResult<()> {
        self.start_element("sSup")?;

        self.start_element("e")?;
        self.write_node(base)?;
        self.end_element("e")?;

        self.start_element("sup")?;
        self.write_node(sup)?;
        self.end_element("sup")?;

        self.end_element("sSup")?;
        Ok(())
    }

    /// Write combined sub/superscript element
    fn write_sub_superscript(
        &mut self,
        base: &MathNode,
        sub: &MathNode,
        sup: &MathNode,
    ) -> MathResult<()> {
        self.start_element("sSubSup")?;

        self.start_element("e")?;
        self.write_node(base)?;
        self.end_element("e")?;

        self.start_element("sub")?;
        self.write_node(sub)?;
        self.end_element("sub")?;

        self.start_element("sup")?;
        self.write_node(sup)?;
        self.end_element("sup")?;

        self.end_element("sSubSup")?;
        Ok(())
    }

    /// Write n-ary element
    fn write_nary(
        &mut self,
        op: char,
        sub_sup_placement: SubSupPlacement,
        sub: Option<&MathNode>,
        sup: Option<&MathNode>,
        base: &MathNode,
    ) -> MathResult<()> {
        self.start_element("nary")?;

        // Write properties
        self.start_element("naryPr")?;

        let mut chr_elem = BytesStart::new(format!("{}:chr", MATH_NS));
        chr_elem.push_attribute((format!("{}:val", MATH_NS).as_str(), op.to_string().as_str()));
        self.writer
            .write_event(Event::Empty(chr_elem))
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;

        let mut lim_loc = BytesStart::new(format!("{}:limLoc", MATH_NS));
        let loc_val = match sub_sup_placement {
            SubSupPlacement::Inline => "subSup",
            SubSupPlacement::AboveBelow => "undOvr",
        };
        lim_loc.push_attribute((format!("{}:val", MATH_NS).as_str(), loc_val));
        self.writer
            .write_event(Event::Empty(lim_loc))
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;

        self.end_element("naryPr")?;

        // Write sub
        self.start_element("sub")?;
        if let Some(s) = sub {
            self.write_node(s)?;
        }
        self.end_element("sub")?;

        // Write sup
        self.start_element("sup")?;
        if let Some(s) = sup {
            self.write_node(s)?;
        }
        self.end_element("sup")?;

        // Write base
        self.start_element("e")?;
        self.write_node(base)?;
        self.end_element("e")?;

        self.end_element("nary")?;
        Ok(())
    }

    /// Write delimiter element
    fn write_delimiter(
        &mut self,
        open: char,
        close: char,
        separators: &[char],
        content: &[MathNode],
        grow: bool,
    ) -> MathResult<()> {
        self.start_element("d")?;

        // Write properties
        self.start_element("dPr")?;

        let mut beg_chr = BytesStart::new(format!("{}:begChr", MATH_NS));
        beg_chr.push_attribute((format!("{}:val", MATH_NS).as_str(), open.to_string().as_str()));
        self.writer
            .write_event(Event::Empty(beg_chr))
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;

        let mut end_chr = BytesStart::new(format!("{}:endChr", MATH_NS));
        end_chr.push_attribute((format!("{}:val", MATH_NS).as_str(), close.to_string().as_str()));
        self.writer
            .write_event(Event::Empty(end_chr))
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;

        if !separators.is_empty() {
            let sep_str: String = separators.iter().collect();
            let mut sep_chr = BytesStart::new(format!("{}:sepChr", MATH_NS));
            sep_chr.push_attribute((format!("{}:val", MATH_NS).as_str(), sep_str.as_str()));
            self.writer
                .write_event(Event::Empty(sep_chr))
                .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
        }

        if !grow {
            let mut grow_elem = BytesStart::new(format!("{}:grow", MATH_NS));
            grow_elem.push_attribute((format!("{}:val", MATH_NS).as_str(), "0"));
            self.writer
                .write_event(Event::Empty(grow_elem))
                .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
        }

        self.end_element("dPr")?;

        // Write content elements
        for item in content {
            self.start_element("e")?;
            self.write_node(item)?;
            self.end_element("e")?;
        }

        self.end_element("d")?;
        Ok(())
    }

    /// Write matrix element
    fn write_matrix(
        &mut self,
        rows: &[Vec<MathNode>],
        _row_spacing: f32,
        _col_spacing: f32,
    ) -> MathResult<()> {
        self.start_element("m")?;

        for row in rows {
            self.start_element("mr")?;
            for cell in row {
                self.start_element("e")?;
                self.write_node(cell)?;
                self.end_element("e")?;
            }
            self.end_element("mr")?;
        }

        self.end_element("m")?;
        Ok(())
    }

    /// Write equation array element
    fn write_eq_array(&mut self, rows: &[Vec<MathNode>]) -> MathResult<()> {
        self.start_element("eqArr")?;

        for row in rows {
            self.start_element("e")?;
            for item in row {
                self.write_node(item)?;
            }
            self.end_element("e")?;
        }

        self.end_element("eqArr")?;
        Ok(())
    }

    /// Write box element
    fn write_box(&mut self, base: &MathNode) -> MathResult<()> {
        self.start_element("box")?;

        self.start_element("e")?;
        self.write_node(base)?;
        self.end_element("e")?;

        self.end_element("box")?;
        Ok(())
    }

    /// Write bar element
    fn write_bar(&mut self, base: &MathNode, position: BarPosition) -> MathResult<()> {
        self.start_element("bar")?;

        // Write properties
        self.start_element("barPr")?;
        let mut pos_elem = BytesStart::new(format!("{}:pos", MATH_NS));
        let pos_val = match position {
            BarPosition::Top => "top",
            BarPosition::Bottom => "bot",
        };
        pos_elem.push_attribute((format!("{}:val", MATH_NS).as_str(), pos_val));
        self.writer
            .write_event(Event::Empty(pos_elem))
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
        self.end_element("barPr")?;

        self.start_element("e")?;
        self.write_node(base)?;
        self.end_element("e")?;

        self.end_element("bar")?;
        Ok(())
    }

    /// Write accent element
    fn write_accent(&mut self, base: &MathNode, accent_char: char) -> MathResult<()> {
        self.start_element("acc")?;

        // Write properties
        self.start_element("accPr")?;
        let mut chr_elem = BytesStart::new(format!("{}:chr", MATH_NS));
        chr_elem.push_attribute((
            format!("{}:val", MATH_NS).as_str(),
            accent_char.to_string().as_str(),
        ));
        self.writer
            .write_event(Event::Empty(chr_elem))
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
        self.end_element("accPr")?;

        self.start_element("e")?;
        self.write_node(base)?;
        self.end_element("e")?;

        self.end_element("acc")?;
        Ok(())
    }

    /// Write limit element
    fn write_limit(
        &mut self,
        func: &MathNode,
        limit: &MathNode,
        position: LimitPosition,
    ) -> MathResult<()> {
        let tag = match position {
            LimitPosition::Lower => "limLow",
            LimitPosition::Upper => "limUpp",
        };

        self.start_element(tag)?;

        self.start_element("e")?;
        self.write_node(func)?;
        self.end_element("e")?;

        self.start_element("lim")?;
        self.write_node(limit)?;
        self.end_element("lim")?;

        self.end_element(tag)?;
        Ok(())
    }

    /// Write function element
    fn write_function(&mut self, name: &str, base: &MathNode) -> MathResult<()> {
        self.start_element("func")?;

        // Write function name
        self.start_element("fName")?;
        self.write_run(name, &MathStyle::normal())?;
        self.end_element("fName")?;

        self.start_element("e")?;
        self.write_node(base)?;
        self.end_element("e")?;

        self.end_element("func")?;
        Ok(())
    }

    /// Write group character element
    fn write_group_char(
        &mut self,
        base: &MathNode,
        chr: char,
        position: BarPosition,
    ) -> MathResult<()> {
        self.start_element("groupChr")?;

        // Write properties
        self.start_element("groupChrPr")?;

        let mut chr_elem = BytesStart::new(format!("{}:chr", MATH_NS));
        chr_elem.push_attribute((format!("{}:val", MATH_NS).as_str(), chr.to_string().as_str()));
        self.writer
            .write_event(Event::Empty(chr_elem))
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;

        let mut pos_elem = BytesStart::new(format!("{}:pos", MATH_NS));
        let pos_val = match position {
            BarPosition::Top => "top",
            BarPosition::Bottom => "bot",
        };
        pos_elem.push_attribute((format!("{}:val", MATH_NS).as_str(), pos_val));
        self.writer
            .write_event(Event::Empty(pos_elem))
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;

        self.end_element("groupChrPr")?;

        self.start_element("e")?;
        self.write_node(base)?;
        self.end_element("e")?;

        self.end_element("groupChr")?;
        Ok(())
    }

    /// Write border box element
    fn write_border_box(
        &mut self,
        base: &MathNode,
        hide_top: bool,
        hide_bottom: bool,
        hide_left: bool,
        hide_right: bool,
    ) -> MathResult<()> {
        self.start_element("borderBox")?;

        // Write properties if any borders are hidden
        if hide_top || hide_bottom || hide_left || hide_right {
            self.start_element("borderBoxPr")?;

            if hide_top {
                let mut elem = BytesStart::new(format!("{}:hideTop", MATH_NS));
                elem.push_attribute((format!("{}:val", MATH_NS).as_str(), "1"));
                self.writer
                    .write_event(Event::Empty(elem))
                    .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
            }
            if hide_bottom {
                let mut elem = BytesStart::new(format!("{}:hideBot", MATH_NS));
                elem.push_attribute((format!("{}:val", MATH_NS).as_str(), "1"));
                self.writer
                    .write_event(Event::Empty(elem))
                    .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
            }
            if hide_left {
                let mut elem = BytesStart::new(format!("{}:hideLeft", MATH_NS));
                elem.push_attribute((format!("{}:val", MATH_NS).as_str(), "1"));
                self.writer
                    .write_event(Event::Empty(elem))
                    .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
            }
            if hide_right {
                let mut elem = BytesStart::new(format!("{}:hideRight", MATH_NS));
                elem.push_attribute((format!("{}:val", MATH_NS).as_str(), "1"));
                self.writer
                    .write_event(Event::Empty(elem))
                    .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
            }

            self.end_element("borderBoxPr")?;
        }

        self.start_element("e")?;
        self.write_node(base)?;
        self.end_element("e")?;

        self.end_element("borderBox")?;
        Ok(())
    }

    /// Write phantom element
    fn write_phantom(
        &mut self,
        base: &MathNode,
        zero_width: bool,
        zero_height: bool,
    ) -> MathResult<()> {
        self.start_element("phant")?;

        if zero_width || zero_height {
            self.start_element("phantPr")?;

            if zero_width {
                let mut elem = BytesStart::new(format!("{}:zeroWid", MATH_NS));
                elem.push_attribute((format!("{}:val", MATH_NS).as_str(), "1"));
                self.writer
                    .write_event(Event::Empty(elem))
                    .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
            }
            if zero_height {
                let mut elem = BytesStart::new(format!("{}:zeroAsc", MATH_NS));
                elem.push_attribute((format!("{}:val", MATH_NS).as_str(), "1"));
                self.writer
                    .write_event(Event::Empty(elem))
                    .map_err(|e| MathError::OmmlWrite(e.to_string()))?;

                let mut elem = BytesStart::new(format!("{}:zeroDesc", MATH_NS));
                elem.push_attribute((format!("{}:val", MATH_NS).as_str(), "1"));
                self.writer
                    .write_event(Event::Empty(elem))
                    .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
            }

            self.end_element("phantPr")?;
        }

        self.start_element("e")?;
        self.write_node(base)?;
        self.end_element("e")?;

        self.end_element("phant")?;
        Ok(())
    }

    /// Write run element
    fn write_run(&mut self, text: &str, style: &MathStyle) -> MathResult<()> {
        self.start_element("r")?;

        // Write run properties if non-default
        let needs_props = style.font_style != MathFontStyle::Italic || style.literal;
        if needs_props {
            self.start_element("rPr")?;

            if style.font_style != MathFontStyle::Italic {
                let mut sty_elem = BytesStart::new(format!("{}:sty", MATH_NS));
                let sty_val = match style.font_style {
                    MathFontStyle::Normal => "p",
                    MathFontStyle::Bold => "b",
                    MathFontStyle::Italic => "i",
                    MathFontStyle::BoldItalic => "bi",
                    _ => "i", // Default to italic for other styles
                };
                sty_elem.push_attribute((format!("{}:val", MATH_NS).as_str(), sty_val));
                self.writer
                    .write_event(Event::Empty(sty_elem))
                    .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
            }

            if style.literal {
                let mut lit_elem = BytesStart::new(format!("{}:lit", MATH_NS));
                lit_elem.push_attribute((format!("{}:val", MATH_NS).as_str(), "1"));
                self.writer
                    .write_event(Event::Empty(lit_elem))
                    .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
            }

            self.end_element("rPr")?;
        }

        // Write text
        self.start_element("t")?;
        self.writer
            .write_event(Event::Text(BytesText::new(text)))
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
        self.end_element("t")?;

        self.end_element("r")?;
        Ok(())
    }

    /// Write operator
    fn write_operator(&mut self, chr: char, _form: OperatorForm) -> MathResult<()> {
        // Operators are written as runs with the character
        self.write_run(&chr.to_string(), &MathStyle::normal())
    }

    /// Write plain text
    fn write_text(&mut self, text: &str) -> MathResult<()> {
        self.write_run(text, &MathStyle::normal())
    }

    /// Write number
    fn write_number(&mut self, num: &str) -> MathResult<()> {
        self.write_run(num, &MathStyle::normal())
    }

    /// Write unknown/preserved XML
    fn write_unknown(&mut self, _tag: &str, content: &str) -> MathResult<()> {
        // Write raw content - this preserves unknown elements for round-trip
        self.writer
            .get_mut()
            .write_all(content.as_bytes())
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
        Ok(())
    }

    /// Helper to start an element with the math namespace
    fn start_element(&mut self, name: &str) -> MathResult<()> {
        let elem = BytesStart::new(format!("{}:{}", MATH_NS, name));
        self.writer
            .write_event(Event::Start(elem))
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
        Ok(())
    }

    /// Helper to end an element
    fn end_element(&mut self, name: &str) -> MathResult<()> {
        self.writer
            .write_event(Event::End(BytesEnd::new(format!("{}:{}", MATH_NS, name))))
            .map_err(|e| MathError::OmmlWrite(e.to_string()))?;
        Ok(())
    }
}

/// Convert a MathNode to OMML XML string
pub fn to_omml(node: &MathNode) -> MathResult<String> {
    let mut buffer = Vec::new();
    {
        let mut writer = OmmlWriter::new(&mut buffer);
        writer.write(node)?;
    }
    String::from_utf8(buffer).map_err(|e| MathError::OmmlWrite(e.to_string()))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::omml_parser::parse_omml;

    #[test]
    fn test_write_simple_run() {
        let node = MathNode::omath(vec![MathNode::run("x")]);
        let xml = to_omml(&node).unwrap();
        assert!(xml.contains("m:oMath"));
        assert!(xml.contains("m:r"));
        assert!(xml.contains("m:t"));
        assert!(xml.contains(">x<"));
    }

    #[test]
    fn test_write_fraction() {
        let node = MathNode::omath(vec![MathNode::fraction(
            MathNode::run("a"),
            MathNode::run("b"),
        )]);
        let xml = to_omml(&node).unwrap();
        assert!(xml.contains("m:f"));
        assert!(xml.contains("m:num"));
        assert!(xml.contains("m:den"));
    }

    #[test]
    fn test_write_radical() {
        let node = MathNode::omath(vec![MathNode::sqrt(MathNode::run("x"))]);
        let xml = to_omml(&node).unwrap();
        assert!(xml.contains("m:rad"));
        assert!(xml.contains("m:degHide"));
    }

    #[test]
    fn test_write_subscript() {
        let node = MathNode::omath(vec![MathNode::subscript(
            MathNode::run("x"),
            MathNode::number("2"),
        )]);
        let xml = to_omml(&node).unwrap();
        assert!(xml.contains("m:sSub"));
        assert!(xml.contains("m:sub"));
    }

    #[test]
    fn test_write_superscript() {
        let node = MathNode::omath(vec![MathNode::superscript(
            MathNode::run("x"),
            MathNode::number("2"),
        )]);
        let xml = to_omml(&node).unwrap();
        assert!(xml.contains("m:sSup"));
        assert!(xml.contains("m:sup"));
    }

    #[test]
    fn test_write_delimiter() {
        let node = MathNode::omath(vec![MathNode::parens(vec![MathNode::run("x")])]);
        let xml = to_omml(&node).unwrap();
        assert!(xml.contains("m:d"));
        assert!(xml.contains("m:dPr"));
        assert!(xml.contains("m:begChr"));
    }

    #[test]
    fn test_write_matrix() {
        let node = MathNode::omath(vec![MathNode::matrix(vec![
            vec![MathNode::number("1"), MathNode::number("2")],
            vec![MathNode::number("3"), MathNode::number("4")],
        ])]);
        let xml = to_omml(&node).unwrap();
        assert!(xml.contains("m:m"));
        assert!(xml.contains("m:mr"));
    }

    #[test]
    fn test_roundtrip_fraction() {
        let original = MathNode::omath(vec![MathNode::fraction(
            MathNode::run("1"),
            MathNode::run("2"),
        )]);
        let xml = to_omml(&original).unwrap();
        let parsed = parse_omml(&xml).unwrap();
        assert_eq!(parsed.len(), 1);

        // Verify structure
        if let MathNode::OMath(children) = &parsed[0] {
            assert!(matches!(children[0], MathNode::Fraction { .. }));
        } else {
            panic!("Expected OMath");
        }
    }

    #[test]
    fn test_roundtrip_nested() {
        let original = MathNode::omath(vec![MathNode::fraction(
            MathNode::superscript(MathNode::run("x"), MathNode::number("2")),
            MathNode::sqrt(MathNode::run("y")),
        )]);
        let xml = to_omml(&original).unwrap();
        let parsed = parse_omml(&xml).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn test_write_bar() {
        let node = MathNode::omath(vec![MathNode::overline(MathNode::run("x"))]);
        let xml = to_omml(&node).unwrap();
        assert!(xml.contains("m:bar"));
        assert!(xml.contains("m:barPr"));
        assert!(xml.contains("m:pos"));
    }
}
