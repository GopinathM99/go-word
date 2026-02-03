# Style Cascade and Resolution

## Sources
- Document defaults (docDefaults)
- Style inheritance tree
- Direct formatting

## Resolution Order
1. Document defaults
2. Base style(s)
3. Applied style
4. Direct formatting overrides

## Paragraph Style
- Affects paragraph properties and run properties (if defined).

## Character Style
- Applies only to run properties.

## Table Style
- Applies to table, row, cell, and run based on conditional flags.

## Conditional Table Formatting
- First row, last row, header row, banded rows, first/last column.
- Apply conditional props in order, then direct formatting.

## Caching
- Cache resolved style per node to speed render.
- Invalidate cache on style edits.
