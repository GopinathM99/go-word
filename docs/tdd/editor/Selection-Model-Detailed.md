# Selection Model — Detailed Spec

## Types
- Caret selection: collapsed range.
- Range selection: anchor/focus across runs.
- Block selection: table cells or block-level regions.

## Data Structure
```
SelectionSet {
  selections: [Selection]
  primary: index
}
Selection { anchor: Position, focus: Position }
Position { nodeId, offset, affinity }
```

## Affinity
- Affinity controls caret at line breaks (up/down navigation).
- Values: upstream, downstream.

## Navigation
- Word navigation uses Unicode word boundaries.
- Line navigation uses layout line boxes.

## Table Selections
- Cell selection: range of cells in a table grid.
- Block selection stored separately from text selections.

## Serialization
- For collaboration: serialize selections as node IDs + offsets.
- For UI: map to layout coordinates on render.
