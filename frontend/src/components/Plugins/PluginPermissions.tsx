/**
 * Plugin Permissions Dialog
 *
 * Displays a permission grant dialog when a plugin requests permissions.
 * Features:
 * - Shows requested permissions with descriptions
 * - Highlights sensitive permissions
 * - Allows granting or denying individual permissions
 * - Remembers user's choice
 */

import { useState, useCallback, useEffect } from 'react';
import { Permission } from './PluginManager';
import './Plugins.css';

/** Permission request from a plugin */
export interface PermissionRequest {
  pluginId: string;
  pluginName: string;
  permissions: Permission[];
  reason?: string;
}

/** Permission details for display */
interface PermissionInfo {
  label: string;
  description: string;
  isSensitive: boolean;
  icon: string;
}

/** Get detailed permission information */
function getPermissionInfo(permission: Permission): PermissionInfo {
  const info: Record<Permission, PermissionInfo> = {
    document_read: {
      label: 'Read Documents',
      description: 'Access and read the content of your documents.',
      isSensitive: false,
      icon: 'R',
    },
    document_write: {
      label: 'Modify Documents',
      description: 'Make changes to your documents, including adding, editing, or deleting content.',
      isSensitive: true,
      icon: 'W',
    },
    ui_toolbar: {
      label: 'Add Toolbar Items',
      description: 'Add buttons and controls to the application toolbar.',
      isSensitive: false,
      icon: 'T',
    },
    ui_panel: {
      label: 'Create Panels',
      description: 'Create side panels in the application interface.',
      isSensitive: false,
      icon: 'P',
    },
    ui_dialog: {
      label: 'Show Dialogs',
      description: 'Display dialog windows and notifications.',
      isSensitive: false,
      icon: 'D',
    },
    network: {
      label: 'Network Access',
      description: 'Connect to the internet to send and receive data.',
      isSensitive: true,
      icon: 'N',
    },
    storage: {
      label: 'Store Data',
      description: 'Save data locally on your computer.',
      isSensitive: false,
      icon: 'S',
    },
    clipboard: {
      label: 'Clipboard Access',
      description: 'Read from and write to your system clipboard.',
      isSensitive: true,
      icon: 'C',
    },
  };
  return info[permission];
}

interface PluginPermissionsProps {
  isOpen: boolean;
  request: PermissionRequest | null;
  onGrant: (pluginId: string, permissions: Permission[]) => void;
  onDeny: (pluginId: string) => void;
  onDenyPermanently?: (pluginId: string) => void;
}

export function PluginPermissions({
  isOpen,
  request,
  onGrant,
  onDeny,
  onDenyPermanently,
}: PluginPermissionsProps) {
  const [selectedPermissions, setSelectedPermissions] = useState<Set<Permission>>(
    new Set()
  );
  const [rememberChoice, setRememberChoice] = useState(false);

  // Initialize selected permissions when request changes
  useEffect(() => {
    if (request) {
      setSelectedPermissions(new Set(request.permissions));
      setRememberChoice(false);
    }
  }, [request]);

  // Close on escape
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen && request) {
        onDeny(request.pluginId);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, request, onDeny]);

  const handleTogglePermission = useCallback((permission: Permission) => {
    setSelectedPermissions((prev) => {
      const next = new Set(prev);
      if (next.has(permission)) {
        next.delete(permission);
      } else {
        next.add(permission);
      }
      return next;
    });
  }, []);

  const handleSelectAll = useCallback(() => {
    if (request) {
      setSelectedPermissions(new Set(request.permissions));
    }
  }, [request]);

  const handleDeselectAll = useCallback(() => {
    setSelectedPermissions(new Set());
  }, []);

  const handleGrant = useCallback(() => {
    if (request) {
      onGrant(request.pluginId, Array.from(selectedPermissions));
    }
  }, [request, selectedPermissions, onGrant]);

  const handleDeny = useCallback(() => {
    if (request) {
      if (rememberChoice && onDenyPermanently) {
        onDenyPermanently(request.pluginId);
      } else {
        onDeny(request.pluginId);
      }
    }
  }, [request, rememberChoice, onDeny, onDenyPermanently]);

  if (!isOpen || !request) return null;

  const sensitivePermissions = request.permissions.filter(
    (p) => getPermissionInfo(p).isSensitive
  );
  const hasSensitivePermissions = sensitivePermissions.length > 0;

  return (
    <div className="plugin-permissions-overlay">
      <div
        className="plugin-permissions-dialog"
        role="dialog"
        aria-labelledby="permissions-title"
        aria-modal="true"
      >
        <header className="plugin-permissions-header">
          <h2 id="permissions-title">Permission Request</h2>
        </header>

        <div className="plugin-permissions-content">
          <div className="plugin-permissions-intro">
            <strong>{request.pluginName}</strong> is requesting the following permissions:
          </div>

          {request.reason && (
            <div className="plugin-permissions-reason">
              <em>Reason: {request.reason}</em>
            </div>
          )}

          {hasSensitivePermissions && (
            <div className="plugin-permissions-warning">
              <span className="warning-icon">!</span>
              <span>
                This plugin requests sensitive permissions. Please review carefully before
                granting.
              </span>
            </div>
          )}

          <div className="plugin-permissions-list">
            {request.permissions.map((permission) => {
              const info = getPermissionInfo(permission);
              const isSelected = selectedPermissions.has(permission);

              return (
                <div
                  key={permission}
                  className={`plugin-permission-item ${info.isSensitive ? 'sensitive' : ''} ${
                    isSelected ? 'selected' : ''
                  }`}
                >
                  <label className="permission-checkbox">
                    <input
                      type="checkbox"
                      checked={isSelected}
                      onChange={() => handleTogglePermission(permission)}
                      aria-label={`Grant ${info.label} permission`}
                    />
                    <span className="permission-icon">{info.icon}</span>
                    <div className="permission-details">
                      <span className="permission-label">
                        {info.label}
                        {info.isSensitive && (
                          <span className="sensitive-badge">Sensitive</span>
                        )}
                      </span>
                      <span className="permission-description">{info.description}</span>
                    </div>
                  </label>
                </div>
              );
            })}
          </div>

          <div className="plugin-permissions-quick-actions">
            <button
              type="button"
              className="quick-action-btn"
              onClick={handleSelectAll}
            >
              Select All
            </button>
            <button
              type="button"
              className="quick-action-btn"
              onClick={handleDeselectAll}
            >
              Deselect All
            </button>
          </div>

          {onDenyPermanently && (
            <label className="plugin-permissions-remember">
              <input
                type="checkbox"
                checked={rememberChoice}
                onChange={(e) => setRememberChoice(e.target.checked)}
              />
              <span>Remember my choice for this plugin</span>
            </label>
          )}
        </div>

        <footer className="plugin-permissions-footer">
          <div className="plugin-permissions-summary">
            {selectedPermissions.size} of {request.permissions.length} permissions selected
          </div>
          <div className="plugin-permissions-actions">
            <button
              className="plugin-permissions-btn deny"
              onClick={handleDeny}
            >
              Deny
            </button>
            <button
              className="plugin-permissions-btn grant"
              onClick={handleGrant}
              disabled={selectedPermissions.size === 0}
            >
              Grant Selected
            </button>
          </div>
        </footer>
      </div>
    </div>
  );
}

/**
 * Hook for managing permission requests
 */
export function usePermissionRequests() {
  const [currentRequest, setCurrentRequest] = useState<PermissionRequest | null>(null);
  const [pendingRequests, setPendingRequests] = useState<PermissionRequest[]>([]);

  const requestPermissions = useCallback((request: PermissionRequest) => {
    if (currentRequest) {
      setPendingRequests((prev) => [...prev, request]);
    } else {
      setCurrentRequest(request);
    }
  }, [currentRequest]);

  const handleGrant = useCallback(
    (pluginId: string, permissions: Permission[]) => {
      // Process grant...
      console.log('Granted permissions for', pluginId, ':', permissions);

      // Move to next request
      setCurrentRequest(pendingRequests[0] || null);
      setPendingRequests((prev) => prev.slice(1));
    },
    [pendingRequests]
  );

  const handleDeny = useCallback(
    (pluginId: string) => {
      // Process deny...
      console.log('Denied permissions for', pluginId);

      // Move to next request
      setCurrentRequest(pendingRequests[0] || null);
      setPendingRequests((prev) => prev.slice(1));
    },
    [pendingRequests]
  );

  return {
    currentRequest,
    pendingCount: pendingRequests.length,
    requestPermissions,
    handleGrant,
    handleDeny,
  };
}
