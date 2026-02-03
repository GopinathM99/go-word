# Phase Three Implementation Plan

> **STATUS: ✅ COMPLETE**
> **Last Updated:** 2026-01-28
> **Verified:** 2026-01-28
>
> ### Implementation Summary
> | Component | Status | Notes |
> |-----------|--------|-------|
> | A: CRDT Infrastructure | ✅ Complete | 14,000+ lines in `/crates/collab/` |
> | B1: WebSocket Client | ✅ Complete | `CollaborationClient.ts` |
> | B1: WebSocket Server | ✅ Complete | `/crates/collab/src/server/` (mod.rs, connection.rs, message.rs) |
> | B2: Sync Engine | ✅ Complete | `sync.rs` (26KB) |
> | B3: Offline Support | ✅ Complete | `offline.rs` (34KB) |
> | C: Presence System | ✅ Complete | `presence.rs`, `RemoteCursors.tsx` |
> | D: Version History | ✅ Complete | `version.rs` (45KB), `VersionHistory.tsx` |
> | E: Permissions & Sharing | ✅ Complete | `permissions.rs` (45KB), `ShareDialog.tsx` |
> | F1: Test Framework | ✅ Complete | 77 tests covering convergence, conflicts, sync, stress |
> | F2: Load Testing | ✅ Complete | Stress tests with 10+ concurrent clients implemented |
>
> ### Server Infrastructure (Added 2026-01-28)
> | Module | File | Description |
> |--------|------|-------------|
> | WebSocket Server | `server/mod.rs` | tokio-tungstenite server with configurable port, auth, graceful shutdown |
> | Connection Manager | `server/connection.rs` | Client tracking, auth provider trait, connection states |
> | Message Protocol | `server/message.rs` | Wire format compatible with TypeScript client |
> | Document Sessions | `server/session.rs` | Per-document client tracking, broadcast, CRDT integration |
> | Operation Router | `server/router.rs` | Multi-session management, permission checking |
> | Client State | `server/client.rs` | Per-client state, permissions, presence |
> | Storage Abstraction | `server/storage.rs` | `OperationStore` trait for persistence |
> | Memory Store | `server/memory_store.rs` | In-memory implementation for dev/testing |
> | File Store | `server/file_store.rs` | File-based persistence with JSON lines |

---

## Overview

Phase 3 transforms the single-user editor into a **real-time collaborative platform**. Multiple users can simultaneously edit the same document with automatic conflict resolution, presence awareness, and version history. This is the most architecturally challenging phase, requiring careful coordination between client and server.

**Prerequisites:** Phase 0-2 must be complete, with particular emphasis on:
- CRDT/OT-compatible command system (Phase 0)
- Inverse operations for all commands (Phase 0)
- Track changes infrastructure (Phase 2)
- Comments with threading (Phase 2)
- All document features stable and tested

**Critical Foundation:** The success of Phase 3 depends entirely on Phase 0 decisions. If the command system was not designed for CRDT/OT from the start, significant refactoring will be required.

---

## Phase 3 Goals

1. **Real-Time Co-Editing:** Multiple users edit simultaneously with instant sync
2. **Presence Awareness:** See other users' cursors, selections, and activity
3. **Conflict Resolution:** Automatic, deterministic resolution of concurrent edits
4. **Version History:** Browse, compare, and restore previous versions
5. **Access Control:** Permissions for view, comment, and edit access
6. **Offline Support:** Continue editing offline with seamless sync on reconnect

---

## Architecture Overview

### High-Level Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Client A      │     │   Client B      │     │   Client C      │
│  ┌───────────┐  │     │  ┌───────────┐  │     │  ┌───────────┐  │
│  │ Document  │  │     │  │ Document  │  │     │  │ Document  │  │
│  │  (CRDT)   │  │     │  │  (CRDT)   │  │     │  │  (CRDT)   │  │
│  └─────┬─────┘  │     │  └─────┬─────┘  │     │  └─────┬─────┘  │
│        │        │     │        │        │     │        │        │
│  ┌─────▼─────┐  │     │  ┌─────▼─────┐  │     │  ┌─────▼─────┐  │
│  │   Sync    │  │     │  │   Sync    │  │     │  │   Sync    │  │
│  │  Engine   │  │     │  │  Engine   │  │     │  │  Engine   │  │
│  └─────┬─────┘  │     │  └─────┬─────┘  │     │  └─────┬─────┘  │
└────────┼────────┘     └────────┼────────┘     └────────┼────────┘
         │                       │                       │
         │    WebSocket          │    WebSocket          │
         └───────────┬───────────┴───────────┬───────────┘
                     │                       │
              ┌──────▼───────────────────────▼──────┐
              │         Collaboration Server        │
              │  ┌─────────────────────────────┐   │
              │  │     Document Authority      │   │
              │  │  (Version History, Auth)    │   │
              │  └─────────────────────────────┘   │
              │  ┌─────────────────────────────┐   │
              │  │      Presence Service       │   │
              │  │  (Cursors, User Status)     │   │
              │  └─────────────────────────────┘   │
              └────────────────────────────────────┘
```

### CRDT vs. OT Decision

| Approach | Pros | Cons |
|----------|------|------|
| **CRDT** | No central authority needed, offline-first, mathematically proven convergence | Larger data structures, tombstone accumulation |
| **OT** | Smaller wire format, mature algorithms | Requires transformation functions for all operation pairs, complex server |

**Recommendation:** Use CRDT (specifically RGA for text sequences) because:
- Better offline support (critical for local-first architecture)
- Simpler mental model
- Phase 0 command system already designed for CRDT compatibility

---

## Task Groups and Dependencies

### Dependency Legend
- **Independent**: Can start immediately after Phase 2
- **Depends on [X]**: Requires task X to be complete first
- **Parallel with [X]**: Can be developed alongside task X

---

## Group A: CRDT Infrastructure (Foundation)

### A1. CRDT Data Structures
**Estimate:** L (2-4 weeks)
**Dependencies:** Independent (builds on Phase 0 document model)

**Implementation Steps:**

1. **Implement RGA (Replicated Growable Array) for text:**
   ```rust
   struct RgaNode {
       id: OpId,              // (client_id, sequence_num)
       value: Option<char>,   // None = tombstone (deleted)
       parent_id: OpId,       // Insert position reference
   }

   struct OpId {
       client_id: ClientId,
       seq: u64,
   }
   ```

2. **Implement CRDT tree for block structure:**
   ```rust
   struct CrdtTree {
       nodes: HashMap<NodeId, CrdtTreeNode>,
       root: NodeId,
   }

   struct CrdtTreeNode {
       id: NodeId,
       parent_id: Option<NodeId>,
       children: RgaList<NodeId>,  // Ordered children
       data: BlockData,
       tombstone: bool,
   }
   ```

3. **Implement LWW (Last-Writer-Wins) Register for formatting:**
   ```rust
   struct LwwRegister<T> {
       value: T,
       timestamp: HybridLogicalClock,
       client_id: ClientId,
   }
   ```

4. **Implement logical clocks:**
   - Lamport timestamps for basic ordering
   - Hybrid Logical Clocks (HLC) for wall-clock proximity
   - Vector clocks for causality tracking (if needed)

5. **Implement operation serialization:**
   - Compact binary format for wire transfer
   - JSON format for debugging
   - Version field for future compatibility

6. **Implement operation application:**
   - Apply remote operation to local CRDT
   - Handle out-of-order delivery
   - Idempotent application (same op twice = no change)

**Deliverables:**
- RGA implementation for text sequences
- CRDT tree for document structure
- LWW registers for formatting
- Logical clock implementations
- Operation serialization

**Architecture Notes:**
- Operations must be commutative (order-independent)
- Operations must be idempotent (apply twice = same result)
- Design for eventual consistency

---

### A2. CRDT-Document Bridge
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on A1 (CRDT Data Structures)

**Implementation Steps:**

1. **Map existing document operations to CRDT ops:**
   ```rust
   // Phase 0 command → CRDT operation
   fn command_to_crdt_ops(cmd: &Command, clock: &mut Clock) -> Vec<CrdtOp> {
       match cmd {
           Command::InsertText { pos, text } => {
               text.chars().map(|c| CrdtOp::RgaInsert {
                   id: clock.next(),
                   parent: pos.to_rga_id(),
                   value: c,
               }).collect()
           }
           // ... other mappings
       }
   }
   ```

2. **Implement document state from CRDT:**
   - Materialize CRDT into renderable document
   - Efficient incremental updates
   - Cache materialized view

3. **Handle position mapping:**
   - Document position (node + offset) ↔ CRDT position (OpId)
   - Update mappings on remote changes
   - Handle tombstones in position calculation

4. **Implement undo/redo with CRDT:**
   - Undo = apply inverse operation as new CRDT op
   - Track user-specific undo stack
   - Handle concurrent undo correctly

**Deliverables:**
- Command → CRDT operation mapping
- CRDT → Document materialization
- Position mapping utilities
- Collaborative undo/redo

---

### A3. Conflict Resolution Engine
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on A1, A2

**Implementation Steps:**

1. **Implement text conflict resolution:**
   - Concurrent inserts at same position: order by OpId
   - Concurrent delete + edit: delete wins (text gone)
   - Character-level granularity

2. **Implement formatting conflict resolution:**
   - Per-attribute LWW (last writer wins)
   - Concurrent bold + italic: both apply (different attributes)
   - Concurrent bold on + bold off: latest timestamp wins

3. **Implement structural conflict resolution:**
   - Concurrent block insert: order by OpId
   - Concurrent block delete + edit: delete wins
   - Parent-child relationship changes: careful ordering

4. **Implement special cases:**
   - Table cell conflicts
   - List item reordering
   - Comment anchor changes

5. **Build conflict visualization (optional):**
   - Highlight recently conflicted areas
   - Show conflict history

**Deliverables:**
- Text conflict resolution
- Formatting conflict resolution
- Structural conflict resolution
- Conflict logging and optional visualization

---

## Group B: Networking Layer

### B1. WebSocket Communication
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on A1 (needs operation format)

**Implementation Steps:**

1. **Implement WebSocket client (TypeScript):**
   ```typescript
   class CollaborationClient {
       private ws: WebSocket;
       private pendingOps: CrdtOp[] = [];
       private acknowledged: Set<OpId> = new Set();

       connect(docId: string, authToken: string): Promise<void>;
       send(op: CrdtOp): void;
       onReceive(callback: (op: CrdtOp) => void): void;
       disconnect(): void;
   }
   ```

2. **Implement message protocol:**
   ```typescript
   type Message =
       | { type: 'auth', token: string }
       | { type: 'join', docId: string }
       | { type: 'ops', ops: CrdtOp[] }
       | { type: 'ack', opIds: OpId[] }
       | { type: 'presence', state: PresenceState }
       | { type: 'sync_request', since: VectorClock }
       | { type: 'sync_response', ops: CrdtOp[] }
       | { type: 'error', code: string, message: string };
   ```

3. **Implement connection management:**
   - Automatic reconnection with exponential backoff
   - Connection state tracking (connecting, connected, disconnected)
   - Heartbeat/ping-pong for connection health

4. **Implement operation batching:**
   - Batch rapid operations (typing burst)
   - Configurable batch window (e.g., 50ms)
   - Flush on batch timeout or explicit request

5. **Implement acknowledgment handling:**
   - Track pending (unacknowledged) operations
   - Retry unacknowledged after timeout
   - Handle acknowledgment gaps

**Deliverables:**
- WebSocket client implementation
- Message protocol
- Connection management
- Operation batching and acknowledgment

---

### B2. Sync Engine
**Estimate:** L (2-4 weeks)
**Dependencies:** Depends on A1, A2, B1

**Implementation Steps:**

1. **Implement local operation queue:**
   ```rust
   struct SyncEngine {
       local_ops: VecDeque<CrdtOp>,      // Pending local ops
       sent_ops: HashMap<OpId, CrdtOp>,  // Sent, awaiting ack
       applied_ops: OpLog,               // All applied ops
       vector_clock: VectorClock,        // Current state version
   }
   ```

2. **Implement outbound sync:**
   - Queue local operations
   - Send to server via WebSocket
   - Track acknowledgments
   - Retry on failure

3. **Implement inbound sync:**
   - Receive remote operations
   - Apply to local CRDT
   - Update materialized document
   - Trigger UI refresh

4. **Implement initial sync:**
   - Request full document state on join
   - Or request ops since last known version
   - Merge with any local pending ops

5. **Implement catch-up sync:**
   - On reconnect, sync missed operations
   - Use vector clock to identify gaps
   - Request missing ops from server

6. **Implement operation log:**
   - Persist all operations locally
   - Compact old operations periodically
   - Support offline operation accumulation

**Deliverables:**
- Local operation queue
- Outbound/inbound sync
- Initial and catch-up sync
- Operation log with persistence

---

### B3. Offline Support
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on B2 (Sync Engine)

**Implementation Steps:**

1. **Detect offline state:**
   - Monitor network connectivity
   - Handle WebSocket disconnect
   - Track time since last sync

2. **Queue offline operations:**
   - Continue editing normally
   - Store operations in local queue
   - Persist queue to survive app restart

3. **Implement reconnection sync:**
   - On reconnect, send queued operations
   - Receive and merge missed remote operations
   - Resolve any conflicts

4. **Handle conflict notification:**
   - Detect significant conflicts after merge
   - Optionally notify user
   - Provide conflict resolution UI (if needed)

5. **Implement offline indicators:**
   - Show offline status in UI
   - Show pending sync count
   - Show last sync time

**Deliverables:**
- Offline detection
- Local operation persistence
- Reconnection merge
- Offline status UI

---

## Group C: Presence System

### C1. Presence Protocol
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on B1 (WebSocket)

**Implementation Steps:**

1. **Define presence state:**
   ```typescript
   interface PresenceState {
       userId: string;
       displayName: string;
       color: string;           // User's cursor color
       cursor: Position | null; // Current cursor position
       selection: Range | null; // Current selection
       isTyping: boolean;
       lastActive: number;      // Timestamp
   }
   ```

2. **Implement presence broadcasting:**
   - Send cursor position on change
   - Send selection on change
   - Throttle updates (30-60 Hz max)
   - Coalesce rapid changes

3. **Implement presence receiving:**
   - Receive other users' presence
   - Update presence state store
   - Handle user join/leave

4. **Implement presence rendering:**
   - Render remote cursors with user color
   - Render remote selections with transparency
   - Show user name label near cursor

5. **Implement user list:**
   - Show active users in document
   - Show user colors
   - Click to jump to user's position

**Deliverables:**
- Presence state model
- Presence broadcasting
- Remote cursor/selection rendering
- User list panel

---

### C2. Cursor and Selection Rendering
**Estimate:** S (days)
**Dependencies:** Depends on C1 (Presence Protocol)

**Implementation Steps:**

1. **Implement remote cursor rendering:**
   ```typescript
   interface RemoteCursor {
       position: LayoutPosition;
       color: string;
       userName: string;
       isVisible: boolean;
   }
   ```

2. **Calculate cursor positions:**
   - Map CRDT position to layout position
   - Update on layout changes
   - Handle cursor in collapsed/hidden content

3. **Render cursor UI:**
   - Vertical line in user's color
   - User name badge (show on hover or always)
   - Smooth animation on position change

4. **Render remote selections:**
   - Highlight rectangles in user's color (transparent)
   - Handle multi-line selections
   - Handle selection in different views

5. **Handle edge cases:**
   - Cursor in deleted content (hide or show placeholder)
   - Multiple users at same position
   - Cursor off-screen (show indicator at edge)

**Deliverables:**
- Remote cursor rendering
- Remote selection highlighting
- User identification labels
- Edge case handling

---

## Group D: Version History

### D1. Version History Backend
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on B2 (Sync Engine provides operation log)

**Implementation Steps:**

1. **Define version model:**
   ```rust
   struct Version {
       id: VersionId,
       timestamp: DateTime<Utc>,
       author: UserId,
       summary: String,           // Auto-generated or user-provided
       parent_version: VersionId,
       ops_since_parent: Vec<CrdtOp>,
   }
   ```

2. **Implement automatic versioning:**
   - Create checkpoint every N operations
   - Create checkpoint every N minutes
   - Create checkpoint on significant events (save, close)

3. **Implement named versions:**
   - User can name a version ("Draft 1", "Final Review")
   - Named versions are never auto-deleted

4. **Implement version retrieval:**
   - Get document state at any version
   - Efficient reconstruction from checkpoints + ops

5. **Implement version comparison:**
   - Diff two versions
   - Show added/removed/changed content
   - Use track-changes style rendering

**Deliverables:**
- Version model
- Automatic checkpointing
- Named versions
- Version retrieval and comparison

---

### D2. Version History UI
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on D1 (Version History Backend)

**Implementation Steps:**

1. **Build version history panel:**
   - Timeline or list of versions
   - Show timestamp, author, summary
   - Visual indicators for named versions

2. **Implement version preview:**
   - Click version to preview (read-only)
   - Side-by-side comparison mode
   - Highlight differences

3. **Implement version restore:**
   - Restore document to previous version
   - Creates new version (doesn't delete history)
   - Confirmation dialog

4. **Implement version diff view:**
   - Compare any two versions
   - Track-changes style diff rendering
   - Navigate between changes

5. **Implement activity feed (optional):**
   - Show recent edits by all users
   - Who changed what, when
   - Click to jump to change

**Deliverables:**
- Version history panel
- Version preview
- Version restore
- Diff view between versions

---

## Group E: Permissions and Sharing

### E1. Permission Model
**Estimate:** M (1-2 weeks)
**Dependencies:** Independent (design can start early)

**Implementation Steps:**

1. **Define permission levels:**
   ```typescript
   enum PermissionLevel {
       Owner,      // Full control, can delete document
       Editor,     // Can edit content
       Commenter,  // Can add comments only
       Viewer,     // Read-only access
   }

   interface DocumentPermission {
       docId: string;
       userId: string | 'anyone';  // 'anyone' for link sharing
       level: PermissionLevel;
       grantedBy: string;
       grantedAt: DateTime;
       expiresAt?: DateTime;
   }
   ```

2. **Implement permission checking:**
   - Check permission before allowing operation
   - Server-side enforcement (never trust client)
   - Client-side UI adaptation

3. **Implement permission inheritance:**
   - Workspace/folder permissions
   - Document overrides workspace permissions

4. **Implement permission caching:**
   - Cache permissions on client
   - Invalidate on permission change
   - Graceful degradation if cache stale

**Deliverables:**
- Permission model
- Permission checking logic
- Server-side enforcement
- Client-side permission cache

---

### E2. Sharing Flow
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on E1 (Permission Model)

**Implementation Steps:**

1. **Build share dialog:**
   - Enter email or search users
   - Select permission level
   - Send invitation

2. **Implement link sharing:**
   - Generate shareable link
   - Set link permission level
   - Optional password protection
   - Optional expiration

3. **Implement invitation system:**
   - Send email invitation
   - In-app notification
   - Accept/decline flow

4. **Build collaborator list:**
   - Show all users with access
   - Show permission levels
   - Allow owner to modify/revoke

5. **Implement transfer ownership:**
   - Owner can transfer to another user
   - Confirmation required
   - Audit log entry

**Deliverables:**
- Share dialog
- Link sharing
- Invitation system
- Collaborator management

---

## Group F: Testing and Quality

### F1. Collaboration Testing
**Estimate:** L (2-4 weeks)
**Dependencies:** Depends on all other tasks

**Implementation Steps:**

1. **Build collaboration test harness:**
   - Simulate multiple clients
   - Inject operations with timing control
   - Verify convergence

2. **Implement convergence tests:**
   - Generate random operations
   - Apply in different orders to different clients
   - Verify final state identical

3. **Implement stress tests:**
   - Many concurrent editors (10, 50, 100)
   - Rapid operation generation
   - Measure latency and throughput

4. **Implement network condition tests:**
   - Simulate latency (100ms, 500ms, 2s)
   - Simulate packet loss
   - Simulate disconnection/reconnection

5. **Implement edge case tests:**
   - Concurrent edits at same position
   - Rapid undo/redo across clients
   - Large paste operations
   - Image/table concurrent edits

**Deliverables:**
- Collaboration test framework
- Convergence test suite
- Stress tests
- Network simulation tests

---

### F2. Load Simulation
**Estimate:** M (1-2 weeks)
**Dependencies:** Depends on F1 (Testing Framework)

**Implementation Steps:**

1. **Build load generator:**
   - Simulate realistic user behavior
   - Typing, formatting, navigation
   - Configurable user count and patterns

2. **Implement server load testing:**
   - Measure server capacity
   - Operations per second
   - Concurrent connection limit

3. **Implement client performance testing:**
   - Measure client latency under load
   - Memory usage with many collaborators
   - Rendering performance with many cursors

4. **Build monitoring dashboard:**
   - Real-time metrics
   - Latency percentiles
   - Error rates

5. **Document capacity limits:**
   - Maximum recommended collaborators
   - Document size limits
   - Performance degradation thresholds

**Deliverables:**
- Load generator
- Server capacity benchmarks
- Client performance benchmarks
- Capacity documentation

---

## Implementation Schedule

### Sprint 1-2: CRDT Foundation
| Task | Estimate | Dependencies |
|------|----------|--------------|
| A1. CRDT Data Structures | L | Start |
| E1. Permission Model | M | Parallel (design) |

### Sprint 3-4: CRDT Integration
| Task | Estimate | Dependencies |
|------|----------|--------------|
| A2. CRDT-Document Bridge | M | After A1 |
| A3. Conflict Resolution | M | After A1, A2 |
| B1. WebSocket Communication | M | After A1 |

### Sprint 5-6: Sync Engine
| Task | Estimate | Dependencies |
|------|----------|--------------|
| B2. Sync Engine | L | After A2, B1 |
| C1. Presence Protocol | M | After B1 |

### Sprint 7-8: Presence and Offline
| Task | Estimate | Dependencies |
|------|----------|--------------|
| C2. Cursor Rendering | S | After C1 |
| B3. Offline Support | M | After B2 |
| E2. Sharing Flow | M | After E1 |

### Sprint 9-10: Version History
| Task | Estimate | Dependencies |
|------|----------|--------------|
| D1. Version History Backend | M | After B2 |
| D2. Version History UI | M | After D1 |

### Sprint 11-14: Testing and Hardening
| Task | Estimate | Dependencies |
|------|----------|--------------|
| F1. Collaboration Testing | L | After all features |
| F2. Load Simulation | M | After F1 |
| Bug fixes and optimization | L | Ongoing |

---

## Dependency Graph

```
Phase 2 (Complete)
    │
    ▼
A1 (CRDT Data Structures) ─────────┬─────────────────────────────┐
    │                               │                             │
    ▼                               ▼                             │
A2 (CRDT-Document Bridge) ◄────► B1 (WebSocket)                  │
    │                               │                             │
    ▼                               │                             │
A3 (Conflict Resolution)           │                             │
    │                               │                             │
    └───────────┬───────────────────┘                             │
                │                                                  │
                ▼                                                  │
           B2 (Sync Engine) ──────────────────────────────────────┤
                │                                                  │
                ├──► B3 (Offline Support)                         │
                │                                                  │
                ├──► D1 (Version History Backend)                 │
                │         │                                        │
                │         └──► D2 (Version History UI)            │
                │                                                  │
                └──► C1 (Presence Protocol)                       │
                          │                                        │
                          └──► C2 (Cursor Rendering)              │
                                                                   │
E1 (Permission Model) ──► E2 (Sharing Flow)                       │
                                                                   │
All Features ──────────────────────────────────────────────────────┘
       │
       ▼
F1 (Collaboration Testing) ──► F2 (Load Simulation)
```

---

## Server Architecture

### Collaboration Server Components

```
┌─────────────────────────────────────────────────────────────────┐
│                    Collaboration Server                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  WebSocket   │  │   Auth       │  │   Document   │          │
│  │  Gateway     │  │   Service    │  │   Service    │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                  │                  │                  │
│         └──────────────────┼──────────────────┘                  │
│                            │                                     │
│  ┌─────────────────────────▼─────────────────────────┐          │
│  │              Operation Router                      │          │
│  │   (Routes ops to correct document session)        │          │
│  └─────────────────────────┬─────────────────────────┘          │
│                            │                                     │
│  ┌─────────────────────────▼─────────────────────────┐          │
│  │           Document Session Manager                 │          │
│  │   (One session per active document)               │          │
│  │   - Maintains document CRDT state                 │          │
│  │   - Broadcasts ops to connected clients           │          │
│  │   - Persists to storage                           │          │
│  └─────────────────────────┬─────────────────────────┘          │
│                            │                                     │
│  ┌─────────────────────────▼─────────────────────────┐          │
│  │              Storage Layer                         │          │
│  │   - Operation log (append-only)                   │          │
│  │   - Document snapshots                            │          │
│  │   - Version history                               │          │
│  └───────────────────────────────────────────────────┘          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Server Technology Recommendations

| Component | Recommendation | Rationale |
|-----------|----------------|-----------|
| WebSocket Server | Rust (tokio + tungstenite) or Node.js | High concurrency, low latency |
| Auth Service | Standard JWT/OAuth | Leverage existing infrastructure |
| Storage | PostgreSQL + S3 | Reliable, scalable |
| Cache | Redis | Presence state, active sessions |
| Message Queue | Redis Pub/Sub or Kafka | Operation distribution |

---

## Technical Specifications

### CRDT Operation Format

```typescript
interface CrdtOp {
    id: OpId;
    type: OpType;
    timestamp: HLC;
    payload: OpPayload;
}

interface OpId {
    clientId: string;
    seq: number;
}

type OpType =
    | 'text_insert'
    | 'text_delete'
    | 'format_set'
    | 'block_insert'
    | 'block_delete'
    | 'block_move';

// Example: text insert
interface TextInsertPayload {
    nodeId: NodeId;
    parentOpId: OpId;  // Insert after this character
    char: string;
}
```

### Wire Protocol

```typescript
// Client → Server
interface ClientMessage {
    type: 'ops' | 'presence' | 'sync_request' | 'ack';
    docId: string;
    payload: any;
}

// Server → Client
interface ServerMessage {
    type: 'ops' | 'presence' | 'sync_response' | 'ack' | 'error';
    payload: any;
}

// Sync request/response
interface SyncRequest {
    vectorClock: VectorClock;
}

interface SyncResponse {
    ops: CrdtOp[];
    serverClock: VectorClock;
}
```

### Presence Protocol

```typescript
// Presence update (throttled)
interface PresenceUpdate {
    cursor?: Position;
    selection?: Range;
    isTyping?: boolean;
}

// Server broadcasts to all clients
interface PresenceBroadcast {
    userId: string;
    displayName: string;
    color: string;
    ...PresenceUpdate;
}
```

---

## Risk Mitigation

### 1. CRDT Complexity
- **Risk:** CRDT implementation is complex and error-prone
- **Mitigation:** Use proven CRDT library if available (Yjs, Automerge)
- **Mitigation:** Extensive property-based testing
- **Mitigation:** Start with text-only CRDT, add structure incrementally

### 2. Performance at Scale
- **Risk:** Latency increases with many collaborators
- **Mitigation:** Operation batching and compression
- **Mitigation:** Efficient CRDT data structures
- **Mitigation:** Document recommended collaborator limits

### 3. Offline Merge Conflicts
- **Risk:** Long offline periods create complex merges
- **Mitigation:** Warn user before merging large offline changes
- **Mitigation:** Provide conflict resolution UI if needed
- **Mitigation:** Consider manual merge option for extreme cases

### 4. Server Reliability
- **Risk:** Server downtime disrupts collaboration
- **Mitigation:** Local-first architecture (continue editing offline)
- **Mitigation:** Server redundancy and failover
- **Mitigation:** Graceful degradation to single-user mode

### 5. Security
- **Risk:** Unauthorized access to documents
- **Mitigation:** Server-side permission enforcement
- **Mitigation:** End-to-end encryption for sensitive documents
- **Mitigation:** Audit logging for compliance

---

## Exit Criteria for Phase 3

Phase 3 is complete when:

1. **Real-Time Editing:**
   - Multiple users can edit simultaneously
   - Changes appear within 500ms on other clients
   - No data loss under normal conditions

2. **Conflict Resolution:**
   - Concurrent edits converge to identical state
   - Formatting conflicts resolve predictably
   - Undo/redo works correctly in collaborative context

3. **Presence:**
   - Remote cursors visible and accurate
   - Remote selections visible
   - User list shows all active collaborators

4. **Offline:**
   - Editing continues without network
   - Changes sync on reconnect
   - Conflicts handled gracefully

5. **Version History:**
   - Can browse document history
   - Can restore previous versions
   - Can compare two versions

6. **Permissions:**
   - Permission levels enforced
   - Sharing via email and link works
   - Collaborator management works

7. **Performance:**
   - Supports 10+ simultaneous editors
   - Latency <500ms for 95th percentile
   - No memory leaks over extended sessions

8. **Testing:**
   - Convergence tests pass
   - Load tests meet capacity targets
   - Network disruption handled gracefully

---

## Estimated Timeline

- **Total Duration:** 16-20 weeks (4-5 months)
- **Team Assumption:** 3-4 engineers (including backend)
- **Critical Path:** CRDT Implementation → Sync Engine → Testing

### Team Allocation Suggestion

| Role | Focus Areas |
|------|-------------|
| Engineer 1 (Rust/Core) | A1, A2, A3 - CRDT implementation |
| Engineer 2 (Full-stack) | B1, B2, B3 - Networking and sync |
| Engineer 3 (Frontend) | C1, C2, D2, E2 - Presence and UI |
| Engineer 4 (Backend) | Server infrastructure, D1, E1 |

---

## Relationship to Phase 4

Phase 3 enables Phase 4 features:

1. **Real-time collaboration** enables:
   - Shared templates and content controls
   - Collaborative mail merge review
   - Multi-user form filling

2. **Version history** provides foundation for:
   - Document comparison features
   - Audit trails for compliance

3. **Permission model** extends to:
   - Plugin permissions
   - External integration access
