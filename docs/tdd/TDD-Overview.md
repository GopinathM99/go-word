# TDD Overview — Rust Core + TypeScript UI

## 1. Technology Choices
- Core engine: Rust (document model, layout, import/export, undo/redo).
- UI shell: TypeScript (Web Canvas/WebGL or Desktop via Tauri/Electron).
- IPC: WASM (web) or FFI (desktop).

## 2. High-Level Architecture
### Core Modules (Rust)
- Document Model: typed tree with formatting metadata.
- Editing Engine: command system, selection model, undo/redo.
- Layout Engine: pagination, line breaking, float layout.
- Render Preparation: layout boxes -> render model.
- Import/Export: DOCX, RTF/ODT (read), PDF.
- Text/Fonts: shaping, metrics, fallback.
- Persistence: local format + autosave.

### UI Modules (TypeScript)
- Editor surface (canvas rendering).
- Input/event controller.
- UI shell: toolbar/ribbon, panels, status bar.
- Render scheduler + virtualization.

## 3. Data Flow
Input -> Editing Engine -> Document Model transform -> Layout cache invalidation -> Layout Engine -> Render Model -> UI render

## 4. Rust Core Module Details
### 4.1 Document Model (doc_model)
- Immutable or persistent tree for fast snapshots.
- Node IDs for stable references.
- Node types: Document, Section, Paragraph, Run, Table, Row, Cell, Image, Shape, List, Field.

### 4.2 Editing Engine (edit_engine)
- Command types: InsertText, DeleteRange, ApplyStyle, SplitParagraph, MergeParagraph, InsertTable, InsertImage.
- Selection stored as (nodeId, offset).
- Undo/redo stores inverse ops with batching by input session.

### 4.3 Layout Engine (layout_engine)
- Produces Layout Tree: PageBox -> ColumnBox -> BlockBox -> LineBox -> InlineBox.
- Incremental reflow using paragraph versioning + cache keys.

### 4.4 Render Preparation (render_model)
- Output render items: glyph runs, rectangles, lines, images.
- Render items contain style + positioned bounds.

### 4.5 Import/Export (io)
- DOCX parser and serializer (OOXML).
- PDF export driven by layout tree.

### 4.6 Text Engine (text_engine)
- Font discovery + fallback.
- Glyph shaping (Harfbuzz or platform APIs).
- Hyphenation (optional).

### 4.7 Persistence (store)
- Native format (JSON + binary).
- Autosave snapshots + incremental log.

## 5. TypeScript UI Details
- Event dispatcher converts input to commands.
- Canvas renderer consumes render model.
- Virtualize pages: render only visible pages + buffer.
- Panels for styles, outline, comments.

## 6. IPC Contract
### Commands
- apply_command(payload) -> { selection, change_summary }
- get_layout(viewport) -> RenderModel
- import_docx(bytes) -> doc_id
- export_docx(doc_id) -> bytes

### Events
- document_changed: { changed_nodes, dirty_pages, selection }

## 7. Data Contracts
### RenderModel
- pages: [{ page_index, width, height, items[] }]
- items: { type, bounds, style, text? }

### Selection
- { anchor: { nodeId, offset }, focus: { nodeId, offset } }
