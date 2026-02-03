import { useState, useEffect, useCallback } from 'react';
import {
  Color,
  ShapeFillType,
  ShapeStrokeType,
  ShapeWrapType,
  DashStyleRender,
  colorToCss,
} from '../lib/types';

// =============================================================================
// Types
// =============================================================================

interface ShapeFormatPanelProps {
  isOpen: boolean;
  onClose: () => void;
  onUpdate: (updates: ShapeFormatUpdates) => void;
  shapeId?: string;
  initialValues?: ShapeFormatValues;
}

interface ShapeFormatValues {
  width: number;
  height: number;
  rotation: number;
  fill: ShapeFillType | null;
  stroke: ShapeStrokeType | null;
  wrapType: ShapeWrapType;
  opacity: number;
  shadow: boolean;
}

interface ShapeFormatUpdates {
  width?: number;
  height?: number;
  rotation?: number;
  fill?: ShapeFillType | null;
  stroke?: ShapeStrokeType | null;
  wrapType?: ShapeWrapType;
  opacity?: number;
  shadow?: boolean;
}

// =============================================================================
// Default Values
// =============================================================================

const DEFAULT_VALUES: ShapeFormatValues = {
  width: 100,
  height: 100,
  rotation: 0,
  fill: { type: 'Solid', color: { r: 68, g: 114, b: 196, a: 255 } },
  stroke: { color: { r: 0, g: 0, b: 0, a: 255 }, width: 1, dash_style: 'Solid' },
  wrapType: 'InFront',
  opacity: 100,
  shadow: false,
};

// Preset colors for the color picker
const PRESET_COLORS: Color[] = [
  { r: 68, g: 114, b: 196, a: 255 }, // Blue
  { r: 192, g: 0, b: 0, a: 255 }, // Red
  { r: 84, g: 130, b: 53, a: 255 }, // Green
  { r: 255, g: 192, b: 0, a: 255 }, // Yellow
  { r: 237, g: 125, b: 49, a: 255 }, // Orange
  { r: 112, g: 48, b: 160, a: 255 }, // Purple
  { r: 0, g: 0, b: 0, a: 255 }, // Black
  { r: 128, g: 128, b: 128, a: 255 }, // Gray
  { r: 255, g: 255, b: 255, a: 255 }, // White
];

const WRAP_TYPE_OPTIONS: { value: ShapeWrapType; label: string }[] = [
  { value: 'Inline', label: 'Inline with text' },
  { value: 'Square', label: 'Square' },
  { value: 'Tight', label: 'Tight' },
  { value: 'Behind', label: 'Behind text' },
  { value: 'InFront', label: 'In front of text' },
];

const DASH_STYLE_OPTIONS: { value: DashStyleRender; label: string }[] = [
  { value: 'Solid', label: 'Solid' },
  { value: 'Dash', label: 'Dashed' },
  { value: 'Dot', label: 'Dotted' },
  { value: 'DashDot', label: 'Dash-Dot' },
  { value: 'DashDotDot', label: 'Dash-Dot-Dot' },
];

// =============================================================================
// Color Picker Component
// =============================================================================

interface ColorPickerProps {
  label: string;
  color: Color | null;
  onChange: (color: Color | null) => void;
  allowNone?: boolean;
}

function ColorPicker({ label, color, onChange, allowNone = false }: ColorPickerProps) {
  const [showPicker, setShowPicker] = useState(false);

  return (
    <div className="color-picker-container">
      <label className="color-label">{label}:</label>
      <div className="color-picker-row">
        <button
          className="color-swatch"
          onClick={() => setShowPicker(!showPicker)}
          style={{
            backgroundColor: color ? colorToCss(color) : 'transparent',
            border: color ? '1px solid #ccc' : '1px dashed #999',
          }}
          title={color ? 'Click to change color' : 'No color'}
        >
          {!color && <span className="no-color-line" />}
        </button>
        {allowNone && (
          <button
            className="no-color-btn"
            onClick={() => onChange(null)}
            title="No color"
          >
            None
          </button>
        )}
      </div>
      {showPicker && (
        <div className="color-picker-dropdown">
          <div className="color-grid">
            {PRESET_COLORS.map((c, idx) => (
              <button
                key={idx}
                className={`color-option ${color && c.r === color.r && c.g === color.g && c.b === color.b ? 'selected' : ''}`}
                style={{ backgroundColor: colorToCss(c) }}
                onClick={() => {
                  onChange(c);
                  setShowPicker(false);
                }}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

// =============================================================================
// ShapeFormatPanel Component
// =============================================================================

export function ShapeFormatPanel({
  isOpen,
  onClose,
  onUpdate,
  shapeId,
  initialValues,
}: ShapeFormatPanelProps) {
  const [values, setValues] = useState<ShapeFormatValues>(initialValues || DEFAULT_VALUES);
  const [lockAspectRatio, setLockAspectRatio] = useState(false);

  // Reset form when dialog opens with new values
  useEffect(() => {
    if (isOpen && initialValues) {
      setValues(initialValues);
    } else if (isOpen) {
      setValues(DEFAULT_VALUES);
    }
  }, [isOpen, initialValues]);

  // Handle width change with aspect ratio lock
  const handleWidthChange = useCallback(
    (newWidth: number) => {
      if (lockAspectRatio && values.height > 0) {
        const ratio = values.width / values.height;
        setValues((prev) => ({
          ...prev,
          width: newWidth,
          height: Math.round(newWidth / ratio),
        }));
      } else {
        setValues((prev) => ({ ...prev, width: newWidth }));
      }
    },
    [lockAspectRatio, values.width, values.height]
  );

  // Handle height change with aspect ratio lock
  const handleHeightChange = useCallback(
    (newHeight: number) => {
      if (lockAspectRatio && values.width > 0) {
        const ratio = values.width / values.height;
        setValues((prev) => ({
          ...prev,
          height: newHeight,
          width: Math.round(newHeight * ratio),
        }));
      } else {
        setValues((prev) => ({ ...prev, height: newHeight }));
      }
    },
    [lockAspectRatio, values.width, values.height]
  );

  const handleFillColorChange = useCallback((color: Color | null) => {
    if (color) {
      setValues((prev) => ({ ...prev, fill: { type: 'Solid', color } }));
    } else {
      setValues((prev) => ({ ...prev, fill: { type: 'None' } }));
    }
  }, []);

  const handleStrokeColorChange = useCallback((color: Color | null) => {
    setValues((prev) => {
      if (color) {
        return {
          ...prev,
          stroke: {
            color,
            width: prev.stroke?.width || 1,
            dash_style: prev.stroke?.dash_style || 'Solid',
          },
        };
      } else {
        return { ...prev, stroke: null };
      }
    });
  }, []);

  const handleStrokeWidthChange = useCallback((width: number) => {
    setValues((prev) => {
      if (prev.stroke) {
        return { ...prev, stroke: { ...prev.stroke, width } };
      }
      return prev;
    });
  }, []);

  const handleDashStyleChange = useCallback((dashStyle: DashStyleRender) => {
    setValues((prev) => {
      if (prev.stroke) {
        return { ...prev, stroke: { ...prev.stroke, dash_style: dashStyle } };
      }
      return prev;
    });
  }, []);

  const handleApply = () => {
    onUpdate({
      width: values.width,
      height: values.height,
      rotation: values.rotation,
      fill: values.fill,
      stroke: values.stroke,
      wrapType: values.wrapType,
      opacity: values.opacity,
      shadow: values.shadow,
    });
    onClose();
  };

  if (!isOpen) return null;

  const fillColor = values.fill?.type === 'Solid' ? values.fill.color : null;

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog shape-format-panel" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>Format Shape</h2>
          <button className="close-button" onClick={onClose} aria-label="Close">
            x
          </button>
        </div>

        <div className="dialog-content">
          {/* Size Section */}
          <div className="form-section">
            <h3>Size</h3>
            <div className="size-controls">
              <div className="form-row">
                <label htmlFor="shape-width">Width:</label>
                <input
                  type="number"
                  id="shape-width"
                  value={values.width}
                  onChange={(e) => handleWidthChange(parseInt(e.target.value) || 0)}
                  min={1}
                  max={2000}
                />
                <span className="unit">pt</span>
              </div>
              <div className="form-row">
                <label htmlFor="shape-height">Height:</label>
                <input
                  type="number"
                  id="shape-height"
                  value={values.height}
                  onChange={(e) => handleHeightChange(parseInt(e.target.value) || 0)}
                  min={1}
                  max={2000}
                />
                <span className="unit">pt</span>
              </div>
              <div className="form-row checkbox-row">
                <label>
                  <input
                    type="checkbox"
                    checked={lockAspectRatio}
                    onChange={(e) => setLockAspectRatio(e.target.checked)}
                  />
                  Lock aspect ratio
                </label>
              </div>
            </div>
          </div>

          {/* Fill Section */}
          <div className="form-section">
            <h3>Fill</h3>
            <ColorPicker
              label="Fill Color"
              color={fillColor}
              onChange={handleFillColorChange}
              allowNone
            />
          </div>

          {/* Stroke Section */}
          <div className="form-section">
            <h3>Outline</h3>
            <ColorPicker
              label="Color"
              color={values.stroke?.color || null}
              onChange={handleStrokeColorChange}
              allowNone
            />
            {values.stroke && (
              <>
                <div className="form-row">
                  <label htmlFor="stroke-width">Width:</label>
                  <input
                    type="number"
                    id="stroke-width"
                    value={values.stroke.width}
                    onChange={(e) => handleStrokeWidthChange(parseFloat(e.target.value) || 1)}
                    min={0.5}
                    max={20}
                    step={0.5}
                  />
                  <span className="unit">pt</span>
                </div>
                <div className="form-row">
                  <label htmlFor="dash-style">Style:</label>
                  <select
                    id="dash-style"
                    value={values.stroke.dash_style}
                    onChange={(e) => handleDashStyleChange(e.target.value as DashStyleRender)}
                  >
                    {DASH_STYLE_OPTIONS.map((opt) => (
                      <option key={opt.value} value={opt.value}>
                        {opt.label}
                      </option>
                    ))}
                  </select>
                </div>
              </>
            )}
          </div>

          {/* Rotation Section */}
          <div className="form-section">
            <h3>Rotation</h3>
            <div className="form-row">
              <label htmlFor="shape-rotation">Angle:</label>
              <input
                type="number"
                id="shape-rotation"
                value={values.rotation}
                onChange={(e) =>
                  setValues((prev) => ({ ...prev, rotation: parseFloat(e.target.value) || 0 }))
                }
                min={-360}
                max={360}
                step={1}
              />
              <span className="unit">degrees</span>
            </div>
            <div className="rotation-presets">
              <button type="button" onClick={() => setValues((prev) => ({ ...prev, rotation: 0 }))}>
                0
              </button>
              <button type="button" onClick={() => setValues((prev) => ({ ...prev, rotation: 90 }))}>
                90
              </button>
              <button type="button" onClick={() => setValues((prev) => ({ ...prev, rotation: 180 }))}>
                180
              </button>
              <button type="button" onClick={() => setValues((prev) => ({ ...prev, rotation: 270 }))}>
                270
              </button>
            </div>
          </div>

          {/* Text Wrapping Section */}
          <div className="form-section">
            <h3>Text Wrapping</h3>
            <div className="wrap-options">
              {WRAP_TYPE_OPTIONS.map((option) => (
                <label key={option.value} className="wrap-option">
                  <input
                    type="radio"
                    name="wrapType"
                    value={option.value}
                    checked={values.wrapType === option.value}
                    onChange={() => setValues((prev) => ({ ...prev, wrapType: option.value }))}
                  />
                  <span>{option.label}</span>
                </label>
              ))}
            </div>
          </div>

          {/* Effects Section */}
          <div className="form-section">
            <h3>Effects</h3>
            <div className="form-row">
              <label htmlFor="shape-opacity">Opacity:</label>
              <input
                type="range"
                id="shape-opacity"
                value={values.opacity}
                onChange={(e) =>
                  setValues((prev) => ({ ...prev, opacity: parseInt(e.target.value) }))
                }
                min={0}
                max={100}
              />
              <span className="unit">{values.opacity}%</span>
            </div>
            <div className="form-row checkbox-row">
              <label>
                <input
                  type="checkbox"
                  checked={values.shadow}
                  onChange={(e) => setValues((prev) => ({ ...prev, shadow: e.target.checked }))}
                />
                Shadow
              </label>
            </div>
          </div>
        </div>

        <div className="dialog-footer">
          <button type="button" className="cancel-button" onClick={onClose}>
            Cancel
          </button>
          <button type="button" className="save-button" onClick={handleApply}>
            Apply
          </button>
        </div>

        <style>{`
          .shape-format-panel {
            width: 400px;
            max-height: 80vh;
            overflow-y: auto;
          }

          .form-section {
            margin-bottom: 16px;
            padding-bottom: 16px;
            border-bottom: 1px solid #e0e0e0;
          }

          .form-section:last-child {
            border-bottom: none;
            margin-bottom: 0;
          }

          .form-section h3 {
            margin: 0 0 12px 0;
            font-size: 13px;
            font-weight: 600;
            color: #333;
          }

          .size-controls {
            display: flex;
            flex-direction: column;
            gap: 8px;
          }

          .form-row {
            display: flex;
            align-items: center;
            gap: 8px;
          }

          .form-row label {
            min-width: 70px;
            font-size: 12px;
          }

          .form-row input[type="number"],
          .form-row select {
            flex: 1;
            max-width: 80px;
            padding: 4px 8px;
            border: 1px solid #ccc;
            border-radius: 4px;
            font-size: 12px;
          }

          .form-row input[type="range"] {
            flex: 1;
          }

          .unit {
            font-size: 11px;
            color: #666;
            min-width: 50px;
          }

          .checkbox-row label {
            min-width: auto;
            display: flex;
            align-items: center;
            gap: 6px;
            cursor: pointer;
          }

          .rotation-presets {
            display: flex;
            gap: 8px;
            margin-top: 8px;
          }

          .rotation-presets button {
            padding: 4px 12px;
            font-size: 11px;
            background: #f0f0f0;
            border: 1px solid #ccc;
            border-radius: 4px;
            cursor: pointer;
          }

          .rotation-presets button:hover {
            background: #e0e0e0;
          }

          .wrap-options {
            display: flex;
            flex-direction: column;
            gap: 6px;
          }

          .wrap-option {
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 12px;
            cursor: pointer;
          }

          .color-picker-container {
            margin-bottom: 8px;
          }

          .color-label {
            display: block;
            font-size: 12px;
            margin-bottom: 4px;
          }

          .color-picker-row {
            display: flex;
            align-items: center;
            gap: 8px;
          }

          .color-swatch {
            width: 32px;
            height: 32px;
            border-radius: 4px;
            cursor: pointer;
            position: relative;
          }

          .no-color-line {
            position: absolute;
            width: 140%;
            height: 2px;
            background: red;
            top: 50%;
            left: -20%;
            transform: rotate(-45deg);
          }

          .no-color-btn {
            padding: 4px 8px;
            font-size: 11px;
            background: #f0f0f0;
            border: 1px solid #ccc;
            border-radius: 4px;
            cursor: pointer;
          }

          .color-picker-dropdown {
            position: absolute;
            background: white;
            border: 1px solid #ccc;
            border-radius: 4px;
            padding: 8px;
            box-shadow: 0 4px 8px rgba(0,0,0,0.15);
            z-index: 100;
          }

          .color-grid {
            display: grid;
            grid-template-columns: repeat(5, 1fr);
            gap: 4px;
          }

          .color-option {
            width: 24px;
            height: 24px;
            border: 1px solid #ccc;
            border-radius: 4px;
            cursor: pointer;
          }

          .color-option.selected {
            border: 2px solid #0066cc;
          }

          .dialog-footer {
            display: flex;
            justify-content: flex-end;
            gap: 8px;
            padding-top: 16px;
            border-top: 1px solid #e0e0e0;
          }

          .cancel-button,
          .save-button {
            padding: 8px 16px;
            font-size: 12px;
            border-radius: 4px;
            cursor: pointer;
          }

          .cancel-button {
            background: #fff;
            border: 1px solid #ccc;
          }

          .cancel-button:hover {
            background: #f0f0f0;
          }

          .save-button {
            background: #0066cc;
            border: 1px solid #0066cc;
            color: white;
          }

          .save-button:hover {
            background: #0052a3;
          }
        `}</style>
      </div>
    </div>
  );
}
