# Test Group 6: Images and Shapes

## Overview
Tests for inserting, editing, and formatting images, shapes, and other graphical elements.

---

## 6.1 Image Insertion

### TC-6.1.1: Insert Image from File
**Steps:**
1. Insert > Picture > From File
2. Select an image file (JPG, PNG)
3. Click Insert
4. Verify image appears in document

**Expected:** Image inserted at cursor position
- [ ] Pass

### TC-6.1.2: Insert Image from Clipboard
**Steps:**
1. Copy image from another application
2. Paste in document (Ctrl+V)
3. Verify image appears

**Expected:** Pasted image inserted
- [ ] Pass

### TC-6.1.3: Drag and Drop Image
**Steps:**
1. Open file explorer
2. Drag image file into document
3. Verify image inserted

**Expected:** Drag-drop inserts image
- [ ] Pass

### TC-6.1.4: Supported Formats
**Steps:**
1. Insert images of different formats:
   - JPEG
   - PNG
   - GIF
   - BMP
   - TIFF
   - SVG
2. Verify each displays correctly

**Expected:** All common formats supported
- [ ] Pass

---

## 6.2 Image Sizing

### TC-6.2.1: Resize with Handles
**Steps:**
1. Select image
2. Drag corner handle to resize
3. Verify image scales proportionally

**Expected:** Aspect ratio preserved when using corners
- [ ] Pass

### TC-6.2.2: Resize from Edge
**Steps:**
1. Select image
2. Drag side handle
3. Verify image stretches (aspect ratio changes)

**Expected:** Non-proportional resize from edges
- [ ] Pass

### TC-6.2.3: Exact Size via Dialog
**Steps:**
1. Select image
2. Open Format > Size dialog
3. Enter exact width: 3"
4. Verify image resizes to 3" width

**Expected:** Precise size control
- [ ] Pass

### TC-6.2.4: Reset to Original Size
**Steps:**
1. Resize an image
2. Click "Reset" or "Original Size"
3. Verify image returns to original dimensions

**Expected:** Original size restored
- [ ] Pass

### TC-6.2.5: Lock Aspect Ratio
**Steps:**
1. Select image
2. Enable "Lock aspect ratio"
3. Change width
4. Verify height changes proportionally

**Expected:** Aspect ratio locked
- [ ] Pass

---

## 6.3 Image Position and Layout

### TC-6.3.1: Inline with Text
**Steps:**
1. Insert image
2. Set layout to "In Line with Text"
3. Verify image acts like a character (moves with text)

**Expected:** Image inline in text flow
- [ ] Pass

### TC-6.3.2: Square Text Wrap
**Steps:**
1. Insert image
2. Set wrap to "Square"
3. Verify text wraps around image rectangle

**Expected:** Text wraps in square around image
- [ ] Pass

### TC-6.3.3: Tight Text Wrap
**Steps:**
1. Insert image (with transparency or irregular shape)
2. Set wrap to "Tight"
3. Verify text wraps closely to image outline

**Expected:** Text follows image contour
- [ ] Pass

### TC-6.3.4: Behind Text
**Steps:**
1. Insert image
2. Set wrap to "Behind Text"
3. Verify text appears over image

**Expected:** Image as background
- [ ] Pass

### TC-6.3.5: In Front of Text
**Steps:**
1. Insert image
2. Set wrap to "In Front of Text"
3. Verify image covers text

**Expected:** Image in foreground
- [ ] Pass

### TC-6.3.6: Move Image Freely
**Steps:**
1. Set image to floating layout
2. Drag image to different position
3. Verify image can be placed anywhere

**Expected:** Free positioning
- [ ] Pass

### TC-6.3.7: Anchor to Paragraph
**Steps:**
1. Insert floating image
2. View anchor indicator
3. Move paragraph
4. Verify image moves with anchored paragraph

**Expected:** Image anchored to paragraph
- [ ] Pass

---

## 6.4 Image Editing

### TC-6.4.1: Crop Image
**Steps:**
1. Select image
2. Click Crop tool
3. Drag crop handles
4. Click outside to apply
5. Verify image cropped

**Expected:** Crop removes image portions
- [ ] Pass

### TC-6.4.2: Rotate Image
**Steps:**
1. Select image
2. Drag rotation handle
3. Verify image rotates

**Expected:** Image rotation works
- [ ] Pass

### TC-6.4.3: Rotate 90 Degrees
**Steps:**
1. Select image
2. Choose Rotate > 90° Clockwise
3. Verify image rotated exactly 90°

**Expected:** Precise rotation
- [ ] Pass

### TC-6.4.4: Flip Horizontal
**Steps:**
1. Select image
2. Flip > Horizontal
3. Verify image mirrored

**Expected:** Horizontal flip
- [ ] Pass

### TC-6.4.5: Flip Vertical
**Steps:**
1. Select image
2. Flip > Vertical
3. Verify image flipped vertically

**Expected:** Vertical flip
- [ ] Pass

---

## 6.5 Image Effects

### TC-6.5.1: Brightness/Contrast
**Steps:**
1. Select image
2. Adjust brightness
3. Adjust contrast
4. Verify visual changes

**Expected:** Brightness/contrast adjustable
- [ ] Pass

### TC-6.5.2: Color Saturation
**Steps:**
1. Select image
2. Adjust saturation/color
3. Verify color intensity changes

**Expected:** Color saturation works
- [ ] Pass

### TC-6.5.3: Artistic Effects
**Steps:**
1. Select image
2. Apply artistic effect (blur, pencil, etc.)
3. Verify effect applied

**Expected:** Artistic effects work
- [ ] Pass (if supported)

### TC-6.5.4: Picture Border
**Steps:**
1. Select image
2. Add border (color, weight)
3. Verify border appears

**Expected:** Image border
- [ ] Pass

### TC-6.5.5: Shadow Effect
**Steps:**
1. Select image
2. Apply shadow
3. Verify shadow appears

**Expected:** Drop shadow on image
- [ ] Pass

---

## 6.6 Shapes

### TC-6.6.1: Insert Basic Shape
**Steps:**
1. Insert > Shapes
2. Select rectangle
3. Draw on document
4. Verify shape appears

**Expected:** Shape inserted
- [ ] Pass

### TC-6.6.2: Insert Various Shapes
**Steps:**
1. Insert shapes:
   - Circle/Oval
   - Triangle
   - Arrow
   - Star
   - Callout
2. Verify each renders correctly

**Expected:** Various shapes available
- [ ] Pass

### TC-6.6.3: Shape Fill Color
**Steps:**
1. Select shape
2. Change fill color
3. Verify color changes

**Expected:** Shape fill works
- [ ] Pass

### TC-6.6.4: Shape Outline
**Steps:**
1. Select shape
2. Change outline color and weight
3. Verify border changes

**Expected:** Shape outline customizable
- [ ] Pass

### TC-6.6.5: Shape Effects
**Steps:**
1. Select shape
2. Apply shadow, glow, or 3D effect
3. Verify effect visible

**Expected:** Shape effects work
- [ ] Pass

### TC-6.6.6: Add Text to Shape
**Steps:**
1. Select shape
2. Type text
3. Verify text appears inside shape

**Expected:** Text inside shape
- [ ] Pass

### TC-6.6.7: Resize and Rotate Shape
**Steps:**
1. Select shape
2. Resize using handles
3. Rotate using rotation handle
4. Verify transformations work

**Expected:** Shape transforms correctly
- [ ] Pass

---

## 6.7 Text Boxes

### TC-6.7.1: Insert Text Box
**Steps:**
1. Insert > Text Box
2. Draw text box
3. Type text inside
4. Verify text contained

**Expected:** Text box created
- [ ] Pass

### TC-6.7.2: Format Text Box
**Steps:**
1. Select text box
2. Change fill, border, effects
3. Verify formatting applies

**Expected:** Text box formatting works
- [ ] Pass

### TC-6.7.3: Link Text Boxes
**Steps:**
1. Create two text boxes
2. Link them
3. Type text in first until overflow
4. Verify overflow continues in second box

**Expected:** Text flows between linked boxes
- [ ] Pass (if supported)

### TC-6.7.4: Text Box Margins
**Steps:**
1. Select text box
2. Adjust internal margins
3. Verify text positioning changes

**Expected:** Internal margins work
- [ ] Pass

---

## 6.8 Grouping and Layering

### TC-6.8.1: Group Objects
**Steps:**
1. Insert multiple shapes
2. Select all
3. Group
4. Verify they move together

**Expected:** Objects grouped
- [ ] Pass

### TC-6.8.2: Ungroup Objects
**Steps:**
1. Select grouped objects
2. Ungroup
3. Verify individual selection works

**Expected:** Objects ungrouped
- [ ] Pass

### TC-6.8.3: Bring to Front
**Steps:**
1. Create overlapping shapes
2. Select back shape
3. Bring to Front
4. Verify it's now on top

**Expected:** Layer order changed
- [ ] Pass

### TC-6.8.4: Send to Back
**Steps:**
1. Select front shape
2. Send to Back
3. Verify it's now behind others

**Expected:** Layer order changed
- [ ] Pass

### TC-6.8.5: Align Objects
**Steps:**
1. Select multiple shapes
2. Align > Left (or Center, etc.)
3. Verify shapes align

**Expected:** Alignment works
- [ ] Pass

### TC-6.8.6: Distribute Objects
**Steps:**
1. Select 3+ shapes
2. Distribute Horizontally
3. Verify equal spacing

**Expected:** Even distribution
- [ ] Pass

---

## 6.9 Alt Text and Accessibility

### TC-6.9.1: Add Alt Text to Image
**Steps:**
1. Select image
2. Open Alt Text panel
3. Enter description
4. Save

**Expected:** Alt text stored with image
- [ ] Pass

### TC-6.9.2: Add Alt Text to Shape
**Steps:**
1. Select shape
2. Add alt text description
3. Verify it's saved

**Expected:** Alt text on shapes
- [ ] Pass

---

## Summary

| Section | Tests | Passed | Failed |
|---------|-------|--------|--------|
| 6.1 Image Insertion | 4 | | |
| 6.2 Image Sizing | 5 | | |
| 6.3 Position and Layout | 7 | | |
| 6.4 Image Editing | 5 | | |
| 6.5 Image Effects | 5 | | |
| 6.6 Shapes | 7 | | |
| 6.7 Text Boxes | 4 | | |
| 6.8 Grouping/Layering | 6 | | |
| 6.9 Accessibility | 2 | | |
| **Total** | **45** | | |
