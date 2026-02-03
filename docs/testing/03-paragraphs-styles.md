# Test Group 3: Paragraphs and Styles

## Overview
Tests for paragraph formatting, alignment, indentation, spacing, and style application.

---

## 3.1 Paragraph Alignment

### TC-3.1.1: Left Align
**Steps:**
1. Type a paragraph
2. Select or place cursor in paragraph
3. Apply left alignment (Ctrl+L or toolbar)
4. Verify text aligns to left margin

**Expected:** Text left-aligned
- [ ] Pass

### TC-3.1.2: Center Align
**Steps:**
1. Type a paragraph
2. Apply center alignment (Ctrl+E)
3. Verify text centered

**Expected:** Text centered between margins
- [ ] Pass

### TC-3.1.3: Right Align
**Steps:**
1. Type a paragraph
2. Apply right alignment (Ctrl+R)
3. Verify text aligns to right margin

**Expected:** Text right-aligned
- [ ] Pass

### TC-3.1.4: Justify
**Steps:**
1. Type a long paragraph (multiple lines)
2. Apply justify (Ctrl+J)
3. Verify text stretches to both margins

**Expected:** Text justified (except last line)
- [ ] Pass

---

## 3.2 Indentation

### TC-3.2.1: Left Indent
**Steps:**
1. Type a paragraph
2. Increase left indent (Tab or toolbar button)
3. Verify paragraph moves right

**Expected:** Paragraph indented from left
- [ ] Pass

### TC-3.2.2: First Line Indent
**Steps:**
1. Type a multi-line paragraph
2. Set first line indent to 0.5"
3. Verify only first line is indented

**Expected:** First line indented, subsequent lines at margin
- [ ] Pass

### TC-3.2.3: Hanging Indent
**Steps:**
1. Type a paragraph
2. Set hanging indent
3. Verify first line at margin, rest indented

**Expected:** Hanging indent creates bibliography-style format
- [ ] Pass

### TC-3.2.4: Right Indent
**Steps:**
1. Type a long paragraph
2. Set right indent to 1"
3. Verify text wraps earlier (margin moved in)

**Expected:** Right margin moved inward
- [ ] Pass

### TC-3.2.5: Nested Indentation
**Steps:**
1. Type paragraph 1
2. Type paragraph 2 with 1 indent level
3. Type paragraph 3 with 2 indent levels
4. Verify progressive indentation

**Expected:** Multiple indent levels work
- [ ] Pass

---

## 3.3 Line Spacing

### TC-3.3.1: Single Spacing
**Steps:**
1. Type multi-line paragraph
2. Set line spacing to "Single"
3. Verify lines close together

**Expected:** Single line spacing (1.0)
- [ ] Pass

### TC-3.3.2: 1.5 Line Spacing
**Steps:**
1. Type multi-line paragraph
2. Set line spacing to "1.5 lines"
3. Verify increased spacing

**Expected:** 1.5 line spacing
- [ ] Pass

### TC-3.3.3: Double Spacing
**Steps:**
1. Type multi-line paragraph
2. Set line spacing to "Double"
3. Verify doubled spacing

**Expected:** Double line spacing (2.0)
- [ ] Pass

### TC-3.3.4: Exact Line Spacing
**Steps:**
1. Type paragraph
2. Set line spacing to "Exactly 18pt"
3. Verify consistent 18pt spacing

**Expected:** Exact spacing regardless of font size
- [ ] Pass

---

## 3.4 Paragraph Spacing

### TC-3.4.1: Space Before Paragraph
**Steps:**
1. Type two paragraphs
2. Set "Space Before" to 12pt on second paragraph
3. Verify gap above second paragraph

**Expected:** Space added before paragraph
- [ ] Pass

### TC-3.4.2: Space After Paragraph
**Steps:**
1. Type two paragraphs
2. Set "Space After" to 12pt on first paragraph
3. Verify gap below first paragraph

**Expected:** Space added after paragraph
- [ ] Pass

### TC-3.4.3: Remove Paragraph Spacing
**Steps:**
1. Type paragraphs with default spacing
2. Set both before and after to 0pt
3. Verify paragraphs have no extra space

**Expected:** Paragraphs with no spacing between them
- [ ] Pass

---

## 3.5 Styles

### TC-3.5.1: Apply Heading 1
**Steps:**
1. Type "Chapter Title"
2. Apply Heading 1 style
3. Verify text is large, bold, styled

**Expected:** Heading 1 formatting applied
- [ ] Pass

### TC-3.5.2: Apply Heading 2
**Steps:**
1. Type "Section Title"
2. Apply Heading 2 style
3. Verify appropriate heading formatting

**Expected:** Heading 2 formatting applied
- [ ] Pass

### TC-3.5.3: Apply Normal Style
**Steps:**
1. Apply Heading 1 to text
2. Apply Normal style
3. Verify text returns to body text formatting

**Expected:** Normal style resets formatting
- [ ] Pass

### TC-3.5.4: Apply Quote Style
**Steps:**
1. Type a quotation
2. Apply Quote or Block Quote style
3. Verify italic/indented formatting

**Expected:** Quote style applies
- [ ] Pass

### TC-3.5.5: Style Inheritance
**Steps:**
1. Apply Heading 1
2. Manually change font color to red
3. Verify heading is red (local override preserved)

**Expected:** Local formatting overrides style
- [ ] Pass

### TC-3.5.6: Multiple Styles in Document
**Steps:**
1. Create document with:
   - Heading 1
   - Normal paragraph
   - Heading 2
   - Normal paragraph
   - Quote
2. Verify each has distinct formatting

**Expected:** All styles render correctly together
- [ ] Pass

---

## 3.6 Modify Styles

### TC-3.6.1: Modify Existing Style
**Steps:**
1. Open style editor/modifier
2. Select Heading 1
3. Change font color to blue
4. Apply changes
5. Verify all Heading 1 text updates

**Expected:** Style modification updates all instances
- [ ] Pass

### TC-3.6.2: Create New Style
**Steps:**
1. Format paragraph (red, 14pt, italic)
2. Create new style from selection "MyStyle"
3. Apply "MyStyle" to another paragraph
4. Verify same formatting applies

**Expected:** Custom style created and applicable
- [ ] Pass

### TC-3.6.3: Delete Custom Style
**Steps:**
1. Create custom style
2. Apply to some text
3. Delete the style
4. Verify text reverts to Normal or base style

**Expected:** Style deletion handled gracefully
- [ ] Pass

---

## 3.7 Paragraph Borders and Shading

### TC-3.7.1: Paragraph Border
**Steps:**
1. Type a paragraph
2. Add border around paragraph
3. Verify visible border

**Expected:** Border surrounds paragraph
- [ ] Pass

### TC-3.7.2: Paragraph Shading
**Steps:**
1. Type a paragraph
2. Apply background shading (light gray)
3. Verify background color appears

**Expected:** Background color behind paragraph
- [ ] Pass

### TC-3.7.3: Border and Shading Combined
**Steps:**
1. Type a paragraph
2. Add black border
3. Add light blue shading
4. Verify both visible

**Expected:** Border and shading work together
- [ ] Pass

---

## 3.8 Keep Options

### TC-3.8.1: Keep with Next
**Steps:**
1. Type heading paragraph
2. Set "Keep with next" on heading
3. Insert page break before heading
4. Verify heading stays with following paragraph

**Expected:** Heading doesn't orphan at page bottom
- [ ] Pass

### TC-3.8.2: Keep Lines Together
**Steps:**
1. Type long paragraph near page break
2. Set "Keep lines together"
3. Verify paragraph moves to next page rather than splitting

**Expected:** Paragraph not split across pages
- [ ] Pass

### TC-3.8.3: Page Break Before
**Steps:**
1. Type paragraphs
2. Set "Page break before" on paragraph 3
3. Verify paragraph 3 starts on new page

**Expected:** Forced page break before paragraph
- [ ] Pass

### TC-3.8.4: Widow/Orphan Control
**Steps:**
1. Create paragraph that would split leaving one line
2. Enable widow/orphan control
3. Verify at least 2 lines on each page

**Expected:** No single lines at page top/bottom
- [ ] Pass

---

## 3.9 Tab Stops

### TC-3.9.1: Default Tabs
**Steps:**
1. Type text
2. Press Tab
3. Verify cursor moves to next default tab stop (0.5")

**Expected:** Tab moves to default positions
- [ ] Pass

### TC-3.9.2: Custom Left Tab
**Steps:**
1. Set left tab at 2"
2. Type "Label", Tab, "Value"
3. Verify "Value" starts at 2"

**Expected:** Custom tab stop works
- [ ] Pass

### TC-3.9.3: Center Tab
**Steps:**
1. Set center tab at 3"
2. Tab and type "Centered"
3. Verify text centered at 3" position

**Expected:** Text centered on tab stop
- [ ] Pass

### TC-3.9.4: Right Tab
**Steps:**
1. Set right tab at 5"
2. Tab and type "Price: $100"
3. Verify text right-aligned at 5"

**Expected:** Text right-aligned at tab stop
- [ ] Pass

### TC-3.9.5: Decimal Tab
**Steps:**
1. Set decimal tab at 3"
2. Type several numbers with decimals:
   - 123.45
   - 1.5
   - 99.999
3. Verify decimals align

**Expected:** Decimal points align vertically
- [ ] Pass

### TC-3.9.6: Tab with Leader
**Steps:**
1. Set right tab with dot leader at 5"
2. Type "Chapter 1", Tab, "Page 1"
3. Verify dots fill space between text and page number

**Expected:** Leader characters fill tab space
- [ ] Pass

---

## Summary

| Section | Tests | Passed | Failed |
|---------|-------|--------|--------|
| 3.1 Paragraph Alignment | 4 | | |
| 3.2 Indentation | 5 | | |
| 3.3 Line Spacing | 4 | | |
| 3.4 Paragraph Spacing | 3 | | |
| 3.5 Styles | 6 | | |
| 3.6 Modify Styles | 3 | | |
| 3.7 Borders and Shading | 3 | | |
| 3.8 Keep Options | 4 | | |
| 3.9 Tab Stops | 6 | | |
| **Total** | **38** | | |
