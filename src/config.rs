use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    pub ui: UiConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub key: String,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry")]
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_emoji")]
    pub emoji: bool,
    #[serde(default = "default_refresh")]
    pub refresh_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,
}

fn default_timeout() -> u64 {
    5000
}
fn default_retry() -> u32 {
    3
}
fn default_theme() -> String {
    "dark".to_string()
}
fn default_emoji() -> bool {
    true
}
fn default_refresh() -> u64 {
    5000
}
fn default_cache_ttl() -> u64 {
    300
}

impl Config {
    pub fn load(path: Option<String>) -> Result<Self> {
        let config_path = path
            .map(PathBuf::from)
            .or_else(|| Self::default_path())
            .context("No config file found")?;

        if config_path.exists() {
            let content =
                std::fs::read_to_string(&config_path).context("Failed to read config file")?;
            toml::from_str(&content).context("Failed to parse config file")
        } else {
            Self::create_default(&config_path)
        }
    }

    fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("govee-tui").join("config.toml"))
    }

    fn create_default(path: &Path) -> Result<Self> {
        let config = Self::default();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(&config)?;
        std::fs::write(path, content)?;

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        let db_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("govee-tui")
            .join("devices.db");

        Self {
            api: ApiConfig {
                key: String::from("YOUR_API_KEY_HERE"),
                timeout_ms: default_timeout(),
                retry_attempts: default_retry(),
            },
            ui: UiConfig {
                theme: default_theme(),
                emoji: default_emoji(),
                refresh_interval_ms: default_refresh(),
            },
            database: DatabaseConfig { path: db_path, cache_ttl_seconds: default_cache_ttl() },
        }
    }
}
