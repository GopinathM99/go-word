# Selection Model and IME Handling

## Selection Data Structure
```
Selection {
  anchor: Position
  focus: Position
  isCollapsed: boolean
}
Position {
  nodeId: string
  offset: integer
}
```

- Offsets use UTF-16 indices to align with JS IME APIs.
- nodeId references a Run or Paragraph depending on selection scope.

## Selection Invariants
- Anchor and focus must always reference valid positions.
- Update positions on edits using transform deltas.
- Support block selections as inclusive ranges.

## IME Lifecycle (Web)
Events:
- compositionstart
- compositionupdate
- compositionend

### Handling Flow
1. compositionstart
   - Create a composition session with a temporary range.
2. compositionupdate
   - Replace current composition range with new text.
   - Mark composition text with an underline style.
3. compositionend
   - Commit final text as a single InsertText command.
   - Clear composition range.

## Undo/Redo With IME
- Treat the entire composition session as a single undo step.
- Do not create undo entries for intermediate updates.

## Selection After Composition
- After commit, move selection to end of inserted text.
- If composition replaced a selection, ensure correct end position.

## Multi-Cursor (Optional)
- Represent selections as an array.
- Apply commands per selection from end-to-start to avoid offset shifts.
