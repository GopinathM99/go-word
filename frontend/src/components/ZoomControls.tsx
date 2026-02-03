/**
 * ZoomControls - UI components for zoom control in the document editor
 *
 * Features:
 * - Zoom slider (25%-500%)
 * - Zoom dropdown with presets
 * - Zoom +/- buttons
 * - Fit page width / Fit whole page options
 * - Custom zoom input
 */

import { useState, useCallback, useRef, useEffect } from 'react';
import { ZoomFitMode, ZOOM_PRESETS } from '../hooks/useZoom';

// =============================================================================
// Types
// =============================================================================

export interface ZoomControlsProps {
  /** Current zoom level (0.25 to 5.0) */
  zoom: number;
  /** Current fit mode */
  fitMode: ZoomFitMode;
  /** Zoom percentage display string */
  zoomPercentage: string;
  /** Whether at minimum zoom */
  isAtMin: boolean;
  /** Whether at maximum zoom */
  isAtMax: boolean;
  /** Set zoom to specific level */
  onZoomChange: (zoom: number) => void;
  /** Zoom in by step */
  onZoomIn: () => void;
  /** Zoom out by step */
  onZoomOut: () => void;
  /** Reset to 100% */
  onResetZoom: () => void;
  /** Fit to page width */
  onFitToWidth: () => void;
  /** Fit whole page */
  onFitToPage: () => void;
}

// =============================================================================
// ZoomSlider Component
// =============================================================================

interface ZoomSliderProps {
  zoom: number;
  onZoomChange: (zoom: number) => void;
  minZoom?: number;
  maxZoom?: number;
}

export function ZoomSlider({
  zoom,
  onZoomChange,
  minZoom = 0.25,
  maxZoom = 5.0,
}: ZoomSliderProps) {
  // Use a logarithmic scale for better UX
  // This makes the slider feel more linear to the user
  const toSliderValue = (z: number) => {
    return Math.log(z / minZoom) / Math.log(maxZoom / minZoom) * 100;
  };

  const fromSliderValue = (v: number) => {
    return minZoom * Math.pow(maxZoom / minZoom, v / 100);
  };

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const sliderValue = parseFloat(e.target.value);
      const newZoom = fromSliderValue(sliderValue);
      onZoomChange(Math.round(newZoom * 100) / 100);
    },
    [onZoomChange]
  );

  return (
    <input
      type="range"
      className="zoom-slider"
      min={0}
      max={100}
      value={toSliderValue(zoom)}
      onChange={handleChange}
      aria-label="Zoom level"
      title={`Zoom: ${Math.round(zoom * 100)}%`}
    />
  );
}

// =============================================================================
// ZoomDropdown Component
// =============================================================================

interface ZoomDropdownProps {
  zoom: number;
  zoomPercentage: string;
  fitMode: ZoomFitMode;
  onZoomChange: (zoom: number) => void;
  onFitToWidth: () => void;
  onFitToPage: () => void;
}

export function ZoomDropdown({
  zoom,
  zoomPercentage,
  fitMode,
  onZoomChange,
  onFitToWidth,
  onFitToPage,
}: ZoomDropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [customValue, setCustomValue] = useState('');
  const [isEditingCustom, setIsEditingCustom] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setIsOpen(false);
        setIsEditingCustom(false);
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [isOpen]);

  // Focus input when editing custom
  useEffect(() => {
    if (isEditingCustom && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditingCustom]);

  const handlePresetClick = useCallback(
    (preset: number) => {
      onZoomChange(preset);
      setIsOpen(false);
    },
    [onZoomChange]
  );

  const handleCustomSubmit = useCallback(() => {
    const parsed = parseInt(customValue, 10);
    if (!isNaN(parsed) && parsed >= 25 && parsed <= 500) {
      onZoomChange(parsed / 100);
    }
    setIsEditingCustom(false);
    setIsOpen(false);
    setCustomValue('');
  }, [customValue, onZoomChange]);

  const handleCustomKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter') {
        handleCustomSubmit();
      } else if (e.key === 'Escape') {
        setIsEditingCustom(false);
        setCustomValue('');
      }
    },
    [handleCustomSubmit]
  );

  const toggleDropdown = useCallback(() => {
    setIsOpen(!isOpen);
    setIsEditingCustom(false);
  }, [isOpen]);

  // Get display text
  const displayText =
    fitMode === 'fit-width'
      ? 'Fit Width'
      : fitMode === 'fit-page'
      ? 'Fit Page'
      : zoomPercentage;

  return (
    <div className="zoom-dropdown" ref={dropdownRef}>
      <button
        className="zoom-dropdown-trigger"
        onClick={toggleDropdown}
        aria-expanded={isOpen}
        aria-haspopup="listbox"
      >
        <span className="zoom-value">{displayText}</span>
        <span className="zoom-dropdown-arrow">{isOpen ? '\u25B2' : '\u25BC'}</span>
      </button>

      {isOpen && (
        <div className="zoom-dropdown-menu" role="listbox">
          {/* Fit options */}
          <button
            className={`zoom-dropdown-item ${fitMode === 'fit-width' ? 'active' : ''}`}
            onClick={() => {
              onFitToWidth();
              setIsOpen(false);
            }}
          >
            Fit Page Width
          </button>
          <button
            className={`zoom-dropdown-item ${fitMode === 'fit-page' ? 'active' : ''}`}
            onClick={() => {
              onFitToPage();
              setIsOpen(false);
            }}
          >
            Fit Whole Page
          </button>

          <div className="zoom-dropdown-divider" />

          {/* Preset levels */}
          {ZOOM_PRESETS.map((preset) => {
            const percentage = Math.round(preset * 100);
            const isActive = fitMode === 'none' && Math.abs(zoom - preset) < 0.01;
            return (
              <button
                key={preset}
                className={`zoom-dropdown-item ${isActive ? 'active' : ''}`}
                onClick={() => handlePresetClick(preset)}
              >
                {percentage}%
              </button>
            );
          })}

          <div className="zoom-dropdown-divider" />

          {/* Custom input */}
          {isEditingCustom ? (
            <div className="zoom-dropdown-custom">
              <input
                ref={inputRef}
                type="number"
                min={25}
                max={500}
                value={customValue}
                onChange={(e) => setCustomValue(e.target.value)}
                onKeyDown={handleCustomKeyDown}
                onBlur={handleCustomSubmit}
                placeholder="25-500"
                className="zoom-custom-input"
              />
              <span>%</span>
            </div>
          ) : (
            <button
              className="zoom-dropdown-item"
              onClick={() => {
                setIsEditingCustom(true);
                setCustomValue(Math.round(zoom * 100).toString());
              }}
            >
              Custom...
            </button>
          )}
        </div>
      )}
    </div>
  );
}

// =============================================================================
// ZoomButtons Component
// =============================================================================

interface ZoomButtonsProps {
  onZoomIn: () => void;
  onZoomOut: () => void;
  isAtMin: boolean;
  isAtMax: boolean;
}

export function ZoomButtons({ onZoomIn, onZoomOut, isAtMin, isAtMax }: ZoomButtonsProps) {
  return (
    <div className="zoom-buttons">
      <button
        className="zoom-button zoom-out"
        onClick={onZoomOut}
        disabled={isAtMin}
        aria-label="Zoom out"
        title="Zoom out (Ctrl+-)"
      >
        -
      </button>
      <button
        className="zoom-button zoom-in"
        onClick={onZoomIn}
        disabled={isAtMax}
        aria-label="Zoom in"
        title="Zoom in (Ctrl++)"
      >
        +
      </button>
    </div>
  );
}

// =============================================================================
// Combined ZoomControls Component
// =============================================================================

export function ZoomControls({
  zoom,
  fitMode,
  zoomPercentage,
  isAtMin,
  isAtMax,
  onZoomChange,
  onZoomIn,
  onZoomOut,
  onFitToWidth,
  onFitToPage,
}: ZoomControlsProps) {
  return (
    <div className="zoom-controls">
      <ZoomButtons
        onZoomIn={onZoomIn}
        onZoomOut={onZoomOut}
        isAtMin={isAtMin}
        isAtMax={isAtMax}
      />
      <ZoomSlider zoom={zoom} onZoomChange={onZoomChange} />
      <ZoomDropdown
        zoom={zoom}
        zoomPercentage={zoomPercentage}
        fitMode={fitMode}
        onZoomChange={onZoomChange}
        onFitToWidth={onFitToWidth}
        onFitToPage={onFitToPage}
      />
    </div>
  );
}
