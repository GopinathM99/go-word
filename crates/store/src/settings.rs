//! Application settings management
//!
//! This module provides settings persistence, loading, and updating
//! for the Go Word application.

use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main application settings container
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    /// General application settings
    pub general: GeneralSettings,
    /// Text editing settings
    pub editing: EditingSettings,
    /// Privacy and telemetry settings
    pub privacy: PrivacySettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
            editing: EditingSettings::default(),
            privacy: PrivacySettings::default(),
        }
    }
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralSettings {
    /// UI language code (e.g., "en", "es", "zh", "fr", "de")
    pub language: String,
    /// Application theme
    pub theme: Theme,
    /// Number of recent files to show in the menu
    pub recent_files_count: u8,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            theme: Theme::System,
            recent_files_count: 10,
        }
    }
}

/// Text editing settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EditingSettings {
    /// Whether autosave is enabled
    pub autosave_enabled: bool,
    /// Autosave interval in seconds
    pub autosave_interval_seconds: u32,
    /// Default font family for new documents
    pub default_font_family: String,
    /// Default font size in points
    pub default_font_size: f32,
    /// Whether to show spelling error indicators
    pub show_spelling_errors: bool,
    /// Whether to show grammar error indicators
    pub show_grammar_errors: bool,
}

impl Default for EditingSettings {
    fn default() -> Self {
        Self {
            autosave_enabled: true,
            autosave_interval_seconds: 60,
            default_font_family: "Times New Roman".to_string(),
            default_font_size: 12.0,
            show_spelling_errors: true,
            show_grammar_errors: true,
        }
    }
}

/// Privacy and telemetry settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrivacySettings {
    /// Whether anonymous telemetry collection is enabled
    pub telemetry_enabled: bool,
    /// Whether automatic crash report submission is enabled
    pub crash_reports_enabled: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            telemetry_enabled: false,
            crash_reports_enabled: true,
        }
    }
}

/// Application theme
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for Theme {
    fn default() -> Self {
        Self::System
    }
}

/// Settings manager for loading, saving, and updating application settings
pub struct SettingsManager {
    /// Path to the settings file
    settings_path: PathBuf,
    /// Current settings (cached)
    current: AppSettings,
}

impl SettingsManager {
    /// Create a new settings manager with the given app data directory
    pub fn new(app_data_dir: PathBuf) -> Self {
        let settings_path = app_data_dir.join("settings.json");
        Self {
            settings_path,
            current: AppSettings::default(),
        }
    }

    /// Get the path to the settings file
    pub fn settings_path(&self) -> &PathBuf {
        &self.settings_path
    }

    /// Load settings from disk, or return defaults if file doesn't exist
    pub async fn load(&mut self) -> Result<&AppSettings> {
        if self.settings_path.exists() {
            let content = tokio::fs::read_to_string(&self.settings_path).await?;
            match serde_json::from_str::<AppSettings>(&content) {
                Ok(settings) => {
                    self.current = settings;
                }
                Err(e) => {
                    // Log the error but use defaults
                    tracing::warn!(
                        "Failed to parse settings file, using defaults: {}",
                        e
                    );
                    self.current = AppSettings::default();
                }
            }
        } else {
            self.current = AppSettings::default();
        }
        Ok(&self.current)
    }

    /// Load settings synchronously (for use during app startup)
    pub fn load_sync(&mut self) -> Result<&AppSettings> {
        if self.settings_path.exists() {
            let content = std::fs::read_to_string(&self.settings_path)?;
            match serde_json::from_str::<AppSettings>(&content) {
                Ok(settings) => {
                    self.current = settings;
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse settings file, using defaults: {}",
                        e
                    );
                    self.current = AppSettings::default();
                }
            }
        } else {
            self.current = AppSettings::default();
        }
        Ok(&self.current)
    }

    /// Save current settings to disk
    pub async fn save(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.settings_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(&self.current)?;
        tokio::fs::write(&self.settings_path, content).await?;
        Ok(())
    }

    /// Save settings synchronously
    pub fn save_sync(&self) -> Result<()> {
        if let Some(parent) = self.settings_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.current)?;
        std::fs::write(&self.settings_path, content)?;
        Ok(())
    }

    /// Get current settings
    pub fn get(&self) -> &AppSettings {
        &self.current
    }

    /// Update settings and save to disk
    pub async fn update(&mut self, settings: AppSettings) -> Result<()> {
        self.current = settings;
        self.save().await
    }

    /// Update settings synchronously
    pub fn update_sync(&mut self, settings: AppSettings) -> Result<()> {
        self.current = settings;
        self.save_sync()
    }

    /// Reset settings to defaults and save
    pub async fn reset(&mut self) -> Result<&AppSettings> {
        self.current = AppSettings::default();
        self.save().await?;
        Ok(&self.current)
    }

    /// Reset settings to defaults synchronously
    pub fn reset_sync(&mut self) -> Result<&AppSettings> {
        self.current = AppSettings::default();
        self.save_sync()?;
        Ok(&self.current)
    }

    /// Update only general settings
    pub async fn update_general(&mut self, general: GeneralSettings) -> Result<()> {
        self.current.general = general;
        self.save().await
    }

    /// Update only editing settings
    pub async fn update_editing(&mut self, editing: EditingSettings) -> Result<()> {
        self.current.editing = editing;
        self.save().await
    }

    /// Update only privacy settings
    pub async fn update_privacy(&mut self, privacy: PrivacySettings) -> Result<()> {
        self.current.privacy = privacy;
        self.save().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();

        assert_eq!(settings.general.language, "en");
        assert_eq!(settings.general.theme, Theme::System);
        assert_eq!(settings.general.recent_files_count, 10);

        assert!(settings.editing.autosave_enabled);
        assert_eq!(settings.editing.autosave_interval_seconds, 60);
        assert_eq!(settings.editing.default_font_family, "Times New Roman");
        assert_eq!(settings.editing.default_font_size, 12.0);
        assert!(settings.editing.show_spelling_errors);
        assert!(settings.editing.show_grammar_errors);

        assert!(!settings.privacy.telemetry_enabled);
        assert!(settings.privacy.crash_reports_enabled);
    }

    #[test]
    fn test_settings_serialization_roundtrip() {
        let settings = AppSettings::default();
        let json = serde_json::to_string_pretty(&settings).unwrap();
        let parsed: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, parsed);
    }

    #[test]
    fn test_theme_serialization() {
        assert_eq!(serde_json::to_string(&Theme::Light).unwrap(), "\"light\"");
        assert_eq!(serde_json::to_string(&Theme::Dark).unwrap(), "\"dark\"");
        assert_eq!(serde_json::to_string(&Theme::System).unwrap(), "\"system\"");
    }

    #[test]
    fn test_settings_manager_load_save_sync() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = SettingsManager::new(temp_dir.path().to_path_buf());

        // Load should return defaults when no file exists
        let settings = manager.load_sync().unwrap();
        assert_eq!(settings, &AppSettings::default());

        // Update settings
        let mut new_settings = AppSettings::default();
        new_settings.general.language = "es".to_string();
        new_settings.general.theme = Theme::Dark;
        manager.update_sync(new_settings.clone()).unwrap();

        // Load again should return updated settings
        let mut manager2 = SettingsManager::new(temp_dir.path().to_path_buf());
        let loaded = manager2.load_sync().unwrap();
        assert_eq!(loaded.general.language, "es");
        assert_eq!(loaded.general.theme, Theme::Dark);
    }

    #[test]
    fn test_settings_manager_reset_sync() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = SettingsManager::new(temp_dir.path().to_path_buf());

        // Change settings
        let mut new_settings = AppSettings::default();
        new_settings.general.language = "zh".to_string();
        manager.update_sync(new_settings).unwrap();

        // Reset should restore defaults
        let settings = manager.reset_sync().unwrap();
        assert_eq!(settings.general.language, "en");
    }

    #[tokio::test]
    async fn test_settings_manager_async() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = SettingsManager::new(temp_dir.path().to_path_buf());

        // Load defaults
        manager.load().await.unwrap();

        // Update
        let mut new_settings = AppSettings::default();
        new_settings.editing.autosave_interval_seconds = 120;
        manager.update(new_settings).await.unwrap();

        // Verify
        let mut manager2 = SettingsManager::new(temp_dir.path().to_path_buf());
        let loaded = manager2.load().await.unwrap();
        assert_eq!(loaded.editing.autosave_interval_seconds, 120);
    }
}
