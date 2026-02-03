import React from 'react';
import './OfflineIndicator.css';

export type ConnectionStatus = 'online' | 'offline' | 'reconnecting' | 'syncing';

export interface OfflineIndicatorProps {
  /** Current connection status */
  status: ConnectionStatus;
  /** Number of changes pending sync */
  pendingChanges: number;
  /** Time in seconds since last successful sync */
  timeSinceSync?: number;
  /** Callback when retry button is clicked */
  onRetryClick?: () => void;
  /** Optional className for custom styling */
  className?: string;
}

/** Status configuration for different connection states */
const statusConfig: Record<ConnectionStatus, { icon: string; color: string; text: string }> = {
  online: { icon: '\u25CF', color: 'green', text: 'Connected' },
  offline: { icon: '\u25CB', color: 'red', text: 'Offline' },
  reconnecting: { icon: '\u25D0', color: 'orange', text: 'Reconnecting...' },
  syncing: { icon: '\u21BB', color: 'blue', text: 'Syncing...' },
};

/**
 * Format seconds into a human-readable time string
 */
function formatTime(seconds: number): string {
  if (seconds < 60) {
    return `${seconds}s ago`;
  }
  if (seconds < 3600) {
    return `${Math.floor(seconds / 60)}m ago`;
  }
  if (seconds < 86400) {
    return `${Math.floor(seconds / 3600)}h ago`;
  }
  return `${Math.floor(seconds / 86400)}d ago`;
}

/**
 * OfflineIndicator displays the current connection status and pending changes.
 *
 * Features:
 * - Shows connection status with colored icon
 * - Displays pending changes count
 * - Shows time since last sync when offline
 * - Provides retry button for manual reconnection
 * - Hides automatically when online with no pending changes
 */
export const OfflineIndicator: React.FC<OfflineIndicatorProps> = ({
  status,
  pendingChanges,
  timeSinceSync,
  onRetryClick,
  className,
}) => {
  // Don't show when online with no pending changes
  if (status === 'online' && pendingChanges === 0) {
    return null;
  }

  const config = statusConfig[status];

  return (
    <div
      className={`offline-indicator offline-indicator--${status} ${className || ''}`}
      role="status"
      aria-live="polite"
      aria-label={`Connection status: ${config.text}${pendingChanges > 0 ? `, ${pendingChanges} pending changes` : ''}`}
    >
      <span
        className="offline-indicator__icon"
        style={{ color: config.color }}
        aria-hidden="true"
      >
        {config.icon}
      </span>
      <span className="offline-indicator__text">{config.text}</span>

      {pendingChanges > 0 && (
        <span className="offline-indicator__pending">
          {pendingChanges} pending
        </span>
      )}

      {timeSinceSync !== undefined && status === 'offline' && (
        <span className="offline-indicator__time">
          Last sync: {formatTime(timeSinceSync)}
        </span>
      )}

      {status === 'offline' && onRetryClick && (
        <button
          className="offline-indicator__retry"
          onClick={onRetryClick}
          title="Try to reconnect"
          aria-label="Retry connection"
        >
          Retry
        </button>
      )}
    </div>
  );
};

/**
 * Hook to manage offline status with automatic updates
 */
export function useOfflineStatus(initialStatus: ConnectionStatus = 'online') {
  const [status, setStatus] = React.useState<ConnectionStatus>(initialStatus);
  const [pendingChanges, setPendingChanges] = React.useState(0);
  const [lastSyncTime, setLastSyncTime] = React.useState<number | undefined>(undefined);
  const [timeSinceSync, setTimeSinceSync] = React.useState<number | undefined>(undefined);

  // Update time since sync every second when offline
  React.useEffect(() => {
    if (status !== 'offline' || lastSyncTime === undefined) {
      return;
    }

    const updateTime = () => {
      const now = Math.floor(Date.now() / 1000);
      setTimeSinceSync(now - lastSyncTime);
    };

    updateTime();
    const interval = setInterval(updateTime, 1000);

    return () => clearInterval(interval);
  }, [status, lastSyncTime]);

  const markSynced = React.useCallback(() => {
    setLastSyncTime(Math.floor(Date.now() / 1000));
    setTimeSinceSync(0);
  }, []);

  const addPendingChange = React.useCallback(() => {
    setPendingChanges((prev) => prev + 1);
  }, []);

  const clearPendingChanges = React.useCallback(() => {
    setPendingChanges(0);
  }, []);

  return {
    status,
    setStatus,
    pendingChanges,
    setPendingChanges,
    addPendingChange,
    clearPendingChanges,
    timeSinceSync,
    markSynced,
  };
}

/**
 * Hook to detect browser online/offline status
 */
export function useBrowserOnlineStatus(): boolean {
  const [isOnline, setIsOnline] = React.useState(
    typeof navigator !== 'undefined' ? navigator.onLine : true
  );

  React.useEffect(() => {
    const handleOnline = () => setIsOnline(true);
    const handleOffline = () => setIsOnline(false);

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, []);

  return isOnline;
}

export default OfflineIndicator;
