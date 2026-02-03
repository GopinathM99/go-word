# OOXML Drawings and Shapes

## Elements
- w:drawing: DrawingML container
- wp:inline / wp:anchor: inline or floating
- a:graphicData: embedded graphic

## Mapping
- Inline drawing -> Image/Shape node with inline anchor.
- Anchor drawing -> floating object with wrap.

## Wrap Types
- wrapNone, wrapSquare, wrapTight, wrapThrough, wrapTopAndBottom
- Map to internal wrap modes.

## Size and Position
- cx, cy (EMU units) -> convert to points/pixels.
- anchor position relative to page/column/paragraph.

## Fallback
- If unsupported drawing type:
  - Extract fallback bitmap if present.
  - Preserve original XML for round-trip.
