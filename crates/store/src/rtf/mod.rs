//! RTF Import/Export Module
//!
//! This module provides functionality to read and write Rich Text Format (RTF) files.
//! RTF is a document file format specification developed by Microsoft for cross-platform
//! document interchange.
//!
//! ## RTF Structure
//!
//! RTF files consist of:
//! - Control words (e.g., \b for bold, \i for italic)
//! - Control symbols (e.g., \\ for backslash)
//! - Groups (delimited by braces { })
//! - Text content
//!
//! ## Supported Features
//!
//! - Text formatting (bold, italic, underline, font size, font family)
//! - Paragraph formatting (alignment, indentation, spacing)
//! - Tables (basic support)
//! - Images (embedded pictures)
//! - Character encoding (ANSI, Unicode escapes)

mod error;
mod parser;
mod writer;
mod api;

pub use error::{RtfError, RtfResult};
pub use api::{import_rtf, export_rtf, import_rtf_bytes, export_rtf_bytes};
pub use api::{ImportResult, ImportWarning, WarningKind};

/// RTF control word constants
pub mod control_words {
    // Document structure
    pub const RTF: &str = "rtf";
    pub const ANSI: &str = "ansi";
    pub const DEFF: &str = "deff";
    pub const FONTTBL: &str = "fonttbl";
    pub const COLORTBL: &str = "colortbl";
    pub const STYLESHEET: &str = "stylesheet";
    pub const INFO: &str = "info";

    // Font table entries
    pub const F: &str = "f";
    pub const FNIL: &str = "fnil";
    pub const FROMAN: &str = "froman";
    pub const FSWISS: &str = "fswiss";
    pub const FMODERN: &str = "fmodern";
    pub const FSCRIPT: &str = "fscript";
    pub const FDECOR: &str = "fdecor";
    pub const FTECH: &str = "ftech";
    pub const FBIDI: &str = "fbidi";
    pub const FCHARSET: &str = "fcharset";
    pub const FPRQ: &str = "fprq";

    // Character formatting
    pub const B: &str = "b";
    pub const I: &str = "i";
    pub const UL: &str = "ul";
    pub const ULNONE: &str = "ulnone";
    pub const STRIKE: &str = "strike";
    pub const FS: &str = "fs";
    pub const CF: &str = "cf";
    pub const CB: &str = "cb";
    pub const SUPER: &str = "super";
    pub const SUB: &str = "sub";
    pub const NOSUPERSUB: &str = "nosupersub";
    pub const CAPS: &str = "caps";
    pub const SCAPS: &str = "scaps";
    pub const PLAIN: &str = "plain";

    // Paragraph formatting
    pub const PARD: &str = "pard";
    pub const PAR: &str = "par";
    pub const QL: &str = "ql";
    pub const QC: &str = "qc";
    pub const QR: &str = "qr";
    pub const QJ: &str = "qj";
    pub const LI: &str = "li";
    pub const RI: &str = "ri";
    pub const FI: &str = "fi";
    pub const SB: &str = "sb";
    pub const SA: &str = "sa";
    pub const SL: &str = "sl";
    pub const SLMULT: &str = "slmult";
    pub const KEEPN: &str = "keepn";
    pub const KEEP: &str = "keep";
    pub const PAGEBB: &str = "pagebb";

    // Table formatting
    pub const TROWD: &str = "trowd";
    pub const TRRH: &str = "trrh";
    pub const TRGAPH: &str = "trgaph";
    pub const TRLEFT: &str = "trleft";
    pub const CELLX: &str = "cellx";
    pub const CELL: &str = "cell";
    pub const ROW: &str = "row";
    pub const INTBL: &str = "intbl";
    pub const CLBRDRT: &str = "clbrdrt";
    pub const CLBRDRB: &str = "clbrdrb";
    pub const CLBRDRL: &str = "clbrdrl";
    pub const CLBRDRR: &str = "clbrdrr";
    pub const CLCBPAT: &str = "clcbpat";
    pub const CLVERTALT: &str = "clvertalt";
    pub const CLVERTALC: &str = "clvertalc";
    pub const CLVERTALB: &str = "clvertalb";
    pub const CLMGF: &str = "clmgf";
    pub const CLMRG: &str = "clmrg";
    pub const CLVMGF: &str = "clvmgf";
    pub const CLVMRG: &str = "clvmrg";

    // Image formatting
    pub const PICT: &str = "pict";
    pub const PNGBLIP: &str = "pngblip";
    pub const JPEGBLIP: &str = "jpegblip";
    pub const WMETAFILE: &str = "wmetafile";
    pub const DIBITMAP: &str = "dibitmap";
    pub const PICW: &str = "picw";
    pub const PICH: &str = "pich";
    pub const PICWGOAL: &str = "picwgoal";
    pub const PICHGOAL: &str = "pichgoal";
    pub const PICSCALEX: &str = "picscalex";
    pub const PICSCALEY: &str = "picscaley";

    // Special characters
    pub const LINE: &str = "line";
    pub const TAB: &str = "tab";
    pub const PAGE: &str = "page";
    pub const SECT: &str = "sect";

    // Unicode
    pub const U: &str = "u";
    pub const UC: &str = "uc";

    // Border styles
    pub const BRDRS: &str = "brdrs";
    pub const BRDRDOT: &str = "brdrdot";
    pub const BRDRDASH: &str = "brdrdash";
    pub const BRDRDB: &str = "brdrdb";
    pub const BRDRW: &str = "brdrw";
    pub const BRDRCF: &str = "brdrcf";
    pub const BRDRNONE: &str = "brdrnone";
}

/// Character set identifiers used in RTF font tables
pub mod charsets {
    pub const ANSI: u8 = 0;
    pub const DEFAULT: u8 = 1;
    pub const SYMBOL: u8 = 2;
    pub const MAC: u8 = 77;
    pub const SHIFTJIS: u8 = 128;
    pub const HANGUL: u8 = 129;
    pub const JOHAB: u8 = 130;
    pub const GB2312: u8 = 134;
    pub const CHINESEBIG5: u8 = 136;
    pub const GREEK: u8 = 161;
    pub const TURKISH: u8 = 162;
    pub const VIETNAMESE: u8 = 163;
    pub const HEBREW: u8 = 177;
    pub const ARABIC: u8 = 178;
    pub const BALTIC: u8 = 186;
    pub const RUSSIAN: u8 = 204;
    pub const THAI: u8 = 222;
    pub const EASTEUROPE: u8 = 238;
    pub const OEM: u8 = 255;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Basic test to verify module compiles
        assert_eq!(control_words::RTF, "rtf");
        assert_eq!(control_words::B, "b");
        assert_eq!(control_words::PARD, "pard");
    }
}
