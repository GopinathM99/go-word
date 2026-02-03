import { useState, useCallback, useEffect } from 'react';
import './PageSetupDialog.css';

// Page size presets in points
const PAGE_SIZES = {
  Letter: { width: 612, height: 792, label: 'Letter (8.5" x 11")' },
  A4: { width: 595.276, height: 841.89, label: 'A4 (210mm x 297mm)' },
  Legal: { width: 612, height: 1008, label: 'Legal (8.5" x 14")' },
  A3: { width: 841.89, height: 1190.55, label: 'A3 (297mm x 420mm)' },
  A5: { width: 419.53, height: 595.28, label: 'A5 (148mm x 210mm)' },
  B5: { width: 515.91, height: 728.50, label: 'B5 (182mm x 257mm)' },
  Executive: { width: 522, height: 756, label: 'Executive (7.25" x 10.5")' },
  Tabloid: { width: 792, height: 1224, label: 'Tabloid (11" x 17")' },
  Custom: { width: 0, height: 0, label: 'Custom Size' },
};

// Margin presets
const MARGIN_PRESETS = {
  Normal: { top: 72, bottom: 72, left: 72, right: 72, label: 'Normal (1" all)' },
  Narrow: { top: 36, bottom: 36, left: 36, right: 36, label: 'Narrow (0.5" all)' },
  Moderate: { top: 72, bottom: 72, left: 54, right: 54, label: 'Moderate' },
  Wide: { top: 72, bottom: 72, left: 144, right: 144, label: 'Wide (2" sides)' },
  Custom: { top: 0, bottom: 0, left: 0, right: 0, label: 'Custom' },
};

// Line numbering restart modes
export type LineNumberRestartMode = 'each-page' | 'each-section' | 'continuous';

// Line numbering settings
export interface LineNumberingSettings {
  enabled: boolean;
  startAt: number;
  countBy: number;
  restartMode: LineNumberRestartMode;
  distanceFromText: number; // in points
}

export interface PageSetupSettings {
  // Paper size
  paperSize: keyof typeof PAGE_SIZES;
  customWidth?: number;
  customHeight?: number;
  orientation: 'portrait' | 'landscape';
  // Margins
  marginTop: number;
  marginBottom: number;
  marginLeft: number;
  marginRight: number;
  marginHeader: number;
  marginFooter: number;
  gutter: number;
  gutterPosition: 'left' | 'top';
  // Layout
  sectionStart: 'new-page' | 'continuous' | 'even-page' | 'odd-page';
  differentFirstPage: boolean;
  differentOddEven: boolean;
  verticalAlignment: 'top' | 'center' | 'justified' | 'bottom';
  // Line numbering
  lineNumbering?: LineNumberingSettings;
}

interface PageSetupDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onApply: (settings: PageSetupSettings) => void;
  currentSettings?: Partial<PageSetupSettings>;
}

// Convert points to inches for display
function pointsToInches(points: number): number {
  return Math.round((points / 72) * 100) / 100;
}

// Convert inches to points
function inchesToPoints(inches: number): number {
  return inches * 72;
}

export function PageSetupDialog({
  isOpen,
  onClose,
  onApply,
  currentSettings,
}: PageSetupDialogProps) {
  // Active tab
  const [activeTab, setActiveTab] = useState<'margins' | 'paper' | 'layout' | 'line-numbers'>('margins');

  // Paper settings
  const [paperSize, setPaperSize] = useState<keyof typeof PAGE_SIZES>('Letter');
  const [customWidth, setCustomWidth] = useState(8.5);
  const [customHeight, setCustomHeight] = useState(11);
  const [orientation, setOrientation] = useState<'portrait' | 'landscape'>('portrait');

  // Margin settings (stored in inches for display)
  const [marginTop, setMarginTop] = useState(1);
  const [marginBottom, setMarginBottom] = useState(1);
  const [marginLeft, setMarginLeft] = useState(1);
  const [marginRight, setMarginRight] = useState(1);
  const [marginHeader, setMarginHeader] = useState(0.5);
  const [marginFooter, setMarginFooter] = useState(0.5);
  const [gutter, setGutter] = useState(0);
  const [gutterPosition, setGutterPosition] = useState<'left' | 'top'>('left');
  const [marginPreset, setMarginPreset] = useState<keyof typeof MARGIN_PRESETS>('Normal');

  // Layout settings
  const [sectionStart, setSectionStart] = useState<PageSetupSettings['sectionStart']>('new-page');
  const [differentFirstPage, setDifferentFirstPage] = useState(false);
  const [differentOddEven, setDifferentOddEven] = useState(false);
  const [verticalAlignment, setVerticalAlignment] = useState<PageSetupSettings['verticalAlignment']>('top');

  // Line numbering settings
  const [lineNumberingEnabled, setLineNumberingEnabled] = useState(false);
  const [lineNumberStartAt, setLineNumberStartAt] = useState(1);
  const [lineNumberCountBy, setLineNumberCountBy] = useState(1);
  const [lineNumberRestartMode, setLineNumberRestartMode] = useState<LineNumberRestartMode>('each-page');
  const [lineNumberDistanceFromText, setLineNumberDistanceFromText] = useState(0.25); // in inches

  // Reset form when dialog opens
  useEffect(() => {
    if (isOpen && currentSettings) {
      if (currentSettings.paperSize) setPaperSize(currentSettings.paperSize);
      if (currentSettings.customWidth) setCustomWidth(pointsToInches(currentSettings.customWidth));
      if (currentSettings.customHeight) setCustomHeight(pointsToInches(currentSettings.customHeight));
      if (currentSettings.orientation) setOrientation(currentSettings.orientation);
      if (currentSettings.marginTop !== undefined) setMarginTop(pointsToInches(currentSettings.marginTop));
      if (currentSettings.marginBottom !== undefined) setMarginBottom(pointsToInches(currentSettings.marginBottom));
      if (currentSettings.marginLeft !== undefined) setMarginLeft(pointsToInches(currentSettings.marginLeft));
      if (currentSettings.marginRight !== undefined) setMarginRight(pointsToInches(currentSettings.marginRight));
      if (currentSettings.marginHeader !== undefined) setMarginHeader(pointsToInches(currentSettings.marginHeader));
      if (currentSettings.marginFooter !== undefined) setMarginFooter(pointsToInches(currentSettings.marginFooter));
      if (currentSettings.gutter !== undefined) setGutter(pointsToInches(currentSettings.gutter));
      if (currentSettings.gutterPosition) setGutterPosition(currentSettings.gutterPosition);
      if (currentSettings.sectionStart) setSectionStart(currentSettings.sectionStart);
      if (currentSettings.differentFirstPage !== undefined) setDifferentFirstPage(currentSettings.differentFirstPage);
      if (currentSettings.differentOddEven !== undefined) setDifferentOddEven(currentSettings.differentOddEven);
      if (currentSettings.verticalAlignment) setVerticalAlignment(currentSettings.verticalAlignment);
      // Line numbering settings
      if (currentSettings.lineNumbering) {
        setLineNumberingEnabled(currentSettings.lineNumbering.enabled);
        setLineNumberStartAt(currentSettings.lineNumbering.startAt);
        setLineNumberCountBy(currentSettings.lineNumbering.countBy);
        setLineNumberRestartMode(currentSettings.lineNumbering.restartMode);
        setLineNumberDistanceFromText(pointsToInches(currentSettings.lineNumbering.distanceFromText));
      }
    }
  }, [isOpen, currentSettings]);

  // Handle margin preset selection
  const handleMarginPreset = useCallback((preset: keyof typeof MARGIN_PRESETS) => {
    setMarginPreset(preset);
    if (preset !== 'Custom') {
      const presetValues = MARGIN_PRESETS[preset];
      setMarginTop(pointsToInches(presetValues.top));
      setMarginBottom(pointsToInches(presetValues.bottom));
      setMarginLeft(pointsToInches(presetValues.left));
      setMarginRight(pointsToInches(presetValues.right));
    }
  }, []);

  // Handle paper size selection
  const handlePaperSize = useCallback((size: keyof typeof PAGE_SIZES) => {
    setPaperSize(size);
    if (size !== 'Custom') {
      const sizeValues = PAGE_SIZES[size];
      setCustomWidth(pointsToInches(sizeValues.width));
      setCustomHeight(pointsToInches(sizeValues.height));
    }
  }, []);

  const handleApply = useCallback(() => {
    const settings: PageSetupSettings = {
      paperSize,
      customWidth: inchesToPoints(customWidth),
      customHeight: inchesToPoints(customHeight),
      orientation,
      marginTop: inchesToPoints(marginTop),
      marginBottom: inchesToPoints(marginBottom),
      marginLeft: inchesToPoints(marginLeft),
      marginRight: inchesToPoints(marginRight),
      marginHeader: inchesToPoints(marginHeader),
      marginFooter: inchesToPoints(marginFooter),
      gutter: inchesToPoints(gutter),
      gutterPosition,
      sectionStart,
      differentFirstPage,
      differentOddEven,
      verticalAlignment,
      lineNumbering: {
        enabled: lineNumberingEnabled,
        startAt: lineNumberStartAt,
        countBy: lineNumberCountBy,
        restartMode: lineNumberRestartMode,
        distanceFromText: inchesToPoints(lineNumberDistanceFromText),
      },
    };
    onApply(settings);
    onClose();
  }, [
    paperSize, customWidth, customHeight, orientation,
    marginTop, marginBottom, marginLeft, marginRight,
    marginHeader, marginFooter, gutter, gutterPosition,
    sectionStart, differentFirstPage, differentOddEven, verticalAlignment,
    lineNumberingEnabled, lineNumberStartAt, lineNumberCountBy,
    lineNumberRestartMode, lineNumberDistanceFromText,
    onApply, onClose,
  ]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
    } else if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleApply();
    }
  }, [onClose, handleApply]);

  if (!isOpen) return null;

  // Calculate preview dimensions
  const previewScale = 0.15;
  let pageWidth = paperSize === 'Custom'
    ? inchesToPoints(customWidth)
    : PAGE_SIZES[paperSize].width;
  let pageHeight = paperSize === 'Custom'
    ? inchesToPoints(customHeight)
    : PAGE_SIZES[paperSize].height;

  // Apply orientation
  if (orientation === 'landscape') {
    [pageWidth, pageHeight] = [pageHeight, pageWidth];
  }

  const previewWidth = pageWidth * previewScale;
  const previewHeight = pageHeight * previewScale;
  const previewMarginTop = inchesToPoints(marginTop) * previewScale;
  const previewMarginBottom = inchesToPoints(marginBottom) * previewScale;
  const previewMarginLeft = inchesToPoints(marginLeft) * previewScale;
  const previewMarginRight = inchesToPoints(marginRight) * previewScale;

  return (
    <div className="page-setup-dialog-overlay" onClick={onClose} onKeyDown={handleKeyDown}>
      <div className="page-setup-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="page-setup-dialog-header">
          <h2>Page Setup</h2>
          <button className="close-button" onClick={onClose}>x</button>
        </div>

        {/* Tab navigation */}
        <div className="page-setup-dialog-tabs">
          <button
            className={`tab-button ${activeTab === 'margins' ? 'active' : ''}`}
            onClick={() => setActiveTab('margins')}
          >
            Margins
          </button>
          <button
            className={`tab-button ${activeTab === 'paper' ? 'active' : ''}`}
            onClick={() => setActiveTab('paper')}
          >
            Paper
          </button>
          <button
            className={`tab-button ${activeTab === 'layout' ? 'active' : ''}`}
            onClick={() => setActiveTab('layout')}
          >
            Layout
          </button>
          <button
            className={`tab-button ${activeTab === 'line-numbers' ? 'active' : ''}`}
            onClick={() => setActiveTab('line-numbers')}
          >
            Line Numbers
          </button>
        </div>

        <div className="page-setup-dialog-content">
          <div className="page-setup-main">
            {activeTab === 'margins' && (
              <>
                {/* Margin Presets */}
                <fieldset className="form-section">
                  <legend>Preset</legend>
                  <select
                    value={marginPreset}
                    onChange={(e) => handleMarginPreset(e.target.value as keyof typeof MARGIN_PRESETS)}
                    className="preset-select"
                  >
                    {Object.entries(MARGIN_PRESETS).map(([key, value]) => (
                      <option key={key} value={key}>{value.label}</option>
                    ))}
                  </select>
                </fieldset>

                {/* Margins */}
                <fieldset className="form-section">
                  <legend>Margins</legend>
                  <div className="form-row">
                    <div className="form-field">
                      <label htmlFor="margin-top">Top:</label>
                      <div className="input-with-unit">
                        <input
                          type="number"
                          id="margin-top"
                          value={marginTop}
                          onChange={(e) => {
                            setMarginTop(parseFloat(e.target.value) || 0);
                            setMarginPreset('Custom');
                          }}
                          min="0"
                          max="5"
                          step="0.1"
                        />
                        <span className="unit">"</span>
                      </div>
                    </div>
                    <div className="form-field">
                      <label htmlFor="margin-bottom">Bottom:</label>
                      <div className="input-with-unit">
                        <input
                          type="number"
                          id="margin-bottom"
                          value={marginBottom}
                          onChange={(e) => {
                            setMarginBottom(parseFloat(e.target.value) || 0);
                            setMarginPreset('Custom');
                          }}
                          min="0"
                          max="5"
                          step="0.1"
                        />
                        <span className="unit">"</span>
                      </div>
                    </div>
                  </div>
                  <div className="form-row">
                    <div className="form-field">
                      <label htmlFor="margin-left">Left:</label>
                      <div className="input-with-unit">
                        <input
                          type="number"
                          id="margin-left"
                          value={marginLeft}
                          onChange={(e) => {
                            setMarginLeft(parseFloat(e.target.value) || 0);
                            setMarginPreset('Custom');
                          }}
                          min="0"
                          max="5"
                          step="0.1"
                        />
                        <span className="unit">"</span>
                      </div>
                    </div>
                    <div className="form-field">
                      <label htmlFor="margin-right">Right:</label>
                      <div className="input-with-unit">
                        <input
                          type="number"
                          id="margin-right"
                          value={marginRight}
                          onChange={(e) => {
                            setMarginRight(parseFloat(e.target.value) || 0);
                            setMarginPreset('Custom');
                          }}
                          min="0"
                          max="5"
                          step="0.1"
                        />
                        <span className="unit">"</span>
                      </div>
                    </div>
                  </div>
                  <div className="form-row">
                    <div className="form-field">
                      <label htmlFor="gutter">Gutter:</label>
                      <div className="input-with-unit">
                        <input
                          type="number"
                          id="gutter"
                          value={gutter}
                          onChange={(e) => setGutter(parseFloat(e.target.value) || 0)}
                          min="0"
                          max="2"
                          step="0.1"
                        />
                        <span className="unit">"</span>
                      </div>
                    </div>
                    <div className="form-field">
                      <label htmlFor="gutter-position">Gutter position:</label>
                      <select
                        id="gutter-position"
                        value={gutterPosition}
                        onChange={(e) => setGutterPosition(e.target.value as 'left' | 'top')}
                      >
                        <option value="left">Left</option>
                        <option value="top">Top</option>
                      </select>
                    </div>
                  </div>
                </fieldset>

                {/* Header/Footer distances */}
                <fieldset className="form-section">
                  <legend>From edge</legend>
                  <div className="form-row">
                    <div className="form-field">
                      <label htmlFor="margin-header">Header:</label>
                      <div className="input-with-unit">
                        <input
                          type="number"
                          id="margin-header"
                          value={marginHeader}
                          onChange={(e) => setMarginHeader(parseFloat(e.target.value) || 0)}
                          min="0"
                          max="2"
                          step="0.1"
                        />
                        <span className="unit">"</span>
                      </div>
                    </div>
                    <div className="form-field">
                      <label htmlFor="margin-footer">Footer:</label>
                      <div className="input-with-unit">
                        <input
                          type="number"
                          id="margin-footer"
                          value={marginFooter}
                          onChange={(e) => setMarginFooter(parseFloat(e.target.value) || 0)}
                          min="0"
                          max="2"
                          step="0.1"
                        />
                        <span className="unit">"</span>
                      </div>
                    </div>
                  </div>
                </fieldset>
              </>
            )}

            {activeTab === 'paper' && (
              <>
                {/* Paper Size */}
                <fieldset className="form-section">
                  <legend>Paper size</legend>
                  <select
                    value={paperSize}
                    onChange={(e) => handlePaperSize(e.target.value as keyof typeof PAGE_SIZES)}
                    className="paper-size-select"
                  >
                    {Object.entries(PAGE_SIZES).map(([key, value]) => (
                      <option key={key} value={key}>{value.label}</option>
                    ))}
                  </select>
                  <div className="form-row">
                    <div className="form-field">
                      <label htmlFor="paper-width">Width:</label>
                      <div className="input-with-unit">
                        <input
                          type="number"
                          id="paper-width"
                          value={customWidth}
                          onChange={(e) => {
                            setCustomWidth(parseFloat(e.target.value) || 0);
                            setPaperSize('Custom');
                          }}
                          min="1"
                          max="50"
                          step="0.1"
                        />
                        <span className="unit">"</span>
                      </div>
                    </div>
                    <div className="form-field">
                      <label htmlFor="paper-height">Height:</label>
                      <div className="input-with-unit">
                        <input
                          type="number"
                          id="paper-height"
                          value={customHeight}
                          onChange={(e) => {
                            setCustomHeight(parseFloat(e.target.value) || 0);
                            setPaperSize('Custom');
                          }}
                          min="1"
                          max="50"
                          step="0.1"
                        />
                        <span className="unit">"</span>
                      </div>
                    </div>
                  </div>
                </fieldset>

                {/* Orientation */}
                <fieldset className="form-section">
                  <legend>Orientation</legend>
                  <div className="orientation-options">
                    <label className="orientation-option">
                      <input
                        type="radio"
                        name="orientation"
                        value="portrait"
                        checked={orientation === 'portrait'}
                        onChange={() => setOrientation('portrait')}
                      />
                      <div className="orientation-icon portrait">
                        <div className="orientation-page"></div>
                      </div>
                      <span>Portrait</span>
                    </label>
                    <label className="orientation-option">
                      <input
                        type="radio"
                        name="orientation"
                        value="landscape"
                        checked={orientation === 'landscape'}
                        onChange={() => setOrientation('landscape')}
                      />
                      <div className="orientation-icon landscape">
                        <div className="orientation-page"></div>
                      </div>
                      <span>Landscape</span>
                    </label>
                  </div>
                </fieldset>
              </>
            )}

            {activeTab === 'layout' && (
              <>
                {/* Section Start */}
                <fieldset className="form-section">
                  <legend>Section</legend>
                  <div className="form-field">
                    <label htmlFor="section-start">Section start:</label>
                    <select
                      id="section-start"
                      value={sectionStart}
                      onChange={(e) => setSectionStart(e.target.value as PageSetupSettings['sectionStart'])}
                    >
                      <option value="new-page">New page</option>
                      <option value="continuous">Continuous</option>
                      <option value="even-page">Even page</option>
                      <option value="odd-page">Odd page</option>
                    </select>
                  </div>
                </fieldset>

                {/* Headers and Footers */}
                <fieldset className="form-section">
                  <legend>Headers and footers</legend>
                  <div className="checkbox-group">
                    <label className="checkbox-label">
                      <input
                        type="checkbox"
                        checked={differentFirstPage}
                        onChange={(e) => setDifferentFirstPage(e.target.checked)}
                      />
                      <span>Different first page</span>
                    </label>
                    <label className="checkbox-label">
                      <input
                        type="checkbox"
                        checked={differentOddEven}
                        onChange={(e) => setDifferentOddEven(e.target.checked)}
                      />
                      <span>Different odd and even</span>
                    </label>
                  </div>
                </fieldset>

                {/* Vertical Alignment */}
                <fieldset className="form-section">
                  <legend>Page</legend>
                  <div className="form-field">
                    <label htmlFor="vertical-alignment">Vertical alignment:</label>
                    <select
                      id="vertical-alignment"
                      value={verticalAlignment}
                      onChange={(e) => setVerticalAlignment(e.target.value as PageSetupSettings['verticalAlignment'])}
                    >
                      <option value="top">Top</option>
                      <option value="center">Center</option>
                      <option value="justified">Justified</option>
                      <option value="bottom">Bottom</option>
                    </select>
                  </div>
                </fieldset>
              </>
            )}

            {activeTab === 'line-numbers' && (
              <>
                {/* Line Numbering Enable */}
                <fieldset className="form-section">
                  <legend>Line Numbers</legend>
                  <div className="checkbox-group">
                    <label className="checkbox-label">
                      <input
                        type="checkbox"
                        checked={lineNumberingEnabled}
                        onChange={(e) => setLineNumberingEnabled(e.target.checked)}
                      />
                      <span>Add line numbering</span>
                    </label>
                  </div>
                </fieldset>

                {/* Line Numbering Options */}
                <fieldset className="form-section" style={{ opacity: lineNumberingEnabled ? 1 : 0.5 }}>
                  <legend>Options</legend>
                  <div className="form-row">
                    <div className="form-field">
                      <label htmlFor="line-number-start">Start at:</label>
                      <input
                        type="number"
                        id="line-number-start"
                        value={lineNumberStartAt}
                        onChange={(e) => setLineNumberStartAt(Math.max(1, parseInt(e.target.value) || 1))}
                        min="1"
                        disabled={!lineNumberingEnabled}
                      />
                    </div>
                    <div className="form-field">
                      <label htmlFor="line-number-count-by">Count by:</label>
                      <div className="input-with-hint">
                        <input
                          type="number"
                          id="line-number-count-by"
                          value={lineNumberCountBy}
                          onChange={(e) => setLineNumberCountBy(Math.max(1, parseInt(e.target.value) || 1))}
                          min="1"
                          disabled={!lineNumberingEnabled}
                        />
                        <span className="field-hint">(show every N lines)</span>
                      </div>
                    </div>
                  </div>
                  <div className="form-row">
                    <div className="form-field">
                      <label htmlFor="line-number-restart">Restart:</label>
                      <select
                        id="line-number-restart"
                        value={lineNumberRestartMode}
                        onChange={(e) => setLineNumberRestartMode(e.target.value as LineNumberRestartMode)}
                        disabled={!lineNumberingEnabled}
                      >
                        <option value="each-page">Restart each page</option>
                        <option value="each-section">Restart each section</option>
                        <option value="continuous">Continuous</option>
                      </select>
                    </div>
                  </div>
                  <div className="form-row">
                    <div className="form-field">
                      <label htmlFor="line-number-distance">From text:</label>
                      <div className="input-with-unit">
                        <input
                          type="number"
                          id="line-number-distance"
                          value={lineNumberDistanceFromText}
                          onChange={(e) => setLineNumberDistanceFromText(parseFloat(e.target.value) || 0)}
                          min="0"
                          max="2"
                          step="0.05"
                          disabled={!lineNumberingEnabled}
                        />
                        <span className="unit">in</span>
                      </div>
                    </div>
                  </div>
                </fieldset>
              </>
            )}
          </div>

          {/* Preview Panel */}
          <div className="page-setup-preview">
            <label>Preview:</label>
            <div className="preview-container">
              <div
                className="preview-page"
                style={{
                  width: previewWidth,
                  height: previewHeight,
                }}
              >
                <div
                  className="preview-content"
                  style={{
                    top: previewMarginTop,
                    left: previewMarginLeft,
                    right: previewMarginRight,
                    bottom: previewMarginBottom,
                  }}
                >
                  <div className="preview-lines">
                    <div className="preview-line"></div>
                    <div className="preview-line"></div>
                    <div className="preview-line short"></div>
                    <div className="preview-line"></div>
                    <div className="preview-line"></div>
                    <div className="preview-line short"></div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div className="page-setup-dialog-footer">
          <div className="button-group">
            <button className="cancel-button" onClick={onClose}>
              Cancel
            </button>
            <button className="submit-button" onClick={handleApply}>
              OK
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
