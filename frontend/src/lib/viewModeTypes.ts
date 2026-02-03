/**
 * View Mode Types
 *
 * Types for the document view mode system including:
 * - PrintLayout: Shows pages as they will print (default)
 * - Draft: Continuous scroll without page breaks for fast editing
 * - Outline: Hierarchical heading view for document navigation
 * - WebLayout: For HTML export preview
 */

// =============================================================================
// View Mode Enum
// =============================================================================

/**
 * Available view modes for the document editor
 */
export type ViewMode = 'print_layout' | 'draft' | 'outline' | 'web_layout';

/**
 * View mode display information
 */
export interface ViewModeInfo {
  mode: ViewMode;
  displayName: string;
  shortcut: string;
  showsPageBreaks: boolean;
  isContinuous: boolean;
  icon: 'print' | 'draft' | 'outline' | 'web';
  description: string;
}

/**
 * View mode definitions
 */
export const VIEW_MODE_INFO: Record<ViewMode, ViewModeInfo> = {
  print_layout: {
    mode: 'print_layout',
    displayName: 'Print Layout',
    shortcut: 'Ctrl+Alt+P',
    showsPageBreaks: true,
    isContinuous: false,
    icon: 'print',
    description: 'Shows the document as it will appear when printed, with page breaks and margins.',
  },
  draft: {
    mode: 'draft',
    displayName: 'Draft',
    shortcut: 'Ctrl+Alt+N',
    showsPageBreaks: false,
    isContinuous: true,
    icon: 'draft',
    description: 'Simplified view for fast editing. No page breaks, optional style names in margin.',
  },
  outline: {
    mode: 'outline',
    displayName: 'Outline',
    shortcut: 'Ctrl+Alt+O',
    showsPageBreaks: false,
    isContinuous: false,
    icon: 'outline',
    description: 'Hierarchical view of document headings. Useful for reorganizing content.',
  },
  web_layout: {
    mode: 'web_layout',
    displayName: 'Web Layout',
    shortcut: 'Ctrl+Alt+W',
    showsPageBreaks: false,
    isContinuous: true,
    icon: 'web',
    description: 'Shows the document as it would appear in a web browser.',
  },
};

// =============================================================================
// Draft View Options
// =============================================================================

/**
 * Options for Draft view mode
 */
export interface DraftViewOptions {
  /** Show style names in left margin */
  showStyleNames: boolean;
  /** Show images (false = show placeholders for speed) */
  showImages: boolean;
  /** Wrap text to window width */
  wrapToWindow: boolean;
  /** Show paragraph markers */
  showParagraphMarks: boolean;
  /** Line spacing multiplier */
  lineSpacingMultiplier: number;
  /** Left margin for style names (in points) */
  styleNameMargin: number;
}

/**
 * Default draft view options
 */
export const DEFAULT_DRAFT_OPTIONS: DraftViewOptions = {
  showStyleNames: false,
  showImages: false,
  wrapToWindow: true,
  showParagraphMarks: false,
  lineSpacingMultiplier: 1.0,
  styleNameMargin: 100,
};

/**
 * Fast editing draft options (maximum speed)
 */
export const FAST_DRAFT_OPTIONS: DraftViewOptions = {
  showStyleNames: false,
  showImages: false,
  wrapToWindow: true,
  showParagraphMarks: false,
  lineSpacingMultiplier: 1.0,
  styleNameMargin: 0,
};

// =============================================================================
// Outline View Options
// =============================================================================

/**
 * Options for Outline view mode
 */
export interface OutlineViewOptions {
  /** Start of heading levels to show (1-6) */
  showLevelsStart: number;
  /** End of heading levels to show (2-7, exclusive) */
  showLevelsEnd: number;
  /** Show body text under headings */
  showBodyText: boolean;
  /** Show only first line of body text */
  showFirstLineOnly: boolean;
  /** Show heading level indicators */
  showLevelIndicators: boolean;
  /** Allow drag-and-drop reordering */
  enableDragDrop: boolean;
  /** Indent per level in pixels */
  indentPerLevel: number;
}

/**
 * Default outline view options
 */
export const DEFAULT_OUTLINE_OPTIONS: OutlineViewOptions = {
  showLevelsStart: 1,
  showLevelsEnd: 7, // Show all levels by default
  showBodyText: false,
  showFirstLineOnly: true,
  showLevelIndicators: true,
  enableDragDrop: true,
  indentPerLevel: 20,
};

/**
 * Compact outline options (top levels only)
 */
export const COMPACT_OUTLINE_OPTIONS: OutlineViewOptions = {
  showLevelsStart: 1,
  showLevelsEnd: 4, // Show H1-H3 only
  showBodyText: false,
  showFirstLineOnly: false,
  showLevelIndicators: true,
  enableDragDrop: true,
  indentPerLevel: 16,
};

// =============================================================================
// View Mode Configuration
// =============================================================================

/**
 * Complete view mode configuration
 */
export interface ViewModeConfig {
  /** Current view mode */
  mode: ViewModeInfo;
  /** Draft view options */
  draftOptions: DraftViewOptions;
  /** Outline view options */
  outlineOptions: OutlineViewOptions;
}

/**
 * Default view mode configuration
 */
export const DEFAULT_VIEW_MODE_CONFIG: ViewModeConfig = {
  mode: VIEW_MODE_INFO.print_layout,
  draftOptions: DEFAULT_DRAFT_OPTIONS,
  outlineOptions: DEFAULT_OUTLINE_OPTIONS,
};

// =============================================================================
// Draft Layout Types
// =============================================================================

/**
 * A block in draft layout
 */
export interface DraftBlock {
  /** Node ID in document */
  nodeId: string;
  /** X position */
  x: number;
  /** Y position */
  y: number;
  /** Width */
  width: number;
  /** Height */
  height: number;
  /** Style name (if showStyleNames is enabled) */
  styleName?: string;
  /** Whether this is a heading */
  isHeading: boolean;
  /** Heading level (1-6) if isHeading */
  headingLevel?: number;
}

/**
 * Draft layout (continuous, no page breaks)
 */
export interface DraftLayout {
  /** All blocks in document order */
  blocks: DraftBlock[];
  /** Total content height */
  totalHeight: number;
  /** Content width */
  contentWidth: number;
}

// =============================================================================
// Outline Types
// =============================================================================

/**
 * A heading in the outline view
 */
export interface OutlineHeadingItem {
  /** Unique identifier */
  id: string;
  /** Node ID in document */
  nodeId: string;
  /** Heading level (1-6) */
  level: number;
  /** Heading text */
  text: string;
  /** Child headings */
  children: OutlineHeadingItem[];
  /** First line of body text (if showBodyText is enabled) */
  bodyPreview?: string;
  /** Character offset in document */
  offset: number;
  /** Whether this heading is expanded in the view */
  expanded: boolean;
}

/**
 * Complete outline data for a document
 */
export interface OutlineData {
  /** Root-level headings */
  headings: OutlineHeadingItem[];
  /** Total heading count */
  totalCount: number;
}

// =============================================================================
// Helper Functions
// =============================================================================

/**
 * Get heading levels to show as array
 */
export function getShowLevelsArray(options: OutlineViewOptions): number[] {
  const levels: number[] = [];
  for (let i = options.showLevelsStart; i < options.showLevelsEnd && i <= 6; i++) {
    levels.push(i);
  }
  return levels;
}

/**
 * Check if a heading level should be shown
 */
export function shouldShowLevel(level: number, options: OutlineViewOptions): boolean {
  return level >= options.showLevelsStart && level < options.showLevelsEnd;
}

/**
 * Get view mode from legacy format
 */
export function parseViewMode(mode: string): ViewMode {
  switch (mode.toLowerCase().replace(/[-_]/g, '')) {
    case 'printlayout':
      return 'print_layout';
    case 'draft':
      return 'draft';
    case 'outline':
      return 'outline';
    case 'weblayout':
      return 'web_layout';
    default:
      return 'print_layout';
  }
}

/**
 * Get keyboard shortcut for platform
 */
export function getShortcutForPlatform(mode: ViewMode): string {
  const isMac = typeof navigator !== 'undefined' && navigator.platform.toUpperCase().indexOf('MAC') >= 0;
  const modifier = isMac ? 'Cmd' : 'Ctrl';

  switch (mode) {
    case 'print_layout':
      return `${modifier}+Alt+P`;
    case 'draft':
      return `${modifier}+Alt+N`;
    case 'outline':
      return `${modifier}+Alt+O`;
    case 'web_layout':
      return `${modifier}+Alt+W`;
  }
}

/**
 * Count total headings in outline
 */
export function countOutlineHeadings(headings: OutlineHeadingItem[]): number {
  let count = headings.length;
  for (const heading of headings) {
    count += countOutlineHeadings(heading.children);
  }
  return count;
}

/**
 * Flatten outline headings to a list
 */
export function flattenOutlineHeadings(
  headings: OutlineHeadingItem[],
  expandedIds: Set<string>
): OutlineHeadingItem[] {
  const result: OutlineHeadingItem[] = [];

  function traverse(items: OutlineHeadingItem[]) {
    for (const heading of items) {
      result.push(heading);
      if (heading.children.length > 0 && expandedIds.has(heading.id)) {
        traverse(heading.children);
      }
    }
  }

  traverse(headings);
  return result;
}
