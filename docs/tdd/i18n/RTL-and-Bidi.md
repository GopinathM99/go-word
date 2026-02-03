# Internationalization: RTL and BiDi

## Goals
- Correct rendering of mixed RTL/LTR text.
- Proper caret movement and selection behavior.

## Unicode BiDi
- Apply Unicode BiDi algorithm per paragraph.
- Segment runs into directional runs.

## Cursor Movement
- Arrow keys move by visual order, not logical order.
- Maintain logical caret position for editing.

## Selection Rendering
- Selection is in logical range but rendered in visual order.

## Text Shaping
- Shape each script run with appropriate font.
- Use fallback fonts per script.

## Mirroring
- Mirror punctuation and brackets in RTL context when required.

## Mixed Direction Layout
- Ensure tab stops and alignment respect paragraph direction.
- Default alignment for RTL paragraphs is right.
