# Compatibility Matrix (DOCX Feature Coverage)

## Legend
- Full: feature supported with high fidelity
- Partial: supported with minor limitations
- None: not supported (round-trip only)

| Feature | Import | Edit | Export | Notes |
| --- | --- | --- | --- | --- |
| Paragraph styles | Full | Full | Full | Style IDs preserved |
| Character styles | Full | Full | Full | Style link preserved |
| Table styles | Partial | Partial | Partial | Conditional formatting may be limited |
| Lists (numbering) | Full | Full | Full | List overrides preserved |
| Headers/footers | Full | Partial | Full | Editing in phase 2 |
| Footnotes/endnotes | Full | Partial | Full | Editing in phase 2 |
| Track changes | Partial | Partial | Full | Preserve revisions |
| Comments | Full | Partial | Full | Threaded comments phase 2 |
| Fields (PAGE/DATE) | Full | None | Full | Preserve field code |
| TOC | Partial | None | Partial | Regeneration later |
| SmartArt | Partial | None | Partial | Preserved as opaque drawing |
| Charts | Partial | None | Partial | Preserved as image |
| Content controls | Partial | None | Partial | Preserve SDT | 
| RTL text | Full | Partial | Full | Rendering tested |
| CJK | Full | Full | Full | IME and shaping |
