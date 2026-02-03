# Command System and Undo/Redo

## Command Structure
Each command defines:
- apply(document) -> document
- invert(document) -> command
- selection_transform

## Command Categories
- Text: InsertText, DeleteRange, ReplaceRange
- Structure: SplitParagraph, MergeParagraph, InsertBlock, DeleteBlock
- Formatting: ApplyInlineStyle, ApplyParagraphStyle
- Objects: InsertImage, InsertTable, ResizeImage

## Batching
- Typing burst: merge sequential InsertText within time threshold.
- IME composition: single command for entire composition.

## Undo/Redo Stack
- Stack entries are commands with inverse operations.
- Limit by entry count or memory usage.

## Selection Transform
- Each command provides a selection transform function.
- Apply transforms to anchor/focus after command.
