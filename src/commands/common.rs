//! Shared helpers used across command modules.

use std::path::PathBuf;

/// Returns the current user's home directory.
pub fn home_dir() -> Result<PathBuf, String> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .map_err(|_| "home directory not set ($HOME / $USERPROFILE)".to_string())
}

/// Returns `~/.ktool/`.
pub fn ktool_dir() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".ktool"))
}
