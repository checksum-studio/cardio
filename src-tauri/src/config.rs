use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing::{debug, error, instrument};
use utoipa::ToSchema;

static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();
pub fn init_config_dir(path: PathBuf) {
    let _ = CONFIG_DIR.set(path);
}

fn config_dir() -> PathBuf {
    CONFIG_DIR
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("."))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AppConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_true")]
    pub auto_connect: bool,
    #[serde(default)]
    pub auto_launch: bool,
    #[serde(default)]
    pub start_minimized: bool,
    #[serde(default)]
    pub last_device_id: Option<String>,
    #[serde(default)]
    pub last_device_name: Option<String>,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    2328
}

fn default_true() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 2328,
            auto_connect: true,
            auto_launch: false,
            start_minimized: false,
            last_device_id: None,
            last_device_name: None,
        }
    }
}

impl AppConfig {
    fn config_path() -> PathBuf {
        config_dir().join("config.json")
    }

    #[instrument(level = "debug")]
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(contents) => match serde_json::from_str(&contents) {
                    Ok(config) => {
                        debug!(path = %path.display(), "loaded config");
                        return config;
                    }
                    Err(e) => {
                        error!(error = %e, path = %path.display(), "failed to parse config");
                    }
                },
                Err(e) => {
                    error!(error = %e, path = %path.display(), "failed to read config file");
                }
            }
        }
        debug!("using default config");
        Self::default()
    }

    #[instrument(level = "debug", skip(self))]
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        debug!(path = %path.display(), "saved config");
        Ok(())
    }
}
