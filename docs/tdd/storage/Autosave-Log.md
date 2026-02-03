# Autosave and Change Log

## Snapshot Strategy
- Full snapshot every N minutes or on major changes.
- Incremental change log between snapshots.

## Change Log Format
- Sequence of commands with timestamps.
- Each entry: { seq, time, command, selection }.

## Recovery
- On crash: load last snapshot + replay log.
- Validate document integrity after replay.

## Retention
- Keep last K snapshots (configurable).
- Prune older logs after successful snapshot.
