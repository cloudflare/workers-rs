use std::sync::LazyLock;

macro_rules! version {
    ($v:expr) => {
        LazyLock::new(|| semver::Version::parse($v).unwrap())
    };
}

// Current build toolchain, always used exactly for builds, unless overridden by {}_BIN env vars
pub(crate) static CUR_WASM_BINDGEN_VERSION: LazyLock<semver::Version> = version!("0.2.106");
pub(crate) static CUR_WASM_OPT_VERSION: &str = "125";
pub(crate) static CUR_ESBUILD_VERSION: LazyLock<semver::Version> = version!("0.27.0");

// Minimum required libraries, validated before build
pub(crate) static MIN_WASM_BINDGEN_LIB_VERSION: LazyLock<semver::Version> = version!("0.2.106"); // 0.2.106 schema version
pub(crate) static MIN_RUSTC_VERSION: LazyLock<semver::Version> = version!("1.71.0"); // wasm-bindgen MSRV

pub(crate) static MIN_WORKER_LIB_VERSION: LazyLock<semver::Version> = version!(&format!(
    "{}.0",
    env!("CARGO_PKG_VERSION")
        .split('.')
        .collect::<Vec<&str>>()
        .split_last()
        .unwrap()
        .1
        .join(".")
));
pub(crate) static CUR_WORKER_VERSION: LazyLock<semver::Version> =
    version!(env!("CARGO_PKG_VERSION"));
