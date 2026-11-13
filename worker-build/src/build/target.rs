//! Checking for the wasm32 target

use crate::build::utils;
use crate::build::BuildProfile;
use crate::build::PBAR;
use crate::emoji;
use crate::emscripten::{self, Toolchain};
use crate::versions::MIN_RUSTC_VERSION;
use anyhow::{anyhow, bail, Context, Result};
use cargo_metadata::ArtifactDebuginfo;
use core::str;
use log::error;
use log::info;
use serde::Deserialize;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

const NIGHTLY_TOOLCHAIN: &str = "nightly";

pub const WASM32_UNKNOWN: &str = "wasm32-unknown-unknown";
pub const WASM32_EMSCRIPTEN: &str = "wasm32-unknown-emscripten";

struct Wasm32Check {
    target: &'static str,
    rustc_path: PathBuf,
    sysroot: PathBuf,
    found: bool,
    is_rustup: bool,
}

impl fmt::Display for Wasm32Check {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let target = self.target;

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

/// Ensure that `rustup` has the given wasm32 target installed for the
/// current toolchain
pub fn check_for_wasm32_target(target: &'static str) -> Result<()> {
    let msg = format!("{}Checking for the Wasm target...", emoji::TARGET);
    PBAR.info(&msg);

    // Check if wasm32 target is present, otherwise bail.
    match check_wasm32_target(target) {
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

/// Get the target libdir for a wasm32 target
fn get_rustc_wasm32_target_libdir(target: &str) -> Result<PathBuf> {
    let command = Command::new("rustc")
        .args(["--target", target, "--print", "target-libdir"])
        .output()?;

    if command.status.success() {
        Ok(String::from_utf8(command.stdout)?.trim().into())
    } else {
        Err(anyhow!(
            "Getting rustc's {target} target wasn't successful. Got {}",
            command.status
        ))
    }
}

fn does_wasm32_target_libdir_exist(target: &str) -> bool {
    let result = get_rustc_wasm32_target_libdir(target);

    match result {
        Ok(wasm32_target_libdir_path) => {
            if wasm32_target_libdir_path.exists() {
                info!("Found {target} in {wasm32_target_libdir_path:?}");
                true
            } else {
                info!("Failed to find {target} in {wasm32_target_libdir_path:?}");
                false
            }
        }
        Err(_) => {
            error!("Some error in getting the target libdir!");
            false
        }
    }
}

fn check_wasm32_target(target: &'static str) -> Result<Wasm32Check> {
    let sysroot = get_rustc_sysroot()?;
    let rustc_path = which::which("rustc")?;

    if does_wasm32_target_libdir_exist(target) {
        Ok(Wasm32Check {
            target,
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
            rustup_add_wasm_target(target).map(|()| Wasm32Check {
                target,
                rustc_path,
                sysroot,
                found: true,
                is_rustup: true,
            })
        } else {
            Ok(Wasm32Check {
                target,
                rustc_path,
                sysroot,
                found: false,
                is_rustup: false,
            })
        }
    }
}

/// Add a wasm32 target using `rustup`.
fn rustup_add_wasm_target(target: &str) -> Result<()> {
    let mut cmd = Command::new("rustup");
    cmd.arg("target").arg("add").arg(target);
    utils::run(cmd, "rustup").with_context(|| format!("Adding the {target} target with rustup"))?;

    Ok(())
}

/// Ensure that the nightly toolchain is installed and has the `rust-src` component
/// and `wasm32-unknown-unknown` target, which are required for `-Z build-std`.
pub fn check_nightly_prerequisites() -> Result<()> {
    let msg = format!(
        "{}Checking nightly toolchain prerequisites for panic=unwind...",
        emoji::TARGET
    );
    PBAR.info(&msg);

    let nightly_sysroot = get_nightly_sysroot()?;

    if !nightly_sysroot.exists() {
        install_nightly_toolchain()?;
    }

    if !has_rust_src_component()? {
        install_rust_src_component()?;
    }

    if !does_nightly_wasm32_target_exist() {
        rustup_add_wasm_target_nightly()?;
    }

    Ok(())
}

fn get_nightly_sysroot() -> Result<PathBuf> {
    let command = Command::new("rustc")
        .args(["+nightly", "--print", "sysroot"])
        .output()?;

    if command.status.success() {
        Ok(String::from_utf8(command.stdout)?.trim().into())
    } else {
        Err(anyhow!(
            "Getting nightly rustc's sysroot wasn't successful. Got {}",
            command.status
        ))
    }
}

fn install_nightly_toolchain() -> Result<()> {
    let msg = format!(
        "{}Installing nightly toolchain via rustup...",
        emoji::TARGET
    );
    PBAR.info(&msg);

    let mut cmd = Command::new("rustup");
    cmd.arg("toolchain").arg("install").arg(NIGHTLY_TOOLCHAIN);
    utils::run(cmd, "rustup").context("Installing the nightly toolchain with rustup")?;

    Ok(())
}

fn has_rust_src_component() -> Result<bool> {
    let command = Command::new("rustup")
        .args(["component", "list", "--toolchain", NIGHTLY_TOOLCHAIN])
        .output()?;

    if !command.status.success() {
        return Ok(false);
    }

    let stdout = String::from_utf8(command.stdout)?;
    Ok(stdout
        .lines()
        .any(|line| line.starts_with("rust-src") && line.contains("(installed)")))
}

fn install_rust_src_component() -> Result<()> {
    let msg = format!(
        "{}Installing rust-src component for nightly toolchain...",
        emoji::TARGET
    );
    PBAR.info(&msg);

    let mut cmd = Command::new("rustup");
    cmd.arg("component")
        .arg("add")
        .arg("rust-src")
        .arg("--toolchain")
        .arg(NIGHTLY_TOOLCHAIN);
    utils::run(cmd, "rustup").context("Adding the rust-src component with rustup")?;

    Ok(())
}

fn does_nightly_wasm32_target_exist() -> bool {
    let command = Command::new("rustc")
        .args([
            "+nightly",
            "--target",
            "wasm32-unknown-unknown",
            "--print",
            "target-libdir",
        ])
        .output();

    match command {
        Ok(output) if output.status.success() => {
            let path: PathBuf = String::from_utf8(output.stdout)
                .ok()
                .map(|s| s.trim().into())
                .unwrap_or_default();
            path.exists()
        }
        _ => false,
    }
}

fn rustup_add_wasm_target_nightly() -> Result<()> {
    let msg = format!(
        "{}Adding wasm32-unknown-unknown target for nightly toolchain...",
        emoji::TARGET
    );
    PBAR.info(&msg);

    let mut cmd = Command::new("rustup");
    cmd.arg("target")
        .arg("add")
        .arg("wasm32-unknown-unknown")
        .arg("--toolchain")
        .arg(NIGHTLY_TOOLCHAIN);
    utils::run(cmd, "rustup")
        .context("Adding the wasm32-unknown-unknown target for nightly with rustup")?;

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

/// Append to a space-separated environment variable, preserving user flags.
fn append_env(name: &str, extra: String) -> String {
    match std::env::var(name) {
        Ok(existing) if !existing.is_empty() => format!("{existing} {extra}"),
        _ => extra,
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

/// Emscripten link configuration for `cargo_build_wasm`.
pub struct EmscriptenBuild<'a> {
    pub toolchain: &'a Toolchain,
    pub bin: &'a str,
    /// The `worker/experimental_tokio` feature is enabled: pass the unstable cfgs it builds on.
    pub tokio: bool,
    /// Directory containing the `wasm-bindgen` CLI emcc runs post-link.
    pub bindgen_dir: &'a Path,
    /// Have emcc write a source map beside the linked wasm.
    pub source_map: bool,
}

/// The `-Clink-arg`s that configure the emcc link.
fn emscripten_link_args(source_map: bool) -> Vec<String> {
    let mut args: Vec<String> = emscripten::LINK_ARGS
        .iter()
        .map(|arg| format!("-Clink-arg={arg}"))
        .collect();
    if source_map {
        // This follows rustc's own `-g0` or `--profiling-funcs`, and a later
        // `-g0` would turn the source map off again. Unlike `-g`, this keeps
        // DWARF out of the final module, so Binaryen optimizes fully.
        args.push("-Clink-arg=-gsource-map".into());
    }
    args
}

/// Run `cargo build` targetting `wasm32-unknown-unknown`, or
/// `wasm32-unknown-emscripten` with emcc as the linker. For builds with a
/// source map, returns the `bin`'s debug level as Cargo resolved it.
pub fn cargo_build_wasm(
    path: &Path,
    profile: BuildProfile,
    extra_options: &[String],
    panic_unwind: bool,
    emscripten: Option<EmscriptenBuild<'_>>,
) -> Result<Option<ArtifactDebuginfo>> {
    let msg = if panic_unwind {
        format!("{}Compiling to Wasm (with panic=unwind)...", emoji::CYCLONE)
    } else if emscripten.is_some() {
        format!("{}Compiling to Wasm (emscripten)...", emoji::CYCLONE)
    } else {
        format!("{}Compiling to Wasm...", emoji::CYCLONE)
    };
    PBAR.info(&msg);

    let mut cmd = Command::new("cargo");

    // When panic_unwind is enabled, use nightly toolchain
    if panic_unwind {
        cmd.arg("+nightly");
    }

    cmd.current_dir(path).arg("build");
    match &emscripten {
        Some(em) => cmd.arg("--bin").arg(em.bin),
        None => cmd.arg("--lib"),
    };

    if PBAR.quiet() {
        cmd.arg("--quiet");
    }

    match &profile {
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

    cmd.arg("--target").arg(if emscripten.is_some() {
        WASM32_EMSCRIPTEN
    } else {
        WASM32_UNKNOWN
    });

    // Tell wasm-bindgen's proc-macro to use `js_sys::futures` instead of
    // `wasm_bindgen_futures`. We pass this as an environment variable rather
    // than `--cfg` because Cargo does not propagate `RUSTFLAGS`/`--cfg` to
    // host proc-macros when `--target` is in use. The proc-macro reads this
    // env var at expansion time.
    cmd.env("WASM_BINDGEN_USE_JS_SYS", "1");

    // When panic_unwind is enabled, rebuild std with panic=unwind and pass
    // `-Cpanic=unwind`. Combine with any user-provided RUSTFLAGS so we
    // don't clobber their flags.
    let mut rustflags: Vec<String> = Vec::new();
    if panic_unwind {
        cmd.arg("-Z").arg("build-std=std,panic_unwind");
        rustflags.push("-Cpanic=unwind".into());
    }

    if let Some(em) = &emscripten {
        rustflags.extend(emscripten::RUSTFLAGS.iter().map(|f| f.to_string()));
        if em.tokio {
            rustflags.extend(emscripten::TOKIO_RUSTFLAGS.iter().map(|f| f.to_string()));
        }
        rustflags.extend(emscripten_link_args(em.source_map));
        cmd.env(
            "CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_LINKER",
            em.toolchain.emcc(),
        );
        cmd.env("EM_CONFIG", &em.toolchain.em_config);
        cmd.env(
            "EMCC_CFLAGS",
            append_env("EMCC_CFLAGS", emscripten::EMCC_CFLAGS.join(" ")),
        );
        // Binaryen's worker threads get small stacks on macOS and overflow on
        // large modules; the main thread's stack is fine.
        if cfg!(target_os = "macos") && std::env::var_os("BINARYEN_CORES").is_none() {
            cmd.env("BINARYEN_CORES", "1");
        }
        let mut paths = vec![
            em.toolchain.emscripten_dir.clone(),
            em.bindgen_dir.to_path_buf(),
        ];
        paths.extend(
            std::env::var_os("PATH")
                .map(|p| std::env::split_paths(&p).collect::<Vec<_>>())
                .unwrap_or_default(),
        );
        cmd.env(
            "PATH",
            std::env::join_paths(paths).context("Building PATH for emcc")?,
        );
    }

    if !rustflags.is_empty() {
        cmd.env("RUSTFLAGS", append_env("RUSTFLAGS", rustflags.join(" ")));
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

    let mut debuginfo = None;
    if let Some(em) = emscripten.filter(|em| em.source_map) {
        let levels = read_levels(&cmd, em.bin);
        let override_set =
            std::env::var_os(profile_env_var(&profile, "BUILD_OVERRIDE_DEBUG")).is_some();
        cmd.envs(build_env(levels.as_ref(), &profile, override_set));
        debuginfo = levels.and_then(|levels| levels.bin);
    }

    utils::run(cmd, "cargo build").context("Compiling your crate to WebAssembly failed")?;
    Ok(debuginfo)
}

/// The debug levels Cargo resolved for the build.
#[derive(Debug, Default, PartialEq)]
struct Levels {
    /// The `bin` target's.
    bin: Option<ArtifactDebuginfo>,
    /// Build scripts' and proc macros', if they all have the same one.
    host: Option<ArtifactDebuginfo>,
}

/// Read the debug levels Cargo resolves for `build` from its unit graph. On
/// failure, warns and returns `None`.
fn read_levels(build: &Command, bin: &str) -> Option<Levels> {
    let mut cmd = unit_graph_command(build);
    info!("Running {cmd:?}");
    let error = match cmd.output() {
        Ok(output) if output.status.success() => match levels(&output.stdout, bin) {
            Ok(levels) => return Some(levels),
            Err(e) => format!("{e:#}"),
        },
        Ok(output) => String::from_utf8_lossy(&output.stderr).trim().to_string(),
        Err(e) => e.to_string(),
    };
    PBAR.warn(&format!(
        "worker-build couldn't read the Cargo profile's debug level, so it builds as \
         configured: {error}"
    ));
    None
}

/// `cargo build` with `--unit-graph`, which prints the units Cargo would
/// build, with their resolved profiles, and builds nothing. The flag is
/// unstable, so this call alone gets `RUSTC_BOOTSTRAP=1`, and it doesn't use
/// Cargo's cache of rustc's output, which would keep nightly-only cfgs from
/// that run for the build.
fn unit_graph_command(build: &Command) -> Command {
    let mut cmd = Command::new(build.get_program());
    if let Some(dir) = build.get_current_dir() {
        cmd.current_dir(dir);
    }
    for (name, value) in build.get_envs() {
        match value {
            Some(value) => cmd.env(name, value),
            None => cmd.env_remove(name),
        };
    }
    cmd.args(build.get_args())
        .args(["--unit-graph", "-Zunstable-options"])
        .env("RUSTC_BOOTSTRAP", "1")
        .env("CARGO_CACHE_RUSTC_INFO", "0");
    cmd
}

/// The parts of Cargo's unit graph that `levels` reads.
#[derive(Deserialize)]
struct UnitGraph {
    version: u32,
    units: Vec<Unit>,
}

#[derive(Deserialize)]
struct Unit {
    target: UnitTarget,
    profile: UnitProfile,
    /// The target triple; `None` for the host.
    platform: Option<String>,
    mode: String,
}

#[derive(Deserialize)]
struct UnitTarget {
    name: String,
    kind: Vec<String>,
}

#[derive(Deserialize)]
struct UnitProfile {
    debuginfo: ArtifactDebuginfo,
}

/// The debug levels in Cargo's unit graph.
fn levels(graph: &[u8], bin: &str) -> Result<Levels> {
    let graph: UnitGraph = serde_json::from_slice(graph).context("Invalid unit graph")?;
    if graph.version != 1 {
        bail!("Unknown unit graph version {}", graph.version);
    }
    let mut levels = Levels::default();
    let mut host = Vec::new();
    for unit in graph.units.into_iter().filter(|unit| unit.mode == "build") {
        let is = |kind: &str| unit.target.kind.iter().any(|k| k == kind);
        if unit.platform.as_deref() == Some(WASM32_EMSCRIPTEN)
            && unit.target.name == bin
            && is("bin")
        {
            levels.bin = Some(unit.profile.debuginfo);
        } else if unit.platform.is_none() && (is("custom-build") || is("proc-macro")) {
            host.push(unit.profile.debuginfo);
        }
    }
    if levels.bin.is_none() {
        bail!("Cargo's unit graph has no `{bin}` bin");
    }
    if host.windows(2).all(|pair| pair[0] == pair[1]) {
        levels.host = host.pop();
    }
    Ok(levels)
}

/// The environment to add to the build for the debug levels Cargo resolved.
/// Below line tables, it raises the profile's `debug`, and keeps build
/// scripts and proc macros at their level, so that the build compiles them
/// as it would have.
fn build_env(
    levels: Option<&Levels>,
    profile: &BuildProfile,
    override_set: bool,
) -> Vec<(String, &'static str)> {
    let Some(levels) = levels.filter(|c| c.bin.as_ref().is_some_and(needs_line_tables)) else {
        return Vec::new();
    };
    // emcc builds the source map from DWARF line tables.
    let mut env = vec![(profile_env_var(profile, "DEBUG"), "line-tables-only")];
    if let Some(host) = levels.host.as_ref().and_then(debug_value) {
        if !override_set {
            env.push((profile_env_var(profile, "BUILD_OVERRIDE_DEBUG"), host));
        }
    }
    env
}

/// Whether `debuginfo` is below the line tables a source map needs.
fn needs_line_tables(debuginfo: &ArtifactDebuginfo) -> bool {
    matches!(
        debuginfo,
        ArtifactDebuginfo::None | ArtifactDebuginfo::LineDirectivesOnly
    )
}

/// `debuginfo` as a Cargo profile's `debug` value.
fn debug_value(debuginfo: &ArtifactDebuginfo) -> Option<&'static str> {
    Some(match debuginfo {
        ArtifactDebuginfo::None => "0",
        ArtifactDebuginfo::Limited => "1",
        ArtifactDebuginfo::Full => "2",
        ArtifactDebuginfo::LineTablesOnly => "line-tables-only",
        ArtifactDebuginfo::LineDirectivesOnly => "line-directives-only",
        _ => return None,
    })
}

/// The environment variable that sets `key` in the Cargo profile that
/// `profile` builds with.
fn profile_env_var(profile: &BuildProfile, key: &str) -> String {
    let name = match profile {
        BuildProfile::Release | BuildProfile::Profiling => "release",
        BuildProfile::Dev => "dev",
        BuildProfile::Custom(name) => name,
    };
    format!(
        "CARGO_PROFILE_{}_{key}",
        name.to_uppercase().replace('-', "_")
    )
}

#[cfg(test)]
mod test {
    use super::{
        build_env, debug_value, emscripten_link_args, levels, needs_line_tables, profile_env_var,
        unit_graph_command, ArtifactDebuginfo, BuildProfile, Levels,
    };
    use std::path::{Path, PathBuf};
    use std::process::Command;

    #[test]
    fn test_emscripten_link_args_source_map_off() {
        let args = emscripten_link_args(false);
        assert!(args.iter().all(|arg| arg.starts_with("-Clink-arg=")));
        assert!(!args.iter().any(|arg| arg.contains("-g")));
    }

    #[test]
    fn test_emscripten_link_args_source_map_last() {
        let off = emscripten_link_args(false).len();
        let args = emscripten_link_args(true);
        assert_eq!(&args[off..], ["-Clink-arg=-gsource-map"]);
    }

    #[test]
    fn test_profile_env_var() {
        let var = |profile| profile_env_var(&profile, "DEBUG");
        assert_eq!(var(BuildProfile::Release), "CARGO_PROFILE_RELEASE_DEBUG");
        assert_eq!(var(BuildProfile::Profiling), "CARGO_PROFILE_RELEASE_DEBUG");
        assert_eq!(var(BuildProfile::Dev), "CARGO_PROFILE_DEV_DEBUG");
        assert_eq!(
            var(BuildProfile::Custom("release-small".into())),
            "CARGO_PROFILE_RELEASE_SMALL_DEBUG"
        );
        assert_eq!(
            profile_env_var(&BuildProfile::Release, "BUILD_OVERRIDE_DEBUG"),
            "CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_DEBUG"
        );
    }

    #[test]
    fn test_needs_line_tables() {
        assert!(needs_line_tables(&ArtifactDebuginfo::None));
        assert!(needs_line_tables(&ArtifactDebuginfo::LineDirectivesOnly));
        assert!(!needs_line_tables(&ArtifactDebuginfo::LineTablesOnly));
        assert!(!needs_line_tables(&ArtifactDebuginfo::Limited));
        assert!(!needs_line_tables(&ArtifactDebuginfo::Full));
    }

    #[test]
    fn test_debug_value() {
        for (debuginfo, value) in [
            (ArtifactDebuginfo::None, Some("0")),
            (ArtifactDebuginfo::Limited, Some("1")),
            (ArtifactDebuginfo::Full, Some("2")),
            (ArtifactDebuginfo::LineTablesOnly, Some("line-tables-only")),
            (
                ArtifactDebuginfo::LineDirectivesOnly,
                Some("line-directives-only"),
            ),
            (ArtifactDebuginfo::UnknownInt(3), None),
        ] {
            assert_eq!(debug_value(&debuginfo), value, "{debuginfo}");
        }
    }

    #[test]
    fn test_build_env() {
        let release = BuildProfile::Release;
        let levels = |bin, host| Levels {
            bin: Some(bin),
            host,
        };
        let raise = || {
            (
                "CARGO_PROFILE_RELEASE_DEBUG".to_string(),
                "line-tables-only",
            )
        };
        let host = (
            "CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_DEBUG".to_string(),
            "0",
        );
        let none = Some(ArtifactDebuginfo::None);
        assert_eq!(
            build_env(
                Some(&levels(ArtifactDebuginfo::None, none.clone())),
                &release,
                false
            ),
            [raise(), host]
        );
        let directives = levels(ArtifactDebuginfo::LineDirectivesOnly, None);
        assert_eq!(build_env(Some(&directives), &release, false), [raise()]);
        // A build override the user set, or host units at several levels.
        let zero = levels(ArtifactDebuginfo::None, none);
        assert_eq!(build_env(Some(&zero), &release, true), [raise()]);
        let mixed = levels(ArtifactDebuginfo::None, None);
        assert_eq!(build_env(Some(&mixed), &release, false), [raise()]);
        for kept in [
            ArtifactDebuginfo::LineTablesOnly,
            ArtifactDebuginfo::Limited,
            ArtifactDebuginfo::Full,
        ] {
            let levels = levels(kept, Some(ArtifactDebuginfo::None));
            assert!(build_env(Some(&levels), &release, false).is_empty());
        }
        // The probe failed, or didn't report the bin.
        assert!(build_env(None, &release, false).is_empty());
        assert!(build_env(Some(&Levels::default()), &release, false).is_empty());
    }

    #[test]
    fn test_unit_graph_command() {
        let mut build = Command::new("cargo");
        build
            .current_dir("/w")
            .env("RUSTFLAGS", "-Cx")
            .env_remove("CARGO_TARGET_DIR")
            .args(["+nightly", "build", "--bin", "app", "-v"]);
        let probe = unit_graph_command(&build);
        let args: Vec<_> = probe.get_args().map(|a| a.to_str().unwrap()).collect();
        assert_eq!(
            args,
            [
                "+nightly",
                "build",
                "--bin",
                "app",
                "-v",
                "--unit-graph",
                "-Zunstable-options"
            ]
        );
        assert_eq!(probe.get_current_dir().unwrap(), Path::new("/w"));
        let env: Vec<_> = probe.get_envs().collect();
        assert!(env.contains(&("RUSTFLAGS".as_ref(), Some("-Cx".as_ref()))));
        assert!(env.contains(&("CARGO_TARGET_DIR".as_ref(), None)));
        assert!(env.contains(&("RUSTC_BOOTSTRAP".as_ref(), Some("1".as_ref()))));
        assert!(env.contains(&("CARGO_CACHE_RUSTC_INFO".as_ref(), Some("0".as_ref()))));
        // Only on the probe.
        assert!(build
            .get_envs()
            .all(|(name, _)| name != "RUSTC_BOOTSTRAP" && name != "CARGO_CACHE_RUSTC_INFO"));
    }

    /// A unit of Cargo's unit graph.
    fn unit(name: &str, kind: &str, mode: &str, platform: &str, debuginfo: &str) -> String {
        format!(
            r#"{{"pkg_id":"path+file:///w#x@0.1.0","target":{{"kind":["{kind}"],"crate_types":["{kind}"],"name":"{name}","src_path":"/w/src/main.rs","edition":"2021","doc":true,"doctest":false,"test":true}},"profile":{{"name":"release","opt_level":"s","lto":"false","codegen_backend":null,"codegen_units":null,"debuginfo":{debuginfo},"split_debuginfo":null,"debug_assertions":false,"overflow_checks":false,"rpath":false,"incremental":false,"panic":"abort","strip":{{"deferred":"None"}}}},"platform":{platform},"mode":"{mode}","features":[],"dependencies":[]}}"#
        )
    }

    const WASM: &str = r#""wasm32-unknown-emscripten""#;

    /// A unit graph of `units`, at `version`.
    fn graph(version: u32, units: &[String]) -> Vec<u8> {
        format!(
            r#"{{"version":{version},"units":[{}],"roots":[0]}}"#,
            units.join(",")
        )
        .into_bytes()
    }

    #[test]
    fn test_levels() {
        for (debuginfo, expected) in [
            ("0", ArtifactDebuginfo::None),
            (r#""none""#, ArtifactDebuginfo::None),
            ("null", ArtifactDebuginfo::None),
            (
                r#""line-directives-only""#,
                ArtifactDebuginfo::LineDirectivesOnly,
            ),
            (r#""line-tables-only""#, ArtifactDebuginfo::LineTablesOnly),
            ("1", ArtifactDebuginfo::Limited),
            (r#""limited""#, ArtifactDebuginfo::Limited),
            ("2", ArtifactDebuginfo::Full),
            (r#""full""#, ArtifactDebuginfo::Full),
        ] {
            let units = [
                // A lib and a bin of the same crate.
                unit("app", "lib", "build", WASM, "2"),
                unit("app", "bin", "build", WASM, debuginfo),
                unit("worker", "lib", "build", WASM, "2"),
                unit("build-script-build", "custom-build", "build", "null", "0"),
                unit("worker_macros", "proc-macro", "build", "null", "0"),
                // Running a build script, and a lib for the host.
                unit(
                    "build-script-build",
                    "custom-build",
                    "run-custom-build",
                    "null",
                    "2",
                ),
                unit("syn", "lib", "build", "null", "2"),
            ];
            assert_eq!(
                levels(&graph(1, &units), "app").unwrap(),
                Levels {
                    bin: Some(expected),
                    host: Some(ArtifactDebuginfo::None),
                },
                "{debuginfo}"
            );
        }
    }

    #[test]
    fn test_levels_host_levels() {
        let bin = unit("app", "bin", "build", WASM, "0");
        let mixed = [
            bin.clone(),
            unit("build-script-build", "custom-build", "build", "null", "0"),
            unit("worker_macros", "proc-macro", "build", "null", "2"),
        ];
        assert_eq!(levels(&graph(1, &mixed), "app").unwrap().host, None);
        assert_eq!(levels(&graph(1, &[bin]), "app").unwrap().host, None);
    }

    #[test]
    fn test_levels_fails() {
        let bin = unit("app", "bin", "build", WASM, "0");
        assert!(levels(&graph(1, std::slice::from_ref(&bin)), "app").is_ok());
        assert!(levels(&graph(2, std::slice::from_ref(&bin)), "app").is_err());
        assert!(levels(&graph(1, &[bin]), "other").is_err());
        let host_bin = unit("app", "bin", "build", "null", "0");
        assert!(levels(&graph(1, &[host_bin]), "app").is_err());
        let lib = unit("app", "lib", "build", WASM, "0");
        assert!(levels(&graph(1, &[lib]), "app").is_err());
        assert!(levels(b"error: no such command", "app").is_err());
    }

    /// A temporary directory, removed when dropped.
    struct TempDir(PathBuf);

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Pins the unit graph's format and debug levels for each `debug`
    /// setting on stable Cargo with `RUSTC_BOOTSTRAP=1`, that the profile's
    /// environment variable raises the level, and that `--config` beats that.
    #[test]
    fn test_cargo_unit_graph_debuginfo() {
        let dir =
            TempDir(std::env::temp_dir().join(format!("wb-unit-graph-{}", std::process::id())));
        let dir = &dir.0;
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let levels = |debug: &str, env: Option<&str>, args: &[&str]| {
            std::fs::write(
                dir.join("Cargo.toml"),
                format!(
                    "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\
                     [workspace]\n[profile.release]\ndebug = {debug}\n"
                ),
            )
            .unwrap();
            let mut build = Command::new(&cargo);
            build
                .args(["build", "--release", "--offline"])
                .args(["--target", "wasm32-unknown-emscripten"])
                .args(args)
                .current_dir(dir)
                .env("CARGO_TARGET_DIR", dir.join("target"))
                // An empty Cargo home: no registries or credentials.
                .env("CARGO_HOME", dir.join("cargo-home"))
                .env_remove("RUSTC_BOOTSTRAP")
                .env_remove("CARGO_PROFILE_RELEASE_DEBUG");
            if let Some(env) = env {
                build.env("CARGO_PROFILE_RELEASE_DEBUG", env);
            }
            let output = unit_graph_command(&build).output().unwrap();
            assert!(output.status.success(), "{debug}");
            levels(&output.stdout, "app").unwrap().bin
        };
        for (debug, expected) in [
            ("0", ArtifactDebuginfo::None),
            (
                "\"line-directives-only\"",
                ArtifactDebuginfo::LineDirectivesOnly,
            ),
            ("\"line-tables-only\"", ArtifactDebuginfo::LineTablesOnly),
            ("1", ArtifactDebuginfo::Limited),
            ("2", ArtifactDebuginfo::Full),
        ] {
            assert_eq!(levels(debug, None, &[]), Some(expected), "{debug}");
        }
        let raised = Some(ArtifactDebuginfo::LineTablesOnly);
        assert_eq!(levels("0", Some("line-tables-only"), &[]), raised);
        let config = ["--config", "profile.release.debug=0"];
        assert_eq!(
            levels("0", Some("line-tables-only"), &config),
            Some(ArtifactDebuginfo::None)
        );
    }
}
