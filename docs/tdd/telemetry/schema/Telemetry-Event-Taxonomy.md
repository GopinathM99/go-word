# Telemetry Event Taxonomy

## Core Events
- app_start
- app_exit
- doc_open
- doc_close
- doc_save
- doc_export
- command_execute
- crash_report

## Editor Events
- selection_change
- style_apply
- insert_table
- insert_image
- paste

## Performance Events
- input_latency
- layout_duration
- render_frame_time
- import_duration
- export_duration

## Collaboration Events
- collab_join
- collab_leave
- collab_conflict

## Event Payload Schema (Example)
```json
{
  "event": "doc_open",
  "timestamp": "2026-01-25T00:00:00Z",
  "session_id": "s-123",
  "app_version": "1.0.0",
  "platform": "macOS",
  "payload": {
    "format": "docx",
    "page_count": 10,
    "word_count": 2000
  }
}
```

## Privacy
- No document content or filenames.
- Use hashed IDs only.
