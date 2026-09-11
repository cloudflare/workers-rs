use std::sync::LazyLock;

macro_rules! version {
    ($v:expr) => {
        LazyLock::new(|| semver::Version::parse($v).unwrap())
    };
}

// Current build toolchain, always used exactly for builds, unless overridden by {}_BIN env vars
pub(crate) static LATEST_WASM_BINDGEN_VERSION: LazyLock<semver::Version> = version!("0.2.128");
pub(crate) static CUR_WASM_OPT_VERSION: &str = "132";
pub(crate) static CUR_EMSCRIPTEN_VERSION: &str = "6.0.9";
pub(crate) static CUR_ESBUILD_VERSION: LazyLock<semver::Version> = version!("0.28.2");

// Minimum required libraries, validated before build
pub(crate) static MIN_WASM_BINDGEN_LIB_VERSION: LazyLock<semver::Version> = version!("0.2.122");
// First release with wasm-bindgen/wasm-bindgen#5328 and #5332
pub(crate) static MIN_EMSCRIPTEN_WASM_BINDGEN_VERSION: LazyLock<semver::Version> =
    version!("0.2.129");
pub(crate) static MIN_RUSTC_VERSION: LazyLock<semver::Version> = version!("1.77.0"); // workers-rs MSRV

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
