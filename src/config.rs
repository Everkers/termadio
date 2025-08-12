use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub app_name: String,
    pub version: String,
    pub debug: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_name: "termadio".to_string(),
            version: "0.1.0".to_string(),
            debug: false,
        }
    }
}

impl Config {
    #[allow(dead_code)]
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content)?;

        Ok(())
    }

    #[allow(dead_code)]
    fn config_path() -> Result<PathBuf> {
        let mut path =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        path.push(".config");
        path.push("termadio");
        path.push("config.json");
        Ok(path)
    }
}

// Helper function to get home directory (since dirs crate is not included)
pub mod dirs {
    use std::path::PathBuf;

    #[allow(dead_code)]
    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}
