/**
 * Tests for useVirtualizedPages hook
 */

import { describe, it, expect } from 'vitest';
import {
  getPageVisibilityInfo,
  getPageAtPosition,
  getScrollPositionForPage,
} from './useVirtualizedPages';

// =============================================================================
// Test Utilities
// =============================================================================

/**
 * Create test page heights array
 */
function createTestPageHeights(count: number, height: number): number[] {
  return Array(count).fill(height);
}

// =============================================================================
// getPageVisibilityInfo Tests
// =============================================================================

describe('getPageVisibilityInfo', () => {
  it('should return empty array for no pages', () => {
    const result = getPageVisibilityInfo({
      totalPages: 0,
      pageHeights: [],
      containerHeight: 800,
      scrollTop: 0,
    });

    expect(result).toHaveLength(0);
  });

  it('should mark visible pages correctly', () => {
    const pageHeights = createTestPageHeights(5, 800);
    const result = getPageVisibilityInfo({
      totalPages: 5,
      pageHeights,
      containerHeight: 800,
      scrollTop: 0,
      pageGap: 20,
    });

    // First page should be visible (at y=20)
    expect(result[0].isVisible).toBe(true);
    expect(result[0].visibilityRatio).toBeGreaterThan(0);

    // Second page might be partially visible
    // Third and beyond should not be visible
    expect(result[3].isVisible).toBe(false);
  });

  it('should calculate visibility ratios correctly', () => {
    const pageHeights = createTestPageHeights(3, 800);
    const result = getPageVisibilityInfo({
      totalPages: 3,
      pageHeights,
      containerHeight: 800,
      scrollTop: 420, // Scroll to show part of page 0 and part of page 1
      pageGap: 20,
    });

    // Page 0: top=20, bottom=820
    // Viewport: top=420, bottom=1220
    // Visible portion: 420-820 = 400 of 800 = 0.5
    expect(result[0].isVisible).toBe(true);
    expect(result[0].visibilityRatio).toBeCloseTo(0.5, 1);
  });

  it('should mark buffered pages correctly', () => {
    const pageHeights = createTestPageHeights(10, 800);
    const result = getPageVisibilityInfo({
      totalPages: 10,
      pageHeights,
      containerHeight: 800,
      scrollTop: 0,
      pageGap: 20,
      bufferPages: 2,
    });

    // Pages outside buffer should not be buffered
    // With buffer of 2 and first page visible, pages 2-3 might be buffered
    const nonVisibleNonBuffered = result.filter(p => !p.isVisible && !p.isBuffered);
    expect(nonVisibleNonBuffered.length).toBeGreaterThan(0);
  });
});

// =============================================================================
// getPageAtPosition Tests
// =============================================================================

describe('getPageAtPosition', () => {
  it('should return 0 for position in first page', () => {
    const pageHeights = createTestPageHeights(5, 800);
    const result = getPageAtPosition(100, pageHeights, 20);
    expect(result).toBe(0);
  });

  it('should return correct page for middle position', () => {
    const pageHeights = createTestPageHeights(5, 800);
    // Page 0: 20-820
    // Page 1: 840-1640
    // Page 2: 1660-2460
    const result = getPageAtPosition(1000, pageHeights, 20);
    expect(result).toBe(1);
  });

  it('should return last page for position beyond all pages', () => {
    const pageHeights = createTestPageHeights(5, 800);
    const result = getPageAtPosition(10000, pageHeights, 20);
    expect(result).toBe(4); // Last page index
  });

  it('should handle gap between pages', () => {
    const pageHeights = createTestPageHeights(5, 800);
    // Gap at 820-840 (between page 0 and 1)
    // Position 830 should return page 1 (next page)
    const result = getPageAtPosition(830, pageHeights, 20);
    expect(result).toBe(1);
  });
});

// =============================================================================
// getScrollPositionForPage Tests
// =============================================================================

describe('getScrollPositionForPage', () => {
  it('should return 0 for first page with start alignment', () => {
    const pageHeights = createTestPageHeights(5, 800);
    const result = getScrollPositionForPage(0, pageHeights, 800, 20, 'start');
    expect(result).toBe(0); // 20 (gap) - 20 (offset) = 0
  });

  it('should return correct position for middle page', () => {
    const pageHeights = createTestPageHeights(5, 800);
    // Page 2 starts at: 20 + 800 + 20 + 800 + 20 = 1660
    const result = getScrollPositionForPage(2, pageHeights, 800, 20, 'start');
    expect(result).toBe(1660 - 20); // Subtract gap offset
  });

  it('should center page for center alignment', () => {
    const pageHeights = createTestPageHeights(5, 800);
    // Page 0 starts at 20, center = 20 + 400 = 420
    // Container center = 400
    // Scroll to: 420 - 400 = 20
    const result = getScrollPositionForPage(0, pageHeights, 800, 20, 'center');
    expect(result).toBe(20);
  });

  it('should handle end alignment', () => {
    const pageHeights = createTestPageHeights(5, 800);
    // Page 0: 20-820
    // End alignment: 820 - 800 + 20 = 40
    const result = getScrollPositionForPage(0, pageHeights, 800, 20, 'end');
    expect(result).toBe(40);
  });

  it('should return 0 for invalid page index', () => {
    const pageHeights = createTestPageHeights(5, 800);
    const result = getScrollPositionForPage(-1, pageHeights, 800, 20, 'start');
    expect(result).toBe(0);

    const result2 = getScrollPositionForPage(10, pageHeights, 800, 20, 'start');
    expect(result2).toBe(0);
  });
});

// =============================================================================
// Edge Cases
// =============================================================================

describe('edge cases', () => {
  it('should handle single page document', () => {
    const pageHeights = [800];
    const result = getPageVisibilityInfo({
      totalPages: 1,
      pageHeights,
      containerHeight: 1000,
      scrollTop: 0,
    });

    expect(result).toHaveLength(1);
    expect(result[0].isVisible).toBe(true);
    expect(result[0].visibilityRatio).toBe(1); // Fully visible
  });

  it('should handle very large scroll position', () => {
    const pageHeights = createTestPageHeights(100, 800);
    const result = getPageVisibilityInfo({
      totalPages: 100,
      pageHeights,
      containerHeight: 800,
      scrollTop: 50000,
      pageGap: 20,
    });

    // Should still return valid results
    expect(result).toHaveLength(100);
    const visiblePages = result.filter(p => p.isVisible);
    expect(visiblePages.length).toBeGreaterThan(0);
  });

  it('should handle different page heights', () => {
    const pageHeights = [400, 800, 600, 1000, 500];
    const result = getPageVisibilityInfo({
      totalPages: 5,
      pageHeights,
      containerHeight: 800,
      scrollTop: 0,
    });

    expect(result).toHaveLength(5);
    // First two pages should be visible with viewport at top
    expect(result[0].isVisible).toBe(true);
    expect(result[1].isVisible).toBe(true);
  });

  it('should handle zero height pages', () => {
    const pageHeights = [800, 0, 800, 0, 800];
    const result = getPageVisibilityInfo({
      totalPages: 5,
      pageHeights,
      containerHeight: 800,
      scrollTop: 0,
    });

    // Should not crash and return valid results
    expect(result).toHaveLength(5);
    // Zero-height pages have undefined visibility ratio
    expect(result[1].height).toBe(0);
  });
});
