/**
 * Settings Dialog - Modal for editing application settings
 *
 * Features:
 * - Tabbed interface (General, Editing, Privacy)
 * - Save/Cancel/Apply buttons
 * - Reset to Defaults button
 * - Live preview for some settings (theme)
 */

import { useState, useEffect, useCallback } from 'react';
import { useSettings } from '../contexts/SettingsContext';
import {
  AppSettings,
  Theme,
  Language,
  LANGUAGE_OPTIONS,
  FONT_FAMILIES,
} from '../lib/types';
import './SettingsDialog.css';

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

type TabId = 'general' | 'editing' | 'privacy';

export function SettingsDialog({ isOpen, onClose }: SettingsDialogProps) {
  const { settings, updateSettings, resetSettings } = useSettings();
  const [activeTab, setActiveTab] = useState<TabId>('general');
  const [localSettings, setLocalSettings] = useState<AppSettings>(settings);
  const [hasChanges, setHasChanges] = useState(false);
  const [saving, setSaving] = useState(false);

  // Reset local settings when dialog opens
  useEffect(() => {
    if (isOpen) {
      setLocalSettings(settings);
      setHasChanges(false);
    }
  }, [isOpen, settings]);

  // Track changes
  useEffect(() => {
    const changed = JSON.stringify(localSettings) !== JSON.stringify(settings);
    setHasChanges(changed);
  }, [localSettings, settings]);

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

  const handleSave = useCallback(async () => {
    setSaving(true);
    try {
      await updateSettings(localSettings);
      onClose();
    } catch (e) {
      console.error('Failed to save settings:', e);
      alert('Failed to save settings. Please try again.');
    } finally {
      setSaving(false);
    }
  }, [localSettings, updateSettings, onClose]);

  const handleApply = useCallback(async () => {
    setSaving(true);
    try {
      await updateSettings(localSettings);
    } catch (e) {
      console.error('Failed to apply settings:', e);
      alert('Failed to apply settings. Please try again.');
    } finally {
      setSaving(false);
    }
  }, [localSettings, updateSettings]);

  const handleReset = useCallback(async () => {
    if (confirm('Are you sure you want to reset all settings to defaults?')) {
      try {
        await resetSettings();
        setLocalSettings(settings);
      } catch (e) {
        console.error('Failed to reset settings:', e);
        alert('Failed to reset settings. Please try again.');
      }
    }
  }, [resetSettings, settings]);

  const updateGeneral = useCallback(
    (updates: Partial<AppSettings['general']>) => {
      setLocalSettings((prev) => ({
        ...prev,
        general: { ...prev.general, ...updates },
      }));
    },
    []
  );

  const updateEditing = useCallback(
    (updates: Partial<AppSettings['editing']>) => {
      setLocalSettings((prev) => ({
        ...prev,
        editing: { ...prev.editing, ...updates },
      }));
    },
    []
  );

  const updatePrivacy = useCallback(
    (updates: Partial<AppSettings['privacy']>) => {
      setLocalSettings((prev) => ({
        ...prev,
        privacy: { ...prev.privacy, ...updates },
      }));
    },
    []
  );

  if (!isOpen) return null;

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div
        className="settings-dialog"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-labelledby="settings-title"
        aria-modal="true"
      >
        <header className="settings-header">
          <h2 id="settings-title">Settings</h2>
          <button
            className="settings-close-btn"
            onClick={onClose}
            aria-label="Close settings"
          >
            x
          </button>
        </header>

        <div className="settings-body">
          <nav className="settings-tabs" role="tablist">
            <button
              role="tab"
              aria-selected={activeTab === 'general'}
              aria-controls="panel-general"
              className={`settings-tab ${activeTab === 'general' ? 'active' : ''}`}
              onClick={() => setActiveTab('general')}
            >
              General
            </button>
            <button
              role="tab"
              aria-selected={activeTab === 'editing'}
              aria-controls="panel-editing"
              className={`settings-tab ${activeTab === 'editing' ? 'active' : ''}`}
              onClick={() => setActiveTab('editing')}
            >
              Editing
            </button>
            <button
              role="tab"
              aria-selected={activeTab === 'privacy'}
              aria-controls="panel-privacy"
              className={`settings-tab ${activeTab === 'privacy' ? 'active' : ''}`}
              onClick={() => setActiveTab('privacy')}
            >
              Privacy
            </button>
          </nav>

          <div className="settings-content">
            {/* General Tab */}
            <div
              id="panel-general"
              role="tabpanel"
              aria-labelledby="tab-general"
              hidden={activeTab !== 'general'}
              className="settings-panel"
            >
              <div className="settings-group">
                <label className="settings-label" htmlFor="language">
                  Language
                </label>
                <select
                  id="language"
                  className="settings-select"
                  value={localSettings.general.language}
                  onChange={(e) =>
                    updateGeneral({ language: e.target.value as Language })
                  }
                >
                  {Object.entries(LANGUAGE_OPTIONS).map(([code, name]) => (
                    <option key={code} value={code}>
                      {name}
                    </option>
                  ))}
                </select>
                <p className="settings-description">
                  Set the display language for the application interface.
                </p>
              </div>

              <div className="settings-group">
                <label className="settings-label" htmlFor="theme">
                  Theme
                </label>
                <select
                  id="theme"
                  className="settings-select"
                  value={localSettings.general.theme}
                  onChange={(e) =>
                    updateGeneral({ theme: e.target.value as Theme })
                  }
                >
                  <option value="light">Light</option>
                  <option value="dark">Dark</option>
                  <option value="system">System (Auto)</option>
                </select>
                <p className="settings-description">
                  Choose the color theme for the application.
                </p>
              </div>

              <div className="settings-group">
                <label className="settings-label" htmlFor="recent-files">
                  Recent Files Count
                </label>
                <input
                  id="recent-files"
                  type="number"
                  className="settings-input"
                  min={0}
                  max={50}
                  value={localSettings.general.recent_files_count}
                  onChange={(e) =>
                    updateGeneral({
                      recent_files_count: Math.max(
                        0,
                        Math.min(50, parseInt(e.target.value) || 0)
                      ),
                    })
                  }
                />
                <p className="settings-description">
                  Number of recent files to show in the File menu (0-50).
                </p>
              </div>
            </div>

            {/* Editing Tab */}
            <div
              id="panel-editing"
              role="tabpanel"
              aria-labelledby="tab-editing"
              hidden={activeTab !== 'editing'}
              className="settings-panel"
            >
              <div className="settings-group">
                <div className="settings-toggle-row">
                  <label className="settings-label" htmlFor="autosave">
                    Autosave
                  </label>
                  <label className="settings-toggle">
                    <input
                      id="autosave"
                      type="checkbox"
                      checked={localSettings.editing.autosave_enabled}
                      onChange={(e) =>
                        updateEditing({ autosave_enabled: e.target.checked })
                      }
                    />
                    <span className="settings-toggle-slider"></span>
                  </label>
                </div>
                <p className="settings-description">
                  Automatically save documents at regular intervals.
                </p>
              </div>

              <div className="settings-group">
                <label className="settings-label" htmlFor="autosave-interval">
                  Autosave Interval (seconds)
                </label>
                <input
                  id="autosave-interval"
                  type="number"
                  className="settings-input"
                  min={10}
                  max={600}
                  value={localSettings.editing.autosave_interval_seconds}
                  onChange={(e) =>
                    updateEditing({
                      autosave_interval_seconds: Math.max(
                        10,
                        Math.min(600, parseInt(e.target.value) || 60)
                      ),
                    })
                  }
                  disabled={!localSettings.editing.autosave_enabled}
                />
                <p className="settings-description">
                  How often to autosave (10-600 seconds).
                </p>
              </div>

              <div className="settings-group">
                <label className="settings-label" htmlFor="default-font">
                  Default Font Family
                </label>
                <select
                  id="default-font"
                  className="settings-select"
                  value={localSettings.editing.default_font_family}
                  onChange={(e) =>
                    updateEditing({ default_font_family: e.target.value })
                  }
                >
                  {FONT_FAMILIES.map((font) => (
                    <option key={font} value={font} style={{ fontFamily: font }}>
                      {font}
                    </option>
                  ))}
                </select>
                <p className="settings-description">
                  Default font for new documents.
                </p>
              </div>

              <div className="settings-group">
                <label className="settings-label" htmlFor="default-font-size">
                  Default Font Size (pt)
                </label>
                <input
                  id="default-font-size"
                  type="number"
                  className="settings-input"
                  min={6}
                  max={72}
                  step={0.5}
                  value={localSettings.editing.default_font_size}
                  onChange={(e) =>
                    updateEditing({
                      default_font_size: Math.max(
                        6,
                        Math.min(72, parseFloat(e.target.value) || 12)
                      ),
                    })
                  }
                />
                <p className="settings-description">
                  Default font size for new documents (6-72 pt).
                </p>
              </div>

              <div className="settings-group">
                <div className="settings-toggle-row">
                  <label className="settings-label" htmlFor="spelling">
                    Show Spelling Errors
                  </label>
                  <label className="settings-toggle">
                    <input
                      id="spelling"
                      type="checkbox"
                      checked={localSettings.editing.show_spelling_errors}
                      onChange={(e) =>
                        updateEditing({ show_spelling_errors: e.target.checked })
                      }
                    />
                    <span className="settings-toggle-slider"></span>
                  </label>
                </div>
                <p className="settings-description">
                  Underline spelling errors with a red wavy line.
                </p>
              </div>

              <div className="settings-group">
                <div className="settings-toggle-row">
                  <label className="settings-label" htmlFor="grammar">
                    Show Grammar Errors
                  </label>
                  <label className="settings-toggle">
                    <input
                      id="grammar"
                      type="checkbox"
                      checked={localSettings.editing.show_grammar_errors}
                      onChange={(e) =>
                        updateEditing({ show_grammar_errors: e.target.checked })
                      }
                    />
                    <span className="settings-toggle-slider"></span>
                  </label>
                </div>
                <p className="settings-description">
                  Underline grammar errors with a blue wavy line.
                </p>
              </div>
            </div>

            {/* Privacy Tab */}
            <div
              id="panel-privacy"
              role="tabpanel"
              aria-labelledby="tab-privacy"
              hidden={activeTab !== 'privacy'}
              className="settings-panel"
            >
              <div className="settings-group">
                <div className="settings-toggle-row">
                  <label className="settings-label" htmlFor="telemetry">
                    Send Anonymous Usage Data
                  </label>
                  <label className="settings-toggle">
                    <input
                      id="telemetry"
                      type="checkbox"
                      checked={localSettings.privacy.telemetry_enabled}
                      onChange={(e) =>
                        updatePrivacy({ telemetry_enabled: e.target.checked })
                      }
                    />
                    <span className="settings-toggle-slider"></span>
                  </label>
                </div>
                <p className="settings-description">
                  Help improve the application by sending anonymous usage
                  statistics. No personal data or document content is collected.
                </p>
              </div>

              <div className="settings-group">
                <div className="settings-toggle-row">
                  <label className="settings-label" htmlFor="crash-reports">
                    Send Crash Reports
                  </label>
                  <label className="settings-toggle">
                    <input
                      id="crash-reports"
                      type="checkbox"
                      checked={localSettings.privacy.crash_reports_enabled}
                      onChange={(e) =>
                        updatePrivacy({ crash_reports_enabled: e.target.checked })
                      }
                    />
                    <span className="settings-toggle-slider"></span>
                  </label>
                </div>
                <p className="settings-description">
                  Automatically send crash reports to help diagnose and fix
                  issues. Reports include technical information about the crash
                  but no personal data.
                </p>
              </div>

              <div className="settings-info-box">
                <h4>Your Privacy Matters</h4>
                <p>
                  We take your privacy seriously. Any data collected is used
                  solely to improve the application and is never sold or shared
                  with third parties.
                </p>
              </div>
            </div>
          </div>
        </div>

        <footer className="settings-footer">
          <button
            className="settings-btn settings-btn-secondary"
            onClick={handleReset}
          >
            Reset to Defaults
          </button>
          <div className="settings-footer-right">
            <button
              className="settings-btn settings-btn-secondary"
              onClick={onClose}
            >
              Cancel
            </button>
            <button
              className="settings-btn settings-btn-secondary"
              onClick={handleApply}
              disabled={!hasChanges || saving}
            >
              Apply
            </button>
            <button
              className="settings-btn settings-btn-primary"
              onClick={handleSave}
              disabled={saving}
            >
              {saving ? 'Saving...' : 'Save'}
            </button>
          </div>
        </footer>
      </div>
    </div>
  );
}
