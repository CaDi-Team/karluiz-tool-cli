use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::commands::common::ktool_dir;

/// Persistent configuration stored in `~/.ktool/config.toml`.
///
/// The `token` field was removed in v0.2.0 — authentication is now managed
/// by the auth module. Unknown fields from older configs are silently ignored
/// thanks to `#[serde(default)]` on every field.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Default application name (`--set-app`).
    #[serde(default)]
    pub app: Option<String>,
    /// Default environment (`--set-env`).
    #[serde(default)]
    pub env: Option<String>,
}

/// Return the path to the config file, creating parent directories if needed.
pub fn config_path() -> Result<PathBuf, String> {
    let dir = ktool_dir()?;
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
    let raw =
        fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    // Use toml::Value first to strip unknown fields, then deserialize.
    // This tolerates old configs that still have a `token` field.
    let table: toml::Value =
        toml::from_str(&raw).map_err(|e| format!("Failed to parse config: {e}"))?;
    let cfg: Config = serde::Deserialize::deserialize(table)
        .map_err(|e| format!("Failed to deserialize config: {e}"))?;
    Ok(cfg)
}

pub(crate) fn save_to(path: &PathBuf, cfg: &Config) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create config directory {}: {e}",
                parent.display()
            )
        })?;
    }
    let content =
        toml::to_string_pretty(cfg).map_err(|e| format!("Failed to serialise config: {e}"))?;
    fs::write(path, content).map_err(|e| format!("Failed to write {}: {e}", path.display()))
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
        assert!(cfg.app.is_none());
        assert!(cfg.env.is_none());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let (_dir, path) = temp_config_path();
        let cfg = Config {
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

    #[test]
    fn tolerates_unknown_fields() {
        let (_dir, path) = temp_config_path();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            "token = \"old-secret\"\napp = \"my-app\"\nenv = \"prod\"\n",
        )
        .unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.app, Some("my-app".to_string()));
        assert_eq!(loaded.env, Some("prod".to_string()));
    }

    #[test]
    fn config_with_only_app_saves_and_loads() {
        let (_dir, path) = temp_config_path();
        let cfg = Config {
            app: Some("only-app".to_string()),
            env: None,
        };
        save_to(&path, &cfg).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.app, Some("only-app".to_string()));
        assert_eq!(loaded.env, None);
    }

    #[test]
    fn config_with_only_env_saves_and_loads() {
        let (_dir, path) = temp_config_path();
        let cfg = Config {
            app: None,
            env: Some("staging".to_string()),
        };
        save_to(&path, &cfg).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.app, None);
        assert_eq!(loaded.env, Some("staging".to_string()));
    }

    #[test]
    fn empty_string_values_are_preserved() {
        let (_dir, path) = temp_config_path();
        let cfg = Config {
            app: Some("".to_string()),
            env: Some("".to_string()),
        };
        save_to(&path, &cfg).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(
            loaded.app,
            Some("".to_string()),
            "empty app should be preserved as Some(\"\")"
        );
        assert_eq!(
            loaded.env,
            Some("".to_string()),
            "empty env should be preserved as Some(\"\")"
        );
    }
}
