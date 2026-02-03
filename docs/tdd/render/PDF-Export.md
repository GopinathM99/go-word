# PDF Export — Detailed Spec

## Inputs
- Layout boxes (pages with positioned items).
- Font resources and images.

## Output Goals
- Pixel-perfect match to print view.
- Embedded fonts for portability.

## Pipeline
1. For each page, create PDF page with correct size.
2. Emit text runs with font embedding.
3. Emit vector shapes and lines.
4. Embed images at original resolution.

## Font Embedding
- Subset fonts to reduce size.
- Embed fonts for all glyphs used.

## Image Handling
- Use original raster for PNG/JPEG.
- For SVG or vector, convert to PDF vector when possible.

## Metadata
- Preserve document metadata (title, author).

## Compliance
- Support PDF/A as an optional export mode.
