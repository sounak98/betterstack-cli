use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;

use anyhow::{Context, Result};

use crate::output::CommandOutput;

const REPO: &str = "sounak98/betterstack-cli";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn run() -> Result<CommandOutput> {
    let latest = fetch_latest_version().await?;
    let latest_clean = latest.trim_start_matches('v');

    if latest_clean == CURRENT_VERSION {
        return Ok(CommandOutput::Message(format!(
            "Already up to date (v{CURRENT_VERSION})."
        )));
    }

    eprintln!("Updating bs v{CURRENT_VERSION} -> {latest}");

    let target = detect_target()?;
    let url = format!("https://github.com/{REPO}/releases/download/{latest}/bs-{target}.tar.gz");

    let tmp = tempfile::tempdir().context("Failed to create temp dir")?;
    let tarball = tmp.path().join("bs.tar.gz");

    // Download
    eprint!("Downloading... ");
    let resp = reqwest::get(&url).await?.error_for_status()?;
    let bytes = resp.bytes().await?;
    let mut file = fs::File::create(&tarball)?;
    file.write_all(&bytes)?;
    eprintln!("done.");

    // Extract
    let status = Command::new("tar")
        .args([
            "xzf",
            tarball.to_str().unwrap(),
            "-C",
            tmp.path().to_str().unwrap(),
        ])
        .status()
        .context("Failed to extract archive")?;
    if !status.success() {
        anyhow::bail!("tar extraction failed");
    }

    // Replace current binary
    let current_exe = env::current_exe().context("Failed to detect current binary path")?;
    let new_bin = tmp.path().join("bs");

    // Atomic-ish replace: rename old, move new, delete old
    let backup = current_exe.with_extension("old");
    if backup.exists() {
        fs::remove_file(&backup).ok();
    }
    fs::rename(&current_exe, &backup)
        .context("Failed to replace binary. Try running with appropriate permissions.")?;
    match fs::copy(&new_bin, &current_exe) {
        Ok(_) => {
            fs::remove_file(&backup).ok();
        }
        Err(e) => {
            // Rollback
            fs::rename(&backup, &current_exe).ok();
            anyhow::bail!("Failed to install new binary: {e}");
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&current_exe, fs::Permissions::from_mode(0o755)).ok();
    }

    Ok(CommandOutput::Message(format!(
        "Updated bs to {latest} successfully!"
    )))
}

async fn fetch_latest_version() -> Result<String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "bs-cli")
        .send()
        .await?
        .error_for_status()?;
    let body: serde_json::Value = resp.json().await?;
    body["tag_name"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Could not determine latest version"))
}

fn detect_target() -> Result<String> {
    let os = if cfg!(target_os = "linux") {
        "unknown-linux-gnu"
    } else if cfg!(target_os = "macos") {
        "apple-darwin"
    } else {
        anyhow::bail!("Unsupported OS for self-update");
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        anyhow::bail!("Unsupported architecture for self-update");
    };

    Ok(format!("{arch}-{os}"))
}
