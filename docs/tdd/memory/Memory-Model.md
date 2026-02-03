# Memory Model and Persistence

## Goals
- Efficient handling of large documents.
- Fast undo/redo without copying entire tree.

## Document Model
- Use persistent data structures (copy-on-write).
- Node IDs stable across edits.

## Change Tracking
- Commands store diffs, not full snapshots.
- Periodic snapshots for fast recovery.

## Sharing
- Layout cache references stable node IDs.
- Render model can reference shared glyph caches.

## GC/Compaction
- Remove unused nodes after command merge.
- Compact change log after snapshot.
