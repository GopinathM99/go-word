# Find and Replace — Detailed Spec

## Search Modes
- Plain text
- Case sensitive
- Whole word
- Regex (optional later)

## Scope
- Entire document
- Selection only
- Current section

## Algorithm
- Build an index of paragraphs (optional).
- For large documents, use incremental scanning to avoid UI freeze.

## Replace
- Replace next, replace all.
- Track replacements for undo as a single command.
