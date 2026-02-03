# Test Group 12: Advanced Features

## Overview
Tests for equations, charts, mail merge, content controls, and other advanced functionality.

---

## 12.1 Equations

### TC-12.1.1: Insert Equation
**Steps:**
1. Insert > Equation
2. Verify equation editor opens
3. Type simple equation: x + y = z

**Expected:** Equation inserted
- [ ] Pass

### TC-12.1.2: Fraction
**Steps:**
1. Insert equation
2. Create fraction a/b
3. Verify fraction renders stacked

**Expected:** Fraction display
- [ ] Pass

### TC-12.1.3: Square Root
**Steps:**
1. Insert square root symbol
2. Enter content under radical
3. Verify renders correctly

**Expected:** Radical/root
- [ ] Pass

### TC-12.1.4: Superscript/Subscript in Equation
**Steps:**
1. Type x^2 (superscript)
2. Type a_n (subscript)
3. Verify positioning

**Expected:** Script positioning
- [ ] Pass

### TC-12.1.5: Greek Letters
**Steps:**
1. Insert alpha, beta, gamma, pi
2. Verify Greek symbols appear

**Expected:** Greek letter support
- [ ] Pass

### TC-12.1.6: Matrix
**Steps:**
1. Insert matrix structure
2. Fill in values
3. Verify matrix layout

**Expected:** Matrix editing
- [ ] Pass

### TC-12.1.7: Integral/Summation
**Steps:**
1. Insert integral symbol with limits
2. Insert summation with limits
3. Verify limits positioned correctly

**Expected:** Large operators
- [ ] Pass

### TC-12.1.8: Equation Numbering
**Steps:**
1. Insert equation
2. Add equation number
3. Verify "(1)" appears

**Expected:** Equation numbering
- [ ] Pass

---

## 12.2 Charts

### TC-12.2.1: Insert Chart
**Steps:**
1. Insert > Chart
2. Select chart type (e.g., Bar)
3. Verify chart inserted with sample data

**Expected:** Chart created
- [ ] Pass

### TC-12.2.2: Edit Chart Data
**Steps:**
1. Select chart
2. Edit data values
3. Verify chart updates

**Expected:** Data editing
- [ ] Pass

### TC-12.2.3: Change Chart Type
**Steps:**
1. Select chart
2. Change from Bar to Line
3. Verify chart type changes

**Expected:** Chart type switching
- [ ] Pass

### TC-12.2.4: Chart Colors/Style
**Steps:**
1. Select chart
2. Apply different color scheme
3. Verify colors change

**Expected:** Chart styling
- [ ] Pass

### TC-12.2.5: Chart Title
**Steps:**
1. Add chart title
2. Edit title text
3. Verify title displays

**Expected:** Chart title
- [ ] Pass

### TC-12.2.6: Chart Legend
**Steps:**
1. Show/hide legend
2. Move legend position
3. Verify legend updates

**Expected:** Legend control
- [ ] Pass

### TC-12.2.7: Resize Chart
**Steps:**
1. Drag chart handles
2. Verify chart resizes

**Expected:** Chart resizing
- [ ] Pass

---

## 12.3 Content Controls

### TC-12.3.1: Plain Text Control
**Steps:**
1. Insert plain text content control
2. Type in control
3. Verify text confined to control

**Expected:** Text control
- [ ] Pass

### TC-12.3.2: Rich Text Control
**Steps:**
1. Insert rich text control
2. Apply formatting inside
3. Verify formatting preserved

**Expected:** Rich text control
- [ ] Pass

### TC-12.3.3: Checkbox Control
**Steps:**
1. Insert checkbox control
2. Click to toggle
3. Verify checked/unchecked states

**Expected:** Checkbox
- [ ] Pass

### TC-12.3.4: Dropdown List Control
**Steps:**
1. Insert dropdown control
2. Add list items
3. Select from dropdown
4. Verify selection

**Expected:** Dropdown control
- [ ] Pass

### TC-12.3.5: Date Picker Control
**Steps:**
1. Insert date picker
2. Click to open calendar
3. Select date
4. Verify date inserted

**Expected:** Date picker
- [ ] Pass

### TC-12.3.6: Picture Control
**Steps:**
1. Insert picture content control
2. Click to insert image
3. Verify image in control

**Expected:** Picture placeholder
- [ ] Pass

### TC-12.3.7: Locked Content Control
**Steps:**
1. Insert control
2. Lock control (prevent deletion)
3. Try to delete
4. Verify deletion blocked

**Expected:** Control locking
- [ ] Pass

### TC-12.3.8: Repeating Section
**Steps:**
1. Insert repeating section control
2. Add items
3. Verify section repeats

**Expected:** Repeating sections
- [ ] Pass

---

## 12.4 Mail Merge

### TC-12.4.1: Create Mail Merge Document
**Steps:**
1. Start mail merge wizard
2. Select document type (Letters)
3. Create template with merge fields

**Expected:** Mail merge setup
- [ ] Pass

### TC-12.4.2: Connect Data Source (CSV)
**Steps:**
1. Connect to CSV file
2. Verify fields available

**Expected:** CSV data source
- [ ] Pass

### TC-12.4.3: Insert Merge Field
**Steps:**
1. Insert merge field (e.g., <<FirstName>>)
2. Verify field placeholder shown

**Expected:** Merge field insertion
- [ ] Pass

### TC-12.4.4: Preview Merge
**Steps:**
1. Preview results
2. Navigate through records
3. Verify data substituted

**Expected:** Merge preview
- [ ] Pass

### TC-12.4.5: Finish Merge to New Document
**Steps:**
1. Complete merge
2. Output to new document
3. Verify merged letters created

**Expected:** Merge to document
- [ ] Pass

### TC-12.4.6: Finish Merge to Printer
**Steps:**
1. Complete merge
2. Send to printer
3. Verify print jobs created

**Expected:** Merge to printer
- [ ] Pass (if printer available)

### TC-12.4.7: Conditional Merge (IF field)
**Steps:**
1. Insert IF field based on data
2. Preview with different records
3. Verify conditional content

**Expected:** Conditional merge
- [ ] Pass

### TC-12.4.8: Address Block
**Steps:**
1. Insert Address Block
2. Map fields
3. Verify complete address formats

**Expected:** Address block
- [ ] Pass

---

## 12.5 Templates

### TC-12.5.1: Open Template
**Steps:**
1. File > New from Template
2. Select template
3. Verify new document created from template

**Expected:** Template usage
- [ ] Pass

### TC-12.5.2: Save as Template
**Steps:**
1. Create document with styles/formatting
2. Save as template
3. Verify template saved

**Expected:** Template creation
- [ ] Pass

### TC-12.5.3: Template with Content Controls
**Steps:**
1. Create template with fillable controls
2. Create document from template
3. Fill in controls
4. Verify original template unchanged

**Expected:** Fillable template
- [ ] Pass

---

## 12.6 Macros/Automation (if supported)

### TC-12.6.1: Record Macro
**Steps:**
1. Start macro recording
2. Perform actions
3. Stop recording
4. Verify macro saved

**Expected:** Macro recording
- [ ] Pass (if supported)
- [ ] N/A

### TC-12.6.2: Run Macro
**Steps:**
1. Run saved macro
2. Verify actions replay

**Expected:** Macro execution
- [ ] Pass (if supported)
- [ ] N/A

---

## 12.7 Symbols and Special Characters

### TC-12.7.1: Insert Symbol
**Steps:**
1. Insert > Symbol
2. Select symbol (©, ™, etc.)
3. Verify symbol inserted

**Expected:** Symbol insertion
- [ ] Pass

### TC-12.7.2: Insert Special Character
**Steps:**
1. Insert special characters:
   - Em dash (—)
   - En dash (–)
   - Non-breaking space
2. Verify each inserted

**Expected:** Special characters
- [ ] Pass

### TC-12.7.3: Symbol Shortcuts
**Steps:**
1. Type (c) for ©
2. Type (r) for ®
3. Verify auto-correct symbols

**Expected:** Auto-correct symbols
- [ ] Pass

---

## 12.8 Outline View

### TC-12.8.1: Switch to Outline View
**Steps:**
1. View > Outline
2. Verify document shows outline format

**Expected:** Outline view
- [ ] Pass

### TC-12.8.2: Promote/Demote Headings
**Steps:**
1. In outline view
2. Select heading
3. Promote/demote level
4. Verify heading style changes

**Expected:** Heading level changes
- [ ] Pass

### TC-12.8.3: Expand/Collapse Sections
**Steps:**
1. Collapse section
2. Verify content hidden
3. Expand section
4. Verify content shown

**Expected:** Section collapse/expand
- [ ] Pass

### TC-12.8.4: Move Sections in Outline
**Steps:**
1. Select section in outline
2. Move up/down
3. Verify document reorganized

**Expected:** Outline reordering
- [ ] Pass

---

## 12.9 Navigation Pane

### TC-12.9.1: Open Navigation Pane
**Steps:**
1. View > Navigation Pane
2. Verify pane shows document structure

**Expected:** Navigation pane opens
- [ ] Pass

### TC-12.9.2: Navigate via Headings
**Steps:**
1. Click heading in navigation
2. Verify jumps to that heading

**Expected:** Heading navigation
- [ ] Pass

### TC-12.9.3: Search from Navigation
**Steps:**
1. Type search term in navigation
2. Verify results highlighted

**Expected:** Search integration
- [ ] Pass

### TC-12.9.4: Page Thumbnails
**Steps:**
1. Switch to page thumbnails view
2. Click thumbnail
3. Verify navigates to page

**Expected:** Thumbnail navigation
- [ ] Pass

---

## Summary

| Section | Tests | Passed | Failed |
|---------|-------|--------|--------|
| 12.1 Equations | 8 | | |
| 12.2 Charts | 7 | | |
| 12.3 Content Controls | 8 | | |
| 12.4 Mail Merge | 8 | | |
| 12.5 Templates | 3 | | |
| 12.6 Macros | 2 | | |
| 12.7 Symbols | 3 | | |
| 12.8 Outline View | 4 | | |
| 12.9 Navigation Pane | 4 | | |
| **Total** | **47** | | |
