# Collaboration Conflict Resolution Policies

## Text Conflicts
- CRDT ensures convergence for text inserts/deletes.
- For overlapping deletes, apply in timestamp order.

## Formatting Conflicts
- Use last-writer-wins per attribute.
- If two edits change different attributes, merge both.

## Block Conflicts
- If two users insert blocks at same position:
  - Order by timestamp, tie-break by userId.

## Track Changes Integration
- If track changes enabled:
  - All remote edits are wrapped as tracked changes.
  - Local accept/reject operations generate CRDT ops.

## Comments
- Comment threads are CRDT lists.
- Resolve comment deletion with last-writer-wins.

## Offline Edits
- Store ops in local queue.
- Merge on reconnect with CRDT ordering.
