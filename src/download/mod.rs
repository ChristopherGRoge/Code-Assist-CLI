use anyhow::{anyhow, Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

const GCS_BUCKET: &str = "https://storage.googleapis.com/claude-code-dist-86c565f3-f756-42ad-8dfa-d59b1c096819/claude-code-releases";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DownloadSource {
    Remote,
    LocalFallback,
}

/// Get the latest version from remote or local fallback
pub fn get_latest_version(local_dir: &Path) -> Result<(String, DownloadSource)> {
    // Try remote first
    let url = format!("{}/latest", GCS_BUCKET);
    match reqwest::blocking::get(&url) {
        Ok(response) if response.status().is_success() => {
            let version = response.text()?.trim().to_string();
            return Ok((version, DownloadSource::Remote));
        }
        _ => {}
    }

    // Fall back to local
    let local_path = local_dir.join("latest");
    if local_path.exists() {
        println!(
            "  {} Remote unavailable, using local fallback",
            style("!").yellow().bold()
        );
        let version = std::fs::read_to_string(&local_path)
            .context("Failed to read local version file")?
            .trim()
            .to_string();
        return Ok((version, DownloadSource::LocalFallback));
    }

    Err(anyhow!("Could not get version from remote or local fallback"))
}

/// Get the manifest for a version
pub fn get_manifest(version: &str, local_dir: &Path) -> Result<(serde_json::Value, DownloadSource)> {
    // Try remote first
    let url = format!("{}/{}/manifest.json", GCS_BUCKET, version);
    match reqwest::blocking::get(&url) {
        Ok(response) if response.status().is_success() => {
            let manifest: serde_json::Value = response.json()?;
            return Ok((manifest, DownloadSource::Remote));
        }
        _ => {}
    }

    // Fall back to local
    let local_path = local_dir.join(version).join("manifest.json");
    if local_path.exists() {
        println!(
            "  {} Remote unavailable, using local manifest",
            style("!").yellow().bold()
        );
        let content = std::fs::read_to_string(&local_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&content)?;
        return Ok((manifest, DownloadSource::LocalFallback));
    }

    Err(anyhow!("Could not get manifest from remote or local fallback"))
}

/// Download binary with fallback to local
pub fn download_binary(
    version: &str,
    platform: &str,
    binary_name: &str,
    local_dir: &Path,
    output_path: &Path,
    expected_checksum: &str,
) -> Result<DownloadSource> {
    // Try remote first
    let url = format!("{}/{}/{}/{}", GCS_BUCKET, version, platform, binary_name);

    println!("  Downloading {}...", style(binary_name).cyan());

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("  {spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Connecting to remote server...");

    let remote_result = download_from_url(&url, output_path, &pb);

    if remote_result.is_ok() {
        pb.finish_and_clear();
        // Verify checksum
        if verify_checksum(output_path, expected_checksum)? {
            println!(
                "  {} Downloaded and verified",
                style("✓").green().bold()
            );
            return Ok(DownloadSource::Remote);
        } else {
            std::fs::remove_file(output_path).ok();
            println!(
                "  {} Checksum verification failed, trying local fallback",
                style("!").yellow().bold()
            );
        }
    } else {
        pb.finish_and_clear();
        println!(
            "  {} Remote download failed, trying local fallback",
            style("!").yellow().bold()
        );
    }

    // Fall back to local
    let local_path = local_dir.join(version).join(platform).join(binary_name);
    if local_path.exists() {
        std::fs::copy(&local_path, output_path)
            .context("Failed to copy local binary")?;

        if verify_checksum(output_path, expected_checksum)? {
            println!(
                "  {} Using local fallback (verified)",
                style("✓").green().bold()
            );
            return Ok(DownloadSource::LocalFallback);
        } else {
            std::fs::remove_file(output_path).ok();
            return Err(anyhow!("Local fallback checksum verification failed"));
        }
    }

    Err(anyhow!("Remote unavailable and no local fallback found"))
}

fn download_from_url(url: &str, output_path: &Path, pb: &ProgressBar) -> Result<()> {
    let response = reqwest::blocking::get(url)?;

    if !response.status().is_success() {
        return Err(anyhow!("HTTP error: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);

    if total_size > 0 {
        pb.set_length(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  {spinner:.cyan} [{bar:30.cyan/dim}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("█▓░"),
        );
    }

    let mut file = std::fs::File::create(output_path)?;
    let mut downloaded: u64 = 0;

    let mut reader = response;
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        std::io::Write::write_all(&mut file, &buffer[..bytes_read])?;
        downloaded += bytes_read as u64;
        pb.set_position(downloaded);
    }

    Ok(())
}

fn verify_checksum(file_path: &Path, expected: &str) -> Result<bool> {
    let mut file = std::fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let actual = hex::encode(hasher.finalize());
    Ok(actual == expected)
}
