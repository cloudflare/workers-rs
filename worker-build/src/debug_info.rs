use std::{fs, path::Path};

use anyhow::{Context, Result};
use wasmparser::{Parser, Payload};

/// Split DWARF into complete `.debug.wasm` sidecars after all transforms have
/// run, leaving each runtime module with an `external_debug_info` reference.
pub(crate) fn split_debug_info(out_dir: &Path) -> Result<()> {
    for entry in fs::read_dir(out_dir)
        .with_context(|| format!("Failed to read directory {}", out_dir.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("wasm")
            || path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".debug.wasm"))
        {
            continue;
        }

        let sidecar_path = path.with_extension("debug.wasm");
        let sidecar_name = sidecar_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                anyhow::anyhow!("invalid WebAssembly output name: {}", path.display())
            })?;
        let wasm = fs::read(&path).with_context(|| format!("Failed to read {}", path.display()))?;
        let runtime = split_module(&wasm, sidecar_name)
            .with_context(|| format!("Failed to split debug info from {}", path.display()))?;

        fs::write(&sidecar_path, &wasm)
            .with_context(|| format!("Failed to write {}", sidecar_path.display()))?;
        fs::write(&path, runtime).with_context(|| format!("Failed to write {}", path.display()))?;
    }

    Ok(())
}

/// Mirror wasm-bindgen's `--split-debug-info` encoding: remove `.debug_*`
/// custom sections and append an `external_debug_info` custom section whose
/// data is a WebAssembly name containing the sidecar URL.
fn split_module(wasm: &[u8], sidecar_name: &str) -> Result<Vec<u8>> {
    let mut kept = Vec::new();
    let mut keep_from = 0;
    let mut section_start = 0;

    for payload in Parser::new(0).parse_all(wasm) {
        let payload = payload?;
        if let Payload::CustomSection(section) = &payload {
            if section.name().starts_with(".debug_") {
                kept.push(keep_from..section_start);
                keep_from = section.range().end;
            }
        }

        if let Payload::Version { range, .. } = &payload {
            section_start = range.end;
        } else if let Some((_, range)) = payload.as_section() {
            section_start = range.end;
        }
    }
    kept.push(keep_from..wasm.len());

    let mut contents = Vec::new();
    append_name(&mut contents, "external_debug_info")?;
    append_name(&mut contents, sidecar_name)?;

    let kept_len = kept.iter().map(|range| range.len()).sum::<usize>();
    let mut runtime = Vec::with_capacity(kept_len + contents.len() + 6);
    for range in kept {
        runtime.extend_from_slice(&wasm[range]);
    }
    runtime.push(0);
    append_u32_leb128(&mut runtime, contents.len().try_into()?);
    runtime.extend_from_slice(&contents);
    Ok(runtime)
}

fn append_name(bytes: &mut Vec<u8>, name: &str) -> Result<()> {
    append_u32_leb128(bytes, name.len().try_into()?);
    bytes.extend_from_slice(name.as_bytes());
    Ok(())
}

fn append_u32_leb128(bytes: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{split_debug_info, split_module};
    use wasmparser::{Parser, Payload};

    fn section(id: u8, contents: &[u8]) -> Vec<u8> {
        let mut section = vec![id, contents.len() as u8];
        section.extend_from_slice(contents);
        section
    }

    fn custom_section(name: &str, data: &[u8]) -> Vec<u8> {
        let mut contents = vec![name.len() as u8];
        contents.extend_from_slice(name.as_bytes());
        contents.extend_from_slice(data);
        section(0, &contents)
    }

    fn test_module() -> Vec<u8> {
        let mut wasm = b"\0asm\x01\0\0\0".to_vec();
        wasm.extend(section(1, &[1, 0x60, 0, 0]));
        wasm.extend(custom_section("name", b"names"));
        wasm.extend(custom_section(".debug_info", b"dwarf"));
        wasm.extend(section(3, &[1, 0]));
        wasm.extend(section(10, &[1, 2, 0, 0x0b]));
        wasm.extend(custom_section(".debug_line", b"lines"));
        wasm.extend(custom_section("other", b"kept"));
        wasm
    }

    fn custom_sections(wasm: &[u8]) -> Vec<(&str, &[u8])> {
        Parser::new(0)
            .parse_all(wasm)
            .filter_map(|payload| match payload.unwrap() {
                Payload::CustomSection(section) => Some((section.name(), section.data())),
                _ => None,
            })
            .collect()
    }

    fn code_section(wasm: &[u8]) -> &[u8] {
        Parser::new(0)
            .parse_all(wasm)
            .find_map(|payload| match payload.unwrap() {
                Payload::CodeSectionStart { range, .. } => Some(&wasm[range]),
                _ => None,
            })
            .unwrap()
    }

    #[test]
    fn splits_dwarf_without_changing_code_or_other_sections() {
        let sidecar = test_module();
        let runtime = split_module(&sidecar, "index_bg.debug.wasm").unwrap();

        wasmparser::validate(&runtime).unwrap();
        assert_eq!(code_section(&runtime), code_section(&sidecar));

        let runtime_sections = custom_sections(&runtime);
        assert!(runtime_sections
            .iter()
            .all(|(name, _)| !name.starts_with(".debug_")));
        assert!(runtime_sections
            .iter()
            .any(|(name, data)| *name == "name" && *data == b"names"));
        assert!(runtime_sections
            .iter()
            .any(|(name, data)| *name == "other" && *data == b"kept"));

        let external = runtime_sections
            .iter()
            .filter(|(name, _)| *name == "external_debug_info")
            .collect::<Vec<_>>();
        assert_eq!(external.len(), 1);
        assert_eq!(external[0].1, b"\x13index_bg.debug.wasm");

        let sidecar_sections = custom_sections(&sidecar);
        assert!(sidecar_sections
            .iter()
            .any(|(name, _)| *name == ".debug_info"));
        assert!(sidecar_sections
            .iter()
            .any(|(name, _)| *name == ".debug_line"));
        assert!(sidecar_sections
            .iter()
            .all(|(name, _)| *name != "external_debug_info"));
    }

    #[test]
    fn rejects_malformed_wasm() {
        let error = split_module(b"not wasm", "index_bg.debug.wasm").unwrap_err();
        assert!(error.to_string().contains("magic header"));
    }

    #[test]
    fn writes_complete_sidecar_next_to_runtime_module() {
        let out_dir = tempfile::tempdir().unwrap();
        let runtime_path = out_dir.path().join("index_bg.wasm");
        let original = test_module();
        fs::write(&runtime_path, &original).unwrap();

        split_debug_info(out_dir.path()).unwrap();

        let sidecar_path = out_dir.path().join("index_bg.debug.wasm");
        assert_eq!(fs::read(sidecar_path).unwrap(), original);
        let runtime = fs::read(runtime_path).unwrap();
        assert!(custom_sections(&runtime).iter().any(|(name, data)| {
            *name == "external_debug_info" && *data == b"\x13index_bg.debug.wasm"
        }));
    }
}
