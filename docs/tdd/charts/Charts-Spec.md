# Charts and Diagrams Specification

## Supported Chart Types
- Bar, line, pie, scatter, area.

## Import
- Preserve chart XML (chart1.xml) as resource.
- If no chart engine, render chart as image.

## Editing
- Phase 1: read-only.
- Phase 2: basic data editing (table view).

## Data Binding
- Chart data stored in embedded spreadsheet part.
- Preserve for round-trip.

## Export
- DOCX: embed chart XML as-is.
- PDF: render chart to vector or high-res raster.
