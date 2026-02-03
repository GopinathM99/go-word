/**
 * AccessibilityBridge.test.ts
 *
 * Tests for the accessibility bridge functionality.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { AccessibilityBridge, CursorInfo, SelectionInfo } from './AccessibilityBridge';
import { RenderModel } from './types';

describe('AccessibilityBridge', () => {
  let container: HTMLElement;
  let bridge: AccessibilityBridge;

  beforeEach(() => {
    container = document.createElement('div');
    document.body.appendChild(container);
    bridge = new AccessibilityBridge(container);
  });

  afterEach(() => {
    bridge.destroy();
    container.remove();
  });

  describe('initialization', () => {
    it('should create accessibility container', () => {
      const a11yContainer = container.querySelector('#accessibility-bridge');
      expect(a11yContainer).toBeTruthy();
    });

    it('should create live regions', () => {
      const polite = container.querySelector('[aria-live="polite"]');
      const assertive = container.querySelector('[aria-live="assertive"]');
      expect(polite).toBeTruthy();
      expect(assertive).toBeTruthy();
    });

    it('should create document root with proper ARIA attributes', () => {
      const docRoot = container.querySelector('[role="application"]');
      expect(docRoot).toBeTruthy();
      expect(docRoot?.getAttribute('aria-label')).toBe('Document Editor');
    });
  });

  describe('updateFromRenderModel', () => {
    it('should handle null model', () => {
      bridge.updateFromRenderModel(null);
      const docContent = container.querySelector('[role="document"]');
      expect(docContent).toBeFalsy();
    });

    it('should create page regions', () => {
      const model: RenderModel = {
        pages: [
          {
            page_index: 0,
            width: 816,
            height: 1056,
            items: [],
          },
          {
            page_index: 1,
            width: 816,
            height: 1056,
            items: [],
          },
        ],
      };

      bridge.updateFromRenderModel(model);

      const pages = container.querySelectorAll('[role="region"]');
      expect(pages.length).toBe(2);
      expect(pages[0].getAttribute('aria-label')).toBe('Page 1');
      expect(pages[1].getAttribute('aria-label')).toBe('Page 2');
    });

    it('should create accessible text content', () => {
      const model: RenderModel = {
        pages: [
          {
            page_index: 0,
            width: 816,
            height: 1056,
            items: [
              {
                type: 'GlyphRun',
                text: 'Hello World',
                font_family: 'Arial',
                font_size: 12,
                bold: false,
                italic: false,
                underline: false,
                color: { r: 0, g: 0, b: 0, a: 255 },
                x: 0,
                y: 0,
                hyperlink: null,
              },
            ],
          },
        ],
      };

      bridge.updateFromRenderModel(model);

      const paragraph = container.querySelector('[role="paragraph"]');
      expect(paragraph).toBeTruthy();
      expect(paragraph?.textContent).toContain('Hello World');
    });

    it('should create accessible hyperlinks', () => {
      const model: RenderModel = {
        pages: [
          {
            page_index: 0,
            width: 816,
            height: 1056,
            items: [
              {
                type: 'GlyphRun',
                text: 'Click here',
                font_family: 'Arial',
                font_size: 12,
                bold: false,
                italic: false,
                underline: true,
                color: { r: 0, g: 0, b: 255, a: 255 },
                x: 0,
                y: 0,
                hyperlink: {
                  node_id: 'link-1',
                  target: 'https://example.com',
                  tooltip: 'Example website',
                  link_type: 'External',
                },
              },
            ],
          },
        ],
      };

      bridge.updateFromRenderModel(model);

      const link = container.querySelector('[role="link"]');
      expect(link).toBeTruthy();
      expect(link?.getAttribute('href')).toBe('https://example.com');
    });

    it('should create accessible images', () => {
      const model: RenderModel = {
        pages: [
          {
            page_index: 0,
            width: 816,
            height: 1056,
            items: [
              {
                type: 'Image',
                node_id: 'img-1',
                resource_id: 'res-1',
                bounds: { x: 0, y: 0, width: 100, height: 100 },
                rotation: 0,
                alt_text: 'A beautiful sunset',
                title: 'Sunset Photo',
                selected: false,
              },
            ],
          },
        ],
      };

      bridge.updateFromRenderModel(model);

      const img = container.querySelector('[role="img"]');
      expect(img).toBeTruthy();
      expect(img?.getAttribute('aria-label')).toBe('A beautiful sunset');
    });
  });

  describe('announcements', () => {
    it('should announce polite messages', async () => {
      bridge.announce('Test message', 'polite');

      await new Promise((resolve) => requestAnimationFrame(resolve));

      const politeRegion = container.querySelector('[aria-live="polite"]');
      expect(politeRegion?.textContent).toBe('Test message');
    });

    it('should announce assertive messages', async () => {
      bridge.announce('Urgent message', 'assertive');

      await new Promise((resolve) => requestAnimationFrame(resolve));

      const assertiveRegion = container.querySelector('[aria-live="assertive"]');
      expect(assertiveRegion?.textContent).toBe('Urgent message');
    });
  });

  describe('cursor position', () => {
    it('should announce cursor position', async () => {
      vi.useFakeTimers();

      const cursor: CursorInfo = {
        line: 5,
        column: 23,
        pageNumber: 1,
        totalPages: 3,
        inParagraph: true,
        paragraphStyle: 'Normal',
        isBold: true,
        isItalic: false,
        isUnderline: false,
      };

      bridge.updateCursorPosition(cursor);

      vi.advanceTimersByTime(400);
      await Promise.resolve();

      const politeRegion = container.querySelector('[aria-live="polite"]');
      expect(politeRegion?.textContent).toContain('Line 5, Column 23');
      expect(politeRegion?.textContent).toContain('Bold');

      vi.useRealTimers();
    });

    it('should not re-announce same position', async () => {
      vi.useFakeTimers();

      const cursor: CursorInfo = {
        line: 5,
        column: 23,
        pageNumber: 1,
        totalPages: 3,
        inParagraph: true,
      };

      bridge.updateCursorPosition(cursor);
      vi.advanceTimersByTime(400);

      // Clear the region
      const politeRegion = container.querySelector('[aria-live="polite"]') as HTMLElement;
      politeRegion.textContent = '';

      // Announce same position again
      bridge.updateCursorPosition(cursor);
      vi.advanceTimersByTime(400);

      // Should not have announced again
      expect(politeRegion?.textContent).toBe('');

      vi.useRealTimers();
    });
  });

  describe('selection updates', () => {
    it('should announce short selections', () => {
      const selection: SelectionInfo = {
        text: 'Hello World',
        wordCount: 2,
        characterCount: 11,
        isCollapsed: false,
      };

      bridge.updateSelection(selection);
    });

    it('should announce long selections with word count', () => {
      const longText = 'This is a very long selection that contains many words and characters that would be too long to read directly to the user';
      const selection: SelectionInfo = {
        text: longText,
        wordCount: 22,
        characterCount: longText.length,
        isCollapsed: false,
      };

      bridge.updateSelection(selection);
    });

    it('should not announce collapsed selections', () => {
      const selection: SelectionInfo = {
        text: '',
        wordCount: 0,
        characterCount: 0,
        isCollapsed: true,
      };

      bridge.updateSelection(selection);
    });
  });

  describe('format announcements', () => {
    it('should announce applied formatting', async () => {
      bridge.announceFormatChange('Bold', true);

      await new Promise((resolve) => requestAnimationFrame(resolve));

      const assertiveRegion = container.querySelector('[aria-live="assertive"]');
      expect(assertiveRegion?.textContent).toBe('Bold applied');
    });

    it('should announce removed formatting', async () => {
      bridge.announceFormatChange('Italic', false);

      await new Promise((resolve) => requestAnimationFrame(resolve));

      const assertiveRegion = container.querySelector('[aria-live="assertive"]');
      expect(assertiveRegion?.textContent).toBe('Italic removed');
    });
  });

  describe('page navigation', () => {
    it('should announce page changes', async () => {
      bridge.announcePageChange(2, 5);

      await new Promise((resolve) => requestAnimationFrame(resolve));

      const politeRegion = container.querySelector('[aria-live="polite"]');
      expect(politeRegion?.textContent).toBe('Page 2 of 5');
    });
  });

  describe('document status', () => {
    it('should announce document saved', async () => {
      bridge.announceDocumentSaved();

      await new Promise((resolve) => requestAnimationFrame(resolve));

      const assertiveRegion = container.querySelector('[aria-live="assertive"]');
      expect(assertiveRegion?.textContent).toBe('Document saved');
    });

    it('should announce errors', async () => {
      bridge.announceError('Failed to save document');

      await new Promise((resolve) => requestAnimationFrame(resolve));

      const assertiveRegion = container.querySelector('[aria-live="assertive"]');
      expect(assertiveRegion?.textContent).toBe('Error: Failed to save document');
    });
  });

  describe('cleanup', () => {
    it('should remove container on destroy', () => {
      bridge.destroy();

      const a11yContainer = container.querySelector('#accessibility-bridge');
      expect(a11yContainer).toBeFalsy();
    });
  });
});
