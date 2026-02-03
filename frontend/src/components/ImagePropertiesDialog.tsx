import { useState, useEffect, useCallback } from 'react';
import { ImageWrapType, ImageProperties } from '../lib/types';

interface ImagePropertiesDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (properties: ImageProperties) => void;
  imageId?: string;
  initialProperties?: Partial<ImageProperties>;
  originalWidth?: number;
  originalHeight?: number;
}

const WRAP_TYPE_OPTIONS: { value: ImageWrapType; label: string; description: string }[] = [
  { value: 'Inline', label: 'Inline with text', description: 'Image flows with text like a character' },
  { value: 'Square', label: 'Square', description: 'Text wraps around the image bounding box' },
  { value: 'Tight', label: 'Tight', description: 'Text wraps close to the image shape' },
  { value: 'Behind', label: 'Behind text', description: 'Image appears behind text' },
  { value: 'InFront', label: 'In front of text', description: 'Image appears in front of text' },
];

export function ImagePropertiesDialog({
  isOpen,
  onClose,
  onSave,
  imageId,
  initialProperties,
  originalWidth = 100,
  originalHeight = 100,
}: ImagePropertiesDialogProps) {
  const [width, setWidth] = useState(initialProperties?.width ?? originalWidth);
  const [height, setHeight] = useState(initialProperties?.height ?? originalHeight);
  const [wrapType, setWrapType] = useState<ImageWrapType>(initialProperties?.wrapType ?? 'Inline');
  const [rotation, setRotation] = useState(initialProperties?.rotation ?? 0);
  const [altText, setAltText] = useState(initialProperties?.altText ?? '');
  const [title, setTitle] = useState(initialProperties?.title ?? '');
  const [lockAspectRatio, setLockAspectRatio] = useState(initialProperties?.lockAspectRatio ?? true);

  // Calculate aspect ratio
  const aspectRatio = originalWidth / originalHeight;

  // Reset form when dialog opens with new properties
  useEffect(() => {
    if (isOpen) {
      setWidth(initialProperties?.width ?? originalWidth);
      setHeight(initialProperties?.height ?? originalHeight);
      setWrapType(initialProperties?.wrapType ?? 'Inline');
      setRotation(initialProperties?.rotation ?? 0);
      setAltText(initialProperties?.altText ?? '');
      setTitle(initialProperties?.title ?? '');
      setLockAspectRatio(initialProperties?.lockAspectRatio ?? true);
    }
  }, [isOpen, initialProperties, originalWidth, originalHeight]);

  // Handle width change with aspect ratio lock
  const handleWidthChange = useCallback(
    (newWidth: number) => {
      setWidth(newWidth);
      if (lockAspectRatio && aspectRatio > 0) {
        setHeight(Math.round(newWidth / aspectRatio));
      }
    },
    [lockAspectRatio, aspectRatio]
  );

  // Handle height change with aspect ratio lock
  const handleHeightChange = useCallback(
    (newHeight: number) => {
      setHeight(newHeight);
      if (lockAspectRatio && aspectRatio > 0) {
        setWidth(Math.round(newHeight * aspectRatio));
      }
    },
    [lockAspectRatio, aspectRatio]
  );

  // Reset to original size
  const handleResetSize = useCallback(() => {
    setWidth(originalWidth);
    setHeight(originalHeight);
  }, [originalWidth, originalHeight]);

  // Handle save
  const handleSave = () => {
    onSave({
      width,
      height,
      wrapType,
      rotation,
      altText: altText || undefined,
      title: title || undefined,
      lockAspectRatio,
    });
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog image-properties-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>Image Properties</h2>
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
                <label htmlFor="img-width">Width:</label>
                <input
                  type="number"
                  id="img-width"
                  value={width}
                  onChange={(e) => handleWidthChange(parseInt(e.target.value) || 0)}
                  min={1}
                  max={5000}
                />
                <span className="unit">px</span>
              </div>
              <div className="form-row">
                <label htmlFor="img-height">Height:</label>
                <input
                  type="number"
                  id="img-height"
                  value={height}
                  onChange={(e) => handleHeightChange(parseInt(e.target.value) || 0)}
                  min={1}
                  max={5000}
                />
                <span className="unit">px</span>
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
                <button type="button" className="reset-button" onClick={handleResetSize}>
                  Reset to Original
                </button>
              </div>
            </div>
          </div>

          {/* Wrap Type Section */}
          <div className="form-section">
            <h3>Text Wrapping</h3>
            <div className="wrap-options">
              {WRAP_TYPE_OPTIONS.map((option) => (
                <label key={option.value} className="wrap-option">
                  <input
                    type="radio"
                    name="wrapType"
                    value={option.value}
                    checked={wrapType === option.value}
                    onChange={() => setWrapType(option.value)}
                  />
                  <div className="wrap-option-content">
                    <span className="wrap-label">{option.label}</span>
                    <span className="wrap-description">{option.description}</span>
                  </div>
                </label>
              ))}
            </div>
          </div>

          {/* Rotation Section */}
          <div className="form-section">
            <h3>Rotation</h3>
            <div className="form-row">
              <label htmlFor="img-rotation">Angle:</label>
              <input
                type="number"
                id="img-rotation"
                value={rotation}
                onChange={(e) => setRotation(parseFloat(e.target.value) || 0)}
                min={-360}
                max={360}
                step={1}
              />
              <span className="unit">degrees</span>
            </div>
            <div className="rotation-presets">
              <button type="button" onClick={() => setRotation(0)}>0</button>
              <button type="button" onClick={() => setRotation(90)}>90</button>
              <button type="button" onClick={() => setRotation(180)}>180</button>
              <button type="button" onClick={() => setRotation(270)}>270</button>
            </div>
          </div>

          {/* Alt Text Section */}
          <div className="form-section">
            <h3>Accessibility</h3>
            <div className="form-row full-width">
              <label htmlFor="img-alt">Alt Text:</label>
              <input
                type="text"
                id="img-alt"
                value={altText}
                onChange={(e) => setAltText(e.target.value)}
                placeholder="Describe the image for screen readers"
              />
            </div>
            <div className="form-row full-width">
              <label htmlFor="img-title">Title (tooltip):</label>
              <input
                type="text"
                id="img-title"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="Text shown when hovering over the image"
              />
            </div>
          </div>
        </div>

        <div className="dialog-footer">
          <button type="button" className="cancel-button" onClick={onClose}>
            Cancel
          </button>
          <button type="button" className="save-button" onClick={handleSave}>
            Save
          </button>
        </div>
      </div>

      <style>{`
        .image-properties-dialog {
          width: 500px;
          max-height: 80vh;
          overflow-y: auto;
        }

        .form-section {
          margin-bottom: 20px;
          padding-bottom: 20px;
          border-bottom: 1px solid #e0e0e0;
        }

        .form-section:last-child {
          border-bottom: none;
          margin-bottom: 0;
        }

        .form-section h3 {
          margin: 0 0 12px 0;
          font-size: 14px;
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
          min-width: 80px;
          font-size: 13px;
        }

        .form-row input[type="number"],
        .form-row input[type="text"] {
          flex: 1;
          padding: 6px 8px;
          border: 1px solid #ccc;
          border-radius: 4px;
          font-size: 13px;
        }

        .form-row input[type="number"] {
          max-width: 100px;
        }

        .form-row.full-width input[type="text"] {
          max-width: none;
        }

        .unit {
          font-size: 12px;
          color: #666;
          min-width: 40px;
        }

        .checkbox-row {
          justify-content: space-between;
          margin-top: 8px;
        }

        .checkbox-row label {
          min-width: auto;
          display: flex;
          align-items: center;
          gap: 6px;
          cursor: pointer;
        }

        .reset-button {
          padding: 4px 8px;
          font-size: 12px;
          background: #f0f0f0;
          border: 1px solid #ccc;
          border-radius: 4px;
          cursor: pointer;
        }

        .reset-button:hover {
          background: #e0e0e0;
        }

        .wrap-options {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .wrap-option {
          display: flex;
          align-items: flex-start;
          gap: 8px;
          padding: 8px;
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          cursor: pointer;
          transition: border-color 0.15s, background 0.15s;
        }

        .wrap-option:hover {
          border-color: #0066cc;
          background: #f8f9fa;
        }

        .wrap-option input[type="radio"] {
          margin-top: 2px;
        }

        .wrap-option-content {
          display: flex;
          flex-direction: column;
          gap: 2px;
        }

        .wrap-label {
          font-size: 13px;
          font-weight: 500;
        }

        .wrap-description {
          font-size: 11px;
          color: #666;
        }

        .rotation-presets {
          display: flex;
          gap: 8px;
          margin-top: 8px;
        }

        .rotation-presets button {
          padding: 4px 12px;
          font-size: 12px;
          background: #f0f0f0;
          border: 1px solid #ccc;
          border-radius: 4px;
          cursor: pointer;
        }

        .rotation-presets button:hover {
          background: #e0e0e0;
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
          font-size: 13px;
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
  );
}
