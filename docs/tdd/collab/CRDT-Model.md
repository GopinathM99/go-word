# Collaboration CRDT Model

## Objectives
- Concurrent edits converge deterministically.
- Preserve formatting and structure.

## Data Model
- Text runs as CRDT sequences (RGA or similar).
- Block structure maintained as a tree with CRDT-ordered children.

## Operations
- InsertText(nodeId, posId, text)
- DeleteText(nodeId, rangeId)
- ApplyStyle(nodeId, rangeId, style)
- InsertBlock(parentId, posId, block)
- DeleteBlock(blockId)

## Ordering
- Use Lamport timestamps or logical clocks to order inserts.
- Tie-breakers by client ID.

## Formatting Conflicts
- Last-writer-wins on style changes.
- Optionally store style as CRDT map for per-attribute resolution.

## Performance
- Batch operations per user input session.
- Compact tombstones periodically.
