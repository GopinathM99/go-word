/**
 * AccessibilityBridge.ts
 *
 * Creates a hidden DOM structure mirroring the canvas document
 * so screen readers can read the content. The canvas itself is
 * not accessible, but this shadow DOM provides the accessible tree.
 */

import { RenderModel, PageRender, RenderItem, HyperlinkRenderInfo } from './types';

// =============================================================================
// Types
// =============================================================================

export interface CursorInfo {
  line: number;
  column: number;
  pageNumber: number;
  totalPages: number;
  inParagraph: boolean;
  paragraphStyle?: string;
  characterStyle?: string;
  isBold?: boolean;
  isItalic?: boolean;
  isUnderline?: boolean;
}

export interface SelectionInfo {
  text: string;
  wordCount: number;
  characterCount: number;
  isCollapsed: boolean;
}

export interface DocumentStructure {
  paragraphs: ParagraphInfo[];
  headings: HeadingInfo[];
  tables: TableInfo[];
  images: ImageInfo[];
  lists: ListInfo[];
}

export interface ParagraphInfo {
  id: string;
  text: string;
  pageIndex: number;
}

export interface HeadingInfo {
  id: string;
  text: string;
  level: number;
  pageIndex: number;
}

export interface TableInfo {
  id: string;
  rowCount: number;
  columnCount: number;
  pageIndex: number;
  caption?: string;
}

export interface ImageInfo {
  id: string;
  altText: string;
  title?: string;
  pageIndex: number;
}

export interface ListInfo {
  id: string;
  type: 'bullet' | 'numbered';
  itemCount: number;
  pageIndex: number;
}

// =============================================================================
// AccessibilityBridge Class
// =============================================================================

export class AccessibilityBridge {
  private container: HTMLElement;
  private documentRoot: HTMLElement;
  private liveRegionPolite: HTMLElement;
  private liveRegionAssertive: HTMLElement;
  private nodeMap: Map<string, HTMLElement>;
  private currentCursor: CursorInfo | null = null;
  private currentSelection: SelectionInfo | null = null;
  private lastAnnouncedCursor: string = '';
  private announceDebounceTimer: number | null = null;

  constructor(parentElement: HTMLElement) {
    this.nodeMap = new Map();

    // Create visually hidden but screen-reader accessible container
    this.container = document.createElement('div');
    this.container.id = 'accessibility-bridge';
    this.container.setAttribute('aria-hidden', 'false');
    this.container.style.cssText = `
      position: absolute;
      width: 1px;
      height: 1px;
      padding: 0;
      margin: -1px;
      overflow: hidden;
      clip: rect(0, 0, 0, 0);
      white-space: nowrap;
      border: 0;
    `;

    // Create document root with proper ARIA roles
    this.documentRoot = document.createElement('div');
    this.documentRoot.setAttribute('role', 'application');
    this.documentRoot.setAttribute('aria-label', 'Document Editor');
    this.documentRoot.setAttribute('aria-roledescription', 'word processor');
    this.container.appendChild(this.documentRoot);

    // Create live regions for dynamic announcements
    this.liveRegionPolite = this.createLiveRegion('polite');
    this.liveRegionAssertive = this.createLiveRegion('assertive');
    this.container.appendChild(this.liveRegionPolite);
    this.container.appendChild(this.liveRegionAssertive);

    // Append to parent
    parentElement.appendChild(this.container);
  }

  /**
   * Create an ARIA live region for screen reader announcements
   */
  private createLiveRegion(priority: 'polite' | 'assertive'): HTMLElement {
    const region = document.createElement('div');
    region.setAttribute('role', 'status');
    region.setAttribute('aria-live', priority);
    region.setAttribute('aria-atomic', 'true');
    region.setAttribute('aria-relevant', 'additions text');
    region.className = `live-region-${priority}`;
    region.style.cssText = `
      position: absolute;
      width: 1px;
      height: 1px;
      padding: 0;
      margin: -1px;
      overflow: hidden;
      clip: rect(0, 0, 0, 0);
      white-space: nowrap;
      border: 0;
    `;
    return region;
  }

  /**
   * Build the accessible tree from render model
   */
  updateFromRenderModel(model: RenderModel | null): void {
    if (!model) {
      this.clearDocument();
      return;
    }

    // Clear existing nodes
    this.documentRoot.innerHTML = '';
    this.nodeMap.clear();

    // Create document content wrapper
    const documentContent = document.createElement('div');
    documentContent.setAttribute('role', 'document');
    documentContent.setAttribute('aria-label', 'Document content');
    documentContent.tabIndex = 0;

    // Process each page
    model.pages.forEach((page, pageIndex) => {
      const pageElement = this.createPageElement(page, pageIndex);
      documentContent.appendChild(pageElement);
    });

    this.documentRoot.appendChild(documentContent);
  }

  /**
   * Create an accessible page element
   */
  private createPageElement(page: PageRender, pageIndex: number): HTMLElement {
    const pageElement = document.createElement('div');
    pageElement.setAttribute('role', 'region');
    pageElement.setAttribute('aria-label', `Page ${pageIndex + 1}`);
    pageElement.id = `a11y-page-${pageIndex}`;

    // Group render items into logical structures
    const textRuns: RenderItem[] = [];
    let currentParagraphId: string | null = null;
    let currentParagraphElement: HTMLElement | null = null;
    let currentTableElement: HTMLElement | null = null;

    for (const item of page.items) {
      switch (item.type) {
        case 'GlyphRun': {
          // Create or append to paragraph
          if (!currentParagraphElement) {
            currentParagraphElement = document.createElement('p');
            currentParagraphElement.setAttribute('role', 'paragraph');
            pageElement.appendChild(currentParagraphElement);
          }

          // Create text span
          const textSpan = document.createElement('span');
          textSpan.textContent = item.text;

          // Add hyperlink wrapper if present
          if (item.hyperlink) {
            const linkWrapper = this.createHyperlinkElement(item.hyperlink, item.text);
            currentParagraphElement.appendChild(linkWrapper);
          } else {
            // Apply text formatting as ARIA attributes
            if (item.bold) {
              textSpan.setAttribute('aria-label', `${item.text}, bold`);
            }
            if (item.italic) {
              textSpan.setAttribute('aria-label', `${item.text}, italic`);
            }
            currentParagraphElement.appendChild(textSpan);
          }
          break;
        }

        case 'Image': {
          // Create image placeholder with alt text
          const imgElement = document.createElement('div');
          imgElement.setAttribute('role', 'img');
          imgElement.setAttribute('aria-label', item.alt_text || 'Image');
          if (item.title) {
            imgElement.setAttribute('title', item.title);
          }
          imgElement.id = `a11y-image-${item.node_id}`;
          this.nodeMap.set(item.node_id, imgElement);

          if (currentParagraphElement) {
            currentParagraphElement.appendChild(imgElement);
          } else {
            pageElement.appendChild(imgElement);
          }
          break;
        }

        case 'TableCell': {
          // Handle table cells - create table structure if needed
          if (!currentTableElement) {
            currentTableElement = document.createElement('div');
            currentTableElement.setAttribute('role', 'table');
            currentTableElement.setAttribute('aria-label', 'Table');
            pageElement.appendChild(currentTableElement);
          }
          // Table cell rendering is complex - simplified here
          break;
        }

        case 'Selection': {
          // Selection is visual only, no a11y representation needed
          break;
        }

        case 'Caret': {
          // Caret position - handled by cursor position updates
          break;
        }

        case 'Rectangle':
        case 'Line':
        case 'TableBorder': {
          // Visual elements, no a11y representation needed
          break;
        }
      }
    }

    return pageElement;
  }

  /**
   * Create an accessible hyperlink element
   */
  private createHyperlinkElement(hyperlink: HyperlinkRenderInfo, text: string): HTMLElement {
    const link = document.createElement('a');
    link.textContent = text;
    link.href = hyperlink.target;
    link.setAttribute('role', 'link');

    if (hyperlink.tooltip) {
      link.setAttribute('title', hyperlink.tooltip);
      link.setAttribute('aria-describedby', `tooltip-${hyperlink.node_id}`);
    }

    // Set link type description
    const linkTypeDesc = this.getLinkTypeDescription(hyperlink.link_type);
    link.setAttribute('aria-label', `${text}, ${linkTypeDesc}`);

    // For external links, indicate they open in new context
    if (hyperlink.link_type === 'External') {
      link.setAttribute('aria-description', 'Opens in browser');
    }

    return link;
  }

  /**
   * Get human-readable link type description
   */
  private getLinkTypeDescription(linkType: string): string {
    switch (linkType) {
      case 'External':
        return 'external link';
      case 'Internal':
        return 'bookmark link';
      case 'Email':
        return 'email link';
      default:
        return 'link';
    }
  }

  /**
   * Clear the document tree
   */
  private clearDocument(): void {
    this.documentRoot.innerHTML = '';
    this.nodeMap.clear();
  }

  /**
   * Announce a message to screen readers
   */
  announce(message: string, priority: 'polite' | 'assertive' = 'polite'): void {
    const region = priority === 'assertive' ? this.liveRegionAssertive : this.liveRegionPolite;

    // Clear and set new message (forces re-announcement)
    region.textContent = '';

    // Use requestAnimationFrame to ensure DOM update
    requestAnimationFrame(() => {
      region.textContent = message;
    });
  }

  /**
   * Announce a message with debouncing for rapid updates
   */
  announceDebounced(message: string, priority: 'polite' | 'assertive' = 'polite', delay: number = 300): void {
    if (this.announceDebounceTimer !== null) {
      window.clearTimeout(this.announceDebounceTimer);
    }

    this.announceDebounceTimer = window.setTimeout(() => {
      this.announce(message, priority);
      this.announceDebounceTimer = null;
    }, delay);
  }

  /**
   * Update and announce cursor position
   */
  updateCursorPosition(cursor: CursorInfo): void {
    this.currentCursor = cursor;

    // Build cursor description
    const parts: string[] = [];
    parts.push(`Line ${cursor.line}, Column ${cursor.column}`);

    if (cursor.paragraphStyle && cursor.paragraphStyle !== 'Normal') {
      parts.push(cursor.paragraphStyle);
    }

    // Format indicators
    const formats: string[] = [];
    if (cursor.isBold) formats.push('Bold');
    if (cursor.isItalic) formats.push('Italic');
    if (cursor.isUnderline) formats.push('Underline');
    if (formats.length > 0) {
      parts.push(formats.join(', '));
    }

    const announcement = parts.join(', ');

    // Only announce if changed
    if (announcement !== this.lastAnnouncedCursor) {
      this.lastAnnouncedCursor = announcement;
      this.announceDebounced(announcement);
    }
  }

  /**
   * Update and announce selection changes
   */
  updateSelection(selection: SelectionInfo): void {
    this.currentSelection = selection;

    if (selection.isCollapsed) {
      // No selection - already handled by cursor position
      return;
    }

    // Announce selection
    let announcement: string;
    if (selection.text.length <= 50) {
      announcement = `Selected: ${selection.text}`;
    } else {
      announcement = `Selection: ${selection.wordCount} words, ${selection.characterCount} characters`;
    }

    this.announce(announcement);
  }

  /**
   * Announce formatting changes
   */
  announceFormatChange(format: string, applied: boolean): void {
    const action = applied ? 'applied' : 'removed';
    this.announce(`${format} ${action}`, 'assertive');
  }

  /**
   * Announce page navigation
   */
  announcePageChange(currentPage: number, totalPages: number): void {
    this.announce(`Page ${currentPage} of ${totalPages}`);
  }

  /**
   * Announce document save status
   */
  announceDocumentSaved(): void {
    this.announce('Document saved', 'assertive');
  }

  /**
   * Announce an error
   */
  announceError(message: string): void {
    this.announce(`Error: ${message}`, 'assertive');
  }

  /**
   * Get current cursor info
   */
  getCursorInfo(): CursorInfo | null {
    return this.currentCursor;
  }

  /**
   * Get current selection info
   */
  getSelectionInfo(): SelectionInfo | null {
    return this.currentSelection;
  }

  /**
   * Cleanup and remove from DOM
   */
  destroy(): void {
    if (this.announceDebounceTimer !== null) {
      window.clearTimeout(this.announceDebounceTimer);
    }
    this.container.remove();
  }
}

// =============================================================================
// Hook for React integration
// =============================================================================

import { useEffect, useRef, useCallback } from 'react';

export function useAccessibilityBridge(containerRef: React.RefObject<HTMLElement | null>) {
  const bridgeRef = useRef<AccessibilityBridge | null>(null);

  useEffect(() => {
    if (containerRef.current && !bridgeRef.current) {
      bridgeRef.current = new AccessibilityBridge(containerRef.current);
    }

    return () => {
      if (bridgeRef.current) {
        bridgeRef.current.destroy();
        bridgeRef.current = null;
      }
    };
  }, [containerRef]);

  const updateFromRenderModel = useCallback((model: RenderModel | null) => {
    bridgeRef.current?.updateFromRenderModel(model);
  }, []);

  const announce = useCallback((message: string, priority: 'polite' | 'assertive' = 'polite') => {
    bridgeRef.current?.announce(message, priority);
  }, []);

  const updateCursorPosition = useCallback((cursor: CursorInfo) => {
    bridgeRef.current?.updateCursorPosition(cursor);
  }, []);

  const updateSelection = useCallback((selection: SelectionInfo) => {
    bridgeRef.current?.updateSelection(selection);
  }, []);

  const announceFormatChange = useCallback((format: string, applied: boolean) => {
    bridgeRef.current?.announceFormatChange(format, applied);
  }, []);

  const announcePageChange = useCallback((currentPage: number, totalPages: number) => {
    bridgeRef.current?.announcePageChange(currentPage, totalPages);
  }, []);

  const announceDocumentSaved = useCallback(() => {
    bridgeRef.current?.announceDocumentSaved();
  }, []);

  const announceError = useCallback((message: string) => {
    bridgeRef.current?.announceError(message);
  }, []);

  return {
    bridge: bridgeRef.current,
    updateFromRenderModel,
    announce,
    updateCursorPosition,
    updateSelection,
    announceFormatChange,
    announcePageChange,
    announceDocumentSaved,
    announceError,
  };
}
