# Test Group 14: Accessibility

## Overview
Tests for keyboard navigation, screen reader support, high contrast modes, and other accessibility features.

---

## 14.1 Keyboard Navigation

### TC-14.1.1: Tab Through UI
**Steps:**
1. Press Tab repeatedly
2. Verify focus moves through all UI elements
3. Verify focus indicator visible

**Expected:** Tab navigation works
- [ ] Pass

### TC-14.1.2: Shift+Tab Reverse
**Steps:**
1. Press Shift+Tab
2. Verify focus moves backwards

**Expected:** Reverse tab works
- [ ] Pass

### TC-14.1.3: Enter Activates Buttons
**Steps:**
1. Tab to button
2. Press Enter
3. Verify button activated

**Expected:** Enter activates controls
- [ ] Pass

### TC-14.1.4: Space Toggles Checkboxes
**Steps:**
1. Tab to checkbox
2. Press Space
3. Verify checkbox toggles

**Expected:** Space toggles
- [ ] Pass

### TC-14.1.5: Escape Closes Dialogs
**Steps:**
1. Open dialog
2. Press Escape
3. Verify dialog closes

**Expected:** Escape closes dialogs
- [ ] Pass

### TC-14.1.6: Arrow Keys in Menus
**Steps:**
1. Open menu
2. Use arrow keys to navigate
3. Verify correct navigation

**Expected:** Menu arrow navigation
- [ ] Pass

### TC-14.1.7: Access Keys (Alt+Letter)
**Steps:**
1. Press Alt to show access keys
2. Press corresponding letter
3. Verify menu/command activates

**Expected:** Access keys work
- [ ] Pass

### TC-14.1.8: F6 Pane Navigation
**Steps:**
1. Press F6
2. Verify focus moves between panes

**Expected:** Pane navigation
- [ ] Pass

---

## 14.2 Focus Management

### TC-14.2.1: Visible Focus Indicator
**Steps:**
1. Tab through controls
2. Verify focus ring visible on each element

**Expected:** Clear focus indicator
- [ ] Pass

### TC-14.2.2: Focus Trap in Modal
**Steps:**
1. Open modal dialog
2. Tab through all elements
3. Verify focus stays within modal

**Expected:** Focus trapped in modal
- [ ] Pass

### TC-14.2.3: Focus Returns After Modal
**Steps:**
1. Note focused element
2. Open and close modal
3. Verify focus returns to original element

**Expected:** Focus restoration
- [ ] Pass

### TC-14.2.4: Focus on New Content
**Steps:**
1. Action that creates new content (e.g., insert table)
2. Verify focus moves to new content

**Expected:** Focus follows new content
- [ ] Pass

---

## 14.3 Screen Reader Support

### TC-14.3.1: Document Content Read
**Steps:**
1. Enable screen reader (NVDA, VoiceOver, etc.)
2. Navigate document
3. Verify text is read aloud

**Expected:** Document text announced
- [ ] Pass

### TC-14.3.2: Heading Navigation
**Steps:**
1. Use screen reader heading navigation (H key)
2. Verify headings announced with level

**Expected:** Heading navigation
- [ ] Pass

### TC-14.3.3: List Announcement
**Steps:**
1. Navigate to list
2. Verify screen reader announces list and items

**Expected:** List information
- [ ] Pass

### TC-14.3.4: Table Navigation
**Steps:**
1. Navigate to table
2. Use table navigation keys (Ctrl+Alt+Arrows)
3. Verify cell content and position announced

**Expected:** Table navigation
- [ ] Pass

### TC-14.3.5: Image Alt Text
**Steps:**
1. Navigate to image
2. Verify alt text is announced

**Expected:** Image descriptions read
- [ ] Pass

### TC-14.3.6: Button/Control Labels
**Steps:**
1. Navigate to toolbar
2. Verify each button's name announced

**Expected:** Control labels
- [ ] Pass

### TC-14.3.7: Form Control Labels
**Steps:**
1. Navigate dialog with form fields
2. Verify labels associated and announced

**Expected:** Form labels
- [ ] Pass

### TC-14.3.8: Live Region Updates
**Steps:**
1. Perform action with status update
2. Verify screen reader announces update

**Expected:** Live announcements
- [ ] Pass

---

## 14.4 High Contrast Mode

### TC-14.4.1: System High Contrast
**Steps:**
1. Enable system high contrast mode
2. Open application
3. Verify UI respects high contrast

**Expected:** High contrast support
- [ ] Pass

### TC-14.4.2: Text Visibility
**Steps:**
1. In high contrast mode
2. Verify all text is readable
3. Verify sufficient contrast

**Expected:** Text readable
- [ ] Pass

### TC-14.4.3: UI Element Visibility
**Steps:**
1. Verify buttons, borders visible
2. Verify icons have sufficient contrast

**Expected:** UI elements visible
- [ ] Pass

### TC-14.4.4: Focus Indicator in High Contrast
**Steps:**
1. Tab through controls
2. Verify focus indicator visible in high contrast

**Expected:** Focus visible in HC
- [ ] Pass

---

## 14.5 Zoom and Magnification

### TC-14.5.1: Document Zoom
**Steps:**
1. Zoom to 200%
2. Verify document scales correctly
3. Verify text remains sharp

**Expected:** Document zoom works
- [ ] Pass

### TC-14.5.2: UI Scaling
**Steps:**
1. Set system to 150% scaling
2. Open application
3. Verify UI scales appropriately

**Expected:** UI respects scaling
- [ ] Pass

### TC-14.5.3: Screen Magnifier Compatible
**Steps:**
1. Use screen magnifier tool
2. Navigate application
3. Verify magnifier can follow focus

**Expected:** Magnifier compatible
- [ ] Pass

### TC-14.5.4: Zoom Keyboard Shortcuts
**Steps:**
1. Press Ctrl+Plus to zoom in
2. Press Ctrl+Minus to zoom out
3. Press Ctrl+0 to reset
4. Verify each works

**Expected:** Zoom shortcuts
- [ ] Pass

---

## 14.6 Color and Contrast

### TC-14.6.1: Text Contrast Ratio
**Steps:**
1. Measure contrast of body text
2. Verify at least 4.5:1 ratio

**Expected:** WCAG AA text contrast
- [ ] Pass

### TC-14.6.2: UI Contrast Ratio
**Steps:**
1. Measure contrast of UI elements
2. Verify at least 3:1 ratio

**Expected:** UI contrast
- [ ] Pass

### TC-14.6.3: Not Color Alone
**Steps:**
1. Check error states
2. Verify not indicated by color alone (icons, text)

**Expected:** Multiple indicators
- [ ] Pass

### TC-14.6.4: Link Identification
**Steps:**
1. Verify links underlined or otherwise identifiable
2. Not just by color

**Expected:** Links identifiable
- [ ] Pass

---

## 14.7 Motion and Animation

### TC-14.7.1: Reduce Motion Setting
**Steps:**
1. Enable "Reduce Motion" in system
2. Verify animations reduced/disabled

**Expected:** Respects reduce motion
- [ ] Pass

### TC-14.7.2: No Auto-Playing Animation
**Steps:**
1. Check for auto-playing animations
2. Verify none that can't be paused

**Expected:** No forced animation
- [ ] Pass

### TC-14.7.3: Focus Doesn't Trigger Motion
**Steps:**
1. Tab through interface
2. Verify no disorienting animations

**Expected:** Safe focus changes
- [ ] Pass

---

## 14.8 Error Handling

### TC-14.8.1: Error Identification
**Steps:**
1. Cause a form error
2. Verify error clearly identified
3. Verify error announced to screen readers

**Expected:** Errors identified
- [ ] Pass

### TC-14.8.2: Error Recovery Instructions
**Steps:**
1. Cause an error
2. Verify instructions provided for fixing

**Expected:** Error guidance
- [ ] Pass

### TC-14.8.3: Error Focus
**Steps:**
1. Submit form with error
2. Verify focus moves to error or field

**Expected:** Focus on error
- [ ] Pass

---

## 14.9 Document Accessibility Features

### TC-14.9.1: Add Alt Text to Images
**Steps:**
1. Insert image
2. Add alt text via accessibility panel
3. Verify alt text saved

**Expected:** Alt text support
- [ ] Pass

### TC-14.9.2: Mark Decorative Images
**Steps:**
1. Insert decorative image
2. Mark as decorative
3. Verify screen reader skips it

**Expected:** Decorative marking
- [ ] Pass

### TC-14.9.3: Table Headers
**Steps:**
1. Create table
2. Mark header row
3. Verify screen reader uses headers

**Expected:** Table headers
- [ ] Pass

### TC-14.9.4: Reading Order
**Steps:**
1. Create complex layout
2. Navigate with screen reader
3. Verify logical reading order

**Expected:** Correct reading order
- [ ] Pass

### TC-14.9.5: Accessibility Checker
**Steps:**
1. Run accessibility checker
2. Verify issues identified
3. Verify suggestions provided

**Expected:** Built-in checker
- [ ] Pass

---

## 14.10 Time and Timing

### TC-14.10.1: Adjustable Timeouts
**Steps:**
1. If any feature has timeout
2. Verify user can extend or disable

**Expected:** Adjustable timeouts
- [ ] Pass

### TC-14.10.2: Autosave Doesn't Interrupt
**Steps:**
1. Type during autosave
2. Verify no interruption or lost input

**Expected:** Non-disruptive autosave
- [ ] Pass

---

## Summary

| Section | Tests | Passed | Failed |
|---------|-------|--------|--------|
| 14.1 Keyboard Navigation | 8 | | |
| 14.2 Focus Management | 4 | | |
| 14.3 Screen Reader | 8 | | |
| 14.4 High Contrast | 4 | | |
| 14.5 Zoom/Magnification | 4 | | |
| 14.6 Color/Contrast | 4 | | |
| 14.7 Motion | 3 | | |
| 14.8 Error Handling | 3 | | |
| 14.9 Document A11y | 5 | | |
| 14.10 Time/Timing | 2 | | |
| **Total** | **45** | | |

---

## WCAG 2.1 Checklist

Quick reference for WCAG conformance:

### Level A (Minimum)
- [ ] 1.1.1 Non-text Content (alt text)
- [ ] 1.3.1 Info and Relationships (semantic structure)
- [ ] 1.4.1 Use of Color (not sole indicator)
- [ ] 2.1.1 Keyboard (all functionality)
- [ ] 2.1.2 No Keyboard Trap
- [ ] 2.4.1 Bypass Blocks (skip to content)
- [ ] 2.4.2 Page Titled
- [ ] 2.4.3 Focus Order
- [ ] 2.4.4 Link Purpose
- [ ] 3.1.1 Language of Page
- [ ] 3.2.1 On Focus (no unexpected changes)
- [ ] 3.2.2 On Input (no unexpected changes)
- [ ] 3.3.1 Error Identification
- [ ] 4.1.1 Parsing (valid markup)
- [ ] 4.1.2 Name, Role, Value

### Level AA (Recommended)
- [ ] 1.4.3 Contrast Minimum (4.5:1)
- [ ] 1.4.4 Resize Text (200%)
- [ ] 1.4.10 Reflow
- [ ] 1.4.11 Non-text Contrast (3:1)
- [ ] 2.4.5 Multiple Ways (search, nav)
- [ ] 2.4.6 Headings and Labels
- [ ] 2.4.7 Focus Visible
- [ ] 3.3.3 Error Suggestion
- [ ] 3.3.4 Error Prevention
