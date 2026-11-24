use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::models::SavedWifiDevice;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub adb_path: Option<PathBuf>,
    pub refresh_interval_ms: u64,
    pub logcat_max_lines: usize,
    pub export_dir: Option<PathBuf>,
    pub saved_devices: Vec<SavedWifiDevice>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            adb_path: None,
            refresh_interval_ms: 2000,
            logcat_max_lines: 10_000,
            export_dir: None,
            saved_devices: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn project_dirs() -> Option<ProjectDirs> {
        ProjectDirs::from("dev", "adbsight", "adbsight")
    }

    pub fn config_file_path() -> PathBuf {
        Self::project_dirs()
            .map(|d| d.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("adbsight.toml"))
    }

    pub fn default_db_path() -> PathBuf {
        Self::project_dirs()
            .map(|d| d.data_local_dir().join("adbsight.db"))
            .unwrap_or_else(|| PathBuf::from("adbsight.db"))
    }

    pub fn default_log_path() -> PathBuf {
        Self::project_dirs()
            .map(|d| d.data_local_dir().join("adbsight.log"))
            .unwrap_or_else(|| PathBuf::from("adbsight.log"))
    }

    pub fn resolved_export_dir(&self) -> PathBuf {
        if let Some(ref dir) = self.export_dir {
            dir.clone()
        } else {
            PathBuf::from("exports")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_file_path();
        if let Ok(raw) = fs::read_to_string(&path)
            && let Ok(cfg) = toml::from_str::<AppConfig>(&raw)
        {
            return cfg;
        }
        Self::default()
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        let body = toml::to_string_pretty(self).context("failed to serialize config")?;
        fs::write(path, body)?;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::config_file_path())
    }

    pub fn bookmark_wifi_device(&mut self, name: String, address: String) {
        let now = chrono::Utc::now();
        if let Some(existing) = self.saved_devices.iter_mut().find(|d| d.address == address) {
            existing.name = name;
            existing.last_connected = Some(now);
        } else {
            self.saved_devices.push(SavedWifiDevice {
                name,
                address,
                last_connected: Some(now),
            });
        }
    }
}
