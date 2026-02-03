# Error Handling and Recovery Spec

## DOCX Import Errors
### Parse Failures
- If XML is malformed: attempt recovery by skipping invalid nodes.
- Log errors with node path and continue where possible.

### Missing Parts
- If optional part missing (styles.xml): use defaults.
- If required part missing (document.xml): abort and show error.

### Relationship Errors
- If relationship target missing:
  - Replace with placeholder node.
  - Preserve relationship entry for round-trip.

## Font Errors
- Missing fonts trigger fallback mapping.
- Show non-blocking warning.

## Image Errors
- Missing image resource:
  - Insert placeholder box with error icon.
  - Preserve relationship for round-trip.

## Autosave Errors
- If autosave fails due to disk:
  - Notify user.
  - Continue in-memory with retry.

## Corrupt Internal File
- If .wdb/.wdj fails integrity check:
  - Offer recovery from last snapshot.
  - Log error details for diagnostics.

## Crash Recovery
- Load last snapshot + change log.
- If replay fails, offer restore to last good snapshot.
