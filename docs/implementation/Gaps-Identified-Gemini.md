# Gaps Identified in Implementation Plan

This document captures discrepancies between the PRD/Technical Specs and the current `Phased-Implementation-Plan.md`.

> **Status:** All gaps identified below have been addressed in the updated Phased-Implementation-Plan.md (January 2026 revision).

## Platform Architecture Clarification
The application follows a **Hybrid Cross-Platform** architecture:
- **Core Engine**: Rust (handling document model, layout, import/export, undo/redo for performance).
- **UI Shell**: TypeScript (handling the view layer, inputs, and canvas interactions).
- **Targets**:
    - **Web**: WASM compilation for the core.
    - **Desktop**: Windows and macOS support via a wrapper (e.g., Electron/Tauri) utilizing the same Rust core.

## Missing Features (Gap Analysis)

### 1. RTF & ODT Support
- **Status**: Listed in PRD Scope (Section 5, Functional Requirements) but missing from Phase 1/2 tasks in the implementation plan.
- **Requirement**: Read/write RTF and read-only ODT support.
- **Action**: Should be added to Phase 2 (or Phase 1 depending on fidelity requirements).

### 2. Accessibility Engine
- **Status**: Detailed in `docs/tdd/accessibility/Accessibility-Spec.md` but missing a specific implementation task.
- **Requirement**: Full keyboard support, screen reader compatibility, and ARIA tree mapping for the custom canvas.
- **Action**: Add "Accessibility Bridge (ARIA mapping)" to Phase 1 (Foundational).

### 3. Printing & Print Preview
- **Status**: Distinct from PDF export; currently missing UI and pipeline integration tasks.
- **Requirement**: Accurate print preview matching the PDF export and physical print output.
- **Action**: Add "Print Preview UI & Pipeline" to Phase 1.

### 4. Settings/Preferences UI
- **Status**: Missing from the plan.
- **Requirement**: User interface for application-wide settings (theme, default fonts, autosave intervals).
- **Action**: Add to Phase 1.

## Phasing Logic & Sequencing Risks

### 1. Collaboration (Phase 3) vs. Core Model (Phase 0)
- **Risk**: Deferring "CRDT/OT engine integration" to Phase 3 is a **high risk**.
- **Reasoning**: If the Core Document Model (Phase 0) and Command System (Phase 0) are not designed with distributed concurrency (CRDTs/Operation Transformation) in mind from the start, Phase 3 will likely require a **complete rewrite** of the core engine.
- **Recommendation**: The "Document tree" and "Command system" in Phase 0 must be implemented as a CRDT-compliant structure or an event-sourced model immediately, even if the collaboration network layer is deferred.

### 2. Undo/Redo (Phase 0) Implementation
- **Risk**: A simple stack-based Undo/Redo implemented in Phase 0 will break in Phase 3 (Multi-user).
- **Reasoning**: In a collaborative setting, simple undo stacks cause divergent states.
- **Recommendation**: Implement "Selective Undo" or "Inverse Operation" logic compatible with the chosen CRDT/OT approach in Phase 0.

### 3. Layout Engine Evolution (Phase 0 vs. Phase 2)
- **Risk**: "Line breaking + pagination" (Phase 0) followed by "Sections + multi-column" (Phase 2).
- **Reasoning**: A simple paginator usually assumes a single flow. Adding sections and columns later often requires fundamentally changing the layout tree structure (e.g., from `Page -> Line` to `Page -> Area -> Line`).
- **Recommendation**: Ensure the Phase 0 Layout Engine architecture allows for arbitrary content flows/boxes, even if only one is used initially.

### 4. Tables (Phase 1)
- **Risk**: Placing "Tables" in Phase 1 (MVP) is aggressive.
- **Reasoning**: Tables are arguably the most complex part of a layout engine (nested tables, row spanning, breaking across pages).
- **Recommendation**: Consider splitting Tables: "Basic Grid" in Phase 1, and "Advanced Table Layout" (breaking, spanning) in Phase 2, or acknowledge that Phase 1 might be prolonged by this feature.

---

## Additional Gaps Identified (January 2026 Review)

The following gaps were identified during a comprehensive cross-reference of PRD, PRD-Expanded, and all TDD documents:

### 5. Shapes Support
- **Status**: PRD Section 5 lists "shapes" as in-scope v1; PRD-Expanded 8.5 mentions "Shapes: basic shapes with styling"
- **TDD References**: Document-Schema.md defines Shape node; OOXML-Drawings.md details shape handling
- **Resolution**: ✅ Added "Basic shapes" to Phase 1, "Advanced shapes" to Phase 2

### 6. Hyperlinks
- **Status**: DOCX-Mapping.md maps `w:hyperlink`; Command-Surface.md lists hyperlinks in Insert menu
- **Resolution**: ✅ Added to Phase 1

### 7. Bookmarks
- **Status**: Command-Surface.md lists bookmarks in Insert > Links
- **Resolution**: ✅ Added to Phase 1

### 8. Cross-References
- **Status**: PRD-Expanded Section 8.7 requires "Cross-references to headings, figures, tables"
- **Resolution**: ✅ Added to Phase 2 (depends on Fields)

### 9. Captions
- **Status**: Command-Surface.md lists captions in References tab
- **Resolution**: ✅ Added to Phase 2

### 10. View Modes
- **Status**: PRD Section 6 specifies "View modes: Print, Web, Outline, Draft"
- **Resolution**: ✅ Print Layout added to Phase 1; Draft/Outline added to Phase 2

### 11. BiDi/RTL Text Support
- **Status**: PRD requires RTL languages; dedicated TDD (RTL-and-Bidi.md)
- **Resolution**: ✅ Architecture foundation in Phase 0; implementation in Phase 1

### 12. Font Fallback/Substitution
- **Status**: PRD requires "Font fallback mapping with warnings"; TDD Font-Substitution-Matrix.md
- **Resolution**: ✅ Added to Phase 1

### 13. Document Outline/Navigation Panel
- **Status**: PRD Section 6 specifies "Side panels: styles, comments, outline"
- **Resolution**: ✅ Added to Phase 2

### 14. Status Bar
- **Status**: PRD Section 6 specifies "Status bar: page/word count, language, zoom"
- **Resolution**: ✅ Added to Phase 1

### 15. Zoom Controls
- **Status**: Command-Surface.md lists zoom in View tab
- **Resolution**: ✅ Added to Phase 1

### 16. Symbol Insertion
- **Status**: Command-Surface.md lists symbols in Insert tab
- **Resolution**: ✅ Added to Phase 2

### 17. Text Boxes
- **Status**: Command-Surface.md lists text boxes in Insert > Text
- **Resolution**: ✅ Added to Phase 2

---

## Resolution Summary

| Gap | Phase Added | Status |
|-----|-------------|--------|
| RTF & ODT Support | Phase 2 | ✅ Resolved |
| Accessibility Engine | Phase 1 | ✅ Resolved |
| Print Preview | Phase 1 | ✅ Resolved |
| Settings/Preferences UI | Phase 1 | ✅ Resolved |
| Shapes | Phase 1 (basic), Phase 2 (advanced) | ✅ Resolved |
| Hyperlinks | Phase 1 | ✅ Resolved |
| Bookmarks | Phase 1 | ✅ Resolved |
| Cross-References | Phase 2 | ✅ Resolved |
| Captions | Phase 2 | ✅ Resolved |
| View Modes | Phase 1-2 | ✅ Resolved |
| BiDi/RTL | Phase 0-1 | ✅ Resolved |
| Font Fallback | Phase 1 | ✅ Resolved |
| Document Outline | Phase 2 | ✅ Resolved |
| Status Bar | Phase 1 | ✅ Resolved |
| Zoom Controls | Phase 1 | ✅ Resolved |
| Symbol Insertion | Phase 2 | ✅ Resolved |
| Text Boxes | Phase 2 | ✅ Resolved |

All phasing/sequencing risks have been addressed with architecture notes in Phase 0.
