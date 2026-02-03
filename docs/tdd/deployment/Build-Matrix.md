# Build and Deployment Matrix

## Target Platforms
- Windows (x64)
- macOS (arm64/x64)
- Web (modern browsers)

## Desktop Packaging
- Electron or Tauri packaging.
- Auto-update via signed update channels.

## Web Deployment
- Static assets + WASM core.
- CDN for asset delivery.

## Build Variants
- Debug: symbols + verbose logging.
- Release: optimized, logging minimal.

## CI Pipeline
- Lint + unit tests.
- Integration tests for import/export.
- Package builds for each platform.
