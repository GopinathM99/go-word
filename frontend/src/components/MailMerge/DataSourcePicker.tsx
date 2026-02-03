/**
 * DataSourcePicker - Component for selecting and loading mail merge data sources
 *
 * Features:
 * - File type selection (CSV, JSON)
 * - File path input or file browser
 * - CSV delimiter configuration
 * - JSON root path configuration
 * - Auto-detection of CSV settings
 */

import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import './MailMerge.css';

export interface DataSourceInfo {
  id: string;
  sourceType: string;
  columnCount: number;
  recordCount: number;
  columnNames: string[];
}

interface DataSourcePickerProps {
  onDataSourceLoaded: (dataSource: DataSourceInfo) => void;
  onError?: (error: string) => void;
}

type SourceType = 'csv' | 'json';

export function DataSourcePicker({
  onDataSourceLoaded,
  onError,
}: DataSourcePickerProps) {
  const [sourceType, setSourceType] = useState<SourceType>('csv');
  const [filePath, setFilePath] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // CSV-specific options
  const [delimiter, setDelimiter] = useState<string>(',');
  const [hasHeader, setHasHeader] = useState(true);

  // JSON-specific options
  const [rootPath, setRootPath] = useState('');

  const handleBrowse = useCallback(async () => {
    try {
      const extensions = sourceType === 'csv' ? ['csv', 'tsv', 'txt'] : ['json'];
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: sourceType === 'csv' ? 'CSV Files' : 'JSON Files',
            extensions,
          },
          { name: 'All Files', extensions: ['*'] },
        ],
      });

      if (selected && typeof selected === 'string') {
        setFilePath(selected);
        setError(null);

        // Auto-detect settings for CSV
        if (sourceType === 'csv') {
          try {
            // Read file content for auto-detection
            const content = await invoke<string>('read_text_file', {
              path: selected,
            }).catch(() => null);

            if (content) {
              const detectedDelimiter = await invoke<string>(
                'detect_csv_delimiter',
                { content }
              );
              const detectedHasHeader = await invoke<boolean>(
                'detect_csv_has_header',
                { content, delimiter: detectedDelimiter }
              );

              setDelimiter(detectedDelimiter);
              setHasHeader(detectedHasHeader);
            }
          } catch {
            // Ignore auto-detection errors
          }
        }
      }
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      setError(errorMsg);
      onError?.(errorMsg);
    }
  }, [sourceType, onError]);

  const handleLoad = useCallback(async () => {
    if (!filePath) {
      setError('Please select a file');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      let result: DataSourceInfo;

      if (sourceType === 'csv') {
        result = await invoke<DataSourceInfo>('load_csv_data_source', {
          path: filePath,
          delimiter: delimiter.charAt(0),
          hasHeader,
        });
      } else {
        result = await invoke<DataSourceInfo>('load_json_data_source', {
          path: filePath,
          rootPath: rootPath || null,
        });
      }

      onDataSourceLoaded(result);
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      setError(errorMsg);
      onError?.(errorMsg);
    } finally {
      setLoading(false);
    }
  }, [
    filePath,
    sourceType,
    delimiter,
    hasHeader,
    rootPath,
    onDataSourceLoaded,
    onError,
  ]);

  const delimiterOptions = [
    { value: ',', label: 'Comma (,)' },
    { value: ';', label: 'Semicolon (;)' },
    { value: '\t', label: 'Tab' },
    { value: '|', label: 'Pipe (|)' },
  ];

  return (
    <div className="mail-merge-picker">
      <h3 className="mail-merge-picker-title">Select Data Source</h3>

      <div className="mail-merge-picker-section">
        <label className="mail-merge-label">Source Type</label>
        <div className="mail-merge-radio-group">
          <label className="mail-merge-radio">
            <input
              type="radio"
              name="sourceType"
              value="csv"
              checked={sourceType === 'csv'}
              onChange={() => setSourceType('csv')}
            />
            <span>CSV / TSV</span>
          </label>
          <label className="mail-merge-radio">
            <input
              type="radio"
              name="sourceType"
              value="json"
              checked={sourceType === 'json'}
              onChange={() => setSourceType('json')}
            />
            <span>JSON</span>
          </label>
        </div>
      </div>

      <div className="mail-merge-picker-section">
        <label className="mail-merge-label" htmlFor="filePath">
          File Path
        </label>
        <div className="mail-merge-file-input">
          <input
            id="filePath"
            type="text"
            className="mail-merge-input"
            value={filePath}
            onChange={(e) => setFilePath(e.target.value)}
            placeholder="Select a file..."
            readOnly
          />
          <button
            type="button"
            className="mail-merge-btn mail-merge-btn-secondary"
            onClick={handleBrowse}
          >
            Browse...
          </button>
        </div>
      </div>

      {sourceType === 'csv' && (
        <>
          <div className="mail-merge-picker-section">
            <label className="mail-merge-label" htmlFor="delimiter">
              Delimiter
            </label>
            <select
              id="delimiter"
              className="mail-merge-select"
              value={delimiter}
              onChange={(e) => setDelimiter(e.target.value)}
            >
              {delimiterOptions.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>

          <div className="mail-merge-picker-section">
            <label className="mail-merge-checkbox">
              <input
                type="checkbox"
                checked={hasHeader}
                onChange={(e) => setHasHeader(e.target.checked)}
              />
              <span>First row contains headers</span>
            </label>
          </div>
        </>
      )}

      {sourceType === 'json' && (
        <div className="mail-merge-picker-section">
          <label className="mail-merge-label" htmlFor="rootPath">
            Root Path (optional)
          </label>
          <input
            id="rootPath"
            type="text"
            className="mail-merge-input"
            value={rootPath}
            onChange={(e) => setRootPath(e.target.value)}
            placeholder="e.g., data.customers"
          />
          <p className="mail-merge-hint">
            Use dot notation to specify the path to the data array within the
            JSON structure.
          </p>
        </div>
      )}

      {error && <div className="mail-merge-error">{error}</div>}

      <div className="mail-merge-picker-actions">
        <button
          type="button"
          className="mail-merge-btn mail-merge-btn-primary"
          onClick={handleLoad}
          disabled={!filePath || loading}
        >
          {loading ? 'Loading...' : 'Load Data Source'}
        </button>
      </div>
    </div>
  );
}
