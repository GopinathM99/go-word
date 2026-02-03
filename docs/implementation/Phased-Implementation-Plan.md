# Phased Implementation Plan (T-Shirt Sizing)

This plan sequences foundational features first, then builds add-ons. Effort is estimated using t-shirt sizes: S, M, L, XL, XXL.

## Assumptions and Core Decisions
- Hybrid cross-platform architecture (Rust core + TypeScript UI).
- Web target via WASM; desktop via wrapper (Electron/Tauri).
- Local-first storage with optional cloud collaboration in later phases.
- Primary formats: DOCX (high fidelity), PDF export, internal WDJ/WDB.

## Basic Features (Foundational, Build First)
- Document model (blocks/inlines, styles, IDs).
- Editing engine (selection, commands, CRDT/OT-compatible undo/redo).
- Layout engine (line breaking + pagination, future-ready for sections/columns/BiDi).
- Rendering pipeline (page-based canvas).
- Styles system (paragraph/character styles + overrides).
- Tables (basic grid, simple layout).
- Images (inline, basic wrapping).
- Shapes (basic inline/floating shapes).
- Hyperlinks and bookmarks.
- Font fallback and substitution system.
- RTL/BiDi text support.
- Import/export: DOCX read/write (core subset), PDF export.
- Autosave, recovery, integrity checks.
- Accessibility bridge (ARIA mapping for canvas + keyboard support).
- Print preview pipeline (matching PDF output).
- Settings/preferences UI (defaults, autosave, privacy).
- Status bar (page/word count, language, zoom).
- Zoom controls and view modes.

## Add-On Features (Built on Top)
- Track changes + comments.
- Advanced layout (sections, columns, keep rules, floats).
- Cross-references and captions.
- Document outline panel and navigation.
- Text boxes and advanced shapes.
- Symbol insertion.
- View modes (Draft, Outline, Web).
- Collaboration (CRDT/OT, presence, version history).
- Templates + locked regions.
- Mail merge.
- Content controls + validation.
- Math/equations and charts/diagrams.
- PDF/A export.
- Plugins/extensions.

---

## Phase 0 — Prototype (Foundational Core)
**Goal:** minimal editor loop with deterministic layout and render.

Tasks and estimates:
- Document tree + IDs + selection model (M)
- Command system with CRDT/OT-compatible ops + inverse undo (L)
- Basic text input (IME-capable) (M)
- Line breaking + pagination with multi-flow-ready layout tree (L)
- BiDi/RTL architecture foundation in layout tree (S)
- Simple renderer (pages, caret, selection) (M)
- Persistence snapshot (S)

**Architecture Notes:**
- Layout tree must support arbitrary content flows for future sections/columns.
- Command model must be CRDT/OT-compatible from day one to avoid Phase 3 rewrites.
- BiDi text direction must be considered in layout box model design.

## Phase 1 — MVP Single-User (Core Editing)
**Goal:** usable editor with core formatting and import/export.

Tasks and estimates:
- Style cascade + inspector basics (M)
- Paragraph formatting (alignment, spacing, indents) (M)
- Lists + numbering (M)
- Tables (grid + basic layout) (L)
- Images (inline + basic wrap) (M)
- Basic shapes (inline + floating with simple wrap) (M)
- Hyperlinks (inline links with URL handling) (S)
- Bookmarks (basic bookmark creation and navigation) (S)
- Headers/footers + page setup (L)
- DOCX import/export (core subset) (XL)
- PDF export (M)
- Print preview UI + pipeline (M)
- Spellcheck + find/replace (M)
- Autosave + recovery + integrity checks (M)
- Accessibility bridge (ARIA mapping + keyboard) (M)
- Settings/preferences UI (S)
- Font fallback + substitution system (M)
- RTL/BiDi text rendering + cursor movement (L)
- Status bar (page/word count, language indicator, zoom slider) (S)
- Zoom controls + Print Layout view mode (S)
- Performance tuning (layout cache, virtualization) (L)

## Phase 2 — Feature Parity (Advanced Layout + Review)
**Goal:** professional document workflows and higher DOCX fidelity.

Tasks and estimates:
- Sections + multi-column layout (L)
- Advanced layout rules (keep with next, widow/orphan) (M)
- Footnotes/endnotes (M)
- Advanced table layout (spans, row breaks, nested tables) (L)
- Track changes (L)
- Comments + review pane (M)
- Fields (PAGE/TOC/REF update rules) (M)
- Cross-references (to headings, figures, tables) (M)
- Captions (figure/table captions with auto-numbering) (S)
- Document outline panel + navigation (M)
- View modes: Draft + Outline (M)
- Text boxes (floating text containers) (M)
- Advanced shapes (editing, styling, grouping) (L)
- Symbol insertion (special characters, common symbols) (S)
- DOCX fidelity improvements (tables, drawings, styles) (XL)
- RTF read/write + ODT read-only import (M)
- Templates + style packs (M)
- PDF/A export (M)

## Phase 3 — Collaboration (Multi-User)
**Goal:** real-time co-editing and presence.

Tasks and estimates:
- Collaboration sync layer + CRDT/OT networking (XL)
- Presence protocol + cursors (M)
- Version history UI + conflict policy (M)
- Permissions + sharing flow (M)
- Collaboration testing + load simulation (L)

## Phase 4 — Ecosystem and Enterprise Add-Ons
**Goal:** extensibility and specialized workflows.

Tasks and estimates:
- Content controls UX + data binding (L)
- Mail merge (M)
- Equation editor (L)
- Charts/diagrams editing (L)
- Plugin/extension system (XL)
- Advanced diagnostics/telemetry tooling (M)

---

## Additional Implementation Details (Summary)
- DOCX round-trip strategy preserves unknown OOXML as metadata; stable ordering and IDs.
- Rendering correctness uses deterministic metrics and pixel snapping.
- Accessibility: full keyboard support, ARIA roles, screen reader compatibility.
- Internationalization: BiDi/RTL handling (Phase 1), IME edge cases, locale-aware fields.
- Security: safe parsing, macro preservation without execution, external link prompts.
- Testing: golden DOCX corpus with pixel-diff PDFs; OOXML conformance tests.
- Print preview matches PDF output; PDF/A is a Phase 2 target.
- RTF write + ODT read-only are Phase 2 deliverables.
- Font fallback uses platform fonts with deterministic substitution rules (Phase 1).
- Hyperlinks support internal bookmarks and external URLs with security prompts.
- Shapes use DrawingML mapping; basic shapes in Phase 1, advanced editing in Phase 2.
- View modes: Print Layout (Phase 1), Draft/Outline (Phase 2).

## Dependency Notes
- Layout engine and selection model are prerequisites for most UI work.
- DOCX import/export requires stable internal schema + style system.
- Collaboration depends on a CRDT/OT-compatible command model from Phase 0.
- BiDi/RTL requires layout engine support; architecture must be planned in Phase 0.
- Font fallback is required before DOCX import to handle missing fonts gracefully.
- Shapes depend on the same floating/anchor system as images.
- Cross-references depend on Fields infrastructure.
- Document outline depends on heading style detection and navigation model.

## T-Shirt Size Guidance
- S: days
- M: 1-2 weeks
- L: 2-4 weeks
- XL: 1-2 months
- XXL: 2+ months
