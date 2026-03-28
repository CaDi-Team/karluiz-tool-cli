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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_dir_returns_non_empty_path() {
        let home = home_dir().expect("home_dir should succeed in test env");
        assert!(
            !home.as_os_str().is_empty(),
            "home directory path should not be empty"
        );
    }

    #[test]
    fn ktool_dir_ends_with_dot_ktool() {
        let dir = ktool_dir().expect("ktool_dir should succeed in test env");
        assert!(
            dir.ends_with(".ktool"),
            "ktool_dir should end with .ktool, got: {}",
            dir.display()
        );
    }

    #[test]
    fn ktool_dir_is_under_home() {
        let home = home_dir().unwrap();
        let dir = ktool_dir().unwrap();
        assert!(
            dir.starts_with(&home),
            "ktool_dir ({}) should be under home ({})",
            dir.display(),
            home.display()
        );
    }
}
