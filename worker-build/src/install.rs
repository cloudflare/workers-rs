use std::{
    fs::{create_dir_all, read_dir, remove_file, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use flate2::read::GzDecoder;

fn cache_path(name: &str) -> Result<PathBuf> {
    let path = dirs_next::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("worker-build")
        .join(name);
    let parent = path.parent().unwrap();
    if !parent.exists() {
        create_dir_all(parent)?;
    }
    Ok(path)
}

fn remove_files_with_prefix(path: &Path) -> Result<usize> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let prefix = path.file_name().unwrap().to_str().unwrap();

    let mut deleted_count = 0;

    for entry in read_dir(dir)? {
        let entry = entry?;
        let file_name = entry.file_name();

        if let Some(name_str) = file_name.to_str() {
            if name_str.starts_with(prefix) {
                remove_file(entry.path())?;
                deleted_count += 1;
            }
        }
    }

    Ok(deleted_count)
}

const ESBUILD_VERSION: &str = "0.25.10";
const BINARY_EXTENSION: &str = if cfg!(windows) { ".exe" } else { "" };

pub fn ensure_esbuild() -> Result<PathBuf> {
    let esbuild_prefix = format!("esbuild-{}{BINARY_EXTENSION}", esbuild_platform_pkg());

    let esbuild_binary = format!("{esbuild_prefix}-{ESBUILD_VERSION}");

    let esbuild_bin_path = cache_path(&esbuild_binary)?;

    if esbuild_bin_path.exists() {
        return Ok(esbuild_bin_path);
    }

    // Clear old versions cache
    remove_files_with_prefix(&cache_path(&esbuild_prefix)?)?;

    let mut options = &mut std::fs::OpenOptions::new();
    options = fix_permissions(options);

    let mut file = options.create(true).write(true).open(&esbuild_bin_path)?;

    println!("Installing esbuild {ESBUILD_VERSION}...");

    if let Err(e) = download_esbuild(&mut file) {
        // Make sure we close the file before we remove it.
        drop(file);

        std::fs::remove_file(&esbuild_bin_path)?;
        return Err(e);
    }

    Ok(esbuild_bin_path)
}

fn download_esbuild(writer: &mut impl Write) -> Result<()> {
    let esbuild_url = format!(
        "https://registry.npmjs.org/@esbuild/{0}/-/{0}-{ESBUILD_VERSION}.tgz",
        esbuild_platform_pkg()
    );

    let mut res = ureq::get(&esbuild_url)
        .call()
        .with_context(|| format!("Failed to fetch URL {esbuild_url}"))?;
    let body = res.body_mut().as_reader();
    let deflater = GzDecoder::new(body);
    let mut archive = tar::Archive::new(deflater);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        if path
            .file_name()
            .map(|name| name == format!("esbuild{BINARY_EXTENSION}").as_str())
            .unwrap_or(false)
        {
            std::io::copy(&mut entry, writer)?;
            return Ok(());
        }
    }

    anyhow::bail!("no esbuild binary in archive")
}

#[cfg(target_family = "unix")]
fn fix_permissions(options: &mut OpenOptions) -> &mut OpenOptions {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o770)
}

#[cfg(target_family = "windows")]
fn fix_permissions(options: &mut OpenOptions) -> &mut OpenOptions {
    options
}

/// Converts the user's platform from their Rust representation to their esbuild representation.
/// https://esbuild.github.io/getting-started/#download-a-build
pub fn esbuild_platform_pkg() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("android", "arm") => "android-arm",
        ("android", "aarch64") => "android-arm64",
        ("android", "x86_64") => "android-x64",
        ("macos", "aarch64") => "darwin-arm64",
        ("macos", "x86_64") => "darwin-x64",
        ("freebsd", "aarch64") => "freebsd-arm64",
        ("freebsd", "x86_64") => "freebsd-x64",
        ("linux", "arm") => "linux-arm",
        ("linux", "aarch64") => "linux-arm64",
        ("linux", "x86") => "linux-ia32",
        ("linux", "powerpc64") => "linux-ppc64",
        ("linux", "s390x") => "linux-s390x",
        ("linux", "x86_64") => "linux-x64",
        ("netbsd", "aarch64") => "netbsd-arm64",
        ("netbsd", "x86_64") => "netbsd-x64",
        ("openbsd", "aarch64") => "openbsd-arm64",
        ("openbsd", "x86_64") => "openbsd-x64",
        ("solaris", "x86_64") => "sunos-x64",
        ("windows", "aarch64") => "win32-arm64",
        ("windows", "x86") => "win32-ia32",
        ("windows", "x86_64") => "win32-x64",
        _ => panic!("Platform unsupported by esbuild."),
    }
}

/// Verifies that esbuild is available in PATH and returns the executable name.
/// Returns an error if esbuild is not found or not executable.
pub fn verify_esbuild_in_path() -> Result<PathBuf> {
    let esbuild_name = format!("esbuild{BINARY_EXTENSION}");

    // Try to run 'esbuild --version' to verify it exists and is executable
    let output = Command::new(&esbuild_name)
        .arg("--version")
        .output()
        .with_context(|| format!("Failed to find '{}' in PATH. Please ensure esbuild is installed and available in your PATH when using --mode no-install", esbuild_name))?;

    if !output.status.success() {
        anyhow::bail!("Found '{}' in PATH but it failed to execute. Exit code: {}", esbuild_name, output.status);
    }

    // Parse the version and compare with expected version
    let version_output = String::from_utf8_lossy(&output.stdout);
    let found_version = version_output.trim();

    if let Err(e) = check_version_warning(found_version, ESBUILD_VERSION) {
        eprintln!("Warning: {}", e);
    }

    // Return just the executable name - the system will find it in PATH when executed
    Ok(PathBuf::from(&esbuild_name))
}

/// Checks if the found version is lower than the expected version and returns a warning message.
fn check_version_warning(found_version: &str, expected_version: &str) -> Result<()> {
    match compare_versions(found_version, expected_version) {
        Ok(std::cmp::Ordering::Less) => {
            anyhow::bail!(
                "esbuild version {} found in PATH is lower than the expected version {}. \
                Consider upgrading to avoid potential compatibility issues.",
                found_version,
                expected_version
            )
        }
        Ok(_) => Ok(()),
        Err(_) => {
            // If we can't parse the version, just warn but don't fail
            eprintln!(
                "Warning: Could not parse esbuild version '{}'. Expected version is {}.",
                found_version,
                expected_version
            );
            Ok(())
        }
    }
}

/// Compares two semantic versions (e.g., "0.25.10" vs "0.25.9").
/// Returns Ok(Ordering) if versions can be compared, Err if parsing fails.
fn compare_versions(v1: &str, v2: &str) -> Result<std::cmp::Ordering> {
    let parse_version = |v: &str| -> Result<Vec<u32>> {
        v.split('.')
            .map(|s| s.parse::<u32>().map_err(|e| anyhow::anyhow!("Invalid version component: {}", e)))
            .collect()
    };

    let v1_parts = parse_version(v1)?;
    let v2_parts = parse_version(v2)?;

    // Compare version parts (major, minor, patch, etc.)
    for (a, b) in v1_parts.iter().zip(v2_parts.iter()) {
        match a.cmp(b) {
            std::cmp::Ordering::Equal => continue,
            other => return Ok(other),
        }
    }

    // If all compared parts are equal, the version with fewer parts is considered lower
    Ok(v1_parts.len().cmp(&v2_parts.len()))
}
