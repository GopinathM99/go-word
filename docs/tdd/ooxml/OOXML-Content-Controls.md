# OOXML Content Controls (SDT)

## Elements
- w:sdt: structured document tag container
- w:sdtPr: properties (tag, alias, binding)
- w:sdtContent: content

## Internal Representation
- ContentControl node:
  - id
  - tag
  - alias
  - binding (if present)
  - placeholder text
  - child content

## Mapping Rules
- Preserve tag and alias for round-trip.
- Preserve data binding info even if not interpreted.
- Content controls can wrap blocks or inline runs.

## Rendering
- Optional: render content control boundaries in design mode.
- Default: render child content normally.
