# Test Group 1: Basic Editing

## Overview
Tests for fundamental text editing operations including typing, cursor movement, selection, and undo/redo.

---

## 1.1 Text Input

### TC-1.1.1: Basic Typing
**Steps:**
1. Create a new document
2. Type "Hello World"
3. Verify text appears correctly

**Expected:** Text appears as typed, cursor moves forward
- [ ] Pass

### TC-1.1.2: Multi-line Input
**Steps:**
1. Type "Line 1"
2. Press Enter
3. Type "Line 2"
4. Press Enter
5. Type "Line 3"

**Expected:** Three separate paragraphs created
- [ ] Pass

### TC-1.1.3: Special Characters
**Steps:**
1. Type: `!@#$%^&*()_+-=[]{}|;':",.<>?/\``
2. Verify all characters display correctly

**Expected:** All special characters render correctly
- [ ] Pass

### TC-1.1.4: Unicode Characters
**Steps:**
1. Type: `Ñ é ü ö ä ß`
2. Type: `你好世界`
3. Type: `مرحبا`
4. Type: `🎉 👍 🚀`

**Expected:** All Unicode characters display correctly
- [ ] Pass

### TC-1.1.5: IME Input (if applicable)
**Steps:**
1. Switch to Japanese IME
2. Type "nihongo" and select 日本語
3. Verify text is inserted

**Expected:** IME composition works, text inserted correctly
- [ ] Pass

---

## 1.2 Cursor Navigation

### TC-1.2.1: Arrow Keys
**Steps:**
1. Type "Hello World"
2. Press Left Arrow 5 times
3. Verify cursor is before "W"
4. Press Right Arrow 2 times
5. Verify cursor is after "Wo"

**Expected:** Cursor moves one character per arrow press
- [ ] Pass

### TC-1.2.2: Home/End Keys
**Steps:**
1. Type "Hello World"
2. Press Home
3. Verify cursor at start of line
4. Press End
5. Verify cursor at end of line

**Expected:** Home goes to line start, End goes to line end
- [ ] Pass

### TC-1.2.3: Ctrl+Arrow (Word Navigation)
**Steps:**
1. Type "one two three four"
2. Press Ctrl+Left
3. Verify cursor jumps to start of "four"
4. Press Ctrl+Left again
5. Verify cursor jumps to start of "three"

**Expected:** Ctrl+Arrow moves by word
- [ ] Pass

### TC-1.2.4: Ctrl+Home/End (Document Navigation)
**Steps:**
1. Create document with multiple paragraphs
2. Press Ctrl+End
3. Verify cursor at document end
4. Press Ctrl+Home
5. Verify cursor at document start

**Expected:** Ctrl+Home/End navigates to document boundaries
- [ ] Pass

### TC-1.2.5: Page Up/Down
**Steps:**
1. Create a document longer than one page
2. Press Page Down
3. Verify view scrolls down approximately one page
4. Press Page Up
5. Verify view scrolls up

**Expected:** Page Up/Down scrolls by roughly one viewport
- [ ] Pass

### TC-1.2.6: Click to Position
**Steps:**
1. Type "Hello World"
2. Click between "o" and " " (space)
3. Verify cursor is positioned there
4. Type "X"
5. Verify result is "HelloX World"

**Expected:** Mouse click positions cursor accurately
- [ ] Pass

---

## 1.3 Text Selection

### TC-1.3.1: Shift+Arrow Selection
**Steps:**
1. Type "Hello World"
2. Press Shift+Left 5 times
3. Verify "World" is selected (highlighted)

**Expected:** Text is highlighted as selection extends
- [ ] Pass

### TC-1.3.2: Shift+Ctrl+Arrow (Word Selection)
**Steps:**
1. Type "one two three"
2. Press Shift+Ctrl+Left
3. Verify "three" is selected
4. Press Shift+Ctrl+Left again
5. Verify "two three" is selected

**Expected:** Shift+Ctrl+Arrow selects by word
- [ ] Pass

### TC-1.3.3: Double-Click Word Selection
**Steps:**
1. Type "Hello World Test"
2. Double-click on "World"
3. Verify only "World" is selected

**Expected:** Double-click selects the word under cursor
- [ ] Pass

### TC-1.3.4: Triple-Click Paragraph Selection
**Steps:**
1. Type a paragraph with multiple sentences
2. Triple-click anywhere in the paragraph
3. Verify entire paragraph is selected

**Expected:** Triple-click selects entire paragraph
- [ ] Pass

### TC-1.3.5: Ctrl+A (Select All)
**Steps:**
1. Type multiple paragraphs
2. Press Ctrl+A
3. Verify all content is selected

**Expected:** Ctrl+A selects entire document
- [ ] Pass

### TC-1.3.6: Click and Drag Selection
**Steps:**
1. Type "Hello World"
2. Click before "H", drag to after "o"
3. Verify "Hello" is selected

**Expected:** Drag creates selection from start to end point
- [ ] Pass

### TC-1.3.7: Shift+Click Extend Selection
**Steps:**
1. Type "Hello World Test"
2. Click before "H"
3. Shift+Click after "Test"
4. Verify entire text is selected

**Expected:** Shift+Click extends selection to click point
- [ ] Pass

---

## 1.4 Delete Operations

### TC-1.4.1: Backspace
**Steps:**
1. Type "Hello"
2. Press Backspace
3. Verify "Hell" remains

**Expected:** Backspace deletes character before cursor
- [ ] Pass

### TC-1.4.2: Delete Key
**Steps:**
1. Type "Hello"
2. Press Home
3. Press Delete
4. Verify "ello" remains

**Expected:** Delete removes character after cursor
- [ ] Pass

### TC-1.4.3: Delete Selection
**Steps:**
1. Type "Hello World"
2. Select "World"
3. Press Delete (or Backspace)
4. Verify "Hello " remains

**Expected:** Delete/Backspace removes selected text
- [ ] Pass

### TC-1.4.4: Ctrl+Backspace (Delete Word)
**Steps:**
1. Type "Hello World"
2. Press Ctrl+Backspace
3. Verify "Hello " remains

**Expected:** Ctrl+Backspace deletes previous word
- [ ] Pass

### TC-1.4.5: Ctrl+Delete (Delete Word Forward)
**Steps:**
1. Type "Hello World"
2. Press Home
3. Press Ctrl+Delete
4. Verify " World" remains (or "World" depending on impl)

**Expected:** Ctrl+Delete deletes next word
- [ ] Pass

---

## 1.5 Undo/Redo

### TC-1.5.1: Basic Undo
**Steps:**
1. Type "Hello"
2. Press Ctrl+Z
3. Verify text is removed or reverted

**Expected:** Undo reverses last action
- [ ] Pass

### TC-1.5.2: Basic Redo
**Steps:**
1. Type "Hello"
2. Press Ctrl+Z (undo)
3. Press Ctrl+Y (or Ctrl+Shift+Z)
4. Verify "Hello" is restored

**Expected:** Redo restores undone action
- [ ] Pass

### TC-1.5.3: Multiple Undo Steps
**Steps:**
1. Type "A"
2. Type "B"
3. Type "C"
4. Press Ctrl+Z three times
5. Verify document is empty

**Expected:** Multiple undos reverse multiple actions
- [ ] Pass

### TC-1.5.4: Undo After Delete
**Steps:**
1. Type "Hello World"
2. Select and delete "World"
3. Press Ctrl+Z
4. Verify "Hello World" is restored

**Expected:** Undo restores deleted text
- [ ] Pass

### TC-1.5.5: Undo Formatting Change
**Steps:**
1. Type "Hello"
2. Select "Hello"
3. Apply bold (Ctrl+B)
4. Press Ctrl+Z
5. Verify text is no longer bold

**Expected:** Undo reverses formatting changes
- [ ] Pass

### TC-1.5.6: Redo After New Action Clears Redo Stack
**Steps:**
1. Type "A"
2. Press Ctrl+Z
3. Type "B"
4. Press Ctrl+Y
5. Verify nothing happens (redo stack cleared)

**Expected:** New action after undo clears redo history
- [ ] Pass

---

## 1.6 Copy/Cut/Paste

### TC-1.6.1: Copy and Paste
**Steps:**
1. Type "Hello"
2. Select "Hello"
3. Press Ctrl+C
4. Move cursor to end
5. Press Ctrl+V
6. Verify "HelloHello"

**Expected:** Copy preserves original, paste inserts copy
- [ ] Pass

### TC-1.6.2: Cut and Paste
**Steps:**
1. Type "Hello World"
2. Select "Hello"
3. Press Ctrl+X
4. Verify " World" remains
5. Move to end
6. Press Ctrl+V
7. Verify " WorldHello"

**Expected:** Cut removes original, paste inserts it
- [ ] Pass

### TC-1.6.3: Paste Replaces Selection
**Steps:**
1. Type "Hello World"
2. Copy "Hello"
3. Select "World"
4. Paste
5. Verify "Hello Hello"

**Expected:** Paste replaces selected text
- [ ] Pass

### TC-1.6.4: Paste Multiple Times
**Steps:**
1. Type "A"
2. Copy "A"
3. Paste 5 times
4. Verify "AAAAAA"

**Expected:** Clipboard content can be pasted multiple times
- [ ] Pass

### TC-1.6.5: Paste from External Source
**Steps:**
1. Copy text from another application
2. Paste into the editor
3. Verify text appears

**Expected:** External clipboard content can be pasted
- [ ] Pass

---

## 1.7 Find and Replace

### TC-1.7.1: Find Text
**Steps:**
1. Type "Hello World Hello"
2. Open Find (Ctrl+F)
3. Search for "Hello"
4. Verify first occurrence is highlighted
5. Click "Find Next"
6. Verify second occurrence is highlighted

**Expected:** Find locates and highlights matches
- [ ] Pass

### TC-1.7.2: Find Case Sensitive
**Steps:**
1. Type "Hello hello HELLO"
2. Open Find
3. Enable "Match case"
4. Search for "Hello"
5. Verify only first occurrence matches

**Expected:** Case sensitive search works
- [ ] Pass

### TC-1.7.3: Find Whole Words
**Steps:**
1. Type "Hello HelloWorld"
2. Open Find
3. Enable "Whole words"
4. Search for "Hello"
5. Verify only standalone "Hello" matches

**Expected:** Whole word search ignores partial matches
- [ ] Pass

### TC-1.7.4: Replace Single
**Steps:**
1. Type "Hello World"
2. Open Replace (Ctrl+H)
3. Find: "World", Replace: "Universe"
4. Click Replace
5. Verify "Hello Universe"

**Expected:** Replace changes single occurrence
- [ ] Pass

### TC-1.7.5: Replace All
**Steps:**
1. Type "cat dog cat dog cat"
2. Open Replace
3. Find: "cat", Replace: "bird"
4. Click Replace All
5. Verify "bird dog bird dog bird"

**Expected:** Replace All changes all occurrences
- [ ] Pass

---

## Summary

| Section | Tests | Passed | Failed |
|---------|-------|--------|--------|
| 1.1 Text Input | 5 | | |
| 1.2 Cursor Navigation | 6 | | |
| 1.3 Text Selection | 7 | | |
| 1.4 Delete Operations | 5 | | |
| 1.5 Undo/Redo | 6 | | |
| 1.6 Copy/Cut/Paste | 5 | | |
| 1.7 Find and Replace | 5 | | |
| **Total** | **39** | | |
