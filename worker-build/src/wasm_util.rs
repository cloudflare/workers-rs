//! Reading and writing pieces of the Wasm binary format.

use anyhow::Result;

/// Check the module header at `pos` and move past it.
pub(crate) fn ensure_wasm_header(bytes: &[u8], pos: &mut usize) -> Result<()> {
    if bytes.get(*pos..*pos + 8) != Some(b"\0asm\x01\0\0\0") {
        anyhow::bail!("invalid wasm header");
    }
    *pos += 8;
    Ok(())
}

pub(crate) fn read_wasm_name<'a>(
    bytes: &'a [u8],
    pos: &mut usize,
    limit: usize,
) -> Result<&'a str> {
    let len = read_u32_leb128(bytes, pos)? as usize;
    let end = pos
        .checked_add(len)
        .filter(|end| *end <= limit)
        .ok_or_else(|| anyhow::anyhow!("invalid wasm name"))?;
    let name = std::str::from_utf8(&bytes[*pos..end])?;
    *pos = end;
    Ok(name)
}

pub(crate) fn append_wasm_name(bytes: &mut Vec<u8>, name: &str) -> Result<()> {
    append_u32_leb128(bytes, name.len().try_into()?);
    bytes.extend_from_slice(name.as_bytes());
    Ok(())
}

pub(crate) fn read_u32_leb128(bytes: &[u8], pos: &mut usize) -> Result<u32> {
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

pub(crate) fn append_u32_leb128(bytes: &mut Vec<u8>, mut value: u32) {
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
