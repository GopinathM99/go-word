# DOCX Round-Trip Strategy

## Goals
- Open DOCX, edit, and re-save with minimal diffs.
- Preserve unsupported content without data loss.

## Style Reconciliation
- Map Word styles 1:1 using w:styleId.
- Preserve all styles from styles.xml, even if unused.
- New styles should be additive and not alter existing IDs.

## Direct Formatting
- Store direct formatting as run/paragraph overrides.
- Do not auto-convert direct formatting into new styles.

## Unsupported Elements
- Preserve unknown OOXML as extension metadata.
- Re-emit original XML on export in the same location.

## Stable Serialization
- Preserve ordering of styles and numbering.
- Keep relationship IDs stable.
- Maintain namespace declarations as-is.

## Minimal Diff Policy
- Avoid cleanup or normalization unless explicitly requested.
- If no edits, re-export should be byte-identical where possible.

## Lists and Numbering
- Preserve numbering definitions in numbering.xml.
- Maintain mapping of list IDs to internal list instances.
- Do not renumber unless required by an explicit edit.

## Track Changes
- Preserve w:ins and w:del blocks.
- Append new tracked changes rather than rewriting existing revisions.
