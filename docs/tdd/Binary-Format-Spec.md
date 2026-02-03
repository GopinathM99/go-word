# Binary Document Format Specification (.wdb)

## Goals
- Fast load/save for large documents.
- Deterministic structure for memory mapping.
- Backward-compatible with versioning.

## Endianness
- All numeric fields are little-endian.

## File Layout
```
[Header]
[Section Table]
[Style Table]
[Resource Table]
[String Table]
[Node Stream]
[Blob Table]
```

## Header
| Field | Size | Notes |
| --- | --- | --- |
| magic | 4 bytes | "WDB1" |
| version_major | u16 | Format major version |
| version_minor | u16 | Format minor version |
| flags | u32 | Feature flags |
| header_size | u32 | Size of header |
| section_table_offset | u64 | File offset |
| style_table_offset | u64 | File offset |
| resource_table_offset | u64 | File offset |
| string_table_offset | u64 | File offset |
| node_stream_offset | u64 | File offset |
| blob_table_offset | u64 | File offset |
| checksum | u64 | Optional integrity checksum |

## Tables (Common Layout)
Each table uses:
- count: u32
- entries: variable length

## String Table
- Strings are UTF-8.
- Entry layout:
  - string_id: u32
  - byte_len: u32
  - bytes

## Style Table
- style_id: u32
- style_type: u8 (paragraph=1, character=2, table=3)
- name_string_id: u32
- based_on_style_id: u32 (0 if none)
- properties_blob_ref: u64 (offset into blob table)

## Resource Table
- resource_id: u32
- resource_type: u8 (image=1, comment=2, etc.)
- metadata_blob_ref: u64
- data_blob_ref: u64 (0 if inline)

## Node Stream
Nodes are serialized in document order for locality.

### Node Record Header
- node_id: u32
- node_type: u8
- parent_id: u32
- next_sibling_id: u32
- prev_sibling_id: u32
- payload_len: u32
- payload bytes

### Node Types
- 1: Document
- 2: Section
- 3: Paragraph
- 4: Run
- 5: Table
- 6: Row
- 7: Cell
- 8: Image
- 9: Shape
- 10: List
- 11: Field

### Example Payloads
**Paragraph**
- style_id: u32
- props_blob_ref: u64
- run_count: u32
- run_ids: [u32]

**Run**
- text_string_id: u32
- char_style_id: u32
- char_props_blob_ref: u64

**Table**
- row_count: u32
- row_ids: [u32]
- tbl_props_blob_ref: u64

## Blob Table
- Entry layout:
  - blob_id: u32
  - byte_len: u64
  - bytes

## Versioning and Compatibility
- Reject if major version mismatch.
- Allow minor version forward-compat with unknown fields skipped via payload_len.

## Integrity
- Optional checksum at header level.
- Optional per-blob checksum.
