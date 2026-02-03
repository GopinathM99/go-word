# Layout Cache and Incremental Reflow

## Cache Keys
- Paragraph cache key = hash(run text, run styles, paragraph props).
- Store line breaks, glyph positions, and height.

## Invalidation Rules
- Edit within paragraph: invalidate that paragraph.
- Style changes: invalidate all paragraphs referencing the style.
- Page setup changes: invalidate all blocks in section.

## Reflow Strategy
1. Recompute dirty paragraph layout.
2. Reflow pages from the first changed block forward.
3. Stop when page breaks and heights stabilize.

## Layout Stability
- Use deterministic rounding for glyph metrics.
- Ensure stable font fallback selection.

## Cache Eviction
- LRU cache for paragraph layouts.
- Evict if memory > threshold (configurable).
