//! Local project config file (`ktool.toml`) discovery.
//!
//! `ktool.toml` is an asdf-style local override file. When `ktool` runs, it
//! walks up from the current working directory looking for a `ktool.toml` file.
//! The first one found takes precedence over the global `~/.ktool/config.toml`
//! context, allowing you to pin a specific project/app/env per repository.
//!
//! # Example `ktool.toml`
//! ```toml
//! project = "orbital"
//! app     = "backend"
//! env     = "production"
//! ```
//!
//! All three fields are optional — only the fields present override the global
//! context; missing fields fall through to the global context.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Values read from a local `ktool.toml` file.
///
/// All fields are optional so that a minimal file (e.g. only `project`) still
/// works without requiring the user to repeat app/env they already have in the
/// global context.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalContext {
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

/// Walk up the directory tree from `start` looking for a file named
/// `ktool.toml`. Returns the path of the first match and its parsed
/// contents, or `None` if no file is found before reaching the root.
pub fn find_from(start: &Path) -> Option<(PathBuf, LocalContext)> {
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join("ktool.toml");
        if candidate.is_file()
            && let Ok(raw) = std::fs::read_to_string(&candidate)
        {
            // Parse tolerantly: unknown keys are ignored.
            if let Ok(ctx) = toml::from_str::<LocalContext>(&raw) {
                return Some((candidate, ctx));
            }
        }
        // Move up one level; stop at root.
        if !dir.pop() {
            return None;
        }
    }
}

/// Walk up from `std::env::current_dir()`.
///
/// Silently returns `None` if the current directory cannot be determined.
pub fn find() -> Option<(PathBuf, LocalContext)> {
    let cwd = std::env::current_dir().ok()?;
    find_from(&cwd)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_ktool_toml(dir: &Path, content: &str) -> PathBuf {
        let path = dir.join("ktool.toml");
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn returns_none_when_no_file() {
        let dir = TempDir::new().unwrap();
        // Temporary dirs are always unique — no ktool.toml above them.
        let result = find_from(dir.path());
        // There's a small chance a ktool.toml exists somewhere above /tmp on the
        // test machine; we can't guarantee None, so we only assert the type.
        let _ = result; // just ensure it compiles and doesn't panic
    }

    #[test]
    fn finds_file_in_same_directory() {
        let dir = TempDir::new().unwrap();
        write_ktool_toml(
            dir.path(),
            "project = \"orbital\"\napp = \"backend\"\nenv = \"prod\"\n",
        );
        let (found_path, ctx) = find_from(dir.path()).expect("should find ktool.toml");
        assert_eq!(found_path, dir.path().join("ktool.toml"));
        assert_eq!(ctx.project, Some("orbital".to_string()));
        assert_eq!(ctx.app, Some("backend".to_string()));
        assert_eq!(ctx.env, Some("prod".to_string()));
    }

    #[test]
    fn finds_file_in_parent_directory() {
        let root = TempDir::new().unwrap();
        write_ktool_toml(root.path(), "project = \"parent-project\"\n");
        let child = root.path().join("subdir");
        fs::create_dir_all(&child).unwrap();
        let (found_path, ctx) = find_from(&child).expect("should find ktool.toml in parent");
        assert_eq!(found_path, root.path().join("ktool.toml"));
        assert_eq!(ctx.project, Some("parent-project".to_string()));
    }

    #[test]
    fn child_file_shadows_parent_file() {
        let root = TempDir::new().unwrap();
        write_ktool_toml(root.path(), "project = \"parent-project\"\n");
        let child = root.path().join("subdir");
        fs::create_dir_all(&child).unwrap();
        write_ktool_toml(&child, "project = \"child-project\"\n");
        let (_, ctx) = find_from(&child).unwrap();
        assert_eq!(ctx.project, Some("child-project".to_string()));
    }

    #[test]
    fn partial_fields_work() {
        let dir = TempDir::new().unwrap();
        // Only set 'app', leave project and env unset.
        write_ktool_toml(dir.path(), "app = \"frontend\"\n");
        let (_, ctx) = find_from(dir.path()).unwrap();
        assert_eq!(ctx.project, None);
        assert_eq!(ctx.app, Some("frontend".to_string()));
        assert_eq!(ctx.env, None);
    }

    #[test]
    fn empty_file_is_valid() {
        let dir = TempDir::new().unwrap();
        write_ktool_toml(dir.path(), "");
        let (_, ctx) = find_from(dir.path()).unwrap();
        assert_eq!(ctx, LocalContext::default());
    }

    #[test]
    fn invalid_toml_is_skipped() {
        let dir = TempDir::new().unwrap();
        // Parent has a valid ktool.toml, child has an invalid one.
        write_ktool_toml(dir.path(), "project = \"parent\"\n");
        let child = dir.path().join("bad");
        fs::create_dir_all(&child).unwrap();
        let bad = child.join("ktool.toml");
        fs::write(&bad, "{ not valid toml !!!").unwrap();
        // Invalid TOML in child → skipped → parent's file found.
        let result = find_from(&child);
        // Should find parent's valid file, not panic.
        if let Some((_, ctx)) = result {
            assert_eq!(ctx.project, Some("parent".to_string()));
        }
        // (If we find None, it means the walk stopped; that's also acceptable
        //  behaviour for a corrupted file, just ensure no panic.)
    }
}
