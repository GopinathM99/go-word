# Test Suite

This folder contains manual test cases for validating the word processor functionality.

## Test Groups

| Group | Document | Coverage |
|-------|----------|----------|
| 1 | [Basic Editing](01-basic-editing.md) | Text input, cursor, selection, undo/redo |
| 2 | [Text Formatting](02-text-formatting.md) | Bold, italic, fonts, colors, alignment |
| 3 | [Paragraphs & Styles](03-paragraphs-styles.md) | Paragraph formatting, styles, headings |
| 4 | [Lists](04-lists.md) | Bullets, numbering, multi-level lists |
| 5 | [Tables](05-tables.md) | Table creation, editing, spanning, layout |
| 6 | [Images & Shapes](06-images-shapes.md) | Image insert, resize, shapes, text boxes |
| 7 | [Document Structure](07-document-structure.md) | Sections, headers/footers, page setup |
| 8 | [Fields & References](08-fields-references.md) | Page numbers, TOC, cross-references, bookmarks |
| 9 | [Track Changes & Comments](09-track-changes-comments.md) | Revisions, comments, review workflow |
| 10 | [Import Export](10-import-export.md) | DOCX, PDF, RTF, ODT |
| 11 | [Collaboration](11-collaboration.md) | Real-time editing, presence, sync |
| 12 | [Advanced Features](12-advanced-features.md) | Equations, charts, mail merge, content controls |
| 13 | [Performance](13-performance.md) | Large documents, stress tests |
| 14 | [Accessibility](14-accessibility.md) | Keyboard navigation, screen readers |

## How to Use

1. Start the application in development mode:
   ```bash
   cd src-tauri && cargo tauri dev
   ```

2. Open the test document for your feature group

3. Follow each test case step by step

4. Record results in the checkbox:
   - `[x]` = Pass
   - `[ ]` = Not tested
   - `[!]` = Fail (add notes)

## Test Status Legend

| Symbol | Meaning |
|--------|---------|
| `[ ]` | Not tested |
| `[x]` | Passed |
| `[!]` | Failed |
| `[~]` | Partial/Flaky |
| `[S]` | Skipped (N/A) |

## Running Automated Tests

```bash
# All tests
cargo test

# Specific crate
cargo test --package doc_model
cargo test --package edit_engine
cargo test --package layout_engine
cargo test --package collab
cargo test --package store
```

## Reporting Issues

When a test fails:
1. Note the exact steps to reproduce
2. Capture any error messages
3. Note expected vs actual behavior
4. Create an issue with the `[TEST]` prefix
