# OOXML Parser Conformance Tests

## Coverage Areas
- document.xml structure
- styles.xml (docDefaults, latentStyles)
- numbering.xml
- headers/footers
- footnotes/endnotes
- comments
- drawings and images

## Test Types
1) Schema validation (OOXML XSD)
2) Round-trip tests (no diffs)
3) Error recovery tests

## Example Cases
- Missing styles.xml
- Invalid numbering levels
- Nested tables
- Corrupt relationships
- Unsupported drawing types

## Pass Criteria
- No crashes.
- Best-effort recovery for invalid but salvageable docs.
- Stable re-save output.
