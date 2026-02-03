/**
 * ChartView Component
 *
 * Displays charts using SVG rendering. Supports bar, line, pie, area,
 * scatter, and other chart types. Handles chart selection and interaction.
 */

import React, { useState, useCallback, useMemo, useRef } from 'react';

// =============================================================================
// Types
// =============================================================================

export interface ChartData {
  id: string;
  chartType: ChartType;
  title?: string;
  categories: string[];
  series: DataSeries[];
  legend?: LegendConfig;
  axes?: AxesConfig;
  style?: ChartStyle;
}

export type ChartType =
  | { kind: 'Bar'; horizontal: boolean; stacked: boolean; stackedPercent: boolean }
  | { kind: 'Column'; stacked: boolean; stackedPercent: boolean }
  | { kind: 'Line'; smooth: boolean; markers: boolean }
  | { kind: 'Pie'; doughnut: boolean; explosion: number }
  | { kind: 'Area'; stacked: boolean }
  | { kind: 'Scatter'; withLines: boolean }
  | { kind: 'Bubble' }
  | { kind: 'Radar'; filled: boolean }
  | { kind: 'Stock' };

export interface DataSeries {
  name: string;
  values: number[];
  color?: string;
}

export interface LegendConfig {
  position: 'top' | 'bottom' | 'left' | 'right' | 'none';
  visible: boolean;
}

export interface AxesConfig {
  categoryAxis?: AxisConfig;
  valueAxis?: AxisConfig;
}

export interface AxisConfig {
  title?: string;
  min?: number;
  max?: number;
  majorGridlines: boolean;
  minorGridlines: boolean;
}

export interface ChartStyle {
  colors: string[];
  background?: string;
  plotAreaBackground?: string;
}

export interface ChartViewProps {
  chart: ChartData;
  width: number;
  height: number;
  selected?: boolean;
  onSelect?: (chartId: string) => void;
  onDeselect?: () => void;
  className?: string;
}

// =============================================================================
// Default Color Palette
// =============================================================================

const DEFAULT_COLORS = [
  '#4F81BD', // Blue
  '#C0504D', // Red
  '#9BBB59', // Green
  '#8064A2', // Purple
  '#4BACC6', // Teal
  '#F79646', // Orange
  '#77923C', // Olive
  '#A6A6A6', // Gray
];

// =============================================================================
// Utility Functions
// =============================================================================

function getSeriesColor(index: number, style?: ChartStyle): string {
  const colors = style?.colors || DEFAULT_COLORS;
  return colors[index % colors.length];
}

function calculateDataRange(
  series: DataSeries[],
  stacked: boolean
): { min: number; max: number } {
  if (series.length === 0) {
    return { min: 0, max: 100 };
  }

  if (stacked) {
    const pointCount = Math.max(...series.map((s) => s.values.length));
    const totals: number[] = new Array(pointCount).fill(0);
    for (const s of series) {
      s.values.forEach((v, i) => {
        totals[i] = (totals[i] || 0) + v;
      });
    }
    return { min: 0, max: Math.max(...totals, 1) };
  }

  let min = Infinity;
  let max = -Infinity;
  for (const s of series) {
    for (const v of s.values) {
      if (v < min) min = v;
      if (v > max) max = v;
    }
  }
  if (min > 0) min = 0;
  if (max < min) max = min + 1;
  return { min, max };
}

function formatValue(value: number): string {
  if (Math.abs(value) >= 1000000) {
    return (value / 1000000).toFixed(1) + 'M';
  }
  if (Math.abs(value) >= 1000) {
    return (value / 1000).toFixed(1) + 'K';
  }
  return value.toFixed(0);
}

// =============================================================================
// Chart Components
// =============================================================================

interface BarChartProps {
  chart: ChartData;
  plotArea: { x: number; y: number; width: number; height: number };
  horizontal: boolean;
  stacked: boolean;
}

function BarChart({ chart, plotArea, horizontal, stacked }: BarChartProps) {
  const { series, categories, style } = chart;
  const categoryCount = Math.max(categories.length, series[0]?.values.length || 1);
  const seriesCount = series.length;

  const { min, max } = calculateDataRange(series, stacked);
  const valueRange = max - min;

  const groupGap = 10;
  const barGap = 2;

  const categorySize = horizontal
    ? plotArea.height / categoryCount
    : plotArea.width / categoryCount;

  const barArea = categorySize - groupGap;
  const barWidth = stacked
    ? barArea
    : (barArea - barGap * (seriesCount - 1)) / seriesCount;

  const bars: React.ReactNode[] = [];
  const stackedOffsets: number[] = new Array(categoryCount).fill(0);

  series.forEach((s, seriesIdx) => {
    const color = s.color || getSeriesColor(seriesIdx, style);

    s.values.forEach((value, catIdx) => {
      const normalized = (value - min) / valueRange;
      const barLength = normalized * (horizontal ? plotArea.width : plotArea.height);

      let x: number, y: number, w: number, h: number;

      if (stacked) {
        if (horizontal) {
          x = plotArea.x + stackedOffsets[catIdx];
          y = plotArea.y + catIdx * categorySize + groupGap / 2;
          w = barLength;
          h = barWidth;
          stackedOffsets[catIdx] += barLength;
        } else {
          x = plotArea.x + catIdx * categorySize + groupGap / 2;
          y = plotArea.y + plotArea.height - stackedOffsets[catIdx] - barLength;
          w = barWidth;
          h = barLength;
          stackedOffsets[catIdx] += barLength;
        }
      } else {
        if (horizontal) {
          x = plotArea.x;
          y =
            plotArea.y +
            catIdx * categorySize +
            groupGap / 2 +
            seriesIdx * (barWidth + barGap);
          w = barLength;
          h = barWidth;
        } else {
          x =
            plotArea.x +
            catIdx * categorySize +
            groupGap / 2 +
            seriesIdx * (barWidth + barGap);
          y = plotArea.y + plotArea.height - barLength;
          w = barWidth;
          h = barLength;
        }
      }

      bars.push(
        <rect
          key={`bar-${seriesIdx}-${catIdx}`}
          x={x}
          y={y}
          width={Math.max(w, 0)}
          height={Math.max(h, 0)}
          fill={color}
          stroke={darkenColor(color)}
          strokeWidth={1}
        >
          <title>{`${s.name}: ${value}`}</title>
        </rect>
      );
    });
  });

  return <g className="chart-bars">{bars}</g>;
}

interface LineChartProps {
  chart: ChartData;
  plotArea: { x: number; y: number; width: number; height: number };
  smooth: boolean;
  markers: boolean;
}

function LineChart({ chart, plotArea, smooth, markers }: LineChartProps) {
  const { series, style } = chart;

  if (series.length === 0) {
    return null;
  }

  const { min, max } = calculateDataRange(series, false);
  const valueRange = max - min || 1;

  const pointCount = Math.max(...series.map((s) => s.values.length));
  const xStep = pointCount > 1 ? plotArea.width / (pointCount - 1) : plotArea.width;

  const elements: React.ReactNode[] = [];

  series.forEach((s, seriesIdx) => {
    const color = s.color || getSeriesColor(seriesIdx, style);
    const points: { x: number; y: number }[] = [];

    s.values.forEach((value, catIdx) => {
      const normalized = (value - min) / valueRange;
      const x = plotArea.x + catIdx * xStep;
      const y = plotArea.y + plotArea.height - normalized * plotArea.height;
      points.push({ x, y });
    });

    // Draw line
    if (points.length > 1) {
      const pathData = smooth
        ? createSmoothPath(points)
        : points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`).join(' ');

      elements.push(
        <path
          key={`line-${seriesIdx}`}
          d={pathData}
          fill="none"
          stroke={color}
          strokeWidth={2}
          strokeLinejoin="round"
          strokeLinecap="round"
        />
      );
    }

    // Draw markers
    if (markers) {
      points.forEach((p, catIdx) => {
        elements.push(
          <circle
            key={`marker-${seriesIdx}-${catIdx}`}
            cx={p.x}
            cy={p.y}
            r={4}
            fill={color}
            stroke={darkenColor(color)}
            strokeWidth={1}
          >
            <title>{`${s.name}: ${s.values[catIdx]}`}</title>
          </circle>
        );
      });
    }
  });

  return <g className="chart-lines">{elements}</g>;
}

interface PieChartProps {
  chart: ChartData;
  plotArea: { x: number; y: number; width: number; height: number };
  doughnut: boolean;
  explosion: number;
}

function PieChart({ chart, plotArea, doughnut, explosion }: PieChartProps) {
  const { series, style } = chart;

  if (series.length === 0 || series[0].values.length === 0) {
    return null;
  }

  const values = series[0].values;
  const total = values.reduce((a, b) => a + b, 0);

  if (total === 0) {
    return null;
  }

  const cx = plotArea.x + plotArea.width / 2;
  const cy = plotArea.y + plotArea.height / 2;
  const maxRadius = Math.min(plotArea.width, plotArea.height) / 2 * 0.9;
  const outerRadius = maxRadius;
  const innerRadius = doughnut ? outerRadius * 0.5 : 0;

  let currentAngle = -Math.PI / 2;
  const slices: React.ReactNode[] = [];

  values.forEach((value, idx) => {
    const percentage = value / total;
    const sweepAngle = percentage * Math.PI * 2;
    const endAngle = currentAngle + sweepAngle;
    const color = getSeriesColor(idx, style);

    // Calculate explosion offset
    const midAngle = (currentAngle + endAngle) / 2;
    const explosionOffset = (explosion / 100) * maxRadius;
    const offsetX = explosionOffset * Math.cos(midAngle);
    const offsetY = explosionOffset * Math.sin(midAngle);

    const path = createArcPath(
      cx + offsetX,
      cy + offsetY,
      innerRadius,
      outerRadius,
      currentAngle,
      endAngle
    );

    slices.push(
      <path
        key={`slice-${idx}`}
        d={path}
        fill={color}
        stroke="white"
        strokeWidth={1}
      >
        <title>{`${chart.categories[idx] || `Slice ${idx + 1}`}: ${value} (${(percentage * 100).toFixed(1)}%)`}</title>
      </path>
    );

    currentAngle = endAngle;
  });

  return <g className="chart-pie">{slices}</g>;
}

interface AreaChartProps {
  chart: ChartData;
  plotArea: { x: number; y: number; width: number; height: number };
  stacked: boolean;
}

function AreaChart({ chart, plotArea, stacked }: AreaChartProps) {
  const { series, style } = chart;

  if (series.length === 0) {
    return null;
  }

  const { min, max } = calculateDataRange(series, stacked);
  const valueRange = max - min || 1;

  const pointCount = Math.max(...series.map((s) => s.values.length));
  const xStep = pointCount > 1 ? plotArea.width / (pointCount - 1) : plotArea.width;

  const elements: React.ReactNode[] = [];
  const baseline: number[] = new Array(pointCount).fill(plotArea.y + plotArea.height);

  series.forEach((s, seriesIdx) => {
    const color = s.color || getSeriesColor(seriesIdx, style);
    const topPoints: string[] = [];
    const bottomPoints: string[] = [];

    s.values.forEach((value, catIdx) => {
      const normalized = (value - min) / valueRange;
      const x = plotArea.x + catIdx * xStep;
      const height = normalized * plotArea.height;

      if (stacked) {
        const topY = baseline[catIdx] - height;
        topPoints.push(`${x},${topY}`);
        bottomPoints.unshift(`${x},${baseline[catIdx]}`);
        baseline[catIdx] = topY;
      } else {
        const y = plotArea.y + plotArea.height - height;
        topPoints.push(`${x},${y}`);
        bottomPoints.unshift(`${x},${plotArea.y + plotArea.height}`);
      }
    });

    const pathData = `M ${topPoints.join(' L ')} L ${bottomPoints.join(' L ')} Z`;

    elements.push(
      <path
        key={`area-${seriesIdx}`}
        d={pathData}
        fill={colorWithAlpha(color, 0.5)}
        stroke={color}
        strokeWidth={1}
      />
    );
  });

  return <g className="chart-areas">{elements}</g>;
}

// =============================================================================
// Helper Functions for SVG Paths
// =============================================================================

function createSmoothPath(points: { x: number; y: number }[]): string {
  if (points.length < 2) {
    return '';
  }

  let path = `M ${points[0].x} ${points[0].y}`;

  for (let i = 0; i < points.length - 1; i++) {
    const p0 = points[Math.max(0, i - 1)];
    const p1 = points[i];
    const p2 = points[i + 1];
    const p3 = points[Math.min(points.length - 1, i + 2)];

    const cp1x = p1.x + (p2.x - p0.x) / 6;
    const cp1y = p1.y + (p2.y - p0.y) / 6;
    const cp2x = p2.x - (p3.x - p1.x) / 6;
    const cp2y = p2.y - (p3.y - p1.y) / 6;

    path += ` C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${p2.x} ${p2.y}`;
  }

  return path;
}

function createArcPath(
  cx: number,
  cy: number,
  innerRadius: number,
  outerRadius: number,
  startAngle: number,
  endAngle: number
): string {
  const largeArc = Math.abs(endAngle - startAngle) > Math.PI ? 1 : 0;

  const outerStartX = cx + outerRadius * Math.cos(startAngle);
  const outerStartY = cy + outerRadius * Math.sin(startAngle);
  const outerEndX = cx + outerRadius * Math.cos(endAngle);
  const outerEndY = cy + outerRadius * Math.sin(endAngle);

  if (innerRadius > 0) {
    const innerStartX = cx + innerRadius * Math.cos(startAngle);
    const innerStartY = cy + innerRadius * Math.sin(startAngle);
    const innerEndX = cx + innerRadius * Math.cos(endAngle);
    const innerEndY = cy + innerRadius * Math.sin(endAngle);

    return [
      `M ${outerStartX} ${outerStartY}`,
      `A ${outerRadius} ${outerRadius} 0 ${largeArc} 1 ${outerEndX} ${outerEndY}`,
      `L ${innerEndX} ${innerEndY}`,
      `A ${innerRadius} ${innerRadius} 0 ${largeArc} 0 ${innerStartX} ${innerStartY}`,
      'Z',
    ].join(' ');
  }

  return [
    `M ${cx} ${cy}`,
    `L ${outerStartX} ${outerStartY}`,
    `A ${outerRadius} ${outerRadius} 0 ${largeArc} 1 ${outerEndX} ${outerEndY}`,
    'Z',
  ].join(' ');
}

function darkenColor(color: string): string {
  // Simple darkening for CSS colors
  if (color.startsWith('#')) {
    const hex = color.slice(1);
    const r = Math.floor(parseInt(hex.slice(0, 2), 16) * 0.7);
    const g = Math.floor(parseInt(hex.slice(2, 4), 16) * 0.7);
    const b = Math.floor(parseInt(hex.slice(4, 6), 16) * 0.7);
    return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;
  }
  return color;
}

function colorWithAlpha(color: string, alpha: number): string {
  if (color.startsWith('#')) {
    const hex = color.slice(1);
    const r = parseInt(hex.slice(0, 2), 16);
    const g = parseInt(hex.slice(2, 4), 16);
    const b = parseInt(hex.slice(4, 6), 16);
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
  }
  return color;
}

// =============================================================================
// Axes and Gridlines
// =============================================================================

interface AxesProps {
  plotArea: { x: number; y: number; width: number; height: number };
  categories: string[];
  min: number;
  max: number;
  config?: AxesConfig;
  horizontal?: boolean;
}

function Axes({ plotArea, categories, min, max, config, horizontal }: AxesProps) {
  const elements: React.ReactNode[] = [];
  const axisColor = '#646464';
  const gridColor = '#e6e6e6';
  const textColor = '#323232';
  const fontSize = 11;

  // Calculate value ticks
  const valueRange = max - min || 1;
  const tickCount = 5;
  const rawStep = valueRange / tickCount;
  const magnitude = Math.pow(10, Math.floor(Math.log10(rawStep)));
  const step = Math.ceil(rawStep / magnitude) * magnitude;

  const valueTicks: number[] = [];
  let tick = Math.floor(min / step) * step;
  while (tick <= max) {
    if (tick >= min) {
      valueTicks.push(tick);
    }
    tick += step;
  }

  // Gridlines
  if (config?.valueAxis?.majorGridlines !== false) {
    valueTicks.forEach((v, i) => {
      const normalized = (v - min) / valueRange;
      if (horizontal) {
        const x = plotArea.x + normalized * plotArea.width;
        elements.push(
          <line
            key={`vgrid-${i}`}
            x1={x}
            y1={plotArea.y}
            x2={x}
            y2={plotArea.y + plotArea.height}
            stroke={gridColor}
            strokeWidth={1}
          />
        );
      } else {
        const y = plotArea.y + plotArea.height - normalized * plotArea.height;
        elements.push(
          <line
            key={`hgrid-${i}`}
            x1={plotArea.x}
            y1={y}
            x2={plotArea.x + plotArea.width}
            y2={y}
            stroke={gridColor}
            strokeWidth={1}
          />
        );
      }
    });
  }

  // Category axis (bottom)
  elements.push(
    <line
      key="cat-axis"
      x1={plotArea.x}
      y1={plotArea.y + plotArea.height}
      x2={plotArea.x + plotArea.width}
      y2={plotArea.y + plotArea.height}
      stroke={axisColor}
      strokeWidth={1}
    />
  );

  // Category labels
  const catStep = plotArea.width / categories.length;
  categories.forEach((cat, i) => {
    elements.push(
      <text
        key={`cat-label-${i}`}
        x={plotArea.x + (i + 0.5) * catStep}
        y={plotArea.y + plotArea.height + 15}
        textAnchor="middle"
        fontSize={fontSize}
        fill={textColor}
      >
        {cat}
      </text>
    );
  });

  // Value axis (left)
  elements.push(
    <line
      key="val-axis"
      x1={plotArea.x}
      y1={plotArea.y}
      x2={plotArea.x}
      y2={plotArea.y + plotArea.height}
      stroke={axisColor}
      strokeWidth={1}
    />
  );

  // Value labels
  valueTicks.forEach((v, i) => {
    const normalized = (v - min) / valueRange;
    const y = plotArea.y + plotArea.height - normalized * plotArea.height;
    elements.push(
      <text
        key={`val-label-${i}`}
        x={plotArea.x - 8}
        y={y}
        textAnchor="end"
        dominantBaseline="middle"
        fontSize={fontSize}
        fill={textColor}
      >
        {formatValue(v)}
      </text>
    );
  });

  // Axis titles
  if (config?.categoryAxis?.title) {
    elements.push(
      <text
        key="cat-title"
        x={plotArea.x + plotArea.width / 2}
        y={plotArea.y + plotArea.height + 35}
        textAnchor="middle"
        fontSize={12}
        fill={textColor}
      >
        {config.categoryAxis.title}
      </text>
    );
  }

  if (config?.valueAxis?.title) {
    elements.push(
      <text
        key="val-title"
        x={plotArea.x - 40}
        y={plotArea.y + plotArea.height / 2}
        textAnchor="middle"
        fontSize={12}
        fill={textColor}
        transform={`rotate(-90, ${plotArea.x - 40}, ${plotArea.y + plotArea.height / 2})`}
      >
        {config.valueAxis.title}
      </text>
    );
  }

  return <g className="chart-axes">{elements}</g>;
}

// =============================================================================
// Legend Component
// =============================================================================

interface LegendProps {
  series: DataSeries[];
  style?: ChartStyle;
  position: 'top' | 'bottom' | 'left' | 'right' | 'none';
  bounds: { x: number; y: number; width: number; height: number };
}

function Legend({ series, style, bounds }: LegendProps) {
  const elements: React.ReactNode[] = [];
  const entryHeight = 20;
  const colorBoxSize = 12;
  const fontSize = 11;
  const textColor = '#323232';

  // Background
  elements.push(
    <rect
      key="legend-bg"
      x={bounds.x}
      y={bounds.y}
      width={bounds.width}
      height={bounds.height}
      fill="rgba(255, 255, 255, 0.9)"
      stroke="#c8c8c8"
      strokeWidth={1}
    />
  );

  series.forEach((s, i) => {
    const color = s.color || getSeriesColor(i, style);
    const y = bounds.y + i * entryHeight + 4;

    elements.push(
      <rect
        key={`legend-box-${i}`}
        x={bounds.x + 4}
        y={y + (entryHeight - colorBoxSize) / 2}
        width={colorBoxSize}
        height={colorBoxSize}
        fill={color}
      />
    );

    elements.push(
      <text
        key={`legend-text-${i}`}
        x={bounds.x + colorBoxSize + 10}
        y={y + entryHeight / 2}
        dominantBaseline="middle"
        fontSize={fontSize}
        fill={textColor}
      >
        {s.name}
      </text>
    );
  });

  return <g className="chart-legend">{elements}</g>;
}

// =============================================================================
// Main ChartView Component
// =============================================================================

export function ChartView({
  chart,
  width,
  height,
  selected = false,
  onSelect,
  onDeselect,
  className,
}: ChartViewProps) {
  const svgRef = useRef<SVGSVGElement>(null);

  // Calculate layout areas
  const padding = 10;
  const titleHeight = chart.title ? 30 : 0;
  const legendWidth = chart.legend?.visible !== false && chart.legend?.position === 'right' ? 120 : 0;
  const legendHeight = chart.legend?.visible !== false && chart.legend?.position === 'bottom' ? 30 : 0;
  const axisLabelSpace = 40;

  const plotArea = useMemo(() => {
    const isPie = chart.chartType.kind === 'Pie';
    return {
      x: padding + (isPie ? 0 : axisLabelSpace),
      y: padding + titleHeight,
      width: width - padding * 2 - legendWidth - (isPie ? 0 : axisLabelSpace),
      height: height - padding * 2 - titleHeight - legendHeight - (isPie ? 0 : axisLabelSpace),
    };
  }, [width, height, titleHeight, legendWidth, legendHeight, chart.chartType.kind]);

  const legendBounds = useMemo(() => {
    if (chart.legend?.position === 'right') {
      return {
        x: width - legendWidth - padding,
        y: padding + titleHeight,
        width: legendWidth - 10,
        height: chart.series.length * 20 + 8,
      };
    }
    if (chart.legend?.position === 'bottom') {
      return {
        x: padding,
        y: height - legendHeight - padding,
        width: width - padding * 2,
        height: legendHeight,
      };
    }
    return { x: 0, y: 0, width: 0, height: 0 };
  }, [width, height, legendWidth, legendHeight, titleHeight, chart.legend?.position, chart.series.length]);

  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      if (onSelect) {
        onSelect(chart.id);
      }
    },
    [chart.id, onSelect]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape' && onDeselect) {
        onDeselect();
      }
    },
    [onDeselect]
  );

  // Calculate data range for axes
  const isStacked =
    (chart.chartType.kind === 'Bar' && chart.chartType.stacked) ||
    (chart.chartType.kind === 'Column' && chart.chartType.stacked) ||
    (chart.chartType.kind === 'Area' && chart.chartType.stacked);
  const { min, max } = calculateDataRange(chart.series, isStacked);

  // Render the appropriate chart type
  const renderChart = () => {
    switch (chart.chartType.kind) {
      case 'Bar':
        return (
          <BarChart
            chart={chart}
            plotArea={plotArea}
            horizontal={chart.chartType.horizontal}
            stacked={chart.chartType.stacked}
          />
        );
      case 'Column':
        return (
          <BarChart
            chart={chart}
            plotArea={plotArea}
            horizontal={false}
            stacked={chart.chartType.stacked}
          />
        );
      case 'Line':
        return (
          <LineChart
            chart={chart}
            plotArea={plotArea}
            smooth={chart.chartType.smooth}
            markers={chart.chartType.markers}
          />
        );
      case 'Pie':
        return (
          <PieChart
            chart={chart}
            plotArea={plotArea}
            doughnut={chart.chartType.doughnut}
            explosion={chart.chartType.explosion}
          />
        );
      case 'Area':
        return (
          <AreaChart
            chart={chart}
            plotArea={plotArea}
            stacked={chart.chartType.stacked}
          />
        );
      case 'Scatter':
        return (
          <LineChart
            chart={chart}
            plotArea={plotArea}
            smooth={false}
            markers={true}
          />
        );
      default:
        return (
          <BarChart
            chart={chart}
            plotArea={plotArea}
            horizontal={false}
            stacked={false}
          />
        );
    }
  };

  return (
    <svg
      ref={svgRef}
      width={width}
      height={height}
      className={`chart-view ${className || ''} ${selected ? 'selected' : ''}`}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      tabIndex={0}
      role="img"
      aria-label={chart.title || 'Chart'}
      style={{
        outline: selected ? '2px solid #0066cc' : 'none',
        cursor: 'pointer',
      }}
    >
      {/* Background */}
      <rect
        x={0}
        y={0}
        width={width}
        height={height}
        fill={chart.style?.background || 'white'}
      />

      {/* Title */}
      {chart.title && (
        <text
          x={width / 2}
          y={padding + titleHeight / 2}
          textAnchor="middle"
          dominantBaseline="middle"
          fontSize={18}
          fontWeight="bold"
          fill="#323232"
        >
          {chart.title}
        </text>
      )}

      {/* Axes (not for pie charts) */}
      {chart.chartType.kind !== 'Pie' && (
        <Axes
          plotArea={plotArea}
          categories={chart.categories}
          min={min}
          max={max}
          config={chart.axes}
          horizontal={chart.chartType.kind === 'Bar' && chart.chartType.horizontal}
        />
      )}

      {/* Chart content */}
      {renderChart()}

      {/* Legend */}
      {chart.legend?.visible !== false && chart.legend?.position !== 'none' && (
        <Legend
          series={chart.series}
          style={chart.style}
          position={chart.legend?.position || 'right'}
          bounds={legendBounds}
        />
      )}

      {/* Selection handles */}
      {selected && (
        <g className="selection-handles">
          <rect
            x={0}
            y={0}
            width={width}
            height={height}
            fill="none"
            stroke="#0066cc"
            strokeWidth={2}
          />
          {/* Corner handles */}
          {[
            { x: 0, y: 0 },
            { x: width, y: 0 },
            { x: 0, y: height },
            { x: width, y: height },
          ].map((pos, i) => (
            <rect
              key={`handle-${i}`}
              x={pos.x - 4}
              y={pos.y - 4}
              width={8}
              height={8}
              fill="#0066cc"
            />
          ))}
        </g>
      )}
    </svg>
  );
}

export default ChartView;
