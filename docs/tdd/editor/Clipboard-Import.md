# Clipboard Import (HTML/RTF)

## Supported Clipboard Types
- text/plain
- text/html
- text/rtf
- image/*

## Priority
1) HTML (if present)
2) RTF
3) Plain text

## HTML Parsing
- Parse into DOM -> internal model mapping:
  - <p> -> Paragraph
  - <b>/<strong> -> Run.bold
  - <i>/<em> -> Run.italic
  - <ul>/<ol> -> List
  - <table> -> Table
  - <img> -> Image

## RTF Parsing
- Use existing RTF parser or simple converter.
- Map common tags to styles.

## Plain Text
- Split on \n into paragraphs.

## Security
- Strip script tags and external references in HTML.
- Sanitize styles to avoid injection.
