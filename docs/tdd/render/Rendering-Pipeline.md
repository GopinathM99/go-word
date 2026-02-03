# Rendering Pipeline — Detailed Spec

## Render Model
- Pages -> render items.
- Render item types:
  - TextRun, Rect, Line, Image, Shape

## Pipeline Steps
1. Receive layout boxes from core.
2. Convert to render items with device-independent coordinates.
3. Apply zoom and scroll transforms.
4. Rasterize via Canvas/WebGL (web) or native drawing (desktop).

## Tiling and Virtualization
- Render only visible pages + buffer.
- Use tile-based invalidation for large pages.

## Pixel Snapping
- Snap text baselines to device pixels.
- Snap lines to half-pixel for crispness.

## Selection Rendering
- Draw selection rectangles behind text.
- Separate overlay layer for caret and selection.

## Performance Targets
- Render frame budget: <= 16ms for smooth scrolling.
- Avoid full-page re-render on small edits.
