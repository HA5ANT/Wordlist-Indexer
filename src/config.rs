use crate::error::WlError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub repos: Vec<PathBuf>,
    pub db_path: String,
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    PathBuf::from(path)
}

pub fn get_config_path() -> Result<PathBuf, WlError> {
    if let Some(p) = std::env::var_os("WL_CONFIG_PATH") {
        return Ok(PathBuf::from(p));
    }
    dirs::config_dir()
        .map(|p| p.join("wl").join("config.toml"))
        .ok_or_else(|| WlError::Config("Could not determine config directory".into()))
}

pub fn load() -> Result<Config, WlError> {
    let config_path = get_config_path()?;
    if !config_path.exists() {
        let default_db_path = std::env::var("WL_DB_PATH")
            .unwrap_or_else(|_| "~/.local/share/wl/index.db".to_string());
        let default_config = Config {
            repos: Vec::new(),
            db_path: default_db_path,
        };
        save(&default_config)?;
        return Ok(default_config);
    }

    let content = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)
        .map_err(|e| WlError::Config(format!("Failed to parse config.toml: {}", e)))?;
    Ok(config)
}

pub fn save(config: &Config) -> Result<(), WlError> {
    let config_path = get_config_path()?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)
        .map_err(|e| WlError::Config(format!("Failed to serialize config: {}", e)))?;
    fs::write(&config_path, content)?;
    Ok(())
}

pub fn add_repo(path: PathBuf) -> Result<(), WlError> {
    let mut config = load()?;
    let absolute_path = fs::canonicalize(&path).unwrap_or(path);
    if !config.repos.contains(&absolute_path) {
        config.repos.push(absolute_path);
        save(&config)?;
    }
    Ok(())
}

pub fn remove_repo(path: PathBuf) -> Result<(), WlError> {
    let mut config = load()?;
    let absolute_path = fs::canonicalize(&path).unwrap_or(path);
    let original_len = config.repos.len();
    config.repos.retain(|p| p != &absolute_path);
    if config.repos.len() != original_len {
        save(&config)?;
    }
    Ok(())
}
