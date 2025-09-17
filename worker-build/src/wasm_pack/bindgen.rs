//! Functionality related to running `wasm-bindgen`.

use crate::wasm_pack::child;
use crate::wasm_pack::command::build::{BuildProfile, Target};
use crate::wasm_pack::install::{self, Tool};
use crate::wasm_pack::manifest::CrateData;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Run the `wasm-bindgen` CLI to generate bindings for the current crate's
/// `.wasm`.
#[allow(clippy::too_many_arguments)]
pub fn wasm_bindgen_build(
    data: &CrateData,
    install_status: &install::Status,
    out_dir: &Path,
    out_name: &Option<String>,
    disable_dts: bool,
    target: Target,
    profile: BuildProfile,
    extra_args: &[String],
    extra_options: &[String],
) -> Result<()> {
    let profile_name = match profile.clone() {
        BuildProfile::Release | BuildProfile::Profiling => "release",
        BuildProfile::Dev => "debug",
        BuildProfile::Custom(profile_name) => &profile_name.clone(),
    };

    let out_dir = out_dir.to_str().unwrap();

    let target_directory = {
        let mut has_target_dir_iter = extra_options.iter();
        has_target_dir_iter
            .find(|&it| it == "--target-dir")
            .and_then(|_| has_target_dir_iter.next())
            .map(Path::new)
            .unwrap_or(data.target_directory())
    };

    let wasm_path = target_directory
        .join("wasm32-unknown-unknown")
        .join(profile_name)
        .join(data.crate_name())
        .with_extension("wasm");

    let dts_arg = if disable_dts {
        "--no-typescript"
    } else {
        "--typescript"
    };
    let bindgen_path = install::get_tool_path(install_status, Tool::WasmBindgen)?
        .binary(&Tool::WasmBindgen.to_string())?;

    let mut cmd = Command::new(&bindgen_path);
    cmd.arg(&wasm_path)
        .arg("--out-dir")
        .arg(out_dir)
        .arg(dts_arg);

    cmd.arg("--target").arg(target.to_string());

    if let Some(value) = out_name {
        cmd.arg("--out-name").arg(value);
    }

    let profile = data.configured_profile(profile);
    if profile.wasm_bindgen_debug_js_glue() {
        cmd.arg("--debug");
    }
    if !profile.wasm_bindgen_demangle_name_section() {
        cmd.arg("--no-demangle");
    }
    if profile.wasm_bindgen_dwarf_debug_info() {
        cmd.arg("--keep-debug");
    }
    if profile.wasm_bindgen_omit_default_module_path() {
        cmd.arg("--omit-default-module-path");
    }
    if profile.wasm_bindgen_split_linked_modules() {
        cmd.arg("--split-linked-modules");
    }

    for arg in extra_args {
        cmd.arg(arg);
    }

    child::run(cmd, "wasm-bindgen").context("Running the wasm-bindgen CLI")?;
    Ok(())
}
