/**
 * AccessibleDialog.tsx
 *
 * A fully accessible dialog component that implements:
 * - Focus trapping
 * - Escape to close
 * - Return focus to trigger element on close
 * - Proper ARIA roles and labels
 * - Backdrop click to close (optional)
 */

import React, { useEffect, useRef, useCallback } from 'react';
import { createPortal } from 'react-dom';

// =============================================================================
// Types
// =============================================================================

export interface AccessibleDialogProps {
  /** Whether the dialog is open */
  isOpen: boolean;
  /** Callback to close the dialog */
  onClose: () => void;
  /** Dialog title (used for aria-labelledby) */
  title: string;
  /** Optional description (used for aria-describedby) */
  description?: string;
  /** Dialog content */
  children: React.ReactNode;
  /** Whether clicking the backdrop closes the dialog */
  closeOnBackdropClick?: boolean;
  /** Whether the dialog is an alert dialog (more urgent) */
  isAlertDialog?: boolean;
  /** Optional CSS class for the dialog */
  className?: string;
  /** Optional CSS class for the overlay */
  overlayClassName?: string;
  /** Footer content (buttons, etc.) */
  footer?: React.ReactNode;
  /** Size variant */
  size?: 'small' | 'medium' | 'large' | 'fullscreen';
  /** Initial focus element selector */
  initialFocusRef?: React.RefObject<HTMLElement>;
  /** Element to return focus to on close */
  returnFocusRef?: React.RefObject<HTMLElement>;
}

// =============================================================================
// Component
// =============================================================================

export function AccessibleDialog({
  isOpen,
  onClose,
  title,
  description,
  children,
  closeOnBackdropClick = true,
  isAlertDialog = false,
  className = '',
  overlayClassName = '',
  footer,
  size = 'medium',
  initialFocusRef,
  returnFocusRef,
}: AccessibleDialogProps) {
  const dialogRef = useRef<HTMLDivElement>(null);
  const previousActiveElement = useRef<HTMLElement | null>(null);
  const titleId = `dialog-title-${React.useId()}`;
  const descriptionId = description ? `dialog-desc-${React.useId()}` : undefined;

  // Focus trap - get all focusable elements
  const getFocusableElements = useCallback(() => {
    if (!dialogRef.current) return [];
    return Array.from(
      dialogRef.current.querySelectorAll(
        'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), ' +
        'textarea:not([disabled]), [tabindex]:not([tabindex="-1"]), [contenteditable]'
      )
    ).filter((el) => {
      if (el instanceof HTMLElement) {
        return el.offsetWidth > 0 && el.offsetHeight > 0;
      }
      return false;
    }) as HTMLElement[];
  }, []);

  // Handle focus trap
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        e.stopPropagation();
        onClose();
        return;
      }

      if (e.key === 'Tab') {
        const focusable = getFocusableElements();
        if (focusable.length === 0) {
          e.preventDefault();
          return;
        }

        const firstElement = focusable[0];
        const lastElement = focusable[focusable.length - 1];

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
      }
    },
    [onClose, getFocusableElements]
  );

  // Handle backdrop click
  const handleBackdropClick = useCallback(
    (e: React.MouseEvent) => {
      if (closeOnBackdropClick && e.target === e.currentTarget) {
        onClose();
      }
    },
    [closeOnBackdropClick, onClose]
  );

  // Focus management on open/close
  useEffect(() => {
    if (isOpen) {
      // Save current focus
      previousActiveElement.current = document.activeElement as HTMLElement;

      // Add keyboard listener
      document.addEventListener('keydown', handleKeyDown);

      // Focus initial element or first focusable
      requestAnimationFrame(() => {
        if (initialFocusRef?.current) {
          initialFocusRef.current.focus();
        } else {
          const focusable = getFocusableElements();
          if (focusable.length > 0) {
            focusable[0].focus();
          } else {
            dialogRef.current?.focus();
          }
        }
      });

      // Prevent body scroll
      document.body.style.overflow = 'hidden';
    } else {
      // Remove keyboard listener
      document.removeEventListener('keydown', handleKeyDown);

      // Restore body scroll
      document.body.style.overflow = '';

      // Return focus
      requestAnimationFrame(() => {
        if (returnFocusRef?.current) {
          returnFocusRef.current.focus();
        } else if (previousActiveElement.current) {
          previousActiveElement.current.focus();
        }
      });
    }

    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      document.body.style.overflow = '';
    };
  }, [isOpen, handleKeyDown, getFocusableElements, initialFocusRef, returnFocusRef]);

  if (!isOpen) return null;

  const sizeStyles: Record<string, React.CSSProperties> = {
    small: { maxWidth: '24rem' },
    medium: { maxWidth: '32rem' },
    large: { maxWidth: '48rem' },
    fullscreen: { maxWidth: '100vw', maxHeight: '100vh', width: '100vw', height: '100vh', borderRadius: 0 },
  };

  const dialog = (
    <div
      className={`dialog-overlay ${overlayClassName}`}
      onClick={handleBackdropClick}
      style={{
        position: 'fixed',
        inset: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}
      aria-hidden="true"
    >
      <div
        ref={dialogRef}
        role={isAlertDialog ? 'alertdialog' : 'dialog'}
        aria-modal="true"
        aria-labelledby={titleId}
        aria-describedby={descriptionId}
        className={`dialog ${className}`}
        style={{
          backgroundColor: 'var(--button-bg, white)',
          borderRadius: '0.5rem',
          boxShadow: '0 1rem 3rem rgba(0, 0, 0, 0.2)',
          width: '90vw',
          maxHeight: '90vh',
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
          ...sizeStyles[size],
        }}
        tabIndex={-1}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div
          className="dialog-header"
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '1rem 1.5rem',
            borderBottom: '1px solid var(--border-color, #ddd)',
          }}
        >
          <h2
            id={titleId}
            className="dialog-title"
            style={{
              margin: 0,
              fontSize: '1.25rem',
              fontWeight: 600,
            }}
          >
            {title}
          </h2>
          <button
            type="button"
            className="dialog-close-button"
            onClick={onClose}
            aria-label="Close dialog"
            style={{
              minWidth: '2.75rem',
              minHeight: '2.75rem',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              background: 'transparent',
              border: 'none',
              borderRadius: '0.25rem',
              cursor: 'pointer',
              fontSize: '1.5rem',
              lineHeight: 1,
            }}
          >
            <span aria-hidden="true">&times;</span>
          </button>
        </div>

        {/* Description (optional) */}
        {description && (
          <p
            id={descriptionId}
            className="dialog-description"
            style={{
              padding: '0 1.5rem',
              margin: '1rem 0 0',
              color: 'var(--text-color-muted, #666)',
              fontSize: '0.875rem',
            }}
          >
            {description}
          </p>
        )}

        {/* Content */}
        <div
          className="dialog-content"
          style={{
            flex: 1,
            padding: '1.5rem',
            overflowY: 'auto',
          }}
        >
          {children}
        </div>

        {/* Footer (optional) */}
        {footer && (
          <div
            className="dialog-footer"
            style={{
              display: 'flex',
              gap: '0.75rem',
              justifyContent: 'flex-end',
              padding: '1rem 1.5rem',
              borderTop: '1px solid var(--border-color, #ddd)',
            }}
          >
            {footer}
          </div>
        )}
      </div>
    </div>
  );

  // Render into portal
  return createPortal(dialog, document.body);
}

// =============================================================================
// Dialog Button Components
// =============================================================================

interface DialogButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger';
}

export function DialogButton({
  variant = 'secondary',
  children,
  style,
  ...props
}: DialogButtonProps) {
  const variantStyles: Record<string, React.CSSProperties> = {
    primary: {
      backgroundColor: 'var(--accent-color, #2563eb)',
      color: 'white',
      border: 'none',
    },
    secondary: {
      backgroundColor: 'var(--button-bg, white)',
      color: 'var(--text-color, #333)',
      border: '1px solid var(--border-color, #ddd)',
    },
    danger: {
      backgroundColor: '#dc2626',
      color: 'white',
      border: 'none',
    },
  };

  return (
    <button
      {...props}
      style={{
        minWidth: '2.75rem',
        minHeight: '2.75rem',
        padding: '0.75rem 1.5rem',
        borderRadius: '0.375rem',
        fontSize: '0.875rem',
        fontWeight: 500,
        cursor: 'pointer',
        ...variantStyles[variant],
        ...style,
      }}
    >
      {children}
    </button>
  );
}

// =============================================================================
// Confirmation Dialog
// =============================================================================

export interface ConfirmDialogProps {
  isOpen: boolean;
  onConfirm: () => void;
  onCancel: () => void;
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  variant?: 'default' | 'danger';
}

export function ConfirmDialog({
  isOpen,
  onConfirm,
  onCancel,
  title,
  message,
  confirmLabel = 'Confirm',
  cancelLabel = 'Cancel',
  variant = 'default',
}: ConfirmDialogProps) {
  const handleConfirm = useCallback(() => {
    onConfirm();
  }, [onConfirm]);

  return (
    <AccessibleDialog
      isOpen={isOpen}
      onClose={onCancel}
      title={title}
      description={message}
      isAlertDialog
      size="small"
      footer={
        <>
          <DialogButton variant="secondary" onClick={onCancel}>
            {cancelLabel}
          </DialogButton>
          <DialogButton
            variant={variant === 'danger' ? 'danger' : 'primary'}
            onClick={handleConfirm}
          >
            {confirmLabel}
          </DialogButton>
        </>
      }
    >
      <p>{message}</p>
    </AccessibleDialog>
  );
}

// =============================================================================
// Hook for Dialog Management
// =============================================================================

export function useDialog(initialOpen = false) {
  const [isOpen, setIsOpen] = React.useState(initialOpen);
  const triggerRef = useRef<HTMLElement>(null);

  const open = useCallback(() => {
    setIsOpen(true);
  }, []);

  const close = useCallback(() => {
    setIsOpen(false);
  }, []);

  const toggle = useCallback(() => {
    setIsOpen((prev) => !prev);
  }, []);

  return {
    isOpen,
    open,
    close,
    toggle,
    triggerRef,
    dialogProps: {
      isOpen,
      onClose: close,
      returnFocusRef: triggerRef,
    },
  };
}
