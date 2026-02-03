/**
 * AddCommentDialog - Popup dialog for adding new comments
 *
 * Features:
 * - Text input for comment content
 * - Shows selected text excerpt
 * - Submit/Cancel buttons
 * - Keyboard shortcuts (Ctrl+Enter to submit, Escape to cancel)
 * - Focus management and accessibility
 */

import React, { useState, useCallback, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { TextSelection, truncateText } from '../hooks/useComments';
import './Comments.css';

// =============================================================================
// Types
// =============================================================================

interface AddCommentDialogProps {
  /** Whether the dialog is open */
  isOpen: boolean;
  /** Callback to close the dialog */
  onClose: () => void;
  /** Callback to submit the comment */
  onSubmit: (content: string) => Promise<boolean>;
  /** The selected text to comment on */
  selection: TextSelection | null;
  /** Loading state while submitting */
  isSubmitting?: boolean;
  /** Error message if submission failed */
  error?: string | null;
}

// =============================================================================
// Component
// =============================================================================

export function AddCommentDialog({
  isOpen,
  onClose,
  onSubmit,
  selection,
  isSubmitting = false,
  error = null,
}: AddCommentDialogProps) {
  const [content, setContent] = useState('');
  const [localError, setLocalError] = useState<string | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const previousActiveElement = useRef<HTMLElement | null>(null);

  // Reset state when dialog opens/closes
  useEffect(() => {
    if (isOpen) {
      setContent('');
      setLocalError(null);
      previousActiveElement.current = document.activeElement as HTMLElement;

      // Focus textarea after a short delay to allow portal to render
      requestAnimationFrame(() => {
        textareaRef.current?.focus();
      });

      // Prevent body scroll
      document.body.style.overflow = 'hidden';
    } else {
      // Restore body scroll
      document.body.style.overflow = '';

      // Return focus to previous element
      requestAnimationFrame(() => {
        previousActiveElement.current?.focus();
      });
    }

    return () => {
      document.body.style.overflow = '';
    };
  }, [isOpen]);

  // Handle form submission
  const handleSubmit = useCallback(async () => {
    if (!content.trim()) {
      setLocalError('Please enter a comment');
      textareaRef.current?.focus();
      return;
    }

    setLocalError(null);

    try {
      const success = await onSubmit(content.trim());
      if (success) {
        setContent('');
        onClose();
      }
    } catch (e) {
      setLocalError(String(e));
    }
  }, [content, onSubmit, onClose]);

  // Handle keyboard events
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
      } else if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        handleSubmit();
      }
    },
    [onClose, handleSubmit]
  );

  // Handle backdrop click
  const handleBackdropClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) {
        onClose();
      }
    },
    [onClose]
  );

  // Focus trap
  const handleTabKey = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key !== 'Tab' || !dialogRef.current) return;

      const focusableElements = dialogRef.current.querySelectorAll(
        'button:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
      );
      const firstElement = focusableElements[0] as HTMLElement;
      const lastElement = focusableElements[focusableElements.length - 1] as HTMLElement;

      if (e.shiftKey) {
        if (document.activeElement === firstElement) {
          e.preventDefault();
          lastElement.focus();
        }
      } else {
        if (document.activeElement === lastElement) {
          e.preventDefault();
          firstElement.focus();
        }
      }
    },
    []
  );

  if (!isOpen) {
    return null;
  }

  const displayError = localError || error;

  const dialog = (
    <div
      className="add-comment-dialog-overlay"
      onClick={handleBackdropClick}
      onKeyDown={(e) => {
        handleKeyDown(e);
        handleTabKey(e);
      }}
      role="presentation"
    >
      <div
        ref={dialogRef}
        className="add-comment-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="add-comment-title"
        aria-describedby={selection ? 'selected-text-desc' : undefined}
      >
        {/* Header */}
        <header className="add-comment-dialog-header">
          <h3 id="add-comment-title">Add Comment</h3>
          <button
            className="add-comment-dialog-close"
            onClick={onClose}
            aria-label="Close dialog"
            disabled={isSubmitting}
          >
            <span aria-hidden="true">&times;</span>
          </button>
        </header>

        {/* Body */}
        <div className="add-comment-dialog-body">
          {/* Selected text preview */}
          {selection && selection.text && (
            <div className="selected-text-preview" id="selected-text-desc">
              <span className="selected-text-label">Commenting on:</span>
              <div className="selected-text-content">
                "{truncateText(selection.text, 150)}"
              </div>
            </div>
          )}

          {/* Comment input */}
          <textarea
            ref={textareaRef}
            className="comment-textarea"
            value={content}
            onChange={(e) => {
              setContent(e.target.value);
              setLocalError(null);
            }}
            placeholder="Write your comment here..."
            aria-label="Comment text"
            aria-invalid={!!displayError}
            aria-describedby={displayError ? 'comment-error' : undefined}
            disabled={isSubmitting}
            rows={4}
          />

          {/* Error message */}
          {displayError && (
            <div
              id="comment-error"
              className="comment-error"
              role="alert"
              style={{
                marginTop: '8px',
                padding: '8px 12px',
                background: 'var(--danger-bg, #fee)',
                color: 'var(--danger-text, #dc2626)',
                borderRadius: '4px',
                fontSize: '13px',
              }}
            >
              {displayError}
            </div>
          )}

          {/* Hint */}
          <div
            style={{
              marginTop: '8px',
              fontSize: '11px',
              color: 'var(--text-tertiary, #888)',
            }}
          >
            Press Ctrl+Enter to submit, Escape to cancel
          </div>
        </div>

        {/* Footer */}
        <footer className="add-comment-dialog-footer">
          <button
            className="dialog-btn dialog-btn-secondary"
            onClick={onClose}
            disabled={isSubmitting}
            type="button"
          >
            Cancel
          </button>
          <button
            className="dialog-btn dialog-btn-primary"
            onClick={handleSubmit}
            disabled={isSubmitting || !content.trim()}
            type="button"
          >
            {isSubmitting ? 'Adding...' : 'Add Comment'}
          </button>
        </footer>
      </div>
    </div>
  );

  // Render into portal
  return createPortal(dialog, document.body);
}

// =============================================================================
// Positioned Dialog (appears near selection)
// =============================================================================

interface AddCommentPopupProps extends AddCommentDialogProps {
  /** Position of the popup relative to the document */
  position?: { top: number; left: number };
  /** Whether to anchor the popup to the selection position */
  anchorToSelection?: boolean;
}

export function AddCommentPopup({
  position,
  anchorToSelection = false,
  ...props
}: AddCommentPopupProps) {
  const [popupPosition, setPopupPosition] = useState<{ top: number; left: number } | null>(null);

  useEffect(() => {
    if (!props.isOpen || !anchorToSelection) {
      setPopupPosition(null);
      return;
    }

    // Get the current selection position
    const selection = window.getSelection();
    if (selection && selection.rangeCount > 0) {
      const range = selection.getRangeAt(0);
      const rect = range.getBoundingClientRect();

      // Position popup below the selection
      setPopupPosition({
        top: rect.bottom + window.scrollY + 10,
        left: Math.max(20, rect.left + window.scrollX),
      });
    } else if (position) {
      setPopupPosition(position);
    }
  }, [props.isOpen, anchorToSelection, position]);

  // If no position and not anchoring to selection, use centered dialog
  if (!popupPosition && !position) {
    return <AddCommentDialog {...props} />;
  }

  // Render positioned popup
  if (!props.isOpen) {
    return null;
  }

  const finalPosition = popupPosition || position;

  return createPortal(
    <div
      className="add-comment-popup"
      style={{
        position: 'fixed',
        top: finalPosition?.top || 0,
        left: finalPosition?.left || 0,
        zIndex: 1000,
        maxWidth: '400px',
      }}
    >
      <div
        className="add-comment-dialog"
        role="dialog"
        aria-modal="false"
        aria-labelledby="add-comment-popup-title"
      >
        <header className="add-comment-dialog-header">
          <h3 id="add-comment-popup-title">Add Comment</h3>
          <button
            className="add-comment-dialog-close"
            onClick={props.onClose}
            aria-label="Close"
          >
            <span aria-hidden="true">&times;</span>
          </button>
        </header>
        <div className="add-comment-dialog-body">
          {props.selection?.text && (
            <div className="selected-text-preview">
              <span className="selected-text-label">Commenting on:</span>
              <div className="selected-text-content">
                "{truncateText(props.selection.text, 100)}"
              </div>
            </div>
          )}
          <QuickCommentInput
            onSubmit={props.onSubmit}
            onCancel={props.onClose}
            isSubmitting={props.isSubmitting}
          />
        </div>
      </div>
    </div>,
    document.body
  );
}

// =============================================================================
// Quick Comment Input (inline version)
// =============================================================================

interface QuickCommentInputProps {
  onSubmit: (content: string) => Promise<boolean>;
  onCancel: () => void;
  isSubmitting?: boolean;
  placeholder?: string;
}

export function QuickCommentInput({
  onSubmit,
  onCancel,
  isSubmitting = false,
  placeholder = 'Add a comment...',
}: QuickCommentInputProps) {
  const [content, setContent] = useState('');
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleSubmit = useCallback(async () => {
    if (!content.trim()) return;

    const success = await onSubmit(content.trim());
    if (success) {
      setContent('');
    }
  }, [content, onSubmit]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        onCancel();
      } else if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        handleSubmit();
      }
    },
    [onCancel, handleSubmit]
  );

  return (
    <div className="quick-comment-input">
      <textarea
        ref={inputRef}
        className="comment-textarea"
        value={content}
        onChange={(e) => setContent(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        disabled={isSubmitting}
        rows={2}
        aria-label="Comment text"
      />
      <div
        style={{
          display: 'flex',
          justifyContent: 'flex-end',
          gap: '8px',
          marginTop: '8px',
        }}
      >
        <button
          className="dialog-btn dialog-btn-secondary"
          onClick={onCancel}
          disabled={isSubmitting}
          type="button"
          style={{ padding: '6px 12px', fontSize: '12px' }}
        >
          Cancel
        </button>
        <button
          className="dialog-btn dialog-btn-primary"
          onClick={handleSubmit}
          disabled={isSubmitting || !content.trim()}
          type="button"
          style={{ padding: '6px 12px', fontSize: '12px' }}
        >
          {isSubmitting ? '...' : 'Comment'}
        </button>
      </div>
    </div>
  );
}

export default AddCommentDialog;
