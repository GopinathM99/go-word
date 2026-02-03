# OOXML Track Changes — Detailed Mapping

## Elements
- w:ins: insertion revision
- w:del: deletion revision
- w:moveFrom/w:moveTo: move revisions

## Revision Metadata
Attributes:
- w:author
- w:date
- w:id

## Internal Representation
- Revision span metadata attached to runs or blocks.
- Store:
  - type (insert/delete/move)
  - author
  - date
  - revision_id

## Rendering
- Insertions: underline or color.
- Deletions: strikethrough or hidden depending on view.

## Editing Rules
- If track changes enabled:
  - New insertions create w:ins spans.
  - Deletions wrap text with w:del instead of removing.

## Round-Trip
- Preserve revision IDs and ordering.
- Do not merge adjacent revisions unless explicitly accepted.
