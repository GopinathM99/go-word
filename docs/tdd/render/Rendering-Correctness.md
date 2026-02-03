# Rendering Correctness Specification

## Coordinate System
- Use device-independent units (points) in layout.
- Convert to device pixels at render time using DPI.

## Pixel Snapping
- Snap text baselines to pixel grid.
- Snap 1px lines to half-pixel to avoid blur.

## Kerning and Ligatures
- Preserve shaping output; do not split glyph clusters.
- Do not apply kerning twice if shaping already includes it.

## DPI Scaling
- Maintain consistent layout across DPI scales.
- Store layout in logical units; scale at render.

## Antialiasing
- Enable subpixel AA where supported.
- For PDF export, use vector text outlines with font embedding.

## Selection and Caret
- Caret width = 1px at 100% zoom, scale with zoom.
- Selection rectangles align to glyph bounds.

## Transparency
- Support alpha blending for highlights and shapes.
- For PDF/A, flatten transparency.
