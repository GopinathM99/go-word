/**
 * Privacy Settings Component
 *
 * Provides UI for configuring telemetry and privacy options.
 * Features:
 * - Master telemetry toggle
 * - Individual category toggles (crash reports, performance, usage)
 * - Clear explanation of what each setting does
 * - Privacy-first design with opt-in approach
 */

import { useState, useCallback } from 'react';
import './PrivacySettings.css';

export interface TelemetryPrivacySettings {
  telemetry_enabled: boolean;
  crash_reports_enabled: boolean;
  performance_metrics_enabled: boolean;
  usage_analytics_enabled: boolean;
}

interface PrivacySettingsProps {
  settings: TelemetryPrivacySettings;
  onChange: (settings: TelemetryPrivacySettings) => void;
  disabled?: boolean;
}

export function PrivacySettings({
  settings,
  onChange,
  disabled = false,
}: PrivacySettingsProps) {
  const handleMasterToggle = useCallback(
    (enabled: boolean) => {
      onChange({
        ...settings,
        telemetry_enabled: enabled,
        // When disabling master, disable all sub-settings
        ...(enabled
          ? {}
          : {
              crash_reports_enabled: false,
              performance_metrics_enabled: false,
              usage_analytics_enabled: false,
            }),
      });
    },
    [settings, onChange]
  );

  const handleSettingToggle = useCallback(
    (key: keyof TelemetryPrivacySettings, value: boolean) => {
      const newSettings = { ...settings, [key]: value };

      // Auto-enable master if any sub-setting is enabled
      if (value && !settings.telemetry_enabled) {
        newSettings.telemetry_enabled = true;
      }

      // Auto-disable master if all sub-settings are disabled
      if (
        !value &&
        !newSettings.crash_reports_enabled &&
        !newSettings.performance_metrics_enabled &&
        !newSettings.usage_analytics_enabled
      ) {
        newSettings.telemetry_enabled = false;
      }

      onChange(newSettings);
    },
    [settings, onChange]
  );

  const subSettingsDisabled = disabled || !settings.telemetry_enabled;

  return (
    <div className="privacy-settings">
      <div className="privacy-settings-header">
        <h3>Privacy and Telemetry</h3>
        <p className="privacy-settings-description">
          Help improve the application by sharing anonymous usage data. Your
          privacy is important to us - no personal data or document content is
          ever collected.
        </p>
      </div>

      {/* Master Toggle */}
      <div className="privacy-setting-item privacy-setting-master">
        <div className="privacy-setting-info">
          <label htmlFor="telemetry-master" className="privacy-setting-label">
            Enable Telemetry
          </label>
          <p className="privacy-setting-help">
            Master switch for all telemetry collection. When disabled, no data
            is collected or sent.
          </p>
        </div>
        <div className="privacy-setting-control">
          <label className="privacy-toggle">
            <input
              id="telemetry-master"
              type="checkbox"
              checked={settings.telemetry_enabled}
              onChange={(e) => handleMasterToggle(e.target.checked)}
              disabled={disabled}
            />
            <span className="privacy-toggle-slider"></span>
          </label>
        </div>
      </div>

      <div
        className={`privacy-sub-settings ${subSettingsDisabled ? 'disabled' : ''}`}
      >
        {/* Crash Reports */}
        <div className="privacy-setting-item">
          <div className="privacy-setting-info">
            <label
              htmlFor="crash-reports"
              className="privacy-setting-label"
            >
              Crash Reports
            </label>
            <p className="privacy-setting-help">
              Automatically send error and crash reports to help identify and
              fix issues. Reports include technical information about the crash
              but never include your document content.
            </p>
          </div>
          <div className="privacy-setting-control">
            <label className="privacy-toggle">
              <input
                id="crash-reports"
                type="checkbox"
                checked={settings.crash_reports_enabled}
                onChange={(e) =>
                  handleSettingToggle('crash_reports_enabled', e.target.checked)
                }
                disabled={subSettingsDisabled}
              />
              <span className="privacy-toggle-slider"></span>
            </label>
          </div>
        </div>

        {/* Performance Metrics */}
        <div className="privacy-setting-item">
          <div className="privacy-setting-info">
            <label
              htmlFor="perf-metrics"
              className="privacy-setting-label"
            >
              Performance Metrics
            </label>
            <p className="privacy-setting-help">
              Collect performance data like response times and memory usage.
              This helps us optimize the application for better performance on
              all devices.
            </p>
          </div>
          <div className="privacy-setting-control">
            <label className="privacy-toggle">
              <input
                id="perf-metrics"
                type="checkbox"
                checked={settings.performance_metrics_enabled}
                onChange={(e) =>
                  handleSettingToggle(
                    'performance_metrics_enabled',
                    e.target.checked
                  )
                }
                disabled={subSettingsDisabled}
              />
              <span className="privacy-toggle-slider"></span>
            </label>
          </div>
        </div>

        {/* Usage Analytics */}
        <div className="privacy-setting-item">
          <div className="privacy-setting-info">
            <label
              htmlFor="usage-analytics"
              className="privacy-setting-label"
            >
              Usage Analytics
            </label>
            <p className="privacy-setting-help">
              Share anonymous usage statistics like which features are used most
              often. This helps us prioritize improvements and new features.
            </p>
          </div>
          <div className="privacy-setting-control">
            <label className="privacy-toggle">
              <input
                id="usage-analytics"
                type="checkbox"
                checked={settings.usage_analytics_enabled}
                onChange={(e) =>
                  handleSettingToggle(
                    'usage_analytics_enabled',
                    e.target.checked
                  )
                }
                disabled={subSettingsDisabled}
              />
              <span className="privacy-toggle-slider"></span>
            </label>
          </div>
        </div>
      </div>

      <div className="privacy-info-box">
        <div className="privacy-info-icon">
          <svg viewBox="0 0 24 24" width="24" height="24">
            <path
              fill="currentColor"
              d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"
            />
          </svg>
        </div>
        <div className="privacy-info-content">
          <h4>Your Privacy Matters</h4>
          <ul>
            <li>All telemetry is opt-in and disabled by default</li>
            <li>No personal information or document content is ever collected</li>
            <li>Data is anonymized and cannot be traced back to you</li>
            <li>You can change these settings at any time</li>
          </ul>
        </div>
      </div>
    </div>
  );
}

/**
 * Consent Dialog Component
 *
 * Shown to first-time users to explain telemetry and request consent.
 */
interface TelemetryConsentDialogProps {
  isOpen: boolean;
  onAcceptAll: () => void;
  onAcceptMinimal: () => void;
  onDecline: () => void;
}

export function TelemetryConsentDialog({
  isOpen,
  onAcceptAll,
  onAcceptMinimal,
  onDecline,
}: TelemetryConsentDialogProps) {
  if (!isOpen) return null;

  return (
    <div className="consent-dialog-overlay">
      <div
        className="consent-dialog"
        role="dialog"
        aria-labelledby="consent-title"
        aria-describedby="consent-description"
        aria-modal="true"
      >
        <div className="consent-dialog-header">
          <div className="consent-dialog-icon">
            <svg viewBox="0 0 24 24" width="48" height="48">
              <path
                fill="currentColor"
                d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"
              />
            </svg>
          </div>
          <h2 id="consent-title">Help Improve the Application</h2>
        </div>

        <div className="consent-dialog-body" id="consent-description">
          <p>
            We would like to collect anonymous usage data to help improve the
            application. This includes:
          </p>

          <div className="consent-categories">
            <div className="consent-category">
              <h4>Crash Reports</h4>
              <p>
                Automatic error reports when something goes wrong, helping us
                fix bugs faster.
              </p>
            </div>

            <div className="consent-category">
              <h4>Performance Metrics</h4>
              <p>
                Response times and resource usage to optimize performance on all
                devices.
              </p>
            </div>

            <div className="consent-category">
              <h4>Usage Analytics</h4>
              <p>
                Which features are used most to help prioritize improvements.
              </p>
            </div>
          </div>

          <div className="consent-privacy-note">
            <strong>Your privacy is protected:</strong>
            <ul>
              <li>No personal information is collected</li>
              <li>Document content is never accessed or transmitted</li>
              <li>All data is anonymized</li>
              <li>You can change your preferences anytime in Settings</li>
            </ul>
          </div>
        </div>

        <div className="consent-dialog-footer">
          <button
            className="consent-btn consent-btn-secondary"
            onClick={onDecline}
          >
            No Thanks
          </button>
          <button
            className="consent-btn consent-btn-secondary"
            onClick={onAcceptMinimal}
          >
            Crash Reports Only
          </button>
          <button
            className="consent-btn consent-btn-primary"
            onClick={onAcceptAll}
          >
            Accept All
          </button>
        </div>
      </div>
    </div>
  );
}

/**
 * Hook for managing telemetry consent state
 */
export function useTelemetryConsent() {
  const [hasShownConsent, setHasShownConsent] = useState(() => {
    // Check localStorage for consent preference
    if (typeof window !== 'undefined') {
      return localStorage.getItem('telemetry_consent_shown') === 'true';
    }
    return false;
  });

  const [settings, setSettings] = useState<TelemetryPrivacySettings>(() => {
    if (typeof window !== 'undefined') {
      const stored = localStorage.getItem('telemetry_settings');
      if (stored) {
        try {
          return JSON.parse(stored);
        } catch {
          // Fall through to default
        }
      }
    }
    return {
      telemetry_enabled: false,
      crash_reports_enabled: false,
      performance_metrics_enabled: false,
      usage_analytics_enabled: false,
    };
  });

  const markConsentShown = useCallback(() => {
    setHasShownConsent(true);
    if (typeof window !== 'undefined') {
      localStorage.setItem('telemetry_consent_shown', 'true');
    }
  }, []);

  const updateSettings = useCallback((newSettings: TelemetryPrivacySettings) => {
    setSettings(newSettings);
    if (typeof window !== 'undefined') {
      localStorage.setItem('telemetry_settings', JSON.stringify(newSettings));
    }
  }, []);

  const acceptAll = useCallback(() => {
    updateSettings({
      telemetry_enabled: true,
      crash_reports_enabled: true,
      performance_metrics_enabled: true,
      usage_analytics_enabled: true,
    });
    markConsentShown();
  }, [updateSettings, markConsentShown]);

  const acceptMinimal = useCallback(() => {
    updateSettings({
      telemetry_enabled: true,
      crash_reports_enabled: true,
      performance_metrics_enabled: false,
      usage_analytics_enabled: false,
    });
    markConsentShown();
  }, [updateSettings, markConsentShown]);

  const decline = useCallback(() => {
    updateSettings({
      telemetry_enabled: false,
      crash_reports_enabled: false,
      performance_metrics_enabled: false,
      usage_analytics_enabled: false,
    });
    markConsentShown();
  }, [updateSettings, markConsentShown]);

  return {
    settings,
    hasShownConsent,
    updateSettings,
    acceptAll,
    acceptMinimal,
    decline,
    shouldShowConsent: !hasShownConsent,
  };
}

export default PrivacySettings;
