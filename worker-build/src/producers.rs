use std::{fs, path::Path};

use anyhow::{Context, Result};

#[derive(Clone, Copy)]
struct ProducersEntry<'a> {
    field: &'a str,
    name: &'a str,
    version: &'a str,
}

struct ProducersSection {
    start: usize,
    end: usize,
    content_start: usize,
    content_end: usize,
}

pub(crate) fn inject_workers_rs_sdk_metadata(out_dir: &Path, version: &'static str) -> Result<()> {
    let sdk_entry = ProducersEntry {
        field: "sdk",
        name: "workers-rs",
        version,
    };

    for entry in fs::read_dir(out_dir)
        .with_context(|| format!("Failed to read directory {}", out_dir.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("wasm") {
            continue;
        }

        let bytes =
            fs::read(&path).with_context(|| format!("Failed to read {}", path.display()))?;
        let updated = merge_producers_section(&bytes, &[sdk_entry]).with_context(|| {
            format!("Failed to update producers metadata in {}", path.display())
        })?;
        fs::write(&path, updated).with_context(|| format!("Failed to write {}", path.display()))?;
    }

    Ok(())
}

fn merge_producers_section<'a>(
    bytes: &'a [u8],
    new_entries: &[ProducersEntry<'a>],
) -> Result<Vec<u8>> {
    let existing = find_producers_section(bytes)?;
    let mut entries = Vec::new();

    if let Some(section) = &existing {
        parse_producers(
            bytes,
            section.content_start,
            section.content_end,
            &mut entries,
        )?;
    }

    for new_entry in new_entries {
        if !entries
            .iter()
            .any(|entry| entry.field == new_entry.field && entry.name == new_entry.name)
        {
            entries.push(*new_entry);
        }
    }

    let producers_section = encode_producers_section(&entries)?;
    let mut output = Vec::with_capacity(bytes.len() + producers_section.len() + 8);

    if let Some(section) = existing {
        output.extend_from_slice(&bytes[..section.start]);
        output.extend_from_slice(&producers_section);
        output.extend_from_slice(&bytes[section.end..]);
    } else {
        output.extend_from_slice(bytes);
        output.extend_from_slice(&producers_section);
    }

    Ok(output)
}

fn find_producers_section(bytes: &[u8]) -> Result<Option<ProducersSection>> {
    ensure_wasm_header(bytes)?;

    let mut pos = 8;
    while pos < bytes.len() {
        let section_start = pos;
        let section_id = *bytes
            .get(pos)
            .ok_or_else(|| anyhow::anyhow!("invalid wasm section"))?;
        pos += 1;

        let section_len = read_u32_leb128(bytes, &mut pos)? as usize;
        let section_end = pos
            .checked_add(section_len)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| anyhow::anyhow!("invalid wasm section length"))?;

        if section_id == 0 {
            let name_len = read_u32_leb128(bytes, &mut pos)? as usize;
            let name_end = pos
                .checked_add(name_len)
                .filter(|end| *end <= section_end)
                .ok_or_else(|| anyhow::anyhow!("invalid wasm custom section name"))?;
            let name = std::str::from_utf8(&bytes[pos..name_end])?;
            pos = name_end;

            if name == "producers" {
                return Ok(Some(ProducersSection {
                    start: section_start,
                    end: section_end,
                    content_start: pos,
                    content_end: section_end,
                }));
            }
        }

        pos = section_end;
    }

    Ok(None)
}

fn parse_producers<'a>(
    bytes: &'a [u8],
    content_start: usize,
    content_end: usize,
    entries: &mut Vec<ProducersEntry<'a>>,
) -> Result<()> {
    let mut pos = content_start;
    let field_count = read_u32_leb128(bytes, &mut pos)?;

    for _ in 0..field_count {
        let field = read_wasm_name(bytes, &mut pos, content_end)?;
        let value_count = read_u32_leb128(bytes, &mut pos)?;

        for _ in 0..value_count {
            let name = read_wasm_name(bytes, &mut pos, content_end)?;
            let version = read_wasm_name(bytes, &mut pos, content_end)?;
            entries.push(ProducersEntry {
                field,
                name,
                version,
            });
        }
    }

    if pos != content_end {
        anyhow::bail!("invalid producers section");
    }

    Ok(())
}

fn encode_producers_section(entries: &[ProducersEntry<'_>]) -> Result<Vec<u8>> {
    let mut content = Vec::new();
    let mut fields = Vec::new();

    for entry in entries {
        if !fields.contains(&entry.field) {
            fields.push(entry.field);
        }
    }

    append_u32_leb128(&mut content, fields.len().try_into()?);
    for field in fields {
        append_wasm_name(&mut content, field)?;

        let field_entries = entries
            .iter()
            .filter(|entry| entry.field == field)
            .collect::<Vec<_>>();
        append_u32_leb128(&mut content, field_entries.len().try_into()?);

        for entry in field_entries {
            append_wasm_name(&mut content, entry.name)?;
            append_wasm_name(&mut content, entry.version)?;
        }
    }

    let mut payload = Vec::new();
    append_wasm_name(&mut payload, "producers")?;
    payload.extend_from_slice(&content);

    let mut section = vec![0];
    append_u32_leb128(&mut section, payload.len().try_into()?);
    section.extend_from_slice(&payload);

    Ok(section)
}

fn ensure_wasm_header(bytes: &[u8]) -> Result<()> {
    if bytes.len() < 8 || &bytes[..4] != b"\0asm" || bytes[4..8] != [1, 0, 0, 0] {
        anyhow::bail!("invalid wasm header");
    }
    Ok(())
}

fn read_wasm_name<'a>(bytes: &'a [u8], pos: &mut usize, limit: usize) -> Result<&'a str> {
    let len = read_u32_leb128(bytes, pos)? as usize;
    let end = pos
        .checked_add(len)
        .filter(|end| *end <= limit)
        .ok_or_else(|| anyhow::anyhow!("invalid wasm name"))?;
    let name = std::str::from_utf8(&bytes[*pos..end])?;
    *pos = end;
    Ok(name)
}

fn append_wasm_name(bytes: &mut Vec<u8>, name: &str) -> Result<()> {
    append_u32_leb128(bytes, name.len().try_into()?);
    bytes.extend_from_slice(name.as_bytes());
    Ok(())
}

fn read_u32_leb128(bytes: &[u8], pos: &mut usize) -> Result<u32> {
    let mut result = 0u32;
    let mut shift = 0;

    loop {
        let byte = *bytes
            .get(*pos)
            .ok_or_else(|| anyhow::anyhow!("invalid LEB128 value"))?;
        *pos += 1;

        result |= u32::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(result);
        }

        shift += 7;
        if shift >= 32 {
            anyhow::bail!("LEB128 value overflow");
        }
    }
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
mod test {
    use super::{find_producers_section, merge_producers_section, parse_producers, ProducersEntry};

    const EMPTY_WASM: &[u8] = b"\0asm\x01\0\0\0";

    #[test]
    fn test_merge_producers_adds_sdk() {
        let wasm = merge_producers_section(
            EMPTY_WASM,
            &[ProducersEntry {
                field: "sdk",
                name: "workers-rs",
                version: "1.2.3",
            }],
        )
        .unwrap();

        let entries = producers_entries(&wasm);
        assert!(entries.iter().any(|entry| {
            entry.field == "sdk" && entry.name == "workers-rs" && entry.version == "1.2.3"
        }));
    }

    #[test]
    fn test_merge_producers_preserves_existing_field_name_pair() {
        let wasm = merge_producers_section(
            EMPTY_WASM,
            &[ProducersEntry {
                field: "sdk",
                name: "workers-rs",
                version: "1.2.3",
            }],
        )
        .unwrap();
        let wasm = merge_producers_section(
            &wasm,
            &[ProducersEntry {
                field: "sdk",
                name: "workers-rs",
                version: "4.5.6",
            }],
        )
        .unwrap();

        let entries = producers_entries(&wasm);
        let workers_rs_entries = entries
            .iter()
            .filter(|entry| entry.field == "sdk" && entry.name == "workers-rs")
            .collect::<Vec<_>>();
        assert_eq!(workers_rs_entries.len(), 1);
        assert_eq!(workers_rs_entries[0].version, "1.2.3");
    }

    #[test]
    fn test_merge_producers_preserves_existing_entries() {
        let wasm = merge_producers_section(
            EMPTY_WASM,
            &[ProducersEntry {
                field: "processed-by",
                name: "wasm-bindgen",
                version: "0.2.0",
            }],
        )
        .unwrap();
        let wasm = merge_producers_section(
            &wasm,
            &[ProducersEntry {
                field: "sdk",
                name: "workers-rs",
                version: "1.2.3",
            }],
        )
        .unwrap();

        let entries = producers_entries(&wasm);
        assert!(entries.iter().any(|entry| {
            entry.field == "processed-by"
                && entry.name == "wasm-bindgen"
                && entry.version == "0.2.0"
        }));
        assert!(entries.iter().any(|entry| {
            entry.field == "sdk" && entry.name == "workers-rs" && entry.version == "1.2.3"
        }));
    }

    fn producers_entries(wasm: &[u8]) -> Vec<ProducersEntry<'_>> {
        let section = find_producers_section(wasm).unwrap().unwrap();
        let mut entries = Vec::new();
        parse_producers(
            wasm,
            section.content_start,
            section.content_end,
            &mut entries,
        )
        .unwrap();
        entries
    }
}
