//! Emscripten toolchain provisioning for `--emscripten` builds.
//!
//! A pinned emsdk release is installed under the worker-build cache directory
//! and the patches in `worker-build/patches/emscripten/` are applied to its
//! frontend. Patches are backports the Rust link depends on that the pinned
//! release does not yet contain; each is dropped when the pin moves past it.

use crate::binary::{cache_root, download};
use crate::build::PBAR;
use crate::emoji::{CONFIG, DOWN_ARROW};
use crate::versions::CUR_EMSCRIPTEN_VERSION;
use anyhow::{anyhow, bail, Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PATCHES: &[(&str, &str)] = &[
    (
        "wasm-bindgen-marker.patch",
        include_str!("../patches/emscripten/wasm-bindgen-marker.patch"),
    ),
    (
        "noderawsockets-dns.patch",
        include_str!("../patches/emscripten/noderawsockets-dns.patch"),
    ),
];

const STAMP: &str = ".worker-build-patches";

/// Codegen flags for every target crate.
pub const RUSTFLAGS: &[&str] = &[
    "-Crelocation-model=static",
    // exnref exception handling throughout: EMCC_CFLAGS gives the C side
    // `-fwasm-exceptions -sWASM_LEGACY_EXCEPTIONS=0`, the prebuilt std is
    // translated at link, and wasm-bindgen's JSPI wrappers use try_table.
    // V8 rejects a module mixing the two encodings.
    "-Cllvm-args=-wasm-use-legacy-eh=false",
];

/// emcc settings for the final link. Kept out of EMCC_CFLAGS so they do not
/// reach C compiles of crates like `ring`, where `-Werror` makes an unused link
/// setting fatal.
pub const LINK_ARGS: &[&str] = &[
    "-sBINARYEN_EXTRA_PASSES=--translate-to-exnref",
    "-sWASM_BINDGEN",
    "-sJSPI",
    "-sMODULARIZE=instance",
    "-sEXPORT_ES6",
    "-sAUTO_INIT",
    "-sSOURCE_PHASE_IMPORTS",
    "-sENVIRONMENT=node",
    "-sNODERAWSOCKETS",
    // With assertions on, shell.js auto-detects the environment at runtime and
    // misdetects workerd (which exposes WorkerGlobalScope) as a web worker.
    "-sASSERTIONS=0",
    "-sALLOW_MEMORY_GROWTH=1",
];

pub const EMCC_CFLAGS: &[&str] = &["-fwasm-exceptions", "-sWASM_LEGACY_EXCEPTIONS=0"];

pub struct Toolchain {
    /// Directory containing `emcc`.
    pub emscripten_dir: PathBuf,
    pub em_config: PathBuf,
}

impl Toolchain {
    pub fn emcc(&self) -> PathBuf {
        self.emscripten_dir.join("emcc")
    }
}

/// Locate or install the toolchain.
///
/// `EMSCRIPTEN` selects a frontend checkout used as-is (never patched) and
/// `EMSDK` a backend (LLVM, Binaryen, Node) install; when only one is given
/// the other comes from the pinned emsdk under the cache directory.
pub fn provision() -> Result<Toolchain> {
    let env_frontend = env::var_os("EMSCRIPTEN").map(PathBuf::from);
    let env_emsdk = env::var_os("EMSDK").map(PathBuf::from);

    let emsdk = match env_emsdk {
        Some(dir) => {
            PBAR.info(&format!("{CONFIG}Using EMSDK: {}", dir.display()));
            if !dir.join("upstream/bin/clang").exists() {
                bail!(
                    "EMSDK={} has no installed backend at upstream/bin",
                    dir.display()
                );
            }
            dir
        }
        None => provision_emsdk(env_frontend.is_none())?,
    };

    let emscripten_dir = match env_frontend {
        Some(dir) => {
            PBAR.info(&format!("{CONFIG}Using EMSCRIPTEN: {}", dir.display()));
            if !dir.join("emcc").exists() {
                bail!("EMSCRIPTEN={} does not contain emcc", dir.display());
            }
            dir
        }
        None => emsdk.join("upstream/emscripten"),
    };

    let em_config = cache_root()?.join(format!("emscripten-{CUR_EMSCRIPTEN_VERSION}.config"));
    write_config(&em_config, &emsdk)?;

    Ok(Toolchain {
        emscripten_dir,
        em_config,
    })
}

/// Install the pinned emsdk release into the cache, patching its frontend when
/// it is the one that will be used.
fn provision_emsdk(patch_frontend: bool) -> Result<PathBuf> {
    let dir = cache_root()?.join(format!("emsdk-{CUR_EMSCRIPTEN_VERSION}"));
    let frontend = dir.join("upstream/emscripten");
    let stamp = frontend.join(STAMP);
    let expected_stamp = stamp_contents();

    let installed = dir.join("upstream/bin/clang").exists() && frontend.join("emcc").exists();
    let stamp_ok = fs::read_to_string(&stamp).ok().as_deref() == Some(&expected_stamp);
    if installed && (stamp_ok || !patch_frontend) {
        return Ok(dir);
    }

    let python = which::which("python3")
        .or_else(|_| which::which("python"))
        .map_err(|_| anyhow!("python3 is required to install the Emscripten SDK"))?;

    if dir.exists() {
        // A stale or unstamped install cannot be patched incrementally.
        fs::remove_dir_all(&dir).with_context(|| format!("Failed to remove {}", dir.display()))?;
    }
    fs::create_dir_all(&dir)?;

    PBAR.info(&format!(
        "{DOWN_ARROW}Downloading Emscripten SDK {CUR_EMSCRIPTEN_VERSION}..."
    ));
    download(
        &format!(
            "https://github.com/emscripten-core/emsdk/archive/refs/tags/{CUR_EMSCRIPTEN_VERSION}.tar.gz"
        ),
        &dir,
    )?;

    PBAR.info(&format!(
        "{DOWN_ARROW}Installing Emscripten {CUR_EMSCRIPTEN_VERSION} (LLVM, Binaryen, Node)..."
    ));
    let mut cmd = Command::new(&python);
    cmd.arg(dir.join("emsdk.py"))
        .arg("install")
        .arg(CUR_EMSCRIPTEN_VERSION)
        .current_dir(&dir);
    crate::build::utils::run(cmd, "emsdk install")?;

    if patch_frontend {
        for (name, contents) in PATCHES {
            PBAR.info(&format!("{CONFIG}Applying {name}"));
            apply_patch(&frontend, contents).with_context(|| format!("Applying {name}"))?;
        }
        fs::write(&stamp, expected_stamp)?;
    }
    Ok(dir)
}

fn stamp_contents() -> String {
    PATCHES
        .iter()
        .map(|(name, contents)| format!("{name} {:016x}\n", fnv1a(contents.as_bytes())))
        .collect()
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325u64, |hash, b| {
        (hash ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}

/// Apply a multi-file `git diff` to `root`. Every hunk must apply exactly.
pub fn apply_patch(root: &Path, patch: &str) -> Result<()> {
    let mut applied = 0;
    for chunk in patch.split("\ndiff --git ").skip(1) {
        let Some(start) = chunk.find("\n--- ") else {
            continue;
        };
        // Splitting consumed the chunk's trailing newline; diffy otherwise
        // treats the final context line as lacking one.
        let text = format!("{}\n", &chunk[start + 1..]);
        let file_patch =
            diffy::Patch::from_str(&text).map_err(|e| anyhow!("Invalid patch: {e}"))?;
        let target = file_patch
            .modified()
            .and_then(|p| p.strip_prefix("b/"))
            .ok_or_else(|| anyhow!("Patch chunk without a b/ target path"))?;
        let path = root.join(target);
        let original = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let patched = diffy::apply(&original, &file_patch)
            .map_err(|e| anyhow!("Failed to apply patch to {target}: {e}"))?;
        fs::write(&path, patched).with_context(|| format!("Failed to write {}", path.display()))?;
        applied += 1;
    }
    if applied == 0 {
        bail!("Patch contains no file diffs");
    }
    Ok(())
}

fn write_config(path: &Path, emsdk: &Path) -> Result<()> {
    let node = emsdk_node(emsdk)
        .or_else(|| which::which("node").ok())
        .ok_or_else(|| anyhow!("node is required by emcc and was not found"))?;
    let upstream = emsdk.join("upstream");
    let contents = format!(
        "LLVM_ROOT = {:?}\nBINARYEN_ROOT = {:?}\nNODE_JS = {:?}\n",
        upstream.join("bin"),
        upstream,
        node
    );
    if fs::read_to_string(path).ok().as_deref() != Some(&contents) {
        fs::write(path, contents).with_context(|| format!("Failed to write {}", path.display()))?;
    }
    Ok(())
}

fn emsdk_node(emsdk: &Path) -> Option<PathBuf> {
    fs::read_dir(emsdk.join("node"))
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path().join("bin/node"))
        .find(|p| p.exists())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn embedded_patches_parse() {
        for (name, contents) in PATCHES {
            let files = contents.matches("\ndiff --git ").count();
            assert!(files > 0, "{name} has no file diffs");
            for chunk in contents.split("\ndiff --git ").skip(1) {
                let start = chunk.find("\n--- ").unwrap();
                diffy::Patch::from_str(&format!("{}\n", &chunk[start + 1..]))
                    .unwrap_or_else(|e| panic!("{name}: {e}"));
            }
        }
    }

    #[test]
    fn apply_patch_multi_file() {
        let dir = std::env::temp_dir().join(format!("wb-patch-{}", std::process::id()));
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("a.txt"), "one\ntwo\nthree\n").unwrap();
        fs::write(dir.join("sub/b.txt"), "x\ny\n").unwrap();
        let patch = "Subject: test\n\n---\n\
diff --git a/a.txt b/a.txt\nindex 1..2 100644\n--- a/a.txt\n+++ b/a.txt\n@@ -1,3 +1,3 @@\n one\n-two\n+2\n three\n\
diff --git a/sub/b.txt b/sub/b.txt\n--- a/sub/b.txt\n+++ b/sub/b.txt\n@@ -1,2 +1,2 @@\n x\n-y\n+z\n";
        apply_patch(&dir, patch).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("a.txt")).unwrap(),
            "one\n2\nthree\n"
        );
        assert_eq!(fs::read_to_string(dir.join("sub/b.txt")).unwrap(), "x\nz\n");
        assert!(apply_patch(&dir, patch).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
}
