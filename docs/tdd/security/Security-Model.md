# Security Model

## Threat Model
- Malicious documents (DOCX with malformed XML, oversized media, macro content).
- Path traversal in embedded resources.
- Data exfiltration via external links.

## Safe Parsing
- Use strict XML parsers with entity expansion disabled.
- Reject or cap oversized media (> configurable size).
- Sanitize HTML/RTF input from clipboard.

## Macros
- Do not execute macros (vbaProject.bin) under any circumstance.
- Preserve macro parts for round-trip only.

## External Links
- Prompt user before fetching remote resources.
- Default to blocking external fetch unless allowed.

## Encryption
- Local file encryption optional.
- Cloud sync uses TLS + at-rest encryption.

## Permissions
- Document permissions: view/comment/edit.
- Enforced in collaboration layer.

## Logging
- Sensitive content never logged.
- Error logs contain hashes or redacted snippets only.
