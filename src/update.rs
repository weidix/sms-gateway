use std::{
    cmp::Ordering,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::Deserialize;

const REPO: &str = "weidix/sms-gateway";
const BIN_NAME: &str = "sms-gateway";

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub async fn run_update() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    let client = Client::builder()
        .user_agent(format!("{}/{}", BIN_NAME, current_version))
        .build()?;

    println!("Checking for updates...");
    let release = fetch_latest_release(&client, REPO).await?;
    let latest_version = normalize_version(&release.tag_name);
    if compare_versions(&latest_version, current_version) != Ordering::Greater {
        println!("Already up to date ({}).", current_version);
        return Ok(());
    }

    println!(
        "Update available: {} -> {}",
        current_version, latest_version
    );
    let asset = select_asset(&release.assets)?;
    println!("Downloading {}...", asset.name);

    let temp_dir = create_temp_dir()?;
    let download_path = temp_dir.join(&asset.name);
    download_asset(&client, &asset.browser_download_url, &download_path).await?;

    let new_binary = prepare_binary(&download_path, &temp_dir)?;
    ensure_executable(&new_binary)?;
    replace_current_binary(&new_binary)?;

    let _ = fs::remove_dir_all(&temp_dir);
    println!("Updated to {}.", latest_version);
    Ok(())
}

async fn fetch_latest_release(client: &Client, repo: &str) -> Result<Release> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        return Err(anyhow!(
            "failed to fetch latest release: {}",
            response.status()
        ));
    }
    response
        .json::<Release>()
        .await
        .context("failed to parse release response")
}

fn normalize_version(tag: &str) -> String {
    let trimmed = tag.trim();
    let trimmed = trimmed.strip_prefix('v').unwrap_or(trimmed);
    trimmed
        .split(|ch| ch == '-' || ch == '+')
        .next()
        .unwrap_or(trimmed)
        .to_string()
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    let left_parts = parse_version(left);
    let right_parts = parse_version(right);
    let max_len = left_parts.len().max(right_parts.len());
    for index in 0..max_len {
        let left_value = *left_parts.get(index).unwrap_or(&0);
        let right_value = *right_parts.get(index).unwrap_or(&0);
        match left_value.cmp(&right_value) {
            Ordering::Equal => continue,
            ordering => return ordering,
        }
    }
    Ordering::Equal
}

fn parse_version(version: &str) -> Vec<u64> {
    version
        .split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}

fn select_asset(assets: &[Asset]) -> Result<&Asset> {
    if assets.is_empty() {
        return Err(anyhow!("latest release has no assets"));
    }

    let mut candidates: Vec<&Asset> = assets
        .iter()
        .filter(|asset| is_download_asset(&asset.name))
        .collect();
    if candidates.is_empty() {
        return Err(anyhow!("latest release has no downloadable assets"));
    }

    let bin_lower = BIN_NAME.to_lowercase();
    let by_bin: Vec<&Asset> = candidates
        .iter()
        .copied()
        .filter(|asset| asset.name.to_lowercase().contains(&bin_lower))
        .collect();
    if !by_bin.is_empty() {
        candidates = by_bin;
    }

    let target = target_triple();
    let target_lower = target.to_lowercase();
    if let Some(asset) = candidates
        .iter()
        .find(|asset| asset.name.to_lowercase().contains(&target_lower))
    {
        return Ok(*asset);
    }

    let os_tokens = os_tokens();
    let arch_tokens = arch_tokens();
    let os_arch_matches: Vec<&Asset> = candidates
        .iter()
        .copied()
        .filter(|asset| {
            let name = asset.name.to_lowercase();
            os_tokens.iter().any(|token| name.contains(token))
                && arch_tokens.iter().any(|token| name.contains(token))
        })
        .collect();
    if os_arch_matches.len() == 1 {
        return Ok(os_arch_matches[0]);
    }

    if candidates.len() == 1 {
        return Ok(candidates[0]);
    }

    Err(anyhow!(
        "could not determine which release asset matches this platform (target {})",
        target
    ))
}

fn is_download_asset(name: &str) -> bool {
    let lower = name.to_lowercase();
    let blocked_suffixes = [".sha256", ".sha256sum", ".sig", ".asc", ".md5"];
    if blocked_suffixes
        .iter()
        .any(|suffix| lower.ends_with(suffix))
    {
        return false;
    }
    !lower.contains("checksum")
}

fn target_triple() -> String {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    match os {
        "linux" => {
            let env = if cfg!(target_env = "musl") {
                "musl"
            } else if cfg!(target_env = "gnu") {
                "gnu"
            } else {
                "unknown"
            };
            format!("{}-unknown-linux-{}", arch, env)
        }
        "macos" => format!("{}-apple-darwin", arch),
        "windows" => {
            let env = if cfg!(target_env = "msvc") {
                "msvc"
            } else if cfg!(target_env = "gnu") {
                "gnu"
            } else {
                "unknown"
            };
            format!("{}-pc-windows-{}", arch, env)
        }
        _ => format!("{}-{}", arch, os),
    }
}

fn os_tokens() -> Vec<String> {
    match std::env::consts::OS {
        "linux" => vec!["linux".to_string()],
        "macos" => vec!["darwin".to_string(), "macos".to_string(), "osx".to_string()],
        "windows" => vec![
            "windows".to_string(),
            "win32".to_string(),
            "win64".to_string(),
        ],
        other => vec![other.to_string()],
    }
}

fn arch_tokens() -> Vec<String> {
    match std::env::consts::ARCH {
        "x86_64" => vec!["x86_64".to_string(), "amd64".to_string()],
        "aarch64" => vec!["aarch64".to_string(), "arm64".to_string()],
        "arm" => vec!["arm".to_string(), "armv7".to_string(), "armv7l".to_string()],
        other => vec![other.to_string()],
    }
}

fn create_temp_dir() -> Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("failed to read system time")?
        .as_millis();
    let dir = std::env::temp_dir().join(format!("sms-gateway-update-{}", timestamp));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

async fn download_asset(client: &Client, url: &str, dest: &Path) -> Result<()> {
    let response = client.get(url).send().await?.error_for_status()?;
    let bytes = response.bytes().await?;
    fs::write(dest, &bytes).context("failed to write downloaded asset")
}

fn prepare_binary(download_path: &Path, work_dir: &Path) -> Result<PathBuf> {
    let name = download_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let lower = name.to_lowercase();

    if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        let extract_dir = work_dir.join("extract");
        fs::create_dir_all(&extract_dir)?;
        let status = Command::new("tar")
            .arg("-xzf")
            .arg(download_path)
            .arg("-C")
            .arg(&extract_dir)
            .status()
            .context("failed to run tar")?;
        if !status.success() {
            return Err(anyhow!("tar extraction failed"));
        }
        return find_binary(&extract_dir);
    }

    if lower.ends_with(".zip") {
        let extract_dir = work_dir.join("extract");
        fs::create_dir_all(&extract_dir)?;
        let status = Command::new("unzip")
            .arg("-q")
            .arg(download_path)
            .arg("-d")
            .arg(&extract_dir)
            .status()
            .context("failed to run unzip")?;
        if !status.success() {
            return Err(anyhow!("unzip extraction failed"));
        }
        return find_binary(&extract_dir);
    }

    Ok(download_path.to_path_buf())
}

fn find_binary(root: &Path) -> Result<PathBuf> {
    let mut matches = Vec::new();
    let mut names = vec![BIN_NAME.to_string()];
    if cfg!(windows) {
        names.push(format!("{}.exe", BIN_NAME));
    }
    collect_binaries(root, &names, &mut matches)?;
    match matches.len() {
        0 => Err(anyhow!("could not find {} in extracted archive", BIN_NAME)),
        1 => Ok(matches.remove(0)),
        _ => Err(anyhow!(
            "multiple {} binaries found in extracted archive",
            BIN_NAME
        )),
    }
}

fn collect_binaries(dir: &Path, names: &[String], matches: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_binaries(&path, names, matches)?;
        } else if file_type.is_file() {
            if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                if names.iter().any(|name| name == file_name) {
                    matches.push(path);
                }
            }
        }
    }
    Ok(())
}

fn ensure_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

fn replace_current_binary(new_binary: &Path) -> Result<()> {
    let current_exe = std::env::current_exe().context("failed to locate current binary")?;
    if cfg!(windows) {
        return Err(anyhow!(
            "self-update is not supported on Windows; downloaded binary at {}",
            new_binary.display()
        ));
    }

    let backup = current_exe.with_extension("old");
    if backup.exists() {
        let _ = fs::remove_file(&backup);
    }

    fs::rename(&current_exe, &backup).context("failed to backup current binary")?;
    if let Err(err) = fs::rename(new_binary, &current_exe) {
        if let Err(copy_err) = fs::copy(new_binary, &current_exe) {
            let _ = fs::rename(&backup, &current_exe);
            return Err(anyhow!(
                "failed to replace binary: {}; copy failed: {}",
                err,
                copy_err
            ));
        }
    }
    ensure_executable(&current_exe)?;
    Ok(())
}
