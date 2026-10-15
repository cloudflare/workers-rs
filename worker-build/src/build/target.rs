//! Checking for the wasm32 target

use crate::build::utils;
use crate::build::BuildProfile;
use crate::build::PBAR;
use crate::emoji;
use crate::emscripten::{self, Toolchain};
use crate::versions::{MIN_KEEP_SECTION_RUSTC_VERSION, MIN_RUSTC_VERSION};
use anyhow::{anyhow, bail, Context, Result};
use core::str;
use log::error;
use log::info;
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
    rustc_minor_version_of(Command::new("rustc"))
}

/// The minor version reported by `rustc`, a command that runs rustc without arguments.
fn rustc_minor_version_of(mut rustc: Command) -> Option<u32> {
    macro_rules! otry {
        ($e:expr) => {
            match $e {
                Some(e) => e,
                None => return None,
            }
        };
    }
    let output = otry!(rustc.arg("--version").output().ok());
    let version = otry!(str::from_utf8(&output.stdout).ok());
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    otry!(pieces.next()).parse().ok()
}

/// The minor version of the rustc that `cargo_build_wasm` links with: the toolchain selected in
/// the crate directory, or nightly for panic=unwind builds.
fn link_rustc_minor_version(path: &Path, panic_unwind: bool) -> Option<u32> {
    let mut rustc = Command::new("rustc");
    if panic_unwind {
        rustc.arg("+nightly");
    }
    rustc.current_dir(path);
    rustc_minor_version_of(rustc)
}

/// Emscripten link configuration for `cargo_build_wasm`.
pub struct EmscriptenBuild<'a> {
    pub toolchain: &'a Toolchain,
    pub bin: &'a str,
    /// The `worker/experimental_tokio` feature is enabled: pass the unstable cfgs it builds on.
    pub tokio: bool,
    /// Directory containing the `wasm-bindgen` CLI emcc runs post-link.
    pub bindgen_dir: &'a Path,
}

/// The Wasm section holding the function names that CPU profiles and stack traces show.
const NAME_SECTION: &str = "name";
/// wasm-bindgen reads this section to enable the reference-types transforms that its catch
/// wrappers need, so stripping it breaks bindgen for crates with `#[wasm_bindgen(catch)]`.
const TARGET_FEATURES_SECTION: &str = "target_features";

/// The sections the final link keeps despite `strip = true`; empty means a plain `cargo build`.
fn final_link_sections(
    profile: &BuildProfile,
    emscripten: bool,
    preserve_names: bool,
    rustc_minor: Option<u32>,
) -> &'static [&'static str] {
    let sections: &'static [&'static str] = match profile {
        _ if emscripten => return &[],
        BuildProfile::Dev | BuildProfile::Custom(_) => return &[],
        BuildProfile::Release if !preserve_names => &[TARGET_FEATURES_SECTION],
        BuildProfile::Release | BuildProfile::Profiling => &[NAME_SECTION, TARGET_FEATURES_SECTION],
    };
    // An unknown version passed `check_rustc_version`, so assume a current toolchain.
    if let Some(minor) =
        rustc_minor.filter(|&minor| minor < MIN_KEEP_SECTION_RUSTC_VERSION.minor as u32)
    {
        PBAR.warn(&format!(
            "Rust 1.{minor}'s Wasm linker cannot keep sections that `strip = true` removes, so \
             the Cargo profile decides whether Wasm names and the `target_features` section \
             survive. Use Rust {} or newer to keep them.",
            *MIN_KEEP_SECTION_RUSTC_VERSION
        ));
        return &[];
    }
    sections
}

/// Run `cargo rustc` targetting `wasm32-unknown-unknown`, or
/// `wasm32-unknown-emscripten` with emcc as the linker.
pub fn cargo_build_wasm(
    path: &Path,
    profile: BuildProfile,
    extra_options: &[String],
    panic_unwind: bool,
    preserve_names: bool,
    emscripten: Option<EmscriptenBuild<'_>>,
) -> Result<()> {
    let msg = if panic_unwind {
        format!("{}Compiling to Wasm (with panic=unwind)...", emoji::CYCLONE)
    } else if emscripten.is_some() {
        format!("{}Compiling to Wasm (emscripten)...", emoji::CYCLONE)
    } else {
        format!("{}Compiling to Wasm...", emoji::CYCLONE)
    };
    PBAR.info(&msg);

    let keep_sections = final_link_sections(
        &profile,
        emscripten.is_some(),
        preserve_names,
        link_rustc_minor_version(path, panic_unwind),
    );
    let cmd = cargo_build_command(
        path,
        profile,
        extra_options,
        panic_unwind,
        emscripten,
        keep_sections,
    )?;
    utils::run(cmd, "cargo").with_context(|| {
        if keep_sections.is_empty() {
            "Compiling your crate to WebAssembly failed"
        } else {
            "Compiling your crate to WebAssembly failed. Try `--profile release` to build without \
             `cargo rustc` and `--keep-section`."
        }
    })?;
    Ok(())
}

/// The Cargo invocation for `cargo_build_wasm`. Keeping sections uses `cargo rustc`, so that
/// the `--keep-section` link arguments reach only the final crate.
fn cargo_build_command(
    path: &Path,
    profile: BuildProfile,
    extra_options: &[String],
    panic_unwind: bool,
    emscripten: Option<EmscriptenBuild<'_>>,
    keep_sections: &[&str],
) -> Result<Command> {
    let mut cmd = Command::new("cargo");

    // When panic_unwind is enabled, use nightly toolchain
    if panic_unwind {
        cmd.arg("+nightly");
    }

    cmd.current_dir(path).arg(if keep_sections.is_empty() {
        "build"
    } else {
        "rustc"
    });
    match &emscripten {
        Some(em) => cmd.arg("--bin").arg(em.bin),
        None => cmd.arg("--lib"),
    };

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
        rustflags.extend(
            emscripten::LINK_ARGS
                .iter()
                .map(|arg| format!("-Clink-arg={arg}")),
        );
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
    if !keep_sections.is_empty() {
        // Arguments after `--` reach only the final crate's rustc, so Cargo's own rustflags
        // resolution (CARGO_ENCODED_RUSTFLAGS, RUSTFLAGS, config) is left as is. The final
        // link still covers dependency code, including under LTO.
        cmd.arg("--");
        for section in keep_sections {
            cmd.arg("-C")
                .arg(format!("link-arg=--keep-section={section}"));
        }
    }
    Ok(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEEP_NAME_ARGS: [&str; 5] = [
        "--",
        "-C",
        "link-arg=--keep-section=name",
        "-C",
        "link-arg=--keep-section=target_features",
    ];
    const KEEP_TARGET_FEATURES_ARGS: [&str; 3] =
        ["--", "-C", "link-arg=--keep-section=target_features"];

    /// A toolchain whose rust-lld accepts `--keep-section`.
    const CURRENT_RUSTC_MINOR: Option<u32> = Some(91);

    fn options(options: &[&str]) -> Vec<String> {
        options.iter().map(|option| option.to_string()).collect()
    }

    /// The Cargo command for a non-Emscripten build, as `cargo_build_wasm` constructs it.
    fn command(
        profile: BuildProfile,
        extra: &[&str],
        panic_unwind: bool,
        preserve_names: bool,
    ) -> Result<Command> {
        let extra = options(extra);
        let keep_sections =
            final_link_sections(&profile, false, preserve_names, CURRENT_RUSTC_MINOR);
        cargo_build_command(
            Path::new("."),
            profile,
            &extra,
            panic_unwind,
            None,
            keep_sections,
        )
    }

    fn args(cmd: &Command) -> Vec<String> {
        cmd.get_args()
            .map(|arg| arg.to_str().unwrap().to_string())
            .collect()
    }

    fn env(cmd: &Command, name: &str) -> Option<String> {
        cmd.get_envs()
            .find(|(key, _)| *key == name)
            .and_then(|(_, value)| value)
            .map(|value| value.to_str().unwrap().to_string())
    }

    fn release_args(extra: &[&str], preserve_names: bool) -> Vec<String> {
        args(&command(BuildProfile::Release, extra, false, preserve_names).unwrap())
    }

    #[test]
    fn release_uses_cargo_rustc_keeping_names_and_target_features() {
        let cmd = command(BuildProfile::Release, &["--features", "http"], false, true).unwrap();
        let mut expected = vec![
            "rustc",
            "--lib",
            "--release",
            "--target",
            WASM32_UNKNOWN,
            "--features",
            "http",
        ];
        expected.extend(KEEP_NAME_ARGS);
        assert_eq!(args(&cmd), expected);
        // Cargo's own rustflags resolution must stay intact.
        assert_eq!(env(&cmd, "RUSTFLAGS"), None);
        assert_eq!(env(&cmd, "CARGO_ENCODED_RUSTFLAGS"), None);
    }

    #[test]
    fn release_opt_out_keeps_only_target_features() {
        let mut expected = vec!["rustc", "--lib", "--release", "--target", WASM32_UNKNOWN];
        expected.extend(KEEP_TARGET_FEATURES_ARGS);
        assert_eq!(release_args(&[], false), expected);
    }

    #[test]
    fn dev_and_custom_profiles_use_cargo_build_without_link_args() {
        for preserve_names in [true, false] {
            let dev = command(BuildProfile::Dev, &["--workspace"], false, preserve_names);
            assert_eq!(
                args(&dev.unwrap()),
                ["build", "--lib", "--target", WASM32_UNKNOWN, "--workspace"]
            );
            let custom = command(
                BuildProfile::Custom("perf".into()),
                &[],
                false,
                preserve_names,
            );
            assert_eq!(
                args(&custom.unwrap()),
                [
                    "build",
                    "--lib",
                    "--profile",
                    "perf",
                    "--target",
                    WASM32_UNKNOWN
                ]
            );
        }
    }

    #[test]
    fn profiling_uses_cargo_rustc_with_final_crate_link_args() {
        let cmd = command(
            BuildProfile::Profiling,
            &["--features", "http"],
            false,
            true,
        )
        .unwrap();
        let mut expected = vec![
            "rustc",
            "--lib",
            "--release",
            "--target",
            WASM32_UNKNOWN,
            "--features",
            "http",
        ];
        expected.extend(KEEP_NAME_ARGS);
        assert_eq!(args(&cmd), expected);
        assert_eq!(env(&cmd, "RUSTFLAGS"), None);
    }

    #[test]
    fn panic_unwind_keeps_build_std_and_rustflags() {
        for profile in [BuildProfile::Release, BuildProfile::Profiling] {
            let cmd = command(profile, &[], true, true).unwrap();
            let args = args(&cmd);
            assert_eq!(args[..2], ["+nightly", "rustc"]);
            assert!(args
                .windows(2)
                .any(|pair| pair == ["-Z", "build-std=std,panic_unwind"]));
            assert!(args.ends_with(&KEEP_NAME_ARGS.map(String::from)));
            let rustflags = env(&cmd, "RUSTFLAGS").unwrap();
            assert!(rustflags.ends_with("-Cpanic=unwind"));
            assert!(!rustflags.contains("keep-section"));
        }
    }

    #[test]
    fn toolchains_without_keep_section_keep_no_sections() {
        for profile in [BuildProfile::Release, BuildProfile::Profiling] {
            for preserve_names in [true, false] {
                assert!(final_link_sections(&profile, false, preserve_names, Some(77)).is_empty());
            }
        }
        // Unknown versions passed `check_rustc_version`, as did the first linker with the flag.
        for rustc_minor in [None, Some(78)] {
            let sections = final_link_sections(&BuildProfile::Release, false, true, rustc_minor);
            assert_eq!(sections, [NAME_SECTION, TARGET_FEATURES_SECTION]);
        }
    }

    #[test]
    fn emscripten_keeps_no_sections() {
        for profile in [BuildProfile::Release, BuildProfile::Profiling] {
            assert!(final_link_sections(&profile, true, true, CURRENT_RUSTC_MINOR).is_empty());
        }
    }
}
