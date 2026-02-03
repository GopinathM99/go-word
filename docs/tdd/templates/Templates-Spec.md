# Template System Specification

## Goals
- Provide reusable document layouts and styles.
- Support locked regions for structured documents.

## Template Package
- File extension: .wdt
- Contents:
  - template.json (metadata)
  - document.wdj (base document)
  - resources/ (images, fonts)

## template.json
```json
{
  "id": "tmpl-001",
  "name": "Business Report",
  "version": "1.0",
  "author": "Team",
  "tags": ["report", "business"],
  "lockedRegions": [
    { "id": "lr1", "label": "Title", "path": "sections[0].blocks[0]" }
  ]
}
```

## Locked Regions
- Prevent edits outside designated blocks.
- Show visual boundaries in editor.

## Style Packs
- Templates can include custom styles.
- Style conflicts resolved by preferring template styles.
