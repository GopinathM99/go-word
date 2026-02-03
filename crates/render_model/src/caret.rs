//! Caret (cursor) rendering with BiDi support
//!
//! This module handles caret positioning for bidirectional text.
//! At RTL/LTR boundaries, the caret position depends on cursor affinity.

use crate::{Color, RenderItem};
use doc_model::Selection;
use layout_engine::{Direction, LayoutTree, InlineBox};

/// Cursor affinity for BiDi boundaries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaretAffinity {
    /// Cursor prefers the leading edge (start of character)
    #[default]
    Leading,
    /// Cursor prefers the trailing edge (end of character)
    Trailing,
}

/// Caret rendering configuration
#[derive(Debug, Clone)]
pub struct CaretConfig {
    /// Caret color
    pub color: Color,
    /// Caret width in pixels
    pub width: f64,
    /// Whether to show direction indicator for BiDi
    pub show_direction_indicator: bool,
}

impl Default for CaretConfig {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            width: 1.0,
            show_direction_indicator: true,
        }
    }
}

/// Caret position with BiDi information
#[derive(Debug, Clone)]
pub struct CaretPosition {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
    /// Height
    pub height: f64,
    /// Direction of text at caret position
    pub direction: Direction,
    /// BiDi level at caret position (even = LTR, odd = RTL)
    pub bidi_level: u8,
}

impl CaretPosition {
    /// Check if the caret is in RTL text
    pub fn is_rtl(&self) -> bool {
        self.bidi_level % 2 == 1
    }
}

/// Calculates caret position from selection and layout with BiDi support
pub struct CaretRenderer {
    config: CaretConfig,
}

impl CaretRenderer {
    pub fn new(config: CaretConfig) -> Self {
        Self { config }
    }

    /// Render caret for a selection
    pub fn render(&self, selection: &Selection, layout: &LayoutTree) -> Option<RenderItem> {
        if !selection.is_collapsed() {
            return None;
        }

        // Find the position in the layout
        let caret_pos = self.find_caret_position(selection, layout)?;

        Some(RenderItem::Caret {
            x: caret_pos.x,
            y: caret_pos.y,
            height: caret_pos.height,
            color: self.config.color,
        })
    }

    /// Render caret with explicit affinity for BiDi boundaries
    pub fn render_with_affinity(
        &self,
        selection: &Selection,
        layout: &LayoutTree,
        affinity: CaretAffinity,
    ) -> Option<RenderItem> {
        if !selection.is_collapsed() {
            return None;
        }

        let caret_pos = self.find_caret_position_with_affinity(selection, layout, affinity)?;

        Some(RenderItem::Caret {
            x: caret_pos.x,
            y: caret_pos.y,
            height: caret_pos.height,
            color: self.config.color,
        })
    }

    /// Find caret position in the layout tree
    fn find_caret_position(
        &self,
        selection: &Selection,
        layout: &LayoutTree,
    ) -> Option<CaretPosition> {
        self.find_caret_position_with_affinity(selection, layout, CaretAffinity::Leading)
    }

    /// Find caret position with explicit affinity
    fn find_caret_position_with_affinity(
        &self,
        selection: &Selection,
        layout: &LayoutTree,
        affinity: CaretAffinity,
    ) -> Option<CaretPosition> {
        let focus = &selection.focus;

        // Walk through the layout tree to find the position
        for page in &layout.pages {
            for area in &page.areas {
                for column in &area.columns {
                    for block in &column.blocks {
                        if block.node_id == focus.node_id {
                            // Found the block, but we need to find the inline
                            for line in &block.lines {
                                if let Some(pos) = self.find_in_line(
                                    line,
                                    focus,
                                    affinity,
                                    page.content_area.x,
                                    block.bounds.y,
                                ) {
                                    return Some(pos);
                                }
                            }
                        }

                        // Check inlines for the matching node
                        for line in &block.lines {
                            if let Some(pos) = self.find_in_line(
                                line,
                                focus,
                                affinity,
                                page.content_area.x,
                                block.bounds.y,
                            ) {
                                return Some(pos);
                            }
                        }
                    }
                }
            }
        }

        // Fallback to a default position
        Some(CaretPosition {
            x: 72.0,
            y: 72.0,
            height: 14.4,
            direction: Direction::Ltr,
            bidi_level: 0,
        })
    }

    /// Find caret position within a line
    fn find_in_line(
        &self,
        line: &layout_engine::LineBox,
        focus: &doc_model::Position,
        affinity: CaretAffinity,
        page_x: f32,
        block_y: f32,
    ) -> Option<CaretPosition> {
        for inline in &line.inlines {
            if inline.node_id == focus.node_id {
                // Found the inline containing the caret
                if focus.offset >= inline.start_offset && focus.offset <= inline.end_offset {
                    let position = self.calculate_caret_x(inline, focus.offset, affinity);
                    return Some(CaretPosition {
                        x: (page_x + inline.bounds.x + position) as f64,
                        y: (block_y + line.bounds.y) as f64,
                        height: line.bounds.height as f64,
                        direction: inline.direction,
                        bidi_level: if inline.direction == Direction::Rtl { 1 } else { 0 },
                    });
                }
            }
        }
        None
    }

    /// Calculate caret X position within an inline box
    fn calculate_caret_x(
        &self,
        inline: &InlineBox,
        offset: usize,
        affinity: CaretAffinity,
    ) -> f32 {
        let relative_offset = offset.saturating_sub(inline.start_offset);
        let total_chars = inline.end_offset.saturating_sub(inline.start_offset);

        if total_chars == 0 {
            return 0.0;
        }

        // Calculate proportional position within the inline
        let ratio = relative_offset as f32 / total_chars as f32;

        // For RTL text, the ratio is inverted
        if inline.direction == Direction::Rtl {
            inline.bounds.width * (1.0 - ratio)
        } else {
            inline.bounds.width * ratio
        }
    }

    /// Get the caret position for visual rendering
    pub fn get_caret_position(
        &self,
        selection: &Selection,
        layout: &LayoutTree,
    ) -> Option<CaretPosition> {
        self.find_caret_position(selection, layout)
    }
}

impl Default for CaretRenderer {
    fn default() -> Self {
        Self::new(CaretConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caret_config_default() {
        let config = CaretConfig::default();
        assert_eq!(config.width, 1.0);
        assert!(config.show_direction_indicator);
    }

    #[test]
    fn test_caret_position_rtl() {
        let pos = CaretPosition {
            x: 100.0,
            y: 50.0,
            height: 14.0,
            direction: Direction::Rtl,
            bidi_level: 1,
        };
        assert!(pos.is_rtl());
    }

    #[test]
    fn test_caret_position_ltr() {
        let pos = CaretPosition {
            x: 100.0,
            y: 50.0,
            height: 14.0,
            direction: Direction::Ltr,
            bidi_level: 0,
        };
        assert!(!pos.is_rtl());
    }
}
