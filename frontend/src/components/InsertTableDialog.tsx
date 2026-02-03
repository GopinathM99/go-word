import { useState, useCallback, useRef, useEffect } from 'react';
import './InsertTableDialog.css';

interface InsertTableDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onInsert: (rows: number, cols: number, width?: number) => void;
}

/**
 * Dialog for inserting a table with a visual grid picker or manual input.
 */
export function InsertTableDialog({
  isOpen,
  onClose,
  onInsert,
}: InsertTableDialogProps) {
  // Grid picker state
  const [hoverRows, setHoverRows] = useState(1);
  const [hoverCols, setHoverCols] = useState(1);
  const [selectedRows, setSelectedRows] = useState(3);
  const [selectedCols, setSelectedCols] = useState(3);

  // Manual input state
  const [showManualInput, setShowManualInput] = useState(false);
  const [manualRows, setManualRows] = useState('3');
  const [manualCols, setManualCols] = useState('3');
  const [tableWidth, setTableWidth] = useState('');
  const [widthType, setWidthType] = useState<'auto' | 'fixed'>('auto');

  const dialogRef = useRef<HTMLDialogElement>(null);

  // Reset state when dialog opens
  useEffect(() => {
    if (isOpen) {
      setHoverRows(1);
      setHoverCols(1);
      setSelectedRows(3);
      setSelectedCols(3);
      setShowManualInput(false);
      setManualRows('3');
      setManualCols('3');
      setTableWidth('');
      setWidthType('auto');
      dialogRef.current?.showModal();
    } else {
      dialogRef.current?.close();
    }
  }, [isOpen]);

  // Handle escape key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        onClose();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  const handleGridClick = useCallback(() => {
    setSelectedRows(hoverRows);
    setSelectedCols(hoverCols);
    handleInsert(hoverRows, hoverCols);
  }, [hoverRows, hoverCols]);

  const handleInsert = useCallback(
    (rows: number, cols: number) => {
      const width =
        widthType === 'fixed' && tableWidth
          ? parseFloat(tableWidth)
          : undefined;
      onInsert(rows, cols, width);
      onClose();
    },
    [widthType, tableWidth, onInsert, onClose]
  );

  const handleManualInsert = useCallback(() => {
    const rows = parseInt(manualRows, 10);
    const cols = parseInt(manualCols, 10);

    if (isNaN(rows) || rows < 1 || rows > 100) {
      return;
    }
    if (isNaN(cols) || cols < 1 || cols > 63) {
      return;
    }

    handleInsert(rows, cols);
  }, [manualRows, manualCols, handleInsert]);

  const handleBackdropClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === dialogRef.current) {
        onClose();
      }
    },
    [onClose]
  );

  if (!isOpen) {
    return null;
  }

  // Grid dimensions (8x10 is typical for Word)
  const gridRows = 8;
  const gridCols = 10;

  return (
    <dialog
      ref={dialogRef}
      className="insert-table-dialog"
      onClick={handleBackdropClick}
    >
      <div className="dialog-content">
        <div className="dialog-header">
          <h2>Insert Table</h2>
          <button className="close-button" onClick={onClose} aria-label="Close">
            ×
          </button>
        </div>

        {!showManualInput ? (
          <div className="grid-picker-section">
            <div className="grid-label">
              {hoverRows} x {hoverCols} Table
            </div>

            <div className="grid-picker">
              {Array.from({ length: gridRows }).map((_, rowIdx) => (
                <div key={rowIdx} className="grid-row">
                  {Array.from({ length: gridCols }).map((_, colIdx) => {
                    const isHovered =
                      rowIdx < hoverRows && colIdx < hoverCols;
                    return (
                      <div
                        key={colIdx}
                        className={`grid-cell ${isHovered ? 'hovered' : ''}`}
                        onMouseEnter={() => {
                          setHoverRows(rowIdx + 1);
                          setHoverCols(colIdx + 1);
                        }}
                        onClick={handleGridClick}
                      />
                    );
                  })}
                </div>
              ))}
            </div>

            <button
              className="manual-input-link"
              onClick={() => setShowManualInput(true)}
            >
              Insert Table...
            </button>
          </div>
        ) : (
          <div className="manual-input-section">
            <div className="input-group">
              <label htmlFor="table-rows">Number of rows:</label>
              <input
                id="table-rows"
                type="number"
                min="1"
                max="100"
                value={manualRows}
                onChange={(e) => setManualRows(e.target.value)}
              />
            </div>

            <div className="input-group">
              <label htmlFor="table-cols">Number of columns:</label>
              <input
                id="table-cols"
                type="number"
                min="1"
                max="63"
                value={manualCols}
                onChange={(e) => setManualCols(e.target.value)}
              />
            </div>

            <div className="input-group">
              <label>Table width:</label>
              <div className="width-options">
                <label>
                  <input
                    type="radio"
                    name="widthType"
                    value="auto"
                    checked={widthType === 'auto'}
                    onChange={() => setWidthType('auto')}
                  />
                  Auto
                </label>
                <label>
                  <input
                    type="radio"
                    name="widthType"
                    value="fixed"
                    checked={widthType === 'fixed'}
                    onChange={() => setWidthType('fixed')}
                  />
                  Fixed
                </label>
                {widthType === 'fixed' && (
                  <input
                    type="number"
                    min="50"
                    max="1000"
                    placeholder="Width in points"
                    value={tableWidth}
                    onChange={(e) => setTableWidth(e.target.value)}
                    className="width-input"
                  />
                )}
              </div>
            </div>

            <div className="dialog-actions">
              <button
                className="back-button"
                onClick={() => setShowManualInput(false)}
              >
                Back
              </button>
              <button className="insert-button" onClick={handleManualInsert}>
                Insert
              </button>
            </div>
          </div>
        )}
      </div>
    </dialog>
  );
}

export default InsertTableDialog;
