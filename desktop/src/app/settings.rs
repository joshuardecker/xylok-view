use serde::{Deserialize, Serialize};

use crate::app::{AppTheme, DisplayType};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct AppSettings {
    pub theme: AppTheme,
    pub default_display_type: DisplayType,
    pub animate: bool,
    pub notify_if_update: bool,
}

#[derive(Debug, Clone)]
pub enum AppSettingsErr {
    CantSave(&'static str),
}

impl AppSettings {
    pub fn default() -> Self {
        Self {
            theme: AppTheme::Dark,
            default_display_type: DisplayType::GroupId,
            animate: true,
            notify_if_update: true,
        }
    }

    /// Save app settings in the users config directory.
    pub fn save(&self) -> Result<(), AppSettingsErr> {
        use std::fs::File;
        use std::io::Write;

        let mut save_dir = dirs::config_local_dir().ok_or(AppSettingsErr::CantSave(
            "Couldn't locate config directory.",
        ))?;

        save_dir.push("stig-view-settings.toml");

        let settings_str = toml::to_string(self)
            .map_err(|_| AppSettingsErr::CantSave("Couldn't save user settings."))?;

        let mut file = File::create(save_dir)
            .map_err(|_| AppSettingsErr::CantSave("Error creating settings.toml save file."))?;

        let err = write!(file, "{}", settings_str);

        if err.is_err() {
            return Err(AppSettingsErr::CantSave(
                "Error writing settings to settings.toml",
            ));
        }

        Ok(())
    }

    /// Load app settings. No errors, just returns None if it could not find the settings.
    pub fn load() -> Option<Self> {
        use std::fs::read_to_string;

        let mut save_dir = dirs::config_local_dir()?;

        save_dir.push("stig-view-settings.toml");

        let settings_str = read_to_string(save_dir).ok()?;

        let settings: AppSettings = toml::from_str(&settings_str).ok()?;

        Some(settings)
    }
}
