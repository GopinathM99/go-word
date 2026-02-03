# Security Review Checklist

## Input Validation
- [ ] XML parsing with entity expansion disabled.
- [ ] Size limits for embedded media.
- [ ] Reject invalid zip bombs (compressed size vs uncompressed).

## File System Safety
- [ ] Prevent path traversal in extracted parts.
- [ ] Sanitize file names for extracted media.

## Macro Safety
- [ ] Do not execute macros.
- [ ] Preserve macro parts for round-trip only.

## External Links
- [ ] Disable remote image fetch by default.
- [ ] Prompt user for external link activation.

## Fuzzing
- [ ] Fuzz DOCX parser with malformed XML.
- [ ] Fuzz RTF and HTML import.

## Logging
- [ ] No document text in logs.
- [ ] Redact filenames or hash.
