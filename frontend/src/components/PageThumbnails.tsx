/**
 * PageThumbnails - Sidebar component for Print Preview showing page thumbnails
 *
 * Features:
 * - Vertical scrollable sidebar showing page thumbnails
 * - Miniature preview of each page
 * - Current page highlighted with selection border
 * - Click to navigate to page
 * - Page numbers below each thumbnail
 * - Efficient rendering (only visible thumbnails + buffer)
 * - Smooth scrolling to keep current page visible
 * - Keyboard navigation support with ARIA labels
 */

import { useState, useCallback, useEffect, useRef, useMemo } from 'react';
import '../styles/PageThumbnails.css';

// =============================================================================
// Types
// =============================================================================

export interface PageThumbnailsProps {
  /** Total number of pages in the document */
  totalPages: number;
  /** Currently selected/displayed page (1-indexed) */
  currentPage: number;
  /** Callback when user selects a page */
  onPageSelect: (pageNumber: number) => void;
  /** Base64 encoded thumbnail images (indexed from 0) */
  thumbnailData?: string[];
  /** Whether thumbnails are being generated */
  isLoading?: boolean;
}

interface ThumbnailItemProps {
  pageNumber: number;
  isSelected: boolean;
  thumbnailSrc?: string;
  isLoading: boolean;
  onClick: () => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  isFocused: boolean;
}

// =============================================================================
// Constants
// =============================================================================

/** Height of each thumbnail item including margin */
const THUMBNAIL_ITEM_HEIGHT = 160;

/** Number of items to render above and below the visible area */
const BUFFER_SIZE = 3;

// =============================================================================
// ThumbnailItem Component
// =============================================================================

function ThumbnailItem({
  pageNumber,
  isSelected,
  thumbnailSrc,
  isLoading,
  onClick,
  onKeyDown,
  isFocused,
}: ThumbnailItemProps) {
  const itemRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (isFocused && itemRef.current) {
      itemRef.current.focus();
    }
  }, [isFocused]);

  return (
    <button
      ref={itemRef}
      className={`thumbnail-item ${isSelected ? 'selected' : ''}`}
      onClick={onClick}
      onKeyDown={onKeyDown}
      aria-label={`Page ${pageNumber}${isSelected ? ', currently selected' : ''}`}
      aria-current={isSelected ? 'page' : undefined}
      role="option"
      aria-selected={isSelected}
      tabIndex={isSelected ? 0 : -1}
      type="button"
    >
      <div className="thumbnail-preview">
        {isLoading ? (
          <div className="thumbnail-skeleton" aria-hidden="true">
            <div className="skeleton-line skeleton-title" />
            <div className="skeleton-line skeleton-text" />
            <div className="skeleton-line skeleton-text" />
            <div className="skeleton-line skeleton-text short" />
            <div className="skeleton-line skeleton-text" />
            <div className="skeleton-line skeleton-text" />
          </div>
        ) : thumbnailSrc ? (
          <img
            src={thumbnailSrc}
            alt={`Preview of page ${pageNumber}`}
            className="thumbnail-image"
            loading="lazy"
          />
        ) : (
          <div className="thumbnail-placeholder" aria-hidden="true">
            <span className="placeholder-page-number">{pageNumber}</span>
          </div>
        )}
      </div>
      <span className="thumbnail-label">{pageNumber}</span>
    </button>
  );
}

// =============================================================================
// PageThumbnails Component
// =============================================================================

export function PageThumbnails({
  totalPages,
  currentPage,
  onPageSelect,
  thumbnailData = [],
  isLoading = false,
}: PageThumbnailsProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [containerHeight, setContainerHeight] = useState(0);
  const [focusedPage, setFocusedPage] = useState<number | null>(null);

  // Calculate visible range for virtual scrolling
  const { visibleItems } = useMemo(() => {
    const start = Math.max(0, Math.floor(scrollTop / THUMBNAIL_ITEM_HEIGHT) - BUFFER_SIZE);
    const visibleCount = Math.ceil(containerHeight / THUMBNAIL_ITEM_HEIGHT);
    const end = Math.min(totalPages - 1, start + visibleCount + BUFFER_SIZE * 2);

    const items: number[] = [];
    for (let i = start; i <= end; i++) {
      items.push(i + 1); // Convert to 1-indexed page numbers
    }

    return {
      startIndex: start,
      endIndex: end,
      visibleItems: items,
    };
  }, [scrollTop, containerHeight, totalPages]);

  // Handle container scroll
  const handleScroll = useCallback(() => {
    if (containerRef.current) {
      setScrollTop(containerRef.current.scrollTop);
    }
  }, []);

  // Update container height on resize
  useEffect(() => {
    const updateHeight = () => {
      if (containerRef.current) {
        setContainerHeight(containerRef.current.clientHeight);
      }
    };

    updateHeight();

    const resizeObserver = new ResizeObserver(updateHeight);
    if (containerRef.current) {
      resizeObserver.observe(containerRef.current);
    }

    return () => resizeObserver.disconnect();
  }, []);

  // Scroll to current page when it changes
  useEffect(() => {
    if (containerRef.current && currentPage >= 1 && currentPage <= totalPages) {
      const targetScrollTop = (currentPage - 1) * THUMBNAIL_ITEM_HEIGHT;
      const currentScrollTop = containerRef.current.scrollTop;
      const viewportHeight = containerRef.current.clientHeight;

      // Only scroll if the current page is not visible
      if (
        targetScrollTop < currentScrollTop ||
        targetScrollTop + THUMBNAIL_ITEM_HEIGHT > currentScrollTop + viewportHeight
      ) {
        containerRef.current.scrollTo({
          top: targetScrollTop - viewportHeight / 2 + THUMBNAIL_ITEM_HEIGHT / 2,
          behavior: 'smooth',
        });
      }
    }
  }, [currentPage, totalPages]);

  // Handle page selection
  const handlePageClick = useCallback(
    (pageNumber: number) => {
      onPageSelect(pageNumber);
      setFocusedPage(pageNumber);
    },
    [onPageSelect]
  );

  // Handle keyboard navigation
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent, pageNumber: number) => {
      let newPage: number | null = null;

      switch (e.key) {
        case 'ArrowUp':
          e.preventDefault();
          newPage = Math.max(1, pageNumber - 1);
          break;
        case 'ArrowDown':
          e.preventDefault();
          newPage = Math.min(totalPages, pageNumber + 1);
          break;
        case 'Home':
          e.preventDefault();
          newPage = 1;
          break;
        case 'End':
          e.preventDefault();
          newPage = totalPages;
          break;
        case 'PageUp':
          e.preventDefault();
          newPage = Math.max(1, pageNumber - 5);
          break;
        case 'PageDown':
          e.preventDefault();
          newPage = Math.min(totalPages, pageNumber + 5);
          break;
        case 'Enter':
        case ' ':
          e.preventDefault();
          onPageSelect(pageNumber);
          return;
      }

      if (newPage !== null) {
        setFocusedPage(newPage);
        onPageSelect(newPage);
      }
    },
    [totalPages, onPageSelect]
  );

  // Handle container keyboard navigation when no item is focused
  const handleContainerKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.target === containerRef.current) {
        if (e.key === 'ArrowDown' || e.key === 'ArrowUp' || e.key === 'Home' || e.key === 'End') {
          e.preventDefault();
          const targetPage = e.key === 'Home' ? 1 : e.key === 'End' ? totalPages : currentPage;
          setFocusedPage(targetPage);
          onPageSelect(targetPage);
        }
      }
    },
    [currentPage, totalPages, onPageSelect]
  );

  // Total height for virtual scrolling
  const totalHeight = totalPages * THUMBNAIL_ITEM_HEIGHT;

  // Get thumbnail source for a page
  const getThumbnailSrc = useCallback(
    (pageNumber: number): string | undefined => {
      const index = pageNumber - 1;
      if (thumbnailData && thumbnailData[index]) {
        // Handle both raw base64 and data URL formats
        const data = thumbnailData[index];
        if (data.startsWith('data:')) {
          return data;
        }
        return `data:image/png;base64,${data}`;
      }
      return undefined;
    },
    [thumbnailData]
  );

  // Check if a specific page thumbnail is loading
  const isThumbnailLoading = useCallback(
    (pageNumber: number): boolean => {
      if (isLoading) return true;
      const index = pageNumber - 1;
      return !thumbnailData || !thumbnailData[index];
    },
    [isLoading, thumbnailData]
  );

  if (totalPages === 0) {
    return (
      <div className="page-thumbnails empty" role="region" aria-label="Page thumbnails">
        <div className="empty-message">No pages</div>
      </div>
    );
  }

  return (
    <div
      className="page-thumbnails"
      role="region"
      aria-label="Page thumbnails navigation"
    >
      <div className="thumbnails-header">
        <h3 className="thumbnails-title">Pages</h3>
        <span className="thumbnails-count">
          {currentPage} of {totalPages}
        </span>
      </div>

      <div
        ref={containerRef}
        className="thumbnails-container"
        onScroll={handleScroll}
        onKeyDown={handleContainerKeyDown}
        role="listbox"
        aria-label={`Document pages, ${totalPages} total`}
        aria-activedescendant={focusedPage ? `thumbnail-${focusedPage}` : undefined}
        tabIndex={0}
      >
        <div
          className="thumbnails-scroll-content"
          style={{ height: totalHeight }}
        >
          {visibleItems.map((pageNumber) => (
            <div
              key={pageNumber}
              className="thumbnail-wrapper"
              style={{
                position: 'absolute',
                top: (pageNumber - 1) * THUMBNAIL_ITEM_HEIGHT,
                left: 0,
                right: 0,
              }}
              id={`thumbnail-${pageNumber}`}
            >
              <ThumbnailItem
                pageNumber={pageNumber}
                isSelected={pageNumber === currentPage}
                thumbnailSrc={getThumbnailSrc(pageNumber)}
                isLoading={isThumbnailLoading(pageNumber)}
                onClick={() => handlePageClick(pageNumber)}
                onKeyDown={(e) => handleKeyDown(e, pageNumber)}
                isFocused={focusedPage === pageNumber}
              />
            </div>
          ))}
        </div>
      </div>

      {isLoading && (
        <div className="loading-indicator" role="status" aria-live="polite">
          <span className="loading-spinner" aria-hidden="true" />
          <span className="loading-text">Generating thumbnails...</span>
        </div>
      )}
    </div>
  );
}

export default PageThumbnails;
