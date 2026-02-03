# Phase Zero Implementation Plan

> **STATUS: ✅ COMPLETE**
> **Completion Date:** January 26, 2026
> **Verified:** January 28, 2026
>
> All 12 tasks in this phase have been fully implemented and verified in the codebase.
> See SESSION-NOTES.md for ongoing project status.

---

## Overview

Phase 0 establishes the **foundational core** of the word processor. The goal is to create a minimal but architecturally sound editor loop with deterministic layout and rendering. Every decision in this phase has long-term implications—the architecture must support future features without requiring rewrites.

**Critical Principle:** Build for Phase 3 (Collaboration) from day one. The document model and command system must be CRDT/OT-compatible even though networking is deferred.

---

## Architecture Decisions (Must Be Made First)

Before implementation begins, these architectural decisions must be finalized:

### 1. Document Model Architecture
- **Decision:** Immutable/persistent tree vs. mutable tree with change tracking
- **Recommendation:** Persistent tree (allows efficient snapshots for undo/collaboration)
- **Impact:** Affects all document operations

### 2. ID Strategy
- **Decision:** How to generate stable, unique node IDs
- **Recommendation:** UUID or Lamport-style logical timestamps
- **Impact:** Required for CRDT compatibility and DOCX round-trip

### 3. Command Model
- **Decision:** Event sourcing vs. command pattern with inverses
- **Recommendation:** Command pattern with explicit inverse operations
- **Impact:** Defines undo/redo and collaboration behavior

### 4. Layout Tree Structure
- **Decision:** Single-flow vs. multi-flow-ready layout boxes
- **Recommendation:** Multi-flow from start (Page → Area → Column → Block → Line → Inline)
- **Impact:** Avoids rewrite when adding sections/columns in Phase 2

### 5. Text Storage
- **Decision:** String per run vs. rope/piece-table
- **Recommendation:** Rope or piece-table for efficient large document editing
- **Impact:** Performance characteristics for all text operations

---

## Task Groups and Dependencies

### Dependency Legend
- **Independent**: Can start immediately
- **Depends on [X]**: Requires task X to be complete first
- **Parallel with [X]**: Can be developed alongside task X

---

## Group A: Document Model (Foundation of Everything)

### A1. Document Tree Structure
**Estimate:** M (1-2 weeks)
**Dependencies:** Independent (first task)

**Implementation Steps:**

1. **Define core node types in Rust:**
   ```
   Document
   ├── Section (placeholder for Phase 2)
   │   └── Block (Paragraph | Table | Image | Shape)
   │       └── Inline (Run | Field | Hyperlink)
   │           └── Text content
   ```

2. **Implement node traits:**
   - `Node` trait with common interface
   - `id()` - returns unique NodeId
   - `node_type()` - returns enum of node types
   - `children()` - returns child iterator
   - `parent()` - returns parent reference

3. **Implement persistent/immutable tree:**
   - Copy-on-write semantics for modifications
   - Efficient structural sharing
   - Snapshot capability for undo history

4. **Implement node ID generation:**
   - UUID-based or logical clock-based IDs
   - IDs must be stable across sessions (for collaboration)
   - IDs must survive document serialization

5. **Implement basic node types:**
   - Document root node
   - Paragraph node with text runs
   - Run node with text content and style reference

6. **Implement tree traversal utilities:**
   - Depth-first iteration
   - Find node by ID
   - Find path from root to node

**Deliverables:**
- `doc_model` Rust crate with core types
- Persistent tree implementation
- Node ID system
- Tree traversal utilities

**Architecture Notes:**
- Leave extension points for Section, Table, Image nodes (implement structure, not behavior)
- Store formatting as style references + override maps (not inline properties)

---

### A2. Selection Model
**Estimate:** S (days) — can be part of A1
**Dependencies:** Depends on A1 (Document Tree)

**Implementation Steps:**

1. **Define selection types:**
   ```rust
   struct Position {
       node_id: NodeId,
       offset: usize,  // character offset within node
   }

   struct Selection {
       anchor: Position,  // where selection started
       focus: Position,   // where selection ends (caret)
       // anchor == focus means collapsed (caret only)
   }
   ```

2. **Implement position resolution:**
   - Resolve Position to actual document location
   - Handle deleted nodes gracefully
   - Normalize positions (e.g., end of node → start of next)

3. **Implement selection operations:**
   - `is_collapsed()` - check if caret only
   - `is_forward()` - check selection direction
   - `contains(position)` - check if position in selection
   - `expand_to_word()` - select current word
   - `expand_to_paragraph()` - select current paragraph

4. **Implement selection validation:**
   - Ensure positions are valid after document changes
   - Clamp to valid ranges

**Deliverables:**
- Selection model types
- Position resolution
- Selection operations

---

## Group B: Command System (Critical for Collaboration)

### B1. Command Infrastructure
**Estimate:** L (2-4 weeks)
**Dependencies:** Depends on A1, A2 (Document Tree + Selection)

**Implementation Steps:**

1. **Define command trait:**
   ```rust
   trait Command {
       fn apply(&self, doc: &Document) -> Result<Document, Error>;
       fn invert(&self, doc: &Document) -> Box<dyn Command>;
       fn transform_selection(&self, sel: &Selection) -> Selection;
       fn merge_with(&self, other: &dyn Command) -> Option<Box<dyn Command>>;
   }
   ```

2. **Implement core text commands:**
   - `InsertText { position, text }` - insert text at position
   - `DeleteRange { start, end }` - delete text range
   - `ReplaceRange { start, end, text }` - replace text

3. **Implement structural commands:**
   - `SplitParagraph { position }` - split at position (Enter key)
   - `MergeParagraph { position }` - merge with previous (Backspace at start)
   - `InsertParagraph { after_node_id }` - insert new paragraph

4. **Implement formatting commands (stubs for Phase 1):**
   - `ApplyCharacterStyle { range, style_id }`
   - `ApplyParagraphStyle { node_id, style_id }`
   - `SetCharacterOverride { range, property, value }`

5. **Implement command execution engine:**
   - Execute command and get new document state
   - Track selection transform
   - Emit change events

**Deliverables:**
- Command trait and infrastructure
- Core text editing commands
- Command execution engine

**Architecture Notes:**
- Commands must be serializable (for collaboration sync)
- Commands must have deterministic inverses
- Commands should be as granular as possible (aids conflict resolution)

---

### B2. Undo/Redo with Inverse Operations
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on B1 (Command Infrastructure)

**Implementation Steps:**

1. **Design undo model:**
   - NOT a simple state stack (breaks in collaboration)
   - Store command + inverse pairs
   - Support selective undo (undo my changes, not others')

2. **Implement undo stack:**
   ```rust
   struct UndoEntry {
       command: Box<dyn Command>,
       inverse: Box<dyn Command>,
       timestamp: LogicalTime,
       author_id: Option<UserId>,  // for collaborative undo
   }

   struct UndoManager {
       undo_stack: Vec<UndoEntry>,
       redo_stack: Vec<UndoEntry>,
   }
   ```

3. **Implement command batching:**
   - Batch typing bursts into single undo entry
   - Time-based batching (e.g., 500ms threshold)
   - Explicit batch boundaries (e.g., after paste)

4. **Implement IME composition handling:**
   - Entire IME composition = single undo entry
   - Don't create undo entries during composition

5. **Implement undo/redo operations:**
   - `undo()` - apply inverse of last command
   - `redo()` - re-apply last undone command
   - Clear redo stack on new edit

6. **Implement undo stack limits:**
   - Limit by count (e.g., 100 entries)
   - Limit by memory usage
   - Coalesce old entries if needed

**Deliverables:**
- Undo manager with command-based history
- Command batching logic
- IME composition support

**Architecture Notes:**
- Design supports "selective undo" for Phase 3 collaboration
- Author ID field enables "undo my changes only"

---

## Group C: Text Input

### C1. Basic Text Input (IME-Capable)
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on A1, A2, B1 (Document + Selection + Commands)

**Implementation Steps:**

1. **Implement input event handling (TypeScript side):**
   - Capture keyboard events
   - Capture IME composition events
   - Route to appropriate handlers

2. **Implement basic key handlers:**
   - Printable characters → InsertText command
   - Backspace → DeleteRange (before caret) or MergeParagraph
   - Delete → DeleteRange (after caret)
   - Enter → SplitParagraph command

3. **Implement IME support:**
   - `compositionstart` → begin composition mode
   - `compositionupdate` → update preview text
   - `compositionend` → commit final text as single command
   - Handle composition cancellation

4. **Implement text input IPC:**
   - TypeScript sends input events to Rust core
   - Rust returns document changes + new selection
   - TypeScript updates UI

5. **Implement dead key handling:**
   - Buffer dead keys (e.g., ´ + e = é)
   - Support combining characters

6. **Implement clipboard operations:**
   - Cut → copy selection + DeleteRange
   - Copy → extract text from selection
   - Paste → InsertText (plain text for Phase 0)

**Deliverables:**
- Keyboard event handling
- IME composition support
- Basic clipboard operations (plain text)

**Architecture Notes:**
- IME support is essential for CJK languages
- Must work correctly with undo (composition = single undo entry)

---

### C2. Cursor/Caret Navigation
**Estimate:** S (days) — can be parallel with C1
**Dependencies:** Depends on A2 (Selection Model)

**Implementation Steps:**

1. **Implement character navigation:**
   - Left/Right arrow → move by grapheme cluster
   - Handle multi-codepoint characters (emoji, combining marks)

2. **Implement word navigation:**
   - Ctrl/Cmd + Left/Right → move by word
   - Implement word boundary detection (Unicode-aware)

3. **Implement line navigation:**
   - Up/Down arrow → move to same visual X on adjacent line
   - Home/End → move to line start/end
   - Requires layout information (Group D dependency)

4. **Implement selection extension:**
   - Shift + any navigation → extend selection
   - Track anchor vs. focus correctly

5. **Implement paragraph navigation:**
   - Ctrl/Cmd + Up/Down → move by paragraph

6. **Implement document navigation:**
   - Ctrl/Cmd + Home/End → move to document start/end

**Deliverables:**
- Character/word/line/paragraph navigation
- Selection extension with Shift
- Keyboard navigation commands

---

## Group D: Layout Engine (Core Algorithm)

### D1. Line Breaking Algorithm
**Estimate:** L (2-4 weeks)
**Dependencies:** Depends on A1 (Document Tree)

**Implementation Steps:**

1. **Implement text shaping pipeline:**
   - Integrate text shaping library (HarfBuzz or rustybuzz)
   - Shape text runs into glyph sequences
   - Get glyph advances and metrics

2. **Implement break opportunity detection:**
   - Use Unicode Line Breaking Algorithm (UAX #14)
   - Identify soft break points (spaces, hyphens)
   - Handle no-break spaces and other special cases

3. **Implement greedy line filling:**
   ```
   for each run in paragraph:
       shape run into glyphs
       for each glyph:
           if current_line_width + glyph_width > available_width:
               break at last opportunity
               start new line
           add glyph to current line
   ```

4. **Implement line metrics calculation:**
   - Line height = max(ascent + descent) across all runs
   - Baseline alignment
   - Handle mixed font sizes

5. **Implement paragraph spacing:**
   - Space before/after paragraph
   - Line spacing (single, 1.5, double, exact)

6. **Implement basic alignment:**
   - Left, center, right alignment
   - Justify (distribute extra space) — basic implementation

**Deliverables:**
- Text shaping integration
- Line breaking algorithm
- Line and paragraph metrics

---

### D2. Pagination Algorithm
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on D1 (Line Breaking)

**Implementation Steps:**

1. **Define layout tree structure:**
   ```
   LayoutTree
   └── PageBox
       └── AreaBox (content area, header area, footer area)
           └── ColumnBox (single column for Phase 0)
               └── BlockBox (paragraph, table, image)
                   └── LineBox
                       └── InlineBox (glyph run)
   ```

2. **Implement page creation:**
   - Create pages based on page size and margins
   - Content area = page size - margins
   - Leave space for header/footer areas (empty in Phase 0)

3. **Implement block pagination:**
   - Place blocks sequentially in column
   - When block doesn't fit:
     - If splittable (paragraph): split at line boundary
     - If not splittable: move to next page

4. **Implement paragraph splitting:**
   - Split paragraphs at line boundaries
   - Create continuation block on next page

5. **Implement layout cache:**
   - Cache line breaks per paragraph (keyed by paragraph version)
   - Invalidate on paragraph edit
   - Reuse cached lines during pagination

6. **Implement incremental reflow:**
   - On edit: mark affected paragraph dirty
   - Reflow from dirty paragraph forward
   - Stop when page breaks stabilize

**Deliverables:**
- Layout tree structure
- Pagination algorithm
- Basic layout cache

**Architecture Notes:**
- AreaBox/ColumnBox structure supports future multi-column (Phase 2)
- Header/footer areas defined but empty (populated in Phase 1)

---

### D3. BiDi/RTL Architecture Foundation
**Estimate:** S (days)
**Dependencies:** Depends on D1 (Line Breaking)

**Implementation Steps:**

1. **Design BiDi-aware layout boxes:**
   - InlineBox has `direction` property (LTR/RTL)
   - LineBox tracks base direction
   - Support mixed-direction content in single line

2. **Integrate BiDi algorithm:**
   - Use icu4x or unicode-bidi crate
   - Run BiDi algorithm per paragraph
   - Segment runs by direction

3. **Design visual vs. logical ordering:**
   - Document stores logical order
   - Layout produces visual order
   - Track mapping between them (for cursor movement)

4. **Stub cursor movement for BiDi:**
   - Left arrow = visual left (not logical previous)
   - Design data structure to support this in Phase 1

**Deliverables:**
- BiDi-aware layout box design
- BiDi algorithm integration
- Logical-to-visual mapping structure

**Note:** Full BiDi rendering and cursor movement implemented in Phase 1. Phase 0 establishes architecture only.

---

## Group E: Rendering

### E1. Simple Renderer
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on D1, D2 (Layout Engine)

**Implementation Steps:**

1. **Define render model:**
   ```rust
   struct RenderItem {
       item_type: RenderItemType,
       bounds: Rect,
       // type-specific data
   }

   enum RenderItemType {
       GlyphRun { glyphs: Vec<Glyph>, font: FontId, color: Color },
       Rectangle { fill: Option<Color>, stroke: Option<Stroke> },
       Image { src: ImageId },
       Caret { position: Point, height: f32 },
       Selection { rects: Vec<Rect>, color: Color },
   }
   ```

2. **Implement layout-to-render conversion:**
   - Walk layout tree
   - Generate render items for each box
   - Position items using box coordinates

3. **Implement canvas renderer (TypeScript):**
   - Receive render model via IPC
   - Draw glyph runs using canvas text APIs or WebGL
   - Draw rectangles, images

4. **Implement caret rendering:**
   - Calculate caret position from selection + layout
   - Render as blinking vertical line
   - Implement blink animation (500ms on/off)

5. **Implement selection rendering:**
   - Calculate selection rectangles from range + layout
   - Handle multi-line selections
   - Render with semi-transparent highlight

6. **Implement page rendering:**
   - Render page boundaries (shadow/border)
   - Render page background
   - Clip content to page bounds

**Deliverables:**
- Render model types
- Layout-to-render conversion
- Canvas renderer for text, caret, selection

---

### E2. Render Scheduling
**Estimate:** S (days)
**Dependencies:** Depends on E1 (Renderer)

**Implementation Steps:**

1. **Implement dirty tracking:**
   - Track which pages need re-render
   - Track caret blink state

2. **Implement render loop:**
   - Request animation frame
   - Check dirty state
   - Render only dirty pages
   - Maintain 60fps target

3. **Implement viewport tracking:**
   - Track visible pages
   - Only render visible + buffer pages

4. **Implement scroll handling:**
   - Update viewport on scroll
   - Trigger re-render for newly visible pages

**Deliverables:**
- Dirty tracking system
- RAF-based render loop
- Viewport-aware rendering

---

## Group F: Persistence

### F1. Persistence Snapshot
**Estimate:** S (days)
**Dependencies:** Depends on A1 (Document Tree)

**Implementation Steps:**

1. **Define internal JSON format (.wdj):**
   - Serialize document tree to JSON
   - Include all node IDs
   - Include style references

2. **Implement serialization:**
   - Document → JSON string
   - Handle all node types
   - Preserve node IDs exactly

3. **Implement deserialization:**
   - JSON string → Document
   - Validate structure
   - Rebuild tree with preserved IDs

4. **Implement file I/O:**
   - Save to file (async)
   - Load from file (async)
   - Handle errors gracefully

5. **Implement dirty tracking:**
   - Track unsaved changes
   - Prompt on close if dirty

**Deliverables:**
- Internal JSON format specification
- Serialize/deserialize implementation
- Basic file save/load

**Architecture Notes:**
- Format designed for easy diffing (stable key order)
- IDs preserved for future collaboration merge

---

## Implementation Schedule

### Week 1-2: Document Model
| Task | Estimate | Dependencies |
|------|----------|--------------|
| A1. Document Tree | M | Start |
| A2. Selection Model | S | After A1 (or parallel late in week) |

### Week 3-4: Command System
| Task | Estimate | Dependencies |
|------|----------|--------------|
| B1. Command Infrastructure | L | After A1, A2 |
| B2. Undo/Redo | M | After B1 |

### Week 5-6: Text Input + Layout Start
| Task | Estimate | Dependencies |
|------|----------|--------------|
| C1. Text Input | M | After B1 |
| C2. Cursor Navigation | S | After A2, parallel with C1 |
| D1. Line Breaking | L | After A1 (parallel with C1) |

### Week 7-8: Layout + Rendering
| Task | Estimate | Dependencies |
|------|----------|--------------|
| D2. Pagination | M | After D1 |
| D3. BiDi Foundation | S | After D1 |
| E1. Simple Renderer | M | After D2 |
| E2. Render Scheduling | S | After E1 |

### Week 9: Persistence + Integration
| Task | Estimate | Dependencies |
|------|----------|--------------|
| F1. Persistence Snapshot | S | After A1 |
| Integration & Testing | M | After all tasks |

---

## Dependency Graph

```
START
  │
  ▼
A1 (Document Tree) ──────────────────────────────────┐
  │                                                   │
  ├──► A2 (Selection Model)                          │
  │         │                                         │
  │         ▼                                         │
  │    B1 (Command Infrastructure) ◄─────────────────┤
  │         │                                         │
  │         ├──► B2 (Undo/Redo)                      │
  │         │                                         │
  │         └──► C1 (Text Input)                     │
  │                   │                               │
  │                   └──► C2 (Cursor Navigation)    │
  │                                                   │
  └──► D1 (Line Breaking) ◄──────────────────────────┘
            │
            ├──► D2 (Pagination)
            │         │
            │         └──► E1 (Renderer)
            │                   │
            │                   └──► E2 (Render Scheduling)
            │
            └──► D3 (BiDi Foundation)

A1 ──► F1 (Persistence) [can run parallel with B/C/D]

All Tasks ──► Integration Testing
```

---

## Parallel Work Opportunities

These task pairs can be developed simultaneously by different engineers:

| Engineer 1 | Engineer 2 |
|------------|------------|
| A1 Document Tree | — |
| B1 Commands | D1 Line Breaking (after A1) |
| B2 Undo | D2 Pagination |
| C1 Text Input | E1 Renderer |
| C2 Navigation | F1 Persistence |

---

## Technical Specifications

### IPC Contract (Rust ↔ TypeScript)

```typescript
// TypeScript → Rust
interface InputEvent {
  type: 'keydown' | 'compositionstart' | 'compositionupdate' | 'compositionend';
  key?: string;
  data?: string;
  modifiers: { ctrl: boolean; shift: boolean; alt: boolean; meta: boolean };
}

// Rust → TypeScript
interface DocumentChange {
  changed_nodes: NodeId[];
  dirty_pages: number[];
  selection: Selection;
  render_model?: RenderModel;  // if requested
}

// Render model
interface RenderModel {
  pages: PageRender[];
}

interface PageRender {
  page_index: number;
  width: number;
  height: number;
  items: RenderItem[];
}
```

### Performance Budgets

| Operation | Target |
|-----------|--------|
| Keystroke → render | ≤50ms |
| Paragraph reflow | ≤5ms |
| Page render | ≤10ms |
| Document load (10 pages) | ≤500ms |

---

## Risk Mitigation

### 1. CRDT Compatibility
- **Risk:** Design decisions that break collaboration later
- **Mitigation:** Review all command designs against CRDT requirements
- **Mitigation:** Ensure all operations have deterministic inverses
- **Mitigation:** Use logical timestamps for ordering

### 2. Performance
- **Risk:** Slow layout or rendering
- **Mitigation:** Profile continuously during development
- **Mitigation:** Implement layout cache from start
- **Mitigation:** Use incremental reflow

### 3. IME Complexity
- **Risk:** Broken CJK input
- **Mitigation:** Test with actual IME (Chinese, Japanese, Korean)
- **Mitigation:** Handle all composition events correctly
- **Mitigation:** Ensure undo works with IME

### 4. Cross-Platform Consistency
- **Risk:** Different behavior on Web vs. Desktop
- **Mitigation:** Core logic 100% in Rust (shared)
- **Mitigation:** Deterministic layout (same fonts → same output)
- **Mitigation:** Use identical font metrics

---

## Exit Criteria for Phase 0

Phase 0 is complete when:

1. **Document Model:**
   - Can create document with paragraphs and text runs
   - Node IDs are stable and unique
   - Selection model works correctly

2. **Editing:**
   - Can type text and see it appear
   - Backspace/Delete work
   - Enter creates new paragraph
   - IME input works (Chinese/Japanese/Korean)
   - Undo/Redo work correctly

3. **Layout:**
   - Text wraps at page width
   - Multiple pages are created when content overflows
   - Line breaking follows Unicode rules

4. **Rendering:**
   - Text renders correctly on canvas
   - Caret blinks at correct position
   - Selection highlights correctly
   - Scrolling works

5. **Persistence:**
   - Document can be saved to JSON file
   - Document can be loaded from JSON file
   - Round-trip preserves all content and IDs

6. **Architecture:**
   - Command system is CRDT-compatible
   - Layout tree supports future multi-column
   - BiDi architecture foundation is in place

---

## Estimated Timeline

- **Total Duration:** 8-10 weeks (2-2.5 months)
- **Team Assumption:** 1-2 engineers
- **Critical Path:** Document Model → Command System → Layout Engine → Renderer

Phase 0 is intentionally minimal—resist the temptation to add features. The goal is architectural soundness, not feature completeness.
