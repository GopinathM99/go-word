# PDF/A Export Rules

## Goals
- Provide archival-grade PDF/A output.

## Compliance Level
- Support PDF/A-1b in initial release.

## Requirements
- Embed all fonts.
- Use device-independent color spaces.
- Include XMP metadata.
- Disallow transparency.

## Color Management
- Convert RGB to sRGB with embedded ICC profile.
- Avoid device-dependent colors.

## Images
- No JPEG2000 (if not supported by PDF/A-1b).
- Embed images with explicit color space.

## Validation
- Run a PDF/A validator before export completion.
