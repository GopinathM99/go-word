/**
 * PrintDialog - Print options dialog
 */

import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import '../styles/PrintPreview.css';

// =============================================================================
// Types
// =============================================================================

export interface PrintDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onPrint: (options: PrintOptions) => void;
  totalPages: number;
  currentPage: number;
  documentTitle: string;
}

export interface PrintOptions {
  printer?: string;
  pageRange: PageRange;
  copies: number;
  collate: boolean;
}

export type PageRange =
  | { type: 'all' }
  | { type: 'current'; page: number }
  | { type: 'custom'; pages: number[] };

interface PrinterInfo {
  name: string;
  isDefault: boolean;
  supportsColor: boolean;
  supportsDuplex: boolean;
}

interface PrintCapabilities {
  printers: PrinterInfo[];
  defaultPrinter: string | null;
}

// =============================================================================
// Utilities
// =============================================================================

function parsePageRange(input: string, maxPage: number): number[] | null {
  const pages = new Set<number>();
  const parts = input.split(',').map(s => s.trim());

  for (const part of parts) {
    if (part.includes('-')) {
      const [startStr, endStr] = part.split('-').map(s => s.trim());
      const start = parseInt(startStr, 10);
      const end = parseInt(endStr, 10);

      if (isNaN(start) || isNaN(end) || start < 1 || end > maxPage || start > end) {
        return null;
      }

      for (let i = start; i <= end; i++) {
        pages.add(i);
      }
    } else {
      const page = parseInt(part, 10);
      if (isNaN(page) || page < 1 || page > maxPage) {
        return null;
      }
      pages.add(page);
    }
  }

  return Array.from(pages).sort((a, b) => a - b);
}

// =============================================================================
// Component
// =============================================================================

export function PrintDialog({
  isOpen,
  onClose,
  onPrint,
  totalPages,
  currentPage,
  documentTitle,
}: PrintDialogProps) {
  const [printers, setPrinters] = useState<PrinterInfo[]>([]);
  const [selectedPrinter, setSelectedPrinter] = useState<string>('');
  const [pageRangeType, setPageRangeType] = useState<'all' | 'current' | 'custom'>('all');
  const [customRange, setCustomRange] = useState('');
  const [copies, setCopies] = useState(1);
  const [collate, setCollate] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  // Load printers when dialog opens
  useEffect(() => {
    if (isOpen) {
      loadPrinters();
    }
  }, [isOpen]);

  const loadPrinters = async () => {
    try {
      const capabilities = await invoke<PrintCapabilities>('get_print_capabilities');
      setPrinters(capabilities.printers);
      if (capabilities.defaultPrinter) {
        setSelectedPrinter(capabilities.defaultPrinter);
      } else if (capabilities.printers.length > 0) {
        setSelectedPrinter(capabilities.printers[0].name);
      }
    } catch (err) {
      console.error('Failed to load printers:', err);
      // Set a default "system" printer option
      setPrinters([{ name: 'System Default', isDefault: true, supportsColor: true, supportsDuplex: false }]);
      setSelectedPrinter('System Default');
    }
  };

  const handlePrint = useCallback(() => {
    setError(null);

    // Validate custom range
    let pageRange: PageRange;
    if (pageRangeType === 'all') {
      pageRange = { type: 'all' };
    } else if (pageRangeType === 'current') {
      pageRange = { type: 'current', page: currentPage };
    } else {
      const pages = parsePageRange(customRange, totalPages);
      if (!pages || pages.length === 0) {
        setError('Invalid page range. Use format like "1-5, 8, 10-12"');
        return;
      }
      pageRange = { type: 'custom', pages };
    }

    const options: PrintOptions = {
      printer: selectedPrinter,
      pageRange,
      copies,
      collate,
    };

    onPrint(options);
  }, [pageRangeType, customRange, totalPages, currentPage, selectedPrinter, copies, collate, onPrint]);

  // Handle escape key
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) {
    return null;
  }

  return (
    <div className="print-dialog-overlay" onClick={onClose}>
      <div className="print-dialog" onClick={e => e.stopPropagation()} role="dialog" aria-label="Print options">
        <div className="print-dialog-header">
          <h2 className="print-dialog-title">Print</h2>
          <button className="print-dialog-close" onClick={onClose} aria-label="Close">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z"/>
            </svg>
          </button>
        </div>

        <div className="print-dialog-body">
          {/* Document info */}
          <div className="print-dialog-section">
            <span className="print-dialog-doc-title">{documentTitle}</span>
          </div>

          {/* Printer selection */}
          <div className="print-dialog-section">
            <label className="print-dialog-label">Printer</label>
            <select
              className="print-dialog-select"
              value={selectedPrinter}
              onChange={e => setSelectedPrinter(e.target.value)}
            >
              {printers.map(printer => (
                <option key={printer.name} value={printer.name}>
                  {printer.name} {printer.isDefault ? '(Default)' : ''}
                </option>
              ))}
            </select>
          </div>

          {/* Page range */}
          <div className="print-dialog-section">
            <label className="print-dialog-label">Pages</label>
            <div className="print-dialog-radio-group">
              <div className="print-dialog-radio">
                <input
                  type="radio"
                  id="range-all"
                  name="pageRange"
                  checked={pageRangeType === 'all'}
                  onChange={() => setPageRangeType('all')}
                />
                <label htmlFor="range-all">All pages ({totalPages})</label>
              </div>

              <div className="print-dialog-radio">
                <input
                  type="radio"
                  id="range-current"
                  name="pageRange"
                  checked={pageRangeType === 'current'}
                  onChange={() => setPageRangeType('current')}
                />
                <label htmlFor="range-current">Current page ({currentPage})</label>
              </div>

              <div className="print-dialog-radio">
                <input
                  type="radio"
                  id="range-custom"
                  name="pageRange"
                  checked={pageRangeType === 'custom'}
                  onChange={() => setPageRangeType('custom')}
                />
                <label htmlFor="range-custom">Custom range</label>
              </div>

              {pageRangeType === 'custom' && (
                <div className="print-dialog-range-input">
                  <input
                    type="text"
                    placeholder="e.g., 1-5, 8, 10-12"
                    value={customRange}
                    onChange={e => setCustomRange(e.target.value)}
                  />
                </div>
              )}
            </div>
          </div>

          {/* Copies */}
          <div className="print-dialog-section">
            <label className="print-dialog-label">Copies</label>
            <div className="print-dialog-copies">
              <div className="print-dialog-copies-input">
                <input
                  type="number"
                  min={1}
                  max={999}
                  value={copies}
                  onChange={e => setCopies(Math.max(1, parseInt(e.target.value, 10) || 1))}
                />
              </div>

              {copies > 1 && (
                <div className="print-dialog-checkbox">
                  <input
                    type="checkbox"
                    id="collate"
                    checked={collate}
                    onChange={e => setCollate(e.target.checked)}
                  />
                  <label htmlFor="collate">Collate</label>
                </div>
              )}
            </div>
          </div>

          {/* Error message */}
          {error && (
            <div className="print-dialog-error">{error}</div>
          )}
        </div>

        <div className="print-dialog-footer">
          <button className="print-dialog-btn print-dialog-btn-cancel" onClick={onClose}>
            Cancel
          </button>
          <button className="print-dialog-btn print-dialog-btn-print" onClick={handlePrint}>
            Print
          </button>
        </div>
      </div>
    </div>
  );
}

export default PrintDialog;
