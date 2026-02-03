//! Viewport management for virtualized rendering
//!
//! This module provides viewport tracking and page visibility calculations
//! to enable render virtualization - only rendering pages that are visible
//! or within a buffer zone of the viewport.

use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Represents the current viewport state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Viewport {
    /// Current vertical scroll position (top of viewport in document coordinates)
    pub scroll_y: f64,
    /// Height of the visible area
    pub visible_height: f64,
    /// Number of pages to render above and below the visible area
    pub buffer_pages: usize,
    /// Current horizontal scroll position (for future horizontal virtualization)
    pub scroll_x: f64,
    /// Width of the visible area
    pub visible_width: f64,
}

impl Viewport {
    /// Create a new viewport with default buffer
    pub fn new(scroll_y: f64, visible_height: f64) -> Self {
        Self {
            scroll_y,
            visible_height,
            buffer_pages: 2,
            scroll_x: 0.0,
            visible_width: 0.0,
        }
    }

    /// Create a new viewport with custom buffer
    pub fn with_buffer(scroll_y: f64, visible_height: f64, buffer_pages: usize) -> Self {
        Self {
            scroll_y,
            visible_height,
            buffer_pages,
            scroll_x: 0.0,
            visible_width: 0.0,
        }
    }

    /// Update the scroll position
    pub fn set_scroll(&mut self, scroll_y: f64) {
        self.scroll_y = scroll_y;
    }

    /// Update the viewport dimensions
    pub fn set_dimensions(&mut self, visible_width: f64, visible_height: f64) {
        self.visible_width = visible_width;
        self.visible_height = visible_height;
    }

    /// Set the buffer pages count
    pub fn set_buffer_pages(&mut self, buffer_pages: usize) {
        self.buffer_pages = buffer_pages;
    }

    /// Get the top position of the viewport
    pub fn top(&self) -> f64 {
        self.scroll_y
    }

    /// Get the bottom position of the viewport
    pub fn bottom(&self) -> f64 {
        self.scroll_y + self.visible_height
    }

    /// Calculate the range of page indices that should be rendered
    ///
    /// Returns a range of page indices that are either visible or within
    /// the buffer zone.
    ///
    /// # Arguments
    /// * `page_tops` - Cumulative Y positions of each page's top edge
    /// * `page_heights` - Heights of each page
    ///
    /// # Returns
    /// A range of page indices to render (start..end, exclusive)
    pub fn visible_page_range(
        &self,
        page_tops: &[f64],
        page_heights: &[f64],
    ) -> Range<usize> {
        if page_tops.is_empty() || page_heights.is_empty() {
            return 0..0;
        }

        let page_count = page_tops.len().min(page_heights.len());

        // Find the first visible page
        let viewport_top = self.scroll_y;
        let viewport_bottom = self.scroll_y + self.visible_height;

        let mut first_visible = page_count;
        let mut last_visible = 0;

        for (i, &page_top) in page_tops.iter().take(page_count).enumerate() {
            let page_bottom = page_top + page_heights[i];

            // Check if this page intersects with the viewport
            if page_bottom > viewport_top && page_top < viewport_bottom {
                if i < first_visible {
                    first_visible = i;
                }
                last_visible = i;
            }
        }

        // If no pages are visible, return empty range
        if first_visible > last_visible {
            return 0..0;
        }

        // Apply buffer pages
        let start = first_visible.saturating_sub(self.buffer_pages);
        let end = (last_visible + 1 + self.buffer_pages).min(page_count);

        start..end
    }

    /// Check if a specific page should be rendered
    ///
    /// Returns true if the page is visible or within the buffer zone
    pub fn should_render_page(
        &self,
        page_index: usize,
        page_tops: &[f64],
        page_heights: &[f64],
    ) -> bool {
        let range = self.visible_page_range(page_tops, page_heights);
        range.contains(&page_index)
    }

    /// Calculate the intersection ratio for a page (0.0 to 1.0)
    ///
    /// Returns 1.0 if the page is fully visible, 0.0 if not visible at all,
    /// and a value in between for partially visible pages.
    pub fn page_visibility_ratio(
        &self,
        page_index: usize,
        page_tops: &[f64],
        page_heights: &[f64],
    ) -> f64 {
        if page_index >= page_tops.len() || page_index >= page_heights.len() {
            return 0.0;
        }

        let page_top = page_tops[page_index];
        let page_height = page_heights[page_index];
        let page_bottom = page_top + page_height;

        let viewport_top = self.scroll_y;
        let viewport_bottom = self.scroll_y + self.visible_height;

        // No intersection
        if page_bottom <= viewport_top || page_top >= viewport_bottom {
            return 0.0;
        }

        // Calculate visible portion
        let visible_top = page_top.max(viewport_top);
        let visible_bottom = page_bottom.min(viewport_bottom);
        let visible_height = visible_bottom - visible_top;

        if page_height > 0.0 {
            (visible_height / page_height).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Check if a page is fully visible (not just partially)
    pub fn is_page_fully_visible(
        &self,
        page_index: usize,
        page_tops: &[f64],
        page_heights: &[f64],
    ) -> bool {
        if page_index >= page_tops.len() || page_index >= page_heights.len() {
            return false;
        }

        let page_top = page_tops[page_index];
        let page_height = page_heights[page_index];
        let page_bottom = page_top + page_height;

        page_top >= self.scroll_y && page_bottom <= self.scroll_y + self.visible_height
    }

    /// Get page visibility information for all pages
    pub fn get_page_visibility(
        &self,
        page_tops: &[f64],
        page_heights: &[f64],
    ) -> Vec<PageVisibility> {
        let page_count = page_tops.len().min(page_heights.len());
        let visible_range = self.visible_page_range(page_tops, page_heights);

        (0..page_count)
            .map(|i| {
                let is_in_range = visible_range.contains(&i);
                let visibility_ratio = if is_in_range {
                    self.page_visibility_ratio(i, page_tops, page_heights)
                } else {
                    0.0
                };

                PageVisibility {
                    page_index: i,
                    is_visible: visibility_ratio > 0.0,
                    is_buffered: is_in_range && visibility_ratio == 0.0,
                    visibility_ratio,
                }
            })
            .collect()
    }
}

/// Information about a single page's visibility state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageVisibility {
    /// Index of the page
    pub page_index: usize,
    /// Whether the page is currently visible (even partially)
    pub is_visible: bool,
    /// Whether the page is in the buffer zone (not visible but should be rendered)
    pub is_buffered: bool,
    /// How much of the page is visible (0.0 to 1.0)
    pub visibility_ratio: f64,
}

/// Result of a virtualized render request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VirtualizedRenderResult {
    /// The range of pages that were rendered
    pub rendered_range: (usize, usize),
    /// Total height of all pages (for scroll container)
    pub total_height: f64,
    /// Offset from top to start rendering (for positioning)
    pub offset_top: f64,
    /// Indices of pages that are fully visible
    pub fully_visible_pages: Vec<usize>,
    /// Indices of pages that are partially visible
    pub partially_visible_pages: Vec<usize>,
    /// Indices of pages in buffer zone
    pub buffered_pages: Vec<usize>,
}

impl VirtualizedRenderResult {
    /// Create from viewport and page data
    pub fn from_viewport(
        viewport: &Viewport,
        page_tops: &[f64],
        page_heights: &[f64],
        page_gap: f64,
    ) -> Self {
        let page_count = page_tops.len().min(page_heights.len());
        if page_count == 0 {
            return Self::default();
        }

        let range = viewport.visible_page_range(page_tops, page_heights);

        // Calculate total height
        let last_page_bottom = if let Some(&last_top) = page_tops.last() {
            if let Some(&last_height) = page_heights.last() {
                last_top + last_height + page_gap
            } else {
                last_top
            }
        } else {
            0.0
        };

        // Calculate offset for positioning
        let offset_top = if range.start < page_tops.len() {
            page_tops[range.start]
        } else {
            0.0
        };

        let mut fully_visible = Vec::new();
        let mut partially_visible = Vec::new();
        let mut buffered = Vec::new();

        for i in range.clone() {
            let ratio = viewport.page_visibility_ratio(i, page_tops, page_heights);
            if ratio >= 1.0 {
                fully_visible.push(i);
            } else if ratio > 0.0 {
                partially_visible.push(i);
            } else {
                buffered.push(i);
            }
        }

        Self {
            rendered_range: (range.start, range.end),
            total_height: last_page_bottom,
            offset_top,
            fully_visible_pages: fully_visible,
            partially_visible_pages: partially_visible,
            buffered_pages: buffered,
        }
    }
}

/// Configuration for virtualized rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualizationConfig {
    /// Number of pages to render above/below visible area
    pub buffer_pages: usize,
    /// Whether to enable page caching
    pub enable_caching: bool,
    /// Maximum number of pages to cache
    pub max_cached_pages: usize,
    /// Threshold for scroll delta to trigger re-render (in pixels)
    pub scroll_threshold: f64,
    /// Debounce delay for scroll handling (in milliseconds)
    pub scroll_debounce_ms: u32,
}

impl Default for VirtualizationConfig {
    fn default() -> Self {
        Self {
            buffer_pages: 2,
            enable_caching: true,
            max_cached_pages: 10,
            scroll_threshold: 50.0,
            scroll_debounce_ms: 16, // ~60fps
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pages(count: usize, height: f64, gap: f64) -> (Vec<f64>, Vec<f64>) {
        let mut tops = Vec::with_capacity(count);
        let heights = vec![height; count];

        let mut current_top = gap;
        for _ in 0..count {
            tops.push(current_top);
            current_top += height + gap;
        }

        (tops, heights)
    }

    #[test]
    fn test_visible_page_range_all_visible() {
        let viewport = Viewport::with_buffer(0.0, 3000.0, 0);
        let (tops, heights) = create_test_pages(3, 800.0, 20.0);

        let range = viewport.visible_page_range(&tops, &heights);
        assert_eq!(range, 0..3);
    }

    #[test]
    fn test_visible_page_range_partial() {
        // Viewport showing only middle page
        let viewport = Viewport::with_buffer(850.0, 800.0, 0);
        let (tops, heights) = create_test_pages(5, 800.0, 20.0);

        let range = viewport.visible_page_range(&tops, &heights);
        // Should see pages around y=850 to y=1650
        // Page 0: 20-820
        // Page 1: 840-1640
        // Page 2: 1660-2460
        assert_eq!(range, 1..2);
    }

    #[test]
    fn test_visible_page_range_with_buffer() {
        let viewport = Viewport::with_buffer(850.0, 800.0, 2);
        let (tops, heights) = create_test_pages(5, 800.0, 20.0);

        let range = viewport.visible_page_range(&tops, &heights);
        // Visible: 1, buffered: 0, 2, 3
        assert_eq!(range, 0..4);
    }

    #[test]
    fn test_page_visibility_ratio() {
        let viewport = Viewport::with_buffer(400.0, 800.0, 0);
        let (tops, heights) = create_test_pages(3, 800.0, 20.0);

        // Page 0: 20-820, viewport: 400-1200
        // Visible portion: 400-820 = 420 out of 800
        let ratio = viewport.page_visibility_ratio(0, &tops, &heights);
        assert!((ratio - 0.525).abs() < 0.01);

        // Page 1: 840-1640, viewport: 400-1200
        // Visible portion: 840-1200 = 360 out of 800
        let ratio1 = viewport.page_visibility_ratio(1, &tops, &heights);
        assert!((ratio1 - 0.45).abs() < 0.01);
    }

    #[test]
    fn test_should_render_page() {
        let viewport = Viewport::with_buffer(0.0, 1000.0, 1);
        let (tops, heights) = create_test_pages(5, 800.0, 20.0);

        // Page 0 and 1 visible, page 2 in buffer
        assert!(viewport.should_render_page(0, &tops, &heights));
        assert!(viewport.should_render_page(1, &tops, &heights));
        assert!(viewport.should_render_page(2, &tops, &heights)); // buffer
        assert!(!viewport.should_render_page(3, &tops, &heights));
    }

    #[test]
    fn test_virtualized_render_result() {
        let viewport = Viewport::with_buffer(850.0, 800.0, 1);
        let (tops, heights) = create_test_pages(5, 800.0, 20.0);

        let result = VirtualizedRenderResult::from_viewport(&viewport, &tops, &heights, 20.0);

        assert_eq!(result.rendered_range, (0, 3));
        assert!(result.total_height > 0.0);
    }

    #[test]
    fn test_empty_pages() {
        let viewport = Viewport::new(0.0, 1000.0);
        let tops: Vec<f64> = vec![];
        let heights: Vec<f64> = vec![];

        let range = viewport.visible_page_range(&tops, &heights);
        assert_eq!(range, 0..0);
    }
}
