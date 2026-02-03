# Font Handling and Metrics

## Font Discovery
- Use system font APIs to enumerate installed fonts.
- Maintain a font cache with family/style mapping.

## Font Fallback
- If font unavailable:
  - Apply fallback map by script (Latin, CJK, Arabic).
  - Preserve original font name for round-trip.

## Metrics
- Use font ascent, descent, line gap for line height.
- Cache glyph advance widths per font+size.

## Shaping
- Use Harfbuzz or platform shaping to produce glyph runs.
- Store glyph IDs + advances + cluster mapping.

## Consistency Rules
- Prefer deterministic fallback order.
- Store chosen fallback font in layout cache to avoid drift.
