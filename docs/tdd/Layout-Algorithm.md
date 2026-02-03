# Line-Breaking and Pagination Algorithm

## Terminology
- Paragraph: block of inline runs.
- Inline: glyph runs with style metrics.
- Line: a row of inline boxes bounded by line width.
- Page: vertical container with columns.

## Inputs
- Paragraph runs with char props.
- Page layout: width, margins, columns.
- Font metrics: ascent, descent, advances.
- Hyphenation dictionary (optional).
- Language settings.

## Output
- For each paragraph: list of line boxes with glyph positions.
- For each page: list of block boxes.

## Algorithm Steps

### A. Normalize Runs
- Merge adjacent runs with identical style.
- Resolve style cascade to compute effective formatting.

### B. Shaping
- Shape runs into glyphs and advances.
- Store grapheme boundaries for cursor movement.

### C. Break Opportunities
- Compute break opportunities using Unicode Line Breaking Algorithm.
- Record soft break points (spaces, hyphenation points).

### D. Greedy Line Fill
- Fill line with glyphs until overflow.
- If overflow:
  - Break at last opportunity if available.
  - Else force break at last glyph.
- Track whitespace width for justification.

### E. Hyphenation (optional)
- If word overflows and hyphenation enabled:
  - Use dictionary to find break points.
  - Insert hyphen glyph + break.

### F. Line Metrics
- Line height = max(ascent + descent + leading) across runs.
- Baseline = max ascent.
- Align inline boxes to baseline.

### G. Paragraph Spacing
- Add spacing before/after paragraph.
- Apply line spacing (single/1.5/double/exact).

### H. Pagination
- For each block:
  - If block height > remaining page space:
    - Split if splittable (paragraph, table row).
    - Otherwise move to next page.
- For paragraphs: split at line boundary.
- For tables: paginate row by row; repeat header row if enabled.

### I. Widow/Orphan Control (later phase)
- Enforce min lines at top/bottom.
- Adjust breaks to avoid isolated lines.

## Incremental Reflow
- Cache line breaks per paragraph with versioning.
- On edit:
  - Mark paragraph dirty.
  - Reflow from that paragraph forward.
  - Stop when page breaks stabilize.

## Determinism
- Use deterministic rounding for glyph positions.
- Ensure consistent font fallback to reduce drift across platforms.
