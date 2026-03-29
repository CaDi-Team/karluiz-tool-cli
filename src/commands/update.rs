//! Self-update command: fetch the latest GitHub release and replace the running binary.
//!
//! Progress and diagnostics go to **stderr**; the final status line goes to **stdout**
//! so that callers can capture or suppress it independently.
//!
//! The command is idempotent: running it when already on the latest version prints
//! "Already up to date" and exits 0.

use std::fs;
use std::io::Read;

// ---------------------------------------------------------------------------
// Compile-time target detection
// ---------------------------------------------------------------------------

/// The Rust target triple this binary was compiled for.
///
/// Used to select the correct release asset from the GitHub release page.
/// For non-musl Linux targets we map to the musl target string because
/// release assets are only published for musl.
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const CURRENT_TARGET: &str = "x86_64-apple-darwin";

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const CURRENT_TARGET: &str = "aarch64-apple-darwin";

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const CURRENT_TARGET: &str = "x86_64-unknown-linux-musl";

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const CURRENT_TARGET: &str = "aarch64-unknown-linux-musl";

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const CURRENT_TARGET: &str = "x86_64-pc-windows-gnu";

// Catch unsupported platforms at compile time.
#[cfg(not(any(
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "linux", target_arch = "aarch64"),
    all(target_os = "windows", target_arch = "x86_64"),
)))]
compile_error!(
    "ktool update: unsupported target platform — add a CURRENT_TARGET constant for this target"
);

const RELEASES_API: &str =
    "https://api.github.com/repos/CaDi-Team/karluiz-tool-cli/releases/latest";

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Checks GitHub for a newer release and, if found, downloads and installs it in place.
///
/// Idempotent: exits 0 with a friendly message when already on the latest version.
pub fn run() -> Result<(), String> {
    eprintln!("Checking for updates...");

    let release = fetch_latest()?;

    let latest_tag = release["tag_name"]
        .as_str()
        .ok_or("GitHub API response missing 'tag_name'")?;

    let current = format!("v{}", env!("CARGO_PKG_VERSION"));

    if latest_tag == current.as_str() {
        println!("Already up to date ({current}).");
        return Ok(());
    }

    println!("Update available: {current} -> {latest_tag}");
    eprintln!("Downloading {latest_tag} for {CURRENT_TARGET}...");

    let url = find_asset_url(&release)?;
    let bytes = download(&url)?;

    eprintln!("Extracting...");
    let binary = extract_binary(&bytes)?;

    eprintln!("Installing...");
    replace_self(&binary)?;

    println!("Updated to {latest_tag}. Restart ktool to use the new version.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Network helpers
// ---------------------------------------------------------------------------

fn fetch_latest() -> Result<serde_json::Value, String> {
    let ua = format!("ktool/{}", env!("CARGO_PKG_VERSION"));
    let resp = ureq::get(RELEASES_API)
        .set("User-Agent", &ua)
        .set("Accept", "application/vnd.github.v3+json")
        .call()
        .map_err(|e| format!("failed to fetch latest release: {e}"))?;

    resp.into_json::<serde_json::Value>()
        .map_err(|e| format!("failed to parse release JSON: {e}"))
}

fn find_asset_url(release: &serde_json::Value) -> Result<String, String> {
    let assets = release["assets"]
        .as_array()
        .ok_or("GitHub API response missing 'assets'")?;

    let suffix = if cfg!(windows) {
        format!("{CURRENT_TARGET}.zip")
    } else {
        format!("{CURRENT_TARGET}.tar.gz")
    };

    for asset in assets {
        if let Some(name) = asset["name"].as_str()
            && name.ends_with(&suffix)
        {
            let url = asset["browser_download_url"]
                .as_str()
                .ok_or("asset missing 'browser_download_url'")?;
            return Ok(url.to_owned());
        }
    }

    Err(format!(
        "no release asset found for target '{CURRENT_TARGET}'"
    ))
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let ua = format!("ktool/{}", env!("CARGO_PKG_VERSION"));
    let resp = ureq::get(url)
        .set("User-Agent", &ua)
        .call()
        .map_err(|e| format!("download failed: {e}"))?;

    let mut buf = Vec::new();
    resp.into_reader()
        .read_to_end(&mut buf)
        .map_err(|e| format!("failed to read download body: {e}"))?;
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Archive extraction — platform-specific
// ---------------------------------------------------------------------------

/// Extracts the `ktool` binary from a `.tar.gz` archive (all Unix targets).
#[cfg(unix)]
fn extract_binary(bytes: &[u8]) -> Result<Vec<u8>, String> {
    use flate2::read::GzDecoder;
    use std::ffi::OsStr;
    use std::io::Read;
    use tar::Archive;

    let mut archive = Archive::new(GzDecoder::new(bytes));
    for entry in archive.entries().map_err(|e| format!("tar error: {e}"))? {
        let mut entry = entry.map_err(|e| format!("tar entry error: {e}"))?;
        let path = entry
            .path()
            .map_err(|e| format!("tar path error: {e}"))?
            .into_owned();
        if path.file_name() == Some(OsStr::new("ktool")) {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("failed to read binary from archive: {e}"))?;
            return Ok(buf);
        }
    }
    Err("'ktool' binary not found in archive".to_string())
}

/// Extracts `ktool.exe` from a `.zip` archive (Windows target).
#[cfg(windows)]
fn extract_binary(bytes: &[u8]) -> Result<Vec<u8>, String> {
    use std::io::Read;
    use zip::ZipArchive;

    let cursor = std::io::Cursor::new(bytes);
    let mut archive =
        ZipArchive::new(cursor).map_err(|e| format!("failed to open zip archive: {e}"))?;
    let mut file = archive
        .by_name("ktool.exe")
        .map_err(|e| format!("ktool.exe not found in archive: {e}"))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|e| format!("failed to read binary from archive: {e}"))?;
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Binary replacement — platform-specific
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_target_is_not_empty() {
        assert!(
            !CURRENT_TARGET.is_empty(),
            "CURRENT_TARGET should not be empty"
        );
    }

    #[test]
    fn current_target_contains_expected_parts() {
        // Every target triple has at least an arch and an OS segment.
        assert!(
            CURRENT_TARGET.contains('-'),
            "CURRENT_TARGET should contain hyphens: {CURRENT_TARGET}"
        );
        // Should contain a known arch.
        let known_archs = ["x86_64", "aarch64"];
        assert!(
            known_archs.iter().any(|a| CURRENT_TARGET.starts_with(a)),
            "CURRENT_TARGET should start with a known arch: {CURRENT_TARGET}"
        );
    }

    /// Build a mock release JSON with the given asset names and URLs.
    fn mock_release(assets: &[(&str, &str)]) -> serde_json::Value {
        let assets_json: Vec<serde_json::Value> = assets
            .iter()
            .map(|(name, browser_url)| {
                serde_json::json!({
                    "name": name,
                    "browser_download_url": browser_url,
                })
            })
            .collect();
        serde_json::json!({
            "tag_name": "v0.2.4",
            "assets": assets_json,
        })
    }

    #[test]
    fn find_asset_url_finds_matching_asset() {
        let suffix = if cfg!(windows) {
            format!("{CURRENT_TARGET}.zip")
        } else {
            format!("{CURRENT_TARGET}.tar.gz")
        };
        let asset_name = format!("ktool-{suffix}");

        let release = mock_release(&[(
            &asset_name,
            "https://github.com/releases/download/asset",
        )]);

        let url = find_asset_url(&release).expect("should find the matching asset");
        assert_eq!(url, "https://github.com/releases/download/asset");
    }

    #[test]
    fn find_asset_url_returns_error_when_no_matching_asset() {
        let release = mock_release(&[(
            "ktool-some-other-target.tar.gz",
            "https://github.com/releases/download/asset",
        )]);

        let result = find_asset_url(&release);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no release asset found"));
    }
}

/// Replaces the running binary with `binary`.
fn replace_self(binary: &[u8]) -> Result<(), String> {
    let current_exe =
        std::env::current_exe().map_err(|e| format!("cannot locate current exe: {e}"))?;
    let tmp = current_exe.with_extension("tmp");

    fs::write(&tmp, binary).map_err(|e| format!("failed to write temp binary: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o755));
        fs::rename(&tmp, &current_exe).map_err(|e| format!("failed to replace binary: {e}"))?;
    }

    #[cfg(windows)]
    {
        let old = current_exe.with_extension("old");
        if old.exists() {
            let _ = fs::remove_file(&old);
        }
        fs::rename(&current_exe, &old)
            .map_err(|e| format!("failed to rename current binary: {e}"))?;
        fs::rename(&tmp, &current_exe).map_err(|e| format!("failed to install new binary: {e}"))?;
    }

    Ok(())
}
