/**
 * useDocumentStats - Document statistics tracking hook
 *
 * Features:
 * - Track page count (from layout engine)
 * - Track word count (count words in all text runs)
 * - Track character count (with and without spaces)
 * - Track paragraph count
 * - Track selection statistics (when text selected)
 * - Debounced updates (don't recalculate on every keystroke)
 */

import { useState, useCallback, useEffect, useMemo, useRef } from 'react';
import { RenderModel, Selection } from '../lib/types';

// =============================================================================
// Types
// =============================================================================

export interface DocumentStats {
  /** Total page count */
  pageCount: number;
  /** Total word count */
  wordCount: number;
  /** Total character count including spaces */
  characterCount: number;
  /** Total character count excluding spaces */
  characterCountNoSpaces: number;
  /** Total paragraph count */
  paragraphCount: number;
  /** Total line count (approximate) */
  lineCount: number;
  /** Estimated reading time in minutes */
  readingTimeMinutes: number;
}

export interface SelectionStats {
  /** Selected word count */
  wordCount: number;
  /** Selected character count including spaces */
  characterCount: number;
  /** Selected character count excluding spaces */
  characterCountNoSpaces: number;
  /** Number of paragraphs in selection */
  paragraphCount: number;
}

export interface UseDocumentStatsOptions {
  /** Debounce delay in milliseconds (default: 300) */
  debounceDelay?: number;
  /** Average words per minute for reading time calculation (default: 200) */
  wordsPerMinute?: number;
  /** Render model to calculate stats from */
  renderModel?: RenderModel | null;
  /** Current selection */
  selection?: Selection | null;
}

export interface UseDocumentStatsReturn {
  /** Document-wide statistics */
  documentStats: DocumentStats;
  /** Selection statistics (null if no selection) */
  selectionStats: SelectionStats | null;
  /** Whether stats are currently being calculated */
  isCalculating: boolean;
  /** Force recalculation of stats */
  recalculate: () => void;
}

// =============================================================================
// Constants
// =============================================================================

const DEFAULT_DEBOUNCE_DELAY = 300;
const DEFAULT_WORDS_PER_MINUTE = 200;
const AVERAGE_CHARS_PER_LINE = 80; // Rough estimate for line count

// =============================================================================
// Helper Functions
// =============================================================================

/**
 * Count words in a text string
 */
function countWords(text: string): number {
  if (!text || !text.trim()) return 0;
  // Split on whitespace and filter out empty strings
  const words = text.trim().split(/\s+/).filter(word => word.length > 0);
  return words.length;
}

/**
 * Count characters in a text string
 */
function countCharacters(text: string, includeSpaces: boolean): number {
  if (!text) return 0;
  if (includeSpaces) {
    return text.length;
  }
  return text.replace(/\s/g, '').length;
}

/**
 * Extract all text from a render model
 */
function extractTextFromRenderModel(renderModel: RenderModel | null): string {
  if (!renderModel || !renderModel.pages) return '';

  const textParts: string[] = [];

  for (const page of renderModel.pages) {
    if (!page.items) continue;

    for (const item of page.items) {
      if (item.type === 'GlyphRun') {
        textParts.push(item.text);
      }
    }
  }

  return textParts.join(' ');
}

/**
 * Estimate paragraph count from render model
 * This is an approximation based on text runs and line breaks
 */
function estimateParagraphCount(renderModel: RenderModel | null): number {
  if (!renderModel || !renderModel.pages) return 0;

  // Count distinct Y positions (rough paragraph estimation)
  const yPositions = new Set<number>();
  let lastY = -1000;
  const lineHeightThreshold = 20; // Assume paragraphs are separated by more than this

  for (const page of renderModel.pages) {
    if (!page.items) continue;

    for (const item of page.items) {
      if (item.type === 'GlyphRun') {
        // If Y position has jumped significantly, it's likely a new paragraph
        if (Math.abs(item.y - lastY) > lineHeightThreshold) {
          yPositions.add(Math.round(item.y / lineHeightThreshold) * lineHeightThreshold);
        }
        lastY = item.y;
      }
    }
  }

  return Math.max(1, yPositions.size);
}

/**
 * Estimate line count from render model
 */
function estimateLineCount(renderModel: RenderModel | null): number {
  if (!renderModel || !renderModel.pages) return 0;

  const yPositions = new Set<string>();

  for (const page of renderModel.pages) {
    if (!page.items) continue;

    for (const item of page.items) {
      if (item.type === 'GlyphRun') {
        // Create unique key for page and Y position
        const key = `${page.page_index}-${Math.round(item.y)}`;
        yPositions.add(key);
      }
    }
  }

  return Math.max(1, yPositions.size);
}

/**
 * Debounce a function
 */
function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return debouncedValue;
}

// =============================================================================
// Hook Implementation
// =============================================================================

export function useDocumentStats(options: UseDocumentStatsOptions = {}): UseDocumentStatsReturn {
  const {
    debounceDelay = DEFAULT_DEBOUNCE_DELAY,
    wordsPerMinute = DEFAULT_WORDS_PER_MINUTE,
    renderModel = null,
    selection = null,
  } = options;

  const [isCalculating, setIsCalculating] = useState(false);
  const calculationIdRef = useRef(0);

  // Debounce the render model to avoid recalculating on every keystroke
  const debouncedRenderModel = useDebounce(renderModel, debounceDelay);

  // Calculate document statistics
  const documentStats = useMemo<DocumentStats>(() => {
    if (!debouncedRenderModel) {
      return {
        pageCount: 0,
        wordCount: 0,
        characterCount: 0,
        characterCountNoSpaces: 0,
        paragraphCount: 0,
        lineCount: 0,
        readingTimeMinutes: 0,
      };
    }

    const text = extractTextFromRenderModel(debouncedRenderModel);
    const wordCount = countWords(text);
    const characterCount = countCharacters(text, true);
    const characterCountNoSpaces = countCharacters(text, false);
    const paragraphCount = estimateParagraphCount(debouncedRenderModel);
    const lineCount = estimateLineCount(debouncedRenderModel);
    const readingTimeMinutes = Math.ceil(wordCount / wordsPerMinute);

    return {
      pageCount: debouncedRenderModel.pages?.length ?? 0,
      wordCount,
      characterCount,
      characterCountNoSpaces,
      paragraphCount,
      lineCount,
      readingTimeMinutes,
    };
  }, [debouncedRenderModel, wordsPerMinute]);

  // Calculate selection statistics
  const selectionStats = useMemo<SelectionStats | null>(() => {
    if (!selection || !debouncedRenderModel) return null;

    // Check if there's actually a selection (not just a cursor)
    const hasSelection =
      selection.anchor.nodeId !== selection.focus.nodeId ||
      selection.anchor.offset !== selection.focus.offset;

    if (!hasSelection) return null;

    // In a real implementation, we would get the selected text from the backend
    // For now, we'll return placeholder values
    // TODO: Implement proper selection text extraction

    return {
      wordCount: 0,
      characterCount: 0,
      characterCountNoSpaces: 0,
      paragraphCount: 0,
    };
  }, [selection, debouncedRenderModel]);

  // Force recalculation
  const recalculate = useCallback(() => {
    calculationIdRef.current += 1;
    setIsCalculating(true);

    // Simulate async calculation (in real impl, this might call backend)
    setTimeout(() => {
      setIsCalculating(false);
    }, 50);
  }, []);

  // Update calculating state based on debounce
  useEffect(() => {
    if (renderModel !== debouncedRenderModel) {
      setIsCalculating(true);
    } else {
      setIsCalculating(false);
    }
  }, [renderModel, debouncedRenderModel]);

  return {
    documentStats,
    selectionStats,
    isCalculating,
    recalculate,
  };
}

// =============================================================================
// Utility Hooks
// =============================================================================

/**
 * Format reading time for display
 */
export function formatReadingTime(minutes: number): string {
  if (minutes < 1) {
    return 'Less than 1 min read';
  }
  if (minutes === 1) {
    return '1 min read';
  }
  if (minutes < 60) {
    return `${minutes} min read`;
  }
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  if (remainingMinutes === 0) {
    return hours === 1 ? '1 hour read' : `${hours} hours read`;
  }
  return `${hours}h ${remainingMinutes}m read`;
}

/**
 * Format number with locale-specific separators
 */
export function formatNumber(value: number, locale: string = 'en-US'): string {
  return value.toLocaleString(locale);
}

export default useDocumentStats;
