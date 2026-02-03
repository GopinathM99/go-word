# OOXML Tables — Detailed Mapping

## Elements
- w:tbl: table container
- w:tblPr: table properties
- w:tblGrid: column definitions
- w:tr: table row
- w:tc: table cell
- w:tcPr: cell properties

## Table Properties
- w:tblStyle: table style ID
- w:tblW: table width (auto/fixed)
- w:tblBorders: borders (top/left/right/bottom/inside)
- w:tblCellMar: cell margins
- w:tblLayout: fixed/auto
- w:tblLook: banding / first row / last row

## Cell Properties
- w:tcW: cell width
- w:gridSpan: column span
- w:vMerge: vertical merge
- w:tcBorders: cell borders
- w:shd: shading
- w:vAlign: vertical alignment

## Mapping Notes
- Preserve gridSpans as colSpan.
- For vMerge:
  - w:vMerge="restart" starts a merged block.
  - w:vMerge without value continues merge.
- Convert to internal row/col span on import.

## Round-Trip
- Preserve tblGrid ordering and widths.
- Keep tblLayout as specified.
