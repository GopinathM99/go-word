/**
 * Settings Context - Provides application settings throughout the app
 *
 * This context handles:
 * - Loading settings from the Rust backend on app start
 * - Providing settings to all components
 * - Updating settings and persisting changes
 * - Applying theme changes to the UI
 */

import {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  ReactNode,
} from 'react';
import { invoke } from '@tauri-apps/api/core';
import { AppSettings, DEFAULT_SETTINGS, Theme } from '../lib/types';

interface SettingsContextValue {
  /** Current application settings */
  settings: AppSettings;
  /** Whether settings are currently loading */
  loading: boolean;
  /** Error message if settings failed to load */
  error: string | null;
  /** Update all settings */
  updateSettings: (settings: AppSettings) => Promise<void>;
  /** Update only general settings */
  updateGeneralSettings: (general: AppSettings['general']) => Promise<void>;
  /** Update only editing settings */
  updateEditingSettings: (editing: AppSettings['editing']) => Promise<void>;
  /** Update only privacy settings */
  updatePrivacySettings: (privacy: AppSettings['privacy']) => Promise<void>;
  /** Reset all settings to defaults */
  resetSettings: () => Promise<void>;
  /** Reload settings from backend */
  reloadSettings: () => Promise<void>;
}

const SettingsContext = createContext<SettingsContextValue | null>(null);

interface SettingsProviderProps {
  children: ReactNode;
}

/**
 * Apply theme to the document body
 */
function applyTheme(theme: Theme) {
  const root = document.documentElement;

  // Remove existing theme classes
  root.classList.remove('theme-light', 'theme-dark');

  if (theme === 'system') {
    // Check system preference
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    root.classList.add(prefersDark ? 'theme-dark' : 'theme-light');
  } else {
    root.classList.add(`theme-${theme}`);
  }
}

export function SettingsProvider({ children }: SettingsProviderProps) {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Load settings on mount
  useEffect(() => {
    loadSettings();
  }, []);

  // Apply theme when settings change
  useEffect(() => {
    applyTheme(settings.general.theme);
  }, [settings.general.theme]);

  // Listen for system theme changes when using 'system' theme
  useEffect(() => {
    if (settings.general.theme !== 'system') return;

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = () => applyTheme('system');

    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  }, [settings.general.theme]);

  const loadSettings = async () => {
    setLoading(true);
    setError(null);

    try {
      const loadedSettings = await invoke<AppSettings>('get_settings');
      setSettings(loadedSettings);
    } catch (e) {
      console.error('Failed to load settings:', e);
      setError(String(e));
      // Use defaults on error
      setSettings(DEFAULT_SETTINGS);
    } finally {
      setLoading(false);
    }
  };

  const updateSettings = useCallback(async (newSettings: AppSettings) => {
    try {
      await invoke('update_settings', { settings: newSettings });
      setSettings(newSettings);
    } catch (e) {
      console.error('Failed to update settings:', e);
      throw e;
    }
  }, []);

  const updateGeneralSettings = useCallback(
    async (general: AppSettings['general']) => {
      const newSettings = { ...settings, general };
      await updateSettings(newSettings);
    },
    [settings, updateSettings]
  );

  const updateEditingSettings = useCallback(
    async (editing: AppSettings['editing']) => {
      const newSettings = { ...settings, editing };
      await updateSettings(newSettings);
    },
    [settings, updateSettings]
  );

  const updatePrivacySettings = useCallback(
    async (privacy: AppSettings['privacy']) => {
      const newSettings = { ...settings, privacy };
      await updateSettings(newSettings);
    },
    [settings, updateSettings]
  );

  const resetSettings = useCallback(async () => {
    try {
      const defaultSettings = await invoke<AppSettings>('reset_settings');
      setSettings(defaultSettings);
    } catch (e) {
      console.error('Failed to reset settings:', e);
      throw e;
    }
  }, []);

  const reloadSettings = useCallback(async () => {
    await loadSettings();
  }, []);

  return (
    <SettingsContext.Provider
      value={{
        settings,
        loading,
        error,
        updateSettings,
        updateGeneralSettings,
        updateEditingSettings,
        updatePrivacySettings,
        resetSettings,
        reloadSettings,
      }}
    >
      {children}
    </SettingsContext.Provider>
  );
}

/**
 * Hook to access settings context
 */
export function useSettings(): SettingsContextValue {
  const context = useContext(SettingsContext);
  if (!context) {
    throw new Error('useSettings must be used within a SettingsProvider');
  }
  return context;
}
