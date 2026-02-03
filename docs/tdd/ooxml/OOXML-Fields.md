# OOXML Fields — Detailed Mapping

## Field Types
- Simple fields: w:fldSimple
- Complex fields: w:fldChar + w:instrText + w:fldChar

## Complex Field Structure
- w:fldChar w:fldCharType="begin"
- w:instrText (field code)
- w:fldChar w:fldCharType="separate"
- Result runs (display text)
- w:fldChar w:fldCharType="end"

## Internal Representation
- Field node:
  - instruction: raw field code
  - result_runs: list of runs
  - locked: boolean
  - dirty: boolean

## Common Fields
- PAGE, NUMPAGES
- DATE, TIME
- TOC (table of contents)
- REF (cross-reference)

## Mapping Rules
- Preserve raw field instruction text.
- Do not recompute field results unless requested.
- For TOC: store field as opaque with cached result runs.

## Round-Trip
- Re-emit field code and cached result exactly as imported unless updated.
