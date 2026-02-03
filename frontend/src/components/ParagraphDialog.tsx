import { useState, useCallback, useEffect } from 'react';
import './ParagraphDialog.css';

export interface ParagraphSettings {
  // Alignment
  alignment?: 'left' | 'center' | 'right' | 'justify';
  // Indentation (in points)
  indentLeft?: number;
  indentRight?: number;
  indentFirstLine?: number;
  indentType?: 'none' | 'first-line' | 'hanging';
  // Spacing (in points)
  spaceBefore?: number;
  spaceAfter?: number;
  lineSpacing?: number;
  lineSpacingType?: 'multiple' | 'exact' | 'at-least';
  // Pagination
  keepWithNext?: boolean;
  keepTogether?: boolean;
  pageBreakBefore?: boolean;
  widowControl?: boolean;
}

interface ParagraphDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onApply: (settings: ParagraphSettings) => void;
  currentSettings?: ParagraphSettings;
}

export function ParagraphDialog({
  isOpen,
  onClose,
  onApply,
  currentSettings,
}: ParagraphDialogProps) {
  // Indentation state
  const [indentLeft, setIndentLeft] = useState(0);
  const [indentRight, setIndentRight] = useState(0);
  const [indentType, setIndentType] = useState<'none' | 'first-line' | 'hanging'>('none');
  const [indentSpecial, setIndentSpecial] = useState(0);

  // Spacing state
  const [spaceBefore, setSpaceBefore] = useState(0);
  const [spaceAfter, setSpaceAfter] = useState(8);
  const [lineSpacingType, setLineSpacingType] = useState<'multiple' | 'exact' | 'at-least'>('multiple');
  const [lineSpacing, setLineSpacing] = useState(1.15);

  // Pagination state
  const [keepWithNext, setKeepWithNext] = useState(false);
  const [keepTogether, setKeepTogether] = useState(false);
  const [pageBreakBefore, setPageBreakBefore] = useState(false);
  const [widowControl, setWidowControl] = useState(true);

  // Active tab
  const [activeTab, setActiveTab] = useState<'indents' | 'spacing' | 'pagination'>('indents');

  // Reset form when dialog opens
  useEffect(() => {
    if (isOpen) {
      if (currentSettings) {
        setIndentLeft(currentSettings.indentLeft ?? 0);
        setIndentRight(currentSettings.indentRight ?? 0);
        setIndentType(currentSettings.indentType ?? 'none');
        setIndentSpecial(Math.abs(currentSettings.indentFirstLine ?? 0));
        setSpaceBefore(currentSettings.spaceBefore ?? 0);
        setSpaceAfter(currentSettings.spaceAfter ?? 8);
        setLineSpacingType(currentSettings.lineSpacingType ?? 'multiple');
        setLineSpacing(currentSettings.lineSpacing ?? 1.15);
        setKeepWithNext(currentSettings.keepWithNext ?? false);
        setKeepTogether(currentSettings.keepTogether ?? false);
        setPageBreakBefore(currentSettings.pageBreakBefore ?? false);
        setWidowControl(currentSettings.widowControl ?? true);
      } else {
        // Reset to defaults
        setIndentLeft(0);
        setIndentRight(0);
        setIndentType('none');
        setIndentSpecial(0);
        setSpaceBefore(0);
        setSpaceAfter(8);
        setLineSpacingType('multiple');
        setLineSpacing(1.15);
        setKeepWithNext(false);
        setKeepTogether(false);
        setPageBreakBefore(false);
        setWidowControl(true);
      }
    }
  }, [isOpen, currentSettings]);

  const handleApply = useCallback(() => {
    const settings: ParagraphSettings = {
      indentLeft,
      indentRight,
      indentType,
      indentFirstLine: indentType === 'none' ? 0 :
                       indentType === 'first-line' ? indentSpecial : -indentSpecial,
      spaceBefore,
      spaceAfter,
      lineSpacingType,
      lineSpacing,
      keepWithNext,
      keepTogether,
      pageBreakBefore,
      widowControl,
    };
    onApply(settings);
  }, [
    indentLeft, indentRight, indentType, indentSpecial,
    spaceBefore, spaceAfter, lineSpacingType, lineSpacing,
    keepWithNext, keepTogether, pageBreakBefore, widowControl,
    onApply,
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

  // Calculate preview line positions based on settings
  const previewIndentLeft = (indentLeft / 72) * 20; // Convert points to preview pixels
  const previewIndentRight = (indentRight / 72) * 20;
  const previewFirstLine = indentType === 'first-line' ? (indentSpecial / 72) * 20 :
                          indentType === 'hanging' ? -(indentSpecial / 72) * 20 : 0;

  return (
    <div className="paragraph-dialog-overlay" onClick={onClose} onKeyDown={handleKeyDown}>
      <div className="paragraph-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="paragraph-dialog-header">
          <h2>Paragraph</h2>
          <button className="close-button" onClick={onClose}>x</button>
        </div>

        {/* Tab navigation */}
        <div className="paragraph-dialog-tabs">
          <button
            className={`tab-button ${activeTab === 'indents' ? 'active' : ''}`}
            onClick={() => setActiveTab('indents')}
          >
            Indents & Spacing
          </button>
          <button
            className={`tab-button ${activeTab === 'pagination' ? 'active' : ''}`}
            onClick={() => setActiveTab('pagination')}
          >
            Line and Page Breaks
          </button>
        </div>

        <div className="paragraph-dialog-content">
          {activeTab === 'indents' && (
            <>
              {/* Indentation Section */}
              <fieldset className="form-section">
                <legend>Indentation</legend>
                <div className="form-row">
                  <div className="form-field">
                    <label htmlFor="indent-left">Left:</label>
                    <div className="input-with-unit">
                      <input
                        type="number"
                        id="indent-left"
                        value={indentLeft}
                        onChange={(e) => setIndentLeft(parseFloat(e.target.value) || 0)}
                        min="0"
                        step="6"
                      />
                      <span className="unit">pt</span>
                    </div>
                  </div>
                  <div className="form-field">
                    <label htmlFor="indent-right">Right:</label>
                    <div className="input-with-unit">
                      <input
                        type="number"
                        id="indent-right"
                        value={indentRight}
                        onChange={(e) => setIndentRight(parseFloat(e.target.value) || 0)}
                        min="0"
                        step="6"
                      />
                      <span className="unit">pt</span>
                    </div>
                  </div>
                </div>
                <div className="form-row">
                  <div className="form-field">
                    <label htmlFor="indent-type">Special:</label>
                    <select
                      id="indent-type"
                      value={indentType}
                      onChange={(e) => setIndentType(e.target.value as 'none' | 'first-line' | 'hanging')}
                    >
                      <option value="none">(none)</option>
                      <option value="first-line">First line</option>
                      <option value="hanging">Hanging</option>
                    </select>
                  </div>
                  <div className="form-field">
                    <label htmlFor="indent-special">By:</label>
                    <div className="input-with-unit">
                      <input
                        type="number"
                        id="indent-special"
                        value={indentSpecial}
                        onChange={(e) => setIndentSpecial(parseFloat(e.target.value) || 0)}
                        min="0"
                        step="6"
                        disabled={indentType === 'none'}
                      />
                      <span className="unit">pt</span>
                    </div>
                  </div>
                </div>
              </fieldset>

              {/* Spacing Section */}
              <fieldset className="form-section">
                <legend>Spacing</legend>
                <div className="form-row">
                  <div className="form-field">
                    <label htmlFor="space-before">Before:</label>
                    <div className="input-with-unit">
                      <input
                        type="number"
                        id="space-before"
                        value={spaceBefore}
                        onChange={(e) => setSpaceBefore(parseFloat(e.target.value) || 0)}
                        min="0"
                        step="6"
                      />
                      <span className="unit">pt</span>
                    </div>
                  </div>
                  <div className="form-field">
                    <label htmlFor="space-after">After:</label>
                    <div className="input-with-unit">
                      <input
                        type="number"
                        id="space-after"
                        value={spaceAfter}
                        onChange={(e) => setSpaceAfter(parseFloat(e.target.value) || 0)}
                        min="0"
                        step="6"
                      />
                      <span className="unit">pt</span>
                    </div>
                  </div>
                </div>
                <div className="form-row">
                  <div className="form-field">
                    <label htmlFor="line-spacing-type">Line spacing:</label>
                    <select
                      id="line-spacing-type"
                      value={lineSpacingType}
                      onChange={(e) => setLineSpacingType(e.target.value as 'multiple' | 'exact' | 'at-least')}
                    >
                      <option value="multiple">Multiple</option>
                      <option value="exact">Exactly</option>
                      <option value="at-least">At least</option>
                    </select>
                  </div>
                  <div className="form-field">
                    <label htmlFor="line-spacing">At:</label>
                    <div className="input-with-unit">
                      <input
                        type="number"
                        id="line-spacing"
                        value={lineSpacing}
                        onChange={(e) => setLineSpacing(parseFloat(e.target.value) || 1)}
                        min={lineSpacingType === 'multiple' ? 0.5 : 6}
                        step={lineSpacingType === 'multiple' ? 0.05 : 1}
                      />
                      <span className="unit">{lineSpacingType === 'multiple' ? '' : 'pt'}</span>
                    </div>
                  </div>
                </div>
              </fieldset>

              {/* Preview */}
              <div className="preview-section">
                <label>Preview:</label>
                <div className="paragraph-preview">
                  <div className="preview-margins">
                    <div
                      className="preview-line first-line"
                      style={{
                        marginLeft: `${previewIndentLeft + previewFirstLine}px`,
                        marginRight: `${previewIndentRight}px`,
                      }}
                    />
                    <div
                      className="preview-line"
                      style={{
                        marginLeft: `${previewIndentLeft}px`,
                        marginRight: `${previewIndentRight}px`,
                      }}
                    />
                    <div
                      className="preview-line"
                      style={{
                        marginLeft: `${previewIndentLeft}px`,
                        marginRight: `${previewIndentRight}px`,
                        width: '60%',
                      }}
                    />
                  </div>
                </div>
              </div>
            </>
          )}

          {activeTab === 'pagination' && (
            <fieldset className="form-section">
              <legend>Pagination</legend>
              <div className="checkbox-group">
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={widowControl}
                    onChange={(e) => setWidowControl(e.target.checked)}
                  />
                  <span>Widow/Orphan control</span>
                </label>
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={keepWithNext}
                    onChange={(e) => setKeepWithNext(e.target.checked)}
                  />
                  <span>Keep with next</span>
                </label>
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={keepTogether}
                    onChange={(e) => setKeepTogether(e.target.checked)}
                  />
                  <span>Keep lines together</span>
                </label>
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={pageBreakBefore}
                    onChange={(e) => setPageBreakBefore(e.target.checked)}
                  />
                  <span>Page break before</span>
                </label>
              </div>
            </fieldset>
          )}
        </div>

        <div className="paragraph-dialog-footer">
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
