# Performance Budget

## Input Latency
- Typing: <= 50ms per keystroke.
- Selection updates: <= 16ms.

## Layout
- Paragraph reflow: <= 5ms for small paragraphs.
- Page reflow: <= 50ms for typical edits.
- Full document reflow: <= 2s for 100 pages.

## Rendering
- Frame budget: 16ms for 60fps.
- Full page render: <= 10ms.

## Import/Export
- DOCX import: <= 5s for 100-page doc.
- DOCX export: <= 5s for 100-page doc.
- PDF export: <= 10s for 100-page doc.

## Memory
- <= 500MB for 500-page doc with images.

## Profiling Plan
- Instrument command execution time.
- Track layout cache hit rate.
- Record render frame times.
