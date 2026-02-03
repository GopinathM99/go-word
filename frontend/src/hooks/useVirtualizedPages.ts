/**
 * useVirtualizedPages - Hook for virtualized page rendering
 *
 * This hook calculates which pages should be rendered based on the current
 * scroll position and viewport dimensions. It enables efficient rendering
 * of large documents by only rendering visible pages plus a configurable
 * buffer zone.
 *
 * Features:
 * - Calculates visible page range based on scroll position
 * - Supports buffer pages above and below viewport
 * - Provides placeholder dimensions for non-rendered pages
 * - Debounces rapid scroll events
 * - Memory-efficient cleanup of off-screen pages
 */

import { useState, useCallback, useMemo, useRef, useEffect } from 'react';

// =============================================================================
// Types
// =============================================================================

/**
 * Configuration options for the virtualization hook
 */
export interface UseVirtualizedPagesOptions {
  /** Total number of pages in the document */
  totalPages: number;
  /** Heights of each page in pixels (array indexed by page) */
  pageHeights: number[];
  /** Height of the visible container in pixels */
  containerHeight: number;
  /** Current scroll position (scrollTop) in pixels */
  scrollTop: number;
  /** Number of pages to render above and below visible area (default: 2) */
  bufferPages?: number;
  /** Gap between pages in pixels (default: 20) */
  pageGap?: number;
  /** Scroll debounce delay in milliseconds (default: 16) */
  scrollDebounceMs?: number;
  /** Callback when visible range changes */
  onVisibleRangeChange?: (start: number, end: number) => void;
}

/**
 * Result of the virtualization calculation
 */
export interface VirtualizedPagesResult {
  /** Range of pages to render { start, end } (end is exclusive) */
  visibleRange: { start: number; end: number };
  /** Offset from top of scroll container to first rendered page */
  offsetTop: number;
  /** Total height of all pages including gaps */
  totalHeight: number;
  /** Array of page indices that are currently visible */
  visiblePageIndices: number[];
  /** Array of page indices in buffer zone (not visible but rendered) */
  bufferedPageIndices: number[];
  /** Cumulative Y positions of each page's top edge */
  pageTops: number[];
  /** Check if a specific page should be rendered */
  shouldRenderPage: (pageIndex: number) => boolean;
  /** Get the Y position of a specific page */
  getPageTop: (pageIndex: number) => number;
  /** Get visibility ratio for a page (0.0 to 1.0) */
  getPageVisibility: (pageIndex: number) => number;
}

/**
 * Page visibility information
 */
export interface PageVisibilityInfo {
  pageIndex: number;
  isVisible: boolean;
  isBuffered: boolean;
  visibilityRatio: number;
  top: number;
  height: number;
}

// =============================================================================
// Constants
// =============================================================================

const DEFAULT_BUFFER_PAGES = 2;
const DEFAULT_PAGE_GAP = 20;
const DEFAULT_SCROLL_DEBOUNCE_MS = 16; // ~60fps

// =============================================================================
// Hook Implementation
// =============================================================================

/**
 * Hook for virtualized page rendering
 *
 * @param options - Configuration options
 * @returns Virtualization calculation results
 *
 * @example
 * ```tsx
 * const { visibleRange, offsetTop, totalHeight, shouldRenderPage } = useVirtualizedPages({
 *   totalPages: 100,
 *   pageHeights: pageHeights, // Array of heights
 *   containerHeight: window.innerHeight,
 *   scrollTop: scrollPosition,
 *   bufferPages: 2,
 * });
 *
 * // Only render visible pages
 * {pages.map((page, index) =>
 *   shouldRenderPage(index) ? <PageComponent key={index} page={page} /> : null
 * )}
 * ```
 */
export function useVirtualizedPages(
  options: UseVirtualizedPagesOptions
): VirtualizedPagesResult {
  const {
    totalPages,
    pageHeights,
    containerHeight,
    scrollTop,
    bufferPages = DEFAULT_BUFFER_PAGES,
    pageGap = DEFAULT_PAGE_GAP,
    scrollDebounceMs = DEFAULT_SCROLL_DEBOUNCE_MS,
    onVisibleRangeChange,
  } = options;

  // Track debounced scroll position
  const [debouncedScrollTop, setDebouncedScrollTop] = useState(scrollTop);
  const scrollTimeoutRef = useRef<number | null>(null);
  const lastVisibleRangeRef = useRef<{ start: number; end: number }>({ start: 0, end: 0 });

  // Debounce scroll updates
  useEffect(() => {
    if (scrollTimeoutRef.current !== null) {
      window.clearTimeout(scrollTimeoutRef.current);
    }

    scrollTimeoutRef.current = window.setTimeout(() => {
      setDebouncedScrollTop(scrollTop);
      scrollTimeoutRef.current = null;
    }, scrollDebounceMs);

    return () => {
      if (scrollTimeoutRef.current !== null) {
        window.clearTimeout(scrollTimeoutRef.current);
      }
    };
  }, [scrollTop, scrollDebounceMs]);

  // Calculate cumulative page tops (memoized for performance)
  const pageTops = useMemo(() => {
    const tops: number[] = [];
    let currentTop = pageGap;

    for (let i = 0; i < totalPages; i++) {
      tops.push(currentTop);
      const height = pageHeights[i] ?? 0;
      currentTop += height + pageGap;
    }

    return tops;
  }, [totalPages, pageHeights, pageGap]);

  // Calculate total height
  const totalHeight = useMemo(() => {
    if (totalPages === 0) return 0;

    let height = pageGap;
    for (let i = 0; i < totalPages; i++) {
      height += (pageHeights[i] ?? 0) + pageGap;
    }
    return height;
  }, [totalPages, pageHeights, pageGap]);

  // Calculate visible page range
  const calculateVisibleRange = useCallback(
    (scroll: number): { start: number; end: number } => {
      if (totalPages === 0 || pageTops.length === 0) {
        return { start: 0, end: 0 };
      }

      const viewportTop = scroll;
      const viewportBottom = scroll + containerHeight;

      let firstVisible = totalPages;
      let lastVisible = 0;

      // Find pages that intersect with the viewport
      for (let i = 0; i < totalPages; i++) {
        const pageTop = pageTops[i];
        const pageHeight = pageHeights[i] ?? 0;
        const pageBottom = pageTop + pageHeight;

        if (pageBottom > viewportTop && pageTop < viewportBottom) {
          if (i < firstVisible) {
            firstVisible = i;
          }
          lastVisible = i;
        }
      }

      // No pages visible
      if (firstVisible > lastVisible) {
        return { start: 0, end: 0 };
      }

      // Apply buffer pages
      const start = Math.max(0, firstVisible - bufferPages);
      const end = Math.min(totalPages, lastVisible + 1 + bufferPages);

      return { start, end };
    },
    [totalPages, pageTops, pageHeights, containerHeight, bufferPages]
  );

  // Calculate the visible range using debounced scroll position
  const visibleRange = useMemo(
    () => calculateVisibleRange(debouncedScrollTop),
    [calculateVisibleRange, debouncedScrollTop]
  );

  // Notify when visible range changes
  useEffect(() => {
    const { start, end } = visibleRange;
    const prev = lastVisibleRangeRef.current;

    if (prev.start !== start || prev.end !== end) {
      lastVisibleRangeRef.current = { start, end };
      onVisibleRangeChange?.(start, end);
    }
  }, [visibleRange, onVisibleRangeChange]);

  // Calculate offset top (for positioning)
  const offsetTop = useMemo(() => {
    if (visibleRange.start >= pageTops.length) {
      return 0;
    }
    return pageTops[visibleRange.start] ?? 0;
  }, [visibleRange.start, pageTops]);

  // Calculate visible and buffered page indices
  const { visiblePageIndices, bufferedPageIndices } = useMemo(() => {
    const visible: number[] = [];
    const buffered: number[] = [];

    const viewportTop = debouncedScrollTop;
    const viewportBottom = debouncedScrollTop + containerHeight;

    for (let i = visibleRange.start; i < visibleRange.end; i++) {
      const pageTop = pageTops[i];
      const pageHeight = pageHeights[i] ?? 0;
      const pageBottom = pageTop + pageHeight;

      // Check if actually visible (not just buffered)
      if (pageBottom > viewportTop && pageTop < viewportBottom) {
        visible.push(i);
      } else {
        buffered.push(i);
      }
    }

    return { visiblePageIndices: visible, bufferedPageIndices: buffered };
  }, [visibleRange, pageTops, pageHeights, debouncedScrollTop, containerHeight]);

  // Helper: Check if a page should be rendered
  const shouldRenderPage = useCallback(
    (pageIndex: number): boolean => {
      return pageIndex >= visibleRange.start && pageIndex < visibleRange.end;
    },
    [visibleRange]
  );

  // Helper: Get the Y position of a specific page
  const getPageTop = useCallback(
    (pageIndex: number): number => {
      return pageTops[pageIndex] ?? 0;
    },
    [pageTops]
  );

  // Helper: Get visibility ratio for a page (0.0 to 1.0)
  const getPageVisibility = useCallback(
    (pageIndex: number): number => {
      if (pageIndex < 0 || pageIndex >= totalPages) {
        return 0;
      }

      const pageTop = pageTops[pageIndex];
      const pageHeight = pageHeights[pageIndex] ?? 0;
      const pageBottom = pageTop + pageHeight;

      const viewportTop = debouncedScrollTop;
      const viewportBottom = debouncedScrollTop + containerHeight;

      // No intersection
      if (pageBottom <= viewportTop || pageTop >= viewportBottom) {
        return 0;
      }

      // Calculate visible portion
      const visibleTop = Math.max(pageTop, viewportTop);
      const visibleBottom = Math.min(pageBottom, viewportBottom);
      const visibleHeight = visibleBottom - visibleTop;

      if (pageHeight > 0) {
        return Math.min(1, Math.max(0, visibleHeight / pageHeight));
      }

      return 0;
    },
    [totalPages, pageTops, pageHeights, debouncedScrollTop, containerHeight]
  );

  return {
    visibleRange,
    offsetTop,
    totalHeight,
    visiblePageIndices,
    bufferedPageIndices,
    pageTops,
    shouldRenderPage,
    getPageTop,
    getPageVisibility,
  };
}

// =============================================================================
// Additional Utilities
// =============================================================================

/**
 * Get detailed visibility information for all pages
 */
export function getPageVisibilityInfo(
  options: UseVirtualizedPagesOptions
): PageVisibilityInfo[] {
  const {
    totalPages,
    pageHeights,
    containerHeight,
    scrollTop,
    bufferPages = DEFAULT_BUFFER_PAGES,
    pageGap = DEFAULT_PAGE_GAP,
  } = options;

  const result: PageVisibilityInfo[] = [];
  const viewportTop = scrollTop;
  const viewportBottom = scrollTop + containerHeight;

  let currentTop = pageGap;

  for (let i = 0; i < totalPages; i++) {
    const pageHeight = pageHeights[i] ?? 0;
    const pageTop = currentTop;
    const pageBottom = pageTop + pageHeight;

    // Calculate visibility
    let visibilityRatio = 0;
    let isVisible = false;

    if (pageBottom > viewportTop && pageTop < viewportBottom) {
      isVisible = true;
      const visibleTop = Math.max(pageTop, viewportTop);
      const visibleBottom = Math.min(pageBottom, viewportBottom);
      visibilityRatio = pageHeight > 0 ? (visibleBottom - visibleTop) / pageHeight : 0;
    }

    // Calculate if in buffer zone
    const bufferDistance = bufferPages * (pageHeight + pageGap);
    const isBuffered =
      !isVisible &&
      pageTop < viewportBottom + bufferDistance &&
      pageBottom > viewportTop - bufferDistance;

    result.push({
      pageIndex: i,
      isVisible,
      isBuffered,
      visibilityRatio,
      top: pageTop,
      height: pageHeight,
    });

    currentTop += pageHeight + pageGap;
  }

  return result;
}

/**
 * Calculate the page index at a given Y coordinate
 */
export function getPageAtPosition(
  y: number,
  pageHeights: number[],
  pageGap: number = DEFAULT_PAGE_GAP
): number {
  let currentTop = pageGap;

  for (let i = 0; i < pageHeights.length; i++) {
    const pageHeight = pageHeights[i] ?? 0;
    const pageBottom = currentTop + pageHeight;

    if (y >= currentTop && y < pageBottom) {
      return i;
    }

    currentTop += pageHeight + pageGap;
  }

  // Return last page if position is beyond all pages
  return Math.max(0, pageHeights.length - 1);
}

/**
 * Calculate scroll position to bring a specific page into view
 */
export function getScrollPositionForPage(
  pageIndex: number,
  pageHeights: number[],
  containerHeight: number,
  pageGap: number = DEFAULT_PAGE_GAP,
  alignment: 'start' | 'center' | 'end' = 'start'
): number {
  if (pageIndex < 0 || pageIndex >= pageHeights.length) {
    return 0;
  }

  // Calculate page top
  let pageTop = pageGap;
  for (let i = 0; i < pageIndex; i++) {
    pageTop += (pageHeights[i] ?? 0) + pageGap;
  }

  const pageHeight = pageHeights[pageIndex] ?? 0;

  switch (alignment) {
    case 'start':
      return pageTop - pageGap; // Small offset so page doesn't touch top
    case 'center':
      return pageTop + pageHeight / 2 - containerHeight / 2;
    case 'end':
      return pageTop + pageHeight - containerHeight + pageGap;
    default:
      return pageTop;
  }
}

/**
 * Hook for managing page render cache
 * Helps track which pages have been rendered and can be cleaned up
 */
export function usePageRenderCache(maxCachedPages: number = 10) {
  const [renderedPages, setRenderedPages] = useState<Set<number>>(new Set());
  const accessOrderRef = useRef<number[]>([]);

  const markPageRendered = useCallback(
    (pageIndex: number) => {
      setRenderedPages((prev) => {
        const next = new Set(prev);
        next.add(pageIndex);
        return next;
      });

      // Update access order
      accessOrderRef.current = accessOrderRef.current.filter((i) => i !== pageIndex);
      accessOrderRef.current.push(pageIndex);

      // Enforce cache limit
      if (accessOrderRef.current.length > maxCachedPages) {
        const toRemove = accessOrderRef.current.slice(
          0,
          accessOrderRef.current.length - maxCachedPages
        );

        setRenderedPages((prev) => {
          const next = new Set(prev);
          for (const index of toRemove) {
            next.delete(index);
          }
          return next;
        });

        accessOrderRef.current = accessOrderRef.current.slice(-maxCachedPages);
      }
    },
    [maxCachedPages]
  );

  const isPageRendered = useCallback(
    (pageIndex: number): boolean => {
      return renderedPages.has(pageIndex);
    },
    [renderedPages]
  );

  const clearCache = useCallback(() => {
    setRenderedPages(new Set());
    accessOrderRef.current = [];
  }, []);

  return {
    renderedPages,
    markPageRendered,
    isPageRendered,
    clearCache,
    cacheSize: renderedPages.size,
  };
}

export default useVirtualizedPages;
