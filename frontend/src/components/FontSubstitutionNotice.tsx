import { useState, useEffect, useCallback } from 'react';
import {
  SubstitutionReason,
  FontSubstitutionRecord,
  FontSubstitutionSummary,
} from '../lib/types';

// Re-export types for convenience
export type { SubstitutionReason, FontSubstitutionRecord, FontSubstitutionSummary };

interface FontSubstitutionNoticeProps {
  /** The substitution summary from the backend */
  summary: FontSubstitutionSummary | null;
  /** Callback when user dismisses the notice */
  onDismiss?: () => void;
  /** Callback when user clicks "Install fonts" */
  onInstallFonts?: (fonts: string[]) => void;
  /** Whether to show the "Don't show again" option */
  showDontShowAgain?: boolean;
  /** Callback for "Don't show again" */
  onDontShowAgain?: () => void;
}

/**
 * Get a human-readable reason for substitution
 */
function getReasonText(reason: SubstitutionReason): string {
  switch (reason) {
    case 'NotInstalled':
      return 'not installed';
    case 'VariantNotAvailable':
      return 'variant not available';
    case 'ScriptNotSupported':
      return 'script not supported';
    case 'FallbackToDefault':
      return 'using default';
    default:
      return 'substituted';
  }
}

/**
 * Format font weight and style for display
 */
function formatFontVariant(
  weight: 'Normal' | 'Bold',
  style: 'Normal' | 'Italic'
): string {
  const parts: string[] = [];
  if (weight === 'Bold') parts.push('Bold');
  if (style === 'Italic') parts.push('Italic');
  return parts.length > 0 ? ` (${parts.join(' ')})` : '';
}

/**
 * FontSubstitutionNotice component
 *
 * Displays a banner/toast notification when fonts have been substituted
 * in the document. Allows users to see details, install missing fonts,
 * or dismiss the notification.
 */
export function FontSubstitutionNotice({
  summary,
  onDismiss,
  onInstallFonts,
  showDontShowAgain = true,
  onDontShowAgain,
}: FontSubstitutionNoticeProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isVisible, setIsVisible] = useState(true);

  // Hide if no substitutions
  useEffect(() => {
    if (!summary || summary.substitutions.length === 0) {
      setIsVisible(false);
    } else {
      setIsVisible(true);
    }
  }, [summary]);

  const handleDismiss = useCallback(() => {
    setIsVisible(false);
    onDismiss?.();
  }, [onDismiss]);

  const handleDontShowAgain = useCallback(() => {
    setIsVisible(false);
    onDontShowAgain?.();
  }, [onDontShowAgain]);

  const handleInstallFonts = useCallback(() => {
    if (!summary) return;
    const missingFonts = summary.substitutions
      .filter((s) => s.reason === 'NotInstalled')
      .map((s) => s.requested_font);
    onInstallFonts?.(missingFonts);
  }, [summary, onInstallFonts]);

  const toggleExpanded = useCallback(() => {
    setIsExpanded((prev) => !prev);
  }, []);

  // Don't render if not visible or no summary
  if (!isVisible || !summary || summary.substitutions.length === 0) {
    return null;
  }

  const substitutionCount = summary.substitutions.length;
  const hasMissingFonts = summary.substitutions.some(
    (s) => s.reason === 'NotInstalled'
  );

  return (
    <div className="font-substitution-notice" role="alert">
      <div className="font-substitution-notice-header">
        <div className="font-substitution-notice-icon">
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              d="M8 1C4.13 1 1 4.13 1 8C1 11.87 4.13 15 8 15C11.87 15 15 11.87 15 8C15 4.13 11.87 1 8 1ZM8 14C4.69 14 2 11.31 2 8C2 4.69 4.69 2 8 2C11.31 2 14 4.69 14 8C14 11.31 11.31 14 8 14ZM7 11H9V13H7V11ZM7 3H9V9H7V3Z"
              fill="currentColor"
            />
          </svg>
        </div>

        <div className="font-substitution-notice-message">
          <span className="font-substitution-notice-title">
            {substitutionCount} font{substitutionCount !== 1 ? 's' : ''}{' '}
            substituted
          </span>
          <button
            className="font-substitution-notice-toggle"
            onClick={toggleExpanded}
            aria-expanded={isExpanded}
          >
            {isExpanded ? 'Hide details' : 'Show details'}
          </button>
        </div>

        <div className="font-substitution-notice-actions">
          {hasMissingFonts && onInstallFonts && (
            <button
              className="font-substitution-notice-button font-substitution-notice-button-primary"
              onClick={handleInstallFonts}
            >
              Install fonts
            </button>
          )}
          <button
            className="font-substitution-notice-button font-substitution-notice-button-dismiss"
            onClick={handleDismiss}
            aria-label="Dismiss"
          >
            <svg
              width="12"
              height="12"
              viewBox="0 0 12 12"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M11 1L1 11M1 1L11 11"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
              />
            </svg>
          </button>
        </div>
      </div>

      {isExpanded && (
        <div className="font-substitution-notice-details">
          <table className="font-substitution-table">
            <thead>
              <tr>
                <th>Requested Font</th>
                <th>Substituted With</th>
                <th>Reason</th>
                <th>Count</th>
              </tr>
            </thead>
            <tbody>
              {summary.substitutions.map((sub, index) => (
                <tr key={index}>
                  <td>
                    {sub.requested_font}
                    {formatFontVariant(sub.requested_weight, sub.requested_style)}
                  </td>
                  <td>{sub.actual_font}</td>
                  <td>{getReasonText(sub.reason)}</td>
                  <td>{sub.occurrence_count}</td>
                </tr>
              ))}
            </tbody>
          </table>

          {showDontShowAgain && onDontShowAgain && (
            <div className="font-substitution-notice-footer">
              <button
                className="font-substitution-notice-link"
                onClick={handleDontShowAgain}
              >
                Don't show again for this document
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

/**
 * Hook to manage font substitution state
 */
export function useFontSubstitution() {
  const [summary, setSummary] = useState<FontSubstitutionSummary | null>(null);
  const [dismissed, setDismissed] = useState(false);
  const [neverShow, setNeverShow] = useState(false);

  const updateSummary = useCallback((newSummary: FontSubstitutionSummary) => {
    setSummary(newSummary);
    setDismissed(false);
  }, []);

  const dismiss = useCallback(() => {
    setDismissed(true);
  }, []);

  const dontShowAgain = useCallback(() => {
    setNeverShow(true);
    setDismissed(true);
  }, []);

  const shouldShow = !dismissed && !neverShow && summary !== null;

  return {
    summary: shouldShow ? summary : null,
    updateSummary,
    dismiss,
    dontShowAgain,
  };
}

export default FontSubstitutionNotice;
