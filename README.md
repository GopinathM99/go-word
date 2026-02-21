# Documentation Index

This folder contains the product and technical design documents for the Word-like editor.

---

## Project Status

**All implementation phases are complete.**

| Phase | Status | Description |
|-------|--------|-------------|
| [Phase 0](implementation/Phase-Zero-Implementation-Plan.md) | ✅ Complete | Core foundation: document model, command system, layout engine |
| [Phase 1](implementation/Phase-One-Implementation-Plan.md) | ✅ Complete | Single-user MVP: formatting, tables, images, import/export |
| [Phase 2](implementation/Phase-Two-Implementation-Plan.md) | ✅ Complete | Advanced features: sections, footnotes, track changes, fields |
| [Phase 3](implementation/Phase-Three-Implementation-Plan.md) | ✅ Complete | Real-time collaboration: CRDT, WebSocket server, presence |
| [Phase 4](implementation/Phase-Four-Implementation-Plan.md) | ✅ Complete | Enterprise: content controls, mail merge, equations, plugins |

---

## Quick Start

### Prerequisites

- Rust 1.75+ (with cargo)
- Node.js 18+ (for frontend)
- pnpm or npm

### Building the Project

```bash
# Build all Rust crates
cargo build --release

# Build frontend
cd frontend && pnpm install && pnpm build
```

### Running the Desktop App (Tauri)

```bash
# Development mode
cd src-tauri && cargo tauri dev

# Production build
cd src-tauri && cargo tauri build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test --package collab          # Collaboration (77 tests)
cargo test --package doc_model       # Document model
cargo test --package layout_engine   # Layout engine
cargo test --package edit_engine     # Edit commands
```

---

## Collaboration Server

The collaboration server enables real-time multi-user editing. It uses WebSockets with CRDT-based conflict resolution.

### Starting the Server

#### Option 1: Using the Server Binary

```bash
# Build with server feature
cargo build --release --package collab --features server

# Create a simple server binary (src/bin/collab_server.rs)
```

#### Option 2: Embedding in Your Application

```rust
use collab::server::{CollaborationServer, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the server
    let config = ServerConfig {
        port: 8080,
        bind_address: "0.0.0.0".to_string(),
        max_connections: 1000,
        max_connections_per_document: 50,
        heartbeat_interval_secs: 30,
        connection_timeout_secs: 60,
    };

    // Create and run server
    let server = CollaborationServer::new(config);

    // Optional: Get shutdown handle for graceful shutdown
    let shutdown_handle = server.shutdown_handle();

    // Handle Ctrl+C
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        shutdown_handle.shutdown();
    });

    println!("Collaboration server running on ws://0.0.0.0:8080");
    server.run().await?;

    Ok(())
}
```

#### Option 3: Quick Start with Defaults

```rust
use collab::server::{CollaborationServer, ServerConfig};

#[tokio::main]
async fn main() {
    let server = CollaborationServer::new(ServerConfig::with_port(8080));
    server.run().await.unwrap();
}
```

### Cargo.toml Configuration

Add the collab crate with the server feature:

```toml
[dependencies]
collab = { path = "crates/collab", features = ["server"] }
tokio = { version = "1", features = ["full"] }
```

### Server Features

| Feature | Description |
|---------|-------------|
| WebSocket Protocol | tokio-tungstenite based, compatible with browser WebSocket API |
| Authentication | Pluggable `AuthProvider` trait for custom auth |
| Document Sessions | Per-document client tracking and broadcasting |
| Operation Routing | Routes CRDT operations to correct document sessions |
| Presence System | Real-time cursor/selection sharing |
| Persistence | `OperationStore` trait with memory and file implementations |
| Graceful Shutdown | Clean disconnect of all clients |

### Connecting from Frontend

The frontend `CollaborationClient` connects to the server:

```typescript
import { CollaborationClient } from './lib/collaboration/CollaborationClient';

const client = new CollaborationClient('ws://localhost:8080');

// Authenticate
await client.connect();
await client.authenticate({ token: 'user-token' });

// Join a document
await client.joinDocument('doc-123');

// Send operations
client.sendOperation({
  type: 'insert',
  position: { nodeId: 'para-1', offset: 0 },
  content: 'Hello'
});

// Listen for remote operations
client.onOperation((op) => {
  console.log('Remote operation:', op);
});

// Update presence
client.updatePresence({
  cursor: { nodeId: 'para-1', offset: 5 },
  selection: null
});
```

### Message Protocol

The server uses JSON messages over WebSocket:

**Client to Server:**
- `auth` - Authenticate with token
- `join` - Join a document session
- `leave` - Leave a document session
- `ops` - Send CRDT operations
- `presence` - Update cursor/selection
- `sync_request` - Request missed operations
- `ping` - Keep-alive

**Server to Client:**
- `auth_success` / `auth_error` - Auth response
- `joined` - Successfully joined document
- `user_joined` / `user_left` - Presence notifications
- `ops` - Remote operations
- `presence` - Other users' cursors
- `sync_response` - Catch-up operations
- `error` - Error messages
- `pong` - Keep-alive response

### Storage Options

#### In-Memory (Development/Testing)

```rust
use collab::server::MemoryOperationStore;

let store = MemoryOperationStore::new();
```

#### File-Based (Local Persistence)

```rust
use collab::server::FileOperationStore;

let store = FileOperationStore::new("./data")?;
// Creates: ./data/{doc_id}/operations.jsonl
//          ./data/{doc_id}/snapshot.json
```

#### Custom Storage

Implement the `OperationStore` trait for databases:

```rust
use collab::server::{OperationStore, StorageResult, StoredOperation, Snapshot, Version};

struct PostgresStore { /* ... */ }

impl OperationStore for PostgresStore {
    async fn save_operation(&self, doc_id: &str, op: CrdtOp) -> StorageResult<Version> {
        // Save to PostgreSQL
    }

    async fn get_operations_since(&self, doc_id: &str, version: Version) -> StorageResult<Vec<StoredOperation>> {
        // Query from PostgreSQL
    }

    // ... other methods
}
```

---

## Project Structure

```
go-word/
├── crates/                    # Rust crates
│   ├── doc_model/            # Document tree, nodes, styles
│   ├── edit_engine/          # Commands, undo/redo, navigation
│   ├── layout_engine/        # Line breaking, pagination, layout
│   ├── render_model/         # Render items, viewport
│   ├── text_engine/          # Font handling, shaping, spellcheck
│   ├── store/                # DOCX/PDF/RTF import/export, autosave
│   ├── collab/               # CRDT, sync, collaboration server
│   ├── math/                 # Equation editor (OMML)
│   ├── charts/               # Chart rendering (DrawingML)
│   ├── mail_merge/           # Mail merge engine
│   ├── plugins/              # Plugin system
│   ├── perf/                 # Performance profiling
│   └── telemetry/            # Crash reporting, diagnostics
├── frontend/                  # React/TypeScript UI
│   └── src/
│       ├── components/       # UI components
│       ├── hooks/            # React hooks
│       └── lib/              # Utilities, collaboration client
├── src-tauri/                 # Tauri desktop app
└── docs/                      # Documentation
    ├── implementation/       # Phase implementation plans
    ├── prd/                  # Product requirements
    └── tdd/                  # Technical design documents
```

---

## Implementation Plans

- [Phase Zero](implementation/Phase-Zero-Implementation-Plan.md) - Foundation
- [Phase One](implementation/Phase-One-Implementation-Plan.md) - Single-User MVP
- [Phase Two](implementation/Phase-Two-Implementation-Plan.md) - Advanced Features
- [Phase Three](implementation/Phase-Three-Implementation-Plan.md) - Collaboration
- [Phase Four](implementation/Phase-Four-Implementation-Plan.md) - Enterprise Features
- [Phased Implementation Plan](implementation/Phased-Implementation-Plan.md) - Overview
- [Session Notes](implementation/SESSION-NOTES.md) - Development log

---

## PRD

- [PRD](prd/PRD.md) - Product Requirements Document
- [PRD Expanded](prd/PRD-Expanded.md) - Detailed requirements

---

## TDD - Technical Design Documents

### Core Architecture
- [TDD Overview](tdd/TDD-Overview.md)
- [Internal Format](tdd/Internal-Format.md)
- [Document Schema](tdd/schema/Document-Schema.md)
- [Binary Format Spec](tdd/Binary-Format-Spec.md)

### DOCX/OOXML
- [DOCX Mapping](tdd/DOCX-Mapping.md)
- [DOCX Roundtrip](tdd/DOCX-Roundtrip.md)
- [OOXML Parts](tdd/ooxml/OOXML-Parts.md)
- [OOXML Styles](tdd/ooxml/OOXML-Styles.md)
- [OOXML Numbering](tdd/ooxml/OOXML-Numbering.md)
- [OOXML Tables](tdd/ooxml/OOXML-Tables.md)
- [OOXML Fields](tdd/ooxml/OOXML-Fields.md)
- [OOXML Track Changes](tdd/ooxml/OOXML-Track-Changes.md)
- [OOXML Content Controls](tdd/ooxml/OOXML-Content-Controls.md)
- [OOXML Drawings](tdd/ooxml/OOXML-Drawings.md)

### Layout Engine
- [Layout Algorithm](tdd/Layout-Algorithm.md)
- [Line Breaking & Justification](tdd/layout/LineBreaking-Justification.md)
- [Pagination Rules](tdd/layout/Pagination-Rules.md)
- [Table Layout](tdd/layout/Table-Layout.md)
- [Float Layout](tdd/layout/Float-Layout.md)
- [Layout Cache](tdd/layout/Layout-Cache.md)
- [Advanced Layout Rules](tdd/layout-advanced/Advanced-Layout-Rules.md)

### Rendering
- [Rendering Pipeline](tdd/render/Rendering-Pipeline.md)
- [Font Handling](tdd/render/Font-Handling.md)
- [PDF Export](tdd/render/PDF-Export.md)
- [PDF/A Export](tdd/pdf/PDF-A-Export.md)

### Editor
- [Command System](tdd/editor/Command-System.md)
- [Command Catalog](tdd/editor/Command-Catalog.md)
- [Selection Model](tdd/editor/Selection-Model-Detailed.md)
- [Selection & IME](tdd/Selection-IME.md)
- [Clipboard Import](tdd/editor/Clipboard-Import.md)
- [Find & Replace](tdd/editor/Find-Replace.md)
- [Spellcheck](tdd/editor/Spellcheck.md)
- [Styles Cascade](tdd/editor/Styles-Cascade.md)

### Collaboration
- [CRDT Model](tdd/collab/CRDT-Model.md)
- [Presence Protocol](tdd/collab/Presence-Protocol.md)
- [Conflict Resolution](tdd/collab/Conflict-Resolution.md)

### Storage
- [Autosave Log](tdd/storage/Autosave-Log.md)
- [File Versioning](tdd/storage/File-Versioning.md)

### UI
- [Command Surface](tdd/ui/Command-Surface.md)
- [Shortcut Matrix](tdd/ui/Shortcut-Matrix.md)
- [Wireflow Overview](tdd/ui/wireflows/Wireflow-Overview.md)
- [Dialog Flows](tdd/ui/dialogs/Dialog-Flows.md)

### Features
- [Equation Spec](tdd/math/Equation-Spec.md)
- [Charts Spec](tdd/charts/Charts-Spec.md)
- [Mail Merge](tdd/mailmerge/Mail-Merge.md)
- [Content Controls UX](tdd/forms/Content-Controls-UX.md)
- [Templates Spec](tdd/templates/Templates-Spec.md)
- [Plugin Architecture](tdd/extensions/Plugin-Architecture.md)
- [Version History UI](tdd/versioning-ui/Version-History-UI.md)

### Internationalization
- [RTL and BiDi](tdd/i18n/RTL-and-Bidi.md)
- [Locale Formats](tdd/i18n/Locale-Formats.md)
- [IME RTL Edge Cases](tdd/i18n/edge-cases/IME-RTL-Edge-Cases.md)

### Testing & Quality
- [Test Corpus Definition](tdd/testing/Test-Corpus-Definition.md)
- [Layout Test Plan](tdd/Layout-Test-Plan.md)
- [OOXML Conformance Tests](tdd/ooxml/testing/OOXML-Conformance-Tests.md)
- [Accessibility Test Plan](tdd/accessibility/testing/Accessibility-Test-Plan.md)

### Other
- [Performance Budget](tdd/perf/Performance-Budget.md)
- [Security Model](tdd/security/Security-Model.md)
- [Security Checklist](tdd/security/review/Security-Checklist.md)
- [Accessibility Spec](tdd/accessibility/Accessibility-Spec.md)
- [Error Handling](tdd/reliability/Error-Handling.md)
- [Telemetry Spec](tdd/telemetry/Telemetry-Spec.md)
- [IPC API Spec](tdd/api/IPC-API-Spec.md)
- [User Preferences](tdd/settings/User-Preferences.md)
- [Compatibility Matrix](tdd/compat/Compatibility-Matrix.md)

---

## Contributing

1. Read the relevant TDD document before implementing a feature
2. Follow the implementation plan for your current phase
3. Ensure all tests pass: `cargo test`
4. Update documentation if adding new features
