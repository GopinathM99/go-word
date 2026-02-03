# Test Group 10: Import and Export

## Overview
Tests for opening, saving, and converting documents in various formats.

---

## 10.1 DOCX Format

### TC-10.1.1: Save as DOCX
**Steps:**
1. Create document with text, formatting, images
2. Save as .docx
3. Close and reopen
4. Verify all content preserved

**Expected:** DOCX save/load roundtrip
- [ ] Pass

### TC-10.1.2: Open External DOCX
**Steps:**
1. Obtain DOCX created in Microsoft Word
2. Open in application
3. Verify content displays correctly

**Expected:** MS Word DOCX compatibility
- [ ] Pass

### TC-10.1.3: Complex DOCX - Tables
**Steps:**
1. Open DOCX with complex tables (merged cells, borders)
2. Verify table renders correctly

**Expected:** Table import fidelity
- [ ] Pass

### TC-10.1.4: Complex DOCX - Styles
**Steps:**
1. Open DOCX with custom styles
2. Verify styles imported and applied

**Expected:** Style import
- [ ] Pass

### TC-10.1.5: Complex DOCX - Track Changes
**Steps:**
1. Open DOCX with tracked changes
2. Verify revisions display correctly

**Expected:** Track changes import
- [ ] Pass

### TC-10.1.6: Complex DOCX - Comments
**Steps:**
1. Open DOCX with comments
2. Verify comments display in margin

**Expected:** Comment import
- [ ] Pass

### TC-10.1.7: DOCX with Embedded Objects
**Steps:**
1. Open DOCX with embedded Excel chart
2. Verify object displays (may be image)

**Expected:** Embedded object handling
- [ ] Pass

---

## 10.2 PDF Export

### TC-10.2.1: Export to PDF
**Steps:**
1. Create document
2. Export/Save as PDF
3. Open PDF in viewer
4. Verify content matches

**Expected:** Basic PDF export
- [ ] Pass

### TC-10.2.2: PDF Text Selection
**Steps:**
1. Export to PDF
2. Open PDF
3. Try to select and copy text
4. Verify text is selectable (not image)

**Expected:** Searchable/selectable PDF
- [ ] Pass

### TC-10.2.3: PDF Images
**Steps:**
1. Create document with images
2. Export to PDF
3. Verify images appear in PDF

**Expected:** Images in PDF
- [ ] Pass

### TC-10.2.4: PDF Tables
**Steps:**
1. Create document with table
2. Export to PDF
3. Verify table layout preserved

**Expected:** Table PDF export
- [ ] Pass

### TC-10.2.5: PDF Links
**Steps:**
1. Create document with hyperlinks
2. Export to PDF
3. Click link in PDF
4. Verify link works

**Expected:** Hyperlinks in PDF
- [ ] Pass

### TC-10.2.6: PDF/A Export
**Steps:**
1. Export as PDF/A (archival format)
2. Verify PDF/A compliance (validator or reader)

**Expected:** PDF/A archival format
- [ ] Pass

### TC-10.2.7: PDF Page Layout
**Steps:**
1. Create multi-page document
2. Export to PDF
3. Verify page breaks match

**Expected:** Pagination preserved
- [ ] Pass

---

## 10.3 RTF Format

### TC-10.3.1: Save as RTF
**Steps:**
1. Create formatted document
2. Save as .rtf
3. Reopen
4. Verify formatting preserved

**Expected:** RTF save works
- [ ] Pass

### TC-10.3.2: Open External RTF
**Steps:**
1. Open RTF from another application
2. Verify content loads

**Expected:** RTF import
- [ ] Pass

### TC-10.3.3: RTF Formatting Fidelity
**Steps:**
1. Create document with bold, italic, colors
2. Save as RTF
3. Open in WordPad or TextEdit
4. Verify formatting preserved

**Expected:** Cross-app RTF compatibility
- [ ] Pass

---

## 10.4 ODT Format

### TC-10.4.1: Open ODT
**Steps:**
1. Obtain ODT file from LibreOffice
2. Open in application
3. Verify content displays

**Expected:** ODT import
- [ ] Pass

### TC-10.4.2: ODT Formatting
**Steps:**
1. Open ODT with styles and formatting
2. Verify formatting imported

**Expected:** ODT formatting import
- [ ] Pass

### TC-10.4.3: Save as ODT (if supported)
**Steps:**
1. Create document
2. Save as ODT
3. Open in LibreOffice
4. Verify content preserved

**Expected:** ODT export
- [ ] Pass (if supported)
- [ ] N/A

---

## 10.5 Plain Text

### TC-10.5.1: Save as Plain Text
**Steps:**
1. Create formatted document
2. Save as .txt
3. Open in text editor
4. Verify text content (formatting stripped)

**Expected:** Text export
- [ ] Pass

### TC-10.5.2: Open Plain Text
**Steps:**
1. Open .txt file
2. Verify content loads as plain text

**Expected:** Text import
- [ ] Pass

### TC-10.5.3: Plain Text Encoding
**Steps:**
1. Create text with special characters
2. Save as UTF-8 text
3. Verify encoding preserved

**Expected:** UTF-8 encoding
- [ ] Pass

---

## 10.6 HTML Export

### TC-10.6.1: Save as HTML
**Steps:**
1. Create formatted document
2. Save as HTML
3. Open in browser
4. Verify rendering

**Expected:** HTML export
- [ ] Pass

### TC-10.6.2: HTML with Images
**Steps:**
1. Document with embedded images
2. Save as HTML
3. Verify images in separate folder or embedded

**Expected:** HTML image handling
- [ ] Pass

### TC-10.6.3: Open HTML
**Steps:**
1. Open HTML file
2. Verify content imports

**Expected:** HTML import
- [ ] Pass

---

## 10.7 Clipboard Operations

### TC-10.7.1: Paste from Word
**Steps:**
1. Copy formatted text from MS Word
2. Paste into application
3. Verify formatting preserved

**Expected:** Rich paste from Word
- [ ] Pass

### TC-10.7.2: Paste from Web Browser
**Steps:**
1. Copy text with links from browser
2. Paste
3. Verify text and possibly links imported

**Expected:** Web content paste
- [ ] Pass

### TC-10.7.3: Paste Plain Text
**Steps:**
1. Copy formatted text
2. Paste Special > Plain Text (Ctrl+Shift+V)
3. Verify only text, no formatting

**Expected:** Plain text paste option
- [ ] Pass

### TC-10.7.4: Paste from Excel
**Steps:**
1. Copy cells from Excel
2. Paste into document
3. Verify creates table

**Expected:** Excel to table conversion
- [ ] Pass

---

## 10.8 Drag and Drop

### TC-10.8.1: Drop Image File
**Steps:**
1. Drag image file from explorer
2. Drop into document
3. Verify image inserted

**Expected:** Image drop
- [ ] Pass

### TC-10.8.2: Drop Text File
**Steps:**
1. Drag .txt file into document
2. Verify text content inserted or linked

**Expected:** Text file drop
- [ ] Pass

### TC-10.8.3: Drop DOCX File
**Steps:**
1. Drag .docx file into document
2. Verify content inserted or opened

**Expected:** Document drop behavior
- [ ] Pass

---

## 10.9 Recent Files and Recovery

### TC-10.9.1: Recent Files List
**Steps:**
1. Open several documents
2. Check File > Recent
3. Verify recently opened files listed

**Expected:** Recent files tracked
- [ ] Pass

### TC-10.9.2: Auto-Save
**Steps:**
1. Enable auto-save
2. Make changes
3. Wait for auto-save interval
4. Check for recovery file

**Expected:** Auto-save creates recovery
- [ ] Pass

### TC-10.9.3: Recover Unsaved Document
**Steps:**
1. Create document without saving
2. Simulate crash (if possible) or close without saving
3. Reopen application
4. Check for recovery option

**Expected:** Document recovery prompt
- [ ] Pass

### TC-10.9.4: Version History
**Steps:**
1. Make changes over time (with auto-save)
2. Access version history
3. View or restore previous version

**Expected:** Version history available
- [ ] Pass

---

## 10.10 Export Options

### TC-10.10.1: Export Range (Current Page)
**Steps:**
1. Multi-page document
2. Export to PDF > Current page only
3. Verify single page PDF

**Expected:** Page range export
- [ ] Pass

### TC-10.10.2: Export Range (Selection)
**Steps:**
1. Select portion of document
2. Export selection only
3. Verify only selection in output

**Expected:** Selection export
- [ ] Pass

### TC-10.10.3: Export with Track Changes
**Steps:**
1. Document with tracked changes
2. Export showing markup
3. Verify markup visible in output

**Expected:** Export with markup
- [ ] Pass

### TC-10.10.4: Export Final (Accept All)
**Steps:**
1. Document with tracked changes
2. Export as final (changes accepted)
3. Verify clean document without markup

**Expected:** Export clean version
- [ ] Pass

---

## Summary

| Section | Tests | Passed | Failed |
|---------|-------|--------|--------|
| 10.1 DOCX Format | 7 | | |
| 10.2 PDF Export | 7 | | |
| 10.3 RTF Format | 3 | | |
| 10.4 ODT Format | 3 | | |
| 10.5 Plain Text | 3 | | |
| 10.6 HTML Export | 3 | | |
| 10.7 Clipboard | 4 | | |
| 10.8 Drag and Drop | 3 | | |
| 10.9 Recent/Recovery | 4 | | |
| 10.10 Export Options | 4 | | |
| **Total** | **41** | | |
