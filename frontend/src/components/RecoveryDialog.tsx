/**
 * Recovery Dialog - Shown on startup when recovery files exist
 *
 * This dialog allows users to:
 * - View available recovery files from a previous crash
 * - Recover a document from a recovery file
 * - Discard recovery files
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  RecoveryFile,
  formatFileSize,
  formatTimestamp,
} from '../lib/types';
import './RecoveryDialog.css';

interface RecoveryDialogProps {
  /** Whether the dialog is open */
  isOpen: boolean;
  /** Callback when the dialog is closed */
  onClose: () => void;
  /** Callback when a document is recovered */
  onRecover?: (documentId: string) => void;
}

export function RecoveryDialog({
  isOpen,
  onClose,
  onRecover,
}: RecoveryDialogProps) {
  const [recoveryFiles, setRecoveryFiles] = useState<RecoveryFile[]>([]);
  const [loading, setLoading] = useState(true);
  const [recovering, setRecovering] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Load recovery files on mount
  useEffect(() => {
    if (isOpen) {
      loadRecoveryFiles();
    }
  }, [isOpen]);

  const loadRecoveryFiles = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const files = await invoke<RecoveryFile[]>('get_recovery_files');
      setRecoveryFiles(files);
    } catch (e) {
      setError(`Failed to load recovery files: ${e}`);
    } finally {
      setLoading(false);
    }
  }, []);

  const handleRecover = useCallback(
    async (recoveryId: string) => {
      setRecovering(recoveryId);
      setError(null);

      try {
        const documentId = await invoke<string>('recover_document', {
          recoveryId,
        });

        // Discard the recovery file after successful recovery
        await invoke('discard_recovery', { recoveryId });

        if (onRecover) {
          onRecover(documentId);
        }

        onClose();
      } catch (e) {
        setError(`Failed to recover document: ${e}`);
      } finally {
        setRecovering(null);
      }
    },
    [onRecover, onClose]
  );

  const handleDiscard = useCallback(
    async (recoveryId: string) => {
      try {
        await invoke('discard_recovery', { recoveryId });
        setRecoveryFiles((prev) => prev.filter((f) => f.id !== recoveryId));
      } catch (e) {
        setError(`Failed to discard recovery file: ${e}`);
      }
    },
    []
  );

  const handleDiscardAll = useCallback(async () => {
    if (!confirm('Are you sure you want to discard all recovery files? This cannot be undone.')) {
      return;
    }

    try {
      await invoke('discard_all_recovery');
      setRecoveryFiles([]);
      onClose();
    } catch (e) {
      setError(`Failed to discard recovery files: ${e}`);
    }
  }, [onClose]);

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

  if (!isOpen) return null;

  return (
    <div className="recovery-overlay" onClick={onClose}>
      <div
        className="recovery-dialog"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-labelledby="recovery-title"
        aria-modal="true"
      >
        <header className="recovery-header">
          <div className="recovery-icon">!</div>
          <div className="recovery-header-text">
            <h2 id="recovery-title">Document Recovery</h2>
            <p className="recovery-subtitle">
              The application was not closed properly. Would you like to recover your unsaved work?
            </p>
          </div>
        </header>

        <div className="recovery-body">
          {loading && (
            <div className="recovery-loading">
              <div className="recovery-spinner"></div>
              <p>Loading recovery files...</p>
            </div>
          )}

          {error && (
            <div className="recovery-error">
              <p>{error}</p>
              <button onClick={loadRecoveryFiles}>Retry</button>
            </div>
          )}

          {!loading && !error && recoveryFiles.length === 0 && (
            <div className="recovery-empty">
              <p>No recovery files found.</p>
            </div>
          )}

          {!loading && !error && recoveryFiles.length > 0 && (
            <div className="recovery-list">
              {recoveryFiles.map((file) => (
                <div key={file.id} className="recovery-item">
                  <div className="recovery-item-info">
                    <div className="recovery-item-name">
                      {file.originalPath
                        ? file.originalPath.split('/').pop() || 'Untitled Document'
                        : 'Untitled Document'}
                    </div>
                    <div className="recovery-item-details">
                      <span className="recovery-item-time">
                        {file.timeDescription}
                      </span>
                      <span className="recovery-item-size">
                        {formatFileSize(file.fileSize)}
                      </span>
                      {file.originalPath && (
                        <span
                          className="recovery-item-path"
                          title={file.originalPath}
                        >
                          {file.originalPath}
                        </span>
                      )}
                    </div>
                  </div>
                  <div className="recovery-item-actions">
                    <button
                      className="recovery-btn recovery-btn-primary"
                      onClick={() => handleRecover(file.id)}
                      disabled={recovering !== null}
                    >
                      {recovering === file.id ? 'Recovering...' : 'Recover'}
                    </button>
                    <button
                      className="recovery-btn recovery-btn-secondary"
                      onClick={() => handleDiscard(file.id)}
                      disabled={recovering !== null}
                    >
                      Discard
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        <footer className="recovery-footer">
          <button
            className="recovery-btn recovery-btn-danger"
            onClick={handleDiscardAll}
            disabled={loading || recoveryFiles.length === 0 || recovering !== null}
          >
            Discard All
          </button>
          <button
            className="recovery-btn recovery-btn-secondary"
            onClick={onClose}
            disabled={recovering !== null}
          >
            Close
          </button>
        </footer>
      </div>
    </div>
  );
}
