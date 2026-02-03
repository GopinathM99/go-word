# Equation and Math Rendering Spec

## Formats
- OOXML math (OMML): m:oMath
- MathML (optional import/export)

## Internal Representation
- Math node with tree of operators and operands.
- Preserve raw OMML for round-trip.

## Rendering
- If native math layout engine exists: render as vector.
- Otherwise render OMML via MathJax to SVG or raster.

## Editing
- Basic equation editor (phase 2).
- If not editable, treat as embedded object.

## Round-Trip
- Preserve OMML as-is if not edited.
- If converted, store original OMML in metadata.
