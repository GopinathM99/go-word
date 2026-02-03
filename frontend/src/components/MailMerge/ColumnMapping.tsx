/**
 * ColumnMapping - Component for mapping data source columns to merge fields
 *
 * Features:
 * - Display available columns from data source
 * - Define merge field mappings
 * - Preview field values
 * - Insert merge field into document
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

export interface FieldMapping {
  columnName: string;
  fieldName: string;
  format?: string;
}

interface ColumnMappingProps {
  dataSourceId: string;
  mappings: FieldMapping[];
  onMappingChange: (mappings: FieldMapping[]) => void;
  onInsertField?: (fieldName: string, columnName: string) => void;
}

export function ColumnMapping({
  dataSourceId,
  mappings,
  onMappingChange,
  onInsertField,
}: ColumnMappingProps) {
  const [columns, setColumns] = useState<ColumnDef[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedColumn, setSelectedColumn] = useState<ColumnDef | null>(null);
  const [newFieldName, setNewFieldName] = useState('');
  const [previewValue, setPreviewValue] = useState<string | null>(null);
  const [previewIndex, setPreviewIndex] = useState(0);

  const loadColumns = useCallback(async () => {
    if (!dataSourceId) return;

    setLoading(true);
    setError(null);

    try {
      const result = await invoke<ColumnDef[]>('get_data_source_columns', {
        id: dataSourceId,
      });
      setColumns(result);
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      setError(errorMsg);
    } finally {
      setLoading(false);
    }
  }, [dataSourceId]);

  useEffect(() => {
    loadColumns();
  }, [loadColumns]);

  const loadPreview = useCallback(
    async (columnName: string, recordIndex: number) => {
      if (!dataSourceId || !columnName) return;

      try {
        const result = await invoke<{ valueType: string; value: string }>(
          'get_data_source_value',
          {
            id: dataSourceId,
            recordIndex,
            columnName,
          }
        );
        setPreviewValue(result.value);
      } catch {
        setPreviewValue(null);
      }
    },
    [dataSourceId]
  );

  const handleColumnSelect = useCallback(
    (col: ColumnDef) => {
      setSelectedColumn(col);
      setNewFieldName(col.displayName || col.name);
      loadPreview(col.name, previewIndex);
    },
    [loadPreview, previewIndex]
  );

  const handleAddMapping = useCallback(() => {
    if (!selectedColumn || !newFieldName.trim()) return;

    const existing = mappings.find(
      (m) => m.columnName === selectedColumn.name
    );
    if (existing) {
      // Update existing mapping
      onMappingChange(
        mappings.map((m) =>
          m.columnName === selectedColumn.name
            ? { ...m, fieldName: newFieldName.trim() }
            : m
        )
      );
    } else {
      // Add new mapping
      onMappingChange([
        ...mappings,
        {
          columnName: selectedColumn.name,
          fieldName: newFieldName.trim(),
        },
      ]);
    }

    setSelectedColumn(null);
    setNewFieldName('');
    setPreviewValue(null);
  }, [selectedColumn, newFieldName, mappings, onMappingChange]);

  const handleRemoveMapping = useCallback(
    (columnName: string) => {
      onMappingChange(mappings.filter((m) => m.columnName !== columnName));
    },
    [mappings, onMappingChange]
  );

  const handleInsertField = useCallback(
    (mapping: FieldMapping) => {
      onInsertField?.(mapping.fieldName, mapping.columnName);
    },
    [onInsertField]
  );

  const handlePreviewChange = useCallback(
    (delta: number) => {
      const newIndex = previewIndex + delta;
      if (newIndex >= 0) {
        setPreviewIndex(newIndex);
        if (selectedColumn) {
          loadPreview(selectedColumn.name, newIndex);
        }
      }
    },
    [previewIndex, selectedColumn, loadPreview]
  );

  const getTypeColor = (dataType: string): string => {
    switch (dataType) {
      case 'text':
        return '#4a90d9';
      case 'number':
        return '#5cb85c';
      case 'date':
        return '#f0ad4e';
      case 'boolean':
        return '#9b59b6';
      default:
        return '#777';
    }
  };

  if (loading && columns.length === 0) {
    return (
      <div className="mail-merge-mapping">
        <div className="mail-merge-loading">Loading columns...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="mail-merge-mapping">
        <div className="mail-merge-error">{error}</div>
      </div>
    );
  }

  return (
    <div className="mail-merge-mapping">
      <div className="mail-merge-mapping-columns">
        <h4 className="mail-merge-section-title">Available Columns</h4>
        <div className="mail-merge-column-list">
          {columns.map((col) => {
            const isMapped = mappings.some((m) => m.columnName === col.name);
            return (
              <button
                key={col.name}
                className={`mail-merge-column-item ${
                  selectedColumn?.name === col.name ? 'selected' : ''
                } ${isMapped ? 'mapped' : ''}`}
                onClick={() => handleColumnSelect(col)}
                title={col.description || `Type: ${col.dataType}`}
              >
                <span
                  className="mail-merge-column-type"
                  style={{ backgroundColor: getTypeColor(col.dataType) }}
                >
                  {col.dataType.charAt(0).toUpperCase()}
                </span>
                <span className="mail-merge-column-name">
                  {col.displayName || col.name}
                </span>
                {isMapped && <span className="mail-merge-mapped-badge">M</span>}
              </button>
            );
          })}
        </div>
      </div>

      {selectedColumn && (
        <div className="mail-merge-mapping-form">
          <h4 className="mail-merge-section-title">Create Field Mapping</h4>

          <div className="mail-merge-form-group">
            <label className="mail-merge-label">Source Column</label>
            <div className="mail-merge-value">
              {selectedColumn.displayName || selectedColumn.name}
              <span className="mail-merge-type-badge">
                {selectedColumn.dataType}
              </span>
            </div>
          </div>

          <div className="mail-merge-form-group">
            <label className="mail-merge-label" htmlFor="fieldName">
              Merge Field Name
            </label>
            <input
              id="fieldName"
              type="text"
              className="mail-merge-input"
              value={newFieldName}
              onChange={(e) => setNewFieldName(e.target.value)}
              placeholder="Enter field name..."
            />
          </div>

          <div className="mail-merge-form-group">
            <label className="mail-merge-label">Preview Value</label>
            <div className="mail-merge-preview-controls">
              <button
                className="mail-merge-btn mail-merge-btn-small"
                onClick={() => handlePreviewChange(-1)}
                disabled={previewIndex === 0}
              >
                Prev
              </button>
              <span className="mail-merge-preview-value">
                {previewValue ?? '-'}
              </span>
              <button
                className="mail-merge-btn mail-merge-btn-small"
                onClick={() => handlePreviewChange(1)}
              >
                Next
              </button>
            </div>
            <div className="mail-merge-preview-index">
              Record #{previewIndex + 1}
            </div>
          </div>

          <div className="mail-merge-form-actions">
            <button
              className="mail-merge-btn mail-merge-btn-primary"
              onClick={handleAddMapping}
              disabled={!newFieldName.trim()}
            >
              Add Mapping
            </button>
            <button
              className="mail-merge-btn mail-merge-btn-secondary"
              onClick={() => {
                setSelectedColumn(null);
                setNewFieldName('');
                setPreviewValue(null);
              }}
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {mappings.length > 0 && (
        <div className="mail-merge-mapping-list">
          <h4 className="mail-merge-section-title">Current Mappings</h4>
          <table className="mail-merge-mapping-table">
            <thead>
              <tr>
                <th>Column</th>
                <th>Field Name</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {mappings.map((mapping) => (
                <tr key={mapping.columnName}>
                  <td>{mapping.columnName}</td>
                  <td>
                    <code>{'<<' + mapping.fieldName + '>>'}</code>
                  </td>
                  <td>
                    <button
                      className="mail-merge-btn mail-merge-btn-small"
                      onClick={() => handleInsertField(mapping)}
                      title="Insert field into document"
                    >
                      Insert
                    </button>
                    <button
                      className="mail-merge-btn mail-merge-btn-small mail-merge-btn-danger"
                      onClick={() => handleRemoveMapping(mapping.columnName)}
                      title="Remove mapping"
                    >
                      Remove
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
