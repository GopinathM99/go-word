import { useRef, useEffect, useCallback, useState, useMemo } from 'react';
import { RenderModel, Selection, RenderItem, colorToCss, PageRender, CompositionState, HyperlinkRenderInfo } from '../lib/types';
import { useInputController, Command } from '../lib/InputController';
import {
  PageLayout,
  useRenderScheduler,
} from '../lib/RenderScheduler';
import { Rulers, DEFAULT_MARGINS, DEFAULT_PAGE_DIMENSIONS } from './Ruler';
import { useVirtualizedPages, usePageRenderCache } from '../hooks/useVirtualizedPages';

// =============================================================================
// Constants
// =============================================================================

const PAGE_GAP = 20; // Gap between pages in pixels
const PAGE_SHADOW_BLUR = 8;
const CARET_BLINK_INTERVAL = 500; // Blink interval in ms
const RULER_SIZE = 20; // Size of ruler in pixels

// =============================================================================
// Types
// =============================================================================

export type ViewMode = 'print-layout' | 'web-layout';

export interface EditorCanvasProps {
  renderModel: RenderModel | null;
  selection: Selection | null;
  onCommand: (command: Command) => void;
  onCompositionChange?: (state: CompositionState) => void;
  dirtyPages?: number[]; // Pages that need re-render from document changes
  /** Current zoom level (0.25 to 5.0) */
  zoom?: number;
  /** View mode */
  viewMode?: ViewMode;
  /** Whether to show rulers */
  showRulers?: boolean;
  /** Callback when container dimensions change */
  onContainerResize?: (width: number, height: number) => void;
  /** Handle wheel events for zoom */
  onWheelZoom?: (deltaY: number, ctrlKey: boolean) => boolean;
  /** Callback when a hyperlink is clicked */
  onHyperlinkClick?: (hyperlink: HyperlinkRenderInfo, ctrlKey: boolean) => void;
}

// =============================================================================
// EditorCanvas Component
// =============================================================================

export function EditorCanvas({
  renderModel,
  selection,
  onCommand,
  onCompositionChange,
  dirtyPages,
  zoom = 1.0,
  viewMode = 'print-layout',
  showRulers = true,
  onContainerResize,
  onWheelZoom,
  onHyperlinkClick,
}: EditorCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const [scrollOffset, setScrollOffset] = useState({ x: 0, y: 0 });
  const [containerSize, setContainerSize] = useState({ width: 0, height: 0 });
  const [caretVisible, setCaretVisible] = useState(true);
  const caretBlinkRef = useRef<number | null>(null);
  const lastKeyTimeRef = useRef<number>(Date.now());
  const [localCompositionState, setLocalCompositionState] = useState<CompositionState>({
    isComposing: false,
    compositionText: '',
    compositionStart: null,
  });

  // Image cache for rendering
  const imageCacheRef = useRef<Map<string, HTMLImageElement>>(new Map());
  const loadingImagesRef = useRef<Set<string>>(new Set());
  const forceRenderRef = useRef<(() => void) | null>(null);

  // Initialize render scheduler
  const {
    setCanvas,
    setRenderModel: setSchedulerRenderModel,
    setPageLayouts,
    markPageDirty,
    markPagesDirty,
    markGlobalDirty,
    handleScroll: schedulerHandleScroll,
    getVisiblePageIndices,
    forceRender,
  } = useRenderScheduler({
    bufferPages: 2,
    enableCaching: true,
    onRenderComplete: () => {
      // Optional: trigger any post-render callbacks
    },
  });

  // Keep forceRender ref up to date
  forceRenderRef.current = forceRender;

  // Load an image for rendering (called when image not in cache)
  const loadImageForRendering = useCallback((resourceId: string) => {
    // Skip if already loading
    if (loadingImagesRef.current.has(resourceId)) {
      return;
    }

    loadingImagesRef.current.add(resourceId);

    // Request image data from backend via Tauri
    // For now, we use a placeholder approach - in a real implementation,
    // this would call the Tauri backend to get the image data URL
    const img = new Image();
    img.onload = () => {
      imageCacheRef.current.set(resourceId, img);
      loadingImagesRef.current.delete(resourceId);
      // Trigger re-render to show the loaded image
      forceRenderRef.current?.();
    };
    img.onerror = () => {
      loadingImagesRef.current.delete(resourceId);
      console.warn(`Failed to load image: ${resourceId}`);
    };

    // For development, use a placeholder image
    // In production, this would be:
    // invoke('get_image_data_url', { resourceId }).then(dataUrl => { img.src = dataUrl; });
    img.src = `data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='100' height='100' viewBox='0 0 100 100'><rect fill='%23ddd' width='100' height='100'/><text x='50' y='50' text-anchor='middle' fill='%23999' font-size='14'>Image</text></svg>`;
  }, []);

  // Handle composition state changes
  const handleCompositionChange = useCallback(
    (state: CompositionState) => {
      setLocalCompositionState(state);
      // Keep caret visible during composition
      if (state.isComposing) {
        setCaretVisible(true);
      }
      // Notify parent of composition changes
      if (onCompositionChange) {
        onCompositionChange(state);
      }
    },
    [onCompositionChange]
  );

  // Initialize the input controller
  const {
    handleKeyDown: inputHandleKeyDown,
    handleCompositionStart: inputHandleCompositionStart,
    handleCompositionUpdate: inputHandleCompositionUpdate,
    handleCompositionEnd: inputHandleCompositionEnd,
    handleInput: inputHandleInput,
    compositionState,
  } = useInputController({
    onCommand,
    onCompositionChange: handleCompositionChange,
    selection,
  });

  // Get page dimensions (use first page or defaults)
  const pageWidth = renderModel?.pages[0]?.width ?? DEFAULT_PAGE_DIMENSIONS.width;
  const pageHeight = renderModel?.pages[0]?.height ?? DEFAULT_PAGE_DIMENSIONS.height;

  // Calculate page heights array for virtualization (accounting for zoom)
  const pageHeights = useMemo(() => {
    if (!renderModel || renderModel.pages.length === 0) {
      return [];
    }
    return renderModel.pages.map((page) => page.height * zoom);
  }, [renderModel, zoom]);

  // Page render cache for memory management
  const pageRenderCache = usePageRenderCache(10);
  const pageRenderCacheRef = useRef(pageRenderCache);
  useEffect(() => { pageRenderCacheRef.current = pageRenderCache; }, [pageRenderCache]);

  // Use virtualized pages hook for efficient rendering
  const {
    visibleRange,
    offsetTop: virtualOffsetTop,
    totalHeight: virtualTotalHeight,
    visiblePageIndices,
    bufferedPageIndices,
    shouldRenderPage: virtualShouldRenderPage,
    getPageTop,
    getPageVisibility,
  } = useVirtualizedPages({
    totalPages: renderModel?.pages.length ?? 0,
    pageHeights,
    containerHeight: containerSize.height,
    scrollTop: scrollOffset.y,
    bufferPages: 2,
    pageGap: PAGE_GAP,
    onVisibleRangeChange: useCallback((start: number, end: number) => {
      // Mark newly visible pages for rendering
      for (let i = start; i < end; i++) {
        markPageDirty(i);
      }
    }, [markPageDirty]),
  });

  // Calculate page layouts (positions of each page in canvas coordinates)
  const calculatePageLayouts = useCallback(
    (canvasWidth: number): PageLayout[] => {
      if (!renderModel || renderModel.pages.length === 0) {
        return [];
      }

      const layouts: PageLayout[] = [];
      let currentY = PAGE_GAP;

      // Account for rulers in print layout mode
      const rulerOffset = showRulers && viewMode === 'print-layout' ? RULER_SIZE : 0;

      for (const page of renderModel.pages) {
        // Center the page horizontally (accounting for zoom)
        const scaledPageWidth = page.width * zoom;
        const availableWidth = canvasWidth - rulerOffset;
        const pageX = Math.max(PAGE_GAP, (availableWidth - scaledPageWidth) / 2) + rulerOffset;

        layouts.push({
          page,
          x: pageX,
          y: currentY + rulerOffset,
          width: page.width,
          height: page.height,
        });

        currentY += page.height * zoom + PAGE_GAP;
      }

      return layouts;
    },
    [renderModel, zoom, showRulers, viewMode]
  );

  // Get total document height (accounting for zoom)
  const getTotalHeight = useCallback((): number => {
    if (!renderModel || renderModel.pages.length === 0) {
      return 0;
    }

    const rulerOffset = showRulers && viewMode === 'print-layout' ? RULER_SIZE : 0;
    let totalHeight = PAGE_GAP + rulerOffset;
    for (const page of renderModel.pages) {
      totalHeight += page.height * zoom + PAGE_GAP;
    }
    return totalHeight;
  }, [renderModel, zoom, showRulers, viewMode]);

  // Get total document width (for horizontal scrolling if needed)
  const getTotalWidth = useCallback((): number => {
    if (!renderModel || renderModel.pages.length === 0) {
      return containerSize.width;
    }

    const rulerOffset = showRulers && viewMode === 'print-layout' ? RULER_SIZE : 0;
    const maxPageWidth = Math.max(...renderModel.pages.map(p => p.width)) * zoom;
    return Math.max(containerSize.width, maxPageWidth + PAGE_GAP * 2 + rulerOffset);
  }, [renderModel, zoom, containerSize.width, showRulers, viewMode]);

  // Render a single render item
  const renderItem = useCallback(
    (ctx: CanvasRenderingContext2D, item: RenderItem, pageX: number, pageY: number) => {
      switch (item.type) {
        case 'GlyphRun': {
          const fontStyle = item.italic ? 'italic' : 'normal';
          const fontWeight = item.bold ? 'bold' : 'normal';
          const fontSize = item.font_size;
          const fontFamily = item.font_family || 'sans-serif';

          ctx.font = `${fontStyle} ${fontWeight} ${fontSize}px ${fontFamily}`;
          ctx.fillStyle = colorToCss(item.color);
          ctx.textBaseline = 'alphabetic';

          // Render text at the baseline position
          const textX = pageX + item.x;
          const textY = pageY + item.y;
          ctx.fillText(item.text, textX, textY);

          // Render underline if needed (for hyperlinks or styled text)
          if (item.underline) {
            const textWidth = ctx.measureText(item.text).width;
            ctx.strokeStyle = colorToCss(item.color);
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(textX, textY + 2);
            ctx.lineTo(textX + textWidth, textY + 2);
            ctx.stroke();
          }
          break;
        }

        case 'Rectangle': {
          const bounds = item.bounds;
          const x = pageX + bounds.x;
          const y = pageY + bounds.y;

          if (item.fill) {
            ctx.fillStyle = colorToCss(item.fill);
            ctx.fillRect(x, y, bounds.width, bounds.height);
          }

          if (item.stroke && item.stroke_width > 0) {
            ctx.strokeStyle = colorToCss(item.stroke);
            ctx.lineWidth = item.stroke_width;
            ctx.strokeRect(x, y, bounds.width, bounds.height);
          }
          break;
        }

        case 'Selection': {
          ctx.fillStyle = colorToCss(item.color);
          for (const rect of item.rects) {
            ctx.fillRect(pageX + rect.x, pageY + rect.y, rect.width, rect.height);
          }
          break;
        }

        case 'Line': {
          ctx.strokeStyle = colorToCss(item.color);
          ctx.lineWidth = item.width;
          ctx.beginPath();
          ctx.moveTo(pageX + item.x1, pageY + item.y1);
          ctx.lineTo(pageX + item.x2, pageY + item.y2);
          ctx.stroke();
          break;
        }

        case 'Image': {
          // Render image placeholder or cached image
          const { bounds, resource_id, rotation, selected, alt_text, title } = item;
          const imgX = pageX + bounds.x;
          const imgY = pageY + bounds.y;
          const imgW = bounds.width;
          const imgH = bounds.height;

          // Check if image is cached
          const cachedImage = imageCacheRef.current.get(resource_id);

          if (cachedImage && cachedImage.complete) {
            // Apply rotation if needed
            if (rotation !== 0) {
              ctx.save();
              ctx.translate(imgX + imgW / 2, imgY + imgH / 2);
              ctx.rotate((rotation * Math.PI) / 180);
              ctx.drawImage(cachedImage, -imgW / 2, -imgH / 2, imgW, imgH);
              ctx.restore();
            } else {
              ctx.drawImage(cachedImage, imgX, imgY, imgW, imgH);
            }
          } else {
            // Draw placeholder while loading
            ctx.fillStyle = '#f0f0f0';
            ctx.fillRect(imgX, imgY, imgW, imgH);
            ctx.strokeStyle = '#ccc';
            ctx.lineWidth = 1;
            ctx.strokeRect(imgX, imgY, imgW, imgH);

            // Draw loading indicator
            ctx.fillStyle = '#999';
            ctx.font = '12px sans-serif';
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillText('Loading...', imgX + imgW / 2, imgY + imgH / 2);

            // Try to load the image if not already loading
            if (!cachedImage) {
              loadImageForRendering(resource_id);
            }
          }

          // Draw selection handles if selected
          if (selected) {
            ctx.strokeStyle = '#0066cc';
            ctx.lineWidth = 2;
            ctx.strokeRect(imgX - 1, imgY - 1, imgW + 2, imgH + 2);

            // Draw resize handles
            const handleSize = 8;
            ctx.fillStyle = '#0066cc';
            // Corner handles
            ctx.fillRect(imgX - handleSize / 2, imgY - handleSize / 2, handleSize, handleSize);
            ctx.fillRect(imgX + imgW - handleSize / 2, imgY - handleSize / 2, handleSize, handleSize);
            ctx.fillRect(imgX - handleSize / 2, imgY + imgH - handleSize / 2, handleSize, handleSize);
            ctx.fillRect(imgX + imgW - handleSize / 2, imgY + imgH - handleSize / 2, handleSize, handleSize);
            // Edge handles
            ctx.fillRect(imgX + imgW / 2 - handleSize / 2, imgY - handleSize / 2, handleSize, handleSize);
            ctx.fillRect(imgX + imgW / 2 - handleSize / 2, imgY + imgH - handleSize / 2, handleSize, handleSize);
            ctx.fillRect(imgX - handleSize / 2, imgY + imgH / 2 - handleSize / 2, handleSize, handleSize);
            ctx.fillRect(imgX + imgW - handleSize / 2, imgY + imgH / 2 - handleSize / 2, handleSize, handleSize);
          }
          break;
        }

        case 'TableCell': {
          // Render table cell background
          const { bounds, background, selected } = item;
          const cellX = pageX + bounds.x;
          const cellY = pageY + bounds.y;
          const cellW = bounds.width;
          const cellH = bounds.height;

          // Draw cell background/shading if set
          if (background) {
            ctx.fillStyle = colorToCss(background);
            ctx.fillRect(cellX, cellY, cellW, cellH);
          }

          // Draw selection highlight if cell is selected
          if (selected) {
            ctx.fillStyle = 'rgba(0, 120, 215, 0.15)';
            ctx.fillRect(cellX, cellY, cellW, cellH);
            ctx.strokeStyle = 'rgba(0, 120, 215, 0.5)';
            ctx.lineWidth = 2;
            ctx.strokeRect(cellX, cellY, cellW, cellH);
          }
          break;
        }

        case 'TableBorder': {
          // Render table border line
          ctx.strokeStyle = colorToCss(item.color);
          ctx.lineWidth = item.width;

          // Handle different border styles
          if (item.style === 'dotted') {
            ctx.setLineDash([2, 2]);
          } else if (item.style === 'dashed') {
            ctx.setLineDash([4, 4]);
          } else if (item.style === 'double') {
            // Draw two lines for double border
            ctx.setLineDash([]);
            ctx.beginPath();
            const offset = item.width / 2;
            // First line
            ctx.moveTo(pageX + item.x1, pageY + item.y1);
            ctx.lineTo(pageX + item.x2, pageY + item.y2);
            ctx.stroke();
            // Second line (offset)
            ctx.beginPath();
            if (item.y1 === item.y2) {
              // Horizontal border
              ctx.moveTo(pageX + item.x1, pageY + item.y1 + offset * 2);
              ctx.lineTo(pageX + item.x2, pageY + item.y2 + offset * 2);
            } else {
              // Vertical border
              ctx.moveTo(pageX + item.x1 + offset * 2, pageY + item.y1);
              ctx.lineTo(pageX + item.x2 + offset * 2, pageY + item.y2);
            }
            ctx.stroke();
            break;
          } else {
            ctx.setLineDash([]);
          }

          ctx.beginPath();
          ctx.moveTo(pageX + item.x1, pageY + item.y1);
          ctx.lineTo(pageX + item.x2, pageY + item.y2);
          ctx.stroke();
          ctx.setLineDash([]);
          break;
        }

        case 'Shape': {
          // Render shape with all its properties
          const { bounds, shape_type, rotation, fill, stroke, shadow, opacity, selected, flip_horizontal, flip_vertical } = item;
          const shapeX = pageX + bounds.x;
          const shapeY = pageY + bounds.y;
          const shapeW = bounds.width;
          const shapeH = bounds.height;

          ctx.save();

          // Apply opacity
          ctx.globalAlpha = opacity;

          // Apply transformations (rotation and flipping)
          if (rotation !== 0 || flip_horizontal || flip_vertical) {
            ctx.translate(shapeX + shapeW / 2, shapeY + shapeH / 2);
            if (rotation !== 0) {
              ctx.rotate((rotation * Math.PI) / 180);
            }
            if (flip_horizontal) ctx.scale(-1, 1);
            if (flip_vertical) ctx.scale(1, -1);
            ctx.translate(-shapeW / 2, -shapeH / 2);
          } else {
            ctx.translate(shapeX, shapeY);
          }

          // Draw shadow if present
          if (shadow) {
            ctx.shadowColor = colorToCss(shadow.color);
            ctx.shadowBlur = shadow.blur_radius;
            ctx.shadowOffsetX = shadow.offset_x;
            ctx.shadowOffsetY = shadow.offset_y;
          }

          // Draw the shape path based on type
          ctx.beginPath();
          const kind = shape_type.kind;

          switch (kind) {
            case 'Rectangle':
              ctx.rect(0, 0, shapeW, shapeH);
              break;
            case 'RoundedRectangle': {
              const radius = (shape_type as { kind: 'RoundedRectangle'; corner_radius: number }).corner_radius;
              const r = Math.min(radius, shapeW / 2, shapeH / 2);
              ctx.moveTo(r, 0);
              ctx.lineTo(shapeW - r, 0);
              ctx.quadraticCurveTo(shapeW, 0, shapeW, r);
              ctx.lineTo(shapeW, shapeH - r);
              ctx.quadraticCurveTo(shapeW, shapeH, shapeW - r, shapeH);
              ctx.lineTo(r, shapeH);
              ctx.quadraticCurveTo(0, shapeH, 0, shapeH - r);
              ctx.lineTo(0, r);
              ctx.quadraticCurveTo(0, 0, r, 0);
              break;
            }
            case 'Oval':
              ctx.ellipse(shapeW / 2, shapeH / 2, shapeW / 2, shapeH / 2, 0, 0, Math.PI * 2);
              break;
            case 'Line':
              ctx.moveTo(0, shapeH);
              ctx.lineTo(shapeW, 0);
              break;
            case 'Arrow':
              ctx.moveTo(0, shapeH / 2);
              ctx.lineTo(shapeW - 10, shapeH / 2);
              // Arrow head
              ctx.moveTo(shapeW, shapeH / 2);
              ctx.lineTo(shapeW - 10, shapeH / 2 - 5);
              ctx.moveTo(shapeW, shapeH / 2);
              ctx.lineTo(shapeW - 10, shapeH / 2 + 5);
              break;
            case 'DoubleArrow':
              ctx.moveTo(10, shapeH / 2);
              ctx.lineTo(shapeW - 10, shapeH / 2);
              // Right arrow head
              ctx.moveTo(shapeW, shapeH / 2);
              ctx.lineTo(shapeW - 10, shapeH / 2 - 5);
              ctx.moveTo(shapeW, shapeH / 2);
              ctx.lineTo(shapeW - 10, shapeH / 2 + 5);
              // Left arrow head
              ctx.moveTo(0, shapeH / 2);
              ctx.lineTo(10, shapeH / 2 - 5);
              ctx.moveTo(0, shapeH / 2);
              ctx.lineTo(10, shapeH / 2 + 5);
              break;
            case 'Triangle':
              ctx.moveTo(shapeW / 2, 0);
              ctx.lineTo(shapeW, shapeH);
              ctx.lineTo(0, shapeH);
              ctx.closePath();
              break;
            case 'Diamond':
              ctx.moveTo(shapeW / 2, 0);
              ctx.lineTo(shapeW, shapeH / 2);
              ctx.lineTo(shapeW / 2, shapeH);
              ctx.lineTo(0, shapeH / 2);
              ctx.closePath();
              break;
            case 'Pentagon': {
              const pentRadius = Math.min(shapeW, shapeH) / 2;
              const pentCx = shapeW / 2;
              const pentCy = shapeH / 2;
              for (let i = 0; i < 5; i++) {
                const angle = (i * 2 * Math.PI) / 5 - Math.PI / 2;
                const x = pentCx + pentRadius * Math.cos(angle);
                const y = pentCy + pentRadius * Math.sin(angle);
                if (i === 0) ctx.moveTo(x, y);
                else ctx.lineTo(x, y);
              }
              ctx.closePath();
              break;
            }
            case 'Hexagon': {
              const hexRadius = Math.min(shapeW, shapeH) / 2;
              const hexCx = shapeW / 2;
              const hexCy = shapeH / 2;
              for (let i = 0; i < 6; i++) {
                const angle = (i * 2 * Math.PI) / 6 - Math.PI / 2;
                const x = hexCx + hexRadius * Math.cos(angle);
                const y = hexCy + hexRadius * Math.sin(angle);
                if (i === 0) ctx.moveTo(x, y);
                else ctx.lineTo(x, y);
              }
              ctx.closePath();
              break;
            }
            case 'Star': {
              const starData = shape_type as { kind: 'Star'; points: number; inner_radius_ratio: number };
              const starCx = shapeW / 2;
              const starCy = shapeH / 2;
              const outerRadius = Math.min(shapeW, shapeH) / 2;
              const innerRadius = outerRadius * starData.inner_radius_ratio;
              const numPoints = starData.points;
              for (let i = 0; i < numPoints * 2; i++) {
                const angle = (i * Math.PI) / numPoints - Math.PI / 2;
                const r = i % 2 === 0 ? outerRadius : innerRadius;
                const x = starCx + r * Math.cos(angle);
                const y = starCy + r * Math.sin(angle);
                if (i === 0) ctx.moveTo(x, y);
                else ctx.lineTo(x, y);
              }
              ctx.closePath();
              break;
            }
            case 'TextBox':
              ctx.rect(0, 0, shapeW, shapeH);
              break;
            case 'RightArrowBlock':
              ctx.moveTo(0, shapeH * 0.25);
              ctx.lineTo(shapeW * 0.6, shapeH * 0.25);
              ctx.lineTo(shapeW * 0.6, 0);
              ctx.lineTo(shapeW, shapeH / 2);
              ctx.lineTo(shapeW * 0.6, shapeH);
              ctx.lineTo(shapeW * 0.6, shapeH * 0.75);
              ctx.lineTo(0, shapeH * 0.75);
              ctx.closePath();
              break;
            case 'LeftArrowBlock':
              ctx.moveTo(shapeW, shapeH * 0.25);
              ctx.lineTo(shapeW * 0.4, shapeH * 0.25);
              ctx.lineTo(shapeW * 0.4, 0);
              ctx.lineTo(0, shapeH / 2);
              ctx.lineTo(shapeW * 0.4, shapeH);
              ctx.lineTo(shapeW * 0.4, shapeH * 0.75);
              ctx.lineTo(shapeW, shapeH * 0.75);
              ctx.closePath();
              break;
            case 'UpArrowBlock':
              ctx.moveTo(shapeW * 0.25, shapeH);
              ctx.lineTo(shapeW * 0.25, shapeH * 0.4);
              ctx.lineTo(0, shapeH * 0.4);
              ctx.lineTo(shapeW / 2, 0);
              ctx.lineTo(shapeW, shapeH * 0.4);
              ctx.lineTo(shapeW * 0.75, shapeH * 0.4);
              ctx.lineTo(shapeW * 0.75, shapeH);
              ctx.closePath();
              break;
            case 'DownArrowBlock':
              ctx.moveTo(shapeW * 0.25, 0);
              ctx.lineTo(shapeW * 0.25, shapeH * 0.6);
              ctx.lineTo(0, shapeH * 0.6);
              ctx.lineTo(shapeW / 2, shapeH);
              ctx.lineTo(shapeW, shapeH * 0.6);
              ctx.lineTo(shapeW * 0.75, shapeH * 0.6);
              ctx.lineTo(shapeW * 0.75, 0);
              ctx.closePath();
              break;
            case 'Callout': {
              const calloutData = shape_type as { kind: 'Callout'; tail_position: [number, number]; tail_width: number };
              const tailX = calloutData.tail_position[0] * shapeW;
              const tailY = calloutData.tail_position[1] * shapeH;
              // Draw main rectangle
              ctx.rect(0, 0, shapeW, shapeH * 0.8);
              // Draw tail
              ctx.moveTo(shapeW * 0.4, shapeH * 0.8);
              ctx.lineTo(tailX, tailY);
              ctx.lineTo(shapeW * 0.5, shapeH * 0.8);
              break;
            }
            default:
              ctx.rect(0, 0, shapeW, shapeH);
          }

          // Apply fill
          if (fill && fill.type !== 'None') {
            if (fill.type === 'Solid') {
              ctx.fillStyle = colorToCss(fill.color);
            } else if (fill.type === 'Gradient') {
              const gradient = ctx.createLinearGradient(0, 0, shapeW, shapeH);
              fill.colors.forEach(([color, stop]) => {
                gradient.addColorStop(stop, colorToCss(color));
              });
              ctx.fillStyle = gradient;
            }
            ctx.fill();
          }

          // Reset shadow before stroke
          ctx.shadowColor = 'transparent';
          ctx.shadowBlur = 0;

          // Apply stroke
          if (stroke) {
            ctx.strokeStyle = colorToCss(stroke.color);
            ctx.lineWidth = stroke.width;

            // Apply dash style
            switch (stroke.dash_style) {
              case 'Dash':
                ctx.setLineDash([8, 4]);
                break;
              case 'Dot':
                ctx.setLineDash([2, 2]);
                break;
              case 'DashDot':
                ctx.setLineDash([8, 4, 2, 4]);
                break;
              case 'DashDotDot':
                ctx.setLineDash([8, 4, 2, 4, 2, 4]);
                break;
              default:
                ctx.setLineDash([]);
            }
            ctx.stroke();
            ctx.setLineDash([]);
          }

          ctx.restore();

          // Draw selection handles if selected
          if (selected) {
            ctx.strokeStyle = '#0066cc';
            ctx.lineWidth = 2;
            ctx.strokeRect(shapeX - 1, shapeY - 1, shapeW + 2, shapeH + 2);

            // Draw resize handles
            const handleSize = 8;
            ctx.fillStyle = '#0066cc';
            // Corner handles
            ctx.fillRect(shapeX - handleSize / 2, shapeY - handleSize / 2, handleSize, handleSize);
            ctx.fillRect(shapeX + shapeW - handleSize / 2, shapeY - handleSize / 2, handleSize, handleSize);
            ctx.fillRect(shapeX - handleSize / 2, shapeY + shapeH - handleSize / 2, handleSize, handleSize);
            ctx.fillRect(shapeX + shapeW - handleSize / 2, shapeY + shapeH - handleSize / 2, handleSize, handleSize);
            // Edge handles
            ctx.fillRect(shapeX + shapeW / 2 - handleSize / 2, shapeY - handleSize / 2, handleSize, handleSize);
            ctx.fillRect(shapeX + shapeW / 2 - handleSize / 2, shapeY + shapeH - handleSize / 2, handleSize, handleSize);
            ctx.fillRect(shapeX - handleSize / 2, shapeY + shapeH / 2 - handleSize / 2, handleSize, handleSize);
            ctx.fillRect(shapeX + shapeW - handleSize / 2, shapeY + shapeH / 2 - handleSize / 2, handleSize, handleSize);

            // Draw rotation handle
            ctx.beginPath();
            ctx.arc(shapeX + shapeW / 2, shapeY - 20, 6, 0, Math.PI * 2);
            ctx.fill();
            ctx.beginPath();
            ctx.moveTo(shapeX + shapeW / 2, shapeY);
            ctx.lineTo(shapeX + shapeW / 2, shapeY - 14);
            ctx.stroke();
          }
          break;
        }

        case 'Caret': {
          // Only render if caret is visible (for blinking)
          // Caret rendering is handled separately to support blinking
          break;
        }
      }
    },
    []
  );

  // Render a single page (used by scheduler for page caching)
  const renderPage = useCallback(
    (
      ctx: CanvasRenderingContext2D,
      page: PageRender,
      pageX: number,
      pageY: number
    ) => {
      // Draw page shadow
      ctx.shadowColor = 'rgba(0, 0, 0, 0.2)';
      ctx.shadowBlur = PAGE_SHADOW_BLUR;
      ctx.shadowOffsetX = 2;
      ctx.shadowOffsetY = 2;

      // Draw page background (white)
      ctx.fillStyle = '#ffffff';
      ctx.fillRect(pageX, pageY, page.width, page.height);

      // Reset shadow for content
      ctx.shadowColor = 'transparent';
      ctx.shadowBlur = 0;
      ctx.shadowOffsetX = 0;
      ctx.shadowOffsetY = 0;

      // Draw margin guides in print layout mode (dotted lines)
      if (viewMode === 'print-layout') {
        ctx.strokeStyle = '#e0e0e0';
        ctx.lineWidth = 0.5;
        ctx.setLineDash([4, 4]);

        // Left margin
        ctx.beginPath();
        ctx.moveTo(pageX + DEFAULT_MARGINS.left, pageY);
        ctx.lineTo(pageX + DEFAULT_MARGINS.left, pageY + page.height);
        ctx.stroke();

        // Right margin
        ctx.beginPath();
        ctx.moveTo(pageX + page.width - DEFAULT_MARGINS.right, pageY);
        ctx.lineTo(pageX + page.width - DEFAULT_MARGINS.right, pageY + page.height);
        ctx.stroke();

        // Top margin
        ctx.beginPath();
        ctx.moveTo(pageX, pageY + DEFAULT_MARGINS.top);
        ctx.lineTo(pageX + page.width, pageY + DEFAULT_MARGINS.top);
        ctx.stroke();

        // Bottom margin
        ctx.beginPath();
        ctx.moveTo(pageX, pageY + page.height - DEFAULT_MARGINS.bottom);
        ctx.lineTo(pageX + page.width, pageY + page.height - DEFAULT_MARGINS.bottom);
        ctx.stroke();

        ctx.setLineDash([]);
      }

      // Render all items on this page (except carets which are rendered separately)
      for (const item of page.items) {
        renderItem(ctx, item, pageX, pageY);
      }
    },
    [renderItem, viewMode]
  );

  // Render carets (separate for blinking support)
  const renderCarets = useCallback(
    (
      ctx: CanvasRenderingContext2D,
      layouts: PageLayout[],
      visible: boolean,
      offsetX: number,
      offsetY: number,
      currentZoom: number
    ) => {
      if (!visible) return;

      let caretFound = false;
      for (const layout of layouts) {
        for (const item of layout.page.items) {
          if (item.type === 'Caret') {
            caretFound = true;
            // Use item color if available, fallback to black
            try {
              ctx.fillStyle = item.color ? colorToCss(item.color) : 'rgba(0, 0, 0, 1)';
            } catch {
              ctx.fillStyle = 'rgba(0, 0, 0, 1)';
            }

            let caretX: number;
            const caretY = (layout.y + item.y * currentZoom) - offsetY;
            const caretHeight = (item.height || 20) * currentZoom;

            // Use measureText for accurate proportional-font caret positioning
            if (item.line_text !== undefined && item.char_offset_in_line !== undefined) {
              // Find a GlyphRun on the same line to match font settings
              let fontFamily = 'sans-serif';
              let fontSize = 14;
              let bold = false;
              let italic = false;
              const MARGIN_PX = 96; // Must match backend MARGIN constant

              for (const gi of layout.page.items) {
                if (gi.type === 'GlyphRun' && Math.abs(gi.y - (item.y + 16)) < 2) {
                  fontFamily = gi.font_family || 'sans-serif';
                  fontSize = gi.font_size;
                  bold = gi.bold;
                  italic = gi.italic;
                  break;
                }
              }

              const fontStyle = italic ? 'italic' : 'normal';
              const fontWeight = bold ? 'bold' : 'normal';
              ctx.font = `${fontStyle} ${fontWeight} ${fontSize}px ${fontFamily}`;
              ctx.textBaseline = 'alphabetic';

              const textBeforeCursor = item.line_text.substring(0, item.char_offset_in_line);
              const measuredWidth = ctx.measureText(textBeforeCursor).width;
              caretX = (layout.x + (MARGIN_PX + measuredWidth) * currentZoom) - offsetX;
            } else {
              // Fallback to backend-provided x position
              caretX = (layout.x + item.x * currentZoom) - offsetX;
            }

            ctx.fillRect(caretX, caretY, 2, caretHeight);
          }
        }
      }

      // Debug: log once if no caret found (helps diagnose cursor visibility issues)
      if (!caretFound && layouts.length > 0) {
        console.warn('[EditorCanvas] renderCarets: No Caret item found in page items. Items types:',
          layouts.flatMap(l => l.page.items.map(i => i.type)));
      }
    },
    []
  );

  // Render IME composition preview text
  const renderCompositionPreview = useCallback(
    (ctx: CanvasRenderingContext2D, layouts: PageLayout[], offsetX: number, offsetY: number, currentZoom: number) => {
      if (!localCompositionState.isComposing || !localCompositionState.compositionText) {
        return;
      }

      // Find the caret position to render composition text
      for (const layout of layouts) {
        for (const item of layout.page.items) {
          if (item.type === 'Caret') {
            // Render composition text with underline at caret position
            const fontSize = 12 * currentZoom;
            ctx.font = `${fontSize}px sans-serif`;
            ctx.fillStyle = '#333';
            ctx.textBaseline = 'alphabetic';

            const text = localCompositionState.compositionText;

            // Calculate accurate x position using measureText (same as renderCarets)
            let caretPixelX: number;
            const MARGIN_PX = 96;
            if (item.line_text !== undefined && item.char_offset_in_line !== undefined) {
              // Find matching GlyphRun for font settings
              let gFont = `${fontSize}px sans-serif`;
              for (const gi of layout.page.items) {
                if (gi.type === 'GlyphRun' && Math.abs(gi.y - (item.y + 16)) < 2) {
                  const fs = gi.italic ? 'italic' : 'normal';
                  const fw = gi.bold ? 'bold' : 'normal';
                  gFont = `${fs} ${fw} ${gi.font_size}px ${gi.font_family || 'sans-serif'}`;
                  break;
                }
              }
              ctx.font = gFont;
              const textBeforeCursor = item.line_text.substring(0, item.char_offset_in_line);
              const measuredWidth = ctx.measureText(textBeforeCursor).width;
              caretPixelX = (layout.x + (MARGIN_PX + measuredWidth) * currentZoom) - offsetX;
              // Reset font for composition text
              ctx.font = `${fontSize}px sans-serif`;
            } else {
              caretPixelX = (layout.x + item.x * currentZoom) - offsetX;
            }

            const x = caretPixelX;
            const y = (layout.y + (item.y + item.height - 4) * currentZoom) - offsetY;

            // Draw the composition text
            ctx.fillText(text, x, y);

            // Draw underline to indicate composition
            const textWidth = ctx.measureText(text).width;
            ctx.strokeStyle = '#333';
            ctx.lineWidth = 1;
            ctx.setLineDash([2, 2]); // Dashed underline
            ctx.beginPath();
            ctx.moveTo(x, y + 2);
            ctx.lineTo(x + textWidth, y + 2);
            ctx.stroke();
            ctx.setLineDash([]); // Reset dash

            return; // Only render at first caret found
          }
        }
      }
    },
    [localCompositionState]
  );

  // Main render function
  const render = useCallback(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Get dimensions
    const canvasWidth = container.clientWidth;
    const canvasHeight = container.clientHeight;

    // Clear canvas with background color
    ctx.save();
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.fillStyle = '#e8e8e8'; // Gray background (typical document editor)
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.restore();

    if (!renderModel || renderModel.pages.length === 0) {
      // Show placeholder
      ctx.fillStyle = '#666';
      ctx.font = '16px system-ui';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText('Loading document...', canvasWidth / 2, canvasHeight / 2);
      return;
    }

    // Calculate page layouts
    const layouts = calculatePageLayouts(canvasWidth);

    // Update scheduler with layouts
    setPageLayouts(layouts);

    // Get visible page indices from virtualization hook for optimized rendering
    // This is more efficient than the scheduler's calculation for large documents
    const virtualizedVisibleSet = new Set([...visiblePageIndices, ...bufferedPageIndices]);

    // Apply scroll transform
    ctx.save();
    ctx.translate(-scrollOffset.x, -scrollOffset.y);

    // Apply zoom transform
    ctx.scale(zoom, zoom);

    // Render only virtualized pages (visible + buffer)
    for (const layout of layouts) {
      const { page, x, y } = layout;
      const pageIndex = page.page_index;

      // Use virtualization hook to determine if page should be rendered
      // This provides better performance for large documents
      if (!virtualShouldRenderPage(pageIndex)) {
        continue; // Skip pages outside virtualized range
      }

      // Double-check with geometric bounds for safety
      // Note: layout.y is already in screen-space (accounts for zoom in page stacking),
      // so we don't multiply by zoom again for the visibility check
      const pageTopPos = y - scrollOffset.y;
      const scaledHeight = page.height * zoom;
      const pageBottomPos = pageTopPos + scaledHeight;

      // Skip pages that are definitely off-screen (with generous buffer)
      if (pageBottomPos < -scaledHeight * 2 || pageTopPos > canvasHeight + scaledHeight * 2) {
        continue;
      }

      // Render the page (coordinates are in unzoomed space, transform handles zoom)
      const unzoomedX = x / zoom;
      const unzoomedY = y / zoom;
      renderPage(ctx, page, unzoomedX, unzoomedY);

      // Mark page as rendered in cache for memory management
      pageRenderCacheRef.current.markPageRendered(pageIndex);
    }

    ctx.restore();

    // Render carets (on top of everything, with blinking) - rendered without zoom transform
    // We apply zoom manually to caret positions for precise rendering
    ctx.save();
    renderCarets(ctx, layouts, caretVisible, scrollOffset.x, scrollOffset.y, zoom);
    ctx.restore();

    // Render IME composition preview
    ctx.save();
    renderCompositionPreview(ctx, layouts, scrollOffset.x, scrollOffset.y, zoom);
    ctx.restore();
  }, [
    renderModel,
    scrollOffset,
    caretVisible,
    zoom,
    calculatePageLayouts,
    renderPage,
    renderCarets,
    renderCompositionPreview,
    setPageLayouts,
    visiblePageIndices,
    bufferedPageIndices,
    virtualShouldRenderPage,
  ]);

  // Convert screen coordinates to document coordinates (accounting for zoom and scroll)
  const screenToDocumentCoords = useCallback(
    (screenX: number, screenY: number): { x: number; y: number } => {
      const rulerOffset = showRulers && viewMode === 'print-layout' ? RULER_SIZE : 0;
      return {
        x: (screenX + scrollOffset.x - rulerOffset) / zoom,
        y: (screenY + scrollOffset.y - rulerOffset) / zoom,
      };
    },
    [scrollOffset, zoom, showRulers, viewMode]
  );

  // Find hyperlink at screen position
  const findHyperlinkAtPosition = useCallback(
    (screenX: number, screenY: number): HyperlinkRenderInfo | null => {
      if (!renderModel) return null;

      const canvas = canvasRef.current;
      if (!canvas) return null;

      const ctx = canvas.getContext('2d');
      if (!ctx) return null;

      const canvasWidth = containerSize.width;
      const layouts = calculatePageLayouts(canvasWidth);

      // Convert screen coords to canvas coords (accounting for scroll)
      const canvasX = screenX + scrollOffset.x;
      const canvasY = screenY + scrollOffset.y;

      for (const layout of layouts) {
        // Check if point is within this page (accounting for zoom)
        const pageLeft = layout.x;
        const pageTop = layout.y;
        const pageRight = pageLeft + layout.width * zoom;
        const pageBottom = pageTop + layout.height * zoom;

        if (canvasX >= pageLeft && canvasX <= pageRight && canvasY >= pageTop && canvasY <= pageBottom) {
          // Convert to page-relative coords
          const pageRelativeX = (canvasX - pageLeft) / zoom;
          const pageRelativeY = (canvasY - pageTop) / zoom;

          // Check each render item for hyperlinks
          for (const item of layout.page.items) {
            if (item.type === 'GlyphRun' && item.hyperlink) {
              // Calculate text bounds
              const fontStyle = item.italic ? 'italic' : 'normal';
              const fontWeight = item.bold ? 'bold' : 'normal';
              const fontSize = item.font_size;
              const fontFamily = item.font_family || 'sans-serif';
              ctx.font = `${fontStyle} ${fontWeight} ${fontSize}px ${fontFamily}`;

              const textWidth = ctx.measureText(item.text).width;
              const textHeight = fontSize; // Approximate

              // Check if point is within text bounds
              if (
                pageRelativeX >= item.x &&
                pageRelativeX <= item.x + textWidth &&
                pageRelativeY >= item.y - textHeight &&
                pageRelativeY <= item.y + 4 // A bit below baseline for underline
              ) {
                return item.hyperlink;
              }
            }
          }
        }
      }

      return null;
    },
    [renderModel, containerSize.width, calculatePageLayouts, scrollOffset, zoom]
  );

  // Handle keyboard input - wraps the input controller with caret blink reset
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      // Reset caret blink on keystroke
      lastKeyTimeRef.current = Date.now();
      setCaretVisible(true);

      // Delegate to input controller
      inputHandleKeyDown(e);
    },
    [inputHandleKeyDown]
  );

  // Handle IME composition events - wraps the input controller with caret management
  const handleCompositionStart = useCallback(
    (e: React.CompositionEvent) => {
      // Keep caret visible during composition
      setCaretVisible(true);
      inputHandleCompositionStart(e);
    },
    [inputHandleCompositionStart]
  );

  const handleCompositionUpdate = useCallback(
    (e: React.CompositionEvent) => {
      // Keep caret visible during composition
      setCaretVisible(true);
      inputHandleCompositionUpdate(e);
    },
    [inputHandleCompositionUpdate]
  );

  const handleCompositionEnd = useCallback(
    (e: React.CompositionEvent) => {
      // Reset blink timer after composition
      lastKeyTimeRef.current = Date.now();
      inputHandleCompositionEnd(e);
    },
    [inputHandleCompositionEnd]
  );

  // Handle input events (for beforeinput support)
  const handleInput = useCallback(
    (e: React.FormEvent<HTMLCanvasElement>) => {
      lastKeyTimeRef.current = Date.now();
      setCaretVisible(true);
      inputHandleInput(e);
    },
    [inputHandleInput]
  );

  // Hit-test a mouse click against GlyphRun items to find paragraph + char offset
  const hitTestClick = useCallback(
    (screenX: number, screenY: number): { paragraph: number; offset: number } | null => {
      if (!renderModel) return null;

      const canvas = canvasRef.current;
      if (!canvas) return null;

      const ctx = canvas.getContext('2d');
      if (!ctx) return null;

      const canvasWidth = containerSize.width;
      const layouts = calculatePageLayouts(canvasWidth);

      // Convert screen coords to canvas coords (accounting for scroll)
      const canvasX = screenX + scrollOffset.x;
      const canvasY = screenY + scrollOffset.y;

      const MARGIN_PX = 96; // Must match backend MARGIN constant
      const LINE_HEIGHT_PX = 22; // Must match backend LINE_HEIGHT constant

      for (const layout of layouts) {
        // Check if point is within this page (accounting for zoom)
        const pageLeft = layout.x;
        const pageTop = layout.y;
        const pageRight = pageLeft + layout.width * zoom;
        const pageBottom = pageTop + layout.height * zoom;

        if (canvasX >= pageLeft && canvasX <= pageRight && canvasY >= pageTop && canvasY <= pageBottom) {
          // Convert to page-relative coords (unzoomed)
          const pageRelativeX = (canvasX - pageLeft) / zoom;
          const pageRelativeY = (canvasY - pageTop) / zoom;

          // Find the closest GlyphRun by y position
          // GlyphRun y is the baseline: MARGIN + visual_line * LINE_HEIGHT + 16
          // So the line top is approximately y - font_size, line bottom is y + 4
          let bestGlyph: (typeof layout.page.items)[number] | null = null;
          let bestYDist = Infinity;

          for (const item of layout.page.items) {
            if (item.type === 'GlyphRun' && item.para_index !== undefined) {
              const lineTop = item.y - item.font_size;
              const lineBottom = item.y + 6;
              if (pageRelativeY >= lineTop && pageRelativeY <= lineBottom) {
                const yDist = Math.abs(pageRelativeY - item.y);
                if (yDist < bestYDist) {
                  bestYDist = yDist;
                  bestGlyph = item;
                }
              }
            }
          }

          if (bestGlyph && bestGlyph.type === 'GlyphRun' && bestGlyph.para_index !== undefined) {
            // Set up the font to match the GlyphRun for accurate measurement
            const fontStyle = bestGlyph.italic ? 'italic' : 'normal';
            const fontWeight = bestGlyph.bold ? 'bold' : 'normal';
            ctx.font = `${fontStyle} ${fontWeight} ${bestGlyph.font_size}px ${bestGlyph.font_family || 'sans-serif'}`;

            const textX = bestGlyph.x; // MARGIN
            const relativeClickX = pageRelativeX - textX;

            if (relativeClickX <= 0) {
              // Clicked before the text start
              return {
                paragraph: bestGlyph.para_index,
                offset: bestGlyph.line_start_char_offset ?? 0,
              };
            }

            // Binary search for the character position
            const text = bestGlyph.text;
            let charOffset = 0;
            for (let i = 0; i <= text.length; i++) {
              const width = ctx.measureText(text.substring(0, i)).width;
              if (width > relativeClickX) {
                // Check if we're closer to this char or the previous one
                const prevWidth = i > 0 ? ctx.measureText(text.substring(0, i - 1)).width : 0;
                charOffset = (relativeClickX - prevWidth < width - relativeClickX) ? i - 1 : i;
                return {
                  paragraph: bestGlyph.para_index,
                  offset: (bestGlyph.line_start_char_offset ?? 0) + charOffset,
                };
              }
            }

            // Clicked past the end of text on this line
            return {
              paragraph: bestGlyph.para_index,
              offset: (bestGlyph.line_start_char_offset ?? 0) + text.length,
            };
          }

          // If no GlyphRun matched, check if click is in an empty line area
          // Calculate which visual line was clicked
          const visualLine = Math.floor((pageRelativeY - MARGIN_PX) / LINE_HEIGHT_PX);
          if (visualLine >= 0) {
            // Find the GlyphRun closest to this visual line or check for empty paragraphs
            // For empty paragraphs there won't be a GlyphRun, so position at start
            let currentLine = 0;
            for (const item of layout.page.items) {
              if (item.type === 'GlyphRun' && item.para_index !== undefined) {
                if (currentLine === visualLine) {
                  return {
                    paragraph: item.para_index,
                    offset: item.line_start_char_offset ?? 0,
                  };
                }
                currentLine++;
              }
            }

            // Clicked below all content - position at end of last paragraph
            const lastGlyph = [...layout.page.items]
              .reverse()
              .find(i => i.type === 'GlyphRun' && i.para_index !== undefined);
            if (lastGlyph && lastGlyph.type === 'GlyphRun' && lastGlyph.para_index !== undefined) {
              return {
                paragraph: lastGlyph.para_index,
                offset: (lastGlyph.line_start_char_offset ?? 0) + lastGlyph.text.length,
              };
            }
          }
        }
      }

      return null;
    },
    [renderModel, containerSize.width, calculatePageLayouts, scrollOffset, zoom]
  );

  // Handle mouse events for selection
  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      // Focus the canvas
      canvasRef.current?.focus();

      // Reset caret blink on click
      lastKeyTimeRef.current = Date.now();
      setCaretVisible(true);

      // Convert to document coordinates
      const rect = canvasRef.current?.getBoundingClientRect();
      if (rect) {
        const screenX = e.clientX - rect.left;
        const screenY = e.clientY - rect.top;

        // Check for hyperlink click
        const hyperlink = findHyperlinkAtPosition(screenX, screenY);
        if (hyperlink && onHyperlinkClick) {
          const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
          const ctrlKey = isMac ? e.metaKey : e.ctrlKey;
          onHyperlinkClick(hyperlink, ctrlKey);
          e.preventDefault();
          return;
        }

        // Hit-test to find paragraph and character offset
        const hitResult = hitTestClick(screenX, screenY);
        if (hitResult) {
          onCommand({
            type: 'SetCursorPosition',
            paragraph: hitResult.paragraph,
            offset: hitResult.offset,
          });
        }
      }
    },
    [hitTestClick, findHyperlinkAtPosition, onHyperlinkClick, onCommand]
  );

  // Handle mouse move for cursor changes over hyperlinks
  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      const rect = canvasRef.current?.getBoundingClientRect();
      if (rect) {
        const screenX = e.clientX - rect.left;
        const screenY = e.clientY - rect.top;
        const hyperlink = findHyperlinkAtPosition(screenX, screenY);

        // Change cursor when over hyperlink
        if (canvasRef.current) {
          canvasRef.current.style.cursor = hyperlink ? 'pointer' : 'text';
        }
      }
    },
    [findHyperlinkAtPosition]
  );

  // Handle wheel events (for zoom with Ctrl/Cmd)
  const handleWheel = useCallback(
    (e: React.WheelEvent) => {
      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const modifierKey = isMac ? e.metaKey : e.ctrlKey;

      if (modifierKey && onWheelZoom) {
        if (onWheelZoom(e.deltaY, true)) {
          e.preventDefault();
        }
      }
    },
    [onWheelZoom]
  );

  // Handle scroll with scheduler integration
  const handleScroll = useCallback(
    (e: React.UIEvent<HTMLDivElement>) => {
      const target = e.target as HTMLDivElement;
      const newScrollX = target.scrollLeft;
      const newScrollY = target.scrollTop;

      setScrollOffset({
        x: newScrollX,
        y: newScrollY,
      });

      // Update scheduler viewport
      const container = containerRef.current;
      if (container) {
        schedulerHandleScroll(
          newScrollX,
          newScrollY,
          container.clientWidth,
          container.clientHeight
        );
      }
    },
    [schedulerHandleScroll]
  );

  // Keep a stable ref to the latest render function so the resize effect
  // doesn't need render in its dependency array (which changes every frame).
  const renderRef = useRef(render);
  useEffect(() => {
    renderRef.current = render;
  }, [render]);

  // Stable ref for onContainerResize to avoid re-running the resize effect
  const onContainerResizeRef = useRef(onContainerResize);
  useEffect(() => {
    onContainerResizeRef.current = onContainerResize;
  }, [onContainerResize]);

  // Set up canvas sizing and high DPI support
  useEffect(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;

    const resizeCanvas = () => {
      const dpr = window.devicePixelRatio || 1;
      const width = container.clientWidth;
      const height = container.clientHeight;

      // Update container size state only if changed
      setContainerSize(prev =>
        prev.width === width && prev.height === height
          ? prev
          : { width, height }
      );

      // Notify parent of resize
      onContainerResizeRef.current?.(width, height);

      // Set canvas size accounting for device pixel ratio
      canvas.width = width * dpr;
      canvas.height = height * dpr;
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;

      // Scale the context to handle high DPI
      const ctx = canvas.getContext('2d');
      if (ctx) {
        ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      }

      // Update scheduler canvas reference
      setCanvas(canvas);

      // Mark global dirty on resize
      markGlobalDirty();

      renderRef.current();
    };

    resizeCanvas();

    // Use ResizeObserver for more accurate resize detection
    const resizeObserver = new ResizeObserver(() => {
      resizeCanvas();
    });
    resizeObserver.observe(container);

    return () => {
      resizeObserver.disconnect();
    };
  }, [setCanvas, markGlobalDirty]);

  // Update scheduler when render model changes
  useEffect(() => {
    setSchedulerRenderModel(renderModel);
    if (renderModel) {
      markGlobalDirty();
      // Ensure caret is visible when a new render model arrives
      // (prevents the blink timer from hiding it before first paint)
      setCaretVisible(true);
      lastKeyTimeRef.current = Date.now();
    }
  }, [renderModel, setSchedulerRenderModel, markGlobalDirty]);

  // Handle dirty pages from document changes
  useEffect(() => {
    if (dirtyPages && dirtyPages.length > 0) {
      markPagesDirty(dirtyPages);
    }
  }, [dirtyPages, markPagesDirty]);

  // NOTE: We intentionally do NOT set a render callback on the scheduler.
  // The scheduler's independent RAF-based render loop clears the entire canvas
  // and redraws pages, but does NOT render carets or composition previews.
  // This caused the caret to be erased immediately after being drawn by the
  // component's render() function. All rendering is handled by the component's
  // own render() function which properly draws pages, carets, and overlays.

  // Re-render when zoom changes
  useEffect(() => {
    markGlobalDirty();
    render();
  }, [zoom, markGlobalDirty, render]);

  // Set up caret blinking
  useEffect(() => {
    const blinkCaret = () => {
      const timeSinceLastKey = Date.now() - lastKeyTimeRef.current;

      // Only blink if we haven't typed recently and not composing
      if (timeSinceLastKey > CARET_BLINK_INTERVAL && !compositionState.isComposing) {
        setCaretVisible((prev) => !prev);
      } else {
        setCaretVisible(true);
      }
    };

    caretBlinkRef.current = window.setInterval(blinkCaret, CARET_BLINK_INTERVAL);

    return () => {
      if (caretBlinkRef.current !== null) {
        window.clearInterval(caretBlinkRef.current);
      }
    };
  }, [compositionState.isComposing]);

  // Re-render when model or caret visibility changes
  useEffect(() => {
    render();
  }, [render, caretVisible, localCompositionState]);

  // Initialize viewport on mount
  useEffect(() => {
    const container = containerRef.current;
    if (container) {
      schedulerHandleScroll(
        container.scrollLeft,
        container.scrollTop,
        container.clientWidth,
        container.clientHeight
      );
    }
  }, [schedulerHandleScroll]);

  // Auto-focus canvas on mount so the user can type immediately and see the caret
  useEffect(() => {
    // Small delay to ensure canvas is fully rendered before focusing
    const timer = setTimeout(() => {
      canvasRef.current?.focus();
    }, 100);
    return () => clearTimeout(timer);
  }, []);

  // Calculate scrollable content size
  const totalHeight = getTotalHeight();
  const totalWidth = getTotalWidth();
  const rulerOffset = showRulers && viewMode === 'print-layout' ? RULER_SIZE : 0;

  const contentStyle = {
    width: totalWidth,
    height: totalHeight,
    minHeight: '100%',
    minWidth: '100%',
  };

  // Calculate page offset for rulers
  const pageOffsetX = containerSize.width > 0
    ? Math.max(PAGE_GAP, (containerSize.width - rulerOffset - pageWidth * zoom) / 2)
    : PAGE_GAP;
  const pageOffsetY = PAGE_GAP;

  return (
    <div
      className={`editor-wrapper ${viewMode}`}
      style={{ position: 'relative', flex: 1, overflow: 'hidden' }}
    >
      {/* Rulers */}
      {viewMode === 'print-layout' && (
        <Rulers
          pageWidth={pageWidth}
          pageHeight={pageHeight}
          zoom={zoom}
          scrollX={scrollOffset.x}
          scrollY={scrollOffset.y}
          pageOffsetX={pageOffsetX}
          pageOffsetY={pageOffsetY}
          marginLeft={DEFAULT_MARGINS.left}
          marginRight={DEFAULT_MARGINS.right}
          marginTop={DEFAULT_MARGINS.top}
          marginBottom={DEFAULT_MARGINS.bottom}
          visible={showRulers}
        />
      )}

      {/* Scrollable container */}
      <div
        ref={scrollContainerRef}
        className="editor-scroll-container"
        onScroll={handleScroll}
        style={{
          position: 'absolute',
          top: rulerOffset,
          left: rulerOffset,
          right: 0,
          bottom: 0,
          overflow: 'auto',
        }}
      >
        <div
          ref={containerRef}
          className="editor-container"
          style={{ position: 'relative', width: '100%', height: '100%' }}
        >
          <div style={contentStyle}>
            <canvas
              ref={canvasRef}
              className="editor-canvas"
              tabIndex={0}
              onKeyDown={handleKeyDown}
              onCompositionStart={handleCompositionStart}
              onCompositionUpdate={handleCompositionUpdate}
              onCompositionEnd={handleCompositionEnd}
              onInput={handleInput}
              onMouseDown={handleMouseDown}
              onMouseMove={handleMouseMove}
              onWheel={handleWheel}
              style={{
                position: 'sticky',
                top: 0,
                left: 0,
                display: 'block',
                outline: 'none', // Remove focus outline, handle visually via caret
              }}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
