use std::{
    fs::{create_dir_all, read_dir, remove_file, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
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
