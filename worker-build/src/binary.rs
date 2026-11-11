use crate::build::PBAR;
use crate::emoji::{CONFIG, DOWN_ARROW};
use crate::versions::{CUR_ESBUILD_VERSION, CUR_WASM_OPT_VERSION};
use anyhow::{bail, Context, Result};
use flate2::read::GzDecoder;
use heck::ToShoutySnakeCase;
use sha2::{Digest, Sha256};
use std::env;
use std::{
    fmt,
    fs::{create_dir_all, read_dir, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

pub trait GetBinary: BinaryDep {
    /// Get the given binary path for a binary dependency
    /// Returns both the path and a boolean indicating if user provided an override
    fn get_binary(&self, bin_name: Option<&str>) -> Result<(PathBuf, bool)> {
        let full_name = self.full_name();
        let name = self.name();
        let target = self.target();
        let version = self.version();

        // 1. First check {BIN_NAME}_BIN env override
        let check_env_var = bin_name.unwrap_or(name).to_shouty_snake_case() + "_BIN";
        if let Ok(custom_bin) = env::var(&check_env_var) {
            match which::which(&custom_bin) {
                Ok(resolved) => {
                    PBAR.info(&format!(
                        "{CONFIG}Using custom {full_name} from {check_env_var}: {}",
                        resolved.display()
                    ));
                    return Ok((resolved, true));
                }
                Err(_) => {
                    PBAR.warn(&format!("{check_env_var}={custom_bin} not found, falling back to internal {full_name} implementation"));
                }
            }
        }

        // 2. Then check the cache path
        let cache_path = cache_path(name, &version, target)?;
        let bin_path = cache_path.join(self.bin_path(bin_name)?);
        if bin_path.exists() {
            return Ok((bin_path, false));
        }

        // 3. Finally perform a download, clearing cache for this name and target first
        let url = self.download_url();
        let _ = remove_all_versions(name, target);
        PBAR.info(&format!("{DOWN_ARROW}Downloading {full_name}@{version}..."));
        let published = self.published_checksum_url();
        let checksum = match &published {
            Some(published) => Checksum::Published(published),
            None => Checksum::Pinned,
        };
        download_with_retries(&url, checksum, &cache_path, full_name)?;
        if !bin_path.exists() {
            bail!(
                "Unable to locate binary {} in {full_name}",
                bin_path.to_string_lossy()
            );
        }
        Ok((bin_path, false))
    }
}

pub trait BinaryDep: Sized {
    /// Returns the name of the binary
    fn name(&self) -> &'static str;

    /// Returns the full name of the binary
    fn full_name(&self) -> &'static str;

    /// Returns the target of the binary
    fn target(&self) -> &'static str;

    /// Returns the latest current version of the binary
    fn version(&self) -> String;

    /// Returns the URL for the binary to be downloaded
    /// as well as the path string within the archive to use
    fn download_url(&self) -> String;

    /// Returns the URL of the sha256 file the release publishes next to the
    /// download, to check the download against instead of `checksums.txt`
    fn published_checksum_url(&self) -> Option<String> {
        None
    }

    /// Get the relative path of the given binary in the package
    /// If None, returns the default binary
    fn bin_path(&self, name: Option<&str>) -> Result<String>;
}

impl<T: BinaryDep> GetBinary for T {}

const MAYBE_EXE: &str = if cfg!(windows) { ".exe" } else { "" };

/// For clearing the cache, remove all files for the given binary and target
fn remove_all_versions(name: &str, target: &str) -> Result<usize> {
    let prefix_name = format!("{name}-{target}-");
    let dir = dirs_next::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("worker-build");

    let mut deleted_count = 0;
    for entry in read_dir(&dir)
        .with_context(|| format!("Failed to read cache directory {}", dir.display()))?
    {
        let entry = entry?;
        let file_name = entry.file_name();

        if let Some(name_str) = file_name.to_str() {
            if name_str.starts_with(&prefix_name) {
                let path = entry.path();
                if path.is_dir() {
                    std::fs::remove_dir_all(&path).with_context(|| {
                        format!("Failed to remove cached directory {}", path.display())
                    })?;
                } else {
                    std::fs::remove_file(&path).with_context(|| {
                        format!("Failed to remove cached file {}", path.display())
                    })?;
                }
                deleted_count += 1;
            }
        }
    }

    Ok(deleted_count)
}

/// Root of the worker-build cache directory
pub(crate) fn cache_root() -> Result<PathBuf> {
    let path = dirs_next::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("worker-build");
    if !path.exists() {
        create_dir_all(&path)
            .with_context(|| format!("Failed to create cache directory {}", path.display()))?;
    }
    Ok(path)
}

/// Cache path for this binary instance
fn cache_path(name: &str, version: &str, target: &str) -> Result<PathBuf> {
    let path_name = format!("{name}-{target}-{version}");
    let path = cache_root()?.join(&path_name);
    if !path.exists() {
        create_dir_all(&path)
            .with_context(|| format!("Failed to create cache directory {}", path.display()))?;
    }
    Ok(path)
}

#[cfg(target_family = "unix")]
fn fix_permissions(options: &mut OpenOptions) -> &mut OpenOptions {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o755)
}

#[cfg(target_family = "windows")]
fn fix_permissions(options: &mut OpenOptions) -> &mut OpenOptions {
    options
}

/// Maximum number of download attempts before giving up
const MAX_DOWNLOAD_ATTEMPTS: u32 = 5;

/// Download this binary instance into its cache path, retrying transient
/// failures with exponential backoff
fn download_with_retries(
    url: &str,
    checksum: Checksum,
    bin_dir: &Path,
    full_name: &str,
) -> Result<()> {
    let mut attempt = 1;
    loop {
        match download(url, checksum, bin_dir) {
            Ok(()) => return Ok(()),
            Err(err) => {
                if err.is::<ChecksumMismatch>() || err.is::<MissingChecksum>() {
                    return Err(err);
                }
                if attempt >= MAX_DOWNLOAD_ATTEMPTS {
                    return Err(err).context(format!(
                        "Failed to download {full_name} after {MAX_DOWNLOAD_ATTEMPTS} attempts"
                    ));
                }
                let delay = std::time::Duration::from_secs(1 << attempt);
                PBAR.warn(&format!(
                    "Failed to download {full_name} (attempt {attempt}/{MAX_DOWNLOAD_ATTEMPTS}): {err:#}. Retrying in {}s...",
                    delay.as_secs()
                ));
                std::thread::sleep(delay);
                attempt += 1;
            }
        }
    }
}

/// sha256 of each esbuild and Binaryen archive worker-build downloads at the
/// versions it pins, by URL. Regenerate with
/// `node worker-build/scripts/update-checksums.mjs`.
const CHECKSUMS: &str = include_str!("../checksums.txt");

/// What a download is checked against before it is extracted
#[derive(Clone, Copy)]
pub(crate) enum Checksum<'a> {
    /// The sha256 in `checksums.txt`, for the esbuild and Binaryen versions
    /// worker-build pins. A URL not listed there is an error.
    Pinned,
    /// The sha256 in the file the release publishes at this URL. wasm-bindgen
    /// follows the project's Cargo.lock, and its releases are immutable and
    /// publish a `.sha256sum` for each archive. A missing file is an error.
    Published(&'a str),
    /// The URL names a git commit. GitHub does not keep the bytes of the
    /// archives it generates stable, so there is no hash to pin.
    Commit,
}

/// A download whose sha256 is not the expected one. Never retried.
#[derive(Debug)]
struct ChecksumMismatch {
    url: String,
    expected: Expected,
    actual: String,
}

impl fmt::Display for ChecksumMismatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "sha256 mismatch for {}\n  expected: {} ({})\n  actual:   {}",
            self.url, self.expected.sha256, self.expected.from, self.actual
        )
    }
}

impl std::error::Error for ChecksumMismatch {}

/// A download with no sha256 to check it against. Never retried, and nothing
/// is downloaded.
#[derive(Debug)]
struct MissingChecksum(String);

impl fmt::Display for MissingChecksum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for MissingChecksum {}

/// The sha256 a download must have, and where it came from
#[derive(Debug)]
struct Expected {
    sha256: String,
    from: String,
}

/// The sha256 for `url` in `checksums.txt`
fn pinned_sha256(url: &str) -> Option<&'static str> {
    CHECKSUMS
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once("  "))
        .find(|(_, pinned_url)| *pinned_url == url)
        .map(|(hash, _)| hash)
}

/// The sha256 `url` must have, or `None` for a download by commit
fn expected_sha256(agent: &ureq::Agent, url: &str, checksum: Checksum) -> Result<Option<Expected>> {
    let version = env!("CARGO_PKG_VERSION");
    Ok(Some(match checksum {
        Checksum::Pinned => Expected {
            sha256: pinned_sha256(url)
                .ok_or_else(|| {
                    MissingChecksum(format!(
                        "No sha256 pinned for {url} in worker-build {version}"
                    ))
                })?
                .to_owned(),
            from: format!("pinned in worker-build {version}"),
        },
        Checksum::Published(from) => Expected {
            sha256: fetch_published_sha256(agent, from, url)?,
            from: from.to_owned(),
        },
        Checksum::Commit => return Ok(None),
    }))
}

/// Read the `<sha256>  <file name>` file a release publishes at `from` for
/// the download at `url`
fn fetch_published_sha256(agent: &ureq::Agent, from: &str, url: &str) -> Result<String> {
    let text = match agent.get(from).call() {
        Ok(mut res) => res
            .body_mut()
            .read_to_string()
            .with_context(|| format!("Failed to read {from}"))?,
        Err(ureq::Error::StatusCode(404)) => {
            return Err(
                MissingChecksum(format!("No sha256 published for {url}: {from} not found")).into(),
            )
        }
        Err(err) => return Err(err).with_context(|| format!("Failed to fetch URL {from}")),
    };
    text.split_whitespace()
        .next()
        .filter(|hash| hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()))
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| MissingChecksum(format!("No sha256 for {url} in {from}")).into())
}

fn verify_sha256(url: &str, expected: Option<Expected>, actual: String) -> Result<()> {
    match expected {
        Some(expected) if !expected.sha256.eq_ignore_ascii_case(&actual) => Err(ChecksumMismatch {
            url: url.to_owned(),
            expected,
            actual,
        }
        .into()),
        Some(_) => {
            log::info!("Verified sha256 of {url}");
            Ok(())
        }
        None => Ok(()),
    }
}

/// Passes writes through, hashing them
struct HashingWriter<W> {
    inner: W,
    hasher: Sha256,
}

impl<W: Write> Write for HashingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .tls_config(
            ureq::tls::TlsConfig::builder()
                .provider(ureq::tls::TlsProvider::NativeTls)
                .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .new_agent()
}

/// Stream `url` into the file at `path`, returning the sha256 of its bytes
fn fetch_to_file(agent: &ureq::Agent, url: &str, path: &Path) -> Result<String> {
    let mut res = agent
        .get(url)
        .call()
        .with_context(|| format!("Failed to fetch URL {url}"))?;
    let mut writer = HashingWriter {
        inner: File::create(path)
            .with_context(|| format!("Failed to create file {}", path.display()))?,
        hasher: Sha256::new(),
    };
    io::copy(&mut res.body_mut().as_reader(), &mut writer)
        .with_context(|| format!("Failed to download {url}"))?;
    Ok(writer
        .hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// Download a tar.gz archive, check its sha256, then extract it, stripping
/// its top-level directory
pub(crate) fn download(url: &str, checksum: Checksum, bin_dir: &Path) -> Result<()> {
    let agent = agent();
    let expected = expected_sha256(&agent, url, checksum)?;
    create_dir_all(bin_dir)
        .with_context(|| format!("Failed to create directory {}", bin_dir.display()))?;
    let archive_path = bin_dir.join(".download.tar.gz");
    let result = fetch_to_file(&agent, url, &archive_path)
        .and_then(|actual| verify_sha256(url, expected, actual))
        .and_then(|()| extract(File::open(&archive_path)?, bin_dir));
    let _ = std::fs::remove_file(&archive_path);
    result
}

/// Extract a tar.gz archive, stripping its top-level directory
fn extract(archive: impl Read, bin_dir: &Path) -> Result<()> {
    let deflater = GzDecoder::new(archive);
    let mut archive = tar::Archive::new(deflater);

    for entry in archive
        .entries()
        .context("Failed to read archive entries")?
    {
        let mut entry = entry?;
        let path_stripped = entry.path()?.components().skip(1).collect::<PathBuf>();
        let entry_type = entry.header().entry_type();
        // Skip the top-level directory itself, pax headers, and links.
        if path_stripped.as_os_str().is_empty() || !(entry_type.is_dir() || entry_type.is_file()) {
            continue;
        }
        let bin_path = bin_dir.join(path_stripped);

        if entry_type.is_dir() {
            std::fs::create_dir_all(&bin_path)
                .with_context(|| format!("Failed to create directory {}", bin_path.display()))?;
        } else {
            if let Some(parent) = bin_path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory {}", parent.display()))?;
            }

            let mut options = std::fs::OpenOptions::new();
            let options = fix_permissions(&mut options);
            let mut file = options
                .create(true)
                .write(true)
                .truncate(true)
                .open(&bin_path)
                .with_context(|| format!("Failed to create file {}", bin_path.display()))?;
            std::io::copy(&mut entry, &mut file)
                .with_context(|| format!("Failed to extract file {}", bin_path.display()))?;
        }
    }

    Ok(())
}

pub struct Esbuild;

impl BinaryDep for Esbuild {
    fn full_name(&self) -> &'static str {
        "Esbuild"
    }
    fn name(&self) -> &'static str {
        "esbuild"
    }
    fn version(&self) -> String {
        CUR_ESBUILD_VERSION.to_string()
    }
    fn target(&self) -> &'static str {
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
    fn download_url(&self) -> String {
        let version = self.version();
        let target = self.target();
        format!("https://registry.npmjs.org/@esbuild/{target}/-/{target}-{version}.tgz")
    }
    fn bin_path(&self, name: Option<&str>) -> Result<String> {
        Ok(match name {
            None | Some("esbuild") => {
                if cfg!(windows) {
                    format!("esbuild{MAYBE_EXE}")
                } else {
                    format!("bin/esbuild{MAYBE_EXE}")
                }
            }
            Some(name) => bail!("Unknown binary {name} in {}", self.full_name()),
        })
    }
}

pub struct WasmOpt;

impl BinaryDep for WasmOpt {
    fn full_name(&self) -> &'static str {
        "Wasm Opt"
    }
    fn name(&self) -> &'static str {
        "wasm-opt"
    }
    fn version(&self) -> String {
        CUR_WASM_OPT_VERSION.to_owned()
    }
    fn target(&self) -> &'static str {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("macos", "aarch64") => "arm64-macos",
            ("macos", "x86_64") => "x86_64-macos",
            ("linux" | "freebsd" | "netbsd" | "openbsd" | "android", "aarch64") => "aarch64-linux",
            ("linux" | "freebsd" | "netbsd" | "openbsd", "x86_64") => "x86_64-linux",
            ("windows", "aarch64") => "arm64-windows",
            ("windows", "x86_64") => "x86_64-windows",
            _ => panic!("Platform unsupported for {}", self.full_name()),
        }
    }
    fn download_url(&self) -> String {
        let version = self.version();
        let target = self.target();
        format!("https://github.com/WebAssembly/binaryen/releases/download/version_{version}/binaryen-version_{version}-{target}.tar.gz")
    }
    fn bin_path(&self, name: Option<&str>) -> Result<String> {
        Ok(match name {
            None | Some("wasm-opt") => format!("bin/wasm-opt{MAYBE_EXE}"),
            Some(name) => bail!("Unknown binary {name} in {}", self.full_name()),
        })
    }
}

pub struct WasmBindgen<'a>(pub &'a str);

impl BinaryDep for WasmBindgen<'_> {
    fn full_name(&self) -> &'static str {
        "Wasm Bindgen"
    }
    fn name(&self) -> &'static str {
        "wasm-bindgen"
    }
    fn version(&self) -> String {
        self.0.to_owned()
    }
    fn target(&self) -> &'static str {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("macos", "aarch64") => "aarch64-apple-darwin",
            ("macos", "x86_64") => "x86_64-apple-darwin",
            ("linux" | "freebsd" | "netbsd" | "openbsd" | "android", "aarch64") => {
                "aarch64-unknown-linux-musl"
            }
            ("linux" | "freebsd" | "netbsd" | "openbsd", "x86_64") => "x86_64-unknown-linux-musl",
            ("windows", "x86_64" | "aarch64") => "x86_64-pc-windows-msvc",
            _ => panic!("Platform unsupported for {}", self.full_name()),
        }
    }
    fn download_url(&self) -> String {
        let version = self.version();
        let target = self.target();
        format!("https://github.com/wasm-bindgen/wasm-bindgen/releases/download/{version}/wasm-bindgen-{version}-{target}.tar.gz")
    }
    fn published_checksum_url(&self) -> Option<String> {
        Some(format!("{}.sha256sum", self.download_url()))
    }
    fn bin_path(&self, name: Option<&str>) -> Result<String> {
        Ok(match name {
            None | Some("wasm-bindgen") => format!("wasm-bindgen{MAYBE_EXE}"),
            Some("wasm-bindgen-test-runner") => format!("wasm-bindgen-test-runner{MAYBE_EXE}"),
            Some(name) => bail!("Unknown binary {name} in {}", self.full_name()),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::versions::LATEST_WASM_BINDGEN_VERSION;
    use std::collections::HashMap;
    use std::net::TcpListener;

    #[test]
    fn checksums_are_well_formed() {
        let mut seen = std::collections::HashSet::new();
        for line in CHECKSUMS.lines().filter(|line| !line.starts_with('#')) {
            let (hash, url) = line.split_once("  ").unwrap();
            assert!(
                hash.len() == 64 && hash.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')),
                "{line}"
            );
            assert!(url.starts_with("https://"), "{line}");
            assert!(seen.insert(url), "duplicate {url}");
        }
    }

    #[test]
    fn pinned_versions_have_checksums() {
        for url in [Esbuild.download_url(), WasmOpt.download_url()] {
            assert!(
                pinned_sha256(&url).is_some(),
                "{url} is not in checksums.txt; run `node worker-build/scripts/update-checksums.mjs`"
            );
        }
    }

    #[test]
    fn only_wasm_bindgen_uses_published_checksum() {
        assert_eq!(Esbuild.published_checksum_url(), None);
        assert_eq!(WasmOpt.published_checksum_url(), None);
        let wasm_bindgen = WasmBindgen(&LATEST_WASM_BINDGEN_VERSION.to_string());
        assert_eq!(
            wasm_bindgen.published_checksum_url(),
            Some(format!("{}.sha256sum", wasm_bindgen.download_url()))
        );
    }

    fn tar_gz(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::default(),
        ));
        for (path, contents) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o755);
            builder
                .append_data(&mut header, format!("top/{path}"), *contents)
                .unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap()
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// Serve `files` over plain HTTP on localhost, 404 for anything else
    fn serve(files: HashMap<String, Vec<u8>>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let mut request = Vec::new();
                let mut buf = [0; 1024];
                while !request.ends_with(b"\r\n\r\n") {
                    match stream.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => request.extend_from_slice(&buf[..n]),
                    }
                }
                let request = String::from_utf8_lossy(&request);
                let path = request.split(' ').nth(1).unwrap_or_default();
                let (status, body) = match files.get(path) {
                    Some(body) => ("200 OK", body.as_slice()),
                    None => ("404 Not Found", &b""[..]),
                };
                let head = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(body);
            }
        });
        base
    }

    /// A path that does not exist yet, as `get_binary` leaves the cache path
    fn temp_dir(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("wb-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// Download an archive whose release publishes `published` in its
    /// `.sha256sum` file, or no such file for `None`
    fn download_with_published(name: &str, published: Option<&str>) -> (Result<()>, PathBuf) {
        let archive = tar_gz(&[("bin/tool", b"#!/bin/sh\n")]);
        let mut files = HashMap::new();
        if let Some(published) = published {
            let hash = published.replace("{sha256}", &sha256_hex(&archive));
            files.insert(
                "/tool.tar.gz.sha256sum".to_owned(),
                format!("{hash}  tool.tar.gz\n").into_bytes(),
            );
        }
        files.insert("/tool.tar.gz".to_owned(), archive);
        let base = serve(files);
        let dir = temp_dir(name);
        let published_url = format!("{base}/tool.tar.gz.sha256sum");
        let result = download(
            &format!("{base}/tool.tar.gz"),
            Checksum::Published(&published_url),
            &dir,
        );
        (result, dir)
    }

    #[test]
    fn download_matching_published_sha256() {
        let (result, dir) = download_with_published("match", Some("{sha256}"));
        result.unwrap();
        assert!(dir.join("bin/tool").exists());
        assert!(!dir.join(".download.tar.gz").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn download_mismatched_published_sha256_extracts_nothing() {
        let wrong = "0".repeat(64);
        let (result, dir) = download_with_published("mismatch", Some(&wrong));
        let err = result.unwrap_err();
        let mismatch = err.downcast_ref::<ChecksumMismatch>().unwrap();
        assert!(mismatch.url.ends_with("/tool.tar.gz"));
        assert_eq!(mismatch.expected.sha256, wrong);
        assert!(err.to_string().contains(&format!("expected: {wrong}")));
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn download_without_published_sha256_is_an_error() {
        let (result, dir) = download_with_published("unpublished", None);
        let err = result.unwrap_err();
        assert!(err.is::<MissingChecksum>(), "{err:#}");
        assert!(err.to_string().contains("tool.tar.gz.sha256sum"), "{err:#}");
        assert!(!dir.exists());
    }

    #[test]
    fn download_unpinned_url_is_an_error() {
        let base = serve(HashMap::from([(
            "/tool.tar.gz".to_owned(),
            tar_gz(&[("bin/tool", b"#!/bin/sh\n")]),
        )]));
        let dir = temp_dir("unpinned");
        let err = download(&format!("{base}/tool.tar.gz"), Checksum::Pinned, &dir).unwrap_err();
        assert!(err.is::<MissingChecksum>(), "{err:#}");
        assert!(!dir.exists());
    }

    #[test]
    fn verify_pinned_sha256() {
        let url = Esbuild.download_url();
        let pinned = pinned_sha256(&url).unwrap();
        let expected = || {
            expected_sha256(&agent(), &url, Checksum::Pinned)
                .unwrap()
                .unwrap()
        };
        assert_eq!(expected().sha256, pinned);
        verify_sha256(&url, Some(expected()), pinned.to_owned()).unwrap();

        let actual = "f".repeat(64);
        let err = verify_sha256(&url, Some(expected()), actual.clone()).unwrap_err();
        assert!(err.is::<ChecksumMismatch>());
        let message = err.to_string();
        assert!(
            message.contains(&url) && message.contains(pinned) && message.contains(&actual),
            "{message}"
        );
    }
}
