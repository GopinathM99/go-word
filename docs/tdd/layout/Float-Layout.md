# Floating Objects and Text Wrapping

## Wrap Modes
- Inline: treated as inline box.
- Square: text wraps around bounding box.
- Tight: wrap around contour (if available).
- Through: text can pass through interior gaps.
- Top and bottom: text above and below only.

## Anchor
- Anchored to page, column, or paragraph.
- Anchor moves with text unless fixed position.

## Layout Algorithm
1. Place object at anchor position.
2. Compute exclusion region based on wrap mode.
3. When laying out lines, reduce available line width by exclusion regions intersecting line y-range.

## Z-Order
- Objects have a z-index; higher renders above text.
- If behind-text, object does not affect line width.

## Collision Resolution
- If floating objects overlap:
  - Shift later object down (Word-like behavior).
  - Preserve relative order.
