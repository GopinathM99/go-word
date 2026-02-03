# Table Layout — Detailed Spec

## Layout Modes
- Fixed: column widths set explicitly.
- Auto-fit: widths based on content + available space.

## Grid Construction
1. Read w:tblGrid or internal column definitions.
2. Resolve grid spans into a cell matrix.
3. Expand for merged cells (rowspan/colspan).

## Column Width Calculation (Auto-fit)
- For each column:
  - min_width = max(min content width)
  - max_width = max(max content width)
- Distribute available width proportional to flexibility.

## Cell Layout
- Cell padding and margins applied to content box.
- Vertical alignment: top/middle/bottom.

## Row Height
- Minimum row height from cell content.
- If row height fixed, clip or expand content accordingly.

## Pagination
- Split table by rows.
- Repeat header row if flag set.

## Borders
- Resolve border conflicts with priority:
  - Cell border > row border > table border.
- Inside borders rendered between cells.
