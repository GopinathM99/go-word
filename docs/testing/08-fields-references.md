# Test Group 8: Fields and References

## Overview
Tests for fields, table of contents, cross-references, bookmarks, footnotes, and endnotes.

---

## 8.1 Basic Fields

### TC-8.1.1: Insert Page Number Field
**Steps:**
1. Insert > Field > Page
2. Verify current page number displays

**Expected:** Page number shown
- [ ] Pass

### TC-8.1.2: Insert Date Field
**Steps:**
1. Insert > Field > Date
2. Verify current date displays
3. Close and reopen document
4. Verify date updates

**Expected:** Dynamic date field
- [ ] Pass

### TC-8.1.3: Insert Time Field
**Steps:**
1. Insert > Field > Time
2. Verify current time displays

**Expected:** Time field works
- [ ] Pass

### TC-8.1.4: Insert Author Field
**Steps:**
1. Insert > Field > Author
2. Verify author name from document properties

**Expected:** Author field shows document author
- [ ] Pass

### TC-8.1.5: Insert File Name Field
**Steps:**
1. Insert > Field > FileName
2. Verify document filename displayed

**Expected:** Filename field works
- [ ] Pass

### TC-8.1.6: Update Field
**Steps:**
1. Insert date field
2. Wait or change system date
3. Right-click > Update Field (or F9)
4. Verify field value updates

**Expected:** Field refresh works
- [ ] Pass

### TC-8.1.7: Toggle Field Code View
**Steps:**
1. Insert a field
2. Press Alt+F9 (toggle field codes)
3. Verify field code shown (e.g., { DATE })
4. Toggle back to see result

**Expected:** Field code visibility toggle
- [ ] Pass

---

## 8.2 Table of Contents

### TC-8.2.1: Insert Basic TOC
**Steps:**
1. Create document with Heading 1, 2, 3 styles
2. Insert > Table of Contents
3. Verify TOC generated with headings

**Expected:** TOC lists all headings
- [ ] Pass

### TC-8.2.2: TOC with Page Numbers
**Steps:**
1. Generate TOC
2. Verify page numbers appear
3. Navigate to heading locations

**Expected:** Page numbers accurate
- [ ] Pass

### TC-8.2.3: Update TOC
**Steps:**
1. Add new heading after TOC
2. Right-click TOC > Update Field
3. Choose "Update entire table"
4. Verify new heading appears

**Expected:** TOC updates with changes
- [ ] Pass

### TC-8.2.4: TOC Hyperlinks
**Steps:**
1. Generate TOC
2. Ctrl+Click on TOC entry
3. Verify navigation to heading

**Expected:** TOC entries are links
- [ ] Pass

### TC-8.2.5: Custom TOC Levels
**Steps:**
1. Insert TOC showing only Heading 1-2
2. Verify Heading 3 excluded

**Expected:** Customizable TOC depth
- [ ] Pass

### TC-8.2.6: TOC Formatting
**Steps:**
1. Modify TOC styles
2. Verify formatting changes apply

**Expected:** TOC style customizable
- [ ] Pass

---

## 8.3 Bookmarks

### TC-8.3.1: Insert Bookmark
**Steps:**
1. Select text "Important Section"
2. Insert > Bookmark
3. Name it "ImportantSection"
4. Verify bookmark created

**Expected:** Bookmark added
- [ ] Pass

### TC-8.3.2: Navigate to Bookmark
**Steps:**
1. Create bookmark
2. Move cursor elsewhere
3. Go To > Bookmark > Select bookmark
4. Verify cursor jumps to bookmarked location

**Expected:** Bookmark navigation works
- [ ] Pass

### TC-8.3.3: List Bookmarks
**Steps:**
1. Create multiple bookmarks
2. Open bookmark dialog
3. Verify all bookmarks listed

**Expected:** Bookmark list shows all
- [ ] Pass

### TC-8.3.4: Delete Bookmark
**Steps:**
1. Open bookmark dialog
2. Select bookmark
3. Delete
4. Verify bookmark removed

**Expected:** Bookmark deletion
- [ ] Pass

### TC-8.3.5: Bookmark Persists in File
**Steps:**
1. Create bookmark
2. Save, close, reopen document
3. Verify bookmark still exists

**Expected:** Bookmarks persist
- [ ] Pass

---

## 8.4 Cross-References

### TC-8.4.1: Cross-Reference to Heading
**Steps:**
1. Create heading "Chapter 1"
2. In body text, Insert > Cross-reference
3. Reference type: Heading
4. Select "Chapter 1"
5. Verify cross-reference inserted

**Expected:** Heading cross-reference
- [ ] Pass

### TC-8.4.2: Cross-Reference to Figure
**Steps:**
1. Insert image with caption "Figure 1: Chart"
2. Insert cross-reference to figure
3. Verify "Figure 1" inserted

**Expected:** Figure cross-reference
- [ ] Pass

### TC-8.4.3: Cross-Reference to Table
**Steps:**
1. Insert table with caption
2. Cross-reference the table
3. Verify reference works

**Expected:** Table cross-reference
- [ ] Pass

### TC-8.4.4: Cross-Reference to Bookmark
**Steps:**
1. Create bookmark
2. Insert cross-reference to bookmark
3. Verify reference text appears

**Expected:** Bookmark cross-reference
- [ ] Pass

### TC-8.4.5: Cross-Reference Page Number
**Steps:**
1. Cross-reference showing page number
2. Verify "see page X" works

**Expected:** Page number reference
- [ ] Pass

### TC-8.4.6: Update Cross-References
**Steps:**
1. Create cross-reference
2. Change the referenced heading text
3. Update fields
4. Verify cross-reference updates

**Expected:** Cross-refs update with source
- [ ] Pass

### TC-8.4.7: Cross-Reference as Hyperlink
**Steps:**
1. Create cross-reference as hyperlink
2. Ctrl+Click
3. Verify jumps to referenced item

**Expected:** Clickable cross-reference
- [ ] Pass

---

## 8.5 Footnotes

### TC-8.5.1: Insert Footnote
**Steps:**
1. Place cursor after word
2. Insert > Footnote
3. Type footnote text
4. Verify superscript number and footnote at page bottom

**Expected:** Footnote created
- [ ] Pass

### TC-8.5.2: Navigate to Footnote
**Steps:**
1. Double-click footnote reference
2. Verify cursor jumps to footnote text
3. Double-click footnote number
4. Verify returns to reference

**Expected:** Footnote navigation
- [ ] Pass

### TC-8.5.3: Multiple Footnotes
**Steps:**
1. Insert 3 footnotes on same page
2. Verify sequential numbering (1, 2, 3)

**Expected:** Auto-numbering
- [ ] Pass

### TC-8.5.4: Footnote Across Pages
**Steps:**
1. Insert footnote near page bottom
2. Add long footnote text
3. Verify footnote continues on next page

**Expected:** Footnote continuation
- [ ] Pass

### TC-8.5.5: Delete Footnote
**Steps:**
1. Delete the footnote reference number in text
2. Verify footnote text also removed
3. Verify remaining footnotes renumber

**Expected:** Footnote deletion and renumbering
- [ ] Pass

### TC-8.5.6: Footnote Formatting
**Steps:**
1. Change footnote number format (i, ii, iii)
2. Verify format changes

**Expected:** Custom footnote numbering
- [ ] Pass

---

## 8.6 Endnotes

### TC-8.6.1: Insert Endnote
**Steps:**
1. Insert > Endnote
2. Type endnote text
3. Verify endnote appears at document end

**Expected:** Endnote at document end
- [ ] Pass

### TC-8.6.2: Convert Footnote to Endnote
**Steps:**
1. Right-click footnote
2. Convert to Endnote
3. Verify moves to end of document

**Expected:** Footnote conversion
- [ ] Pass

### TC-8.6.3: Endnote at Section End
**Steps:**
1. Configure endnotes at end of section
2. Create sections
3. Verify endnotes appear at section breaks

**Expected:** Section-end endnotes
- [ ] Pass

---

## 8.7 Captions

### TC-8.7.1: Add Caption to Image
**Steps:**
1. Insert image
2. Right-click > Insert Caption
3. Verify "Figure 1" caption added

**Expected:** Image caption
- [ ] Pass

### TC-8.7.2: Add Caption to Table
**Steps:**
1. Select table
2. Insert caption
3. Verify "Table 1" caption added

**Expected:** Table caption
- [ ] Pass

### TC-8.7.3: Caption Numbering
**Steps:**
1. Add multiple figures with captions
2. Verify auto-numbering (Figure 1, 2, 3)

**Expected:** Sequential caption numbers
- [ ] Pass

### TC-8.7.4: Custom Caption Label
**Steps:**
1. Create new label "Chart"
2. Add caption with new label
3. Verify "Chart 1" appears

**Expected:** Custom caption labels
- [ ] Pass

### TC-8.7.5: Insert Table of Figures
**Steps:**
1. Add several captioned figures
2. Insert > Table of Figures
3. Verify list generated

**Expected:** Table of figures generated
- [ ] Pass

---

## 8.8 Index

### TC-8.8.1: Mark Index Entry
**Steps:**
1. Select word "Algorithm"
2. Mark as index entry (Alt+Shift+X)
3. Verify entry marked

**Expected:** Index entry created
- [ ] Pass

### TC-8.8.2: Insert Index
**Steps:**
1. Mark several index entries
2. Insert > Index
3. Verify alphabetized index generated

**Expected:** Index generated
- [ ] Pass

### TC-8.8.3: Index with Subentries
**Steps:**
1. Mark entry with subentry
2. Generate index
3. Verify nested entries appear

**Expected:** Hierarchical index
- [ ] Pass

### TC-8.8.4: Update Index
**Steps:**
1. Add new index entries
2. Update index
3. Verify new entries included

**Expected:** Index updates
- [ ] Pass

---

## Summary

| Section | Tests | Passed | Failed |
|---------|-------|--------|--------|
| 8.1 Basic Fields | 7 | | |
| 8.2 Table of Contents | 6 | | |
| 8.3 Bookmarks | 5 | | |
| 8.4 Cross-References | 7 | | |
| 8.5 Footnotes | 6 | | |
| 8.6 Endnotes | 3 | | |
| 8.7 Captions | 5 | | |
| 8.8 Index | 4 | | |
| **Total** | **43** | | |
