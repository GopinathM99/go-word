/**
 * DataPreview - Component for previewing mail merge data
 *
 * Features:
 * - Display first N rows in a table
 * - Show column names and data types
 * - Pagination through records
 * - Record count display
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './MailMerge.css';

export interface ColumnDef {
  name: string;
  dataType: string;
  displayName?: string;
  description?: string;
}

export interface ValueDto {
  valueType: string;
  value: string;
}

export interface RecordDto {
  data: Record<string, ValueDto>;
}

export interface DataPreviewDto {
  columns: ColumnDef[];
  records: RecordDto[];
  totalRecords: number;
}

interface DataPreviewProps {
  dataSourceId: string;
  pageSize?: number;
  onColumnSelect?: (column: ColumnDef) => void;
}

export function DataPreview({
  dataSourceId,
  pageSize = 10,
  onColumnSelect,
}: DataPreviewProps) {
  const [preview, setPreview] = useState<DataPreviewDto | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState(0);

  const loadPreview = useCallback(async () => {
    if (!dataSourceId) return;

    setLoading(true);
    setError(null);

    try {
      const result = await invoke<DataPreviewDto>('get_data_source_preview', {
        id: dataSourceId,
        limit: pageSize,
      });
      setPreview(result);
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      setError(errorMsg);
    } finally {
      setLoading(false);
    }
  }, [dataSourceId, pageSize]);

  useEffect(() => {
    loadPreview();
  }, [loadPreview]);

  const loadPage = useCallback(
    async (page: number) => {
      if (!dataSourceId) return;

      setLoading(true);
      setError(null);

      try {
        const offset = page * pageSize;
        const records = await invoke<RecordDto[]>('get_data_source_records', {
          id: dataSourceId,
          offset,
          limit: pageSize,
        });

        if (preview) {
          setPreview({
            ...preview,
            records,
          });
        }
        setCurrentPage(page);
      } catch (e) {
        const errorMsg = e instanceof Error ? e.message : String(e);
        setError(errorMsg);
      } finally {
        setLoading(false);
      }
    },
    [dataSourceId, pageSize, preview]
  );

  const totalPages = preview
    ? Math.ceil(preview.totalRecords / pageSize)
    : 0;

  const formatValue = (val: ValueDto): string => {
    if (val.valueType === 'null') return '-';
    return val.value;
  };

  const getTypeIcon = (dataType: string): string => {
    switch (dataType) {
      case 'text':
        return 'Aa';
      case 'number':
        return '#';
      case 'date':
        return 'D';
      case 'boolean':
        return 'T/F';
      default:
        return '?';
    }
  };

  if (loading && !preview) {
    return (
      <div className="mail-merge-preview">
        <div className="mail-merge-loading">Loading preview...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="mail-merge-preview">
        <div className="mail-merge-error">{error}</div>
      </div>
    );
  }

  if (!preview) {
    return (
      <div className="mail-merge-preview">
        <div className="mail-merge-empty">No data source loaded</div>
      </div>
    );
  }

  return (
    <div className="mail-merge-preview">
      <div className="mail-merge-preview-header">
        <h3 className="mail-merge-preview-title">Data Preview</h3>
        <div className="mail-merge-preview-stats">
          <span>
            {preview.columns.length} columns, {preview.totalRecords} records
          </span>
        </div>
      </div>

      <div className="mail-merge-table-container">
        <table className="mail-merge-table">
          <thead>
            <tr>
              <th className="mail-merge-row-number">#</th>
              {preview.columns.map((col) => (
                <th
                  key={col.name}
                  className="mail-merge-header-cell"
                  onClick={() => onColumnSelect?.(col)}
                  title={col.description || `Type: ${col.dataType}`}
                >
                  <div className="mail-merge-header-content">
                    <span className="mail-merge-header-type">
                      {getTypeIcon(col.dataType)}
                    </span>
                    <span className="mail-merge-header-name">
                      {col.displayName || col.name}
                    </span>
                  </div>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {preview.records.map((record, rowIndex) => (
              <tr key={rowIndex}>
                <td className="mail-merge-row-number">
                  {currentPage * pageSize + rowIndex + 1}
                </td>
                {preview.columns.map((col) => {
                  const value = record.data[col.name];
                  return (
                    <td
                      key={col.name}
                      className={`mail-merge-cell mail-merge-cell-${value?.valueType || 'null'}`}
                    >
                      {value ? formatValue(value) : '-'}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {totalPages > 1 && (
        <div className="mail-merge-pagination">
          <button
            className="mail-merge-btn mail-merge-btn-small"
            onClick={() => loadPage(0)}
            disabled={currentPage === 0 || loading}
          >
            First
          </button>
          <button
            className="mail-merge-btn mail-merge-btn-small"
            onClick={() => loadPage(currentPage - 1)}
            disabled={currentPage === 0 || loading}
          >
            Previous
          </button>
          <span className="mail-merge-page-info">
            Page {currentPage + 1} of {totalPages}
          </span>
          <button
            className="mail-merge-btn mail-merge-btn-small"
            onClick={() => loadPage(currentPage + 1)}
            disabled={currentPage >= totalPages - 1 || loading}
          >
            Next
          </button>
          <button
            className="mail-merge-btn mail-merge-btn-small"
            onClick={() => loadPage(totalPages - 1)}
            disabled={currentPage >= totalPages - 1 || loading}
          >
            Last
          </button>
        </div>
      )}

      {loading && (
        <div className="mail-merge-loading-overlay">Loading...</div>
      )}
    </div>
  );
}
