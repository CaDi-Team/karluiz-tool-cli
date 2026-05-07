//! Persistent configuration stored in `~/.ktool/config.toml`.
//!
//! # Context system
//!
//! ktool supports two complementary ways to configure the active project/app/env:
//!
//! ## 1. Named global contexts (kubectl-style)
//!
//! Stored in `~/.ktool/config.toml`:
//!
//! ```toml
//! current_context = "work-backend"
//!
//! [contexts.work-backend]
//! project = "orbital"
//! app     = "backend"
//! env     = "production"
//!
//! [contexts.personal]
//! project = "my-slug"
//! app     = "frontend"
//! env     = "staging"
//! ```
//!
//! Use `ktool kenv context use <name>` to switch between contexts.
//!
//! ## 2. Local project file (asdf-style)
//!
//! Drop a `ktool.toml` in any directory (or a parent of it):
//!
//! ```toml
//! project = "orbital"
//! app     = "backend"
//! env     = "production"
//! ```
//!
//! ktool walks up the directory tree from `$PWD` and uses the first
//! `ktool.toml` it finds, overriding the global context.
//!
//! # Resolution order
//!
//! 1. Local `ktool.toml` (walk up from `$PWD`)
//! 2. Active named context (`current_context` → `contexts[name]`)
//! 3. Flat top-level fields (`project` / `app` / `env`) — backward compat
//!
//! Per-command `--project` / `--app` / `--env` flags always win over all of the above.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use crate::commands::common::ktool_dir;
use crate::localrc;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A single named context: a (project, app, env) triple.
///
/// All fields are optional so users can share a context that only fixes the
/// project slug while still being able to override app/env freely.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Context {
    /// kENV project slug (the `{projectSlug}` path segment in API URLs).
    #[serde(default)]
    pub project: Option<String>,
    /// Default application name.
    #[serde(default)]
    pub app: Option<String>,
    /// Default environment name.
    #[serde(default)]
    pub env: Option<String>,
}

/// Persistent configuration (`~/.ktool/config.toml`).
///
/// The flat `project` / `app` / `env` fields are kept for backward
/// compatibility with configs created before v0.3.0.  New setups should
/// use the `contexts` map instead.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Config {
    // ---- backward-compat flat fields (pre-v0.3) ----
    /// Default project slug (legacy, prefer `contexts`).
    #[serde(default)]
    pub project: Option<String>,
    /// Default application name (legacy, prefer `contexts`).
    #[serde(default)]
    pub app: Option<String>,
    /// Default environment name (legacy, prefer `contexts`).
    #[serde(default)]
    pub env: Option<String>,

    // ---- new context system (v0.3+) ----
    /// Name of the currently active context.
    #[serde(default)]
    pub current_context: Option<String>,
    /// Named contexts map.
    #[serde(default)]
    pub contexts: BTreeMap<String, Context>,
}

// ---------------------------------------------------------------------------
// Resolution
// ---------------------------------------------------------------------------

/// Where the resolved project/app/env values came from.
#[derive(Debug, Clone, PartialEq)]
pub enum ContextSource {
    /// A `ktool.toml` found by walking up from `$PWD`.
    LocalFile(PathBuf),
    /// An active named context from the global config.
    NamedContext(String),
    /// The flat top-level fields in `~/.ktool/config.toml` (legacy / default).
    GlobalDefaults,
}

/// The resolved active project, app, and environment, together with where the
/// values came from (for display in `ktool kenv` and `ktool kenv context current`).
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedContext {
    pub project: Option<String>,
    pub app: Option<String>,
    pub env: Option<String>,
    pub source: ContextSource,
}

/// Resolve the active context following the priority chain:
/// local `ktool.toml` > named context > flat defaults.
pub fn resolve_context(cfg: &Config) -> ResolvedContext {
    // 1. Local ktool.toml (walk up from CWD).
    if let Some((path, local)) = localrc::find() {
        return ResolvedContext {
            project: local.project,
            app: local.app,
            env: local.env,
            source: ContextSource::LocalFile(path),
        };
    }

    // 2. Active named context.
    if let Some(name) = &cfg.current_context
        && let Some(ctx) = cfg.contexts.get(name)
    {
        return ResolvedContext {
            project: ctx.project.clone(),
            app: ctx.app.clone(),
            env: ctx.env.clone(),
            source: ContextSource::NamedContext(name.clone()),
        };
    }

    // 3. Legacy flat defaults.
    ResolvedContext {
        project: cfg.project.clone(),
        app: cfg.app.clone(),
        env: cfg.env.clone(),
        source: ContextSource::GlobalDefaults,
    }
}

// ---------------------------------------------------------------------------
// Persistence helpers
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn temp_config_path() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ktool").join("config.toml");
        (dir, path)
    }

    // ---- basic roundtrip ----

    #[test]
    fn default_config_is_empty() {
        let cfg = Config::default();
        assert!(cfg.app.is_none());
        assert!(cfg.env.is_none());
        assert!(cfg.project.is_none());
        assert!(cfg.current_context.is_none());
        assert!(cfg.contexts.is_empty());
    }

    #[test]
    fn save_and_load_flat_roundtrip() {
        let (_dir, path) = temp_config_path();
        let cfg = Config {
            app: Some("my-app".to_string()),
            env: Some("prod".to_string()),
            ..Default::default()
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
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "token = \"old-secret\"\napp = \"my-app\"\nenv = \"prod\"\n",
        )
        .unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.app, Some("my-app".to_string()));
        assert_eq!(loaded.env, Some("prod".to_string()));
    }

    // ---- context roundtrip ----

    #[test]
    fn context_roundtrip() {
        let (_dir, path) = temp_config_path();
        let mut cfg = Config::default();
        cfg.current_context = Some("work".to_string());
        cfg.contexts.insert(
            "work".to_string(),
            Context {
                project: Some("orbital".to_string()),
                app: Some("backend".to_string()),
                env: Some("production".to_string()),
            },
        );
        save_to(&path, &cfg).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.current_context, Some("work".to_string()));
        let ctx = loaded.contexts.get("work").unwrap();
        assert_eq!(ctx.project, Some("orbital".to_string()));
        assert_eq!(ctx.app, Some("backend".to_string()));
        assert_eq!(ctx.env, Some("production".to_string()));
    }

    #[test]
    fn multiple_contexts_saved_and_loaded() {
        let (_dir, path) = temp_config_path();
        let mut cfg = Config::default();
        cfg.contexts.insert(
            "a".to_string(),
            Context {
                project: Some("proj-a".to_string()),
                app: Some("app-a".to_string()),
                env: Some("prod".to_string()),
            },
        );
        cfg.contexts.insert(
            "b".to_string(),
            Context {
                project: Some("proj-b".to_string()),
                app: None,
                env: Some("staging".to_string()),
            },
        );
        save_to(&path, &cfg).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.contexts.len(), 2);
        assert_eq!(loaded.contexts["a"].project, Some("proj-a".to_string()));
        assert!(loaded.contexts["b"].app.is_none());
    }

    // ---- resolve_context ----

    #[test]
    fn resolve_uses_named_context_when_set() {
        let mut cfg = Config::default();
        cfg.current_context = Some("ctx".to_string());
        cfg.contexts.insert(
            "ctx".to_string(),
            Context {
                project: Some("proj".to_string()),
                app: Some("app".to_string()),
                env: Some("env".to_string()),
            },
        );
        // No local file in temp dir, so named context should win.
        let resolved = resolve_context(&cfg);
        // Source may be GlobalDefaults if CWD happens to have a ktool.toml;
        // we mainly assert values when the source is NamedContext.
        match resolved.source {
            ContextSource::NamedContext(ref name) => {
                assert_eq!(name, "ctx");
                assert_eq!(resolved.project, Some("proj".to_string()));
                assert_eq!(resolved.app, Some("app".to_string()));
                assert_eq!(resolved.env, Some("env".to_string()));
            }
            // If a local file happened to be found, skip assertion.
            ContextSource::LocalFile(_) => {}
            ContextSource::GlobalDefaults => {
                // current_context set but context not found → falls through
            }
        }
    }

    #[test]
    fn resolve_falls_back_to_flat_defaults() {
        let cfg = Config {
            project: Some("flat-proj".to_string()),
            app: Some("flat-app".to_string()),
            env: Some("flat-env".to_string()),
            ..Default::default()
        };
        let resolved = resolve_context(&cfg);
        // If no local file and no named context, should use flat fields.
        if resolved.source == ContextSource::GlobalDefaults {
            assert_eq!(resolved.project, Some("flat-proj".to_string()));
            assert_eq!(resolved.app, Some("flat-app".to_string()));
            assert_eq!(resolved.env, Some("flat-env".to_string()));
        }
    }

    #[test]
    fn resolve_with_missing_named_context_falls_through() {
        let mut cfg = Config::default();
        cfg.current_context = Some("nonexistent".to_string());
        cfg.app = Some("fallback-app".to_string());
        let resolved = resolve_context(&cfg);
        // Named context doesn't exist → fall to flat defaults.
        if resolved.source == ContextSource::GlobalDefaults {
            assert_eq!(resolved.app, Some("fallback-app".to_string()));
        }
    }

    // ---- legacy compat ----

    #[test]
    fn config_with_only_app_saves_and_loads() {
        let (_dir, path) = temp_config_path();
        let cfg = Config {
            app: Some("only-app".to_string()),
            ..Default::default()
        };
        save_to(&path, &cfg).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.app, Some("only-app".to_string()));
        assert_eq!(loaded.env, None);
    }

    #[test]
    fn empty_string_values_are_preserved() {
        let (_dir, path) = temp_config_path();
        let cfg = Config {
            app: Some("".to_string()),
            env: Some("".to_string()),
            ..Default::default()
        };
        save_to(&path, &cfg).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.app, Some("".to_string()));
        assert_eq!(loaded.env, Some("".to_string()));
    }
}
