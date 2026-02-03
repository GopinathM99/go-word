# Pagination Rules — Detailed Spec

## Page Geometry
- content box = page size minus margins.
- columns split content width equally unless custom widths set.

## Break Behavior
- Blocks that fit in remaining space stay on current page.
- If block does not fit:
  - If splittable: split at safe boundary.
  - If not splittable: move to next page.

## Splittable Blocks
- Paragraph: split at line boundary.
- Table: split at row boundary.
- List: split at item boundary.

## Keep Properties
- keepNext: keep paragraph with next block if possible.
- keepLines: keep all lines of paragraph together.
- widow/orphan: enforce min lines at top/bottom.

## Page Breaks
- Explicit page breaks always force new page.
- Section breaks force new section and may trigger new page.

## Headers and Footers
- Reserve header/footer space per section.
- Different first page and odd/even handled by section settings.

## Columns
- Column break forces next column.
- Balance columns if section requires balancing.

## Footnotes/Endnotes
- Footnotes occupy page footer area.
- If footnotes exceed remaining space, move lines to next page.

## Orphan/Widow Control
- Default min lines: 2 top and bottom.
- If violation occurs, shift line breaks to satisfy constraints.

## Pagination Stability
- Iteratively reflow from first changed block until page breaks stabilize.
