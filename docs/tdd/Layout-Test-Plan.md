# Layout Test Plan — Golden Document Corpus

## Goals
- Validate layout fidelity across platforms and renderers.
- Detect regressions in pagination and line-breaking.
- Measure pixel-level differences against reference outputs.

## Test Types
1) Golden PDF comparison (pixel diff)
2) Structural layout comparison (page/line/box geometry)
3) Import/export round-trip validation

## Golden Corpus Design

### Corpus Categories
1. **Typography Basics**
   - Mixed fonts, sizes, bold/italic/underline
   - Character spacing, kerning, ligatures

2. **Paragraph Formatting**
   - Alignment (left/center/right/justify)
   - Indents (first-line, hanging)
   - Line spacing (single/1.5/double/exact)
   - Spacing before/after

3. **Lists and Numbering**
   - Bulleted and numbered lists
   - Multi-level lists with custom formats
   - Restart and continuation

4. **Tables**
   - Simple grids, merged cells
   - Fixed vs auto-fit widths
   - Borders/shading
   - Header row repeat

5. **Images and Shapes**
   - Inline images
   - Floating images with wrap
   - Layered shapes and transparency

6. **Sections and Pagination**
   - Page breaks
   - Section breaks with different margins
   - Columns and column breaks

7. **Headers/Footers**
   - First page, odd/even headers
   - Page numbers, fields

8. **Footnotes/Endnotes**
   - Multiple notes per page
   - Continuation across pages

9. **Fields and References**
   - TOC, page numbers, date fields
   - Cross-references

10. **Complex Layout**
    - Mixed content: tables + images + lists
    - Large documents (100+ pages)

### Corpus Size Targets
- 40-60 documents total.
- Minimum 5 documents per category.
- Include 5 large stress documents.

## Reference Outputs
- Generate reference PDFs using Microsoft Word.
- Record Word version and OS for each reference.

## Test Execution
1. Import DOCX into editor.
2. Export to PDF.
3. Run pixel diff vs reference PDF.
4. Collect metrics:
   - % pixel differences
   - max bounding box drift
   - page count differences

## Acceptance Thresholds
- Typographic/layout tests: <= 1% pixel diff for basic docs.
- Complex docs: <= 3% pixel diff.
- Page count must match in all cases.

## Automation Plan
- Use headless PDF render pipeline.
- Store diffs as artifacts for review.
- Gate CI on regression thresholds.

## Failure Triage
- Categorize failures by layout subsystem (line breaking, table layout, pagination).
- Track regressions with link to offending commit.

## Corpus Management
- Store corpus in versioned repo with metadata.
- Include expected output PDF and baseline checksum.
- Maintain a changelog for corpus updates.
