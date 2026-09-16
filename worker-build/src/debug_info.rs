use std::{fs, path::Path};

use anyhow::{bail, Context, Result};
use wasmparser::{BinaryReader, Parser, Payload, Validator};

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
        let summary = debug_info_summary(&wasm, sidecar_name)
            .with_context(|| format!("Failed to inspect debug info in {}", path.display()))?;
        if summary.dwarf_sections == 0
            && summary.external_references == 1
            && summary.expected_external_references == 1
            && sidecar_path.is_file()
        {
            let sidecar = fs::read(&sidecar_path)
                .with_context(|| format!("Failed to read {}", sidecar_path.display()))?;
            if debug_info_summary(&sidecar, sidecar_name)
                .with_context(|| format!("Failed to inspect {}", sidecar_path.display()))?
                .dwarf_sections
                > 0
            {
                continue;
            }
        }
        if summary.dwarf_sections == 0 {
            bail!(
                "cannot create external DWARF: {} does not contain DWARF custom sections",
                path.display()
            );
        }
        let runtime = split_module(&wasm, sidecar_name)
            .with_context(|| format!("Failed to split debug info from {}", path.display()))?;

        fs::write(&sidecar_path, &wasm)
            .with_context(|| format!("Failed to write {}", sidecar_path.display()))?;
        fs::write(&path, runtime).with_context(|| format!("Failed to write {}", path.display()))?;
    }

    Ok(())
}

#[derive(Default)]
struct DebugInfoSummary {
    dwarf_sections: usize,
    external_references: usize,
    expected_external_references: usize,
}

fn debug_info_summary(wasm: &[u8], sidecar_name: &str) -> Result<DebugInfoSummary> {
    Validator::new()
        .validate_all(wasm)
        .context("invalid WebAssembly module")?;
    let mut summary = DebugInfoSummary::default();
    for payload in Parser::new(0).parse_all(wasm) {
        if let Payload::CustomSection(section) = payload? {
            if section.name().starts_with(".debug_") {
                summary.dwarf_sections += 1;
            } else if section.name() == "external_debug_info" {
                if let Ok(reference) = read_external_debug_info(section.data()) {
                    summary.external_references += 1;
                    if reference == sidecar_name {
                        summary.expected_external_references += 1;
                    }
                }
            }
        }
    }

    Ok(summary)
}

fn read_external_debug_info(data: &[u8]) -> Result<&str> {
    let mut reader = BinaryReader::new(data, 0);
    let reference = reader
        .read_string()
        .context("external_debug_info does not contain a WebAssembly name")?;
    if !reader.eof() {
        bail!("external_debug_info contains trailing data");
    }
    Ok(reference)
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
            if section.name().starts_with(".debug_") || section.name() == "external_debug_info" {
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

    use super::{read_external_debug_info, split_debug_info, split_module};
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

    fn external_debug_info(wasm: &[u8]) -> Vec<&str> {
        custom_sections(wasm)
            .into_iter()
            .filter(|(name, _)| *name == "external_debug_info")
            .map(|(_, data)| read_external_debug_info(data).unwrap())
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
        wasmparser::validate(&sidecar).unwrap();
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

        assert_eq!(external_debug_info(&runtime), ["index_bg.debug.wasm"]);

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
        let out_dir = tempfile::tempdir().unwrap();
        let runtime_path = out_dir.path().join("index_bg.wasm");
        fs::write(&runtime_path, b"not wasm").unwrap();

        let error = split_debug_info(out_dir.path()).unwrap_err();

        let message = format!("{error:#}");
        assert!(message.contains("index_bg.wasm"), "{message}");
        assert!(message.contains("invalid WebAssembly module"), "{message}");
        assert!(message.contains("magic header"), "{message}");
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
        assert_eq!(external_debug_info(&runtime), ["index_bg.debug.wasm"]);
    }

    #[test]
    fn splitting_an_already_split_output_is_deterministic() {
        let out_dir = tempfile::tempdir().unwrap();
        let runtime_path = out_dir.path().join("index_bg.wasm");
        let sidecar_path = out_dir.path().join("index_bg.debug.wasm");
        fs::write(&runtime_path, test_module()).unwrap();

        split_debug_info(out_dir.path()).unwrap();
        let first_runtime = fs::read(&runtime_path).unwrap();
        let first_sidecar = fs::read(&sidecar_path).unwrap();

        split_debug_info(out_dir.path()).unwrap();

        assert_eq!(fs::read(runtime_path).unwrap(), first_runtime);
        assert_eq!(fs::read(sidecar_path).unwrap(), first_sidecar);
    }

    #[test]
    fn refuses_to_create_an_external_dwarf_artifact_without_dwarf() {
        let out_dir = tempfile::tempdir().unwrap();
        let runtime_path = out_dir.path().join("index_bg.wasm");
        let mut wasm = b"\0asm\x01\0\0\0".to_vec();
        wasm.extend(section(1, &[1, 0x60, 0, 0]));
        wasm.extend(section(3, &[1, 0]));
        wasm.extend(section(10, &[1, 2, 0, 0x0b]));
        fs::write(&runtime_path, wasm).unwrap();

        let error = split_debug_info(out_dir.path()).unwrap_err();

        let message = format!("{error:#}");
        assert!(message.contains("index_bg.wasm"), "{message}");
        assert!(message.contains("does not contain DWARF"), "{message}");
        assert!(!out_dir.path().join("index_bg.debug.wasm").exists());
    }

    #[test]
    fn replaces_existing_external_debug_info_references() {
        let mut input = test_module();
        input.extend(custom_section("external_debug_info", b"\x0eold.debug.wasm"));

        let runtime = split_module(&input, "index_bg.debug.wasm").unwrap();

        assert_eq!(external_debug_info(&runtime), ["index_bg.debug.wasm"]);
    }

    #[test]
    fn supports_nested_custom_output_directories_and_names() {
        let root = tempfile::tempdir().unwrap();
        let out_dir = root.path().join("nested/custom-output");
        fs::create_dir_all(&out_dir).unwrap();
        fs::write(out_dir.join("custom.wasm"), test_module()).unwrap();

        split_debug_info(&out_dir).unwrap();

        let runtime = fs::read(out_dir.join("custom.wasm")).unwrap();
        let sidecar = fs::read(out_dir.join("custom.debug.wasm")).unwrap();
        assert_eq!(external_debug_info(&runtime), ["custom.debug.wasm"]);
        wasmparser::validate(&runtime).unwrap();
        wasmparser::validate(&sidecar).unwrap();
    }

    #[test]
    fn reports_the_missing_output_directory() {
        let root = tempfile::tempdir().unwrap();
        let missing = root.path().join("missing/output");

        let error = split_debug_info(&missing).unwrap_err();

        let message = format!("{error:#}");
        assert!(message.contains("Failed to read directory"), "{message}");
        assert!(
            message.contains(&missing.display().to_string()),
            "{message}"
        );
    }

    #[test]
    fn reports_sidecar_write_failures_without_modifying_the_runtime() {
        let out_dir = tempfile::tempdir().unwrap();
        let runtime_path = out_dir.path().join("index_bg.wasm");
        let sidecar_path = out_dir.path().join("index_bg.debug.wasm");
        let original = test_module();
        fs::write(&runtime_path, &original).unwrap();
        fs::create_dir(&sidecar_path).unwrap();

        let error = split_debug_info(out_dir.path()).unwrap_err();

        let message = format!("{error:#}");
        assert!(message.contains("Failed to write"), "{message}");
        assert!(
            message.contains(&sidecar_path.display().to_string()),
            "{message}"
        );
        assert_eq!(fs::read(runtime_path).unwrap(), original);
    }
}
