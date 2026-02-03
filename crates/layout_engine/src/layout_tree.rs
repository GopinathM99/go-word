//! Layout tree structure

use doc_model::NodeId;
use serde::{Deserialize, Serialize};

/// A rectangle in layout coordinates
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }
}

/// Text direction
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

/// A page in the layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageBox {
    /// Page index (0-based)
    pub index: usize,
    /// Full page bounds
    pub bounds: Rect,
    /// Content area (excluding margins)
    pub content_area: Rect,
    /// Areas on this page (content, header, footer)
    pub areas: Vec<AreaBox>,
    /// Section ID that this page belongs to
    #[serde(default)]
    pub section_id: Option<NodeId>,
    /// Whether column separators should be drawn
    #[serde(default)]
    pub draw_column_separators: bool,
}

impl PageBox {
    /// Create a new page box
    pub fn new(index: usize, bounds: Rect, content_area: Rect) -> Self {
        Self {
            index,
            bounds,
            content_area,
            areas: Vec::new(),
            section_id: None,
            draw_column_separators: false,
        }
    }

    /// Create a page box for a specific section
    pub fn for_section(index: usize, bounds: Rect, content_area: Rect, section_id: NodeId) -> Self {
        Self {
            index,
            bounds,
            content_area,
            areas: Vec::new(),
            section_id: Some(section_id),
            draw_column_separators: false,
        }
    }

    /// Add an area to this page
    pub fn add_area(&mut self, area: AreaBox) {
        self.areas.push(area);
    }

    /// Get the content area (first non-header/footer area)
    pub fn content_area_box(&self) -> Option<&AreaBox> {
        // The content area is typically the second area (after header)
        // or the first if there's no header
        self.areas.iter().find(|a| !a.columns.is_empty())
    }

    /// Get columns from the content area
    pub fn content_columns(&self) -> impl Iterator<Item = &ColumnBox> {
        self.areas.iter().flat_map(|a| a.columns.iter())
    }

    /// Enable column separator drawing
    pub fn with_column_separators(mut self) -> Self {
        self.draw_column_separators = true;
        self
    }
}

/// Type of area on a page
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AreaType {
    /// Main content area
    #[default]
    Content,
    /// Header area
    Header,
    /// Footer area
    Footer,
}

/// An area within a page (content, header, footer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaBox {
    /// Bounding rectangle for this area
    pub bounds: Rect,
    /// Columns within this area
    pub columns: Vec<ColumnBox>,
    /// Type of area
    #[serde(default)]
    pub area_type: AreaType,
}

impl AreaBox {
    /// Create a new content area
    pub fn content(bounds: Rect) -> Self {
        Self {
            bounds,
            columns: Vec::new(),
            area_type: AreaType::Content,
        }
    }

    /// Create a header area
    pub fn header(bounds: Rect) -> Self {
        Self {
            bounds,
            columns: Vec::new(),
            area_type: AreaType::Header,
        }
    }

    /// Create a footer area
    pub fn footer(bounds: Rect) -> Self {
        Self {
            bounds,
            columns: Vec::new(),
            area_type: AreaType::Footer,
        }
    }

    /// Create a multi-column content area
    pub fn multi_column(bounds: Rect, column_count: usize, column_spacing: f32) -> Self {
        let mut area = Self::content(bounds);

        if column_count <= 1 {
            area.columns.push(ColumnBox::new(bounds, 0));
        } else {
            let total_spacing = column_spacing * (column_count - 1) as f32;
            let column_width = (bounds.width - total_spacing) / column_count as f32;

            for i in 0..column_count {
                let x = bounds.x + i as f32 * (column_width + column_spacing);
                let col_bounds = Rect::new(x, bounds.y, column_width, bounds.height);
                area.columns.push(ColumnBox::new(col_bounds, i));
            }
        }

        area
    }

    /// Create area with custom column bounds
    pub fn with_custom_columns(bounds: Rect, column_bounds: Vec<(f32, f32)>) -> Self {
        let mut area = Self::content(bounds);

        for (i, (x_offset, width)) in column_bounds.into_iter().enumerate() {
            let col_bounds = Rect::new(bounds.x + x_offset, bounds.y, width, bounds.height);
            area.columns.push(ColumnBox::new(col_bounds, i));
        }

        area
    }

    /// Add a column to this area
    pub fn add_column(&mut self, column: ColumnBox) {
        self.columns.push(column);
    }

    /// Get the number of columns
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Check if this is a multi-column area
    pub fn is_multi_column(&self) -> bool {
        self.columns.len() > 1
    }

    /// Get a column by index
    pub fn get_column(&self, index: usize) -> Option<&ColumnBox> {
        self.columns.get(index)
    }

    /// Get a mutable column by index
    pub fn get_column_mut(&mut self, index: usize) -> Option<&mut ColumnBox> {
        self.columns.get_mut(index)
    }
}

/// A column within an area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnBox {
    /// Bounding rectangle for this column
    pub bounds: Rect,
    /// Block-level content within this column
    pub blocks: Vec<BlockBox>,
    /// Column index (0-based)
    #[serde(default)]
    pub column_index: usize,
    /// Section ID this column belongs to
    #[serde(default)]
    pub section_id: Option<NodeId>,
}

impl ColumnBox {
    /// Create a new column box
    pub fn new(bounds: Rect, column_index: usize) -> Self {
        Self {
            bounds,
            blocks: Vec::new(),
            column_index,
            section_id: None,
        }
    }

    /// Create a column box with section ID
    pub fn with_section(bounds: Rect, column_index: usize, section_id: NodeId) -> Self {
        Self {
            bounds,
            blocks: Vec::new(),
            column_index,
            section_id: Some(section_id),
        }
    }

    /// Add a block to this column
    pub fn add_block(&mut self, block: BlockBox) {
        self.blocks.push(block);
    }

    /// Get the remaining height in this column
    pub fn remaining_height(&self) -> f32 {
        let used_height: f32 = self.blocks.iter().map(|b| b.bounds.height).sum();
        (self.bounds.height - used_height).max(0.0)
    }

    /// Get the current Y position for the next block
    pub fn current_y(&self) -> f32 {
        self.blocks.last().map(|b| b.bounds.bottom()).unwrap_or(self.bounds.y)
    }

    /// Check if the column is empty
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

/// A block element (paragraph, table, image)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockBox {
    pub node_id: NodeId,
    pub bounds: Rect,
    pub lines: Vec<LineBox>,
}

/// A line of text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineBox {
    pub bounds: Rect,
    pub baseline: f32,
    pub direction: Direction,
    pub inlines: Vec<InlineBox>,
}

/// Type of inline content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InlineType {
    /// Text run
    Text,
    /// Inline image
    Image,
    /// Inline shape
    Shape,
    /// Inline text box
    TextBox,
    /// List marker (bullet or number)
    ListMarker,
}

impl Default for InlineType {
    fn default() -> Self {
        Self::Text
    }
}

/// List marker information for rendering
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListMarkerInfo {
    /// The formatted marker text (e.g., "1.", "a)", bullet character)
    pub text: String,
    /// The font family for the marker
    pub font: Option<String>,
    /// Whether this is a bullet marker
    pub is_bullet: bool,
    /// The list level (0-8)
    pub level: u8,
}

/// An inline element (text run, inline image)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineBox {
    pub node_id: NodeId,
    pub bounds: Rect,
    pub direction: Direction,
    /// Start offset in the run's text (for text inlines)
    pub start_offset: usize,
    /// End offset in the run's text (for text inlines)
    pub end_offset: usize,
    /// Type of inline content
    #[serde(default)]
    pub inline_type: InlineType,
    /// List marker info (for list markers only)
    #[serde(default)]
    pub list_marker: Option<ListMarkerInfo>,
}

impl InlineBox {
    /// Create a new text inline box
    pub fn text(node_id: NodeId, bounds: Rect, direction: Direction, start: usize, end: usize) -> Self {
        Self {
            node_id,
            bounds,
            direction,
            start_offset: start,
            end_offset: end,
            inline_type: InlineType::Text,
            list_marker: None,
        }
    }

    /// Create a new image inline box
    pub fn image(node_id: NodeId, bounds: Rect) -> Self {
        Self {
            node_id,
            bounds,
            direction: Direction::Ltr,
            start_offset: 0,
            end_offset: 0,
            inline_type: InlineType::Image,
            list_marker: None,
        }
    }

    /// Create a new list marker inline box
    pub fn list_marker(node_id: NodeId, bounds: Rect, marker: ListMarkerInfo) -> Self {
        Self {
            node_id,
            bounds,
            direction: Direction::Ltr,
            start_offset: 0,
            end_offset: 0,
            inline_type: InlineType::ListMarker,
            list_marker: Some(marker),
        }
    }

    /// Check if this is an image inline
    pub fn is_image(&self) -> bool {
        matches!(self.inline_type, InlineType::Image)
    }

    /// Check if this is a text inline
    pub fn is_text(&self) -> bool {
        matches!(self.inline_type, InlineType::Text)
    }

    /// Check if this is a list marker inline
    pub fn is_list_marker(&self) -> bool {
        matches!(self.inline_type, InlineType::ListMarker)
    }

    /// Check if this is a shape inline
    pub fn is_shape(&self) -> bool {
        matches!(self.inline_type, InlineType::Shape)
    }

    /// Create a new shape inline box
    pub fn shape(node_id: NodeId, bounds: Rect) -> Self {
        Self {
            node_id,
            bounds,
            direction: Direction::Ltr,
            start_offset: 0,
            end_offset: 0,
            inline_type: InlineType::Shape,
            list_marker: None,
        }
    }

    /// Check if this is a text box inline
    pub fn is_textbox(&self) -> bool {
        matches!(self.inline_type, InlineType::TextBox)
    }

    /// Create a new text box inline box
    pub fn textbox(node_id: NodeId, bounds: Rect) -> Self {
        Self {
            node_id,
            bounds,
            direction: Direction::Ltr,
            start_offset: 0,
            end_offset: 0,
            inline_type: InlineType::TextBox,
            list_marker: None,
        }
    }
}

/// Floating image layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatingImage {
    /// The image node ID
    pub node_id: NodeId,
    /// Position on the page
    pub bounds: Rect,
    /// The page index this image is on
    pub page_index: usize,
    /// Z-order (higher = in front)
    pub z_order: i32,
}

/// Floating shape layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatingShape {
    /// The shape node ID
    pub node_id: NodeId,
    /// Position on the page
    pub bounds: Rect,
    /// The page index this shape is on
    pub page_index: usize,
    /// Z-order (higher = in front)
    pub z_order: i32,
    /// Rotation in degrees
    pub rotation: f32,
}

/// Floating text box layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatingTextBox {
    /// The text box node ID
    pub node_id: NodeId,
    /// Position on the page
    pub bounds: Rect,
    /// Inner content bounds (accounting for margins/borders)
    pub content_bounds: Rect,
    /// The page index this text box is on
    pub page_index: usize,
    /// Z-order (higher = in front)
    pub z_order: i32,
    /// Rotation in degrees
    pub rotation: f32,
    /// Layout blocks inside the text box
    pub blocks: Vec<BlockBox>,
}

/// Line number information for a single line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineNumberItem {
    /// The line number value to display
    pub number: u32,
    /// X position (right edge of number text, left of content area)
    pub x: f32,
    /// Y position (baseline aligned with the text line)
    pub y: f32,
    /// Font size for the line number
    pub font_size: f32,
}

/// The complete layout tree
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayoutTree {
    pub pages: Vec<PageBox>,
    /// Floating images (positioned independently of text flow)
    #[serde(default)]
    pub floating_images: Vec<FloatingImage>,
    /// Floating shapes (positioned independently of text flow)
    #[serde(default)]
    pub floating_shapes: Vec<FloatingShape>,
    /// Floating text boxes (positioned independently of text flow)
    #[serde(default)]
    pub floating_textboxes: Vec<FloatingTextBox>,
    /// Line numbers per page (indexed by page index)
    #[serde(default)]
    pub line_numbers: Vec<Vec<LineNumberItem>>,
}

impl LayoutTree {
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            floating_images: Vec::new(),
            floating_shapes: Vec::new(),
            floating_textboxes: Vec::new(),
            line_numbers: Vec::new(),
        }
    }

    pub fn add_page(&mut self, page: PageBox) {
        self.pages.push(page);
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Add a floating image to the layout
    pub fn add_floating_image(&mut self, image: FloatingImage) {
        self.floating_images.push(image);
    }

    /// Get floating images for a specific page
    pub fn floating_images_on_page(&self, page_index: usize) -> impl Iterator<Item = &FloatingImage> {
        self.floating_images.iter().filter(move |img| img.page_index == page_index)
    }

    /// Add a floating shape to the layout
    pub fn add_floating_shape(&mut self, shape: FloatingShape) {
        self.floating_shapes.push(shape);
    }

    /// Get floating shapes for a specific page
    pub fn floating_shapes_on_page(&self, page_index: usize) -> impl Iterator<Item = &FloatingShape> {
        self.floating_shapes.iter().filter(move |shape| shape.page_index == page_index)
    }

    /// Add a floating text box to the layout
    pub fn add_floating_textbox(&mut self, textbox: FloatingTextBox) {
        self.floating_textboxes.push(textbox);
    }

    /// Get floating text boxes for a specific page
    pub fn floating_textboxes_on_page(&self, page_index: usize) -> impl Iterator<Item = &FloatingTextBox> {
        self.floating_textboxes.iter().filter(move |tb| tb.page_index == page_index)
    }

    /// Add a line number item for a specific page
    pub fn add_line_number(&mut self, page_index: usize, item: LineNumberItem) {
        // Ensure we have enough page slots
        while self.line_numbers.len() <= page_index {
            self.line_numbers.push(Vec::new());
        }
        self.line_numbers[page_index].push(item);
    }

    /// Get line numbers for a specific page
    pub fn line_numbers_on_page(&self, page_index: usize) -> &[LineNumberItem] {
        self.line_numbers.get(page_index).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Check if there are any line numbers in the layout
    pub fn has_line_numbers(&self) -> bool {
        self.line_numbers.iter().any(|page| !page.is_empty())
    }
}
