/**
 * Export PDF Dialog - Modal for configuring PDF export options
 *
 * Features:
 * - Document metadata (title, author, subject, keywords)
 * - Page range selection
 * - Compression and quality settings
 * - PDF version selection
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import './ExportPdfDialog.css';

interface PdfExportOptions {
  title: string | null;
  author: string | null;
  subject: string | null;
  keywords: string[];
  compress: boolean;
  embedFonts: boolean;
  pdfVersion: string;
  pageRange: { start: number; end: number } | null;
  imageQuality: number;
}

interface ExportPdfDialogProps {
  isOpen: boolean;
  onClose: () => void;
  documentTitle?: string;
  totalPages?: number;
}

const DEFAULT_OPTIONS: PdfExportOptions = {
  title: null,
  author: null,
  subject: null,
  keywords: [],
  compress: true,
  embedFonts: false,
  pdfVersion: 'v14',
  pageRange: null,
  imageQuality: 85,
};

export function ExportPdfDialog({
  isOpen,
  onClose,
  documentTitle = 'Untitled',
  totalPages = 1,
}: ExportPdfDialogProps) {
  const [options, setOptions] = useState<PdfExportOptions>({
    ...DEFAULT_OPTIONS,
    title: documentTitle,
  });
  const [keywordInput, setKeywordInput] = useState('');
  const [pageRangeEnabled, setPageRangeEnabled] = useState(false);
  const [pageStart, setPageStart] = useState(1);
  const [pageEnd, setPageEnd] = useState(totalPages);
  const [exporting, setExporting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Reset options when dialog opens
  useEffect(() => {
    if (isOpen) {
      setOptions({
        ...DEFAULT_OPTIONS,
        title: documentTitle,
      });
      setKeywordInput('');
      setPageRangeEnabled(false);
      setPageStart(1);
      setPageEnd(totalPages);
      setError(null);
    }
  }, [isOpen, documentTitle, totalPages]);

  // Handle escape key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen && !exporting) {
        onClose();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, exporting, onClose]);

  const updateOption = useCallback(
    <K extends keyof PdfExportOptions>(key: K, value: PdfExportOptions[K]) => {
      setOptions((prev) => ({ ...prev, [key]: value }));
    },
    []
  );

  const addKeyword = useCallback(() => {
    const keyword = keywordInput.trim();
    if (keyword && !options.keywords.includes(keyword)) {
      setOptions((prev) => ({
        ...prev,
        keywords: [...prev.keywords, keyword],
      }));
      setKeywordInput('');
    }
  }, [keywordInput, options.keywords]);

  const removeKeyword = useCallback((keyword: string) => {
    setOptions((prev) => ({
      ...prev,
      keywords: prev.keywords.filter((k) => k !== keyword),
    }));
  }, []);

  const handleExport = useCallback(async () => {
    setExporting(true);
    setError(null);

    try {
      // Show save dialog
      const filePath = await save({
        title: 'Export PDF',
        defaultPath: `${options.title || 'document'}.pdf`,
        filters: [{ name: 'PDF Files', extensions: ['pdf'] }],
      });

      if (!filePath) {
        setExporting(false);
        return;
      }

      // Build export options
      const exportOptions = {
        title: options.title,
        author: options.author,
        subject: options.subject,
        keywords: options.keywords,
        compress: options.compress,
        embedFonts: options.embedFonts,
        pdfVersion: options.pdfVersion,
        pageRange: pageRangeEnabled
          ? { start: pageStart - 1, end: pageEnd }
          : null,
        imageQuality: options.imageQuality,
      };

      // Export PDF
      await invoke('export_pdf', {
        docId: 'current', // TODO: Pass actual document ID
        path: filePath,
        options: exportOptions,
      });

      onClose();
    } catch (e) {
      console.error('PDF export failed:', e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setExporting(false);
    }
  }, [options, pageRangeEnabled, pageStart, pageEnd, onClose]);

  if (!isOpen) return null;

  return (
    <div className="export-pdf-overlay" onClick={onClose}>
      <div
        className="export-pdf-dialog"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-labelledby="export-pdf-title"
        aria-modal="true"
      >
        <header className="export-pdf-header">
          <h2 id="export-pdf-title">Export as PDF</h2>
          <button
            className="export-pdf-close-btn"
            onClick={onClose}
            aria-label="Close"
            disabled={exporting}
          >
            x
          </button>
        </header>

        <div className="export-pdf-content">
          {error && (
            <div className="export-pdf-error" role="alert">
              {error}
            </div>
          )}

          {/* Document Metadata Section */}
          <section className="export-pdf-section">
            <h3>Document Properties</h3>

            <div className="export-pdf-field">
              <label htmlFor="pdf-title">Title</label>
              <input
                id="pdf-title"
                type="text"
                value={options.title || ''}
                onChange={(e) => updateOption('title', e.target.value || null)}
                placeholder="Document title"
              />
            </div>

            <div className="export-pdf-field">
              <label htmlFor="pdf-author">Author</label>
              <input
                id="pdf-author"
                type="text"
                value={options.author || ''}
                onChange={(e) => updateOption('author', e.target.value || null)}
                placeholder="Author name"
              />
            </div>

            <div className="export-pdf-field">
              <label htmlFor="pdf-subject">Subject</label>
              <input
                id="pdf-subject"
                type="text"
                value={options.subject || ''}
                onChange={(e) => updateOption('subject', e.target.value || null)}
                placeholder="Document subject"
              />
            </div>

            <div className="export-pdf-field">
              <label htmlFor="pdf-keywords">Keywords</label>
              <div className="export-pdf-keywords-input">
                <input
                  id="pdf-keywords"
                  type="text"
                  value={keywordInput}
                  onChange={(e) => setKeywordInput(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      e.preventDefault();
                      addKeyword();
                    }
                  }}
                  placeholder="Add keyword and press Enter"
                />
                <button
                  type="button"
                  onClick={addKeyword}
                  className="export-pdf-add-btn"
                >
                  Add
                </button>
              </div>
              {options.keywords.length > 0 && (
                <div className="export-pdf-keywords-list">
                  {options.keywords.map((keyword) => (
                    <span key={keyword} className="export-pdf-keyword-tag">
                      {keyword}
                      <button
                        type="button"
                        onClick={() => removeKeyword(keyword)}
                        aria-label={`Remove ${keyword}`}
                      >
                        x
                      </button>
                    </span>
                  ))}
                </div>
              )}
            </div>
          </section>

          {/* Page Range Section */}
          <section className="export-pdf-section">
            <h3>Page Range</h3>

            <div className="export-pdf-toggle-row">
              <label className="export-pdf-checkbox">
                <input
                  type="checkbox"
                  checked={!pageRangeEnabled}
                  onChange={(e) => setPageRangeEnabled(!e.target.checked)}
                />
                <span>All pages ({totalPages})</span>
              </label>
            </div>

            {pageRangeEnabled && (
              <div className="export-pdf-range-inputs">
                <label>
                  From:
                  <input
                    type="number"
                    min={1}
                    max={totalPages}
                    value={pageStart}
                    onChange={(e) =>
                      setPageStart(
                        Math.max(1, Math.min(totalPages, parseInt(e.target.value) || 1))
                      )
                    }
                  />
                </label>
                <label>
                  To:
                  <input
                    type="number"
                    min={1}
                    max={totalPages}
                    value={pageEnd}
                    onChange={(e) =>
                      setPageEnd(
                        Math.max(1, Math.min(totalPages, parseInt(e.target.value) || totalPages))
                      )
                    }
                  />
                </label>
              </div>
            )}
          </section>

          {/* Output Settings Section */}
          <section className="export-pdf-section">
            <h3>Output Settings</h3>

            <div className="export-pdf-field">
              <label htmlFor="pdf-version">PDF Version</label>
              <select
                id="pdf-version"
                value={options.pdfVersion}
                onChange={(e) => updateOption('pdfVersion', e.target.value)}
              >
                <option value="v14">PDF 1.4 (Acrobat 5)</option>
                <option value="v15">PDF 1.5 (Acrobat 6)</option>
                <option value="v17">PDF 1.7 (Acrobat 8)</option>
              </select>
            </div>

            <div className="export-pdf-toggle-row">
              <label className="export-pdf-checkbox">
                <input
                  type="checkbox"
                  checked={options.compress}
                  onChange={(e) => updateOption('compress', e.target.checked)}
                />
                <span>Compress content (smaller file size)</span>
              </label>
            </div>

            <div className="export-pdf-field">
              <label htmlFor="pdf-image-quality">
                Image Quality: {options.imageQuality}%
              </label>
              <input
                id="pdf-image-quality"
                type="range"
                min={10}
                max={100}
                step={5}
                value={options.imageQuality}
                onChange={(e) =>
                  updateOption('imageQuality', parseInt(e.target.value))
                }
              />
              <div className="export-pdf-range-labels">
                <span>Smaller</span>
                <span>Higher quality</span>
              </div>
            </div>
          </section>
        </div>

        <footer className="export-pdf-footer">
          <button
            className="export-pdf-btn export-pdf-btn-secondary"
            onClick={onClose}
            disabled={exporting}
          >
            Cancel
          </button>
          <button
            className="export-pdf-btn export-pdf-btn-primary"
            onClick={handleExport}
            disabled={exporting}
          >
            {exporting ? 'Exporting...' : 'Export'}
          </button>
        </footer>
      </div>
    </div>
  );
}
