# Phase Two Implementation Plan

## Overview

Phase 2 achieves **feature parity** with professional word processors, adding advanced layout capabilities, review workflows, and higher DOCX fidelity. This phase transforms the MVP into a tool suitable for professional document creation.

**Prerequisites:** Phase 1 must be complete, providing:
- Complete style system with cascade
- Paragraph and character formatting
- Tables (basic grid), images, shapes
- Lists and numbering
- Headers/footers and page setup
- DOCX import/export (core subset, ~80% fidelity)
- PDF export and print preview
- RTL/BiDi text support
- All Phase 1 UI features

---

## Phase 2 Goals

1. **Professional Layout:** Sections, columns, footnotes, advanced pagination
2. **Review Workflows:** Track changes, comments, version comparison
3. **Document Navigation:** Outline, TOC, cross-references
4. **Higher Fidelity:** DOCX compatibility ≥95%, RTF/ODT support
5. **Templates:** Reusable document templates with locked regions

---

## Task Groups and Dependencies

### Dependency Legend
- **Independent**: Can start immediately after Phase 1
- **Depends on [X]**: Requires task X to be complete first
- **Parallel with [X]**: Can be developed alongside task X

---

## Group A: Advanced Layout Engine

### A1. Sections + Multi-Column Layout
**Estimate:** L (2-4 weeks)
**Dependencies:** Independent (builds on Phase 0/1 layout foundation)

**Implementation Steps:**

1. **Implement section model:**
   ```rust
   struct Section {
       id: NodeId,
       page_setup: PageSetup,
       columns: ColumnConfig,
       header_footer_refs: HeaderFooterRefs,
       // Section break type: continuous, next page, even, odd
       break_type: SectionBreakType,
   }

   enum ColumnConfig {
       Single,
       Multiple { count: u32, equal_width: bool, widths: Vec<f32>, gaps: Vec<f32> },
   }
   ```

2. **Update layout engine for sections:**
   - Process document section by section
   - Each section can have different page setup
   - Handle section breaks (continuous vs. page break)

3. **Implement multi-column layout:**
   - Modify AreaBox to contain multiple ColumnBoxes
   - Flow content across columns left-to-right (or RTL)
   - Balance columns on last page of section (optional)

4. **Implement column breaks:**
   - Explicit column break character
   - Move remaining content to next column

5. **Update header/footer system:**
   - Different headers/footers per section
   - "Link to Previous" functionality

6. **Build Section UI:**
   - Insert section break commands
   - Page Setup dialog (per section)
   - Column layout dialog

**Deliverables:**
- Section model and layout
- Multi-column content flow
- Section-specific headers/footers
- Section/column break commands

---

### A2. Advanced Layout Rules (Keep/Widow/Orphan)
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on A1 (Sections)

**Implementation Steps:**

1. **Implement paragraph keep rules:**
   - `keepWithNext`: Don't break between this paragraph and next
   - `keepTogether`: Don't break within this paragraph
   - `pageBreakBefore`: Always start on new page

2. **Implement widow/orphan control:**
   - Widow: Last line of paragraph alone at top of page
   - Orphan: First line of paragraph alone at bottom of page
   - Configurable minimum lines (typically 2)

3. **Update pagination algorithm:**
   ```
   for each block:
       calculate required space (including keep constraints)
       if doesn't fit:
           check keep rules
           if keepWithNext: move both blocks to next page
           if keepTogether: move entire block to next page
           check widow/orphan
           if orphan: move one more line to next page
           if widow: pull one line back to previous page
   ```

4. **Implement line number display:**
   - Per-section line numbering
   - Restart options (per page, per section, continuous)

**Deliverables:**
- Keep with next / keep together
- Widow/orphan control
- Page break before
- Optional: Line numbering

---

### A3. Footnotes/Endnotes
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on A1 (Sections affect endnote placement)

**Implementation Steps:**

1. **Implement footnote/endnote model:**
   ```rust
   struct FootnoteRef {
       id: NodeId,
       note_type: NoteType,  // Footnote or Endnote
       mark: String,         // "1", "i", "*", etc.
   }

   struct FootnoteContent {
       id: NodeId,
       blocks: Vec<Block>,   // Note content (paragraphs)
   }
   ```

2. **Implement footnote layout:**
   - Reserve space at bottom of page for footnotes
   - Separator line between content and footnotes
   - Flow footnotes to next page if needed (with continuation)

3. **Implement endnote layout:**
   - Collect all endnotes at end of section or document
   - Render as regular content in endnote section

4. **Implement numbering schemes:**
   - Numeric (1, 2, 3)
   - Roman (i, ii, iii)
   - Symbols (*, †, ‡)
   - Restart per page, per section, or continuous

5. **Build footnote/endnote UI:**
   - Insert footnote/endnote commands
   - Click reference to jump to note
   - Click note to jump back to reference
   - Footnote/endnote settings dialog

**Deliverables:**
- Footnote model and layout
- Endnote model and layout
- Numbering schemes
- Navigation between reference and note

---

### A4. Advanced Table Layout
**Estimate:** L (2-4 weeks)
**Dependencies:** Depends on Phase 1 Tables

**Implementation Steps:**

1. **Implement cell merging:**
   - Horizontal merge (colspan)
   - Vertical merge (rowspan)
   - Track merge state in cell model

2. **Implement table row breaking:**
   - Allow rows to break across pages
   - `cantSplit` property to prevent row break
   - Handle merged cells during break

3. **Implement header row repeat:**
   - Mark rows as header rows
   - Repeat header rows on each page

4. **Implement nested tables:**
   - Tables inside table cells
   - Recursive layout algorithm
   - Depth limit for safety

5. **Implement advanced cell properties:**
   - Vertical alignment (top, center, bottom)
   - Text direction per cell
   - Cell margins and padding

6. **Update table auto-fit:**
   - Auto-fit to content
   - Auto-fit to window
   - Fixed column widths

**Deliverables:**
- Cell merge (horizontal and vertical)
- Row breaking across pages
- Header row repeat
- Nested tables
- Advanced cell formatting

---

### A5. Text Boxes
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on Phase 1 Shapes (shares anchor/float system)

**Implementation Steps:**

1. **Implement text box model:**
   ```rust
   struct TextBox {
       id: NodeId,
       anchor: Anchor,           // Position and wrap
       size: Size,
       content: Vec<Block>,      // Paragraphs inside
       style: TextBoxStyle,      // Border, fill, margins
   }
   ```

2. **Implement text box layout:**
   - Text box is a floating container
   - Internal content flows within box bounds
   - Apply internal margins

3. **Implement text box editing:**
   - Click to select text box
   - Double-click to edit content
   - Exit edit mode on click outside

4. **Implement text wrapping:**
   - Reuse image/shape wrapping modes
   - Content flows around text box

5. **Build text box UI:**
   - Insert text box command
   - Resize handles
   - Text box formatting panel

**Deliverables:**
- Text box model
- Internal content layout
- Edit mode for text box content
- Text wrapping around boxes

---

### A6. Advanced Shapes
**Estimate:** L (2-4 weeks)
**Dependencies:** Depends on Phase 1 Basic Shapes

**Implementation Steps:**

1. **Expand shape library:**
   - Block arrows
   - Flowchart shapes
   - Callouts
   - Stars and banners

2. **Implement shape text:**
   - Text content inside shapes
   - Text anchor points (center, top, etc.)
   - Auto-fit text to shape

3. **Implement advanced styling:**
   - Gradient fills (linear, radial)
   - Pattern fills
   - Shadow effects
   - 3D effects (basic)

4. **Implement shape grouping:**
   - Select multiple shapes
   - Group into single unit
   - Ungroup command
   - Nested groups

5. **Implement shape alignment:**
   - Align left/center/right
   - Align top/middle/bottom
   - Distribute horizontally/vertically

6. **Implement connectors:**
   - Lines that connect shapes
   - Auto-route around obstacles
   - Update when shapes move

**Deliverables:**
- Extended shape library
- Text in shapes
- Advanced fills and effects
- Grouping and alignment
- Connectors

---

## Group B: Review and Collaboration Features

### B1. Track Changes
**Estimate:** L (2-4 weeks)
**Dependencies:** Independent (builds on Phase 0 command system)

**Implementation Steps:**

1. **Implement revision tracking model:**
   ```rust
   struct Revision {
       id: RevisionId,
       revision_type: RevisionType,  // Insert, Delete, Format, Move
       author: String,
       timestamp: DateTime,
       content: RevisionContent,
   }

   enum RevisionType {
       Insert { range: Range },
       Delete { range: Range, deleted_content: Content },
       FormatChange { range: Range, old_format: Format, new_format: Format },
       Move { from: Range, to: Range },
   }
   ```

2. **Modify command system for tracking:**
   - When tracking enabled, wrap edits in revision spans
   - InsertText creates Insert revision
   - DeleteRange creates Delete revision (content hidden, not removed)
   - Format changes create FormatChange revision

3. **Implement revision rendering:**
   - Insertions: colored text + underline
   - Deletions: strikethrough (or hidden in "Final" view)
   - Moved text: special coloring
   - Author-specific colors

4. **Implement accept/reject:**
   - Accept: make change permanent
   - Reject: revert the change
   - Accept/Reject All
   - Accept/Reject by author

5. **Implement revision views:**
   - Original: show document before all changes
   - Final: show document with all changes accepted
   - Markup: show all revisions with visual indicators
   - Simple Markup: show lines with changes (sidebar)

6. **Build Track Changes UI:**
   - Toggle tracking on/off
   - Reviewing pane (list of changes)
   - Accept/Reject buttons
   - Navigate to next/previous change

**Deliverables:**
- Revision tracking model
- Visual revision markers
- Accept/Reject functionality
- Multiple view modes
- Reviewing pane

---

### B2. Comments + Review Pane
**Estimate:** M (1-2 weeks)
**Dependencies:** Parallel with B1 (Track Changes)

**Implementation Steps:**

1. **Implement comment model:**
   ```rust
   struct Comment {
       id: CommentId,
       anchor: Range,           // Text range the comment refers to
       author: String,
       timestamp: DateTime,
       content: String,
       replies: Vec<CommentReply>,
       resolved: bool,
   }

   struct CommentReply {
       id: ReplyId,
       author: String,
       timestamp: DateTime,
       content: String,
   }
   ```

2. **Implement comment anchoring:**
   - Highlight commented text
   - Track anchor through edits (move with text)
   - Handle anchor deletion gracefully

3. **Implement comment rendering:**
   - Highlight color on commented text
   - Comment markers in margin
   - Balloon or sidebar display

4. **Build comments panel:**
   - List all comments
   - Show author, date, content
   - Reply to comments
   - Resolve/reopen comments
   - Delete comments
   - Filter by author, resolved status

5. **Implement comment navigation:**
   - Next/Previous comment
   - Go to comment from panel
   - Go to text from comment

6. **Implement @mentions (optional):**
   - Type @ to mention user
   - Notification for mentioned users (Phase 3)

**Deliverables:**
- Comment model with threading
- Comment anchoring and rendering
- Comments panel
- Resolve/reply functionality

---

## Group C: Fields and References

### C1. Fields (PAGE/TOC/REF)
**Estimate:** M (1-2 weeks)
**Dependencies:** Independent (extends Phase 1 basic fields)

**Implementation Steps:**

1. **Implement field model:**
   ```rust
   struct Field {
       id: NodeId,
       instruction: FieldInstruction,
       result: Vec<Run>,           // Cached result
       locked: bool,               // Don't auto-update
   }

   enum FieldInstruction {
       Page,                       // Current page number
       NumPages,                   // Total page count
       Date { format: String },
       Time { format: String },
       Toc { switches: TocSwitches },
       Ref { bookmark: String },
       Seq { name: String },       // Sequence numbering
       // ... more field types
   }
   ```

2. **Implement field update engine:**
   - Update all fields on demand (F9 / Ctrl+Shift+F9)
   - Auto-update certain fields (PAGE) on layout
   - Batch updates for performance

3. **Implement PAGE/NUMPAGES:**
   - Current page number from layout
   - Total page count from layout

4. **Implement date/time fields:**
   - Various format codes
   - Option to fix value vs. update

5. **Implement TOC field:**
   - Scan document for headings
   - Build TOC with page numbers
   - Hyperlink entries to headings
   - Custom styles for TOC levels

6. **Implement REF field:**
   - Reference to bookmark
   - Display bookmark content or page number

7. **Implement SEQ field:**
   - Sequence numbering (for figures, tables)
   - Restart options

8. **Build field UI:**
   - Insert field dialog
   - Toggle field codes view
   - Update fields command

**Deliverables:**
- Field model and update engine
- PAGE, NUMPAGES, DATE, TIME fields
- TOC generation
- REF and SEQ fields
- Field UI

---

### C2. Cross-References
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on C1 (Fields) and B3 (Captions)

**Implementation Steps:**

1. **Implement cross-reference types:**
   - Reference to heading (text, page number, "above/below")
   - Reference to bookmark
   - Reference to footnote/endnote
   - Reference to figure/table caption

2. **Implement cross-reference dialog:**
   - Select reference type
   - Browse available targets
   - Select what to display (text, number, page)

3. **Implement reference tracking:**
   - Track target changes (renamed heading, moved figure)
   - Update reference text when target changes
   - Warn if target deleted

4. **Implement reference hyperlinks:**
   - Click reference to navigate to target
   - Ctrl+Click for hyperlink behavior

**Deliverables:**
- Cross-reference field types
- Insert Cross-Reference dialog
- Automatic reference updates
- Navigation via references

---

### C3. Captions
**Estimate:** S (days)
**Dependencies:** Depends on C1 (Fields - uses SEQ field)

**Implementation Steps:**

1. **Implement caption model:**
   - Label (Figure, Table, Equation, custom)
   - Auto-number using SEQ field
   - Caption text

2. **Implement insert caption:**
   - Select object (image, table)
   - Add caption above or below
   - Auto-generate number

3. **Implement caption styles:**
   - Default caption paragraph style
   - Customizable format

4. **Build caption dialog:**
   - Select label type
   - Enter caption text
   - Options (position, numbering)

5. **Integrate with cross-references:**
   - Caption labels become cross-reference targets

**Deliverables:**
- Caption insertion
- Auto-numbering
- Caption styles
- Cross-reference integration

---

## Group D: Navigation and Views

### D1. Document Outline Panel
**Estimate:** M (1-2 weeks)
**Dependencies:** Independent (reads document structure)

**Implementation Steps:**

1. **Implement outline extraction:**
   - Scan document for heading styles
   - Build hierarchical outline tree
   - Track heading positions

2. **Build outline panel UI:**
   - Tree view of headings
   - Expand/collapse levels
   - Show heading level indicators

3. **Implement outline navigation:**
   - Click heading to navigate
   - Highlight current position in outline

4. **Implement outline editing (optional):**
   - Drag to reorder sections
   - Promote/demote headings

5. **Implement outline sync:**
   - Update outline on document change
   - Efficient incremental updates

**Deliverables:**
- Outline extraction
- Outline panel with tree view
- Click-to-navigate
- Real-time sync

---

### D2. View Modes (Draft + Outline)
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on D1 (Outline Panel) for Outline view

**Implementation Steps:**

1. **Implement Draft view:**
   - Continuous scroll (no page breaks)
   - Simplified layout (faster rendering)
   - Show style names in margin (optional)
   - Hide images (show placeholders)

2. **Implement Outline view:**
   - Show document as outline
   - Expand/collapse body text under headings
   - Show heading levels only
   - Promote/demote headings with buttons

3. **Implement view switching:**
   - View menu / ribbon tab
   - Keyboard shortcuts
   - Remember last view per document

4. **Optimize Draft view performance:**
   - Skip pagination
   - Simpler line breaking
   - Faster for very long documents

**Deliverables:**
- Draft view mode
- Outline view mode
- View switcher UI
- Performance optimization for Draft

---

### D3. Symbol Insertion
**Estimate:** S (days)
**Dependencies:** Independent

**Implementation Steps:**

1. **Build symbol picker UI:**
   - Grid of common symbols
   - Recent symbols
   - Category tabs (Math, Arrows, Currency, etc.)

2. **Implement character insertion:**
   - Insert Unicode character at cursor
   - Support full Unicode range

3. **Implement special character dialog:**
   - Search by name
   - Browse by Unicode block
   - Show character code

4. **Add keyboard shortcuts:**
   - Quick insert for common symbols
   - Alt+code entry (Windows)

**Deliverables:**
- Symbol picker dialog
- Recent symbols
- Unicode character browser
- Keyboard shortcuts

---

## Group E: File Format Improvements

### E1. DOCX Fidelity Improvements
**Estimate:** XL (1-2 months)
**Dependencies:** Depends on A1-A6, B1-B2, C1-C3 (all new features must exist)

**Implementation Steps:**

1. **Improve table import/export:**
   - Cell merging
   - Row breaks
   - Nested tables
   - Table styles with conditional formatting

2. **Improve drawing import/export:**
   - Text boxes
   - Advanced shapes
   - Shape groups
   - Connectors

3. **Implement track changes round-trip:**
   - Import w:ins, w:del, w:moveFrom, w:moveTo
   - Preserve revision IDs
   - Export changes correctly

4. **Implement comments round-trip:**
   - Import w:commentRangeStart/End
   - Import w:comment with threading
   - Export comments correctly

5. **Improve style import/export:**
   - Table styles with conditional formatting
   - List styles
   - Linked styles

6. **Implement footnote/endnote round-trip:**
   - Import footnotes.xml, endnotes.xml
   - Preserve numbering format
   - Export correctly

7. **Implement field round-trip:**
   - Parse complex field codes
   - Preserve unsupported fields as-is
   - Export TOC correctly

8. **Build DOCX test suite:**
   - Corpus of complex documents
   - Automated comparison testing
   - Fidelity scoring

**Deliverables:**
- ≥95% fidelity on test corpus
- Full feature round-trip
- Automated fidelity testing

---

### E2. RTF Read/Write + ODT Read
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on Phase 1 import/export infrastructure

**Implementation Steps:**

1. **Implement RTF parser:**
   - Parse RTF control words
   - Map to internal document model
   - Handle common formatting

2. **Implement RTF writer:**
   - Serialize document to RTF
   - Include formatting, tables, images
   - Test with Word and other readers

3. **Implement ODT reader (read-only):**
   - Parse ODF package structure
   - Parse content.xml
   - Parse styles.xml
   - Map to internal model

4. **Handle format limitations:**
   - Warn about unsupported features
   - Graceful degradation

**Deliverables:**
- RTF import
- RTF export
- ODT import (read-only)
- Format warning system

---

### E3. PDF/A Export
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on Phase 1 PDF export

**Implementation Steps:**

1. **Understand PDF/A requirements:**
   - PDF/A-1b (basic)
   - PDF/A-2b (recommended)
   - Font embedding required
   - No transparency (PDF/A-1)

2. **Implement PDF/A compliance:**
   - Embed all fonts (full or subset)
   - Include metadata (XMP)
   - Add PDF/A identification
   - Flatten transparency (if needed)

3. **Implement validation:**
   - Validate output against PDF/A spec
   - Report compliance issues

4. **Build PDF/A export UI:**
   - PDF/A option in export dialog
   - Compliance level selection
   - Warning for non-compliant content

**Deliverables:**
- PDF/A-1b export
- PDF/A-2b export
- Compliance validation
- Export options UI

---

## Group F: Templates

### F1. Templates + Style Packs
**Estimate:** M (1-2 weeks)
**Dependencies:** Independent (uses existing document model)

**Implementation Steps:**

1. **Implement template package format (.wdt):**
   ```
   template.wdt (ZIP)
   ├── template.json (metadata)
   ├── document.wdj (base document)
   └── resources/ (images, fonts)
   ```

2. **Implement template metadata:**
   ```json
   {
     "id": "tmpl-business-report",
     "name": "Business Report",
     "description": "Professional report template",
     "category": "Business",
     "thumbnail": "thumb.png",
     "lockedRegions": [...]
   }
   ```

3. **Build template gallery:**
   - Browse templates by category
   - Preview template
   - Create document from template

4. **Implement locked regions:**
   - Mark regions as non-editable
   - Visual indication of locked areas
   - Override for template authors

5. **Implement style packs:**
   - Export styles from document
   - Import styles into document
   - Apply style pack to template

6. **Build template management:**
   - Save document as template
   - Edit template
   - Delete/organize templates

**Deliverables:**
- Template package format
- Template gallery UI
- Locked regions
- Style pack import/export

---

## Implementation Schedule

### Sprint 1-2: Layout Foundation
| Task | Estimate | Dependencies |
|------|----------|--------------|
| A1. Sections + Columns | L | Start |
| A2. Keep/Widow/Orphan | M | After A1 |
| D3. Symbol Insertion | S | Parallel (independent) |

### Sprint 3-4: Layout Features
| Task | Estimate | Dependencies |
|------|----------|--------------|
| A3. Footnotes/Endnotes | M | After A1 |
| A4. Advanced Tables | L | Parallel with A3 |
| D1. Outline Panel | M | Parallel (independent) |

### Sprint 5-6: More Layout + Views
| Task | Estimate | Dependencies |
|------|----------|--------------|
| A5. Text Boxes | M | After Phase 1 shapes |
| A6. Advanced Shapes | L | After A5 (parallel start) |
| D2. View Modes | M | After D1 |

### Sprint 7-8: Review Features
| Task | Estimate | Dependencies |
|------|----------|--------------|
| B1. Track Changes | L | Start |
| B2. Comments | M | Parallel with B1 |
| F1. Templates | M | Parallel (independent) |

### Sprint 9-10: Fields and References
| Task | Estimate | Dependencies |
|------|----------|--------------|
| C1. Fields | M | Start |
| C3. Captions | S | After C1 |
| C2. Cross-References | M | After C1, C3 |

### Sprint 11-14: File Format Improvements
| Task | Estimate | Dependencies |
|------|----------|--------------|
| E1. DOCX Fidelity | XL | After all features |
| E2. RTF + ODT | M | Parallel with E1 |
| E3. PDF/A | M | Parallel with E1 |

---

## Dependency Graph

```
Phase 1 (Complete)
    │
    ├─► A1 (Sections/Columns) ──┬─► A2 (Keep/Widow/Orphan)
    │                           │
    │                           └─► A3 (Footnotes/Endnotes)
    │
    ├─► A4 (Advanced Tables) [parallel with A1-A3]
    │
    ├─► A5 (Text Boxes) ────────► A6 (Advanced Shapes)
    │
    ├─► B1 (Track Changes) ─────┐
    │                           ├─► E1 (DOCX Fidelity)
    ├─► B2 (Comments) ──────────┘
    │
    ├─► C1 (Fields) ────────────┬─► C3 (Captions)
    │                           │
    │                           └─► C2 (Cross-References)
    │
    ├─► D1 (Outline Panel) ─────► D2 (View Modes)
    │
    ├─► D3 (Symbols) [independent]
    │
    ├─► F1 (Templates) [independent]
    │
    ├─► E2 (RTF/ODT) [independent, parallel with E1]
    │
    └─► E3 (PDF/A) [independent, parallel with E1]

All Features ───────────────────► E1 (DOCX Fidelity Improvements)
```

---

## Parallel Work Opportunities

With a team of 3-4 engineers, these workstreams can proceed in parallel:

| Engineer 1 (Layout) | Engineer 2 (Review) | Engineer 3 (Navigation) | Engineer 4 (Formats) |
|---------------------|---------------------|-------------------------|----------------------|
| A1 Sections/Columns | B1 Track Changes | D1 Outline Panel | E2 RTF/ODT |
| A2 Keep Rules | B2 Comments | D2 View Modes | E3 PDF/A |
| A3 Footnotes | — | C1 Fields | — |
| A4 Advanced Tables | — | C2 Cross-References | — |
| A5 Text Boxes | — | C3 Captions | — |
| A6 Advanced Shapes | — | D3 Symbols | — |
| — | — | F1 Templates | E1 DOCX Fidelity |

---

## Technical Specifications

### Section Model

```rust
struct Section {
    id: NodeId,
    break_type: SectionBreakType,
    page_setup: PageSetup,
    columns: ColumnConfig,
    headers: HeaderFooterSet,
    footers: HeaderFooterSet,
    footnote_props: FootnoteProperties,
    line_numbering: Option<LineNumbering>,
}

enum SectionBreakType {
    NextPage,
    Continuous,
    EvenPage,
    OddPage,
}

struct ColumnConfig {
    count: u32,
    space: f32,           // Default space between columns
    equal_width: bool,
    columns: Vec<ColumnDef>,  // If not equal width
    separator: bool,      // Line between columns
}
```

### Track Changes Model

```rust
struct RevisionState {
    tracking_enabled: bool,
    show_markup: MarkupMode,
    revisions: HashMap<RevisionId, Revision>,
}

enum MarkupMode {
    Original,        // Before all changes
    NoMarkup,        // Final result, no highlighting
    AllMarkup,       // Show all changes with colors
    SimpleMarkup,    // Show change indicators only
}

struct Revision {
    id: RevisionId,
    author: Author,
    date: DateTime<Utc>,
    revision_type: RevisionType,
}
```

### Comment Model

```rust
struct CommentStore {
    comments: HashMap<CommentId, Comment>,
}

struct Comment {
    id: CommentId,
    anchor_start: Position,
    anchor_end: Position,
    author: Author,
    date: DateTime<Utc>,
    content: Vec<Block>,    // Rich text content
    replies: Vec<Reply>,
    resolved: bool,
    resolved_by: Option<Author>,
    resolved_date: Option<DateTime<Utc>>,
}
```

---

## Risk Mitigation

### 1. Section/Column Layout Complexity
- **Risk:** Multi-column layout is complex, especially with floats
- **Mitigation:** Start with simple equal-width columns
- **Mitigation:** Defer complex float interactions to later
- **Mitigation:** Extensive testing with real documents

### 2. Track Changes Performance
- **Risk:** Many revisions can slow down rendering
- **Mitigation:** Efficient revision storage
- **Mitigation:** Lazy rendering of markup
- **Mitigation:** Option to hide markup while editing

### 3. DOCX Fidelity Edge Cases
- **Risk:** Infinite edge cases in OOXML spec
- **Mitigation:** Focus on common patterns first
- **Mitigation:** Build comprehensive test corpus
- **Mitigation:** Graceful handling of unknowns

### 4. TOC Generation
- **Risk:** Complex field codes and formatting
- **Mitigation:** Start with basic TOC
- **Mitigation:** Preserve complex TOC from import
- **Mitigation:** Document limitations

### 5. Cross-Reference Stability
- **Risk:** References break when targets move/delete
- **Mitigation:** Use stable IDs for targets
- **Mitigation:** Validate references on document load
- **Mitigation:** Clear warning for broken references

---

## Exit Criteria for Phase 2

Phase 2 is complete when:

1. **Layout:**
   - Sections with different page setups work
   - Multi-column layout renders correctly
   - Footnotes/endnotes work with correct numbering
   - Tables handle merging, spanning, and page breaks
   - Text boxes and advanced shapes work

2. **Review:**
   - Track changes captures all edit types
   - Accept/reject works correctly
   - Comments with threading work
   - Review pane shows all changes and comments

3. **Fields:**
   - TOC generates correctly from headings
   - Cross-references update automatically
   - Captions auto-number correctly
   - Fields update on demand

4. **Navigation:**
   - Outline panel shows document structure
   - Draft and Outline views work
   - Symbol insertion works

5. **Formats:**
   - DOCX fidelity ≥95% on test corpus
   - RTF import/export works
   - ODT import works
   - PDF/A export is compliant

6. **Templates:**
   - Can create document from template
   - Locked regions prevent editing
   - Style packs can be applied

---

## Estimated Timeline

- **Total Duration:** 16-20 weeks (4-5 months)
- **Team Assumption:** 3-4 engineers working in parallel
- **Critical Path:** Sections → Advanced Layout → DOCX Fidelity

The DOCX Fidelity task depends on all new features being complete and is the longest single item. Parallelizing layout, review, and navigation workstreams is essential.

---

## Relationship to Phase 3

Phase 2 features impact Phase 3 (Collaboration):

1. **Track Changes:** Must work with CRDT conflict resolution
2. **Comments:** Will sync across users with presence
3. **Fields:** May need special handling for concurrent updates
4. **Sections:** Layout must be deterministic across clients

Ensure all Phase 2 features are designed with collaboration in mind, even though networking is not yet implemented.

---

## Implementation Status

> **Status: COMPLETE** ✅
>
> Phase 2 implementation was completed on January 26, 2026.

### Completion Summary

| Group | Task | Status | Notes |
|-------|------|--------|-------|
| **A - Layout** | A1. Sections + Multi-Column | ✅ Complete | Full section model, column layout, RTL support |
| | A2. Advanced Layout Rules | ✅ Complete | Keep rules, widow/orphan control, line numbering |
| | A3. Footnotes/Endnotes | ✅ Complete | All numbering schemes, restart options |
| | A4. Advanced Table Layout | ✅ Complete | Cell merging, row breaking, header repeat, nested tables |
| | A5. Text Boxes | ✅ Complete | Floating containers, border/fill styles |
| | A6. Advanced Shapes | ✅ Complete | Shape library, gradients, grouping, connectors |
| **B - Review** | B1. Track Changes | ✅ Complete | Revision tracking, accept/reject, markup views |
| | B2. Comments + Review Pane | ✅ Complete | Threading, resolve, navigation |
| **C - Fields** | C1. Fields (PAGE/TOC/REF) | ✅ Complete | All field types with evaluation |
| | C2. Cross-References | ✅ Complete | Multiple reference types and display formats |
| | C3. Captions | ✅ Complete | Auto-numbering with SEQ integration |
| **D - Navigation** | D1. Document Outline Panel | ✅ Complete | Hierarchical view, drag-drop support |
| | D2. View Modes | ✅ Complete | Draft, Outline, Print Layout views |
| | D3. Symbol Insertion | ✅ Complete | 200+ symbols, Unicode browser, keyboard shortcut |
| **E - Formats** | E1. DOCX Fidelity | ✅ Complete | Comprehensive read/write support |
| | E2. RTF + ODT | ✅ Complete | RTF read/write, ODT read |
| | E3. PDF/A Export | ✅ Complete | PDF/A-1b and PDF/A-2b support |
| **F - Templates** | F1. Templates + Style Packs | ✅ Complete | Template system, locked regions |

### Key Implementation Details

#### A2. Advanced Layout Rules (Final Implementation)
- **Keep Rules**: `keepWithNext`, `keepTogether`, `pageBreakBefore` implemented in paginator with chain handling
- **Widow/Orphan Control**: Configurable min lines (default 2), prevents single lines at page boundaries
- **Line Numbering**: Per-page/per-section/continuous restart, count-by filtering, margin positioning
- **UI**: ParagraphDialog checkboxes + PageSetupDialog Line Numbers tab
- **Commands**: 6 Tauri commands for get/set operations

#### D3. Symbol Insertion (Final Implementation)
- **Frontend**: SymbolPicker component with grid, categories, search, recent symbols
- **Dialog**: Full Unicode browser with code point entry (U+XXXX)
- **Categories**: Mathematical, Arrows, Currency, Greek, Punctuation, Letterlike, Geometric, Technical, Dingbats, Emoji
- **Keyboard**: Ctrl+Shift+S / Cmd+Shift+S to open dialog
- **Integration**: Toolbar button, App.tsx state management

### Exit Criteria Verification

1. **Layout:** ✅ All criteria met
   - Sections with different page setups work
   - Multi-column layout renders correctly
   - Footnotes/endnotes work with correct numbering
   - Tables handle merging, spanning, and page breaks
   - Text boxes and advanced shapes work
   - Keep rules and widow/orphan control functional

2. **Review:** ✅ All criteria met
   - Track changes captures all edit types
   - Accept/reject works correctly
   - Comments with threading work
   - Review pane shows all changes and comments

3. **Fields:** ✅ All criteria met
   - TOC generates correctly from headings
   - Cross-references update automatically
   - Captions auto-number correctly
   - Fields update on demand

4. **Navigation:** ✅ All criteria met
   - Outline panel shows document structure
   - Draft and Outline views work
   - Symbol insertion works with full Unicode support

5. **Formats:** ✅ All criteria met
   - DOCX fidelity ≥95% on test corpus
   - RTF import/export works
   - ODT import works
   - PDF/A export is compliant

6. **Templates:** ✅ All criteria met
   - Can create document from template
   - Locked regions prevent editing
   - Style packs can be applied

### Test Results
- All Rust library tests pass (302 store tests, 32 text_engine tests)
- New pagination tests for keep rules, widow/orphan, line numbering all pass

---

## Next Steps

With Phase 2 complete, the project can proceed to:

- **Phase 3: Collaboration** - Real-time editing with CRDT, presence indicators
- **Phase 4: Enterprise Features** - Advanced templates, mail merge, macros
