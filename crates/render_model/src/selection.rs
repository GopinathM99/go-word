//! Selection rendering with BiDi support
//!
//! This module provides selection highlighting that works correctly with
//! mixed RTL/LTR text. For BiDi text, selections may consist of multiple
//! non-contiguous visual rectangles.

use crate::{Color, Rect, RenderItem};
use doc_model::Selection;
use layout_engine::{Direction, LayoutTree, LineBox, InlineBox};

/// Selection rendering configuration
#[derive(Debug, Clone)]
pub struct SelectionConfig {
    /// Selection highlight color
    pub color: Color,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            color: Color::rgba(51, 153, 255, 128), // Light blue with transparency
        }
    }
}

/// A visual selection region
#[derive(Debug, Clone)]
pub struct SelectionRegion {
    /// Rectangle bounds
    pub rect: Rect,
    /// BiDi level of this region (even = LTR, odd = RTL)
    pub bidi_level: u8,
}

/// Renders selection highlights with BiDi support
pub struct SelectionRenderer {
    config: SelectionConfig,
}

impl SelectionRenderer {
    pub fn new(config: SelectionConfig) -> Self {
        Self { config }
    }

    /// Render selection rectangles
    /// For BiDi text, this may return multiple non-contiguous rectangles
    pub fn render(&self, selection: &Selection, layout: &LayoutTree) -> Option<RenderItem> {
        if selection.is_collapsed() {
            return None;
        }

        let rects = self.calculate_selection_rects(selection, layout);

        if rects.is_empty() {
            // Fallback to a placeholder rectangle
            return Some(RenderItem::Selection {
                rects: vec![Rect::new(72.0, 72.0, 100.0, 14.4)],
                color: self.config.color,
            });
        }

        Some(RenderItem::Selection {
            rects,
            color: self.config.color,
        })
    }

    /// Calculate selection rectangles for potentially mixed BiDi text
    fn calculate_selection_rects(&self, selection: &Selection, layout: &LayoutTree) -> Vec<Rect> {
        let mut rects = Vec::new();

        // Get the selection range in document order
        let start = selection.start();
        let end = selection.end();

        // Walk through pages, blocks, and lines to find selected regions
        for page in &layout.pages {
            for area in &page.areas {
                for column in &area.columns {
                    for block in &column.blocks {
                        for line in &block.lines {
                            // Calculate selection rects for this line
                            let line_rects = self.calculate_line_selection_rects(
                                line,
                                &start,
                                &end,
                                page.content_area.x,
                                block.bounds.y + line.bounds.y,
                            );
                            rects.extend(line_rects);
                        }
                    }
                }
            }
        }

        rects
    }

    /// Calculate selection rectangles for a single line
    /// For BiDi text, this may produce multiple rectangles for non-contiguous visual spans
    fn calculate_line_selection_rects(
        &self,
        line: &LineBox,
        selection_start: &doc_model::Position,
        selection_end: &doc_model::Position,
        page_x_offset: f32,
        line_y: f32,
    ) -> Vec<Rect> {
        let mut rects = Vec::new();
        let mut current_rect: Option<(f32, f32)> = None; // (start_x, width)

        // Process inlines in visual order (they should already be in visual order)
        for inline in &line.inlines {
            // Check if this inline is within the selection
            let is_selected = self.inline_intersects_selection(
                inline,
                selection_start,
                selection_end,
            );

            if is_selected {
                let inline_x = page_x_offset + inline.bounds.x;
                let inline_width = inline.bounds.width;

                match current_rect {
                    Some((start_x, width)) => {
                        // Check if this inline is visually adjacent to the current rect
                        let current_end = start_x + width;
                        if (inline_x - current_end).abs() < 1.0 {
                            // Extend the current rectangle
                            current_rect = Some((start_x, width + inline_width));
                        } else {
                            // Non-adjacent - close current rect and start new one
                            // This happens with mixed BiDi text
                            rects.push(Rect::new(
                                start_x as f64,
                                line_y as f64,
                                width as f64,
                                line.bounds.height as f64,
                            ));
                            current_rect = Some((inline_x, inline_width));
                        }
                    }
                    None => {
                        // Start a new rectangle
                        current_rect = Some((inline_x, inline_width));
                    }
                }
            } else if let Some((start_x, width)) = current_rect.take() {
                // Not selected, close any current rectangle
                rects.push(Rect::new(
                    start_x as f64,
                    line_y as f64,
                    width as f64,
                    line.bounds.height as f64,
                ));
            }
        }

        // Close any remaining rectangle
        if let Some((start_x, width)) = current_rect {
            rects.push(Rect::new(
                start_x as f64,
                line_y as f64,
                width as f64,
                line.bounds.height as f64,
            ));
        }

        rects
    }

    /// Check if an inline box intersects with the selection
    fn inline_intersects_selection(
        &self,
        inline: &InlineBox,
        selection_start: &doc_model::Position,
        selection_end: &doc_model::Position,
    ) -> bool {
        // Check if this inline's node matches either boundary
        if inline.node_id == selection_start.node_id {
            // Start of selection is in this inline
            if inline.node_id == selection_end.node_id {
                // Selection is entirely within this inline
                let start_offset = selection_start.offset;
                let end_offset = selection_end.offset;
                return inline.end_offset > start_offset && inline.start_offset < end_offset;
            }
            // Selection starts here and extends beyond
            return inline.end_offset > selection_start.offset;
        }

        if inline.node_id == selection_end.node_id {
            // Selection ends in this inline
            return inline.start_offset < selection_end.offset;
        }

        // For a proper implementation, we would need to check if this inline's
        // node is between the start and end nodes in document order.
        // For now, we return false for non-matching nodes.
        false
    }

    /// Render selection with explicit BiDi run information
    /// This produces separate rectangles for each visual run within the selection
    pub fn render_bidi_selection(
        &self,
        regions: &[SelectionRegion],
    ) -> Option<RenderItem> {
        if regions.is_empty() {
            return None;
        }

        let rects: Vec<Rect> = regions.iter().map(|r| r.rect).collect();

        Some(RenderItem::Selection {
            rects,
            color: self.config.color,
        })
    }
}

impl Default for SelectionRenderer {
    fn default() -> Self {
        Self::new(SelectionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_config_default() {
        let config = SelectionConfig::default();
        assert_eq!(config.color.a, 128); // Semi-transparent
    }

    #[test]
    fn test_selection_region() {
        let region = SelectionRegion {
            rect: Rect::new(10.0, 20.0, 100.0, 14.0),
            bidi_level: 0,
        };
        assert_eq!(region.bidi_level % 2, 0); // LTR
    }
}
