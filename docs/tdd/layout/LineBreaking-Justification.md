# Line Breaking and Justification — Detailed Spec

## Input
- Shaped glyph runs with advance widths.
- Available line width (column width minus indents).
- Break opportunities (Unicode LB).

## Algorithm
1. Build a list of break opportunities with penalty scores.
2. Greedy fit by default (v1).
3. Optional: Knuth-Plass line breaking (future).

## Justification
- For justified text:
  - Distribute extra space across stretchable spaces.
  - Cap stretch per space to avoid rivers.
  - If line ends with a single word, do not justify.

## Bidi and Script
- Use Unicode BiDi algorithm per paragraph.
- Shape runs with script/language tags for correct glyph selection.

## Ligatures and Kerning
- Preserve shaping results; do not split within ligatures.

## Soft Hyphen
- Soft hyphen only visible if used as a break.
- If used, render hyphen glyph and break.

## Non-breaking Spaces
- Treat as non-breakable with fixed width.

## Tab Stops
- Tabs jump to the next tab stop.
- If a tab is beyond line width, break line.
