//! Authentication management for ktool.
//!
//! Token storage lives at `~/.ktool/tokens/kenv.json`.
//! Resolution order: file > `$KENV_API_TOKEN` env var > error.

use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::common::ktool_dir;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// On-disk token envelope — mirrors the diegops schema for interop.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TokenData {
    /// Schema version (always 1).
    pub version: u32,
    /// The raw API token.
    pub token: String,
    /// RFC 3339 timestamp of when the token was stored.
    pub stored_at: String,
}

// ---------------------------------------------------------------------------
// CLI types
// ---------------------------------------------------------------------------

/// Top-level auth subcommand.
#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    /// Manage kenv API credentials.
    Kenv {
        #[command(subcommand)]
        cmd: KenvAuthCommand,
    },
    /// Show authentication status for all providers.
    Status,
    /// Remove all stored tokens.
    Logout,
}

/// Kenv-specific auth subcommands.
#[derive(Subcommand, Debug)]
pub enum KenvAuthCommand {
    /// Store a kenv API token after validating it.
    Login {
        /// The kenv API token to store.
        token: String,
    },
    /// Remove the stored kenv token.
    Logout,
    /// Show the currently authenticated kenv identity.
    Whoami,
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/// Returns the path to `~/.ktool/tokens/kenv.json`.
fn kenv_token_path() -> Result<PathBuf, String> {
    Ok(ktool_dir()?.join("tokens").join("kenv.json"))
}

// ---------------------------------------------------------------------------
// Token persistence (testable variants)
// ---------------------------------------------------------------------------

/// Save a token to the given path.
pub fn save_kenv_token_to(path: &Path, token: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create token directory: {e}"))?;
    }

    let now = humantime::format_rfc3339_seconds(std::time::SystemTime::now()).to_string();
    let data = TokenData {
        version: 1,
        token: token.to_string(),
        stored_at: now,
    };

    let json =
        serde_json::to_string_pretty(&data).map_err(|e| format!("failed to serialize: {e}"))?;
    fs::write(path, &json).map_err(|e| format!("failed to write token: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }

    Ok(())
}

/// Load a token from the given path. Returns `Ok(None)` if the file does not exist.
pub fn load_kenv_token_from(path: &Path) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("failed to read token: {e}"))?;
    let data: TokenData =
        serde_json::from_str(&raw).map_err(|e| format!("failed to parse token: {e}"))?;
    Ok(Some(data.token))
}

/// Remove the token file at the given path. Idempotent.
pub fn remove_kenv_token_from(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("failed to remove token: {e}"))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Public token resolution
// ---------------------------------------------------------------------------

/// Load the kenv API token, resolving: file > `$KENV_API_TOKEN` > error.
pub fn load_kenv_token() -> Result<String, String> {
    let path = kenv_token_path()?;
    if let Some(tok) = load_kenv_token_from(&path)? {
        return Ok(tok);
    }
    if let Ok(tok) = std::env::var("KENV_API_TOKEN")
        && !tok.is_empty()
    {
        return Ok(tok);
    }
    Err("No kenv token found. Run `ktool auth kenv login <TOKEN>` first.".to_string())
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------

/// `ktool auth kenv login <TOKEN>` — validate and store.
pub fn kenv_login(token: &str) -> Result<(), String> {
    if token.is_empty() {
        return Err("token cannot be empty".to_string());
    }

    // Validate the token by calling the API.
    let client = reqwest::blocking::Client::new();
    let url = format!("{}?app=__ping&env=__ping", crate::api::BASE_URL);
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/json")
        .send()
        .map_err(|e| format!("token validation request failed: {e}"))?;

    if resp.status() == reqwest::StatusCode::UNAUTHORIZED
        || resp.status() == reqwest::StatusCode::FORBIDDEN
    {
        return Err("token is invalid or expired".to_string());
    }

    let path = kenv_token_path()?;
    save_kenv_token_to(&path, token)?;
    println!("Token saved to {}.", path.display());
    Ok(())
}

/// `ktool auth kenv whoami` — show current token (obfuscated).
pub fn kenv_whoami() -> Result<(), String> {
    let token = load_kenv_token()?;
    println!(
        "Authenticated with kenv token: {}",
        crate::api::obfuscate(&token)
    );
    Ok(())
}

/// `ktool auth kenv logout` — remove the kenv token.
pub fn kenv_logout() -> Result<(), String> {
    let path = kenv_token_path()?;
    remove_kenv_token_from(&path)?;
    println!("Kenv token removed.");
    Ok(())
}

/// `ktool auth status` — show authentication status for all providers.
pub fn status() -> Result<(), String> {
    let path = kenv_token_path()?;
    match load_kenv_token_from(&path)? {
        Some(tok) => println!("kenv: authenticated ({})", crate::api::obfuscate(&tok)),
        None => {
            if let Ok(tok) = std::env::var("KENV_API_TOKEN") {
                if !tok.is_empty() {
                    println!(
                        "kenv: authenticated via $KENV_API_TOKEN ({})",
                        crate::api::obfuscate(&tok)
                    );
                } else {
                    println!("kenv: not authenticated");
                }
            } else {
                println!("kenv: not authenticated");
            }
        }
    }
    Ok(())
}

/// `ktool auth logout` — remove all stored tokens.
pub fn logout_all() -> Result<(), String> {
    kenv_logout()?;
    println!("All tokens removed.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_token_path() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("tokens").join("kenv.json");
        (dir, path)
    }

    #[test]
    fn token_roundtrip() {
        let (_dir, path) = temp_token_path();
        save_kenv_token_to(&path, "test-token-abc").unwrap();
        let loaded = load_kenv_token_from(&path).unwrap();
        assert_eq!(loaded, Some("test-token-abc".to_string()));
    }

    #[test]
    fn missing_file_returns_none() {
        let (_dir, path) = temp_token_path();
        let loaded = load_kenv_token_from(&path).unwrap();
        assert_eq!(loaded, None);
    }

    #[test]
    fn idempotent_remove() {
        let (_dir, path) = temp_token_path();
        // Removing when nothing exists should succeed.
        remove_kenv_token_from(&path).unwrap();
        // Save, remove, remove — second remove should also succeed.
        save_kenv_token_to(&path, "tok").unwrap();
        remove_kenv_token_from(&path).unwrap();
        remove_kenv_token_from(&path).unwrap();
    }

    #[test]
    fn stored_at_is_rfc3339() {
        let (_dir, path) = temp_token_path();
        save_kenv_token_to(&path, "tok").unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        let data: TokenData = serde_json::from_str(&raw).unwrap();
        // humantime RFC 3339 always contains a 'T' and ends with 'Z'.
        assert!(data.stored_at.contains('T'));
        assert!(data.stored_at.ends_with('Z'));
    }
}
