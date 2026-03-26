//! One-time migration from `~/.config/ktool/config.toml` to the new layout.
//!
//! - If the old config has a `token` field, save it to `~/.ktool/tokens/kenv.json`
//!   (skip if the token file already exists).
//! - If the old config has `app`/`env` fields, save to `~/.ktool/config.toml`
//!   (skip if the new config already exists).
//!
//! Called at the start of `main()`. Silently succeeds if nothing to migrate.
//! Warnings go to stderr.

use std::fs;
use std::path::PathBuf;

use crate::commands::auth::save_kenv_token_to;
use crate::commands::common::ktool_dir;

/// Returns the path to the old config file (`~/.config/ktool/config.toml`).
fn old_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("ktool").join("config.toml"))
}

/// Represents the old config format (v0.1.x) which may contain a `token` field.
#[derive(serde::Deserialize, Default)]
struct OldConfig {
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    app: Option<String>,
    #[serde(default)]
    env: Option<String>,
}

/// Run the migration. Silently succeeds if nothing to migrate.
pub fn run() {
    if let Err(e) = try_migrate() {
        eprintln!("Warning: migration from old config failed: {e}");
    }
}

fn try_migrate() -> Result<(), String> {
    let old_path = match old_config_path() {
        Some(p) => p,
        None => return Ok(()),
    };

    if !old_path.exists() {
        return Ok(());
    }

    let raw = fs::read_to_string(&old_path)
        .map_err(|e| format!("failed to read {}: {e}", old_path.display()))?;

    let old: OldConfig =
        toml::from_str(&raw).map_err(|e| format!("failed to parse old config: {e}"))?;

    let ktool = ktool_dir()?;

    // Migrate token.
    if let Some(ref token) = old.token {
        let token_path = ktool.join("tokens").join("kenv.json");
        if !token_path.exists() {
            save_kenv_token_to(&token_path, token)?;
            eprintln!("Migrated token to {}.", token_path.display());
        }
    }

    // Migrate app/env to new config.
    if old.app.is_some() || old.env.is_some() {
        let new_config_path = ktool.join("config.toml");
        if !new_config_path.exists() {
            let cfg = crate::config::Config {
                app: old.app,
                env: old.env,
            };
            crate::config::save_to(&new_config_path, &cfg)?;
            eprintln!("Migrated config to {}.", new_config_path.display());
        }
    }

    Ok(())
}
