# Mail Merge Specification

## Data Sources
- CSV
- JSON
- Spreadsheet (optional)

## Merge Fields
- Represent as Field nodes with type "MERGEFIELD".
- Field instruction contains source column name.

## Merge Workflow
1. User selects data source.
2. Map fields to data columns.
3. Preview merged output.
4. Generate output documents (one per row or batch).

## Output Options
- Single combined document.
- Multiple documents.
- Export to PDF.

## Round-Trip
- Preserve merge field codes in DOCX.
- Store data source reference in metadata.
