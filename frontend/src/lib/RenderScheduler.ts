/**
 * RenderScheduler - Efficient render scheduling for the document editor
 *
 * This module provides:
 * - Dirty tracking for pages
 * - RAF-based render loop for smooth 60fps rendering
 * - Viewport tracking to only render visible pages
 * - Canvas layer caching for static content
 * - Debounced scroll handling
 */

import { PageRender, RenderModel } from './types';

// =============================================================================
// Types
// =============================================================================

/**
 * Viewport bounds in document coordinates
 */
export interface ViewportBounds {
  top: number;
  bottom: number;
  left: number;
  right: number;
  width: number;
  height: number;
}

/**
 * Page visibility state
 */
export interface PageVisibility {
  pageIndex: number;
  isVisible: boolean;
  isBuffered: boolean; // In buffer zone above/below visible area
  intersectionRatio: number; // 0-1 how much of page is visible
}

/**
 * Render priority levels
 */
export enum RenderPriority {
  Immediate = 0, // Render this frame (user interaction)
  High = 1, // Render soon (visible content)
  Normal = 2, // Render when convenient (buffer pages)
  Low = 3, // Render in background (off-screen pages)
}

/**
 * Page render request
 */
interface PageRenderRequest {
  pageIndex: number;
  priority: RenderPriority;
  timestamp: number;
}

/**
 * Cached page canvas
 */
interface PageCache {
  canvas: OffscreenCanvas | HTMLCanvasElement;
  version: number; // For invalidation
  pageIndex: number;
}

/**
 * Render callback type
 */
export type RenderCallback = (
  ctx: CanvasRenderingContext2D,
  page: PageRender,
  pageX: number,
  pageY: number
) => void;

/**
 * Render complete callback type
 */
export type RenderCompleteCallback = () => void;

// =============================================================================
// Configuration
// =============================================================================

export interface RenderSchedulerConfig {
  /** Number of pages to buffer above and below viewport */
  bufferPages: number;
  /** Target frame rate (default 60) */
  targetFps: number;
  /** Enable canvas caching for static content */
  enableCaching: boolean;
  /** Maximum cached pages */
  maxCachedPages: number;
  /** Scroll debounce delay in ms */
  scrollDebounceMs: number;
  /** Callback when rendering is complete */
  onRenderComplete?: RenderCompleteCallback;
}

const DEFAULT_CONFIG: RenderSchedulerConfig = {
  bufferPages: 2,
  targetFps: 60,
  enableCaching: true,
  maxCachedPages: 10,
  scrollDebounceMs: 16, // ~1 frame at 60fps
};

// =============================================================================
// PageLayout interface for internal use
// =============================================================================

export interface PageLayout {
  page: PageRender;
  x: number;
  y: number;
  width: number;
  height: number;
}

// =============================================================================
// RenderScheduler Class
// =============================================================================

export class RenderScheduler {
  private config: RenderSchedulerConfig;

  // State
  private renderModel: RenderModel | null = null;
  private pageLayouts: PageLayout[] = [];
  private viewport: ViewportBounds = {
    top: 0,
    bottom: 0,
    left: 0,
    right: 0,
    width: 0,
    height: 0,
  };

  // Dirty tracking
  private dirtyPages: Set<number> = new Set();
  private globalDirty: boolean = true; // Full redraw needed

  // Render queue
  private renderQueue: Map<number, PageRenderRequest> = new Map();
  private rafId: number | null = null;
  private isRendering: boolean = false;
  private lastFrameTime: number = 0;
  private frameInterval: number;

  // Page caching
  private pageCache: Map<number, PageCache> = new Map();
  private pageVersions: Map<number, number> = new Map();

  // Scroll handling
  private scrollDebounceTimer: number | null = null;

  // Callbacks
  private renderCallback: RenderCallback | null = null;
  private onRenderComplete: RenderCompleteCallback | null = null;

  // Canvas reference
  private targetCanvas: HTMLCanvasElement | null = null;
  private targetCtx: CanvasRenderingContext2D | null = null;

  constructor(config: Partial<RenderSchedulerConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.frameInterval = 1000 / this.config.targetFps;
    this.onRenderComplete = config.onRenderComplete || null;
  }

  // ===========================================================================
  // Setup
  // ===========================================================================

  /**
   * Set the target canvas for rendering
   */
  setCanvas(canvas: HTMLCanvasElement): void {
    this.targetCanvas = canvas;
    this.targetCtx = canvas.getContext('2d');
    this.markGlobalDirty();
  }

  /**
   * Set the render callback for drawing page content
   */
  setRenderCallback(callback: RenderCallback): void {
    this.renderCallback = callback;
  }

  /**
   * Update the render model
   */
  setRenderModel(model: RenderModel | null): void {
    this.renderModel = model;
    this.markGlobalDirty();
  }

  /**
   * Update page layouts (calculated by EditorCanvas)
   */
  setPageLayouts(layouts: PageLayout[]): void {
    this.pageLayouts = layouts;
  }

  // ===========================================================================
  // Dirty Tracking
  // ===========================================================================

  /**
   * Mark a specific page as dirty
   */
  markPageDirty(pageIndex: number): void {
    this.dirtyPages.add(pageIndex);
    this.invalidatePageCache(pageIndex);
    this.scheduleRender(pageIndex, RenderPriority.High);
  }

  /**
   * Mark multiple pages as dirty
   */
  markPagesDirty(pageIndices: number[]): void {
    for (const index of pageIndices) {
      this.markPageDirty(index);
    }
  }

  /**
   * Mark all pages as dirty (full redraw)
   */
  markGlobalDirty(): void {
    this.globalDirty = true;
    this.dirtyPages.clear();
    this.clearAllCaches();
    this.requestRender();
  }

  /**
   * Check if a page is dirty
   */
  isPageDirty(pageIndex: number): boolean {
    return this.globalDirty || this.dirtyPages.has(pageIndex);
  }

  /**
   * Clear dirty state for a page
   */
  clearPageDirty(pageIndex: number): void {
    this.dirtyPages.delete(pageIndex);
  }

  /**
   * Clear all dirty state
   */
  clearAllDirty(): void {
    this.globalDirty = false;
    this.dirtyPages.clear();
  }

  // ===========================================================================
  // Viewport Tracking
  // ===========================================================================

  /**
   * Update viewport bounds
   */
  updateViewport(scrollX: number, scrollY: number, width: number, height: number): void {
    const newViewport: ViewportBounds = {
      top: scrollY,
      bottom: scrollY + height,
      left: scrollX,
      right: scrollX + width,
      width,
      height,
    };

    // Check if viewport actually changed
    if (this.viewportEquals(this.viewport, newViewport)) {
      return;
    }

    this.viewport = newViewport;
    this.onViewportChanged();
  }

  /**
   * Handle viewport changes with debouncing
   */
  private onViewportChanged(): void {
    // Immediate update for small changes
    this.scheduleVisiblePagesRender();

    // Debounce buffer page rendering
    if (this.scrollDebounceTimer !== null) {
      window.clearTimeout(this.scrollDebounceTimer);
    }

    this.scrollDebounceTimer = window.setTimeout(() => {
      this.scheduleBufferPagesRender();
      this.scrollDebounceTimer = null;
    }, this.config.scrollDebounceMs);
  }

  /**
   * Compare two viewports for equality
   */
  private viewportEquals(a: ViewportBounds, b: ViewportBounds): boolean {
    return (
      a.top === b.top &&
      a.bottom === b.bottom &&
      a.left === b.left &&
      a.right === b.right &&
      a.width === b.width &&
      a.height === b.height
    );
  }

  /**
   * Get page visibility information
   */
  getPageVisibility(): PageVisibility[] {
    const result: PageVisibility[] = [];

    for (const layout of this.pageLayouts) {
      const pageTop = layout.y;
      const pageBottom = layout.y + layout.height;

      // Check if page intersects viewport
      const isVisible =
        pageTop < this.viewport.bottom && pageBottom > this.viewport.top;

      // Check if page is in buffer zone
      const bufferDistance = this.config.bufferPages * (layout.height + 20); // Include page gap
      const isBuffered =
        !isVisible &&
        pageTop < this.viewport.bottom + bufferDistance &&
        pageBottom > this.viewport.top - bufferDistance;

      // Calculate intersection ratio
      let intersectionRatio = 0;
      if (isVisible) {
        const visibleTop = Math.max(pageTop, this.viewport.top);
        const visibleBottom = Math.min(pageBottom, this.viewport.bottom);
        const visibleHeight = Math.max(0, visibleBottom - visibleTop);
        intersectionRatio = visibleHeight / layout.height;
      }

      result.push({
        pageIndex: layout.page.page_index,
        isVisible,
        isBuffered,
        intersectionRatio,
      });
    }

    return result;
  }

  /**
   * Get indices of visible pages
   */
  getVisiblePageIndices(): number[] {
    return this.getPageVisibility()
      .filter((p) => p.isVisible)
      .map((p) => p.pageIndex);
  }

  /**
   * Get indices of buffered pages (visible + buffer)
   */
  getBufferedPageIndices(): number[] {
    return this.getPageVisibility()
      .filter((p) => p.isVisible || p.isBuffered)
      .map((p) => p.pageIndex);
  }

  // ===========================================================================
  // Render Scheduling
  // ===========================================================================

  /**
   * Schedule a page for rendering
   */
  scheduleRender(pageIndex: number, priority: RenderPriority): void {
    const existing = this.renderQueue.get(pageIndex);

    // Only update if higher priority
    if (!existing || existing.priority > priority) {
      this.renderQueue.set(pageIndex, {
        pageIndex,
        priority,
        timestamp: performance.now(),
      });
    }

    this.requestRender();
  }

  /**
   * Schedule all visible pages for rendering
   */
  private scheduleVisiblePagesRender(): void {
    const visibility = this.getPageVisibility();

    for (const page of visibility) {
      if (page.isVisible) {
        this.scheduleRender(page.pageIndex, RenderPriority.High);
      }
    }
  }

  /**
   * Schedule buffer pages for rendering
   */
  private scheduleBufferPagesRender(): void {
    const visibility = this.getPageVisibility();

    for (const page of visibility) {
      if (page.isBuffered && !page.isVisible) {
        this.scheduleRender(page.pageIndex, RenderPriority.Normal);
      }
    }
  }

  /**
   * Request a render frame
   */
  requestRender(): void {
    if (this.rafId !== null) {
      return; // Already scheduled
    }

    this.rafId = requestAnimationFrame((timestamp) => this.renderFrame(timestamp));
  }

  /**
   * Cancel pending render
   */
  cancelRender(): void {
    if (this.rafId !== null) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
  }

  // ===========================================================================
  // Render Loop
  // ===========================================================================

  /**
   * Main render frame
   */
  private renderFrame(timestamp: number): void {
    this.rafId = null;

    // Frame rate limiting
    const elapsed = timestamp - this.lastFrameTime;
    if (elapsed < this.frameInterval) {
      // Schedule next frame
      this.requestRender();
      return;
    }

    this.lastFrameTime = timestamp;
    this.isRendering = true;

    try {
      this.performRender();
    } finally {
      this.isRendering = false;
    }

    // Continue render loop if there are pending items
    if (this.renderQueue.size > 0 || this.globalDirty || this.dirtyPages.size > 0) {
      this.requestRender();
    } else if (this.onRenderComplete) {
      this.onRenderComplete();
    }
  }

  /**
   * Perform the actual rendering
   */
  private performRender(): void {
    if (!this.targetCanvas || !this.targetCtx || !this.renderCallback) {
      return;
    }

    const ctx = this.targetCtx;

    // Handle global dirty (full redraw)
    if (this.globalDirty) {
      this.renderAllPages(ctx);
      this.globalDirty = false;
      this.renderQueue.clear();
      return;
    }

    // Process render queue by priority
    const requests = Array.from(this.renderQueue.values()).sort(
      (a, b) => a.priority - b.priority || a.timestamp - b.timestamp
    );

    // Render high priority pages this frame (visible pages)
    const highPriorityCount = Math.min(
      requests.filter((r) => r.priority <= RenderPriority.High).length,
      5 // Max 5 pages per frame for high priority
    );

    const toRender = requests.slice(0, Math.max(highPriorityCount, 2));

    for (const request of toRender) {
      this.renderPage(ctx, request.pageIndex);
      this.renderQueue.delete(request.pageIndex);
      this.clearPageDirty(request.pageIndex);
    }
  }

  /**
   * Render all pages (full redraw)
   */
  private renderAllPages(ctx: CanvasRenderingContext2D): void {
    // Clear the entire canvas
    const canvas = this.targetCanvas!;
    ctx.save();
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.fillStyle = '#e8e8e8';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.restore();

    // Get visibility info
    const visibility = this.getPageVisibility();
    const visibleIndices = new Set(
      visibility.filter((p) => p.isVisible || p.isBuffered).map((p) => p.pageIndex)
    );

    // Render visible and buffered pages
    for (const layout of this.pageLayouts) {
      if (visibleIndices.has(layout.page.page_index)) {
        this.renderPageWithLayout(ctx, layout);
      }
    }
  }

  /**
   * Render a single page by index
   */
  private renderPage(ctx: CanvasRenderingContext2D, pageIndex: number): void {
    const layout = this.pageLayouts.find((l) => l.page.page_index === pageIndex);
    if (!layout) return;

    this.renderPageWithLayout(ctx, layout);
  }

  /**
   * Render a page with its layout
   */
  private renderPageWithLayout(ctx: CanvasRenderingContext2D, layout: PageLayout): void {
    const { page, x, y } = layout;

    // Check cache first
    if (this.config.enableCaching) {
      const cached = this.getPageFromCache(page.page_index);
      if (cached) {
        ctx.drawImage(cached.canvas, x - this.viewport.left, y - this.viewport.top);
        return;
      }
    }

    // Render to cache if caching is enabled
    if (this.config.enableCaching) {
      this.renderPageToCache(layout);
      const cached = this.getPageFromCache(page.page_index);
      if (cached) {
        ctx.drawImage(cached.canvas, x - this.viewport.left, y - this.viewport.top);
        return;
      }
    }

    // Direct render (no caching or cache failed)
    if (this.renderCallback) {
      this.renderCallback(ctx, page, x - this.viewport.left, y - this.viewport.top);
    }
  }

  // ===========================================================================
  // Page Caching
  // ===========================================================================

  /**
   * Get a page from cache
   */
  private getPageFromCache(pageIndex: number): PageCache | null {
    const cached = this.pageCache.get(pageIndex);
    if (!cached) return null;

    // Check if cache is still valid
    const currentVersion = this.pageVersions.get(pageIndex) ?? 0;
    if (cached.version !== currentVersion) {
      this.pageCache.delete(pageIndex);
      return null;
    }

    return cached;
  }

  /**
   * Render a page to its cache canvas
   */
  private renderPageToCache(layout: PageLayout): void {
    const { page } = layout;

    // Create or reuse cache canvas
    let cacheCanvas: OffscreenCanvas | HTMLCanvasElement;
    const cached = this.pageCache.get(page.page_index);

    if (cached && cached.canvas.width === page.width && cached.canvas.height === page.height) {
      cacheCanvas = cached.canvas;
    } else {
      // Create new canvas
      if (typeof OffscreenCanvas !== 'undefined') {
        cacheCanvas = new OffscreenCanvas(page.width, page.height);
      } else {
        cacheCanvas = document.createElement('canvas');
        cacheCanvas.width = page.width;
        cacheCanvas.height = page.height;
      }
    }

    // Render to cache canvas
    const cacheCtx = cacheCanvas.getContext('2d') as CanvasRenderingContext2D | null;
    if (!cacheCtx) return;

    // Clear and render
    cacheCtx.clearRect(0, 0, page.width, page.height);

    if (this.renderCallback) {
      // Render with (0, 0) offset since we're rendering to cache
      this.renderCallback(cacheCtx, page, 0, 0);
    }

    // Update cache
    const version = this.pageVersions.get(page.page_index) ?? 0;
    this.pageCache.set(page.page_index, {
      canvas: cacheCanvas,
      version,
      pageIndex: page.page_index,
    });

    // Enforce cache limit
    this.enforceCacheLimit();
  }

  /**
   * Invalidate page cache
   */
  private invalidatePageCache(pageIndex: number): void {
    const currentVersion = this.pageVersions.get(pageIndex) ?? 0;
    this.pageVersions.set(pageIndex, currentVersion + 1);
    this.pageCache.delete(pageIndex);
  }

  /**
   * Clear all caches
   */
  private clearAllCaches(): void {
    this.pageCache.clear();
    // Increment all versions
    for (const [pageIndex, version] of this.pageVersions.entries()) {
      this.pageVersions.set(pageIndex, version + 1);
    }
  }

  /**
   * Enforce maximum cache size
   */
  private enforceCacheLimit(): void {
    if (this.pageCache.size <= this.config.maxCachedPages) {
      return;
    }

    // Get visible pages to keep
    const visibleIndices = new Set(this.getVisiblePageIndices());

    // Remove non-visible pages first
    const toRemove: number[] = [];
    for (const [pageIndex] of this.pageCache) {
      if (!visibleIndices.has(pageIndex)) {
        toRemove.push(pageIndex);
      }
    }

    // Remove until under limit
    for (const pageIndex of toRemove) {
      if (this.pageCache.size <= this.config.maxCachedPages) break;
      this.pageCache.delete(pageIndex);
    }
  }

  // ===========================================================================
  // Scroll Handling
  // ===========================================================================

  /**
   * Handle scroll event (should be called from EditorCanvas)
   */
  handleScroll(scrollX: number, scrollY: number, width: number, height: number): void {
    this.updateViewport(scrollX, scrollY, width, height);
  }

  /**
   * Create a debounced scroll handler
   */
  createScrollHandler(): (scrollX: number, scrollY: number, width: number, height: number) => void {
    let lastScrollX = 0;
    let lastScrollY = 0;
    let rafPending = false;

    return (scrollX: number, scrollY: number, width: number, height: number) => {
      lastScrollX = scrollX;
      lastScrollY = scrollY;

      if (!rafPending) {
        rafPending = true;
        requestAnimationFrame(() => {
          this.handleScroll(lastScrollX, lastScrollY, width, height);
          rafPending = false;
        });
      }
    };
  }

  // ===========================================================================
  // Lifecycle
  // ===========================================================================

  /**
   * Start the render loop
   */
  start(): void {
    this.requestRender();
  }

  /**
   * Stop the render loop
   */
  stop(): void {
    this.cancelRender();
    if (this.scrollDebounceTimer !== null) {
      window.clearTimeout(this.scrollDebounceTimer);
      this.scrollDebounceTimer = null;
    }
  }

  /**
   * Dispose of all resources
   */
  dispose(): void {
    this.stop();
    this.clearAllCaches();
    this.renderCallback = null;
    this.targetCanvas = null;
    this.targetCtx = null;
    this.renderModel = null;
    this.pageLayouts = [];
  }

  // ===========================================================================
  // Utility Methods
  // ===========================================================================

  /**
   * Get render statistics
   */
  getStats(): {
    dirtyPageCount: number;
    cachedPageCount: number;
    renderQueueSize: number;
    isRendering: boolean;
    visiblePageCount: number;
  } {
    return {
      dirtyPageCount: this.dirtyPages.size,
      cachedPageCount: this.pageCache.size,
      renderQueueSize: this.renderQueue.size,
      isRendering: this.isRendering,
      visiblePageCount: this.getVisiblePageIndices().length,
    };
  }

  /**
   * Force immediate render of all visible pages
   */
  forceRender(): void {
    this.markGlobalDirty();
    this.cancelRender();
    if (this.targetCtx) {
      this.performRender();
    }
  }
}

// =============================================================================
// React Hook for RenderScheduler
// =============================================================================

import { useRef, useEffect, useCallback } from 'react';

export interface UseRenderSchedulerOptions {
  bufferPages?: number;
  enableCaching?: boolean;
  onRenderComplete?: RenderCompleteCallback;
}

export interface UseRenderSchedulerReturn {
  scheduler: RenderScheduler;
  setCanvas: (canvas: HTMLCanvasElement | null) => void;
  setRenderModel: (model: RenderModel | null) => void;
  setPageLayouts: (layouts: PageLayout[]) => void;
  markPageDirty: (pageIndex: number) => void;
  markPagesDirty: (pageIndices: number[]) => void;
  markGlobalDirty: () => void;
  handleScroll: (scrollX: number, scrollY: number, width: number, height: number) => void;
  getVisiblePageIndices: () => number[];
  forceRender: () => void;
}

export function useRenderScheduler(
  options: UseRenderSchedulerOptions = {}
): UseRenderSchedulerReturn {
  const schedulerRef = useRef<RenderScheduler | null>(null);

  // Initialize scheduler
  if (!schedulerRef.current) {
    schedulerRef.current = new RenderScheduler({
      bufferPages: options.bufferPages ?? 2,
      enableCaching: options.enableCaching ?? true,
      onRenderComplete: options.onRenderComplete,
    });
  }

  // Start scheduler on mount, stop on unmount
  useEffect(() => {
    const scheduler = schedulerRef.current!;
    scheduler.start();

    return () => {
      scheduler.dispose();
    };
  }, []);

  const setCanvas = useCallback((canvas: HTMLCanvasElement | null) => {
    if (canvas) {
      schedulerRef.current?.setCanvas(canvas);
    }
  }, []);

  const setRenderModel = useCallback((model: RenderModel | null) => {
    schedulerRef.current?.setRenderModel(model);
  }, []);

  const setPageLayouts = useCallback((layouts: PageLayout[]) => {
    schedulerRef.current?.setPageLayouts(layouts);
  }, []);

  const markPageDirty = useCallback((pageIndex: number) => {
    schedulerRef.current?.markPageDirty(pageIndex);
  }, []);

  const markPagesDirty = useCallback((pageIndices: number[]) => {
    schedulerRef.current?.markPagesDirty(pageIndices);
  }, []);

  const markGlobalDirty = useCallback(() => {
    schedulerRef.current?.markGlobalDirty();
  }, []);

  const handleScroll = useCallback(
    (scrollX: number, scrollY: number, width: number, height: number) => {
      schedulerRef.current?.handleScroll(scrollX, scrollY, width, height);
    },
    []
  );

  const getVisiblePageIndices = useCallback(() => {
    return schedulerRef.current?.getVisiblePageIndices() ?? [];
  }, []);

  const forceRender = useCallback(() => {
    schedulerRef.current?.forceRender();
  }, []);

  return {
    scheduler: schedulerRef.current!,
    setCanvas,
    setRenderModel,
    setPageLayouts,
    markPageDirty,
    markPagesDirty,
    markGlobalDirty,
    handleScroll,
    getVisiblePageIndices,
    forceRender,
  };
}
