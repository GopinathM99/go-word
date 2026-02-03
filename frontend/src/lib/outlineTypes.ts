/**
 * Document Outline Types
 *
 * Types for the document outline panel that displays
 * a hierarchical view of document headings.
 */

// =============================================================================
// Outline Heading Types
// =============================================================================

/**
 * Position of a heading in the document
 */
export interface HeadingPosition {
  /** Page number (1-indexed) */
  page: number;
  /** Character offset within the page/document */
  offset: number;
}

/**
 * A heading in the document outline
 */
export interface OutlineHeading {
  /** Unique identifier for this heading */
  id: string;
  /** Heading level (1-6, corresponding to H1-H6) */
  level: number;
  /** Text content of the heading */
  text: string;
  /** Child headings (for hierarchical display) */
  children: OutlineHeading[];
  /** Position in the document */
  position: HeadingPosition;
}

/**
 * State for the outline panel
 */
export interface OutlineState {
  /** Root level headings (with nested children) */
  headings: OutlineHeading[];
  /** Set of heading IDs that are expanded */
  expandedIds: Set<string>;
  /** Currently active/highlighted heading ID */
  currentHeadingId: string | null;
}

// =============================================================================
// Backend Response Types
// =============================================================================

/**
 * Raw heading from backend (flat structure)
 */
export interface RawOutlineHeading {
  id: string;
  level: number;
  text: string;
  page: number;
  offset: number;
}

/**
 * Response from get_document_outline backend command
 */
export interface DocumentOutlineResponse {
  /** Document ID */
  docId: string;
  /** Flat list of headings */
  headings: RawOutlineHeading[];
}

// =============================================================================
// Helper Functions
// =============================================================================

/**
 * Build a hierarchical tree from a flat list of headings
 */
export function buildHeadingTree(flatHeadings: RawOutlineHeading[]): OutlineHeading[] {
  if (flatHeadings.length === 0) {
    return [];
  }

  const result: OutlineHeading[] = [];
  const stack: OutlineHeading[] = [];

  for (const raw of flatHeadings) {
    const heading: OutlineHeading = {
      id: raw.id,
      level: raw.level,
      text: raw.text,
      children: [],
      position: {
        page: raw.page,
        offset: raw.offset,
      },
    };

    // Pop items from stack until we find a parent with lower level
    while (stack.length > 0 && stack[stack.length - 1].level >= heading.level) {
      stack.pop();
    }

    if (stack.length === 0) {
      // No parent, add to root
      result.push(heading);
    } else {
      // Add as child of the last item in stack
      stack[stack.length - 1].children.push(heading);
    }

    // Push current heading to stack
    stack.push(heading);
  }

  return result;
}

/**
 * Find a heading by ID in the tree
 */
export function findHeadingById(
  headings: OutlineHeading[],
  id: string
): OutlineHeading | null {
  for (const heading of headings) {
    if (heading.id === id) {
      return heading;
    }
    const found = findHeadingById(heading.children, id);
    if (found) {
      return found;
    }
  }
  return null;
}

/**
 * Get all heading IDs from the tree (flattened)
 */
export function getAllHeadingIds(headings: OutlineHeading[]): string[] {
  const ids: string[] = [];

  function traverse(items: OutlineHeading[]) {
    for (const heading of items) {
      ids.push(heading.id);
      traverse(heading.children);
    }
  }

  traverse(headings);
  return ids;
}

/**
 * Get all parent IDs for a given heading ID
 */
export function getParentIds(
  headings: OutlineHeading[],
  targetId: string,
  currentPath: string[] = []
): string[] | null {
  for (const heading of headings) {
    if (heading.id === targetId) {
      return currentPath;
    }
    const found = getParentIds(
      heading.children,
      targetId,
      [...currentPath, heading.id]
    );
    if (found !== null) {
      return found;
    }
  }
  return null;
}

/**
 * Get heading level icon/indicator
 */
export function getHeadingLevelIndicator(level: number): string {
  switch (level) {
    case 1:
      return 'H1';
    case 2:
      return 'H2';
    case 3:
      return 'H3';
    case 4:
      return 'H4';
    case 5:
      return 'H5';
    case 6:
      return 'H6';
    default:
      return `H${level}`;
  }
}

/**
 * Get accessible label for a heading
 */
export function getHeadingAccessibleLabel(heading: OutlineHeading): string {
  const levelName = `Heading level ${heading.level}`;
  const pageInfo = `Page ${heading.position.page}`;
  return `${heading.text}, ${levelName}, ${pageInfo}`;
}

/**
 * Count total headings in tree (including children)
 */
export function countHeadings(headings: OutlineHeading[]): number {
  let count = 0;
  for (const heading of headings) {
    count += 1 + countHeadings(heading.children);
  }
  return count;
}
