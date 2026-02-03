# RTF and ODT Mapping (High-Level)

## RTF
- \b, \i, \ul -> Run char props
- \fsN -> font size
- \par -> paragraph break
- \tab -> tab
- \trowd -> table row start

## ODT
- content.xml -> document body
- styles.xml -> style definitions
- meta.xml -> metadata

## Mapping Rules
- Preserve style names where possible.
- Treat unsupported constructs as opaque for round-trip.
