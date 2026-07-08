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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = Config::default();
        assert_eq!(config.api.key, "YOUR_API_KEY_HERE");
        assert_eq!(config.api.timeout_ms, 10_000);
        assert_eq!(config.api.retry_attempts, 3);
        assert_eq!(config.ui.refresh_interval_ms, 30_000);
    }

    #[test]
    fn env_var_override_logic() {
        let mut config = Config::default();
        config.api.key = "file-key".into();

        let key_from_env = "env-key-123";
        if !key_from_env.is_empty() {
            config.api.key = key_from_env.to_string();
        }

        assert_eq!(config.api.key, "env-key-123");
    }

    #[test]
    fn empty_env_var_does_not_override() {
        let mut config = Config::default();
        config.api.key = "file-key".into();

        let key_from_env = "";
        if !key_from_env.is_empty() {
            config.api.key = key_from_env.to_string();
        }

        assert_eq!(config.api.key, "file-key");
    }

    #[test]
    fn api_config_maps_to_client_config() {
        let api = ApiConfig {
            key: "test-key".into(),
            timeout_ms: 5000,
            retry_attempts: 5,
        };

        let client_cfg = govee_api2::ClientConfig {
            timeout: std::time::Duration::from_millis(api.timeout_ms),
            retry_attempts: api.retry_attempts,
            ..govee_api2::ClientConfig::default()
        };

        assert_eq!(client_cfg.timeout, std::time::Duration::from_secs(5));
        assert_eq!(client_cfg.retry_attempts, 5);
        assert_eq!(
            client_cfg.base_url,
            "https://openapi.api.govee.com".to_string()
        );
    }

    #[test]
    fn config_serde_round_trip() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.api.key, config.api.key);
        assert_eq!(deserialized.api.timeout_ms, config.api.timeout_ms);
        assert_eq!(
            deserialized.ui.refresh_interval_ms,
            config.ui.refresh_interval_ms
        );
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
