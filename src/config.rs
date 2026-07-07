use color_eyre::eyre::{ContextCompat, Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    #[serde(default)]
    pub ui: UiConfig,
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
    #[serde(default = "default_refresh")]
    pub refresh_interval_ms: u64,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            refresh_interval_ms: default_refresh(),
        }
    }
}

fn default_timeout() -> u64 {
    10_000
}
fn default_retry() -> u32 {
    3
}
fn default_refresh() -> u64 {
    30_000
}

impl Config {
    pub fn load(path: Option<String>) -> Result<Self> {
        let config_path = path
            .map(PathBuf::from)
            .or_else(Self::default_path)
            .context("No config file found")?;

        let mut config = if config_path.exists() {
            let content =
                std::fs::read_to_string(&config_path).wrap_err("Failed to read config file")?;
            toml::from_str(&content).wrap_err("Failed to parse config file")?
        } else {
            Self::create_default(&config_path)?
        };

        // Environment variable takes precedence over the config file
        if let Ok(key) = std::env::var("GOVEE_API_KEY") {
            if !key.is_empty() {
                config.api.key = key;
            }
        }

        Ok(config)
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
        Self {
            api: ApiConfig {
                key: String::from("YOUR_API_KEY_HERE"),
                timeout_ms: default_timeout(),
                retry_attempts: default_retry(),
            },
            ui: UiConfig::default(),
        }
    }
}
