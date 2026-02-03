# IPC API Specification

## Principles
- Request/response for commands.
- Event stream for document changes.

## Request Envelope
```json
{
  "id": "req-uuid",
  "method": "apply_command",
  "params": { ... }
}
```

## Response Envelope
```json
{
  "id": "req-uuid",
  "result": { ... },
  "error": null
}
```

## Error Object
```json
{
  "code": "INVALID_COMMAND",
  "message": "...",
  "details": { ... }
}
```

## Methods
### apply_command
Params:
- doc_id
- command
Result:
- selection
- change_summary

### get_layout
Params:
- doc_id
- viewport
Result:
- render_model

### import_docx
Params:
- bytes
Result:
- doc_id

### export_docx
Params:
- doc_id
Result:
- bytes

### save_document
Params:
- doc_id
- format
Result:
- path

## Events
### document_changed
Payload:
- doc_id
- changed_nodes
- dirty_pages
- selection
