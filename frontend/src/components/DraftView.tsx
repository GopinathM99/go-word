/**
 * DraftView - Simplified continuous rendering for draft mode
 *
 * Features:
 * - Continuous scroll (no page breaks shown)
 * - Simplified layout (faster rendering)
 * - Optional: Show style names in left margin
 * - Optional: Hide images (show placeholders for speed)
 * - Single column regardless of document columns
 * - Focus on text editing, not appearance
 */

import { useRef, useEffect, useCallback, useState, useMemo } from 'react';
import { RenderModel, Selection, RenderItem, colorToCss, GlyphRunData, PageRender } from '../lib/types';
import { DraftViewOptions } from '../lib/viewModeTypes';
import { Command } from '../lib/InputController';
import './DraftView.css';

// =============================================================================
// Types
// =============================================================================

export interface DraftViewProps {
  /** Render model from backend */
  renderModel: RenderModel | null;
  /** Current selection */
  selection: Selection | null;
  /** Command handler */
  onCommand: (command: Command) => void;
  /** Draft view options */
  options: DraftViewOptions;
  /** Content width (in points) */
  contentWidth?: number;
  /** Zoom level */
  zoom?: number;
  /** Callback when container resizes */
  onContainerResize?: (width: number, height: number) => void;
}

interface DraftBlockData {
  nodeId: string;
  y: number;
  height: number;
  styleName?: string;
  isHeading: boolean;
  headingLevel?: number;
  textItems: Array<{
    text: string;
    fontFamily: string;
    fontSize: number;
    bold: boolean;
    italic: boolean;
    underline: boolean;
    color?: { r: number; g: number; b: number; a: number };
  }>;
  imageItems: Array<{
    src?: string;
    alt?: string;
    width: number;
    height: number;
  }>;
}

// =============================================================================
// Helper Functions
// =============================================================================

/**
 * Convert paginated render model to continuous draft blocks
 */
function convertToDraftBlocks(
  renderModel: RenderModel,
  options: DraftViewOptions
): DraftBlockData[] {
  const blocks: DraftBlockData[] = [];
  let currentY = 0;
  let currentBlock: DraftBlockData | null = null;
  let lastNodeId: string | null = null;

  for (const page of renderModel.pages) {
    for (const item of page.items) {
      // Get node ID from item if it's a GlyphRun or Image
      let nodeId: string | null = null;

      if (item.type === 'GlyphRun' && item.hyperlink) {
        nodeId = item.hyperlink.node_id;
      } else if (item.type === 'Image') {
        nodeId = item.node_id;
      }

      // Use a generated ID if none found
      const effectiveNodeId = nodeId || `block-${blocks.length}`;

      // Start a new block if node ID changes or no current block
      if (!currentBlock || (nodeId && nodeId !== lastNodeId)) {
        if (currentBlock) {
          blocks.push(currentBlock);
        }

        currentBlock = {
          nodeId: effectiveNodeId,
          y: currentY,
          height: 0,
          styleName: undefined,
          isHeading: false,
          headingLevel: undefined,
          textItems: [],
          imageItems: [],
        };
        lastNodeId = nodeId;
      }

      // Add item to current block based on type
      if (item.type === 'GlyphRun') {
        const glyphItem = item as { type: 'GlyphRun' } & GlyphRunData;
        currentBlock.textItems.push({
          text: glyphItem.text,
          fontFamily: glyphItem.font_family,
          fontSize: glyphItem.font_size,
          bold: glyphItem.bold,
          italic: glyphItem.italic,
          underline: glyphItem.underline,
          color: glyphItem.color,
        });
        currentBlock.height = Math.max(currentBlock.height, glyphItem.font_size * 1.5);
      } else if (item.type === 'Image') {
        currentBlock.imageItems.push({
          src: undefined, // Would need to fetch from backend
          alt: 'Image',
          width: item.bounds.width,
          height: item.bounds.height,
        });
        currentBlock.height = Math.max(currentBlock.height, item.bounds.height);
      }
    }
  }

  // Push last block
  if (currentBlock) {
    blocks.push(currentBlock);
  }

  // Update Y positions
  for (const block of blocks) {
    block.y = currentY;
    currentY += block.height + 8; // Add spacing between blocks
  }

  return blocks;
}

/**
 * Extract heading level from style name
 */
function extractHeadingLevel(styleName?: string): number | undefined {
  if (!styleName) return undefined;
  const match = styleName.match(/heading\s*(\d)/i);
  if (match) {
    return parseInt(match[1], 10);
  }
  return undefined;
}

// =============================================================================
// DraftView Component
// =============================================================================

export function DraftView({
  renderModel,
  selection,
  onCommand,
  options,
  contentWidth = 468,
  zoom = 1.0,
  onContainerResize,
}: DraftViewProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const [containerSize, setContainerSize] = useState({ width: 0, height: 0 });
  const [scrollTop, setScrollTop] = useState(0);

  // Calculate effective content width
  const effectiveWidth = options.wrapToWindow
    ? Math.max(containerSize.width / zoom - (options.showStyleNames ? options.styleNameMargin : 0) - 40, 300)
    : contentWidth;

  // Convert render model to draft blocks
  const draftBlocks = useMemo(() => {
    if (!renderModel) return [];
    return convertToDraftBlocks(renderModel, options);
  }, [renderModel, options]);

  // Calculate total content height
  const totalHeight = useMemo(() => {
    if (draftBlocks.length === 0) return 500;
    const lastBlock = draftBlocks[draftBlocks.length - 1];
    return lastBlock.y + lastBlock.height + 100; // Add padding at bottom
  }, [draftBlocks]);

  // Handle container resize
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width, height } = entry.contentRect;
        setContainerSize({ width, height });
        onContainerResize?.(width, height);
      }
    });

    resizeObserver.observe(container);
    return () => resizeObserver.disconnect();
  }, [onContainerResize]);

  // Handle scroll
  const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    setScrollTop(e.currentTarget.scrollTop);
  }, []);

  // Calculate visible range for virtualization
  const visibleRange = useMemo(() => {
    const buffer = 200;
    const top = scrollTop - buffer;
    const bottom = scrollTop + containerSize.height + buffer;

    let start = 0;
    let end = draftBlocks.length;

    for (let i = 0; i < draftBlocks.length; i++) {
      const block = draftBlocks[i];
      if (block.y + block.height >= top) {
        start = i;
        break;
      }
    }

    for (let i = start; i < draftBlocks.length; i++) {
      const block = draftBlocks[i];
      if (block.y > bottom) {
        end = i;
        break;
      }
    }

    return { start, end };
  }, [scrollTop, containerSize.height, draftBlocks]);

  // Visible blocks
  const visibleBlocks = draftBlocks.slice(visibleRange.start, visibleRange.end);

  // Handle keyboard navigation
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      // Let InputController handle most keys
      // Add draft-specific shortcuts here if needed
    },
    []
  );

  // Handle focus
  const handleFocus = useCallback(() => {
    // Ensure the content area is focusable for keyboard input
  }, []);

  return (
    <div
      ref={containerRef}
      className={`draft-view ${options.showStyleNames ? 'show-styles' : ''}`}
      onScroll={handleScroll}
      onKeyDown={handleKeyDown}
      onFocus={handleFocus}
      tabIndex={0}
      role="textbox"
      aria-multiline="true"
      aria-label="Document content (Draft view)"
    >
      {/* Style names column */}
      {options.showStyleNames && (
        <div
          className="draft-style-column"
          style={{ width: options.styleNameMargin * zoom }}
          aria-hidden="true"
        >
          {visibleBlocks.map((block) => (
            <div
              key={`style-${block.nodeId}`}
              className="draft-style-label"
              style={{
                top: block.y * zoom,
                height: block.height * zoom,
              }}
            >
              {block.styleName || 'Normal'}
            </div>
          ))}
        </div>
      )}

      {/* Main content area */}
      <div
        ref={contentRef}
        className="draft-content"
        style={{
          width: effectiveWidth * zoom,
          height: totalHeight * zoom,
          marginLeft: options.showStyleNames ? options.styleNameMargin * zoom : 0,
        }}
      >
        {/* Render visible blocks */}
        {visibleBlocks.map((block) => (
          <DraftBlockRenderer
            key={block.nodeId}
            block={block}
            options={options}
            zoom={zoom}
            effectiveWidth={effectiveWidth}
          />
        ))}

        {/* Empty state */}
        {draftBlocks.length === 0 && (
          <div className="draft-empty">
            <p>Start typing to add content...</p>
          </div>
        )}
      </div>
    </div>
  );
}

// =============================================================================
// Draft Block Renderer
// =============================================================================

interface DraftBlockRendererProps {
  block: DraftBlockData;
  options: DraftViewOptions;
  zoom: number;
  effectiveWidth: number;
}

function DraftBlockRenderer({
  block,
  options,
  zoom,
  effectiveWidth,
}: DraftBlockRendererProps) {
  return (
    <div
      className={`draft-block ${block.isHeading ? 'heading' : ''} ${
        block.headingLevel ? `heading-${block.headingLevel}` : ''
      }`}
      style={{
        position: 'absolute',
        top: block.y * zoom,
        width: effectiveWidth * zoom,
        minHeight: block.height * zoom,
      }}
      data-node-id={block.nodeId}
    >
      {/* Render text items */}
      {block.textItems.map((item, idx) => (
        <span
          key={`text-${idx}`}
          className="draft-text"
          style={{
            fontFamily: item.fontFamily,
            fontSize: item.fontSize * zoom,
            fontWeight: item.bold ? 'bold' : 'normal',
            fontStyle: item.italic ? 'italic' : 'normal',
            textDecoration: item.underline ? 'underline' : 'none',
            color: item.color ? colorToCss(item.color) : undefined,
          }}
        >
          {item.text}
        </span>
      ))}

      {/* Render image items */}
      {block.imageItems.map((item, idx) => {
        if (!options.showImages) {
          return (
            <div
              key={`img-${idx}`}
              className="draft-image-placeholder"
              style={{
                width: item.width * zoom,
                height: item.height * zoom,
              }}
              aria-label={`Image placeholder: ${item.alt || 'Image'}`}
            >
              <span className="placeholder-icon">[IMG]</span>
              {item.alt && <span className="placeholder-alt">{item.alt}</span>}
            </div>
          );
        }
        return (
          <img
            key={`img-${idx}`}
            className="draft-image"
            src={item.src || ''}
            alt={item.alt || 'Image'}
            style={{
              width: item.width * zoom,
              height: item.height * zoom,
            }}
          />
        );
      })}

      {/* Paragraph mark */}
      {options.showParagraphMarks && (
        <span className="paragraph-mark" aria-hidden="true">
          &para;
        </span>
      )}
    </div>
  );
}

// =============================================================================
// Draft View Options Panel
// =============================================================================

export interface DraftViewOptionsPanelProps {
  options: DraftViewOptions;
  onOptionsChange: (options: Partial<DraftViewOptions>) => void;
}

export function DraftViewOptionsPanel({
  options,
  onOptionsChange,
}: DraftViewOptionsPanelProps) {
  return (
    <div className="draft-options-panel" role="group" aria-label="Draft view options">
      <label className="draft-option">
        <input
          type="checkbox"
          checked={options.showStyleNames}
          onChange={(e) => onOptionsChange({ showStyleNames: e.target.checked })}
        />
        <span>Show style names</span>
      </label>

      <label className="draft-option">
        <input
          type="checkbox"
          checked={options.showImages}
          onChange={(e) => onOptionsChange({ showImages: e.target.checked })}
        />
        <span>Show images</span>
      </label>

      <label className="draft-option">
        <input
          type="checkbox"
          checked={options.wrapToWindow}
          onChange={(e) => onOptionsChange({ wrapToWindow: e.target.checked })}
        />
        <span>Wrap to window</span>
      </label>

      <label className="draft-option">
        <input
          type="checkbox"
          checked={options.showParagraphMarks}
          onChange={(e) => onOptionsChange({ showParagraphMarks: e.target.checked })}
        />
        <span>Show paragraph marks</span>
      </label>
    </div>
  );
}

export default DraftView;
