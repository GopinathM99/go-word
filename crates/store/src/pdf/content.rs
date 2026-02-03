//! PDF Content Stream Generation
//!
//! This module provides a builder for PDF content streams, which contain
//! the graphics operators that define the visual appearance of a page.
//!
//! # Operators
//!
//! ## Text Operators
//! - BT/ET: Begin/End text object
//! - Tf: Set font and size
//! - Td: Move text position
//! - Tj: Show text string
//! - TJ: Show text with individual glyph positioning
//! - Tm: Set text matrix
//!
//! ## Graphics Operators
//! - m: Move to
//! - l: Line to
//! - c: Curve to (cubic Bezier)
//! - re: Rectangle
//! - S: Stroke path
//! - f: Fill path
//! - B: Fill and stroke path
//! - W: Set clipping path
//! - n: End path without filling or stroking
//!
//! ## Color Operators
//! - g/G: Set gray color (fill/stroke)
//! - rg/RG: Set RGB color (fill/stroke)
//! - k/K: Set CMYK color (fill/stroke)
//!
//! ## Transform Operators
//! - cm: Concatenate transformation matrix
//! - q: Save graphics state
//! - Q: Restore graphics state

use std::io::Write;

/// Content stream builder
#[derive(Debug, Default)]
pub struct ContentStream {
    /// The content data
    data: Vec<u8>,
    /// Current indentation level (for debugging)
    indent: usize,
}

impl ContentStream {
    /// Create a new empty content stream
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the content stream data
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    /// Get a reference to the content stream data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the length of the content stream
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the content stream is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    // =========================================================================
    // Graphics State Operators
    // =========================================================================

    /// Save the current graphics state (q)
    pub fn save_state(&mut self) -> &mut Self {
        self.write_line("q");
        self.indent += 1;
        self
    }

    /// Restore the graphics state (Q)
    pub fn restore_state(&mut self) -> &mut Self {
        self.indent = self.indent.saturating_sub(1);
        self.write_line("Q");
        self
    }

    /// Set the transformation matrix (cm)
    pub fn transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> &mut Self {
        self.write_fmt(format_args!(
            "{} {} {} {} {} {} cm\n",
            Self::fmt_num(a),
            Self::fmt_num(b),
            Self::fmt_num(c),
            Self::fmt_num(d),
            Self::fmt_num(e),
            Self::fmt_num(f)
        ));
        self
    }

    /// Set the line width (w)
    pub fn set_line_width(&mut self, width: f64) -> &mut Self {
        self.write_fmt(format_args!("{} w\n", Self::fmt_num(width)));
        self
    }

    /// Set the line cap style (J)
    pub fn set_line_cap(&mut self, cap: LineCap) -> &mut Self {
        self.write_fmt(format_args!("{} J\n", cap as i32));
        self
    }

    /// Set the line join style (j)
    pub fn set_line_join(&mut self, join: LineJoin) -> &mut Self {
        self.write_fmt(format_args!("{} j\n", join as i32));
        self
    }

    /// Set the miter limit (M)
    pub fn set_miter_limit(&mut self, limit: f64) -> &mut Self {
        self.write_fmt(format_args!("{} M\n", Self::fmt_num(limit)));
        self
    }

    /// Set the dash pattern (d)
    pub fn set_dash(&mut self, array: &[f64], phase: f64) -> &mut Self {
        self.data.push(b'[');
        for (i, &v) in array.iter().enumerate() {
            if i > 0 {
                self.data.push(b' ');
            }
            self.write_fmt(format_args!("{}", Self::fmt_num(v)));
        }
        self.write_fmt(format_args!("] {} d\n", Self::fmt_num(phase)));
        self
    }

    // =========================================================================
    // Color Operators
    // =========================================================================

    /// Set the fill color to grayscale (g)
    pub fn set_fill_gray(&mut self, gray: f64) -> &mut Self {
        self.write_fmt(format_args!("{} g\n", Self::fmt_num(gray)));
        self
    }

    /// Set the stroke color to grayscale (G)
    pub fn set_stroke_gray(&mut self, gray: f64) -> &mut Self {
        self.write_fmt(format_args!("{} G\n", Self::fmt_num(gray)));
        self
    }

    /// Set the fill color to RGB (rg)
    pub fn set_fill_rgb(&mut self, r: f64, g: f64, b: f64) -> &mut Self {
        self.write_fmt(format_args!(
            "{} {} {} rg\n",
            Self::fmt_num(r),
            Self::fmt_num(g),
            Self::fmt_num(b)
        ));
        self
    }

    /// Set the stroke color to RGB (RG)
    pub fn set_stroke_rgb(&mut self, r: f64, g: f64, b: f64) -> &mut Self {
        self.write_fmt(format_args!(
            "{} {} {} RG\n",
            Self::fmt_num(r),
            Self::fmt_num(g),
            Self::fmt_num(b)
        ));
        self
    }

    /// Set the fill color to CMYK (k)
    pub fn set_fill_cmyk(&mut self, c: f64, m: f64, y: f64, k: f64) -> &mut Self {
        self.write_fmt(format_args!(
            "{} {} {} {} k\n",
            Self::fmt_num(c),
            Self::fmt_num(m),
            Self::fmt_num(y),
            Self::fmt_num(k)
        ));
        self
    }

    /// Set the stroke color to CMYK (K)
    pub fn set_stroke_cmyk(&mut self, c: f64, m: f64, y: f64, k: f64) -> &mut Self {
        self.write_fmt(format_args!(
            "{} {} {} {} K\n",
            Self::fmt_num(c),
            Self::fmt_num(m),
            Self::fmt_num(y),
            Self::fmt_num(k)
        ));
        self
    }

    // =========================================================================
    // Path Construction Operators
    // =========================================================================

    /// Move to a point (m)
    pub fn move_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.write_fmt(format_args!(
            "{} {} m\n",
            Self::fmt_num(x),
            Self::fmt_num(y)
        ));
        self
    }

    /// Line to a point (l)
    pub fn line_to(&mut self, x: f64, y: f64) -> &mut Self {
        self.write_fmt(format_args!(
            "{} {} l\n",
            Self::fmt_num(x),
            Self::fmt_num(y)
        ));
        self
    }

    /// Cubic Bezier curve (c)
    pub fn curve_to(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
    ) -> &mut Self {
        self.write_fmt(format_args!(
            "{} {} {} {} {} {} c\n",
            Self::fmt_num(x1),
            Self::fmt_num(y1),
            Self::fmt_num(x2),
            Self::fmt_num(y2),
            Self::fmt_num(x3),
            Self::fmt_num(y3)
        ));
        self
    }

    /// Rectangle (re)
    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) -> &mut Self {
        self.write_fmt(format_args!(
            "{} {} {} {} re\n",
            Self::fmt_num(x),
            Self::fmt_num(y),
            Self::fmt_num(width),
            Self::fmt_num(height)
        ));
        self
    }

    /// Close the current subpath (h)
    pub fn close_path(&mut self) -> &mut Self {
        self.write_line("h");
        self
    }

    // =========================================================================
    // Path Painting Operators
    // =========================================================================

    /// Stroke the current path (S)
    pub fn stroke(&mut self) -> &mut Self {
        self.write_line("S");
        self
    }

    /// Close and stroke the current path (s)
    pub fn close_and_stroke(&mut self) -> &mut Self {
        self.write_line("s");
        self
    }

    /// Fill the current path using non-zero winding rule (f)
    pub fn fill(&mut self) -> &mut Self {
        self.write_line("f");
        self
    }

    /// Fill the current path using even-odd rule (f*)
    pub fn fill_even_odd(&mut self) -> &mut Self {
        self.write_line("f*");
        self
    }

    /// Fill and stroke the current path (B)
    pub fn fill_and_stroke(&mut self) -> &mut Self {
        self.write_line("B");
        self
    }

    /// Close, fill, and stroke the current path (b)
    pub fn close_fill_and_stroke(&mut self) -> &mut Self {
        self.write_line("b");
        self
    }

    /// End path without filling or stroking (n)
    pub fn end_path(&mut self) -> &mut Self {
        self.write_line("n");
        self
    }

    // =========================================================================
    // Clipping Operators
    // =========================================================================

    /// Set clipping path using non-zero winding rule (W)
    pub fn clip(&mut self) -> &mut Self {
        self.write_line("W");
        self
    }

    /// Set clipping path using even-odd rule (W*)
    pub fn clip_even_odd(&mut self) -> &mut Self {
        self.write_line("W*");
        self
    }

    // =========================================================================
    // Text Operators
    // =========================================================================

    /// Begin a text object (BT)
    pub fn begin_text(&mut self) -> &mut Self {
        self.write_line("BT");
        self.indent += 1;
        self
    }

    /// End a text object (ET)
    pub fn end_text(&mut self) -> &mut Self {
        self.indent = self.indent.saturating_sub(1);
        self.write_line("ET");
        self
    }

    /// Set the font and size (Tf)
    pub fn set_font(&mut self, font_name: &str, size: f64) -> &mut Self {
        self.write_fmt(format_args!("/{} {} Tf\n", font_name, Self::fmt_num(size)));
        self
    }

    /// Move text position (Td)
    pub fn move_text(&mut self, tx: f64, ty: f64) -> &mut Self {
        self.write_fmt(format_args!(
            "{} {} Td\n",
            Self::fmt_num(tx),
            Self::fmt_num(ty)
        ));
        self
    }

    /// Move to the next line (T*)
    pub fn next_line(&mut self) -> &mut Self {
        self.write_line("T*");
        self
    }

    /// Set the text matrix (Tm)
    pub fn set_text_matrix(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> &mut Self {
        self.write_fmt(format_args!(
            "{} {} {} {} {} {} Tm\n",
            Self::fmt_num(a),
            Self::fmt_num(b),
            Self::fmt_num(c),
            Self::fmt_num(d),
            Self::fmt_num(e),
            Self::fmt_num(f)
        ));
        self
    }

    /// Set the text leading (TL)
    pub fn set_text_leading(&mut self, leading: f64) -> &mut Self {
        self.write_fmt(format_args!("{} TL\n", Self::fmt_num(leading)));
        self
    }

    /// Set the character spacing (Tc)
    pub fn set_char_spacing(&mut self, spacing: f64) -> &mut Self {
        self.write_fmt(format_args!("{} Tc\n", Self::fmt_num(spacing)));
        self
    }

    /// Set the word spacing (Tw)
    pub fn set_word_spacing(&mut self, spacing: f64) -> &mut Self {
        self.write_fmt(format_args!("{} Tw\n", Self::fmt_num(spacing)));
        self
    }

    /// Set the text rise (Ts)
    pub fn set_text_rise(&mut self, rise: f64) -> &mut Self {
        self.write_fmt(format_args!("{} Ts\n", Self::fmt_num(rise)));
        self
    }

    /// Set the text rendering mode (Tr)
    pub fn set_text_rendering_mode(&mut self, mode: TextRenderingMode) -> &mut Self {
        self.write_fmt(format_args!("{} Tr\n", mode as i32));
        self
    }

    /// Show a text string (Tj)
    pub fn show_text(&mut self, text: &str) -> &mut Self {
        self.write_pdf_string(text);
        self.write_line(" Tj");
        self
    }

    /// Show text with individual glyph positioning (TJ)
    pub fn show_text_positioned(&mut self, elements: &[TextElement]) -> &mut Self {
        self.data.push(b'[');
        for element in elements {
            match element {
                TextElement::Text(s) => {
                    self.write_pdf_string(s);
                }
                TextElement::Adjustment(adj) => {
                    self.write_fmt(format_args!(" {} ", Self::fmt_num(*adj)));
                }
            }
        }
        self.write_line("] TJ");
        self
    }

    // =========================================================================
    // XObject Operators
    // =========================================================================

    /// Paint an XObject (Do)
    pub fn draw_xobject(&mut self, name: &str) -> &mut Self {
        self.write_fmt(format_args!("/{} Do\n", name));
        self
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    /// Write a line to the content stream
    fn write_line(&mut self, s: &str) {
        self.data.extend_from_slice(s.as_bytes());
        self.data.push(b'\n');
    }

    /// Write formatted data
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) {
        let _ = self.data.write_fmt(args);
    }

    /// Write a PDF string (escaped)
    fn write_pdf_string(&mut self, s: &str) {
        self.data.push(b'(');
        for byte in s.bytes() {
            match byte {
                b'(' | b')' | b'\\' => {
                    self.data.push(b'\\');
                    self.data.push(byte);
                }
                0x0A => {
                    self.data.extend_from_slice(b"\\n");
                }
                0x0D => {
                    self.data.extend_from_slice(b"\\r");
                }
                0x09 => {
                    self.data.extend_from_slice(b"\\t");
                }
                _ => {
                    self.data.push(byte);
                }
            }
        }
        self.data.push(b')');
    }

    /// Format a number for PDF output
    fn fmt_num(n: f64) -> String {
        if n.fract() == 0.0 {
            format!("{:.0}", n)
        } else {
            let s = format!("{:.4}", n);
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        }
    }
}

/// Line cap styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum LineCap {
    /// Butt cap
    Butt = 0,
    /// Round cap
    Round = 1,
    /// Projecting square cap
    Square = 2,
}

/// Line join styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum LineJoin {
    /// Miter join
    Miter = 0,
    /// Round join
    Round = 1,
    /// Bevel join
    Bevel = 2,
}

/// Text rendering modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum TextRenderingMode {
    /// Fill text
    Fill = 0,
    /// Stroke text
    Stroke = 1,
    /// Fill then stroke text
    FillStroke = 2,
    /// Invisible text
    Invisible = 3,
    /// Fill text and add to clipping path
    FillClip = 4,
    /// Stroke text and add to clipping path
    StrokeClip = 5,
    /// Fill, stroke, and add to clipping path
    FillStrokeClip = 6,
    /// Add to clipping path only
    Clip = 7,
}

/// Element for TJ operator
#[derive(Debug, Clone)]
pub enum TextElement {
    /// Text string
    Text(String),
    /// Positioning adjustment (in thousandths of text space unit)
    Adjustment(f64),
}

impl TextElement {
    /// Create a text element
    pub fn text(s: impl Into<String>) -> Self {
        TextElement::Text(s.into())
    }

    /// Create an adjustment element
    pub fn adjustment(adj: f64) -> Self {
        TextElement::Adjustment(adj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_stream_basic() {
        let mut cs = ContentStream::new();
        cs.save_state()
            .set_fill_rgb(1.0, 0.0, 0.0)
            .rect(100.0, 100.0, 200.0, 50.0)
            .fill()
            .restore_state();

        let content = String::from_utf8(cs.into_bytes()).unwrap();
        assert!(content.contains("q"));
        assert!(content.contains("1 0 0 rg"));
        assert!(content.contains("100 100 200 50 re"));
        assert!(content.contains("f"));
        assert!(content.contains("Q"));
    }

    #[test]
    fn test_content_stream_text() {
        let mut cs = ContentStream::new();
        cs.begin_text()
            .set_font("F1", 12.0)
            .move_text(72.0, 720.0)
            .show_text("Hello, World!")
            .end_text();

        let content = String::from_utf8(cs.into_bytes()).unwrap();
        assert!(content.contains("BT"));
        assert!(content.contains("/F1 12 Tf"));
        assert!(content.contains("72 720 Td"));
        assert!(content.contains("(Hello, World!) Tj"));
        assert!(content.contains("ET"));
    }

    #[test]
    fn test_content_stream_line() {
        let mut cs = ContentStream::new();
        cs.set_stroke_gray(0.5)
            .set_line_width(2.0)
            .move_to(0.0, 0.0)
            .line_to(100.0, 100.0)
            .stroke();

        let content = String::from_utf8(cs.into_bytes()).unwrap();
        assert!(content.contains("0.5 G"));
        assert!(content.contains("2 w"));
        assert!(content.contains("0 0 m"));
        assert!(content.contains("100 100 l"));
        assert!(content.contains("S"));
    }

    #[test]
    fn test_fmt_num() {
        assert_eq!(ContentStream::fmt_num(1.0), "1");
        assert_eq!(ContentStream::fmt_num(3.14159), "3.1416");
        assert_eq!(ContentStream::fmt_num(0.5), "0.5");
        assert_eq!(ContentStream::fmt_num(100.0), "100");
    }

    #[test]
    fn test_text_positioning() {
        let mut cs = ContentStream::new();
        cs.begin_text()
            .set_font("F1", 12.0)
            .set_text_matrix(1.0, 0.0, 0.0, 1.0, 72.0, 720.0)
            .show_text_positioned(&[
                TextElement::text("H"),
                TextElement::adjustment(-20.0),
                TextElement::text("ello"),
            ])
            .end_text();

        let content = String::from_utf8(cs.into_bytes()).unwrap();
        assert!(content.contains("1 0 0 1 72 720 Tm"));
        assert!(content.contains("[(H) -20 (ello)] TJ"));
    }
}
