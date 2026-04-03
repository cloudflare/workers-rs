//! Checking for the wasm32 target

use crate::build::utils;
use crate::build::BuildProfile;
use crate::build::PBAR;
use crate::emoji;
use crate::versions::MIN_RUSTC_VERSION;
use anyhow::{anyhow, bail, Context, Result};
use core::str;
use log::error;
use log::info;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// The Rust compilation target triple used for the build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WasmTarget {
    /// Standard wasm32-unknown-unknown (default)
    Unknown,
    /// Emscripten target: wasm32-unknown-emscripten
    Emscripten,
}

impl WasmTarget {
    /// Returns the rustc target triple string.
    pub fn triple(self) -> &'static str {
        match self {
            WasmTarget::Unknown => "wasm32-unknown-unknown",
            WasmTarget::Emscripten => "wasm32-unknown-emscripten",
        }
    }
}

impl fmt::Display for WasmTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.triple())
    }
}

struct Wasm32Check {
    target: WasmTarget,
    rustc_path: PathBuf,
    sysroot: PathBuf,
    found: bool,
    is_rustup: bool,
}

impl fmt::Display for Wasm32Check {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let target = self.target.triple();

        if !self.found {
            let rustup_string = if self.is_rustup {
                "It looks like Rustup is being used.".to_owned()
            } else {
                format!("It looks like Rustup is not being used. For non-Rustup setups, the {target} target needs to be installed manually.")
            };

            writeln!(
                f,
                "{} target not found in sysroot: {:?}",
                target, self.sysroot
            )
            .and_then(|_| {
                writeln!(
                    f,
                    "\nUsed rustc from the following path: {:?}",
                    self.rustc_path
                )
            })
            .and_then(|_| writeln!(f, "{rustup_string}"))
        } else {
            write!(
                f,
                "sysroot: {:?}, rustc path: {:?}, was found: {}, isRustup: {}",
                self.sysroot, self.rustc_path, self.found, self.is_rustup
            )
        }
    }
}

/// Ensure that `rustup` has the requested wasm target installed for
/// current toolchain
pub fn check_for_wasm32_target(wasm_target: WasmTarget) -> Result<()> {
    let msg = format!(
        "{}Checking for the Wasm target ({})...",
        emoji::TARGET,
        wasm_target
    );
    PBAR.info(&msg);

    // Check if wasm32 target is present, otherwise bail.
    match check_wasm32_target(wasm_target) {
        Ok(ref wasm32_check) if wasm32_check.found => Ok(()),
        Ok(wasm32_check) => bail!("{wasm32_check}"),
        Err(err) => Err(err),
    }
}

/// Get rustc's sysroot as a PathBuf
fn get_rustc_sysroot() -> Result<PathBuf> {
    let command = Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()?;

    if command.status.success() {
        Ok(String::from_utf8(command.stdout)?.trim().into())
    } else {
        Err(anyhow!(
            "Getting rustc's sysroot wasn't successful. Got {}",
            command.status
        ))
    }
}

/// Get the target libdir for the given wasm target
fn get_rustc_wasm_target_libdir(wasm_target: WasmTarget) -> Result<PathBuf> {
    let triple = wasm_target.triple();
    let command = Command::new("rustc")
        .args(["--target", triple, "--print", "target-libdir"])
        .output()?;

    if command.status.success() {
        Ok(String::from_utf8(command.stdout)?.trim().into())
    } else {
        Err(anyhow!(
            "Getting rustc's {triple} target wasn't successful. Got {}",
            command.status
        ))
    }
}

fn does_wasm_target_libdir_exist(wasm_target: WasmTarget) -> bool {
    let triple = wasm_target.triple();
    let result = get_rustc_wasm_target_libdir(wasm_target);

    match result {
        Ok(wasm_target_libdir_path) => {
            if wasm_target_libdir_path.exists() {
                info!("Found {triple} in {wasm_target_libdir_path:?}");
                true
            } else {
                info!("Failed to find {triple} in {wasm_target_libdir_path:?}");
                false
            }
        }
        Err(_) => {
            error!("Some error in getting the target libdir!");
            false
        }
    }
}

fn check_wasm32_target(wasm_target: WasmTarget) -> Result<Wasm32Check> {
    let sysroot = get_rustc_sysroot()?;
    let rustc_path = which::which("rustc")?;

    if does_wasm_target_libdir_exist(wasm_target) {
        Ok(Wasm32Check {
            target: wasm_target,
            rustc_path,
            sysroot,
            found: true,
            is_rustup: false,
        })
    // If it doesn't exist, then we need to check if we're using rustup.
    } else {
        // If sysroot contains "rustup", then we can assume we're using rustup
        // and use rustup to add the target.
        if sysroot.to_string_lossy().contains("rustup") {
            rustup_add_wasm_target(wasm_target).map(|()| Wasm32Check {
                target: wasm_target,
                rustc_path,
                sysroot,
                found: true,
                is_rustup: true,
            })
        } else {
            Ok(Wasm32Check {
                target: wasm_target,
                rustc_path,
                sysroot,
                found: false,
                is_rustup: false,
            })
        }
    }
}

/// Add the given wasm target using `rustup`.
fn rustup_add_wasm_target(wasm_target: WasmTarget) -> Result<()> {
    let triple = wasm_target.triple();
    let mut cmd = Command::new("rustup");
    cmd.arg("target").arg("add").arg(triple);
    utils::run(cmd, "rustup").with_context(|| format!("Adding the {triple} target with rustup"))?;

    Ok(())
}

/// Ensure that `rustc` is present and that it is >= 1.30.0
pub fn check_rustc_version() -> Result<String> {
    let local_minor_version = rustc_minor_version();
    match local_minor_version {
        Some(mv) => {
            if mv < MIN_RUSTC_VERSION.minor as u32 {
                bail!(
                    "Your version of Rust, '1.{}', is not supported. Please install Rust version {} or higher.",
                    mv,
                    *MIN_RUSTC_VERSION
                )
            } else {
                Ok(mv.to_string())
            }
        }
        None => bail!("We can't figure out what your Rust version is- which means you might not have Rust installed. Please install Rust version 1.30.0 or higher."),
    }
}

// from https://github.com/alexcrichton/proc-macro2/blob/79e40a113b51836f33214c6d00228934b41bd4ad/build.rs#L44-L61
fn rustc_minor_version() -> Option<u32> {
    macro_rules! otry {
        ($e:expr) => {
            match $e {
                Some(e) => e,
                None => return None,
            }
        };
    }
    let output = otry!(Command::new("rustc").arg("--version").output().ok());
    let version = otry!(str::from_utf8(&output.stdout).ok());
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    otry!(pieces.next()).parse().ok()
}

/// Run `cargo build` targeting the given wasm target.
///
/// If `link_objects` is non-empty, `cargo rustc` is used instead so extra
/// `-Clink-arg=` flags can be injected for the final link step without
/// overriding the user's `[target.*].rustflags` in `.cargo/config.toml`.
pub fn cargo_build_wasm(
    path: &Path,
    profile: BuildProfile,
    extra_options: &[String],
    panic_unwind: bool,
    wasm_target: WasmTarget,
    link_objects: &[PathBuf],
) -> Result<()> {
    let triple = wasm_target.triple();
    let msg = if panic_unwind {
        format!(
            "{}Compiling to Wasm ({triple}, panic=unwind)...",
            emoji::CYCLONE
        )
    } else {
        format!("{}Compiling to Wasm ({triple})...", emoji::CYCLONE)
    };
    PBAR.info(&msg);

    let mut cmd = Command::new("cargo");

    // When panic_unwind is enabled, use nightly toolchain
    if panic_unwind {
        cmd.arg("+nightly");
    }

    if link_objects.is_empty() {
        cmd.current_dir(path).arg("build").arg("--lib");
    } else {
        // Use `cargo rustc` so we can append `-- -Clink-arg=<obj>` flags
        // without overriding the user's target-level rustflags.
        cmd.current_dir(path).arg("rustc").arg("--lib");
    }

    if PBAR.quiet() {
        cmd.arg("--quiet");
    }

    match profile {
        BuildProfile::Profiling => {
            // Once there are DWARF debug info consumers, force enable debug
            // info, because builds that use the release cargo profile disables
            // debug info.
            //
            // cmd.env("RUSTFLAGS", "-g");
            cmd.arg("--release");
        }
        BuildProfile::Release => {
            cmd.arg("--release");
        }
        BuildProfile::Dev => {
            // Plain cargo builds use the dev cargo profile, which includes
            // debug info by default.
        }
        BuildProfile::Custom(arg) => {
            cmd.arg("--profile").arg(arg);
        }
    }

    cmd.arg("--target").arg(triple);

    // When panic_unwind is enabled, we need to rebuild std with panic=unwind support
    if panic_unwind {
        cmd.arg("-Z").arg("build-std=std,panic_unwind");

        // Get existing RUSTFLAGS and append panic=unwind
        let existing_rustflags = std::env::var("RUSTFLAGS").unwrap_or_default();
        let new_rustflags = if existing_rustflags.is_empty() {
            "-Cpanic=unwind".to_string()
        } else {
            format!("{existing_rustflags} -Cpanic=unwind")
        };
        cmd.env("RUSTFLAGS", new_rustflags);
    }

    // The `cargo` command is executed inside the directory at `path`, so relative paths set via extra options won't work.
    // To remedy the situation, all detected paths are converted to absolute paths.
    let mut handle_path = false;
    let extra_options_with_absolute_paths = extra_options
        .iter()
        .map(|option| -> Result<String> {
            let value = if handle_path && Path::new(option).is_relative() {
                std::env::current_dir()?
                    .join(option)
                    .to_str()
                    .ok_or_else(|| anyhow!("path contains non-UTF-8 characters"))?
                    .to_string()
            } else {
                option.to_string()
            };
            handle_path = matches!(&**option, "--target-dir" | "--out-dir" | "--manifest-path");
            Ok(value)
        })
        .collect::<Result<Vec<_>>>()?;
    cmd.args(extra_options_with_absolute_paths);

    // Append extra link objects via `-- -Clink-arg=<obj>` (cargo rustc mode).
    if !link_objects.is_empty() {
        cmd.arg("--");
        for obj in link_objects {
            cmd.arg(format!("-Clink-arg={}", obj.display()));
        }
    }

    utils::run(cmd, "cargo build").context("Compiling your crate to WebAssembly failed")?;
    Ok(())
}

/// If the `EMSDK` environment variable is set, prepend its directories to
/// `PATH` so that `emcc` and friends are available to child processes.
///
/// Prepends `<emsdk>`, `<emsdk>/upstream/emscripten`, and any python dir
/// under `<emsdk>/python/` to `PATH`.
///
/// Returns `true` if the PATH was modified.
fn try_add_emsdk_to_path() -> bool {
    // If emcc is already reachable, nothing to do.
    if which::which("emcc").is_ok() {
        return false;
    }

    let root = match std::env::var_os("EMSDK").map(PathBuf::from) {
        Some(p) => p,
        None => return false,
    };

    let emcc = root.join("upstream/emscripten/emcc");
    if !emcc.exists() {
        return false;
    }

    info!("Using emsdk at {}", root.display());

    let mut new_dirs = vec![root.clone(), root.join("upstream/emscripten")];

    // Also pick up the bundled python if present.
    if let Ok(entries) = std::fs::read_dir(root.join("python")) {
        for entry in entries.flatten() {
            let p = entry.path().join("bin");
            if p.is_dir() {
                new_dirs.push(p);
            }
        }
    }

    let current = std::env::var_os("PATH").unwrap_or_default();
    let mut all: Vec<PathBuf> = new_dirs;
    all.extend(std::env::split_paths(&current));
    if let Ok(joined) = std::env::join_paths(&all) {
        std::env::set_var("PATH", &joined);
        info!("Prepended emsdk directories to PATH");
        return true;
    }

    false
}

/// Check that `emcc` is available on PATH, auto-detecting the emsdk if needed.
pub fn check_for_emcc() -> Result<()> {
    let msg = format!("{}Checking for emcc (Emscripten)...", emoji::TARGET);
    PBAR.info(&msg);

    // Try auto-detection first.
    try_add_emsdk_to_path();

    match which::which("emcc") {
        Ok(path) => {
            info!("Found emcc at {path:?}");
            Ok(())
        }
        Err(_) => {
            bail!(
                "emcc not found on PATH. The Emscripten SDK (emsdk) is required for --emscripten builds.\n\
                 \n\
                 Set the EMSDK environment variable to your emsdk installation path\n\
                 (e.g. by running `source <emsdk>/emsdk_env.sh`) and re-run your build."
            )
        }
    }
}
