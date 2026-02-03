# Document Model Schema (Detailed)

This schema describes the internal JSON format (.wdj). It is a conceptual schema, not a strict JSON Schema file.

## Root
- version: string
- docId: string (uuid)
- metadata: object
- styles: object
- sections: array
- resources: object

## Metadata
- title: string
- author: string
- createdAt: ISO-8601 timestamp
- modifiedAt: ISO-8601 timestamp
- language: BCP-47 string

## Styles
- paragraph: array of ParagraphStyle
- character: array of CharacterStyle
- table: array of TableStyle

### ParagraphStyle
- id: string
- name: string
- basedOn: string | null
- properties: ParagraphProps

### CharacterStyle
- id: string
- name: string
- basedOn: string | null
- properties: CharacterProps

### TableStyle
- id: string
- name: string
- basedOn: string | null
- properties: TableProps
- conditional: object (firstRow, lastRow, bandedRows, etc.)

## Sections
### Section
- id: string
- pageSetup: PageSetup
- headerFooterRefs: HeaderFooterRefs
- blocks: array of Block

### PageSetup
- size: string (A4, Letter)
- orientation: portrait | landscape
- margins: { top, bottom, left, right }
- columns: number | array of Column

### Column
- width: number
- gap: number

### HeaderFooterRefs
- header: string | null
- footer: string | null
- firstHeader: string | null
- firstFooter: string | null
- evenHeader: string | null
- evenFooter: string | null

## Block Types
### Paragraph
- type: "paragraph"
- id: string
- styleRef: string
- paraProps: ParagraphProps
- runs: array of Run

### Table
- type: "table"
- id: string
- rows: array of TableRow
- tableProps: TableProps

### Image
- type: "image"
- id: string
- srcRef: string
- size: { width, height }
- wrapMode: string
- anchor: Anchor

### Shape
- type: "shape"
- id: string
- shapeProps: object
- wrapMode: string
- anchor: Anchor

### SectionBreak
- type: "sectionBreak"
- id: string
- sectionProps: PageSetup

## Inline Types
### Run
- text: string
- charStyleRef: string | null
- charProps: CharacterProps

### Field
- type: "field"
- instruction: string
- resultRuns: array of Run

## ParagraphProps
- align: left|center|right|justify
- indentLeft: number
- indentRight: number
- firstLineIndent: number
- spacingBefore: number
- spacingAfter: number
- lineSpacing: number
- lineSpacingRule: single|oneHalf|double|exact
- keepLines: boolean
- keepNext: boolean

## CharacterProps
- font: string
- size: number
- bold: boolean
- italic: boolean
- underline: boolean
- color: string
- highlight: string

## TableProps
- width: number
- layout: fixed|auto
- borders: object
- shading: object

## Anchor
- type: paragraph|page|column
- position: { x, y }
- relativeTo: page|margin|column

## Resources
- images: array of ImageResource
- comments: array of Comment

### ImageResource
- id: string
- mime: string
- dataRef: string

### Comment
- id: string
- author: string
- text: string
- anchor: object
