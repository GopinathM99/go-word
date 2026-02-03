# Internal Document Format (JSON and Binary)

## JSON Format (Canonical, Versioned)
**Extension:** .wdj

### Header
```json
{
  "version": "1.0",
  "docId": "uuid",
  "metadata": {
    "title": "My Doc",
    "author": "Name",
    "createdAt": "2026-01-25T00:00:00Z",
    "modifiedAt": "2026-01-25T00:00:00Z",
    "language": "en-US"
  },
  "styles": { ... },
  "sections": [ ... ],
  "resources": { ... }
}
```

### Styles
```json
"styles": {
  "paragraph": [
    {
      "id": "p1",
      "name": "Normal",
      "basedOn": null,
      "properties": { "font": "Times New Roman", "size": 12, "spacingAfter": 8 }
    }
  ],
  "character": [
    {
      "id": "c1",
      "name": "Emphasis",
      "basedOn": null,
      "properties": { "italic": true }
    }
  ],
  "table": [ ... ]
}
```

### Sections and Blocks
```json
"sections": [
  {
    "id": "s1",
    "pageSetup": {
      "size": "A4",
      "margins": { "top": 72, "bottom": 72, "left": 72, "right": 72 },
      "columns": 1
    },
    "headerFooterRefs": { "header": "h1", "footer": "f1" },
    "blocks": [
      {
        "type": "paragraph",
        "id": "p-001",
        "styleRef": "p1",
        "paraProps": { "align": "left", "spacingBefore": 0, "spacingAfter": 8 },
        "runs": [
          { "text": "Hello ", "charStyleRef": null, "charProps": {} },
          { "text": "world", "charStyleRef": "c1", "charProps": { "bold": true } }
        ]
      },
      {
        "type": "table",
        "id": "t-001",
        "rows": [
          { "cells": [ { "blocks": [ ... ] }, { "blocks": [ ... ] } ] }
        ]
      }
    ]
  }
]
```

### Resources
```json
"resources": {
  "images": [
    { "id": "img1", "mime": "image/png", "dataRef": "binary:img1" }
  ],
  "comments": [
    { "id": "cmt1", "author": "A", "text": "Review this", "anchor": { ... } }
  ]
}
```

## Binary Format (Performance)
**Extension:** .wdb

### Structure
- Header: magic + version
- Global tables: styles, resource index
- Node stream: flat nodes with IDs and offsets
- String table: deduplicated text
- Blob table: images and other binary assets

### Rationale
- Faster load/save for large docs
- Efficient memory mapping
- Compact on disk
