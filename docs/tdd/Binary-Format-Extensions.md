# Binary Format Extensions — Schema IDs and Compression

This expands the binary format with explicit schema identifiers and compression strategy.

## Schema Identification
Each node type and property blob is identified by a schema ID to support forward compatibility.

### Schema ID Layout
- 16-bit namespace ID
- 16-bit schema version
- 32-bit type ID

Example:
- namespace: 0x0001 (core)
- version: 0x0001
- type: 0x0000000A (ParagraphProps)

## Node Type IDs (Core)
| Node | Type ID |
| --- | --- |
| Document | 0x00000001 |
| Section | 0x00000002 |
| Paragraph | 0x00000003 |
| Run | 0x00000004 |
| Table | 0x00000005 |
| Row | 0x00000006 |
| Cell | 0x00000007 |
| Image | 0x00000008 |
| Shape | 0x00000009 |
| List | 0x0000000A |
| Field | 0x0000000B |
| Footnote | 0x0000000C |
| Endnote | 0x0000000D |
| Comment | 0x0000000E |

## Property Blob Schemas
Each blob includes:
- schema_id (u64)
- byte_len (u32)
- payload (schema-defined)

### Example: Paragraph Properties
Schema ID: core:1:ParagraphProps
Fields (ordered):
- align (u8)
- indent_left (i32)
- indent_right (i32)
- spacing_before (i32)
- spacing_after (i32)
- line_spacing (u16)
- line_spacing_rule (u8)

## Compression Strategy
### Goals
- Reduce file size for large docs.
- Keep random-access reads fast.

### Approach
- Chunked compression on tables and node stream.
- Each chunk has its own header + compressed payload.

### Chunk Layout
- chunk_type (u8)
- compression (u8) 0=none, 1=zstd, 2=lz4
- uncompressed_len (u32)
- compressed_len (u32)
- payload

### Recommended Defaults
- Strings: zstd level 3.
- Node stream: lz4 for faster decode.
- Blobs: no compression for images already compressed (png/jpg).

## Indexing for Random Access
- Maintain an index of node_id -> offset within node stream.
- Store index in a separate table at the end of file.

## Backward Compatibility Rules
- Readers must ignore unknown schema IDs by skipping payload length.
- Writers should preserve unknown blobs when round-tripping.

## Integrity
- Optional per-chunk checksum (xxHash64).
- File-level checksum in header.
