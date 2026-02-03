# Paste and Import Heuristics

## HTML Paste
- Preserve semantic tags (<p>, <h1>, <ul>, <table>).
- Convert inline styles to direct formatting only if no style match.
- Collapse nested spans with same style.

## Google Docs Paste
- Detect google-docs-specific classes.
- Normalize to Word-like styles where possible.

## Word Paste
- Prefer RTF if available.
- Preserve lists and tables.

## Plain Text Paste
- Split paragraphs on \n.
- Preserve whitespace runs if user option enabled.

## Image Paste
- Inline image inserted at caret.
- Preserve original binary.

## Sanitization
- Strip script tags and external CSS references.
- Remove unknown attributes.
