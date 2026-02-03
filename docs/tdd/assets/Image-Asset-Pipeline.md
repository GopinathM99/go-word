# Image and Asset Pipeline

## Import
- Accept PNG, JPEG, SVG, GIF.
- On import, read metadata (size, DPI).

## Storage
- Store original binary in resources.
- Generate optimized preview for UI rendering.

## Resizing
- Preserve original; store transform metadata.
- Use high-quality resampling for preview.

## Caching
- Cache decoded bitmaps per zoom level.
- Evict using LRU based on memory cap.

## Export
- DOCX: embed original if possible.
- PDF: embed raster at current size or original resolution.

## Security
- Strip metadata if user opts for privacy.
