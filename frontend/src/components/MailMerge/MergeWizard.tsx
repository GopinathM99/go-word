/**
 * MergeWizard - Step-by-step wizard for executing mail merge
 */

import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { DataSourcePicker } from './DataSourcePicker';
import { ColumnMapping } from './ColumnMapping';
import { DataPreview } from './DataPreview';
import './MailMerge.css';
import type { DataSourceInfo } from './DataSourcePicker';
import type { FieldMapping } from './ColumnMapping';

type WizardStep = 'source' | 'fields' | 'options' | 'preview';
type MergeOutputType = 'single_document' | 'individual_documents' | 'preview';

interface MergeOptions { outputType: MergeOutputType; pageBreakBetweenRecords: boolean; trimValues: boolean; removeEmptyParagraphs: boolean; maxRecords: number; outputNamePattern: string; outputDirectory: string; }
interface MergeProgress { currentRecord: number; totalRecords: number; percent: number; }
interface MergeResultDto { status: string; totalRecords: number; processedCount: number; skippedCount: number; errorCount: number; summary: string; outputPaths: string[]; }

export interface MergeWizardProps { isOpen: boolean; onClose: () => void; onMergeComplete?: (result: MergeResultDto) => void; }

const steps: Array<{ id: WizardStep; label: string }> = [
  { id: 'source', label: '1. Data Source' }, { id: 'fields', label: '2. Fields' },
  { id: 'options', label: '3. Options' }, { id: 'preview', label: '4. Preview & Merge' },
];

const defaultOptions: MergeOptions = { outputType: 'single_document', pageBreakBetweenRecords: true, trimValues: true, removeEmptyParagraphs: false, maxRecords: 0, outputNamePattern: 'merged_{index}.docx', outputDirectory: '' };

export function MergeWizard({ isOpen, onClose, onMergeComplete }: MergeWizardProps) {
  const [currentStep, setCurrentStep] = useState<WizardStep>('source');
  const [dataSource, setDataSource] = useState<DataSourceInfo | null>(null);
  const [mappings, setMappings] = useState<FieldMapping[]>([]);
  const [options, setOptions] = useState<MergeOptions>(defaultOptions);
  const [mergeResult, setMergeResult] = useState<MergeResultDto | null>(null);
  const [merging, setMerging] = useState(false);
  const [progress, setProgress] = useState<MergeProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleDataSourceLoaded = useCallback((ds: DataSourceInfo) => { setDataSource(ds); setError(null); }, []);
  const goToStep = useCallback((step: WizardStep) => { setCurrentStep(step); setError(null); }, []);
  const canAdvance = useCallback((): boolean => { switch(currentStep) { case 'source': return dataSource !== null; case 'fields': return mappings.length > 0; case 'options': return true; case 'preview': return false; } }, [currentStep, dataSource, mappings]);
  const nextStep = useCallback(() => { const i = steps.findIndex((s) => s.id === currentStep); if (i < steps.length - 1) goToStep(steps[i + 1].id); }, [currentStep, goToStep]);
  const prevStep = useCallback(() => { const i = steps.findIndex((s) => s.id === currentStep); if (i > 0) goToStep(steps[i - 1].id); }, [currentStep, goToStep]);

  const handleExecuteMerge = useCallback(async () => {
    if (!dataSource) return;
    setMerging(true); setError(null); setMergeResult(null);
    try {
      const result = await invoke<MergeResultDto>('execute_mail_merge', { dataSourceId: dataSource.id, mappings, options });
      setMergeResult(result); onMergeComplete?.(result);
    } catch (e) { setError(e instanceof Error ? e.message : String(e)); }
    finally { setMerging(false); setProgress(null); }
  }, [dataSource, mappings, options, onMergeComplete]);

  const handlePreviewMerge = useCallback(async () => {
    if (!dataSource) return;
    setMerging(true); setError(null);
    try {
      const result = await invoke<MergeResultDto>('preview_mail_merge', { dataSourceId: dataSource.id, mappings, count: 5 });
      setMergeResult(result);
    } catch (e) { setError(e instanceof Error ? e.message : String(e)); }
    finally { setMerging(false); }
  }, [dataSource, mappings]);

  if (!isOpen) return null;

  return (
    <div className="mail-merge-wizard-overlay" onClick={onClose}>
      <div className="mail-merge-wizard" onClick={(e) => e.stopPropagation()}>
        <div className="mail-merge-wizard-header">
          <h2 className="mail-merge-wizard-title">Mail Merge</h2>
          <button type="button" className="mail-merge-wizard-close" onClick={onClose}>&times;</button>
        </div>
        <div className="mail-merge-wizard-steps">
          {steps.map((step, idx) => (
            <button key={step.id} type="button"
              className={`mail-merge-step ${currentStep === step.id ? 'active' : ''} ${steps.findIndex((s) => s.id === currentStep) > idx ? 'completed' : ''}`}
              onClick={() => goToStep(step.id)}>{step.label}</button>
          ))}
        </div>
        <div className="mail-merge-wizard-body">
          {currentStep === 'source' && <DataSourcePicker onDataSourceLoaded={handleDataSourceLoaded} onError={(e) => setError(e)} />}
          {currentStep === 'fields' && dataSource && (
            <>
              <ColumnMapping dataSourceId={dataSource.id} mappings={mappings} onMappingChange={setMappings} />
              <div style={{ marginTop: '16px' }}><DataPreview dataSourceId={dataSource.id} pageSize={5} /></div>
            </>
          )}
          {currentStep === 'options' && (
            <div className="mail-merge-options">
              <h3 className="mail-merge-section-title">Output Options</h3>
              <div className="mail-merge-picker-section">
                <label className="mail-merge-label">Output Type</label>
                <div className="mail-merge-radio-group" style={{ flexDirection: 'column', gap: '8px' }}>
                  <label className="mail-merge-radio">
                    <input type="radio" name="outputType" checked={options.outputType === 'single_document'}
                      onChange={() => setOptions({ ...options, outputType: 'single_document' })} />
                    <span>Single document (all records merged into one file)</span>
                  </label>
                  <label className="mail-merge-radio">
                    <input type="radio" name="outputType" checked={options.outputType === 'individual_documents'}
                      onChange={() => setOptions({ ...options, outputType: 'individual_documents' })} />
                    <span>Individual documents (one file per record)</span>
                  </label>
                </div>
              </div>
              {options.outputType === 'single_document' && (
                <div className="mail-merge-picker-section">
                  <label className="mail-merge-checkbox">
                    <input type="checkbox" checked={options.pageBreakBetweenRecords}
                      onChange={(e) => setOptions({ ...options, pageBreakBetweenRecords: e.target.checked })} />
                    <span>Page break between records</span>
                  </label>
                </div>
              )}
              {options.outputType === 'individual_documents' && (
                <div className="mail-merge-picker-section">
                  <label className="mail-merge-label">File Name Pattern</label>
                  <input type="text" className="mail-merge-input" value={options.outputNamePattern}
                    onChange={(e) => setOptions({ ...options, outputNamePattern: e.target.value })}
                    placeholder="e.g., Letter_{last_name}.docx" />
                  <p className="mail-merge-hint">Use {'{field_name}'} to include field values. Use {'{index}'} for record number.</p>
                </div>
              )}
              <div className="mail-merge-picker-section">
                <label className="mail-merge-checkbox">
                  <input type="checkbox" checked={options.trimValues} onChange={(e) => setOptions({ ...options, trimValues: e.target.checked })} />
                  <span>Trim whitespace from field values</span>
                </label>
              </div>
              <div className="mail-merge-picker-section">
                <label className="mail-merge-checkbox">
                  <input type="checkbox" checked={options.removeEmptyParagraphs} onChange={(e) => setOptions({ ...options, removeEmptyParagraphs: e.target.checked })} />
                  <span>Remove paragraphs that are empty after merge</span>
                </label>
              </div>
              <div className="mail-merge-picker-section">
                <label className="mail-merge-label">Max Records (0 = all)</label>
                <input type="number" className="mail-merge-input" value={options.maxRecords}
                  onChange={(e) => setOptions({ ...options, maxRecords: parseInt(e.target.value) || 0 })} min={0} style={{ width: '120px' }} />
              </div>
            </div>
          )}
          {currentStep === 'preview' && (
            <div className="mail-merge-preview-step">
              <div className="mail-merge-preview-actions">
                <button type="button" className="mail-merge-btn mail-merge-btn-secondary" onClick={handlePreviewMerge} disabled={merging}>
                  {merging ? 'Generating...' : 'Preview Results'}
                </button>
              </div>
              {mergeResult && (
                <div className="mail-merge-result">
                  <h4 className="mail-merge-section-title">{mergeResult.status === 'completed' ? 'Merge Results' : 'Preview Results'}</h4>
                  <div className="mail-merge-result-stats">
                    <div className="mail-merge-stat"><span className="mail-merge-stat-label">Total Records</span><span className="mail-merge-stat-value">{mergeResult.totalRecords}</span></div>
                    <div className="mail-merge-stat"><span className="mail-merge-stat-label">Processed</span><span className="mail-merge-stat-value">{mergeResult.processedCount}</span></div>
                    <div className="mail-merge-stat"><span className="mail-merge-stat-label">Skipped</span><span className="mail-merge-stat-value">{mergeResult.skippedCount}</span></div>
                    <div className="mail-merge-stat"><span className="mail-merge-stat-label">Errors</span><span className="mail-merge-stat-value">{mergeResult.errorCount}</span></div>
                  </div>
                  <p className="mail-merge-result-summary">{mergeResult.summary}</p>
                </div>
              )}
              {progress && (
                <div className="mail-merge-progress">
                  <div className="mail-merge-progress-bar"><div className="mail-merge-progress-fill" style={{ width: `${progress.percent}%` }} /></div>
                  <span className="mail-merge-progress-text">Record {progress.currentRecord} of {progress.totalRecords} ({Math.round(progress.percent)}%)</span>
                </div>
              )}
            </div>
          )}
          {error && <div className="mail-merge-error">{error}</div>}
        </div>
        <div className="mail-merge-wizard-footer">
          <div className="mail-merge-wizard-footer-left">
            {dataSource && <span className="mail-merge-source-info">Source: {dataSource.sourceType.toUpperCase()} &middot; {dataSource.recordCount} records &middot; {dataSource.columnCount} columns</span>}
          </div>
          <div className="mail-merge-wizard-footer-right">
            {currentStep !== 'source' && <button type="button" className="mail-merge-btn mail-merge-btn-secondary" onClick={prevStep} disabled={merging}>Back</button>}
            {currentStep === 'preview' ? (
              <button type="button" className="mail-merge-btn mail-merge-btn-primary"
                onClick={handleExecuteMerge} disabled={merging || !dataSource || mappings.length === 0}>
                {merging ? 'Merging...' : 'Execute Merge'}
              </button>
            ) : (
              <button type="button" className="mail-merge-btn mail-merge-btn-primary" onClick={nextStep} disabled={!canAdvance()}>Next</button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
