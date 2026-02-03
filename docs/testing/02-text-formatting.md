# Test Group 2: Text Formatting

## Overview
Tests for character and inline formatting including bold, italic, underline, fonts, colors, and other text decorations.

---

## 2.1 Bold/Italic/Underline

### TC-2.1.1: Apply Bold
**Steps:**
1. Type "Hello World"
2. Select "Hello"
3. Press Ctrl+B
4. Verify "Hello" appears bold

**Expected:** Selected text becomes bold
- [ ] Pass

### TC-2.1.2: Toggle Bold Off
**Steps:**
1. Type and bold "Hello"
2. Select "Hello"
3. Press Ctrl+B again
4. Verify bold is removed

**Expected:** Ctrl+B toggles bold off
- [ ] Pass

### TC-2.1.3: Apply Italic
**Steps:**
1. Type "Hello World"
2. Select "World"
3. Press Ctrl+I
4. Verify "World" appears italic

**Expected:** Selected text becomes italic
- [ ] Pass

### TC-2.1.4: Apply Underline
**Steps:**
1. Type "Hello World"
2. Select "Hello"
3. Press Ctrl+U
4. Verify "Hello" has underline

**Expected:** Selected text becomes underlined
- [ ] Pass

### TC-2.1.5: Combined Formatting
**Steps:**
1. Type "Text"
2. Select "Text"
3. Apply Bold (Ctrl+B)
4. Apply Italic (Ctrl+I)
5. Apply Underline (Ctrl+U)
6. Verify text is bold, italic, AND underlined

**Expected:** Multiple formats can be combined
- [ ] Pass

### TC-2.1.6: Type with Formatting Active
**Steps:**
1. Press Ctrl+B (bold on)
2. Type "Bold Text"
3. Press Ctrl+B (bold off)
4. Type " Normal Text"
5. Verify "Bold Text" is bold, " Normal Text" is not

**Expected:** Formatting applies to newly typed text
- [ ] Pass

---

## 2.2 Font Family

### TC-2.2.1: Change Font
**Steps:**
1. Type "Hello World"
2. Select all text
3. Change font to "Arial" (via toolbar/menu)
4. Verify font changes

**Expected:** Font family changes for selected text
- [ ] Pass

### TC-2.2.2: Multiple Fonts in Document
**Steps:**
1. Type "Line 1" with Arial
2. Type "Line 2" with Times New Roman
3. Verify each line uses different font

**Expected:** Different fonts can coexist in document
- [ ] Pass

### TC-2.2.3: Font Fallback
**Steps:**
1. Select a rare/unavailable font
2. Type text
3. Verify text renders with fallback font (not missing glyphs)

**Expected:** Missing fonts fall back gracefully
- [ ] Pass

---

## 2.3 Font Size

### TC-2.3.1: Change Font Size
**Steps:**
1. Type "Hello"
2. Select text
3. Change size to 24pt
4. Verify text is larger

**Expected:** Font size changes visually
- [ ] Pass

### TC-2.3.2: Mixed Sizes
**Steps:**
1. Type "Big" (24pt)
2. Type "Small" (10pt)
3. Verify different sizes on same line

**Expected:** Different sizes render correctly inline
- [ ] Pass

### TC-2.3.3: Increase/Decrease Size
**Steps:**
1. Type "Hello" at 12pt
2. Select text
3. Use "Increase Font Size" (Ctrl+Shift+> or toolbar)
4. Verify size increases (e.g., to 14pt)
5. Use "Decrease Font Size"
6. Verify size decreases

**Expected:** Font size increments/decrements work
- [ ] Pass

---

## 2.4 Text Color

### TC-2.4.1: Change Text Color
**Steps:**
1. Type "Hello World"
2. Select "Hello"
3. Change text color to red
4. Verify "Hello" appears red

**Expected:** Text color changes for selection
- [ ] Pass

### TC-2.4.2: Change Highlight Color
**Steps:**
1. Type "Hello World"
2. Select "World"
3. Apply yellow highlight
4. Verify yellow background behind "World"

**Expected:** Highlight/background color applies
- [ ] Pass

### TC-2.4.3: Multiple Colors
**Steps:**
1. Type "Red Green Blue"
2. Color "Red" red
3. Color "Green" green
4. Color "Blue" blue
5. Verify each word has correct color

**Expected:** Multiple text colors in same paragraph
- [ ] Pass

---

## 2.5 Strikethrough and Effects

### TC-2.5.1: Strikethrough
**Steps:**
1. Type "Delete this"
2. Select text
3. Apply strikethrough
4. Verify line through text

**Expected:** Strikethrough line appears
- [ ] Pass

### TC-2.5.2: Superscript
**Steps:**
1. Type "x2"
2. Select "2"
3. Apply superscript
4. Verify "2" is raised and smaller

**Expected:** Superscript raises and shrinks text
- [ ] Pass

### TC-2.5.3: Subscript
**Steps:**
1. Type "H2O"
2. Select "2"
3. Apply subscript
4. Verify "2" is lowered and smaller

**Expected:** Subscript lowers and shrinks text
- [ ] Pass

### TC-2.5.4: Small Caps
**Steps:**
1. Type "Hello World"
2. Select text
3. Apply small caps
4. Verify lowercase letters become small capitals

**Expected:** Small caps formatting applies
- [ ] Pass

### TC-2.5.5: All Caps
**Steps:**
1. Type "Hello World"
2. Select text
3. Apply all caps
4. Verify text displays as "HELLO WORLD"

**Expected:** All caps transforms display
- [ ] Pass

---

## 2.6 Clear Formatting

### TC-2.6.1: Clear All Formatting
**Steps:**
1. Type "Hello" with bold, italic, red color, 24pt
2. Select text
3. Clear formatting (Ctrl+Space or menu)
4. Verify text returns to default style

**Expected:** All formatting removed, returns to Normal style
- [ ] Pass

### TC-2.6.2: Clear Formatting Preserves Text
**Steps:**
1. Apply various formatting to "Hello World"
2. Clear formatting
3. Verify text content unchanged, only formatting cleared

**Expected:** Text preserved, formatting removed
- [ ] Pass

---

## 2.7 Format Painter

### TC-2.7.1: Copy Format Once
**Steps:**
1. Type "Source" with bold, red, 16pt
2. Type "Target" with no formatting
3. Select "Source"
4. Click Format Painter
5. Click/drag over "Target"
6. Verify "Target" now has bold, red, 16pt

**Expected:** Format painter copies formatting
- [ ] Pass

### TC-2.7.2: Copy Format Multiple (Double-Click)
**Steps:**
1. Format "Source" text
2. Double-click Format Painter
3. Apply to "Target1"
4. Apply to "Target2"
5. Press Escape
6. Verify both targets have source formatting

**Expected:** Double-click enables multiple applications
- [ ] Pass

---

## 2.8 Character Spacing

### TC-2.8.1: Expanded Spacing
**Steps:**
1. Type "Hello"
2. Select text
3. Open Font dialog
4. Set character spacing to "Expanded" by 2pt
5. Verify letters are further apart

**Expected:** Letters have increased spacing
- [ ] Pass

### TC-2.8.2: Condensed Spacing
**Steps:**
1. Type "Hello"
2. Select text
3. Set character spacing to "Condensed" by 1pt
4. Verify letters are closer together

**Expected:** Letters have decreased spacing
- [ ] Pass

---

## 2.9 Edge Cases

### TC-2.9.1: Format Empty Selection
**Steps:**
1. Place cursor (no selection)
2. Press Ctrl+B
3. Type "Hello"
4. Verify "Hello" is bold

**Expected:** Formatting applies to subsequent typing
- [ ] Pass

### TC-2.9.2: Format Across Paragraphs
**Steps:**
1. Type "Line 1"
2. Press Enter
3. Type "Line 2"
4. Select from "1" to "Line 2"
5. Apply bold
6. Verify formatting crosses paragraph boundary

**Expected:** Formatting spans multiple paragraphs
- [ ] Pass

### TC-2.9.3: Partial Word Formatting
**Steps:**
1. Type "Hello"
2. Select "ell"
3. Apply bold
4. Verify only "ell" is bold, "H" and "o" are not

**Expected:** Formatting applies to exact selection
- [ ] Pass

---

## Summary

| Section | Tests | Passed | Failed |
|---------|-------|--------|--------|
| 2.1 Bold/Italic/Underline | 6 | | |
| 2.2 Font Family | 3 | | |
| 2.3 Font Size | 3 | | |
| 2.4 Text Color | 3 | | |
| 2.5 Strikethrough/Effects | 5 | | |
| 2.6 Clear Formatting | 2 | | |
| 2.7 Format Painter | 2 | | |
| 2.8 Character Spacing | 2 | | |
| 2.9 Edge Cases | 3 | | |
| **Total** | **29** | | |
