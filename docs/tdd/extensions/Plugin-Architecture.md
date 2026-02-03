# Plugin and Extension Architecture

## Goals
- Allow third-party features without compromising security.
- Provide a stable API surface.

## Plugin Types
- UI plugins (toolbars, panels)
- Document processors (format converters)
- Command extensions

## Sandbox Model
- Plugins run in isolated processes or web workers.
- Limited API access; no direct file system access.

## Plugin Manifest
```json
{
  "id": "plugin.example",
  "name": "Example Plugin",
  "version": "1.0",
  "entry": "main.js",
  "permissions": ["read-doc", "write-doc"]
}
```

## API Surface
- Document read APIs (read-only snapshot).
- Command dispatch APIs.
- UI injection points.

## Security
- User must approve plugin installation.
- Plugins signed or checksum verified.

## Versioning
- API versioning to prevent breakage.
