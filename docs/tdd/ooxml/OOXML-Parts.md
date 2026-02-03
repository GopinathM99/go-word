# OOXML Package Structure and Parts

This document specifies which OOXML parts are parsed and how relationships are resolved.

## Package (ZIP) Conventions
- Use OPC (Open Packaging Conventions) rules.
- Preserve original part ordering when re-writing.
- Preserve content types in [Content_Types].xml.

## Required Core Parts
| Part | Purpose | Notes |
| --- | --- | --- |
| /word/document.xml | Main document body | Required.
| /word/styles.xml | Styles | Required for style resolution.
| /word/numbering.xml | Lists/numbering | Optional but common.
| /word/settings.xml | Compatibility flags | Preserve for round-trip.
| /word/fontTable.xml | Font list | Use for fallback hints.
| /word/theme/theme1.xml | Theme definitions | Preserve to avoid loss.
| /word/webSettings.xml | Web settings | Preserve if present.
| /word/endnotes.xml | Endnotes | Optional.
| /word/footnotes.xml | Footnotes | Optional.
| /word/comments.xml | Comments | Optional.

## Headers and Footers
- Headers/footers are stored in separate parts with rels:
  - /word/header#.xml
  - /word/footer#.xml
- Resolve via section properties in document.xml.

## Relationships
- /word/_rels/document.xml.rels resolves:
  - images: /word/media/*
  - headers/footers
  - footnotes/endnotes
  - comments
  - hyperlinks (external)
- Preserve relationship IDs (rId*) for minimal diffs.

## Content Types
- Ensure all referenced parts appear in [Content_Types].xml.
- Preserve original content-type ordering and defaults.

## Media
- Images stored under /word/media/.
- Preserve original file names when possible.

## Custom XML
- If custom XML parts exist:
  - Preserve all /customXml/* parts.
  - Preserve /customXml/_rels/*.rels relationships.

## Macro-Enabled Documents
- If VBA parts exist (vbaProject.bin):
  - Preserve without modification.
  - Avoid loading/executing any macro content.

## Round-Trip Rules
- Preserve unknown parts and relationships.
- Never drop content types for unknown parts.
- If a part is not modified, emit byte-identical content if possible.
