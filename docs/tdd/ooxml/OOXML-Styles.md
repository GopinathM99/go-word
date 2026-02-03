# OOXML Styles Mapping — Detailed

## Parts
- /word/styles.xml
- /word/stylesWithEffects.xml (if present)

## Style Types
- Paragraph (w:style w:type="paragraph")
- Character (w:style w:type="character")
- Table (w:style w:type="table")
- Numbering (w:style w:type="numbering")

## Defaults
- w:docDefaults defines default run and paragraph properties.
- Internal defaults should mirror docDefaults, falling back to system defaults only when absent.

## Latent Styles
- w:latentStyles defines behavior of built-in styles.
- Preserve latent styles for round-trip even if not used.

## Inheritance Rules
- Style basedOn -> parent style.
- Effective properties = parent style props + child props.
- Direct formatting overrides styles at render time.

## Style Link
- w:link links paragraph and character style (e.g., Heading 1 + Heading 1 Char).
- Preserve link to ensure Word-compatible behavior.

## Table Style Rules
- Table styles include conditional formatting (first row, last row, banded rows, etc.).
- Preserve and apply conditional rules when rendering.

## Run Properties
- Map w:rPr to Run.charProps:
  - w:rFonts -> font family set
  - w:sz / w:szCs -> font size
  - w:b, w:i, w:u, w:color
  - w:highlight

## Paragraph Properties
- Map w:pPr to Paragraph.paraProps:
  - w:jc alignment
  - w:ind indent
  - w:spacing before/after/line
  - w:keepLines, w:keepNext, w:pageBreakBefore

## Theme Fonts
- If style references theme fonts (w:themeFont):
  - Resolve via /word/theme/theme1.xml
  - Store resolved font name + theme reference.

## Round-Trip Rules
- Preserve the original style order.
- Do not remove unused styles.
- Preserve style IDs and names.
