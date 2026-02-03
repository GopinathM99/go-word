/**
 * Plugin Manager Component
 *
 * Displays a list of installed plugins with options to:
 * - Enable/disable plugins
 * - View plugin details
 * - Uninstall plugins
 * - Access plugin settings
 */

import { useState, useCallback, useEffect } from 'react';
import './Plugins.css';

/** Plugin permission types */
export type Permission =
  | 'document_read'
  | 'document_write'
  | 'ui_toolbar'
  | 'ui_panel'
  | 'ui_dialog'
  | 'network'
  | 'storage'
  | 'clipboard';

/** Plugin activation event */
export interface ActivationEvent {
  type: 'on_command' | 'on_document_open' | 'on_startup' | 'on_language';
  value?: string;
}

/** Plugin manifest */
export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  entry: string;
  permissions: Permission[];
  activation_events: ActivationEvent[];
}

/** Installed plugin with state */
export interface InstalledPlugin {
  manifest: PluginManifest;
  enabled: boolean;
  path: string;
  installedAt?: number;
}

interface PluginManagerProps {
  isOpen: boolean;
  onClose: () => void;
  plugins: InstalledPlugin[];
  onEnablePlugin: (pluginId: string) => void;
  onDisablePlugin: (pluginId: string) => void;
  onUninstallPlugin: (pluginId: string) => void;
  onOpenSettings?: (pluginId: string) => void;
  onBrowsePlugins?: () => void;
}

/** Get human-readable permission name */
function getPermissionLabel(permission: Permission): string {
  const labels: Record<Permission, string> = {
    document_read: 'Read Documents',
    document_write: 'Modify Documents',
    ui_toolbar: 'Add Toolbar Items',
    ui_panel: 'Create Panels',
    ui_dialog: 'Show Dialogs',
    network: 'Network Access',
    storage: 'Store Data',
    clipboard: 'Clipboard Access',
  };
  return labels[permission] || permission;
}

/** Check if a permission is sensitive */
function isSensitivePermission(permission: Permission): boolean {
  return ['document_write', 'network', 'clipboard'].includes(permission);
}

export function PluginManager({
  isOpen,
  onClose,
  plugins,
  onEnablePlugin,
  onDisablePlugin,
  onUninstallPlugin,
  onOpenSettings,
  onBrowsePlugins,
}: PluginManagerProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedPlugin, setSelectedPlugin] = useState<string | null>(null);
  const [filter, setFilter] = useState<'all' | 'enabled' | 'disabled'>('all');
  const [confirmUninstall, setConfirmUninstall] = useState<string | null>(null);

  // Close on escape
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        if (confirmUninstall) {
          setConfirmUninstall(null);
        } else if (selectedPlugin) {
          setSelectedPlugin(null);
        } else {
          onClose();
        }
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose, selectedPlugin, confirmUninstall]);

  // Filter plugins
  const filteredPlugins = plugins.filter((plugin) => {
    // Apply search filter
    const matchesSearch =
      searchQuery === '' ||
      plugin.manifest.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      plugin.manifest.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
      plugin.manifest.author.toLowerCase().includes(searchQuery.toLowerCase());

    // Apply status filter
    const matchesFilter =
      filter === 'all' ||
      (filter === 'enabled' && plugin.enabled) ||
      (filter === 'disabled' && !plugin.enabled);

    return matchesSearch && matchesFilter;
  });

  const handleTogglePlugin = useCallback(
    (plugin: InstalledPlugin) => {
      if (plugin.enabled) {
        onDisablePlugin(plugin.manifest.id);
      } else {
        onEnablePlugin(plugin.manifest.id);
      }
    },
    [onEnablePlugin, onDisablePlugin]
  );

  const handleUninstall = useCallback(
    (pluginId: string) => {
      onUninstallPlugin(pluginId);
      setConfirmUninstall(null);
      if (selectedPlugin === pluginId) {
        setSelectedPlugin(null);
      }
    },
    [onUninstallPlugin, selectedPlugin]
  );

  const selectedPluginData = selectedPlugin
    ? plugins.find((p) => p.manifest.id === selectedPlugin)
    : null;

  if (!isOpen) return null;

  return (
    <div className="plugin-manager-overlay" onClick={onClose}>
      <div
        className="plugin-manager-dialog"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-labelledby="plugin-manager-title"
        aria-modal="true"
      >
        <header className="plugin-manager-header">
          <h2 id="plugin-manager-title">Plugin Manager</h2>
          <button
            className="plugin-manager-close"
            onClick={onClose}
            aria-label="Close plugin manager"
          >
            x
          </button>
        </header>

        <div className="plugin-manager-content">
          {/* Left panel: Plugin list */}
          <div className="plugin-manager-list-panel">
            <div className="plugin-manager-toolbar">
              <input
                type="search"
                className="plugin-search"
                placeholder="Search plugins..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                aria-label="Search plugins"
              />
              <select
                className="plugin-filter"
                value={filter}
                onChange={(e) => setFilter(e.target.value as typeof filter)}
                aria-label="Filter plugins"
              >
                <option value="all">All</option>
                <option value="enabled">Enabled</option>
                <option value="disabled">Disabled</option>
              </select>
            </div>

            <div className="plugin-list">
              {filteredPlugins.length === 0 ? (
                <div className="plugin-list-empty">
                  {plugins.length === 0
                    ? 'No plugins installed'
                    : 'No plugins match your search'}
                </div>
              ) : (
                filteredPlugins.map((plugin) => (
                  <div
                    key={plugin.manifest.id}
                    className={`plugin-list-item ${
                      selectedPlugin === plugin.manifest.id ? 'selected' : ''
                    } ${!plugin.enabled ? 'disabled' : ''}`}
                    onClick={() => setSelectedPlugin(plugin.manifest.id)}
                    role="button"
                    tabIndex={0}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' || e.key === ' ') {
                        setSelectedPlugin(plugin.manifest.id);
                      }
                    }}
                  >
                    <div className="plugin-item-info">
                      <span className="plugin-item-name">{plugin.manifest.name}</span>
                      <span className="plugin-item-version">v{plugin.manifest.version}</span>
                    </div>
                    <div className="plugin-item-author">by {plugin.manifest.author}</div>
                    <label className="plugin-toggle" onClick={(e) => e.stopPropagation()}>
                      <input
                        type="checkbox"
                        checked={plugin.enabled}
                        onChange={() => handleTogglePlugin(plugin)}
                        aria-label={`${plugin.enabled ? 'Disable' : 'Enable'} ${plugin.manifest.name}`}
                      />
                      <span className="plugin-toggle-slider"></span>
                    </label>
                  </div>
                ))
              )}
            </div>

            {onBrowsePlugins && (
              <button className="plugin-browse-btn" onClick={onBrowsePlugins}>
                Browse More Plugins
              </button>
            )}
          </div>

          {/* Right panel: Plugin details */}
          <div className="plugin-manager-detail-panel">
            {selectedPluginData ? (
              <>
                <div className="plugin-detail-header">
                  <h3>{selectedPluginData.manifest.name}</h3>
                  <span className="plugin-detail-version">
                    v{selectedPluginData.manifest.version}
                  </span>
                </div>

                <div className="plugin-detail-meta">
                  <div className="plugin-detail-author">
                    <strong>Author:</strong> {selectedPluginData.manifest.author}
                  </div>
                  <div className="plugin-detail-id">
                    <strong>ID:</strong> {selectedPluginData.manifest.id}
                  </div>
                </div>

                <div className="plugin-detail-description">
                  {selectedPluginData.manifest.description || 'No description provided.'}
                </div>

                <div className="plugin-detail-section">
                  <h4>Permissions</h4>
                  {selectedPluginData.manifest.permissions.length === 0 ? (
                    <p className="plugin-detail-empty">No permissions required</p>
                  ) : (
                    <ul className="plugin-permissions-list">
                      {selectedPluginData.manifest.permissions.map((permission) => (
                        <li
                          key={permission}
                          className={`plugin-permission ${
                            isSensitivePermission(permission) ? 'sensitive' : ''
                          }`}
                        >
                          {getPermissionLabel(permission)}
                          {isSensitivePermission(permission) && (
                            <span className="permission-warning" title="Sensitive permission">
                              !
                            </span>
                          )}
                        </li>
                      ))}
                    </ul>
                  )}
                </div>

                <div className="plugin-detail-section">
                  <h4>Activation</h4>
                  <ul className="plugin-activation-list">
                    {selectedPluginData.manifest.activation_events.map((event, index) => (
                      <li key={index}>
                        {event.type === 'on_startup' && 'Activates on startup'}
                        {event.type === 'on_command' && `Activates on command: ${event.value}`}
                        {event.type === 'on_document_open' &&
                          `Activates for documents: ${event.value}`}
                        {event.type === 'on_language' && `Activates for language: ${event.value}`}
                      </li>
                    ))}
                  </ul>
                </div>

                <div className="plugin-detail-actions">
                  {onOpenSettings && (
                    <button
                      className="plugin-action-btn"
                      onClick={() => onOpenSettings(selectedPluginData.manifest.id)}
                    >
                      Settings
                    </button>
                  )}
                  <button
                    className="plugin-action-btn plugin-action-danger"
                    onClick={() => setConfirmUninstall(selectedPluginData.manifest.id)}
                  >
                    Uninstall
                  </button>
                </div>
              </>
            ) : (
              <div className="plugin-detail-placeholder">
                Select a plugin to view details
              </div>
            )}
          </div>
        </div>

        {/* Uninstall confirmation dialog */}
        {confirmUninstall && (
          <div className="plugin-confirm-overlay">
            <div className="plugin-confirm-dialog">
              <h3>Uninstall Plugin?</h3>
              <p>
                Are you sure you want to uninstall{' '}
                <strong>
                  {plugins.find((p) => p.manifest.id === confirmUninstall)?.manifest.name}
                </strong>
                ? This action cannot be undone.
              </p>
              <div className="plugin-confirm-actions">
                <button
                  className="plugin-action-btn"
                  onClick={() => setConfirmUninstall(null)}
                >
                  Cancel
                </button>
                <button
                  className="plugin-action-btn plugin-action-danger"
                  onClick={() => handleUninstall(confirmUninstall)}
                >
                  Uninstall
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
