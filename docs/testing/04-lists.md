# Test Group 4: Lists

## Overview
Tests for bulleted lists, numbered lists, multi-level lists, and list formatting.

---

## 4.1 Bulleted Lists

### TC-4.1.1: Create Bulleted List
**Steps:**
1. Click bullet list button (or Format > Bullets)
2. Type "Item 1", Enter
3. Type "Item 2", Enter
4. Type "Item 3"
5. Verify all items have bullets

**Expected:** Three items with bullets
- [ ] Pass

### TC-4.1.2: Convert Paragraph to Bullet
**Steps:**
1. Type three paragraphs (normal text)
2. Select all three
3. Apply bullets
4. Verify all become bullet items

**Expected:** Existing paragraphs converted to list
- [ ] Pass

### TC-4.1.3: Remove Bullets
**Steps:**
1. Create bulleted list
2. Select all items
3. Click bullet button again (toggle off)
4. Verify bullets removed, text remains

**Expected:** Bullets removed, text preserved
- [ ] Pass

### TC-4.1.4: Change Bullet Style
**Steps:**
1. Create bulleted list
2. Select list
3. Choose different bullet style (square, arrow, etc.)
4. Verify bullet character changes

**Expected:** Bullet style changed
- [ ] Pass

### TC-4.1.5: Custom Bullet Symbol
**Steps:**
1. Create bulleted list
2. Open bullet options
3. Select custom symbol (e.g., checkmark)
4. Apply
5. Verify custom bullet appears

**Expected:** Custom bullet symbol used
- [ ] Pass

---

## 4.2 Numbered Lists

### TC-4.2.1: Create Numbered List
**Steps:**
1. Click numbered list button
2. Type "First", Enter
3. Type "Second", Enter
4. Type "Third"
5. Verify items numbered 1, 2, 3

**Expected:** Sequential numbering
- [ ] Pass

### TC-4.2.2: Continue Numbering
**Steps:**
1. Create numbered list (1, 2, 3)
2. Press Enter twice (exit list)
3. Type normal paragraph
4. Start new numbered list
5. Verify it can continue (4, 5, 6) or restart (1, 2, 3)

**Expected:** Option to continue or restart numbering
- [ ] Pass

### TC-4.2.3: Restart Numbering
**Steps:**
1. Create numbered list
2. Right-click on item 3
3. Choose "Restart at 1"
4. Verify numbering restarts

**Expected:** Numbering resets to 1
- [ ] Pass

### TC-4.2.4: Change Number Format
**Steps:**
1. Create numbered list
2. Change format to:
   - a, b, c
   - i, ii, iii
   - A, B, C
3. Verify each format works

**Expected:** Number formats change appropriately
- [ ] Pass

### TC-4.2.5: Set Starting Number
**Steps:**
1. Create numbered list
2. Set starting value to 5
3. Verify first item is "5"

**Expected:** List starts at specified number
- [ ] Pass

---

## 4.3 Multi-Level Lists

### TC-4.3.1: Increase Indent Level
**Steps:**
1. Create numbered list with 3 items
2. Place cursor on item 2
3. Press Tab (or Increase Indent)
4. Verify item 2 becomes sub-item (1.1 or a)

**Expected:** Item becomes nested sub-item
- [ ] Pass

### TC-4.3.2: Decrease Indent Level
**Steps:**
1. Create nested list item
2. Press Shift+Tab (or Decrease Indent)
3. Verify item moves up one level

**Expected:** Item moves to parent level
- [ ] Pass

### TC-4.3.3: Three-Level List
**Steps:**
1. Create list:
   - 1. First
     - 1.1 Sub-first
       - 1.1.1 Sub-sub-first
2. Verify three levels display correctly

**Expected:** Three levels properly indented and numbered
- [ ] Pass

### TC-4.3.4: Mixed Bullet and Number
**Steps:**
1. Create numbered list
2. Add sub-items as bullets
3. Verify:
   - 1. Numbered
     - Bullet sub-item
     - Bullet sub-item
   - 2. Numbered

**Expected:** Mixed numbering/bullets in same list
- [ ] Pass

### TC-4.3.5: Outline Numbering
**Steps:**
1. Apply outline/legal numbering style
2. Create multi-level list
3. Verify format like:
   - 1.
   - 1.1
   - 1.1.1
   - 1.2

**Expected:** Legal/outline numbering format
- [ ] Pass

---

## 4.4 List Editing

### TC-4.4.1: Add Item in Middle
**Steps:**
1. Create list: 1, 2, 3
2. Place cursor at end of item 1
3. Press Enter
4. Type "New Item"
5. Verify: 1, 2 (new), 3, 4

**Expected:** Items renumber automatically
- [ ] Pass

### TC-4.4.2: Delete List Item
**Steps:**
1. Create list: 1, 2, 3, 4
2. Delete item 2 entirely
3. Verify: 1, 2, 3 (renumbered)

**Expected:** Remaining items renumber
- [ ] Pass

### TC-4.4.3: Move List Item Up
**Steps:**
1. Create list with 3 items
2. Select item 3
3. Move up (Alt+Shift+Up or drag)
4. Verify item order changes

**Expected:** Item reordered, numbering updates
- [ ] Pass

### TC-4.4.4: Move List Item Down
**Steps:**
1. Create list with 3 items
2. Select item 1
3. Move down
4. Verify item order changes

**Expected:** Item reordered, numbering updates
- [ ] Pass

### TC-4.4.5: Copy List Item
**Steps:**
1. Create numbered list
2. Copy item 2
3. Paste after item 3
4. Verify new item is numbered 4

**Expected:** Pasted item gets correct number
- [ ] Pass

---

## 4.5 List Formatting

### TC-4.5.1: Format List Item Text
**Steps:**
1. Create bulleted list
2. Apply bold to one item's text
3. Verify only text is bold, not bullet

**Expected:** Text formatting independent of bullet
- [ ] Pass

### TC-4.5.2: Change List Indentation
**Steps:**
1. Create list
2. Adjust left indent of entire list
3. Verify all items move together

**Expected:** Entire list indentation changes
- [ ] Pass

### TC-4.5.3: Adjust Text Indent (Bullet Distance)
**Steps:**
1. Create bulleted list
2. Increase distance between bullet and text
3. Verify spacing increases

**Expected:** Bullet-to-text distance adjustable
- [ ] Pass

### TC-4.5.4: Line Spacing in List
**Steps:**
1. Create list
2. Apply 1.5 line spacing
3. Verify spacing between items increases

**Expected:** Line spacing applies to list
- [ ] Pass

---

## 4.6 Special List Cases

### TC-4.6.1: Multi-Paragraph List Item
**Steps:**
1. Create numbered list
2. In item 1, press Shift+Enter (soft return)
3. Type second line
4. Verify both lines are part of item 1

**Expected:** Single list item spans multiple lines
- [ ] Pass

### TC-4.6.2: List with Image
**Steps:**
1. Create bulleted list
2. Insert image in one item
3. Verify image aligns with list

**Expected:** Image fits within list item
- [ ] Pass

### TC-4.6.3: Nested List Across Pages
**Steps:**
1. Create long multi-level list
2. Verify list continues correctly across page break
3. Check numbering remains correct

**Expected:** List maintains integrity across pages
- [ ] Pass

### TC-4.6.4: Empty List Item
**Steps:**
1. Create numbered list
2. Press Enter twice on empty item
3. Verify list ends, returns to normal paragraph

**Expected:** Double-enter exits list
- [ ] Pass

### TC-4.6.5: List from AutoFormat
**Steps:**
1. Type "1. First item" and press Enter
2. Verify auto-converts to numbered list
3. Type "* Bullet item" and Enter
4. Verify auto-converts to bullet list

**Expected:** Auto-format creates lists from typing
- [ ] Pass

---

## Summary

| Section | Tests | Passed | Failed |
|---------|-------|--------|--------|
| 4.1 Bulleted Lists | 5 | | |
| 4.2 Numbered Lists | 5 | | |
| 4.3 Multi-Level Lists | 5 | | |
| 4.4 List Editing | 5 | | |
| 4.5 List Formatting | 4 | | |
| 4.6 Special Cases | 5 | | |
| **Total** | **29** | | |
