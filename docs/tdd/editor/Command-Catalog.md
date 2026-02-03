# Command Catalog (Complete)

This catalog enumerates all edit commands and their parameters.

## Text Commands
- InsertText(doc_id, position, text, attributes)
- DeleteRange(doc_id, start, end)
- ReplaceRange(doc_id, start, end, text, attributes)
- SplitParagraph(doc_id, position)
- MergeParagraph(doc_id, paragraph_id)

## Formatting Commands
- ApplyInlineStyle(doc_id, range, styleRef)
- ApplyParagraphStyle(doc_id, paragraph_id, styleRef)
- ApplyDirectFormatting(doc_id, range, props)
- ClearFormatting(doc_id, range)

## Block Commands
- InsertParagraph(doc_id, position, styleRef)
- InsertTable(doc_id, position, rows, cols, tableProps)
- InsertImage(doc_id, position, imageRef, size, wrap)
- InsertShape(doc_id, position, shapeProps)
- InsertSectionBreak(doc_id, position, sectionProps)
- DeleteBlock(doc_id, block_id)

## List Commands
- ToggleList(doc_id, paragraph_id, listStyle)
- IncreaseListIndent(doc_id, paragraph_id)
- DecreaseListIndent(doc_id, paragraph_id)
- RestartListNumbering(doc_id, paragraph_id)

## Table Commands
- InsertRow(doc_id, table_id, index)
- InsertColumn(doc_id, table_id, index)
- DeleteRow(doc_id, table_id, index)
- DeleteColumn(doc_id, table_id, index)
- MergeCells(doc_id, cell_range)
- SplitCell(doc_id, cell_id)

## Review Commands
- AddComment(doc_id, range, text)
- ResolveComment(doc_id, comment_id)
- ToggleTrackChanges(doc_id, enabled)
- AcceptChange(doc_id, change_id)
- RejectChange(doc_id, change_id)

## Metadata Commands
- UpdateDocumentMetadata(doc_id, metadata)
- UpdatePageSetup(doc_id, section_id, pageSetup)
