# Test Group 7: Document Structure

## Overview
Tests for sections, headers/footers, page setup, columns, and document organization.

---

## 7.1 Page Setup

### TC-7.1.1: Change Page Size
**Steps:**
1. Open Page Setup (Layout > Size)
2. Select A4
3. Verify page dimensions change

**Expected:** Page size updated
- [ ] Pass

### TC-7.1.2: Custom Page Size
**Steps:**
1. Open Page Setup
2. Enter custom dimensions (8" x 10")
3. Apply
4. Verify custom size used

**Expected:** Custom page size works
- [ ] Pass

### TC-7.1.3: Change Orientation
**Steps:**
1. Document in Portrait
2. Change to Landscape
3. Verify page is wider than tall

**Expected:** Orientation switches
- [ ] Pass

### TC-7.1.4: Set Margins
**Steps:**
1. Open Page Setup
2. Set all margins to 1.5"
3. Verify text area shrinks

**Expected:** Margins applied
- [ ] Pass

### TC-7.1.5: Mirror Margins
**Steps:**
1. Enable mirror margins
2. Verify Inside/Outside margins for book layout

**Expected:** Mirror margins for binding
- [ ] Pass

### TC-7.1.6: Gutter Margin
**Steps:**
1. Set gutter margin (e.g., 0.5")
2. Verify extra space on binding edge

**Expected:** Gutter space added
- [ ] Pass

---

## 7.2 Page Breaks

### TC-7.2.1: Insert Page Break
**Steps:**
1. Type text
2. Insert > Page Break (or Ctrl+Enter)
3. Verify content after break on new page

**Expected:** Manual page break inserted
- [ ] Pass

### TC-7.2.2: Delete Page Break
**Steps:**
1. Insert page break
2. Position cursor at break
3. Press Delete
4. Verify break removed

**Expected:** Page break deleted
- [ ] Pass

### TC-7.2.3: Automatic Page Break
**Steps:**
1. Type enough text to fill a page
2. Verify text automatically flows to page 2

**Expected:** Auto pagination works
- [ ] Pass

---

## 7.3 Sections

### TC-7.3.1: Insert Section Break - Next Page
**Steps:**
1. Type "Section 1 content"
2. Insert Section Break > Next Page
3. Type "Section 2 content"
4. Verify content starts on new page

**Expected:** Section break creates new page
- [ ] Pass

### TC-7.3.2: Insert Section Break - Continuous
**Steps:**
1. Insert Section Break > Continuous
2. Verify section changes without page break

**Expected:** Section break on same page
- [ ] Pass

### TC-7.3.3: Different Page Setup per Section
**Steps:**
1. Create 2 sections
2. Set Section 1 to Portrait
3. Set Section 2 to Landscape
4. Verify different orientations

**Expected:** Per-section page setup
- [ ] Pass

### TC-7.3.4: Different Margins per Section
**Steps:**
1. Create 2 sections
2. Set different margins in each
3. Verify different margins applied

**Expected:** Per-section margins
- [ ] Pass

### TC-7.3.5: Delete Section Break
**Steps:**
1. Insert section break
2. Show formatting marks
3. Delete section break
4. Verify sections merge

**Expected:** Section break removed
- [ ] Pass

---

## 7.4 Headers and Footers

### TC-7.4.1: Add Header
**Steps:**
1. Double-click header area (or Insert > Header)
2. Type "Document Title"
3. Click outside header
4. Verify header appears on all pages

**Expected:** Header added
- [ ] Pass

### TC-7.4.2: Add Footer
**Steps:**
1. Double-click footer area
2. Type "Confidential"
3. Exit footer
4. Verify footer on all pages

**Expected:** Footer added
- [ ] Pass

### TC-7.4.3: Different First Page
**Steps:**
1. Enable "Different First Page"
2. Set different header for page 1
3. Verify page 1 has different header

**Expected:** First page header different
- [ ] Pass

### TC-7.4.4: Different Odd/Even Pages
**Steps:**
1. Enable "Different Odd & Even"
2. Set "Left Page" header
3. Set "Right Page" header
4. Verify alternating headers

**Expected:** Alternating headers
- [ ] Pass

### TC-7.4.5: Different Header per Section
**Steps:**
1. Create 2 sections
2. Unlink Section 2 from Section 1
3. Set different header in Section 2
4. Verify independent headers

**Expected:** Per-section headers
- [ ] Pass

### TC-7.4.6: Page Numbers in Footer
**Steps:**
1. Open footer
2. Insert > Page Number
3. Verify page numbers appear

**Expected:** Page numbers in footer
- [ ] Pass

### TC-7.4.7: Page X of Y
**Steps:**
1. Insert page number with "Page X of Y" format
2. Verify displays like "Page 1 of 5"

**Expected:** Total pages shown
- [ ] Pass

---

## 7.5 Columns

### TC-7.5.1: Two Columns
**Steps:**
1. Select text
2. Layout > Columns > Two
3. Verify text flows in 2 columns

**Expected:** Two-column layout
- [ ] Pass

### TC-7.5.2: Three Columns
**Steps:**
1. Apply three-column layout
2. Verify balanced columns

**Expected:** Three-column layout
- [ ] Pass

### TC-7.5.3: Column Break
**Steps:**
1. In two-column layout
2. Insert column break
3. Verify text jumps to next column

**Expected:** Manual column break
- [ ] Pass

### TC-7.5.4: Unequal Columns
**Steps:**
1. Set left column wider than right
2. Verify unequal widths

**Expected:** Asymmetric columns
- [ ] Pass

### TC-7.5.5: Column with Line Between
**Steps:**
1. Create columns
2. Enable "Line between"
3. Verify vertical line between columns

**Expected:** Column separator line
- [ ] Pass

### TC-7.5.6: Column Spacing
**Steps:**
1. Set column spacing to 0.75"
2. Verify gap between columns increases

**Expected:** Column gap adjustable
- [ ] Pass

---

## 7.6 Line Numbers

### TC-7.6.1: Enable Line Numbers
**Steps:**
1. Layout > Line Numbers > Continuous
2. Verify numbers appear in margin

**Expected:** Line numbers shown
- [ ] Pass

### TC-7.6.2: Restart Each Page
**Steps:**
1. Set line numbers to restart each page
2. Verify page 2 starts at 1

**Expected:** Per-page line numbering
- [ ] Pass

### TC-7.6.3: Restart Each Section
**Steps:**
1. Create sections
2. Set line numbers to restart per section
3. Verify numbering restarts at section breaks

**Expected:** Per-section line numbering
- [ ] Pass

---

## 7.7 Watermarks

### TC-7.7.1: Text Watermark
**Steps:**
1. Insert > Watermark > Custom
2. Enter "DRAFT" as text
3. Verify watermark appears diagonally

**Expected:** Text watermark on all pages
- [ ] Pass

### TC-7.7.2: Image Watermark
**Steps:**
1. Insert > Watermark > Picture
2. Select image
3. Verify image appears faded behind text

**Expected:** Picture watermark
- [ ] Pass

### TC-7.7.3: Remove Watermark
**Steps:**
1. Apply watermark
2. Remove watermark
3. Verify watermark gone

**Expected:** Watermark removed
- [ ] Pass

---

## 7.8 Cover Page

### TC-7.8.1: Insert Cover Page
**Steps:**
1. Insert > Cover Page
2. Select template
3. Verify cover page inserted at start

**Expected:** Cover page template inserted
- [ ] Pass

### TC-7.8.2: Fill Cover Page Fields
**Steps:**
1. Insert cover page with placeholders
2. Click and fill in title, author, date
3. Verify fields populate

**Expected:** Cover page fields editable
- [ ] Pass

---

## 7.9 Blank Page

### TC-7.9.1: Insert Blank Page
**Steps:**
1. Insert > Blank Page
2. Verify new blank page inserted

**Expected:** Empty page added
- [ ] Pass

### TC-7.9.2: Delete Blank Page
**Steps:**
1. Navigate to blank page
2. Select and delete content/breaks
3. Verify blank page removed

**Expected:** Blank page deleted
- [ ] Pass

---

## Summary

| Section | Tests | Passed | Failed |
|---------|-------|--------|--------|
| 7.1 Page Setup | 6 | | |
| 7.2 Page Breaks | 3 | | |
| 7.3 Sections | 5 | | |
| 7.4 Headers/Footers | 7 | | |
| 7.5 Columns | 6 | | |
| 7.6 Line Numbers | 3 | | |
| 7.7 Watermarks | 3 | | |
| 7.8 Cover Page | 2 | | |
| 7.9 Blank Page | 2 | | |
| **Total** | **37** | | |
