# Phase One Implementation Plan

## Implementation Status

> **Last Updated:** 2026-01-28
> **Overall Progress:** ✅ 100% Complete
> **Total Tests:** 500+ passing

### Completed Sprints

| Sprint | Tasks | Status | Tests |
|--------|-------|--------|-------|
| 1-2 | A1, A3, B5, F2, F4 | ✅ Complete | 91 |
| 3-4 | A2, B1, B2, B3, B6 | ✅ Complete | 156 |
| 5-6 | B4, C1, C2, F1 | ✅ Complete | 202 |
| 7-8 | E1, E2, F3 | ✅ Complete | 301 |
| 9-10 | D1, D2 | ✅ Complete | 433 |
| 11-12 | D3 (Print Preview) | ✅ Complete | 470 |
| 13-14 | G1 (Performance Tuning) | ✅ Complete | 500+ |

### Implementation Notes

- **D3 (Print Preview):** Fully implemented in `frontend/src/components/PrintPreview.tsx`, `PrintDialog.tsx`, `usePrintPreview.ts`
- **G1 (Performance Tuning):** Core systems complete - `crates/layout_engine/src/cache.rs`, `crates/render_model/src/viewport.rs`, `crates/perf/src/`

---

## Overview

Phase 1 builds on the Phase 0 prototype to deliver a **fully functional single-user editor** with core formatting, import/export, and essential UI features. This phase transforms the prototype into a usable MVP.

**Prerequisites:** Phase 0 must be complete, providing:
- Document tree with IDs and selection model
- Command system with CRDT/OT-compatible operations
- Basic text input with IME support
- Line breaking and pagination (multi-flow-ready)
- BiDi/RTL architecture foundation
- Simple renderer (pages, caret, selection)
- Persistence snapshot

---

## Task Groups and Dependencies

### Dependency Legend
- **Independent**: Can start immediately after Phase 0
- **Depends on [X]**: Requires task X to be complete first
- **Parallel with [X]**: Can be developed alongside task X

---

## Group A: Core Formatting System (Foundation for All Content)

These tasks extend the document model and must be completed early as other features depend on them.

### A1. Style Cascade + Inspector Basics ✅ COMPLETE
**Estimate:** M (1-2 weeks)
**Dependencies:** Independent (builds on Phase 0 document model)
**Status:** Implemented in `crates/doc_model/src/style.rs` with StyleRegistry, 15 built-in styles, basedOn inheritance

**Implementation Steps:**
1. Implement style resolution algorithm (basedOn inheritance chain)
2. Create style registry with paragraph, character, and table styles
3. Implement direct formatting override logic (style + local overrides)
4. Build style inspector UI panel showing computed styles
5. Add style gallery component for quick style application
6. Implement "New Style" and "Modify Style" dialogs

**Deliverables:**
- Style resolution engine in Rust core
- Style inspector panel in TypeScript UI
- Style gallery with built-in styles (Normal, Heading 1-6, etc.)

---

### A2. Paragraph Formatting ✅ COMPLETE
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on A1 (Style Cascade)
**Status:** Implemented in `crates/edit_engine/src/paragraph_commands.rs` with alignment, indentation, spacing commands

**Implementation Steps:**
1. Extend paragraph properties in document model:
   - Alignment (left, center, right, justify)
   - Indentation (left, right, first-line, hanging)
   - Spacing (before, after, line spacing)
2. Update layout engine to respect paragraph properties
3. Implement paragraph formatting commands
4. Build paragraph formatting toolbar/ribbon section
5. Create paragraph dialog for advanced settings

**Deliverables:**
- Paragraph property support in layout engine
- Formatting toolbar with alignment, indent, spacing controls
- Paragraph settings dialog

---

### A3. Font Fallback + Substitution System ✅ COMPLETE
**Estimate:** M (1-2 weeks)
**Dependencies:** Independent (builds on Phase 0 text engine)
**Status:** Implemented in `crates/text_engine/src/` with FontManager, discovery, fallback chains, substitution matrix

**Implementation Steps:**
1. Build font discovery module (enumerate system fonts)
2. Create font substitution matrix (map common fonts to fallbacks)
3. Implement per-script fallback chains (Latin, CJK, Arabic, etc.)
4. Add missing font detection and warning system
5. Store font substitution decisions in document metadata
6. Build "Font Substitution" notification UI

**Deliverables:**
- Font discovery and caching system
- Substitution matrix configuration
- Missing font warnings with substitution info

---

## Group B: Content Types (Parallel Development Possible)

These tasks add different content types to the editor. Most can be developed in parallel once Group A is complete.

### B1. Lists + Numbering ✅ COMPLETE
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on A1 (Style Cascade), A2 (Paragraph Formatting)
**Status:** Implemented in `crates/doc_model/src/list.rs` with AbstractNum, NumberingInstance, ListLevel, NumberingRegistry

**Implementation Steps:**
1. Implement list definition model (abstractNum, num, lvl)
2. Create numbering format engine (decimal, roman, bullet, etc.)
3. Add list-related paragraph properties (numId, ilvl)
4. Implement list commands:
   - Toggle bullet/number list
   - Increase/decrease indent
   - Change list type
   - Restart numbering
5. Build list formatting UI (toolbar buttons, list gallery)
6. Handle list continuation across paragraphs

**Deliverables:**
- List model with multi-level support
- Numbering format engine
- List toolbar and gallery

---

### B2. Tables (Grid + Basic Layout) ✅ COMPLETE
**Estimate:** L (2-4 weeks)
**Dependencies:** Depends on A1 (Style Cascade)
**Status:** Implemented in `crates/doc_model/src/table.rs` and `crates/layout_engine/src/table_layout.rs`

**Implementation Steps:**
1. Implement table model (table, rows, cells, grid)
2. Build table layout algorithm:
   - Fixed width layout
   - Auto-fit to content
   - Column width distribution
3. Implement cell properties (borders, shading, padding)
4. Add table editing commands:
   - Insert table (with dialog)
   - Insert/delete rows and columns
   - Select table/row/column/cell
5. Implement table navigation (Tab to next cell)
6. Build table formatting toolbar
7. Handle table pagination (basic row breaking)

**Deliverables:**
- Table layout engine
- Table editing commands
- Insert Table dialog
- Table formatting controls

**Note:** Advanced features (nested tables, row spanning, complex breaks) deferred to Phase 2.

---

### B3. Images (Inline + Basic Wrap) ✅ COMPLETE
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on A1 (Style Cascade), Parallel with B2
**Status:** Implemented in `crates/doc_model/src/image.rs` and `crates/store/src/image_store.rs`

**Implementation Steps:**
1. Implement image node in document model
2. Build image resource manager (store, retrieve, cache)
3. Add inline image layout (treated as large glyph)
4. Implement basic floating image layout:
   - Wrap square mode
   - Position relative to paragraph
5. Add image commands:
   - Insert from file
   - Insert from clipboard
   - Resize handles
6. Build image properties dialog (size, wrap, alt text)

**Deliverables:**
- Image model and resource manager
- Inline and basic float layout
- Image insertion and resize UI

---

### B4. Basic Shapes (Inline + Floating) ✅ COMPLETE
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on B3 (Images) — shares floating/anchor system
**Status:** Implemented in `crates/doc_model/src/shape.rs` with 17 shape types, fill, stroke, effects

**Implementation Steps:**
1. Implement shape node in document model
2. Create basic shape library (rectangle, oval, line, arrow)
3. Reuse image floating/anchor system for shape positioning
4. Implement shape rendering (fill, stroke, basic effects)
5. Add shape commands:
   - Insert shape (from gallery)
   - Resize and move
   - Format shape (fill, outline)
6. Build shape formatting panel

**Deliverables:**
- Shape model with basic shape types
- Shape rendering engine
- Shape gallery and formatting UI

---

### B5. Hyperlinks ✅ COMPLETE
**Estimate:** S (days)
**Dependencies:** Independent (builds on Phase 0 inline model)
**Status:** Implemented in `crates/doc_model/src/hyperlink.rs` with URL validation and security checks

**Implementation Steps:**
1. Implement hyperlink inline type in document model
2. Add hyperlink commands:
   - Insert/edit hyperlink
   - Remove hyperlink
3. Implement hyperlink rendering (underline, color)
4. Add click handling with security prompt for external URLs
5. Build hyperlink dialog (URL, display text, tooltip)
6. Support internal document links (to bookmarks)

**Deliverables:**
- Hyperlink model and rendering
- Insert/Edit Hyperlink dialog
- External link security prompt

---

### B6. Bookmarks ✅ COMPLETE
**Estimate:** S (days)
**Dependencies:** Depends on B5 (Hyperlinks use bookmarks for internal links)
**Status:** Implemented in `crates/doc_model/src/bookmark.rs` with BookmarkRegistry and navigation

**Implementation Steps:**
1. Implement bookmark model (name, range)
2. Add bookmark commands:
   - Insert bookmark
   - Delete bookmark
   - Go to bookmark
3. Build bookmark dialog (list, add, delete)
4. Implement bookmark visualization (brackets or hidden marks)
5. Connect to hyperlink system for internal navigation

**Deliverables:**
- Bookmark model
- Bookmark dialog
- Navigation to bookmarks

---

## Group C: Page Layout Features

### C1. Headers/Footers + Page Setup ✅ COMPLETE
**Estimate:** L (2-4 weeks)
**Dependencies:** Depends on A1 (Style Cascade), A2 (Paragraph Formatting)
**Status:** Implemented in `crates/doc_model/src/section.rs` and `crates/doc_model/src/field.rs`

**Implementation Steps:**
1. Implement section model with page setup properties:
   - Page size (A4, Letter, custom)
   - Margins (top, bottom, left, right, gutter)
   - Orientation (portrait, landscape)
2. Implement header/footer model:
   - Default header/footer
   - First page different
   - Odd/even different
3. Update layout engine for header/footer areas
4. Add header/footer editing mode (click to edit)
5. Build Page Setup dialog
6. Implement page number fields in headers/footers

**Deliverables:**
- Section and page setup model
- Header/footer layout and editing
- Page Setup dialog

---

### C2. RTL/BiDi Text Rendering + Cursor Movement ✅ COMPLETE
**Estimate:** L (2-4 weeks)
**Dependencies:** Depends on A3 (Font Fallback), builds on Phase 0 BiDi foundation
**Status:** Implemented in `crates/edit_engine/src/navigation.rs` with BiDiNavigator, visual cursor, selection rendering

**Implementation Steps:**
1. Implement Unicode BiDi Algorithm (UBA) for paragraph layout
2. Update text shaping to handle mixed direction runs
3. Implement visual cursor movement (arrow keys follow visual order)
4. Handle selection rendering for mixed-direction text
5. Implement paragraph direction property (LTR/RTL default)
6. Add UI for paragraph direction toggle
7. Handle tab stops and alignment in RTL context

**Deliverables:**
- BiDi text layout
- Visual cursor movement
- RTL paragraph support

---

## Group D: Import/Export (Critical Path)

### D1. DOCX Import/Export (Core Subset) ✅ COMPLETE
**Estimate:** XL (1-2 months)
**Dependencies:** Depends on A1, A2, B1, B2, B3, C1 (all content types must exist)
**Status:** Implemented in `crates/store/src/docx/` with 19 modules for full import/export support

**Implementation Steps:**

**Import:**
1. Implement OOXML package reader (ZIP + OPC)
2. Parse document.xml for body content
3. Parse styles.xml and map to internal styles
4. Parse numbering.xml for list definitions
5. Parse relationships for images and hyperlinks
6. Import paragraphs with formatting
7. Import tables (basic structure)
8. Import images (inline and floating)
9. Import headers/footers
10. Preserve unknown elements as extension metadata

**Export:**
1. Implement OOXML package writer
2. Serialize document model to document.xml
3. Generate styles.xml from internal styles
4. Generate numbering.xml for lists
5. Write images to media folder
6. Generate relationships file
7. Re-emit preserved unknown elements
8. Validate output against OOXML schema

**Deliverables:**
- DOCX import with ~80% fidelity
- DOCX export with round-trip preservation
- Import error handling and warnings

---

### D2. PDF Export ✅ COMPLETE
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on D1 (uses same layout output)
**Status:** Implemented in `crates/store/src/pdf/` with 11 modules, flate2 compression, standard fonts

**Implementation Steps:**
1. Select PDF library (pdf-writer in Rust or similar)
2. Convert layout tree to PDF commands:
   - Text with fonts and positions
   - Images
   - Shapes and lines
   - Tables
3. Implement font embedding (subset fonts)
4. Add PDF metadata (title, author, etc.)
5. Implement hyperlink annotations
6. Build Export to PDF dialog (options, filename)

**Deliverables:**
- PDF export matching print layout
- Embedded fonts
- Clickable hyperlinks in PDF

---

### D3. Print Preview UI + Pipeline
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on D2 (PDF Export), Parallel development possible

**Implementation Steps:**
1. Build print preview mode UI:
   - Full page view with navigation
   - Page thumbnails
   - Zoom controls
2. Connect to layout engine for page rendering
3. Implement "Print" command connecting to OS print dialog
4. Ensure preview matches PDF output exactly
5. Add print options (page range, copies)

**Deliverables:**
- Print preview mode
- OS print integration
- Print dialog

---

## Group E: Editing Features

### E1. Spellcheck + Find/Replace ✅ COMPLETE
**Estimate:** M (1-2 weeks)
**Dependencies:** Independent (builds on Phase 0 text model)
**Status:** Implemented in `crates/text_engine/src/spellcheck.rs` and `crates/edit_engine/src/find_replace.rs`

**Implementation Steps:**

**Spellcheck:**
1. Integrate spellcheck library (Hunspell or platform API)
2. Implement async spellcheck pipeline (don't block editing)
3. Add misspelled word highlighting (red underline)
4. Build suggestion popup (right-click or hover)
5. Implement "Add to Dictionary" and "Ignore" actions
6. Support multiple languages with detection

**Find/Replace:**
1. Implement text search algorithm
2. Build Find dialog (search text, options)
3. Build Replace dialog (find, replace, options)
4. Add options: case-sensitive, whole word, regex
5. Implement "Find Next", "Replace", "Replace All"
6. Highlight all matches in document

**Deliverables:**
- Inline spellcheck with suggestions
- Find/Replace dialogs
- Language detection

---

### E2. Autosave + Recovery + Integrity Checks ✅ COMPLETE
**Estimate:** M (1-2 weeks)
**Dependencies:** Independent (builds on Phase 0 persistence)
**Status:** Implemented in `crates/store/src/` with autosave, recovery, integrity, versions modules (47 tests)

**Implementation Steps:**
1. Implement autosave timer (configurable interval)
2. Create incremental save format (delta from last save)
3. Build recovery file management:
   - Store recovery files separately
   - Clean up after successful save
4. Implement crash detection on startup
5. Build recovery dialog (list recoverable documents)
6. Add document integrity checks:
   - Checksum validation
   - Structure validation
7. Implement "Recovered Document" notification

**Deliverables:**
- Autosave with configurable interval
- Crash recovery system
- Document integrity validation

---

## Group F: UI Infrastructure

### F1. Accessibility Bridge (ARIA + Keyboard) ✅ COMPLETE
**Estimate:** M (1-2 weeks)
**Dependencies:** Parallel with other UI work
**Status:** Implemented in `frontend/src/lib/AccessibilityBridge.ts` and `KeyboardNavigation.ts` with LiveRegion, high-contrast theme

**Implementation Steps:**
1. Implement accessibility tree for canvas:
   - Map document structure to ARIA roles
   - Expose text content to screen readers
2. Implement keyboard navigation:
   - Full keyboard access to all commands
   - Focus management
   - Tab order
3. Add screen reader announcements for:
   - Cursor position
   - Selection changes
   - Formatting changes
4. Implement high contrast theme
5. Support UI scaling up to 200%

**Deliverables:**
- ARIA-accessible canvas
- Full keyboard navigation
- High contrast theme

---

### F2. Settings/Preferences UI ✅ COMPLETE
**Estimate:** S (days)
**Dependencies:** Independent
**Status:** Implemented in `crates/store/src/settings.rs` with AppSettings, General/Editing/Privacy settings

**Implementation Steps:**
1. Define settings schema:
   - General (language, theme)
   - Editing (autosave interval, default font)
   - Privacy (telemetry opt-in/out)
2. Build settings storage (local file or OS settings)
3. Create Settings dialog with tabs/sections
4. Implement settings change handlers
5. Add "Reset to Defaults" option

**Deliverables:**
- Settings dialog
- Persistent settings storage
- Default configuration

---

### F3. Status Bar ✅ COMPLETE
**Estimate:** S (days)
**Dependencies:** Depends on C1 (page count), E1 (word count), F4 (zoom)
**Status:** Implemented in `frontend/src/components/StatusBar.tsx` with DocumentStatsDialog, GoToDialog, useViewMode

**Implementation Steps:**
1. Build status bar component:
   - Page X of Y indicator
   - Word count
   - Language indicator
   - Zoom slider/dropdown
2. Connect to document model for counts
3. Implement click actions (zoom popup, language selector)
4. Update counts efficiently on document changes

**Deliverables:**
- Status bar with page/word count
- Zoom slider
- Language indicator

---

### F4. Zoom Controls + Print Layout View ✅ COMPLETE
**Estimate:** S (days)
**Dependencies:** Independent
**Status:** Implemented in `frontend/src/components/ZoomControls.tsx` and `frontend/src/hooks/useZoom.ts` with Ruler

**Implementation Steps:**
1. Implement zoom levels (25% - 500%)
2. Build zoom UI:
   - Zoom slider in status bar
   - Zoom dropdown in View tab
   - Keyboard shortcuts (Ctrl+Plus/Minus)
3. Implement Print Layout view mode:
   - Show page boundaries
   - Show margins
   - Show rulers
4. Store zoom level per document

**Deliverables:**
- Zoom controls
- Print Layout view mode
- Persistent zoom per document

---

## Group G: Performance (Final Phase)

### G1. Performance Tuning
**Estimate:** L (2-4 weeks)
**Dependencies:** Depends on all other tasks (optimize the complete system)

**Implementation Steps:**
1. Implement layout caching:
   - Cache paragraph line breaks
   - Invalidate on edit
   - Cache page breaks
2. Implement render virtualization:
   - Only render visible pages
   - Buffer 1-2 pages above/below
3. Profile and optimize:
   - Command execution time
   - Layout time per paragraph
   - Render frame time
4. Implement lazy loading for large documents
5. Add performance telemetry hooks

**Deliverables:**
- Layout cache with efficient invalidation
- Virtualized rendering
- Performance within budget (50ms input latency)

---

## Implementation Schedule

### Sprint 1-2: Foundation ✅ COMPLETE
| Task | Estimate | Status | Deliverables |
|------|----------|--------|--------------|
| A1. Style Cascade | M | ✅ | Style resolution engine, 15 built-in styles, StyleRegistry |
| A3. Font Fallback | M | ✅ | Font discovery, substitution matrix, FontManager |
| B5. Hyperlinks | S | ✅ | Hyperlink model, URL validation, security checks |
| F2. Settings UI | S | ✅ | AppSettings, General/Editing/Privacy settings |
| F4. Zoom Controls | S | ✅ | ZoomControls component, useZoom hook, Ruler |

### Sprint 3-4: Formatting & Content ✅ COMPLETE
| Task | Estimate | Status | Deliverables |
|------|----------|--------|--------------|
| A2. Paragraph Formatting | M | ✅ | Paragraph commands, alignment/indent/spacing |
| B1. Lists + Numbering | M | ✅ | AbstractNum, NumberingInstance, ListLevel, NumberingRegistry |
| B2. Tables | L | ✅ | Table model, TableLayoutEngine, cell properties |
| B3. Images | M | ✅ | ImageNode, ImageStore, format detection |
| B6. Bookmarks | S | ✅ | Bookmark model, BookmarkRegistry, navigation |

### Sprint 5-6: Layout & More Content ✅ COMPLETE
| Task | Estimate | Status | Deliverables |
|------|----------|--------|--------------|
| B4. Basic Shapes | M | ✅ | 17 ShapeTypes, ShapeFill/Stroke/Effects |
| C1. Headers/Footers | L | ✅ | Section model, PageSetup, HeaderFooterSet, fields |
| C2. RTL/BiDi | L | ✅ | BiDiNavigator, visual cursor movement, selection rendering |
| F1. Accessibility | M | ✅ | AccessibilityBridge, KeyboardNavigation, LiveRegion, high-contrast |

### Sprint 7-8: Editing Features ✅ COMPLETE
| Task | Estimate | Status | Deliverables |
|------|----------|--------|--------------|
| E1. Spellcheck + Find/Replace | M | ✅ | SpellChecker, DictionarySpellChecker, FindEngine, ReplaceEngine, squiggly underlines |
| E2. Autosave + Recovery | M | ✅ | AutosaveManager, RecoveryManager, IntegrityChecker, VersionManager |
| F3. Status Bar | S | ✅ | StatusBar, DocumentStatsDialog, GoToDialog, useViewMode |

### Sprint 9-10: Import/Export ✅ COMPLETE
| Task | Estimate | Status | Deliverables |
|------|----------|--------|--------------|
| D1. DOCX Import/Export | XL | ✅ | 19 modules: reader, parser, document, styles, tables, lists, images, hyperlinks, writers, api |
| D2. PDF Export | M | ✅ | 11 modules: objects, document, content, fonts, images, renderer, writer, options, api |

### Sprint 11-12: Print Preview ⏳ PENDING
| Task | Estimate | Status | Deliverables |
|------|----------|--------|--------------|
| D3. Print Preview | M | ⏳ | Print preview UI, page navigation, OS print integration |

### Sprint 13-14: Polish & Performance ⏳ PENDING
| Task | Estimate | Status | Deliverables |
|------|----------|--------|--------------|
| G1. Performance Tuning | L | ⏳ | Layout caching, render virtualization, profiling |

---

## Dependency Graph

```
Phase 0 (Complete)
    │
    ├─► A1 (Style Cascade) ─────┬─► A2 (Paragraph) ─► B1 (Lists)
    │                           │
    │                           ├─► B2 (Tables)
    │                           │
    │                           └─► C1 (Headers/Footers)
    │
    ├─► A3 (Font Fallback) ─────► C2 (RTL/BiDi)
    │
    ├─► B5 (Hyperlinks) ────────► B6 (Bookmarks)
    │
    ├─► B3 (Images) ────────────► B4 (Shapes)
    │
    ├─► F2 (Settings) [Independent]
    │
    ├─► F4 (Zoom) [Independent]
    │
    ├─► E1 (Spellcheck) [Independent]
    │
    ├─► E2 (Autosave) [Independent]
    │
    └─► F1 (Accessibility) [Independent]

    All Content Types ──────────► D1 (DOCX Import/Export)
                                      │
                                      ├─► D2 (PDF Export)
                                      │       │
                                      │       └─► D3 (Print Preview)
                                      │
                                      └─► F3 (Status Bar)

    All Features ───────────────► G1 (Performance Tuning)
```

---

## Risk Mitigation

### High-Risk Items

1. **DOCX Import/Export (D1)**
   - Risk: OOXML complexity, edge cases
   - Mitigation: Start with minimal subset, expand iteratively
   - Mitigation: Build test corpus early, test continuously

2. **Tables (B2)**
   - Risk: Layout complexity, especially with pagination
   - Mitigation: Limit Phase 1 to basic grid (no nested, no complex spans)
   - Mitigation: Defer advanced table features to Phase 2

3. **RTL/BiDi (C2)**
   - Risk: Complex algorithm, affects many systems
   - Mitigation: Phase 0 already includes architecture foundation
   - Mitigation: Use proven Unicode BiDi implementation (icu4x)

4. **Performance (G1)**
   - Risk: May reveal architectural issues late
   - Mitigation: Profile continuously during development
   - Mitigation: Set performance budgets and test regularly

---

## Exit Criteria for Phase 1

Phase 1 is complete when:

1. **Editing:** User can create and edit documents with all basic formatting
2. **Content:** Tables, images, shapes, lists, hyperlinks work correctly
3. **Layout:** Headers/footers, page setup, RTL text render properly
4. **Import/Export:** DOCX opens and saves with ~80% fidelity
5. **PDF:** Export produces accurate PDF matching screen layout
6. **Print:** Print preview and OS print work correctly
7. **Accessibility:** Screen reader and keyboard navigation functional
8. **Performance:** Input latency ≤50ms on 100-page documents
9. **Reliability:** Autosave and crash recovery functional

---

## Estimated Timeline

- **Total Duration:** 14-16 weeks (3.5-4 months)
- **Team Assumption:** 2-3 engineers working in parallel
- **Critical Path:** Style System → Content Types → DOCX Import/Export

The DOCX Import/Export task is the longest single item and defines the critical path. Parallelizing other work around it is essential for meeting timeline goals.
