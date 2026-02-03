# DOCX Mapping Appendix — Edge Cases and Advanced OOXML

This appendix enumerates advanced OOXML features and how they map to the internal model.

## Fields and Field Codes
| OOXML | Internal model | Notes |
| --- | --- | --- |
| w:fldChar (begin/separate/end) | Field node with segments | Preserve raw field code text and result run. |
| w:instrText | Field.instruction | Store as string; do not normalize spacing. |
| w:fldSimple | Field node | Treat as a compact field; expand to begin/separate/end on export if needed. |

## Bookmarks and References
| OOXML | Internal model | Notes |
| --- | --- | --- |
| w:bookmarkStart/w:bookmarkEnd | Bookmark range | Store id + name; keep stable IDs. |
| w:hyperlink @w:anchor | Link to bookmark | Map to internal link with target bookmark ID. |

## Content Controls (Structured Document Tags)
| OOXML | Internal model | Notes |
| --- | --- | --- |
| w:sdt | ContentControl node | Store tag, alias, and child content. |
| w:sdtPr | ContentControl.props | Preserve data binding and placeholder text. |

## SmartArt and DrawingML
| OOXML | Internal model | Notes |
| --- | --- | --- |
| w:drawing / a:graphicData | Embedded object | Preserve as opaque drawing resource with fallback image. |
| w:pict (legacy) | Legacy image | Convert to image node with resource reference. |

## Charts
| OOXML | Internal model | Notes |
| --- | --- | --- |
| c:chart | Chart resource | Store chart relationship; render as image if no native chart engine. |

## Footnotes/Endnotes
| OOXML | Internal model | Notes |
| --- | --- | --- |
| w:footnoteRef | Inline footnote marker | Map to note anchor. |
| w:footnote | Note content | Store as note block with id. |

## Comments and Revisions
| OOXML | Internal model | Notes |
| --- | --- | --- |
| w:commentRangeStart/End | Comment span | Map to span over runs. |
| w:comment | Comment thread | Preserve author, date, text. |
| w:ins/w:del | Revision metadata | Store as revision node or metadata on runs. |

## Sections and Page Numbering
| OOXML | Internal model | Notes |
| --- | --- | --- |
| w:sectPr | Section properties | page size, margins, columns, title page. |
| w:pgNumType | Section page numbering | Store format (decimal/roman), start. |

## Headers/Footers
| OOXML | Internal model | Notes |
| --- | --- | --- |
| w:headerReference/w:footerReference | Section header/footer refs | Different first page and even/odd references. |

## Lists and Numbering Edge Cases
| OOXML | Internal model | Notes |
| --- | --- | --- |
| w:num, w:abstractNum | List definitions | Preserve numbering formats, levels, restarts. |
| w:lvlOverride | List override | Store per-list overrides without altering base. |

## Tabs and Alignment
| OOXML | Internal model | Notes |
| --- | --- | --- |
| w:tabs | Paragraph tab stops | Preserve type, position, leader. |

## Page Breaks and Column Breaks
| OOXML | Internal model | Notes |
| --- | --- | --- |
| w:br w:type="page" | Page break block | Prefer explicit page break node. |
| w:br w:type="column" | Column break | Map to block-level column break. |

## Equation Objects
| OOXML | Internal model | Notes |
| --- | --- | --- |
| m:oMath | Math node | Preserve as Math block; render as image if no math engine. |

## Text Effects
| OOXML | Internal model | Notes |
| --- | --- | --- |
| w:textOutline, w:shadow | Run effects | Preserve where supported; otherwise round-trip as metadata. |

## Compatibility Settings
| OOXML | Internal model | Notes |
| --- | --- | --- |
| w:compat | Document compat flags | Preserve and re-emit; do not interpret unless required. |

## Fallback Strategy
- If feature unsupported, store as raw OOXML blob attached to node.
- If feature affects layout, include fallback image or approximation and preserve OOXML for round-trip.
