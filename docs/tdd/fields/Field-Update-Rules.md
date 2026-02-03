# Field Update Rules

## General Policy
- Fields update on explicit user action (Update Fields).
- Some fields update automatically on print/export.

## Field Types
### PAGE / NUMPAGES
- Update on pagination changes or print/export.

### DATE / TIME
- Update on open if field is configured as "update on open".

### TOC
- Update only on explicit user action.
- Preserve result text otherwise.

### REF
- Update when target bookmark changes and on explicit update.

## Editing Behavior
- If user edits a field result directly, mark field as "dirty".
- Dirty fields are not auto-updated until user chooses.

## Round-Trip
- Preserve field instruction text and dirty flag.
