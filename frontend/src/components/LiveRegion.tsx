/**
 * LiveRegion.tsx
 *
 * Provides ARIA live regions for dynamic announcements to screen readers.
 * Implements different priority levels and announcement types.
 */

import React, { useCallback, useRef, useEffect, createContext, useContext, useState } from 'react';

// =============================================================================
// Types
// =============================================================================

export type AnnouncementPriority = 'polite' | 'assertive';

export type AnnouncementType =
  | 'cursor'
  | 'selection'
  | 'formatting'
  | 'document'
  | 'error'
  | 'status'
  | 'navigation';

export interface Announcement {
  id: string;
  message: string;
  priority: AnnouncementPriority;
  type: AnnouncementType;
  timestamp: number;
}

export interface LiveRegionContextValue {
  announce: (message: string, priority?: AnnouncementPriority, type?: AnnouncementType) => void;
  announceCursorPosition: (line: number, column: number, extras?: string[]) => void;
  announceSelection: (text: string, wordCount: number, characterCount: number) => void;
  announceFormatting: (format: string, applied: boolean) => void;
  announceDocumentStatus: (status: string) => void;
  announcePageChange: (currentPage: number, totalPages: number) => void;
  announceError: (error: string) => void;
  announceNavigation: (target: string) => void;
  clearAnnouncements: () => void;
}

// =============================================================================
// Context
// =============================================================================

const LiveRegionContext = createContext<LiveRegionContextValue | null>(null);

export function useLiveRegion(): LiveRegionContextValue {
  const context = useContext(LiveRegionContext);
  if (!context) {
    throw new Error('useLiveRegion must be used within a LiveRegionProvider');
  }
  return context;
}

// =============================================================================
// Provider Component
// =============================================================================

interface LiveRegionProviderProps {
  children: React.ReactNode;
  /** Debounce delay in ms for cursor/selection announcements */
  debounceDelay?: number;
  /** Maximum number of announcements to keep in history */
  maxHistory?: number;
}

export function LiveRegionProvider({
  children,
  debounceDelay = 300,
  maxHistory = 50,
}: LiveRegionProviderProps) {
  const [politeMessage, setPoliteMessage] = useState('');
  const [assertiveMessage, setAssertiveMessage] = useState('');
  const [history, setHistory] = useState<Announcement[]>([]);

  const debounceTimerRef = useRef<number | null>(null);
  const lastCursorAnnouncementRef = useRef('');
  const announcementIdRef = useRef(0);

  /**
   * Generate unique announcement ID
   */
  const generateId = useCallback(() => {
    announcementIdRef.current += 1;
    return `announcement-${announcementIdRef.current}`;
  }, []);

  /**
   * Add announcement to history
   */
  const addToHistory = useCallback((announcement: Announcement) => {
    setHistory(prev => {
      const updated = [...prev, announcement];
      if (updated.length > maxHistory) {
        return updated.slice(-maxHistory);
      }
      return updated;
    });
  }, [maxHistory]);

  /**
   * Core announce function
   */
  const announce = useCallback((
    message: string,
    priority: AnnouncementPriority = 'polite',
    type: AnnouncementType = 'status'
  ) => {
    const announcement: Announcement = {
      id: generateId(),
      message,
      priority,
      type,
      timestamp: Date.now(),
    };

    addToHistory(announcement);

    // Clear and set message to force re-announcement
    if (priority === 'assertive') {
      setAssertiveMessage('');
      requestAnimationFrame(() => {
        setAssertiveMessage(message);
      });
    } else {
      setPoliteMessage('');
      requestAnimationFrame(() => {
        setPoliteMessage(message);
      });
    }
  }, [generateId, addToHistory]);

  /**
   * Announce with debouncing (for rapid updates like cursor movement)
   */
  const announceDebounced = useCallback((
    message: string,
    priority: AnnouncementPriority = 'polite',
    type: AnnouncementType = 'status'
  ) => {
    if (debounceTimerRef.current !== null) {
      window.clearTimeout(debounceTimerRef.current);
    }

    debounceTimerRef.current = window.setTimeout(() => {
      announce(message, priority, type);
      debounceTimerRef.current = null;
    }, debounceDelay);
  }, [announce, debounceDelay]);

  /**
   * Announce cursor position
   */
  const announceCursorPosition = useCallback((
    line: number,
    column: number,
    extras: string[] = []
  ) => {
    const parts = [`Line ${line}, Column ${column}`];
    parts.push(...extras.filter(Boolean));
    const message = parts.join(', ');

    // Only announce if changed
    if (message !== lastCursorAnnouncementRef.current) {
      lastCursorAnnouncementRef.current = message;
      announceDebounced(message, 'polite', 'cursor');
    }
  }, [announceDebounced]);

  /**
   * Announce selection change
   */
  const announceSelection = useCallback((
    text: string,
    wordCount: number,
    characterCount: number
  ) => {
    let message: string;

    if (!text) {
      // No selection
      return;
    }

    if (text.length <= 50) {
      message = `Selected: ${text}`;
    } else {
      message = `Selection: ${wordCount} word${wordCount !== 1 ? 's' : ''}, ${characterCount} character${characterCount !== 1 ? 's' : ''}`;
    }

    announce(message, 'polite', 'selection');
  }, [announce]);

  /**
   * Announce formatting change
   */
  const announceFormatting = useCallback((format: string, applied: boolean) => {
    const action = applied ? 'applied' : 'removed';
    announce(`${format} ${action}`, 'assertive', 'formatting');
  }, [announce]);

  /**
   * Announce document status
   */
  const announceDocumentStatus = useCallback((status: string) => {
    announce(status, 'polite', 'document');
  }, [announce]);

  /**
   * Announce page change
   */
  const announcePageChange = useCallback((currentPage: number, totalPages: number) => {
    announce(`Page ${currentPage} of ${totalPages}`, 'polite', 'navigation');
  }, [announce]);

  /**
   * Announce error
   */
  const announceError = useCallback((error: string) => {
    announce(`Error: ${error}`, 'assertive', 'error');
  }, [announce]);

  /**
   * Announce navigation
   */
  const announceNavigation = useCallback((target: string) => {
    announce(`Navigated to ${target}`, 'polite', 'navigation');
  }, [announce]);

  /**
   * Clear announcements
   */
  const clearAnnouncements = useCallback(() => {
    setPoliteMessage('');
    setAssertiveMessage('');
  }, []);

  // Cleanup debounce timer on unmount
  useEffect(() => {
    return () => {
      if (debounceTimerRef.current !== null) {
        window.clearTimeout(debounceTimerRef.current);
      }
    };
  }, []);

  const contextValue: LiveRegionContextValue = {
    announce,
    announceCursorPosition,
    announceSelection,
    announceFormatting,
    announceDocumentStatus,
    announcePageChange,
    announceError,
    announceNavigation,
    clearAnnouncements,
  };

  return (
    <LiveRegionContext.Provider value={contextValue}>
      {children}
      <LiveRegionOutput politeMessage={politeMessage} assertiveMessage={assertiveMessage} />
    </LiveRegionContext.Provider>
  );
}

// =============================================================================
// Live Region Output Component
// =============================================================================

interface LiveRegionOutputProps {
  politeMessage: string;
  assertiveMessage: string;
}

function LiveRegionOutput({ politeMessage, assertiveMessage }: LiveRegionOutputProps) {
  const visuallyHiddenStyle: React.CSSProperties = {
    position: 'absolute',
    width: '1px',
    height: '1px',
    padding: 0,
    margin: '-1px',
    overflow: 'hidden',
    clip: 'rect(0, 0, 0, 0)',
    whiteSpace: 'nowrap',
    border: 0,
  };

  return (
    <>
      {/* Polite live region - for non-urgent updates */}
      <div
        role="status"
        aria-live="polite"
        aria-atomic="true"
        aria-relevant="additions text"
        style={visuallyHiddenStyle}
        data-testid="live-region-polite"
      >
        {politeMessage}
      </div>

      {/* Assertive live region - for urgent updates */}
      <div
        role="alert"
        aria-live="assertive"
        aria-atomic="true"
        aria-relevant="additions text"
        style={visuallyHiddenStyle}
        data-testid="live-region-assertive"
      >
        {assertiveMessage}
      </div>
    </>
  );
}

// =============================================================================
// SkipLinks Component
// =============================================================================

interface SkipLinksProps {
  /** Custom skip links */
  links?: Array<{ label: string; targetId: string }>;
}

export function SkipLinks({ links }: SkipLinksProps) {
  const defaultLinks = [
    { label: 'Skip to main content', targetId: 'main-content' },
    { label: 'Skip to toolbar', targetId: 'main-toolbar' },
  ];

  const allLinks = links ?? defaultLinks;

  const handleSkip = (targetId: string) => (e: React.MouseEvent | React.KeyboardEvent) => {
    e.preventDefault();
    const target = document.getElementById(targetId);
    if (target) {
      target.focus();
      target.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  };

  return (
    <nav className="skip-links" aria-label="Skip navigation">
      {allLinks.map(link => (
        <a
          key={link.targetId}
          href={`#${link.targetId}`}
          className="skip-link"
          onClick={handleSkip(link.targetId)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              handleSkip(link.targetId)(e);
            }
          }}
        >
          {link.label}
        </a>
      ))}
    </nav>
  );
}

// =============================================================================
// Formatted Announcement Helpers
// =============================================================================

/**
 * Format cursor position for announcement
 */
export function formatCursorAnnouncement(
  line: number,
  column: number,
  options?: {
    paragraphStyle?: string;
    isBold?: boolean;
    isItalic?: boolean;
    isUnderline?: boolean;
    fontSize?: number;
    fontFamily?: string;
  }
): string[] {
  const extras: string[] = [];

  if (options) {
    if (options.paragraphStyle && options.paragraphStyle !== 'Normal') {
      extras.push(options.paragraphStyle);
    }

    const formats: string[] = [];
    if (options.isBold) formats.push('Bold');
    if (options.isItalic) formats.push('Italic');
    if (options.isUnderline) formats.push('Underline');
    if (formats.length > 0) {
      extras.push(formats.join(', '));
    }

    if (options.fontSize) {
      extras.push(`${options.fontSize} points`);
    }
  }

  return extras;
}

/**
 * Count words in text
 */
export function countWords(text: string): number {
  if (!text.trim()) return 0;
  return text.trim().split(/\s+/).length;
}

/**
 * Format selection info for announcement
 */
export function formatSelectionAnnouncement(
  text: string,
  maxPreviewLength: number = 50
): { preview: string; wordCount: number; characterCount: number } {
  const wordCount = countWords(text);
  const characterCount = text.length;

  let preview = text;
  if (text.length > maxPreviewLength) {
    preview = text.substring(0, maxPreviewLength) + '...';
  }

  return { preview, wordCount, characterCount };
}

// =============================================================================
// Export Utilities
// =============================================================================

export { LiveRegionContext };
