# Test Group 11: Collaboration

## Overview
Tests for real-time collaboration, presence, synchronization, and sharing features.

**Prerequisites:**
- Collaboration server running (`cargo run --package collab --features server`)
- Two or more clients connected

---

## 11.1 Server Setup

### TC-11.1.1: Start Collaboration Server
**Steps:**
1. Build server: `cargo build --package collab --features server`
2. Run server on port 8080
3. Verify server starts and listens

**Expected:** Server running
```
Collaboration server running on ws://0.0.0.0:8080
```
- [ ] Pass

### TC-11.1.2: Server Accepts Connections
**Steps:**
1. Start server
2. Connect client application
3. Verify connection established

**Expected:** Client connects successfully
- [ ] Pass

### TC-11.1.3: Server Graceful Shutdown
**Steps:**
1. With clients connected
2. Send shutdown signal (Ctrl+C)
3. Verify clients notified
4. Verify clean shutdown

**Expected:** Graceful disconnection
- [ ] Pass

---

## 11.2 Client Connection

### TC-11.2.1: Connect to Server
**Steps:**
1. Open application
2. Connect to collaboration server
3. Verify connection indicator shows connected

**Expected:** Client connects
- [ ] Pass

### TC-11.2.2: Authentication
**Steps:**
1. Connect to server
2. Provide authentication token
3. Verify authentication succeeds

**Expected:** Auth works
- [ ] Pass

### TC-11.2.3: Join Document Session
**Steps:**
1. Connect to server
2. Join document "test-doc"
3. Verify document loads

**Expected:** Join session works
- [ ] Pass

### TC-11.2.4: Reconnection on Disconnect
**Steps:**
1. Connect to server
2. Interrupt network briefly
3. Verify automatic reconnection

**Expected:** Auto-reconnect
- [ ] Pass

### TC-11.2.5: Offline Queue
**Steps:**
1. Make edits while connected
2. Disconnect network
3. Make more edits
4. Reconnect
5. Verify offline edits sync

**Expected:** Offline edits preserved and synced
- [ ] Pass

---

## 11.3 Real-Time Editing

### TC-11.3.1: See Other User's Edits
**Steps:**
1. User A and User B open same document
2. User A types "Hello"
3. Verify User B sees "Hello" appear

**Expected:** Edits visible in real-time
- [ ] Pass

### TC-11.3.2: Concurrent Typing
**Steps:**
1. User A types at beginning of doc
2. User B types at end of doc simultaneously
3. Verify both edits preserved

**Expected:** Concurrent edits don't conflict
- [ ] Pass

### TC-11.3.3: Concurrent Same-Location Edit
**Steps:**
1. Both users type at same position
2. Verify consistent result (both texts present)
3. Verify both clients converge to same state

**Expected:** CRDT convergence
- [ ] Pass

### TC-11.3.4: Delete While Other Types
**Steps:**
1. User A selects paragraph
2. User B types in same paragraph
3. User A deletes selection
4. Verify consistent resolution

**Expected:** Delete/edit conflict resolved
- [ ] Pass

### TC-11.3.5: Formatting Changes Sync
**Steps:**
1. User A applies bold to text
2. Verify User B sees bold formatting

**Expected:** Formatting syncs
- [ ] Pass

### TC-11.3.6: Table Edits Sync
**Steps:**
1. User A edits table cell
2. Verify User B sees change

**Expected:** Table edits sync
- [ ] Pass

---

## 11.4 Presence Awareness

### TC-11.4.1: See Other User's Cursor
**Steps:**
1. Two users in same document
2. User A moves cursor
3. Verify User B sees User A's cursor position

**Expected:** Remote cursor visible
- [ ] Pass

### TC-11.4.2: Cursor Color/Label
**Steps:**
1. View remote cursor
2. Verify cursor has distinct color
3. Verify user name label visible

**Expected:** Cursor identified by color/name
- [ ] Pass

### TC-11.4.3: See Other User's Selection
**Steps:**
1. User A selects text
2. Verify User B sees selection highlight

**Expected:** Remote selection visible
- [ ] Pass

### TC-11.4.4: User List
**Steps:**
1. Multiple users join document
2. View collaborator list
3. Verify all users shown

**Expected:** Active user list
- [ ] Pass

### TC-11.4.5: User Joins Notification
**Steps:**
1. User A in document
2. User B joins
3. Verify User A sees notification

**Expected:** Join notification
- [ ] Pass

### TC-11.4.6: User Leaves Notification
**Steps:**
1. Users A and B in document
2. User B disconnects
3. Verify User A sees notification
4. Verify User B's cursor disappears

**Expected:** Leave notification
- [ ] Pass

### TC-11.4.7: Typing Indicator
**Steps:**
1. User A starts typing
2. Verify User B sees typing indicator

**Expected:** Typing status shown
- [ ] Pass

---

## 11.5 Conflict Resolution

### TC-11.5.1: Insert at Same Position
**Steps:**
1. Disconnect both clients from network
2. User A types "AAA" at position 0
3. User B types "BBB" at position 0
4. Reconnect both
5. Verify deterministic merge (e.g., "AAABBB" or "BBBAAA")
6. Verify both clients show same result

**Expected:** Consistent conflict resolution
- [ ] Pass

### TC-11.5.2: Overlapping Deletions
**Steps:**
1. Both users delete overlapping ranges
2. Verify text deleted once (not double-deleted)

**Expected:** Delete idempotency
- [ ] Pass

### TC-11.5.3: Format Conflict
**Steps:**
1. User A makes text bold
2. User B makes same text italic (simultaneously)
3. Verify both formatting applied (bold italic)

**Expected:** Non-conflicting formats merge
- [ ] Pass

### TC-11.5.4: Conflicting Format (Same Attribute)
**Steps:**
1. User A sets text to red
2. User B sets same text to blue (simultaneously)
3. Verify last-writer-wins or consistent resolution

**Expected:** Deterministic format conflict resolution
- [ ] Pass

---

## 11.6 Version History

### TC-11.6.1: View Version History
**Steps:**
1. Make several edits over time
2. Open version history panel
3. Verify versions listed with timestamps

**Expected:** Version list available
- [ ] Pass

### TC-11.6.2: Preview Version
**Steps:**
1. Select a previous version
2. Preview it
3. Verify shows document at that point

**Expected:** Version preview
- [ ] Pass

### TC-11.6.3: Restore Version
**Steps:**
1. Select previous version
2. Restore it
3. Verify document reverts
4. Verify all clients see restored version

**Expected:** Version restore syncs
- [ ] Pass

### TC-11.6.4: Name a Version
**Steps:**
1. Create named checkpoint "Before Review"
2. Verify named version in history
3. Verify named versions not auto-deleted

**Expected:** Named versions persist
- [ ] Pass

### TC-11.6.5: Compare Versions
**Steps:**
1. Select two versions
2. Compare
3. Verify diff shown

**Expected:** Version comparison
- [ ] Pass

---

## 11.7 Permissions

### TC-11.7.1: Owner Has Full Access
**Steps:**
1. Document owner opens document
2. Verify can edit, share, delete

**Expected:** Owner permissions
- [ ] Pass

### TC-11.7.2: Editor Can Edit
**Steps:**
1. Share document with user as Editor
2. Editor opens document
3. Verify can edit content

**Expected:** Editor permissions
- [ ] Pass

### TC-11.7.3: Commenter Can Only Comment
**Steps:**
1. Share as Commenter
2. Try to edit text
3. Verify editing blocked
4. Try to add comment
5. Verify comment allowed

**Expected:** Commenter restrictions
- [ ] Pass

### TC-11.7.4: Viewer is Read-Only
**Steps:**
1. Share as Viewer
2. Try to edit
3. Verify all edits blocked

**Expected:** Viewer restrictions
- [ ] Pass

### TC-11.7.5: Permission Denied Message
**Steps:**
1. Viewer tries to edit
2. Verify clear message about permissions

**Expected:** Permission feedback
- [ ] Pass

---

## 11.8 Sharing

### TC-11.8.1: Share via Email
**Steps:**
1. Open sharing dialog
2. Enter email address
3. Select permission level
4. Send invitation
5. Verify recipient can access

**Expected:** Email sharing
- [ ] Pass

### TC-11.8.2: Generate Share Link
**Steps:**
1. Create shareable link
2. Set permission (view/edit)
3. Share link
4. Open link in new browser
5. Verify access granted

**Expected:** Link sharing
- [ ] Pass

### TC-11.8.3: Revoke Access
**Steps:**
1. Share with user
2. Revoke their access
3. Verify they can no longer open document

**Expected:** Access revocation
- [ ] Pass

### TC-11.8.4: Transfer Ownership
**Steps:**
1. Transfer ownership to another user
2. Verify new owner has full control
3. Verify original owner demoted

**Expected:** Ownership transfer
- [ ] Pass

---

## 11.9 Stress Testing

### TC-11.9.1: 5 Concurrent Users
**Steps:**
1. Connect 5 clients to same document
2. All edit simultaneously
3. Verify convergence

**Expected:** 5 users stable
- [ ] Pass

### TC-11.9.2: 10 Concurrent Users
**Steps:**
1. Connect 10 clients
2. All edit simultaneously
3. Verify convergence and performance

**Expected:** 10 users stable
- [ ] Pass

### TC-11.9.3: Rapid Typing Sync
**Steps:**
1. User types very quickly (100+ WPM)
2. Verify remote user sees near-real-time updates

**Expected:** Low latency sync
- [ ] Pass

### TC-11.9.4: Large Document Collaboration
**Steps:**
1. Open document with 100+ pages
2. Multiple users edit
3. Verify performance acceptable

**Expected:** Large doc collaboration
- [ ] Pass

---

## Summary

| Section | Tests | Passed | Failed |
|---------|-------|--------|--------|
| 11.1 Server Setup | 3 | | |
| 11.2 Client Connection | 5 | | |
| 11.3 Real-Time Editing | 6 | | |
| 11.4 Presence Awareness | 7 | | |
| 11.5 Conflict Resolution | 4 | | |
| 11.6 Version History | 5 | | |
| 11.7 Permissions | 5 | | |
| 11.8 Sharing | 4 | | |
| 11.9 Stress Testing | 4 | | |
| **Total** | **43** | | |
