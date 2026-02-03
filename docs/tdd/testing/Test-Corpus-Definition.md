# Test Corpus Definition

## Corpus Structure
- /corpus/docs/*.docx
- /corpus/reference/*.pdf
- /corpus/metadata/*.json

## Metadata Schema
```json
{
  "id": "doc-001",
  "category": "tables",
  "word_version": "16.x",
  "os": "Windows 11",
  "notes": "Merged cells and banded rows"
}
```

## Inclusion Criteria
- Must cover feature areas in mapping and layout specs.
- Include edge cases: long unbroken words, nested tables, RTL text.

## Update Policy
- Changes require new reference outputs.
- Keep old versions for regression comparisons.
