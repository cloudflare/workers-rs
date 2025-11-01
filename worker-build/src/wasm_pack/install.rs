//! Minimal install module for remaining utilities

use crate::wasm_pack::child;
use anyhow::{bail, Error, Result};
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

/// Fetches the version of a CLI tool
pub fn get_cli_version(tool: &str, path: &Path) -> Result<String> {
    let mut cmd = Command::new(path);
    cmd.arg("--version");
    let stdout = child::run_capture_stdout(cmd, tool)?;
    let version = stdout.split_whitespace().nth(1);
    match version {
        Some(v) => Ok(v.to_string()),
        None => bail!("Something went wrong! We couldn't determine your version of the wasm-bindgen CLI. We were supposed to set that up for you, so it's likely not your fault! You should file an issue: https://github.com/rustwasm/wasm-pack/issues/new?template=bug_report.md.")
    }
}

/// The `InstallMode` determines which mode of initialization we are running, and
/// what install steps we perform.
#[derive(Clone, Copy, Debug, Default)]
pub enum InstallMode {
    /// Perform all the install steps.
    #[default]
    Normal,
    /// Don't install tools like `wasm-bindgen`, just use the global
    /// environment's existing versions to do builds.
    Noinstall,
    /// Skip the rustc version check
    Force,
}

impl FromStr for InstallMode {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "no-install" => Ok(InstallMode::Noinstall),
            "normal" => Ok(InstallMode::Normal),
            "force" => Ok(InstallMode::Force),
            _ => bail!("Unknown build mode: {}", s),
        }
    }
}

impl InstallMode {
    /// Determines if installation is permitted during a function call based on --mode flag
    pub fn install_permitted(self) -> bool {
        match self {
            InstallMode::Normal => true,
            InstallMode::Force => true,
            InstallMode::Noinstall => false,
        }
    }
}
