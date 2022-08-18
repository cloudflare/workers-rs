use std::{fs::OpenOptions, io::Write, path::PathBuf, process::Command};

use anyhow::Result;
use flate2::read::GzDecoder;

/// Checks if a binary with the specified name is on the user's path.
pub fn is_installed(name: &str) -> Result<Option<PathBuf>> {
    let path = std::env::var_os("PATH").expect("could not read PATH environment variable");
    let path_directories = std::env::split_paths(&path).filter_map(|path| {
        std::fs::metadata(&path)
            .ok()
            .map(|meta| meta.is_dir())
            .unwrap_or(false)
            .then_some(path)
    });

    for dir in path_directories {
        for entry in dir.read_dir()? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let is_file_or_symlink = file_type.is_symlink() || file_type.is_file();

            if is_file_or_symlink && entry.file_name() == name {
                return Ok(Some(entry.path()));
            }
        }
    }

    Ok(None)
}

pub fn ensure_wasm_pack() -> Result<()> {
    if is_installed("wasm-pack")?.is_none() {
        println!("Installing wasm-pack...");
        let exit_status = Command::new("cargo")
            .args(&["install", "wasm-pack"])
            .spawn()?
            .wait()?;

        match exit_status.success() {
            true => Ok(()),
            false => anyhow::bail!(
                "installation of wasm-pack exited with status {}",
                exit_status
            ),
        }
    } else {
        Ok(())
    }
}

const ESBUILD_VERSION: &str = "0.14.47";

pub fn ensure_esbuild() -> Result<PathBuf> {
    // If we already have it we can skip the download.
    if let Some(path) = is_installed("esbuild")? {
        return Ok(path);
    };

    let esbuild_binary = format!("esbuild-{}", platform());
    let esbuild_bin_path = dirs_next::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(esbuild_binary);

    if esbuild_bin_path.exists() {
        return Ok(esbuild_bin_path);
    }

    let mut options = &mut std::fs::OpenOptions::new();

    options = fix_permissions(options);

    let mut file = options.create(true).write(true).open(&esbuild_bin_path)?;

    println!("Installing esbuild...");

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
        "https://registry.npmjs.org/esbuild-{0}/-/esbuild-{0}-{ESBUILD_VERSION}.tgz",
        platform()
    );

    let body = ureq::get(&esbuild_url).call()?.into_reader();
    let deflater = GzDecoder::new(body);
    let mut archive = tar::Archive::new(deflater);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        if path
            .file_name()
            .map(|name| name == "esbuild")
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
pub fn platform() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "x86_64") => "darwin-64",
        ("macos", "aarch64") => "darwin-arm64",
        ("linux", "x86") => "linux-32",
        ("linux", "x86_64") => "linux-64",
        ("linux", "arm") => "linux-arm",
        ("linux", "aarch64") => "linux-arm64",
        ("windows", "x86") => "windows-32",
        ("windows", "x86_64") => "windows-64",
        ("windows", "aarch64") => "windows-arm64",
        _ => panic!("Platform unsupported by esbuild."),
    }
}
