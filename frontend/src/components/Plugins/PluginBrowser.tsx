/**
 * Plugin Browser Component
 *
 * Browse and install plugins from a marketplace (future feature).
 * Currently provides a placeholder UI for:
 * - Browsing available plugins
 * - Searching plugins
 * - Viewing plugin details
 * - Installing plugins
 */

import { useState, useEffect, useCallback } from 'react';
import { Permission, PluginManifest } from './PluginManager';
import './Plugins.css';

/** Available plugin from marketplace */
export interface AvailablePlugin {
  manifest: PluginManifest;
  downloadUrl: string;
  downloadCount: number;
  rating: number;
  ratingCount: number;
  screenshots: string[];
  changelog?: string;
  isInstalled: boolean;
}

/** Plugin category */
export interface PluginCategory {
  id: string;
  name: string;
  icon?: string;
}

interface PluginBrowserProps {
  isOpen: boolean;
  onClose: () => void;
  availablePlugins: AvailablePlugin[];
  categories: PluginCategory[];
  onInstallPlugin: (downloadUrl: string) => Promise<void>;
  installedPluginIds: string[];
  isLoading?: boolean;
  error?: string | null;
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

/** Format download count */
function formatDownloadCount(count: number): string {
  if (count >= 1000000) {
    return `${(count / 1000000).toFixed(1)}M`;
  }
  if (count >= 1000) {
    return `${(count / 1000).toFixed(1)}K`;
  }
  return count.toString();
}

/** Render star rating */
function StarRating({ rating, count }: { rating: number; count: number }) {
  const fullStars = Math.floor(rating);
  const hasHalfStar = rating - fullStars >= 0.5;
  const emptyStars = 5 - fullStars - (hasHalfStar ? 1 : 0);

  return (
    <div className="plugin-rating">
      {Array(fullStars)
        .fill(0)
        .map((_, i) => (
          <span key={`full-${i}`} className="star full">
            *
          </span>
        ))}
      {hasHalfStar && <span className="star half">*</span>}
      {Array(emptyStars)
        .fill(0)
        .map((_, i) => (
          <span key={`empty-${i}`} className="star empty">
            *
          </span>
        ))}
      <span className="rating-count">({count})</span>
    </div>
  );
}

export function PluginBrowser({
  isOpen,
  onClose,
  availablePlugins,
  categories,
  onInstallPlugin,
  installedPluginIds,
  isLoading = false,
  error = null,
}: PluginBrowserProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [selectedPlugin, setSelectedPlugin] = useState<string | null>(null);
  const [installing, setInstalling] = useState<string | null>(null);
  const [installError, setInstallError] = useState<string | null>(null);

  // Close on escape
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        if (selectedPlugin) {
          setSelectedPlugin(null);
        } else {
          onClose();
        }
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose, selectedPlugin]);

  // Filter plugins
  const filteredPlugins = availablePlugins.filter((plugin) => {
    const matchesSearch =
      searchQuery === '' ||
      plugin.manifest.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      plugin.manifest.description.toLowerCase().includes(searchQuery.toLowerCase());

    // For now, categories aren't implemented in the manifest, so we skip category filtering
    const matchesCategory = selectedCategory === null;

    return matchesSearch && matchesCategory;
  });

  const handleInstall = useCallback(
    async (plugin: AvailablePlugin) => {
      setInstalling(plugin.manifest.id);
      setInstallError(null);

      try {
        await onInstallPlugin(plugin.downloadUrl);
      } catch (err) {
        setInstallError(err instanceof Error ? err.message : 'Installation failed');
      } finally {
        setInstalling(null);
      }
    },
    [onInstallPlugin]
  );

  const selectedPluginData = selectedPlugin
    ? availablePlugins.find((p) => p.manifest.id === selectedPlugin)
    : null;

  if (!isOpen) return null;

  return (
    <div className="plugin-browser-overlay" onClick={onClose}>
      <div
        className="plugin-browser-dialog"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-labelledby="plugin-browser-title"
        aria-modal="true"
      >
        <header className="plugin-browser-header">
          <h2 id="plugin-browser-title">Plugin Marketplace</h2>
          <button
            className="plugin-browser-close"
            onClick={onClose}
            aria-label="Close plugin browser"
          >
            x
          </button>
        </header>

        <div className="plugin-browser-content">
          {/* Sidebar with categories */}
          <nav className="plugin-browser-sidebar">
            <div className="plugin-browser-search">
              <input
                type="search"
                placeholder="Search plugins..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                aria-label="Search plugins"
              />
            </div>

            <ul className="plugin-category-list">
              <li
                className={`plugin-category-item ${selectedCategory === null ? 'active' : ''}`}
                onClick={() => setSelectedCategory(null)}
              >
                All Plugins
              </li>
              {categories.map((category) => (
                <li
                  key={category.id}
                  className={`plugin-category-item ${
                    selectedCategory === category.id ? 'active' : ''
                  }`}
                  onClick={() => setSelectedCategory(category.id)}
                >
                  {category.icon && <span className="category-icon">{category.icon}</span>}
                  {category.name}
                </li>
              ))}
            </ul>
          </nav>

          {/* Main content area */}
          <div className="plugin-browser-main">
            {isLoading ? (
              <div className="plugin-browser-loading">Loading plugins...</div>
            ) : error ? (
              <div className="plugin-browser-error">
                <p>Failed to load plugins</p>
                <p className="error-message">{error}</p>
              </div>
            ) : selectedPluginData ? (
              /* Plugin detail view */
              <div className="plugin-browser-detail">
                <button
                  className="plugin-detail-back"
                  onClick={() => setSelectedPlugin(null)}
                >
                  Back to list
                </button>

                <div className="plugin-detail-header">
                  <div className="plugin-detail-title">
                    <h3>{selectedPluginData.manifest.name}</h3>
                    <span className="plugin-detail-version">
                      v{selectedPluginData.manifest.version}
                    </span>
                  </div>
                  <div className="plugin-detail-author">
                    by {selectedPluginData.manifest.author}
                  </div>
                  <StarRating
                    rating={selectedPluginData.rating}
                    count={selectedPluginData.ratingCount}
                  />
                  <div className="plugin-detail-downloads">
                    {formatDownloadCount(selectedPluginData.downloadCount)} downloads
                  </div>
                </div>

                <div className="plugin-detail-description">
                  {selectedPluginData.manifest.description}
                </div>

                {selectedPluginData.screenshots.length > 0 && (
                  <div className="plugin-detail-screenshots">
                    {selectedPluginData.screenshots.map((url, index) => (
                      <img
                        key={index}
                        src={url}
                        alt={`Screenshot ${index + 1}`}
                        className="plugin-screenshot"
                      />
                    ))}
                  </div>
                )}

                <div className="plugin-detail-section">
                  <h4>Required Permissions</h4>
                  {selectedPluginData.manifest.permissions.length === 0 ? (
                    <p>This plugin doesn't require any special permissions.</p>
                  ) : (
                    <ul className="plugin-permissions-list">
                      {selectedPluginData.manifest.permissions.map((permission) => (
                        <li key={permission} className="plugin-permission">
                          {getPermissionLabel(permission)}
                        </li>
                      ))}
                    </ul>
                  )}
                </div>

                {selectedPluginData.changelog && (
                  <div className="plugin-detail-section">
                    <h4>Changelog</h4>
                    <pre className="plugin-changelog">{selectedPluginData.changelog}</pre>
                  </div>
                )}

                <div className="plugin-detail-actions">
                  {installError && (
                    <div className="plugin-install-error">{installError}</div>
                  )}
                  {installedPluginIds.includes(selectedPluginData.manifest.id) ? (
                    <span className="plugin-installed-badge">Installed</span>
                  ) : (
                    <button
                      className="plugin-install-btn"
                      onClick={() => handleInstall(selectedPluginData)}
                      disabled={installing === selectedPluginData.manifest.id}
                    >
                      {installing === selectedPluginData.manifest.id
                        ? 'Installing...'
                        : 'Install'}
                    </button>
                  )}
                </div>
              </div>
            ) : (
              /* Plugin grid view */
              <div className="plugin-browser-grid">
                {filteredPlugins.length === 0 ? (
                  <div className="plugin-browser-empty">
                    {searchQuery
                      ? 'No plugins match your search'
                      : 'No plugins available'}
                  </div>
                ) : (
                  filteredPlugins.map((plugin) => (
                    <div
                      key={plugin.manifest.id}
                      className="plugin-card"
                      onClick={() => setSelectedPlugin(plugin.manifest.id)}
                      role="button"
                      tabIndex={0}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter' || e.key === ' ') {
                          setSelectedPlugin(plugin.manifest.id);
                        }
                      }}
                    >
                      <div className="plugin-card-header">
                        <h4 className="plugin-card-name">{plugin.manifest.name}</h4>
                        <span className="plugin-card-version">
                          v{plugin.manifest.version}
                        </span>
                      </div>
                      <div className="plugin-card-author">
                        by {plugin.manifest.author}
                      </div>
                      <p className="plugin-card-description">
                        {plugin.manifest.description}
                      </p>
                      <div className="plugin-card-footer">
                        <StarRating rating={plugin.rating} count={plugin.ratingCount} />
                        <span className="plugin-card-downloads">
                          {formatDownloadCount(plugin.downloadCount)}
                        </span>
                      </div>
                      {installedPluginIds.includes(plugin.manifest.id) && (
                        <span className="plugin-card-installed">Installed</span>
                      )}
                    </div>
                  ))
                )}
              </div>
            )}
          </div>
        </div>

        <footer className="plugin-browser-footer">
          <p className="plugin-browser-disclaimer">
            Plugins are provided by third-party developers. Please review permissions
            carefully before installing.
          </p>
        </footer>
      </div>
    </div>
  );
}
