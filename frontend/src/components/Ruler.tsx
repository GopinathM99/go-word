/**
 * Ruler - Horizontal and vertical rulers for the document editor
 *
 * Features:
 * - Measurement marks in inches (with cm option)
 * - Margin indicators
 * - Zoom-aware scaling
 * - Tab stop indicators (visual only, for future use)
 */

import { useMemo, useCallback } from 'react';

// =============================================================================
// Types
// =============================================================================

export type RulerUnit = 'in' | 'cm';

export interface RulerProps {
  /** Ruler orientation */
  orientation: 'horizontal' | 'vertical';
  /** Length in pixels (page width or height) */
  length: number;
  /** Current zoom level */
  zoom: number;
  /** Current scroll offset */
  scrollOffset: number;
  /** Page offset (distance from edge to page) */
  pageOffset: number;
  /** Left/top margin in pixels */
  marginStart: number;
  /** Right/bottom margin in pixels */
  marginEnd: number;
  /** Unit of measurement */
  unit?: RulerUnit;
  /** DPI for conversion (default: 96) */
  dpi?: number;
  /** Whether ruler is visible */
  visible?: boolean;
}

export interface RulersProps {
  /** Page width in pixels */
  pageWidth: number;
  /** Page height in pixels */
  pageHeight: number;
  /** Current zoom level */
  zoom: number;
  /** Horizontal scroll offset */
  scrollX: number;
  /** Vertical scroll offset */
  scrollY: number;
  /** Horizontal page offset (centering) */
  pageOffsetX: number;
  /** Vertical page offset */
  pageOffsetY: number;
  /** Left margin in pixels */
  marginLeft: number;
  /** Right margin in pixels */
  marginRight: number;
  /** Top margin in pixels */
  marginTop: number;
  /** Bottom margin in pixels */
  marginBottom: number;
  /** Unit of measurement */
  unit?: RulerUnit;
  /** Whether rulers are visible */
  visible?: boolean;
}

// =============================================================================
// Constants
// =============================================================================

const RULER_SIZE = 20; // Height of horizontal ruler / Width of vertical ruler
const TICK_SMALL = 4;
const TICK_MEDIUM = 8;
const TICK_LARGE = 12;

// Pixels per inch at 96 DPI
const DEFAULT_DPI = 96;

// =============================================================================
// Helper Functions
// =============================================================================

function pixelsToUnit(pixels: number, unit: RulerUnit, dpi: number): number {
  const inches = pixels / dpi;
  return unit === 'cm' ? inches * 2.54 : inches;
}

function unitToPixels(value: number, unit: RulerUnit, dpi: number): number {
  const inches = unit === 'cm' ? value / 2.54 : value;
  return inches * dpi;
}

// =============================================================================
// Ruler Component
// =============================================================================

export function Ruler({
  orientation,
  length,
  zoom,
  scrollOffset,
  pageOffset,
  marginStart,
  marginEnd,
  unit = 'in',
  dpi = DEFAULT_DPI,
  visible = true,
}: RulerProps) {
  const isHorizontal = orientation === 'horizontal';

  // Calculate tick marks
  const ticks = useMemo(() => {
    const result: Array<{
      position: number;
      size: 'small' | 'medium' | 'large';
      label?: string;
    }> = [];

    // Unit size in pixels (at zoom 1.0)
    const unitPixels = unitToPixels(1, unit, dpi);
    const subDivisions = unit === 'cm' ? 10 : 8; // 10 mm per cm, 8ths per inch
    const subUnitPixels = unitPixels / subDivisions;

    // Calculate visible range
    const scaledLength = length * zoom;
    const totalUnits = Math.ceil(pixelsToUnit(length, unit, dpi));

    // Generate ticks for each unit
    for (let u = 0; u <= totalUnits; u++) {
      // Major tick at each unit
      const majorPos = u * unitPixels * zoom;
      if (majorPos <= scaledLength) {
        result.push({
          position: majorPos,
          size: 'large',
          label: u.toString(),
        });
      }

      // Sub-unit ticks
      for (let s = 1; s < subDivisions; s++) {
        const subPos = (u * unitPixels + s * subUnitPixels) * zoom;
        if (subPos <= scaledLength) {
          // Medium tick at half-unit, small otherwise
          const isMedium = unit === 'in' ? s === 4 : s === 5;
          result.push({
            position: subPos,
            size: isMedium ? 'medium' : 'small',
          });
        }
      }
    }

    return result;
  }, [length, zoom, unit, dpi]);

  // Calculate margin indicator positions
  const marginStartPos = marginStart * zoom;
  const marginEndPos = (length - marginEnd) * zoom;

  const getTickSize = (size: 'small' | 'medium' | 'large'): number => {
    switch (size) {
      case 'small':
        return TICK_SMALL;
      case 'medium':
        return TICK_MEDIUM;
      case 'large':
        return TICK_LARGE;
    }
  };

  if (!visible) {
    return null;
  }

  // Adjust position based on scroll and page offset
  const offset = pageOffset * zoom - scrollOffset;

  return (
    <div
      className={`ruler ruler-${orientation}`}
      style={{
        [isHorizontal ? 'width' : 'height']: length * zoom + pageOffset * zoom,
        [isHorizontal ? 'height' : 'width']: RULER_SIZE,
        [isHorizontal ? 'left' : 'top']: isHorizontal ? 0 : RULER_SIZE,
      }}
    >
      {/* Ruler background */}
      <div className="ruler-background" />

      {/* Tick marks container - positioned relative to page */}
      <svg
        className="ruler-ticks"
        style={{
          [isHorizontal ? 'left' : 'top']: offset,
          [isHorizontal ? 'width' : 'height']: length * zoom,
        }}
      >
        {ticks.map((tick, index) => {
          const tickSize = getTickSize(tick.size);
          const pos = tick.position;

          return (
            <g key={index}>
              {isHorizontal ? (
                <line
                  x1={pos}
                  y1={RULER_SIZE - tickSize}
                  x2={pos}
                  y2={RULER_SIZE}
                  className="ruler-tick"
                />
              ) : (
                <line
                  x1={RULER_SIZE - tickSize}
                  y1={pos}
                  x2={RULER_SIZE}
                  y2={pos}
                  className="ruler-tick"
                />
              )}
              {tick.label && (
                <text
                  x={isHorizontal ? pos + 2 : RULER_SIZE / 2}
                  y={isHorizontal ? RULER_SIZE - TICK_LARGE - 2 : pos + 10}
                  className="ruler-label"
                >
                  {tick.label}
                </text>
              )}
            </g>
          );
        })}

        {/* Margin indicators */}
        {isHorizontal ? (
          <>
            <rect
              x={0}
              y={0}
              width={marginStartPos}
              height={RULER_SIZE}
              className="ruler-margin"
            />
            <rect
              x={marginEndPos}
              y={0}
              width={length * zoom - marginEndPos}
              height={RULER_SIZE}
              className="ruler-margin"
            />
            {/* Margin handles */}
            <line
              x1={marginStartPos}
              y1={0}
              x2={marginStartPos}
              y2={RULER_SIZE}
              className="ruler-margin-handle"
            />
            <line
              x1={marginEndPos}
              y1={0}
              x2={marginEndPos}
              y2={RULER_SIZE}
              className="ruler-margin-handle"
            />
          </>
        ) : (
          <>
            <rect
              x={0}
              y={0}
              width={RULER_SIZE}
              height={marginStartPos}
              className="ruler-margin"
            />
            <rect
              x={0}
              y={marginEndPos}
              width={RULER_SIZE}
              height={length * zoom - marginEndPos}
              className="ruler-margin"
            />
            {/* Margin handles */}
            <line
              x1={0}
              y1={marginStartPos}
              x2={RULER_SIZE}
              y2={marginStartPos}
              className="ruler-margin-handle"
            />
            <line
              x1={0}
              y1={marginEndPos}
              x2={RULER_SIZE}
              y2={marginEndPos}
              className="ruler-margin-handle"
            />
          </>
        )}
      </svg>
    </div>
  );
}

// =============================================================================
// Combined Rulers Component
// =============================================================================

export function Rulers({
  pageWidth,
  pageHeight,
  zoom,
  scrollX,
  scrollY,
  pageOffsetX,
  pageOffsetY,
  marginLeft,
  marginRight,
  marginTop,
  marginBottom,
  unit = 'in',
  visible = true,
}: RulersProps) {
  if (!visible) {
    return null;
  }

  return (
    <>
      {/* Corner piece */}
      <div
        className="ruler-corner"
        style={{
          width: RULER_SIZE,
          height: RULER_SIZE,
        }}
      />

      {/* Horizontal ruler */}
      <Ruler
        orientation="horizontal"
        length={pageWidth}
        zoom={zoom}
        scrollOffset={scrollX}
        pageOffset={pageOffsetX}
        marginStart={marginLeft}
        marginEnd={marginRight}
        unit={unit}
        visible={visible}
      />

      {/* Vertical ruler */}
      <Ruler
        orientation="vertical"
        length={pageHeight}
        zoom={zoom}
        scrollOffset={scrollY}
        pageOffset={pageOffsetY}
        marginStart={marginTop}
        marginEnd={marginBottom}
        unit={unit}
        visible={visible}
      />
    </>
  );
}

// =============================================================================
// Default Page Margins (US Letter at 96 DPI)
// =============================================================================

export const DEFAULT_MARGINS = {
  left: 96, // 1 inch
  right: 96, // 1 inch
  top: 96, // 1 inch
  bottom: 96, // 1 inch
};

// Default page dimensions (US Letter at 96 DPI)
export const DEFAULT_PAGE_DIMENSIONS = {
  width: 816, // 8.5 inches
  height: 1056, // 11 inches
};
