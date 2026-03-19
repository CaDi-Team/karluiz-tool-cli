use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persistent configuration stored in `~/.config/ktool/config.toml`.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Bearer token used to authenticate against the karluiz API.
    pub token: Option<String>,
    /// Default application name (`--set-app`).
    pub app: Option<String>,
    /// Default environment (`--set-env`).
    pub env: Option<String>,
}

/// Return the path to the config file, creating parent directories if needed.
pub fn config_path() -> Result<PathBuf, String> {
    let base = dirs::config_dir()
        .ok_or_else(|| "Cannot locate the user config directory.".to_string())?;
    let dir = base.join("ktool");
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create config directory {}: {e}", dir.display()))?;
    Ok(dir.join("config.toml"))
}

/// Load the config from disk, returning a default `Config` if the file does not exist yet.
pub fn load() -> Result<Config, String> {
    load_from(&config_path()?)
}

/// Persist the config to disk.
pub fn save(cfg: &Config) -> Result<(), String> {
    save_to(&config_path()?, cfg)
}

// Internal helpers used directly in tests so we never mutate global env vars.

pub(crate) fn load_from(path: &PathBuf) -> Result<Config, String> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    toml::from_str(&raw).map_err(|e| format!("Failed to parse config: {e}"))
}

pub(crate) fn save_to(path: &PathBuf, cfg: &Config) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory {}: {e}", parent.display()))?;
    }
    let content = toml::to_string_pretty(cfg)
        .map_err(|e| format!("Failed to serialise config: {e}"))?;
    fs::write(path, content)
        .map_err(|e| format!("Failed to write {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_config_path() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ktool").join("config.toml");
        (dir, path)
    }

    #[test]
    fn default_config_is_empty() {
        let cfg = Config::default();
        assert!(cfg.token.is_none());
        assert!(cfg.app.is_none());
        assert!(cfg.env.is_none());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let (_dir, path) = temp_config_path();
        let cfg = Config {
            token: Some("tok_test".to_string()),
            app: Some("my-app".to_string()),
            env: Some("prod".to_string()),
        };
        save_to(&path, &cfg).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(cfg, loaded);
    }

    #[test]
    fn load_returns_default_when_file_missing() {
        let (_dir, path) = temp_config_path();
        let result = load_from(&path).unwrap();
        assert_eq!(result, Config::default());
    }
}
