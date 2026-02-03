# Document Migration Plan

## Versioning
- Semantic versioning for document schema.
- Major version changes require explicit migration.

## Migration Flow
1. Detect schema version on open.
2. If older:
   - Apply sequential migration steps.
   - Validate after each step.
3. If newer:
   - Open in read-only or warn user.

## Example Migrations
- v1.0 -> v1.1: add new paragraph prop field.
- v1.1 -> v1.2: change table props structure.

## Backward Compatibility
- Preserve unknown fields in a metadata blob.
- Re-emit unknown fields on save.
