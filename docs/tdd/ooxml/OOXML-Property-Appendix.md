# OOXML Property Appendix (Key w:* Properties)

This appendix lists common w:* properties and their internal mappings.

## Run Properties (w:rPr)
| Property | Internal mapping | Notes |
| --- | --- | --- |
| w:b | Run.charProps.bold | Boolean |
| w:i | Run.charProps.italic | Boolean |
| w:u | Run.charProps.underline | Style + color |
| w:color | Run.charProps.color | Hex RGB |
| w:highlight | Run.charProps.highlight | Named or hex |
| w:sz | Run.charProps.size | Half-points |
| w:rFonts | Run.charProps.font | Ascii/CS/EastAsia map |
| w:lang | Run.charProps.language | BCP-47 |
| w:caps | Run.charProps.caps | Boolean |
| w:smallCaps | Run.charProps.smallCaps | Boolean |
| w:strike | Run.charProps.strike | Boolean |
| w:vertAlign | Run.charProps.vertAlign | superscript/subscript |
| w:spacing | Run.charProps.letterSpacing | Twips |

## Paragraph Properties (w:pPr)
| Property | Internal mapping | Notes |
| --- | --- | --- |
| w:jc | Paragraph.paraProps.align | left/center/right/justify |
| w:ind | Paragraph.paraProps.indent | left/right/first/hanging |
| w:spacing | Paragraph.paraProps.spacing | before/after/line |
| w:keepLines | Paragraph.paraProps.keepLines | Boolean |
| w:keepNext | Paragraph.paraProps.keepNext | Boolean |
| w:pageBreakBefore | Paragraph.paraProps.pageBreakBefore | Boolean |
| w:tabs | Paragraph.paraProps.tabs | Tab stops |

## Section Properties (w:sectPr)
| Property | Internal mapping | Notes |
| --- | --- | --- |
| w:pgSz | Section.pageSetup.size | width/height |
| w:pgMar | Section.pageSetup.margins | top/bottom/left/right |
| w:cols | Section.pageSetup.columns | num + widths |
| w:titlePg | Section.headerFooterRefs.first | first-page header/footer |
| w:pgNumType | Section.pageSetup.pageNumbering | start + format |

## Table Properties (w:tblPr)
| Property | Internal mapping | Notes |
| --- | --- | --- |
| w:tblStyle | Table.tableProps.styleRef | Style ID |
| w:tblW | Table.tableProps.width | fixed/auto |
| w:tblLayout | Table.tableProps.layout | fixed/auto |
| w:tblBorders | Table.tableProps.borders | border set |
| w:tblLook | Table.tableProps.look | banding/first/last row |

## Cell Properties (w:tcPr)
| Property | Internal mapping | Notes |
| --- | --- | --- |
| w:tcW | TableCell.props.width | width in twips |
| w:gridSpan | TableCell.props.colSpan | integer |
| w:vMerge | TableCell.props.rowSpan | vertical merge |
| w:tcBorders | TableCell.props.borders | borders |
| w:shd | TableCell.props.shading | fill |
