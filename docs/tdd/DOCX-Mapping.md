# DOCX Mapping Spec — OOXML to Internal Model

## Mapping Table

| OOXML element/part | Internal model mapping | Notes |
| --- | --- | --- |
| word/document.xml | Document tree root | Parse body, sections, blocks. |
| w:body | Document.sections[].blocks | Section breaks split sections. |
| w:sectPr | Section.pageSetup | Page size, margins, columns. |
| w:p | Paragraph | w:pPr -> paragraph props. |
| w:r | Run | Merge adjacent runs with same style. |
| w:t | Run.text | Preserve whitespace rules. |
| w:rPr | Run.charProps | Font, size, bold, italic, underline. |
| w:pPr | Paragraph.paraProps | Alignment, spacing, indents. |
| w:style (styles.xml) | Style definitions | Map paragraph/character/table styles. |
| w:basedOn | Style inheritance | Build style tree. |
| w:numPr (numbering.xml) | List definitions | Map to list structures. |
| w:tbl | Table | Rows, cells, grid. |
| w:tr | TableRow | Row properties. |
| w:tc | TableCell | Cell props + nested blocks. |
| w:tcPr | TableCell.props | Borders, shading, width. |
| w:tblPr | Table.props | Table style, borders. |
| w:tblGrid | Table.columns | Column widths. |
| w:drawing | Image/shape | Resolve image refs via rels. |
| w:footer/w:header | Section.headerFooterRefs | Link to header/footer parts. |
| w:fldSimple/w:instrText | Field nodes | Page number, date, etc. |
| w:footnote/w:endnote | Footnote/Endnote blocks | Store note content + markers. |
| w:commentRangeStart/End | Comment anchors | Map to comment spans. |
| w:comment (comments.xml) | Comment thread | Author, time, text. |
| w:ins/w:del | Track changes | Store revision metadata. |
| w:hyperlink | Inline link | Preserve URL + style. |
| w:br | Line/section break | Hard line or page break by type. |
| w:tab | Tab inline | Keep as tab inline. |

## Mapping Rules
- Direct formatting overrides styles at render time.
- Round-trip: avoid generating new styles unless necessary.
- Preserve unsupported elements as extension metadata and re-emit on export.

## Relationship Parts
- document.xml.rels -> image/hyperlink targets.
- styles.xml -> style map.
- numbering.xml -> list definitions.
- settings.xml -> compatibility flags (optional).
