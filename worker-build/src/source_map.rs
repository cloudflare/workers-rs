//! Wasm source maps for `--emscripten` builds, which emcc writes beside the
//! linked module when linking with `-gsource-map`.

use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
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

/// V8's limit on the functions in a module (`kV8MaxWasmFunctions`); a module
/// with more can't run.
const MAX_FUNCTIONS: usize = 1_000_000;

/// The names in the function-names subsection of the module's `name`
/// section, by function index: `""` for functions without one, up to the
/// highest named index. Empty when the module has no `name` section.
pub(crate) fn function_names(wasm: &[u8]) -> Result<Vec<String>> {
    let mut names = Vec::new();
    let Some(section) = custom_section(wasm, "name")? else {
        return Ok(names);
    };
    let wasm = &wasm[..section.end];
    let mut pos = section.start + 1;
    read_u32_leb128(wasm, &mut pos)?;
    read_wasm_name(wasm, &mut pos, section.end)?;
    while pos < section.end {
        let id = wasm[pos];
        pos += 1;
        let len = read_u32_leb128(wasm, &mut pos)? as usize;
        let end = pos
            .checked_add(len)
            .filter(|end| *end <= section.end)
            .ok_or_else(|| anyhow!("invalid name subsection length"))?;
        if id == 1 {
            let subsection = &wasm[..end];
            for _ in 0..read_u32_leb128(subsection, &mut pos)? {
                let index = read_u32_leb128(subsection, &mut pos)? as usize;
                if index >= MAX_FUNCTIONS {
                    bail!("function index {index} in the name section is out of range");
                }
                let name = read_wasm_name(subsection, &mut pos, end)?;
                if index >= names.len() {
                    names.resize(index + 1, String::new());
                }
                names[index] = name.to_string();
            }
        }
        pos = end;
    }
    Ok(names)
}

/// The module without its custom section called `name`.
pub(crate) fn remove_custom_section(wasm: &[u8], name: &str) -> Result<Vec<u8>> {
    Ok(match custom_section(wasm, name)? {
        Some(range) => [&wasm[..range.start], &wasm[range.end..]].concat(),
        None => wasm.to_vec(),
    })
}

/// The source map field that holds the module's function names: entry `i`
/// names the function V8 reports as `wasm-function[i]`.
const FUNCTION_NAMES_FIELD: &str = "x_cloudflare_wasm_function_names";

/// Write `function_names` to the map's `FUNCTION_NAMES_FIELD`.
fn add_function_names(json: &str, function_names: Vec<String>) -> Result<String> {
    let mut map: serde_json::Value = serde_json::from_str(json)?;
    map.as_object_mut()
        .ok_or_else(|| anyhow!("source map is not a JSON object"))?
        .insert(FUNCTION_NAMES_FIELD.into(), function_names.into());
    Ok(serde_json::to_string(&map)?)
}

/// The map with the module's function names added, and the module, without
/// its `name` section if `strip`. Fails if the names can't go into the map,
/// so that the caller ships the module with them.
pub(crate) fn move_function_names(
    json: &str,
    wasm: &[u8],
    strip: bool,
) -> Result<(String, Vec<u8>)> {
    let names = function_names(wasm)?;
    let json = if names.is_empty() {
        json.to_string()
    } else {
        add_function_names(json, names)?
    };
    // The name section follows the code section, so removing it doesn't
    // move the offsets the map refers to.
    let wasm = if strip {
        remove_custom_section(wasm, "name")?
    } else {
        wasm.to_vec()
    };
    Ok((json, wasm))
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

    /// A `name` section's content holding `subsections`.
    fn name_section(subsections: &[(u8, Vec<u8>)]) -> Vec<u8> {
        let mut content = Vec::new();
        for (id, subsection) in subsections {
            content.push(*id);
            append_u32_leb128(&mut content, subsection.len() as u32);
            content.extend_from_slice(subsection);
        }
        content
    }

    /// A name map, as in the function-names subsection.
    fn name_map(names: &[(u32, &str)]) -> Vec<u8> {
        let mut map = Vec::new();
        append_u32_leb128(&mut map, names.len() as u32);
        for (index, name) in names {
            append_u32_leb128(&mut map, *index);
            append_wasm_name(&mut map, name).unwrap();
        }
        map
    }

    #[test]
    fn test_function_names() {
        let names = name_section(&[
            (0, b"\x03mod".to_vec()),
            (1, name_map(&[(1, "a"), (3, "b::c"), (130, "d")])),
            (2, b"\x01\x01\x01\x00\x01x".to_vec()),
        ]);
        let mut expected = vec![""; 131];
        expected[1] = "a";
        expected[3] = "b::c";
        expected[130] = "d";
        assert_eq!(
            function_names(&module(&[("producers", b"\0"), ("name", &names)])).unwrap(),
            expected
        );
    }

    #[test]
    fn test_function_names_with_import() {
        // Imports come first in the function index space.
        let mut wasm = module(&[(
            "name",
            &name_section(&[(1, name_map(&[(0, "imported"), (1, "defined")]))]),
        )]);
        wasm.splice(14..14, *b"\x02\x07\x01\x01m\x01f\x00\x00");
        assert_eq!(function_names(&wasm).unwrap(), ["imported", "defined"]);
    }

    #[test]
    fn test_function_names_empty() {
        assert!(function_names(&module(&[])).unwrap().is_empty());
        assert!(function_names(&module(&[("name", b"")]))
            .unwrap()
            .is_empty());
        let module_name_only = name_section(&[(0, b"\x03mod".to_vec())]);
        assert!(function_names(&module(&[("name", &module_name_only)]))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_function_names_rejects_invalid_utf8() {
        let mut map = name_map(&[(0, "a")]);
        *map.last_mut().unwrap() = 0xff;
        assert!(function_names(&module(&[("name", &name_section(&[(1, map)]))])).is_err());
    }

    #[test]
    fn test_function_names_rejects_malformed() {
        let after = ("producers", &b"\0"[..]);
        for subsection in [
            // Index 0xffffffff.
            b"\x01\xff\xff\xff\xff\x0f\x00".to_vec(),
            // A name longer than the subsection.
            b"\x01\x00\x05a".to_vec(),
            // A count cut off by the end of the subsection.
            b"\x80".to_vec(),
        ] {
            let names = name_section(&[(1, subsection), (2, b"\x00".to_vec())]);
            assert!(function_names(&module(&[("name", &names), after])).is_err());
        }
    }

    #[test]
    fn test_add_function_names() {
        // `names` keeps its own meaning, whatever it holds.
        let json =
            r#"{"version":3,"file":"x.wasm","sources":["a.rs"],"names":["x"],"mappings":"gBAAAA"}"#;
        let updated = add_function_names(json, vec!["".into(), "f".into()]).unwrap();
        let mut expected: serde_json::Value = serde_json::from_str(json).unwrap();
        expected[FUNCTION_NAMES_FIELD] = serde_json::json!(["", "f"]);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&updated).unwrap(),
            expected
        );
    }

    const MAP: &str = r#"{"version":3,"sources":["a.rs"],"names":[],"mappings":"gBAAA"}"#;

    fn map_names(json: &str) -> serde_json::Value {
        serde_json::from_str::<serde_json::Value>(json).unwrap()[FUNCTION_NAMES_FIELD].clone()
    }

    #[test]
    fn test_move_function_names() {
        let names = name_section(&[(1, name_map(&[(0, "f")]))]);
        let wasm = module(&[("name", &names), (SOURCE_MAPPING_URL, b"\x01x")]);
        let (json, stripped) = move_function_names(MAP, &wasm, true).unwrap();
        assert_eq!(map_names(&json), serde_json::json!(["f"]));
        assert_eq!(stripped, remove_custom_section(&wasm, "name").unwrap());
        let (json, kept) = move_function_names(MAP, &wasm, false).unwrap();
        assert_eq!(map_names(&json), serde_json::json!(["f"]));
        assert_eq!(kept, wasm);
        // Without a name section the map is unchanged.
        assert_eq!(
            move_function_names(MAP, &module(&[]), true).unwrap(),
            (MAP.to_string(), module(&[]))
        );
    }

    #[test]
    fn test_move_function_names_fails_when_names_are_unreadable() {
        let mut unreadable = name_map(&[(0, "f")]);
        *unreadable.last_mut().unwrap() = 0xff;
        let wasm = module(&[("name", &name_section(&[(1, unreadable)]))]);
        assert!(move_function_names(MAP, &wasm, true).is_err());
    }

    #[test]
    fn test_remove_custom_section() {
        let wasm = module(&[("name", b"\0"), (SOURCE_MAPPING_URL, b"\x01x")]);
        let stripped = remove_custom_section(&wasm, "name").unwrap();
        assert!(custom_section(&stripped, "name").unwrap().is_none());
        assert!(custom_section(&stripped, SOURCE_MAPPING_URL)
            .unwrap()
            .is_some());
        // The type, function and code sections are unchanged.
        assert_eq!(stripped[..24], wasm[..24]);
        assert_eq!(remove_custom_section(&stripped, "name").unwrap(), stripped);
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
