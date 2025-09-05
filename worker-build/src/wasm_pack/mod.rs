//! Your favorite rust -> wasm workflow tool!

#![deny(missing_docs)]

extern crate anyhow;
extern crate cargo_metadata;
extern crate console;
extern crate glob;
extern crate parking_lot;
extern crate semver;
extern crate serde;
extern crate strsim;
extern crate which;

pub(crate) mod bindgen;
pub(crate) mod build;
pub(crate) mod cache;
pub(crate) mod child;
pub(crate) mod command;
pub(crate) mod emoji;
pub(crate) mod install;
pub(crate) mod license;
pub(crate) mod lockfile;
pub(crate) mod manifest;
pub(crate) mod progressbar;
pub(crate) mod readme;
pub(crate) mod target;
pub(crate) mod utils;
pub(crate) mod wasm_opt;

use crate::wasm_pack::progressbar::ProgressOutput;

/// The global progress bar and user-facing message output.
pub(crate) static PBAR: ProgressOutput = ProgressOutput::new();
