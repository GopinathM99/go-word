# Telemetry and Diagnostics

## Principles
- Opt-in by default.
- No document content collected.
- Aggregate metrics only.

## Event Schema
Each event includes:
- event_id
- timestamp
- session_id
- app_version
- platform
- payload (event-specific)

## Core Events
- app_start
- doc_open
- doc_save
- doc_export
- command_execute
- crash_report

## Performance Metrics
- input_latency_ms
- layout_time_ms
- render_time_ms
- import_time_ms
- export_time_ms

## Crash Reports
- Stack trace
- Last command executed
- Document size (pages/words)

## Privacy
- Hash or redact file names.
- Never include document text.
