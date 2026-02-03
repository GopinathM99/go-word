# DOCX Minimal-Diff Rules

## Objectives
- Preserve document structure and ordering to minimize diffs.
- Avoid touching elements not affected by edits.
- Maintain re-open fidelity across Word and the editor.

## Packaging Rules
- Preserve original part ordering in the ZIP package where possible.
- Preserve XML namespaces and prefixes.
- Keep relationship IDs stable unless new relationships are added.

## Styles
- Preserve all existing styles in styles.xml.
- Do not reorder styles or remove unused styles.
- Only add new styles when the user explicitly creates them.

## Numbering
- Preserve numbering.xml ordering and IDs.
- Avoid renumbering list IDs on export.

## Paragraphs and Runs
- Retain original run boundaries if unchanged.
- Do not merge or split runs unless necessary for an edit.
- When a run is edited, only update the modified run contents.

## Properties
- Do not normalize properties (e.g., removing defaults) by default.
- Preserve explicit properties as written in the source.

## Unknown/Unsupported Elements
- Store unknown XML in metadata.
- Re-emit in its original location during export.

## Track Changes
- Preserve w:ins and w:del tags.
- New edits with track changes enabled should create new w:ins/w:del blocks.

## Fields and Bookmarks
- Preserve field codes and bookmarks with original IDs.
- Avoid rebuilding field structures on save.

## Comments
- Keep comment IDs stable.
- Preserve order in comments.xml.
