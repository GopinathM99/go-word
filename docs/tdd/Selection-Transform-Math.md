# Selection Transform Math

This defines how selections update after document edits.

## Selection Representation
- Position: (nodeId, offset)
- Selection: { anchor, focus }

Offsets are UTF-16 indices for compatibility with web IME APIs.

## Notation
- p = edit position (nodeId, offset)
- len = length of inserted or deleted text
- pos = selection position to update

## Insert Text at p
If pos.nodeId == p.nodeId:
- If pos.offset < p.offset: unchanged.
- If pos.offset >= p.offset: pos.offset += len.

If pos.nodeId != p.nodeId: unchanged.

## Delete Range [p, p+len)
If pos.nodeId == p.nodeId:
- If pos.offset < p.offset: unchanged.
- If p.offset <= pos.offset < p.offset + len: pos.offset = p.offset.
- If pos.offset >= p.offset + len: pos.offset -= len.

If pos.nodeId != p.nodeId: unchanged.

## Replace Range [p, p+len) with new text (len2)
Equivalent to Delete then Insert at p:
- Apply delete transform.
- Then apply insert transform with len2.

## Split Paragraph at p
- Paragraph split creates a new paragraph after the original.
- If pos.nodeId == old paragraph:
  - If pos.offset <= p.offset: remains in old paragraph.
  - If pos.offset > p.offset: moves to new paragraph with offset -= p.offset.

## Merge Paragraphs (prev + next)
- If pos.nodeId == next paragraph:
  - pos.nodeId = prev paragraph.
  - pos.offset += prev_length.

## Insert Block Before Paragraph
- Selection positions inside existing paragraph remain unchanged.
- Selection references by nodeId are stable; only reordering affects visual position.

## Table Edits (cell insertion/deletion)
- If selection is inside a modified cell:
  - Apply the same rules as paragraph edits within that cell.
- If selection is in a different cell: unchanged.

## Multi-Cursor Ordering
When applying edits to multiple selections:
- Sort selections by document order.
- Apply edits from end to start to avoid offset shifts.
