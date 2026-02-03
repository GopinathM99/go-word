# File Versioning and History

## Version Model
- Maintain version IDs for each saved state.
- Store version metadata: author, timestamp, summary.

## Storage
- Local: version snapshots + delta logs.
- Cloud (optional): server-side version history.

## Restore
- Restore a previous version creates a new head version.
- Allow diff view between versions (text + layout).
