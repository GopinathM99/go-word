# Test Group 9: Track Changes and Comments

## Overview
Tests for revision tracking, comments, compare/merge, and review workflow features.

---

## 9.1 Track Changes - Basic

### TC-9.1.1: Enable Track Changes
**Steps:**
1. Review > Track Changes > Enable
2. Type new text
3. Verify text appears with revision marking (underlined, colored)

**Expected:** New text marked as insertion
- [ ] Pass

### TC-9.1.2: Track Deletions
**Steps:**
1. Enable Track Changes
2. Delete existing text
3. Verify deleted text shown with strikethrough

**Expected:** Deleted text marked
- [ ] Pass

### TC-9.1.3: Track Formatting Changes
**Steps:**
1. Enable Track Changes
2. Change text formatting (bold, color)
3. Verify formatting change tracked

**Expected:** Format changes recorded
- [ ] Pass

### TC-9.1.4: Disable Track Changes
**Steps:**
1. Enable Track Changes
2. Make edits (tracked)
3. Disable Track Changes
4. Make edits (not tracked)
5. Verify second edits have no markup

**Expected:** Can toggle tracking on/off
- [ ] Pass

### TC-9.1.5: Change User Name/Color
**Steps:**
1. Change review user name
2. Make tracked changes
3. Verify new name/color used

**Expected:** Different reviewers identified
- [ ] Pass

---

## 9.2 Track Changes - Display

### TC-9.2.1: Show All Markup
**Steps:**
1. Make tracked changes
2. Set Display to "All Markup"
3. Verify all changes visible with formatting

**Expected:** Full markup visible
- [ ] Pass

### TC-9.2.2: Show Simple Markup
**Steps:**
1. Set Display to "Simple Markup"
2. Verify changes indicated by margin line

**Expected:** Simplified view
- [ ] Pass

### TC-9.2.3: Show No Markup (Final)
**Steps:**
1. Set Display to "No Markup"
2. Verify document appears as if changes accepted

**Expected:** Clean view with changes applied
- [ ] Pass

### TC-9.2.4: Show Original
**Steps:**
1. Set Display to "Original"
2. Verify document appears as if changes rejected

**Expected:** View without changes
- [ ] Pass

### TC-9.2.5: Show Specific Reviewers
**Steps:**
1. Multiple reviewers make changes
2. Filter to show only Reviewer A
3. Verify only Reviewer A's changes shown

**Expected:** Filter by reviewer
- [ ] Pass

### TC-9.2.6: Balloons in Margin
**Steps:**
1. Enable "Show Revisions in Balloons"
2. Make changes
3. Verify balloons appear in margin

**Expected:** Margin balloons
- [ ] Pass

---

## 9.3 Accept/Reject Changes

### TC-9.3.1: Accept Single Change
**Steps:**
1. Make tracked insertion
2. Right-click > Accept Change
3. Verify text becomes normal (untracked)

**Expected:** Change accepted
- [ ] Pass

### TC-9.3.2: Reject Single Change
**Steps:**
1. Make tracked insertion
2. Right-click > Reject Change
3. Verify insertion removed

**Expected:** Change rejected
- [ ] Pass

### TC-9.3.3: Accept Deletion
**Steps:**
1. Track delete some text
2. Accept the deletion
3. Verify text permanently removed

**Expected:** Deletion finalized
- [ ] Pass

### TC-9.3.4: Reject Deletion
**Steps:**
1. Track delete some text
2. Reject the deletion
3. Verify text restored to normal

**Expected:** Deletion undone
- [ ] Pass

### TC-9.3.5: Accept All Changes
**Steps:**
1. Make multiple tracked changes
2. Accept All Changes
3. Verify all markup removed, changes applied

**Expected:** Bulk accept
- [ ] Pass

### TC-9.3.6: Reject All Changes
**Steps:**
1. Make multiple tracked changes
2. Reject All Changes
3. Verify document reverts to original

**Expected:** Bulk reject
- [ ] Pass

### TC-9.3.7: Navigate Between Changes
**Steps:**
1. Make several changes throughout document
2. Use Next/Previous Change buttons
3. Verify navigation through changes

**Expected:** Change navigation
- [ ] Pass

---

## 9.4 Comments - Basic

### TC-9.4.1: Insert Comment
**Steps:**
1. Select text
2. Review > New Comment
3. Type comment text
4. Verify comment appears in margin

**Expected:** Comment added
- [ ] Pass

### TC-9.4.2: View Comment
**Steps:**
1. Insert comment
2. Click comment in margin
3. Verify comment text visible

**Expected:** Comment readable
- [ ] Pass

### TC-9.4.3: Reply to Comment
**Steps:**
1. Insert comment
2. Click Reply
3. Type reply
4. Verify threaded reply

**Expected:** Comment threading
- [ ] Pass

### TC-9.4.4: Resolve Comment
**Steps:**
1. Insert comment
2. Click Resolve
3. Verify comment marked as resolved (dimmed)

**Expected:** Comment resolution
- [ ] Pass

### TC-9.4.5: Reopen Comment
**Steps:**
1. Resolve a comment
2. Reopen it
3. Verify comment active again

**Expected:** Can reopen resolved
- [ ] Pass

### TC-9.4.6: Delete Comment
**Steps:**
1. Insert comment
2. Right-click > Delete Comment
3. Verify comment removed

**Expected:** Comment deletion
- [ ] Pass

### TC-9.4.7: Delete All Comments
**Steps:**
1. Insert multiple comments
2. Delete All Comments
3. Verify all removed

**Expected:** Bulk comment delete
- [ ] Pass

---

## 9.5 Comments - Display

### TC-9.5.1: Show Comments
**Steps:**
1. Add comments
2. Toggle Show Comments on
3. Verify comments visible

**Expected:** Comments shown
- [ ] Pass

### TC-9.5.2: Hide Comments
**Steps:**
1. Toggle Show Comments off
2. Verify comments hidden (highlights may remain)

**Expected:** Comments hidden
- [ ] Pass

### TC-9.5.3: Show Specific Reviewer Comments
**Steps:**
1. Multiple reviewers add comments
2. Filter to single reviewer
3. Verify only their comments shown

**Expected:** Filter comments by reviewer
- [ ] Pass

### TC-9.5.4: Reviewing Pane
**Steps:**
1. Open Reviewing Pane
2. Verify all comments listed
3. Click entry to navigate

**Expected:** Review pane shows all
- [ ] Pass

---

## 9.6 Compare Documents

### TC-9.6.1: Compare Two Documents
**Steps:**
1. Create two versions of a document
2. Review > Compare > Compare
3. Select original and revised
4. Verify comparison document generated

**Expected:** Comparison shows differences
- [ ] Pass

### TC-9.6.2: Compare Shows Insertions
**Steps:**
1. Compare documents
2. Find text added in revised version
3. Verify shown as insertion

**Expected:** Additions detected
- [ ] Pass

### TC-9.6.3: Compare Shows Deletions
**Steps:**
1. Compare documents
2. Find text removed in revised version
3. Verify shown as deletion

**Expected:** Deletions detected
- [ ] Pass

### TC-9.6.4: Compare with Specific Settings
**Steps:**
1. Open Compare settings
2. Choose to compare formatting changes
3. Verify formatting differences shown

**Expected:** Configurable comparison
- [ ] Pass

---

## 9.7 Combine Documents

### TC-9.7.1: Combine Two Reviews
**Steps:**
1. Send document to two reviewers
2. Get back two reviewed copies
3. Review > Combine
4. Verify both sets of changes merged

**Expected:** Multiple reviews combined
- [ ] Pass

### TC-9.7.2: Combine Conflict Resolution
**Steps:**
1. Two reviewers edit same text differently
2. Combine documents
3. Verify conflict shown for resolution

**Expected:** Conflicts identified
- [ ] Pass

---

## 9.8 Protect Document

### TC-9.8.1: Allow Only Comments
**Steps:**
1. Review > Restrict Editing
2. Allow only comments
3. Try to edit text
4. Verify editing blocked, comments allowed

**Expected:** Edit restriction with comments
- [ ] Pass

### TC-9.8.2: Allow Only Track Changes
**Steps:**
1. Restrict to tracked changes only
2. Verify all edits are tracked (can't turn off)

**Expected:** Forced tracking
- [ ] Pass

### TC-9.8.3: Password Protect Restrictions
**Steps:**
1. Set editing restrictions
2. Add password
3. Try to remove restrictions without password
4. Verify password required

**Expected:** Password protection
- [ ] Pass

### TC-9.8.4: Remove Protection
**Steps:**
1. Protect document with password
2. Enter password to unprotect
3. Verify editing enabled

**Expected:** Can unprotect with password
- [ ] Pass

---

## Summary

| Section | Tests | Passed | Failed |
|---------|-------|--------|--------|
| 9.1 Track Changes - Basic | 5 | | |
| 9.2 Track Changes - Display | 6 | | |
| 9.3 Accept/Reject Changes | 7 | | |
| 9.4 Comments - Basic | 7 | | |
| 9.5 Comments - Display | 4 | | |
| 9.6 Compare Documents | 4 | | |
| 9.7 Combine Documents | 2 | | |
| 9.8 Protect Document | 4 | | |
| **Total** | **39** | | |
