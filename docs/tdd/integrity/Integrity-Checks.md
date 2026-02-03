# Document Integrity Checks

## On Load
- Verify schema version.
- Validate tree structure (parent/child consistency).
- Validate style references.
- Validate resource references (images, comments).

## On Save
- Ensure no dangling node references.
- Ensure all referenced resources exist.

## Checksum
- Optional document-level checksum stored in metadata.
- Validate on open and warn if mismatch.

## Repair Strategy
- Remove or quarantine invalid nodes.
- Replace missing resources with placeholders.
- Log all repairs in diagnostics.
