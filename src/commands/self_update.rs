//! Self-update command for the lin CLI.
//!
//! Allows the CLI to update itself by downloading the latest release from GitHub.

use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::LinError;
use crate::output::{output, HumanDisplay, OutputFormat};
use crate::Result;

/// GitHub repository for releases.
const GITHUB_REPO: &str = "boyeln/lin";

/// Current version from Cargo.toml.
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub release information.
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

/// GitHub release asset.
#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// Response for the update check command.
#[derive(Debug, Serialize)]
pub struct UpdateCheckResponse {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub message: String,
}

impl HumanDisplay for UpdateCheckResponse {
    fn human_fmt(&self) -> String {
        use colored::Colorize;

        if self.update_available {
            format!(
                "{} {} {} {}\n\nRun {} to update.",
                "Update available:".bold(),
                self.current_version.yellow(),
                "→".dimmed(),
                self.latest_version.green(),
                "lin update".cyan()
            )
        } else {
            format!(
                "{} You're on the latest version ({}).",
                "✓".green(),
                self.current_version.green()
            )
        }
    }
}

/// Response for the update command.
#[derive(Debug, Serialize)]
pub struct UpdateResponse {
    pub previous_version: String,
    pub new_version: String,
    pub message: String,
}

impl HumanDisplay for UpdateResponse {
    fn human_fmt(&self) -> String {
        use colored::Colorize;

        format!(
            "{} Updated lin from {} to {}",
            "✓".green(),
            self.previous_version.yellow(),
            self.new_version.green()
        )
    }
}

/// Response when already up to date.
#[derive(Debug, Serialize)]
pub struct AlreadyUpToDateResponse {
    pub current_version: String,
    pub message: String,
}

impl HumanDisplay for AlreadyUpToDateResponse {
    fn human_fmt(&self) -> String {
        use colored::Colorize;

        format!(
            "{} Already up to date (v{})",
            "✓".green(),
            self.current_version.green()
        )
    }
}

/// Detect the current platform and return the target triple.
fn detect_target() -> Result<&'static str> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match (os, arch) {
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-gnu"),
        ("linux", "aarch64") => Ok("aarch64-unknown-linux-gnu"),
        ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
        ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
        ("windows", "x86_64") => Ok("x86_64-pc-windows-msvc"),
        _ => Err(LinError::config(format!(
            "Unsupported platform: {}-{}. Use the install script or build from source.",
            os, arch
        ))),
    }
}

/// Fetch the latest release information from GitHub.
fn fetch_latest_release() -> Result<GitHubRelease> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("lin/{}", CURRENT_VERSION))
        .build()
        .map_err(|e| LinError::api(format!("Failed to create HTTP client: {}", e)))?;

    let response = client
        .get(&url)
        .send()
        .map_err(|e| LinError::api(format!("Failed to fetch release info: {}", e)))?;

    if !response.status().is_success() {
        if response.status().as_u16() == 404 {
            return Err(LinError::api(
                "No releases found. This might be the first release.",
            ));
        }
        return Err(LinError::api(format!(
            "GitHub API returned status {}",
            response.status()
        )));
    }

    response
        .json::<GitHubRelease>()
        .map_err(|e| LinError::parse(format!("Failed to parse release info: {}", e)))
}

/// Parse a version string (e.g., "v0.1.0" or "0.1.0") into comparable parts.
fn parse_version(version: &str) -> (u32, u32, u32) {
    let v = version.trim_start_matches('v');
    let parts: Vec<u32> = v.split('.').filter_map(|p| p.parse().ok()).collect();

    (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    )
}

/// Compare two version strings. Returns true if `latest` is newer than `current`.
fn is_newer_version(current: &str, latest: &str) -> bool {
    let current = parse_version(current);
    let latest = parse_version(latest);
    latest > current
}

/// Get the path to the current executable.
fn get_current_exe() -> Result<PathBuf> {
    env::current_exe()
        .map_err(|e| LinError::config(format!("Failed to get executable path: {}", e)))
}

/// Download a file from a URL to a destination path.
fn download_file(url: &str, dest: &Path) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("lin/{}", CURRENT_VERSION))
        .build()
        .map_err(|e| LinError::api(format!("Failed to create HTTP client: {}", e)))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| LinError::api(format!("Failed to download: {}", e)))?;

    if !response.status().is_success() {
        return Err(LinError::api(format!(
            "Download failed with status {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .map_err(|e| LinError::api(format!("Failed to read download: {}", e)))?;

    let mut file = File::create(dest)?;
    file.write_all(&bytes)?;

    Ok(())
}

/// Extract the binary from a tar.gz archive.
fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> Result<PathBuf> {
    use std::process::Command;

    // Use tar command to extract (available on Linux/macOS)
    let status = Command::new("tar")
        .args([
            "xzf",
            archive_path.to_str().unwrap(),
            "-C",
            dest_dir.to_str().unwrap(),
        ])
        .status()
        .map_err(|e| LinError::config(format!("Failed to run tar: {}", e)))?;

    if !status.success() {
        return Err(LinError::config("Failed to extract archive"));
    }

    Ok(dest_dir.join("lin"))
}

/// Extract the binary from a zip archive (Windows).
fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<PathBuf> {
    use std::process::Command;

    // Use unzip command or PowerShell
    #[cfg(target_os = "windows")]
    {
        let status = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    archive_path.display(),
                    dest_dir.display()
                ),
            ])
            .status()
            .map_err(|e| LinError::config(format!("Failed to run PowerShell: {}", e)))?;

        if !status.success() {
            return Err(LinError::config("Failed to extract archive"));
        }

        Ok(dest_dir.join("lin.exe"))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let status = Command::new("unzip")
            .args([
                "-o",
                archive_path.to_str().unwrap(),
                "-d",
                dest_dir.to_str().unwrap(),
            ])
            .status()
            .map_err(|e| LinError::config(format!("Failed to run unzip: {}", e)))?;

        if !status.success() {
            return Err(LinError::config("Failed to extract archive"));
        }

        Ok(dest_dir.join("lin"))
    }
}

/// Replace the current executable with a new one.
fn replace_executable(new_binary: &Path, current_exe: &Path) -> Result<()> {
    // On Unix, we can replace the running binary directly
    // On Windows, we need to rename the old one first

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Make the new binary executable
        let mut perms = fs::metadata(new_binary)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(new_binary, perms)?;

        // Replace the current executable
        fs::rename(new_binary, current_exe)?;
    }

    #[cfg(windows)]
    {
        // On Windows, rename the old executable first
        let backup_path = current_exe.with_extension("old.exe");
        if backup_path.exists() {
            fs::remove_file(&backup_path)?;
        }
        fs::rename(current_exe, &backup_path)?;
        fs::rename(new_binary, current_exe)?;
        // Clean up the old executable
        let _ = fs::remove_file(&backup_path);
    }

    Ok(())
}

/// Check if an update is available without installing it.
pub fn check_update(format: OutputFormat) -> Result<()> {
    let release = fetch_latest_release()?;
    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let update_available = is_newer_version(CURRENT_VERSION, &release.tag_name);

    let response = UpdateCheckResponse {
        current_version: CURRENT_VERSION.to_string(),
        latest_version: latest_version.clone(),
        update_available,
        message: if update_available {
            format!("Update available: {} → {}", CURRENT_VERSION, latest_version)
        } else {
            format!("Already up to date (v{})", CURRENT_VERSION)
        },
    };

    output(&response, format);
    Ok(())
}

/// Update the CLI to the latest version.
pub fn update(format: OutputFormat) -> Result<()> {
    // Fetch latest release
    eprint!("Checking for updates... ");
    io::stderr().flush().ok();

    let release = fetch_latest_release()?;
    let latest_version = release.tag_name.clone();

    // Check if update is needed
    if !is_newer_version(CURRENT_VERSION, &latest_version) {
        eprintln!("done");
        let response = AlreadyUpToDateResponse {
            current_version: CURRENT_VERSION.to_string(),
            message: format!("Already up to date (v{})", CURRENT_VERSION),
        };
        output(&response, format);
        return Ok(());
    }
    eprintln!("found {}", latest_version);

    // Detect platform
    let target = detect_target()?;
    let is_windows = target.contains("windows");
    let ext = if is_windows { "zip" } else { "tar.gz" };

    // Find the appropriate asset
    let asset_name = format!("lin-{}-{}.{}", latest_version, target, ext);
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| {
            LinError::config(format!(
                "No binary found for your platform ({}). Use the install script or build from source.",
                target
            ))
        })?;

    // Create temp directory
    let temp_dir = tempfile::tempdir()
        .map_err(|e| LinError::config(format!("Failed to create temp directory: {}", e)))?;

    // Download
    eprint!("Downloading {}... ", asset.name);
    io::stderr().flush().ok();

    let archive_path = temp_dir.path().join(&asset.name);
    download_file(&asset.browser_download_url, &archive_path)?;
    eprintln!("done");

    // Extract
    eprint!("Extracting... ");
    io::stderr().flush().ok();

    let new_binary = if is_windows {
        extract_zip(&archive_path, temp_dir.path())?
    } else {
        extract_tar_gz(&archive_path, temp_dir.path())?
    };
    eprintln!("done");

    // Replace current executable
    eprint!("Installing... ");
    io::stderr().flush().ok();

    let current_exe = get_current_exe()?;
    replace_executable(&new_binary, &current_exe)?;
    eprintln!("done");

    let response = UpdateResponse {
        previous_version: CURRENT_VERSION.to_string(),
        new_version: latest_version.trim_start_matches('v').to_string(),
        message: format!(
            "Updated lin from {} to {}",
            CURRENT_VERSION,
            latest_version.trim_start_matches('v')
        ),
    };

    output(&response, format);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("0.1.0"), (0, 1, 0));
        assert_eq!(parse_version("v0.1.0"), (0, 1, 0));
        assert_eq!(parse_version("1.2.3"), (1, 2, 3));
        assert_eq!(parse_version("v10.20.30"), (10, 20, 30));
    }

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("0.1.0", "0.2.0"));
        assert!(is_newer_version("0.1.0", "v0.2.0"));
        assert!(is_newer_version("0.1.0", "1.0.0"));
        assert!(is_newer_version("0.1.9", "0.2.0"));
        assert!(!is_newer_version("0.2.0", "0.1.0"));
        assert!(!is_newer_version("0.1.0", "0.1.0"));
        assert!(!is_newer_version("1.0.0", "0.9.9"));
    }

    #[test]
    fn test_detect_target() {
        // This test will pass on supported platforms
        let result = detect_target();
        // We can't assert the exact value since it depends on the platform
        // but we can check it doesn't fail on supported platforms
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        assert_eq!(result.unwrap(), "x86_64-unknown-linux-gnu");

        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        assert_eq!(result.unwrap(), "x86_64-apple-darwin");

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        assert_eq!(result.unwrap(), "aarch64-apple-darwin");
    }

    #[test]
    fn test_update_check_response_human_fmt() {
        let response = UpdateCheckResponse {
            current_version: "0.1.0".to_string(),
            latest_version: "0.2.0".to_string(),
            update_available: true,
            message: "Update available".to_string(),
        };

        let output = response.human_fmt();
        assert!(output.contains("0.1.0"));
        assert!(output.contains("0.2.0"));
        assert!(output.contains("lin update"));
    }

    #[test]
    fn test_already_up_to_date_response_human_fmt() {
        let response = AlreadyUpToDateResponse {
            current_version: "0.1.0".to_string(),
            message: "Already up to date".to_string(),
        };

        let output = response.human_fmt();
        assert!(output.contains("0.1.0"));
        assert!(output.contains("up to date"));
    }
}
