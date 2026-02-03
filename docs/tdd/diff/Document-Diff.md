# Document Diff and Compare

## Goals
- Provide semantic diffs (structure-aware) and visual diffs.

## Semantic Diff
- Compare document trees:
  - Match nodes by stable IDs.
  - Compute edits: insert, delete, modify.
- Output diff as list of operations.

## Text Diff
- For paragraph text:
  - Use Myers diff or similar.
  - Highlight insertions/deletions.

## Style Diff
- Compare style tables and report differences.

## Visual Diff
- Render both documents to PDF.
- Compute pixel diff with thresholds.

## UI
- Side-by-side compare view.
- Inline change markers.
