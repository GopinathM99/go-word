import { useState, useCallback, useEffect } from 'react';
import { HyperlinkData, HyperlinkTargetType, HyperlinkRenderInfo } from '../lib/types';
import './HyperlinkDialog.css';

interface HyperlinkDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onInsert: (data: HyperlinkData) => void;
  onUpdate?: (data: HyperlinkData) => void;
  onRemove?: () => void;
  selectedText?: string;
  existingHyperlink?: HyperlinkRenderInfo | null;
}

/**
 * Validate URL format
 */
function validateUrl(url: string): { valid: boolean; error?: string } {
  if (!url.trim()) {
    return { valid: false, error: 'URL cannot be empty' };
  }

  // Check for dangerous protocols
  const lowerUrl = url.toLowerCase().trim();
  if (
    lowerUrl.startsWith('javascript:') ||
    lowerUrl.startsWith('data:') ||
    lowerUrl.startsWith('vbscript:')
  ) {
    return { valid: false, error: 'Unsafe URL protocol' };
  }

  // Basic URL validation
  try {
    // If it looks like a relative URL, allow it
    if (url.startsWith('/') || url.startsWith('#') || url.startsWith('./')) {
      return { valid: true };
    }

    // If it doesn't have a protocol, add https:// for validation
    const urlToValidate = url.includes('://') ? url : `https://${url}`;
    new URL(urlToValidate);
    return { valid: true };
  } catch {
    return { valid: false, error: 'Invalid URL format' };
  }
}

/**
 * Validate email address
 */
function validateEmail(email: string): { valid: boolean; error?: string } {
  if (!email.trim()) {
    return { valid: false, error: 'Email cannot be empty' };
  }

  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  if (!emailRegex.test(email)) {
    return { valid: false, error: 'Invalid email format' };
  }

  return { valid: true };
}

/**
 * Validate bookmark name
 */
function validateBookmark(bookmark: string): { valid: boolean; error?: string } {
  if (!bookmark.trim()) {
    return { valid: false, error: 'Bookmark name cannot be empty' };
  }

  return { valid: true };
}

export function HyperlinkDialog({
  isOpen,
  onClose,
  onInsert,
  onUpdate,
  onRemove,
  selectedText = '',
  existingHyperlink,
}: HyperlinkDialogProps) {
  const [targetType, setTargetType] = useState<HyperlinkTargetType>('external');
  const [url, setUrl] = useState('');
  const [bookmark, setBookmark] = useState('');
  const [email, setEmail] = useState('');
  const [subject, setSubject] = useState('');
  const [displayText, setDisplayText] = useState('');
  const [tooltip, setTooltip] = useState('');
  const [error, setError] = useState<string | null>(null);

  const isEditing = existingHyperlink !== null && existingHyperlink !== undefined;

  // Initialize form when dialog opens
  useEffect(() => {
    if (isOpen) {
      if (existingHyperlink) {
        // Editing existing hyperlink
        const linkType = existingHyperlink.link_type;
        setTargetType(
          linkType === 'External' ? 'external' : linkType === 'Internal' ? 'internal' : 'email'
        );

        if (linkType === 'External') {
          setUrl(existingHyperlink.target);
        } else if (linkType === 'Internal') {
          setBookmark(existingHyperlink.target.replace('#', ''));
        } else if (linkType === 'Email') {
          const mailtoRegex = /^mailto:([^?]+)(?:\?subject=(.*))?$/;
          const match = existingHyperlink.target.match(mailtoRegex);
          if (match) {
            setEmail(match[1]);
            setSubject(match[2] ? decodeURIComponent(match[2]) : '');
          }
        }

        setTooltip(existingHyperlink.tooltip || '');
        setDisplayText(selectedText);
      } else {
        // New hyperlink
        setTargetType('external');
        setUrl('');
        setBookmark('');
        setEmail('');
        setSubject('');
        setDisplayText(selectedText);
        setTooltip('');
      }
      setError(null);
    }
  }, [isOpen, existingHyperlink, selectedText]);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      setError(null);

      // Validate based on target type
      let validation: { valid: boolean; error?: string };
      switch (targetType) {
        case 'external':
          validation = validateUrl(url);
          break;
        case 'internal':
          validation = validateBookmark(bookmark);
          break;
        case 'email':
          validation = validateEmail(email);
          break;
        default:
          validation = { valid: false, error: 'Invalid target type' };
      }

      if (!validation.valid) {
        setError(validation.error || 'Invalid input');
        return;
      }

      const data: HyperlinkData = {
        targetType,
        tooltip: tooltip.trim() || undefined,
        displayText: displayText.trim() || undefined,
      };

      switch (targetType) {
        case 'external':
          // Add https:// if no protocol specified
          let finalUrl = url.trim();
          if (!finalUrl.includes('://') && !finalUrl.startsWith('/') && !finalUrl.startsWith('#')) {
            finalUrl = `https://${finalUrl}`;
          }
          data.url = finalUrl;
          break;
        case 'internal':
          data.bookmark = bookmark.trim();
          break;
        case 'email':
          data.email = email.trim();
          data.subject = subject.trim() || undefined;
          break;
      }

      if (isEditing && onUpdate) {
        onUpdate(data);
      } else {
        onInsert(data);
      }

      onClose();
    },
    [targetType, url, bookmark, email, subject, displayText, tooltip, isEditing, onInsert, onUpdate, onClose]
  );

  const handleRemove = useCallback(() => {
    if (onRemove) {
      onRemove();
    }
    onClose();
  }, [onRemove, onClose]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    },
    [onClose]
  );

  if (!isOpen) {
    return null;
  }

  // Preview the link appearance
  const previewText = displayText || url || email || bookmark || 'Link';
  const previewUrl =
    targetType === 'external'
      ? url || 'https://example.com'
      : targetType === 'email'
      ? `mailto:${email}${subject ? `?subject=${encodeURIComponent(subject)}` : ''}`
      : `#${bookmark}`;

  return (
    <div className="hyperlink-dialog-overlay" onClick={onClose} onKeyDown={handleKeyDown}>
      <div
        className="hyperlink-dialog"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-labelledby="hyperlink-dialog-title"
        aria-modal="true"
      >
        <header className="hyperlink-dialog-header">
          <h2 id="hyperlink-dialog-title">{isEditing ? 'Edit Hyperlink' : 'Insert Hyperlink'}</h2>
          <button className="close-button" onClick={onClose} aria-label="Close dialog">
            X
          </button>
        </header>

        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="target-type">Link to:</label>
            <select
              id="target-type"
              value={targetType}
              onChange={(e) => setTargetType(e.target.value as HyperlinkTargetType)}
            >
              <option value="external">Web Page (URL)</option>
              <option value="internal">Place in Document (Bookmark)</option>
              <option value="email">Email Address</option>
            </select>
          </div>

          {targetType === 'external' && (
            <div className="form-group">
              <label htmlFor="url">URL:</label>
              <input
                id="url"
                type="text"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="https://example.com"
                autoFocus
              />
            </div>
          )}

          {targetType === 'internal' && (
            <div className="form-group">
              <label htmlFor="bookmark">Bookmark:</label>
              <input
                id="bookmark"
                type="text"
                value={bookmark}
                onChange={(e) => setBookmark(e.target.value)}
                placeholder="section-name"
                autoFocus
              />
            </div>
          )}

          {targetType === 'email' && (
            <>
              <div className="form-group">
                <label htmlFor="email">Email Address:</label>
                <input
                  id="email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="name@example.com"
                  autoFocus
                />
              </div>
              <div className="form-group">
                <label htmlFor="subject">Subject (optional):</label>
                <input
                  id="subject"
                  type="text"
                  value={subject}
                  onChange={(e) => setSubject(e.target.value)}
                  placeholder="Email subject"
                />
              </div>
            </>
          )}

          <div className="form-group">
            <label htmlFor="display-text">Display Text:</label>
            <input
              id="display-text"
              type="text"
              value={displayText}
              onChange={(e) => setDisplayText(e.target.value)}
              placeholder={selectedText || 'Text to display'}
            />
          </div>

          <div className="form-group">
            <label htmlFor="tooltip">Tooltip (optional):</label>
            <input
              id="tooltip"
              type="text"
              value={tooltip}
              onChange={(e) => setTooltip(e.target.value)}
              placeholder="Tooltip text shown on hover"
            />
          </div>

          {error && <div className="error-message">{error}</div>}

          <div className="preview-section">
            <label>Preview:</label>
            <div className="link-preview">
              <a
                href={previewUrl}
                onClick={(e) => e.preventDefault()}
                title={tooltip || undefined}
                className="preview-link"
              >
                {previewText}
              </a>
            </div>
          </div>

          <footer className="hyperlink-dialog-footer">
            {isEditing && onRemove && (
              <button type="button" className="remove-button" onClick={handleRemove}>
                Remove Link
              </button>
            )}
            <div className="button-group">
              <button type="button" className="cancel-button" onClick={onClose}>
                Cancel
              </button>
              <button type="submit" className="submit-button">
                {isEditing ? 'Update' : 'Insert'}
              </button>
            </div>
          </footer>
        </form>
      </div>
    </div>
  );
}

export default HyperlinkDialog;
