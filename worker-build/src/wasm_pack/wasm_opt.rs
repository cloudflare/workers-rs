//! Support for downloading and executing `wasm-opt`

use crate::binary::{GetBinary, WasmOpt};
use crate::wasm_pack::child;
use crate::wasm_pack::PBAR;
use anyhow::Result;
use binary_install::Cache;
use std::path::Path;
use std::process::Command;

/// Execute `wasm-opt` over wasm binaries found in `out_dir`, downloading if
/// necessary into `cache`. Passes `args` to each invocation of `wasm-opt`.
pub fn run(
    _cache: &Cache,
    out_dir: &Path,
    args: &[String],
    _install_permitted: bool,
) -> Result<()> {
    let wasm_opt_path = WasmOpt.get_binary(None)?;

    PBAR.info("Optimizing wasm binaries with `wasm-opt`...");

    for file in out_dir.read_dir()? {
        let file = file?;
        let path = file.path();
        if path.extension().and_then(|s| s.to_str()) != Some("wasm") {
            continue;
        }

        let tmp = path.with_extension("wasm-opt.wasm");
        let mut cmd = Command::new(&wasm_opt_path);
        cmd.arg(&path).arg("-o").arg(&tmp).args(args);
        child::run(cmd, "wasm-opt")?;
        std::fs::rename(&tmp, &path)?;
    }

    Ok(())
}
