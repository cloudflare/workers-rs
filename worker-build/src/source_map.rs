//! Wasm source maps for `--emscripten` builds, which emcc writes beside the
//! linked module when linking with `-gsource-map`.

use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use path_clean::PathClean;

use crate::wasm_util::{
    append_u32_leb128, append_wasm_name, ensure_wasm_header, read_u32_leb128, read_wasm_name,
};

const SOURCE_MAPPING_URL: &str = "sourceMappingURL";

/// The byte range of the custom section called `name`, if the module has one.
fn custom_section(wasm: &[u8], name: &str) -> Result<Option<Range<usize>>> {
    let mut pos = 0;
    ensure_wasm_header(wasm, &mut pos)?;
    while pos < wasm.len() {
        let start = pos;
        let id = wasm[pos];
        pos += 1;
        let len = read_u32_leb128(wasm, &mut pos)? as usize;
        let end = pos
            .checked_add(len)
            .filter(|end| *end <= wasm.len())
            .ok_or_else(|| anyhow!("invalid wasm section length"))?;
        if id == 0 && read_wasm_name(wasm, &mut pos, end)? == name {
            return Ok(Some(start..end));
        }
        pos = end;
    }
    Ok(None)
}

/// Point the module's `sourceMappingURL` custom section at `url`, replacing
/// emcc's section or appending one.
pub(crate) fn set_source_mapping_url(wasm: &[u8], url: &str) -> Result<Vec<u8>> {
    let mut payload = Vec::new();
    append_wasm_name(&mut payload, SOURCE_MAPPING_URL)?;
    append_wasm_name(&mut payload, url)?;
    let mut section = vec![0];
    append_u32_leb128(&mut section, payload.len().try_into()?);
    section.extend_from_slice(&payload);

    let mut output = Vec::with_capacity(wasm.len() + section.len());
    match custom_section(wasm, SOURCE_MAPPING_URL)? {
        Some(range) => {
            output.extend_from_slice(&wasm[..range.start]);
            output.extend_from_slice(&section);
            output.extend_from_slice(&wasm[range.end..]);
        }
        None => {
            output.extend_from_slice(wasm);
            output.extend_from_slice(&section);
        }
    }
    Ok(output)
}

/// The map emcc wrote for `wasm_name` in one of the directories cargo links
/// into, picking the link whose module is the one cargo uplifted as `wasm`.
pub(crate) fn find_link_map(
    link_dirs: impl IntoIterator<Item = PathBuf>,
    wasm_name: &str,
    wasm: &[u8],
) -> Option<PathBuf> {
    link_dirs.into_iter().find_map(|dir| {
        let map = dir.join(format!("{wasm_name}.map"));
        (map.is_file() && fs::read(dir.join(wasm_name)).ok().as_deref() == Some(wasm))
            .then_some(map)
    })
}

/// A source map moved out of the link directory.
pub(crate) struct RelocatedMap {
    pub json: String,
    /// Some source lies inside the crate directory, i.e. the crate was built
    /// with line tables rather than only the precompiled standard library.
    pub has_crate_sources: bool,
}

/// The directories that determine how a relocated map names its sources.
pub(crate) struct SourceRoots {
    /// The crate being built.
    pub crate_dir: PathBuf,
    /// Sources in the workspace become relative to its root.
    pub workspace_root: PathBuf,
    /// Registry sources under here become `<crate>-<version>/…`, and git
    /// sources drop the `git/checkouts/` prefix.
    pub cargo_home: Option<PathBuf>,
}

/// `$CARGO_HOME`, or `~/.cargo` when it is not set.
pub(crate) fn cargo_home() -> Option<PathBuf> {
    std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs_next::home_dir().map(|home| home.join(".cargo")))
}

/// `path` relative to `dir`, comparing against `dir` as given and canonical.
fn strip_dir<'a>(path: &'a Path, dir: &Path) -> Option<&'a Path> {
    path.strip_prefix(dir.clean()).ok().or_else(|| {
        let canonical = dir.canonicalize().ok()?;
        path.strip_prefix(canonical).ok()
    })
}

/// How the map names `path`, so it carries no machine-specific directories
/// for the workspace and Cargo's dependency sources.
fn relocate_source(path: &Path, roots: &SourceRoots) -> PathBuf {
    if let Some(relative) = strip_dir(path, &roots.workspace_root) {
        return relative.to_path_buf();
    }
    if let Some(cargo_home) = &roots.cargo_home {
        if let Some(relative) = strip_dir(path, &cargo_home.join("registry/src")) {
            // Drop the registry's directory, e.g. `index.crates.io-<hash>`.
            return relative.components().skip(1).collect();
        }
        if let Some(relative) = strip_dir(path, &cargo_home.join("git/checkouts")) {
            return relative.to_path_buf();
        }
    }
    path.to_path_buf()
}

/// Rewrite emcc's map, whose sources are relative to `link_dir`, for use
/// under the name `file`, naming its sources as `relocate_source` does.
pub(crate) fn relocate_map(
    json: &str,
    link_dir: &Path,
    roots: &SourceRoots,
    file: &str,
) -> Result<RelocatedMap> {
    let mut map: serde_json::Value =
        serde_json::from_str(json).context("Failed to parse the source map emcc wrote")?;
    let obj = map
        .as_object_mut()
        .ok_or_else(|| anyhow!("source map is not a JSON object"))?;
    let sources = obj
        .get_mut("sources")
        .and_then(|s| s.as_array_mut())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("source map has no sources"))?;

    let mut has_crate_sources = false;
    for source in sources.iter_mut() {
        let Some(path) = source.as_str() else {
            continue;
        };
        let path = link_dir.join(path).clean();
        has_crate_sources |= strip_dir(&path, &roots.crate_dir).is_some();
        *source = relocate_source(&path, roots)
            .to_string_lossy()
            .replace('\\', "/")
            .into();
    }
    obj.insert("file".into(), file.into());

    Ok(RelocatedMap {
        json: serde_json::to_string(&map)?,
        has_crate_sources,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    /// A module with a type, a function and a code section, then `custom`.
    fn module(custom: &[(&str, &[u8])]) -> Vec<u8> {
        let mut wasm = b"\0asm\x01\0\0\0".to_vec();
        wasm.extend_from_slice(&[1, 4, 1, 0x60, 0, 0]);
        wasm.extend_from_slice(&[3, 2, 1, 0]);
        wasm.extend_from_slice(&[10, 4, 1, 2, 0, 0x0b]);
        for (name, content) in custom {
            let mut payload = Vec::new();
            append_wasm_name(&mut payload, name).unwrap();
            payload.extend_from_slice(content);
            wasm.push(0);
            append_u32_leb128(&mut wasm, payload.len() as u32);
            wasm.extend_from_slice(&payload);
        }
        wasm
    }

    fn url_of(wasm: &[u8]) -> String {
        let section = custom_section(wasm, SOURCE_MAPPING_URL).unwrap().unwrap();
        let mut pos = section.start + 1;
        read_u32_leb128(wasm, &mut pos).unwrap();
        read_wasm_name(wasm, &mut pos, section.end).unwrap();
        read_wasm_name(wasm, &mut pos, section.end)
            .unwrap()
            .to_string()
    }

    #[test]
    fn test_set_source_mapping_url_replaces_in_place() {
        let wasm = module(&[
            (SOURCE_MAPPING_URL, b"\x0femscripten.wasm.map"),
            ("name", b"\0"),
        ]);
        let updated = set_source_mapping_url(&wasm, "index_bg.wasm.map").unwrap();
        assert_eq!(url_of(&updated), "index_bg.wasm.map");
        // The replaced section stays before `name`, and everything before it is unchanged.
        let url = custom_section(&updated, SOURCE_MAPPING_URL)
            .unwrap()
            .unwrap();
        let name = custom_section(&updated, "name").unwrap().unwrap();
        assert_eq!(url.end, name.start);
        assert_eq!(updated[..24], wasm[..24]);
    }

    #[test]
    fn test_set_source_mapping_url_appends() {
        let wasm = module(&[]);
        let updated = set_source_mapping_url(&wasm, "index_bg.wasm.map").unwrap();
        assert_eq!(url_of(&updated), "index_bg.wasm.map");
        assert!(updated.starts_with(&wasm));
    }

    fn roots(crate_dir: &str, workspace_root: &str) -> SourceRoots {
        SourceRoots {
            crate_dir: crate_dir.into(),
            workspace_root: workspace_root.into(),
            cargo_home: Some("/home/u/.cargo".into()),
        }
    }

    fn sources(map: &RelocatedMap) -> serde_json::Value {
        serde_json::from_str::<serde_json::Value>(&map.json).unwrap()["sources"].clone()
    }

    #[test]
    fn test_relocate_map() {
        let json = r#"{"version":3,"sources":[
            "../../src/main.rs",
            "../../../../rustc/abc/library/core/src/panicking.rs",
            "/home/u/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/js-sys-0.3.106/src/lib.rs",
            "/home/u/.cargo/git/checkouts/workers-rs-3f1c/1d668cf/worker/src/lib.rs",
            "/opt/vendor/src/lib.rs"
        ],"names":[],"mappings":"gBAAA"}"#;
        let map = relocate_map(
            json,
            Path::new("/w/crate/target/out"),
            &roots("/w/crate", "/w/crate"),
            "index_bg.wasm",
        )
        .unwrap();
        assert!(map.has_crate_sources);
        assert_eq!(
            sources(&map),
            serde_json::json!([
                "src/main.rs",
                "/rustc/abc/library/core/src/panicking.rs",
                "js-sys-0.3.106/src/lib.rs",
                "workers-rs-3f1c/1d668cf/worker/src/lib.rs",
                "/opt/vendor/src/lib.rs"
            ])
        );
        let value: serde_json::Value = serde_json::from_str(&map.json).unwrap();
        assert_eq!(value["file"], "index_bg.wasm");
        assert_eq!(value["mappings"], "gBAAA");
    }

    #[test]
    fn test_relocate_map_workspace_member() {
        // The crate and a path dependency are both members of the workspace at /w.
        let json = r#"{"version":3,"sources":["../../app/src/main.rs","../../lib/src/lib.rs"],"names":[],"mappings":"gBAAA"}"#;
        let map = relocate_map(
            json,
            Path::new("/w/target/out"),
            &roots("/w/app", "/w"),
            "x.wasm",
        )
        .unwrap();
        assert!(map.has_crate_sources);
        assert_eq!(
            sources(&map),
            serde_json::json!(["app/src/main.rs", "lib/src/lib.rs"])
        );
    }

    #[test]
    fn test_relocate_map_without_crate_sources() {
        let json = r#"{"version":3,"sources":["/rustc/abc/library/core/src/panicking.rs"],"names":[],"mappings":"gBAAA"}"#;
        let map = relocate_map(
            json,
            Path::new("/w/out"),
            &roots("/w/crate", "/w/crate"),
            "x.wasm",
        )
        .unwrap();
        assert!(!map.has_crate_sources);
    }

    #[test]
    fn test_relocate_map_rejects_empty_map() {
        let json = r#"{"version":3,"sources":[],"names":[],"mappings":""}"#;
        assert!(relocate_map(
            json,
            Path::new("/w/out"),
            &roots("/w/crate", "/w/crate"),
            "x.wasm"
        )
        .is_err());
    }
}
