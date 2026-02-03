/**
 * Autosave Indicator - Shows the current save status
 *
 * Displays:
 * - Saved: Document is saved, no unsaved changes
 * - Saving...: Save in progress
 * - Unsaved changes: Document has unsaved changes
 * - Error: Save failed
 */

import { useMemo } from 'react';
import { AutosaveStatus, getSaveState, SaveState } from '../lib/types';
import './AutosaveIndicator.css';

interface AutosaveIndicatorProps {
  /** Current autosave status */
  status: AutosaveStatus;
  /** Whether to show detailed status */
  detailed?: boolean;
  /** Click handler for the indicator */
  onClick?: () => void;
}

export function AutosaveIndicator({
  status,
  detailed = false,
  onClick,
}: AutosaveIndicatorProps) {
  const saveState = useMemo(() => getSaveState(status), [status]);

  const { icon, label, description } = useMemo(() => {
    switch (saveState) {
      case 'saved':
        return {
          icon: 'check',
          label: 'Saved',
          description: status.lastSaveTime
            ? `Last saved ${formatRelativeTime(status.lastSaveTime)}`
            : 'All changes saved',
        };
      case 'saving':
        return {
          icon: 'sync',
          label: 'Saving...',
          description: 'Saving your changes',
        };
      case 'unsaved':
        return {
          icon: 'edit',
          label: 'Unsaved changes',
          description: status.nextSaveInSecs
            ? `Autosave in ${status.nextSaveInSecs}s`
            : 'Changes pending',
        };
      case 'error':
        return {
          icon: 'error',
          label: 'Save failed',
          description: status.lastError || 'Failed to save changes',
        };
    }
  }, [saveState, status]);

  const className = [
    'autosave-indicator',
    `autosave-indicator--${saveState}`,
    onClick ? 'autosave-indicator--clickable' : '',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <div
      className={className}
      onClick={onClick}
      role={onClick ? 'button' : 'status'}
      aria-live="polite"
      title={description}
    >
      <span className={`autosave-icon autosave-icon--${icon}`} aria-hidden="true">
        {getIconContent(icon)}
      </span>
      <span className="autosave-label">{label}</span>
      {detailed && (
        <span className="autosave-description">{description}</span>
      )}
    </div>
  );
}

/**
 * Get the icon content for a given icon type
 */
function getIconContent(icon: string): string {
  switch (icon) {
    case 'check':
      return '\u2713'; // Checkmark
    case 'sync':
      return '\u21BB'; // Clockwise arrow
    case 'edit':
      return '\u25CF'; // Filled circle
    case 'error':
      return '\u26A0'; // Warning triangle
    default:
      return '';
  }
}

/**
 * Format a timestamp as relative time
 */
function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diffSecs = Math.floor((now - timestamp) / 1000);

  if (diffSecs < 5) {
    return 'just now';
  } else if (diffSecs < 60) {
    return `${diffSecs} seconds ago`;
  } else if (diffSecs < 3600) {
    const mins = Math.floor(diffSecs / 60);
    return `${mins} minute${mins === 1 ? '' : 's'} ago`;
  } else if (diffSecs < 86400) {
    const hours = Math.floor(diffSecs / 3600);
    return `${hours} hour${hours === 1 ? '' : 's'} ago`;
  } else {
    const days = Math.floor(diffSecs / 86400);
    return `${days} day${days === 1 ? '' : 's'} ago`;
  }
}

/**
 * Compact autosave indicator for the status bar
 */
interface CompactAutosaveIndicatorProps {
  /** Current autosave status */
  status: AutosaveStatus;
}

export function CompactAutosaveIndicator({
  status,
}: CompactAutosaveIndicatorProps) {
  const saveState = useMemo(() => getSaveState(status), [status]);

  const { icon, tooltip } = useMemo(() => {
    switch (saveState) {
      case 'saved':
        return {
          icon: '\u2713',
          tooltip: 'All changes saved',
        };
      case 'saving':
        return {
          icon: '\u21BB',
          tooltip: 'Saving...',
        };
      case 'unsaved':
        return {
          icon: '\u25CF',
          tooltip: 'Unsaved changes',
        };
      case 'error':
        return {
          icon: '\u26A0',
          tooltip: status.lastError || 'Save failed',
        };
    }
  }, [saveState, status]);

  return (
    <span
      className={`autosave-compact autosave-compact--${saveState}`}
      title={tooltip}
      role="status"
      aria-label={tooltip}
    >
      {icon}
    </span>
  );
}
