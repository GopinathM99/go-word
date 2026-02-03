# Test Group 5: Tables

## Overview
Tests for table creation, editing, formatting, and advanced table features.

---

## 5.1 Table Creation

### TC-5.1.1: Insert Table via Grid
**Steps:**
1. Insert > Table > Select 3x3 grid
2. Verify 3 columns x 3 rows table created

**Expected:** Table with specified dimensions
- [ ] Pass

### TC-5.1.2: Insert Table via Dialog
**Steps:**
1. Insert > Table > Insert Table dialog
2. Enter 5 columns, 4 rows
3. Click OK
4. Verify 5x4 table created

**Expected:** Table from dialog specifications
- [ ] Pass

### TC-5.1.3: Draw Table
**Steps:**
1. Select Draw Table tool
2. Draw rectangle for table boundary
3. Draw lines to create cells
4. Verify custom table structure

**Expected:** Hand-drawn table created
- [ ] Pass (if supported)
- [ ] N/A (if not supported)

### TC-5.1.4: Quick Tables/Templates
**Steps:**
1. Insert > Table > Quick Tables
2. Select a template (Calendar, Matrix, etc.)
3. Verify template table inserted

**Expected:** Pre-formatted table template
- [ ] Pass

---

## 5.2 Table Navigation

### TC-5.2.1: Tab Between Cells
**Steps:**
1. Create 3x3 table
2. Click in first cell
3. Press Tab repeatedly
4. Verify cursor moves through cells left-to-right, top-to-bottom

**Expected:** Tab navigates cells in reading order
- [ ] Pass

### TC-5.2.2: Shift+Tab Backwards
**Steps:**
1. Click in last cell
2. Press Shift+Tab
3. Verify cursor moves backwards through cells

**Expected:** Reverse navigation
- [ ] Pass

### TC-5.2.3: Arrow Keys in Table
**Steps:**
1. Create table with text in cells
2. Use arrow keys within cell text
3. Verify arrows move within text, not between cells

**Expected:** Arrows navigate text, not cells
- [ ] Pass

### TC-5.2.4: Tab Creates New Row
**Steps:**
1. Click in last cell of table
2. Press Tab
3. Verify new row added below

**Expected:** Tab in last cell adds row
- [ ] Pass

---

## 5.3 Cell Content

### TC-5.3.1: Type in Cell
**Steps:**
1. Click in cell
2. Type "Hello World"
3. Verify text appears in cell

**Expected:** Text entered in cell
- [ ] Pass

### TC-5.3.2: Multi-line Cell Content
**Steps:**
1. Click in cell
2. Type text, press Enter
3. Type more text
4. Verify multiple lines in single cell

**Expected:** Cell expands to fit content
- [ ] Pass

### TC-5.3.3: Format Cell Text
**Steps:**
1. Type text in cell
2. Select text
3. Apply bold, color, size
4. Verify formatting applies

**Expected:** Text formatting works in cells
- [ ] Pass

### TC-5.3.4: Image in Cell
**Steps:**
1. Click in cell
2. Insert image
3. Verify image fits in cell (or cell expands)

**Expected:** Image contained in cell
- [ ] Pass

### TC-5.3.5: Nested List in Cell
**Steps:**
1. Click in cell
2. Create bulleted list
3. Verify list contained in cell

**Expected:** List works inside table cell
- [ ] Pass

---

## 5.4 Row and Column Operations

### TC-5.4.1: Insert Row Above
**Steps:**
1. Create 3x3 table
2. Click in row 2
3. Insert > Row Above
4. Verify new row inserted above row 2

**Expected:** Row added above selection
- [ ] Pass

### TC-5.4.2: Insert Row Below
**Steps:**
1. Click in a row
2. Insert > Row Below
3. Verify new row added below

**Expected:** Row added below selection
- [ ] Pass

### TC-5.4.3: Insert Column Left
**Steps:**
1. Click in a column
2. Insert > Column Left
3. Verify new column added to left

**Expected:** Column added to left
- [ ] Pass

### TC-5.4.4: Insert Column Right
**Steps:**
1. Click in a column
2. Insert > Column Right
3. Verify new column added to right

**Expected:** Column added to right
- [ ] Pass

### TC-5.4.5: Delete Row
**Steps:**
1. Click in a row
2. Delete > Row
3. Verify row removed

**Expected:** Row deleted
- [ ] Pass

### TC-5.4.6: Delete Column
**Steps:**
1. Click in a column
2. Delete > Column
3. Verify column removed

**Expected:** Column deleted
- [ ] Pass

### TC-5.4.7: Delete Entire Table
**Steps:**
1. Select entire table
2. Delete > Table
3. Verify table removed

**Expected:** Entire table deleted
- [ ] Pass

---

## 5.5 Cell Selection

### TC-5.5.1: Select Single Cell
**Steps:**
1. Click inside cell near left edge
2. Verify entire cell selected (highlighted)

**Expected:** Single cell selected
- [ ] Pass

### TC-5.5.2: Select Row
**Steps:**
1. Click to left of table row
2. Verify entire row selected

**Expected:** Full row selected
- [ ] Pass

### TC-5.5.3: Select Column
**Steps:**
1. Click at top of column
2. Verify entire column selected

**Expected:** Full column selected
- [ ] Pass

### TC-5.5.4: Select Multiple Cells
**Steps:**
1. Click and drag across multiple cells
2. Verify rectangular selection

**Expected:** Multiple cells selected
- [ ] Pass

### TC-5.5.5: Select Entire Table
**Steps:**
1. Click table selector (top-left corner icon)
2. Verify entire table selected

**Expected:** All cells selected
- [ ] Pass

---

## 5.6 Merge and Split

### TC-5.6.1: Merge Cells Horizontally
**Steps:**
1. Select 3 cells in same row
2. Merge Cells
3. Verify cells combine into one

**Expected:** Horizontal merge creates single wide cell
- [ ] Pass

### TC-5.6.2: Merge Cells Vertically
**Steps:**
1. Select 3 cells in same column
2. Merge Cells
3. Verify cells combine into one tall cell

**Expected:** Vertical merge creates single tall cell
- [ ] Pass

### TC-5.6.3: Merge Rectangular Selection
**Steps:**
1. Select 2x2 cells
2. Merge
3. Verify 4 cells become 1

**Expected:** Rectangular merge works
- [ ] Pass

### TC-5.6.4: Split Cell Horizontally
**Steps:**
1. Click in a cell
2. Split Cell > 3 columns
3. Verify cell divided into 3 columns

**Expected:** Cell splits into columns
- [ ] Pass

### TC-5.6.5: Split Cell Vertically
**Steps:**
1. Click in a cell
2. Split Cell > 2 rows
3. Verify cell divided into 2 rows

**Expected:** Cell splits into rows
- [ ] Pass

---

## 5.7 Table Sizing

### TC-5.7.1: Resize Column Width (Drag)
**Steps:**
1. Hover on column border
2. Drag to resize
3. Verify column width changes

**Expected:** Manual column resize
- [ ] Pass

### TC-5.7.2: Resize Row Height (Drag)
**Steps:**
1. Hover on row border
2. Drag to resize
3. Verify row height changes

**Expected:** Manual row resize
- [ ] Pass

### TC-5.7.3: Auto-Fit to Contents
**Steps:**
1. Create table with varying content
2. Auto-fit > Contents
3. Verify columns shrink/expand to fit content

**Expected:** Columns sized to content
- [ ] Pass

### TC-5.7.4: Auto-Fit to Window
**Steps:**
1. Create table
2. Auto-fit > Window
3. Verify table spans page width

**Expected:** Table fills available width
- [ ] Pass

### TC-5.7.5: Fixed Column Width
**Steps:**
1. Set specific column width (e.g., 2")
2. Verify column doesn't change when typing

**Expected:** Column width stays fixed
- [ ] Pass

### TC-5.7.6: Distribute Columns Evenly
**Steps:**
1. Create table with uneven columns
2. Select columns
3. Distribute Columns
4. Verify all columns same width

**Expected:** Equal column widths
- [ ] Pass

### TC-5.7.7: Distribute Rows Evenly
**Steps:**
1. Create table with uneven rows
2. Select rows
3. Distribute Rows
4. Verify all rows same height

**Expected:** Equal row heights
- [ ] Pass

---

## 5.8 Table Formatting

### TC-5.8.1: Cell Borders
**Steps:**
1. Select cells
2. Apply thick border
3. Verify border style changes

**Expected:** Custom cell borders
- [ ] Pass

### TC-5.8.2: Cell Shading
**Steps:**
1. Select cells
2. Apply background color
3. Verify cells have color fill

**Expected:** Cell background color
- [ ] Pass

### TC-5.8.3: Table Style
**Steps:**
1. Select table
2. Apply built-in table style
3. Verify coordinated formatting (headers, stripes)

**Expected:** Table style applied
- [ ] Pass

### TC-5.8.4: Header Row
**Steps:**
1. Select first row
2. Mark as header row
3. Insert page break in middle of table
4. Verify header repeats on second page

**Expected:** Header row repeats on each page
- [ ] Pass

### TC-5.8.5: Cell Alignment
**Steps:**
1. Type text in cell
2. Apply vertical alignment: Top, Center, Bottom
3. Verify text position changes

**Expected:** Vertical text alignment works
- [ ] Pass

### TC-5.8.6: Cell Margins
**Steps:**
1. Select cell
2. Set cell margins/padding
3. Verify spacing around text

**Expected:** Cell internal margins work
- [ ] Pass

---

## 5.9 Advanced Table Features

### TC-5.9.1: Table Across Pages
**Steps:**
1. Create table with many rows
2. Verify table splits across pages correctly
3. Check rows don't break mid-content

**Expected:** Table pagination works
- [ ] Pass

### TC-5.9.2: Prevent Row Break
**Steps:**
1. Create row with lots of content
2. Set "Don't split row across pages"
3. Verify row moves to next page intact

**Expected:** Row stays together
- [ ] Pass

### TC-5.9.3: Nested Table
**Steps:**
1. Create table
2. Insert another table inside a cell
3. Verify nested table works

**Expected:** Table within table
- [ ] Pass

### TC-5.9.4: Sort Table
**Steps:**
1. Create table with data (names, numbers)
2. Select column
3. Sort ascending
4. Verify rows reorder by that column

**Expected:** Table sorting works
- [ ] Pass

### TC-5.9.5: Convert Table to Text
**Steps:**
1. Select table
2. Convert to text (with delimiter)
3. Verify table becomes text with tabs/commas

**Expected:** Table converted to plain text
- [ ] Pass

### TC-5.9.6: Convert Text to Table
**Steps:**
1. Type tab-separated data:
   ```
   Name    Age    City
   John    25     NYC
   Jane    30     LA
   ```
2. Select text
3. Convert to table
4. Verify structured table created

**Expected:** Text becomes table
- [ ] Pass

---

## Summary

| Section | Tests | Passed | Failed |
|---------|-------|--------|--------|
| 5.1 Table Creation | 4 | | |
| 5.2 Table Navigation | 4 | | |
| 5.3 Cell Content | 5 | | |
| 5.4 Row/Column Ops | 7 | | |
| 5.5 Cell Selection | 5 | | |
| 5.6 Merge and Split | 5 | | |
| 5.7 Table Sizing | 7 | | |
| 5.8 Table Formatting | 6 | | |
| 5.9 Advanced Features | 6 | | |
| **Total** | **49** | | |
