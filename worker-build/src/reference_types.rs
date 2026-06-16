//! Re-advertise `reference-types` in stripped wasm builds.
//!
//! `wasm-bindgen` checks the `target_features` custom section for
//! `reference-types` before running its externref transform. Without it,
//! `#[wasm_bindgen(catch)]` wrappers fail with
//! `externref table required for catch wrappers`.
//!
//! `strip = true` (i.e. `strip = "symbols"`) drops the entire section, even
//! though `wasm32-unknown-unknown` has `reference-types` on by default since
//! Rust 1.82. We patch it back in after cargo and before `wasm-bindgen`.
//! `strip = "debuginfo"` avoids the problem entirely.

use std::path::Path;

use anyhow::{Context, Result};

use crate::build::PBAR;
use crate::producers::{
    append_u32_leb128, append_wasm_name, encode_custom_section, find_custom_section,
    read_u32_leb128, read_wasm_name, CustomSection,
};

const REFERENCE_TYPES: &str = "reference-types";
const TARGET_FEATURES: &str = "target_features";
/// `+` prefix in a `target_features` entry: the feature is enabled/used.
const PREFIX_ENABLED: u8 = b'+';

/// No-op when the feature is already present (with `+` or `-` prefix).
pub(crate) fn ensure_reference_types_feature(wasm_path: &Path) -> Result<()> {
    let bytes = std::fs::read(wasm_path)
        .with_context(|| format!("Failed to read {}", wasm_path.display()))?;

    let section = find_custom_section(&bytes, TARGET_FEATURES)?;
    if let Some(section) = &section {
        if mentions_reference_types(&bytes, section)? {
            return Ok(());
        }
    }

    let patched = add_reference_types_feature(&bytes, section.as_ref())?;
    std::fs::write(wasm_path, patched)
        .with_context(|| format!("Failed to write {}", wasm_path.display()))?;
    PBAR.warn(
        "Compiled wasm did not advertise the `reference-types` target feature, which \
         `wasm-bindgen` needs to generate catch wrappers. This is usually caused by \
         `strip = true` (i.e. `strip = \"symbols\"`) in your release profile dropping the \
         `target_features` section. Re-added it so the build can continue; prefer \
         `strip = \"debuginfo\"` to avoid relying on this.",
    );
    Ok(())
}

/// True if `section` mentions `reference-types` with any prefix.
fn mentions_reference_types(bytes: &[u8], section: &CustomSection) -> Result<bool> {
    let mut pos = section.content_start;
    let count = read_u32_leb128(bytes, &mut pos)?;
    for _ in 0..count {
        // Each entry is a one-byte +/- prefix followed by the feature name.
        let _prefix = *bytes
            .get(pos)
            .ok_or_else(|| anyhow::anyhow!("invalid target_features section"))?;
        pos += 1;
        if read_wasm_name(bytes, &mut pos, section.end)? == REFERENCE_TYPES {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Append `+reference-types` to the existing section, or create one if absent.
fn add_reference_types_feature(bytes: &[u8], section: Option<&CustomSection>) -> Result<Vec<u8>> {
    // Rebuild the section content: the existing feature count + 1, the existing
    // entries verbatim, then our `+reference-types` entry.
    let (existing_count, existing_entries) = match section {
        Some(section) => {
            let mut pos = section.content_start;
            (read_u32_leb128(bytes, &mut pos)?, &bytes[pos..section.end])
        }
        None => (0, &[][..]),
    };

    let mut content = Vec::new();
    append_u32_leb128(&mut content, existing_count + 1);
    content.extend_from_slice(existing_entries);
    content.push(PREFIX_ENABLED);
    append_wasm_name(&mut content, REFERENCE_TYPES)?;

    let new_section = encode_custom_section(TARGET_FEATURES, &content)?;

    // Splice the rebuilt section over the old one, or append it if there was none.
    let (head, tail) = match section {
        Some(section) => (&bytes[..section.start], &bytes[section.end..]),
        None => (bytes, &[][..]),
    };

    let mut output = Vec::with_capacity(head.len() + new_section.len() + tail.len());
    output.extend_from_slice(head);
    output.extend_from_slice(&new_section);
    output.extend_from_slice(tail);
    Ok(output)
}

#[cfg(test)]
mod test {
    use super::*;

    const EMPTY_WASM: &[u8] = b"\0asm\x01\0\0\0";

    /// Minimal wasm with only a `target_features` section.
    fn wasm_with_features(features: &[(u8, &str)]) -> Vec<u8> {
        let mut content = Vec::new();
        append_u32_leb128(&mut content, features.len().try_into().unwrap());
        for (prefix, name) in features {
            content.push(*prefix);
            append_wasm_name(&mut content, name).unwrap();
        }
        let section = encode_custom_section(TARGET_FEATURES, &content).unwrap();
        let mut wasm = EMPTY_WASM.to_vec();
        wasm.extend_from_slice(&section);
        wasm
    }

    /// Check if the wasm mentions `reference-types`.
    fn mentions(wasm: &[u8]) -> bool {
        match find_custom_section(wasm, TARGET_FEATURES).unwrap() {
            Some(section) => mentions_reference_types(wasm, &section).unwrap(),
            None => false,
        }
    }

    #[test]
    fn missing_section_is_not_mentioned() {
        assert!(!mentions(EMPTY_WASM));
    }

    #[test]
    fn enabled_feature_is_detected() {
        let wasm = wasm_with_features(&[
            (PREFIX_ENABLED, "bulk-memory"),
            (PREFIX_ENABLED, REFERENCE_TYPES),
        ]);
        assert!(mentions(&wasm));
    }

    #[test]
    fn disallowed_feature_is_respected() {
        // A `-reference-types` entry counts as mentioned, so we leave it alone.
        let wasm = wasm_with_features(&[(b'-', REFERENCE_TYPES)]);
        assert!(mentions(&wasm));
    }

    #[test]
    fn section_without_reference_types_is_not_mentioned() {
        let wasm = wasm_with_features(&[(PREFIX_ENABLED, "bulk-memory")]);
        assert!(!mentions(&wasm));
    }

    #[test]
    fn adds_section_when_absent() {
        let patched = add_reference_types_feature(EMPTY_WASM, None).unwrap();
        assert!(mentions(&patched));
    }

    #[test]
    fn extends_existing_section_preserving_features() {
        let wasm = wasm_with_features(&[
            (PREFIX_ENABLED, "bulk-memory"),
            (PREFIX_ENABLED, "sign-ext"),
        ]);
        let section = find_custom_section(&wasm, TARGET_FEATURES).unwrap();
        let patched = add_reference_types_feature(&wasm, section.as_ref()).unwrap();

        // reference-types is now advertised...
        assert!(mentions(&patched));

        // ...and the pre-existing features survived.
        let section = find_custom_section(&patched, TARGET_FEATURES)
            .unwrap()
            .unwrap();
        let mut pos = section.content_start;
        let count = read_u32_leb128(&patched, &mut pos).unwrap();
        assert_eq!(count, 3);
        let mut names = Vec::new();
        for _ in 0..count {
            pos += 1; // prefix
            names.push(
                read_wasm_name(&patched, &mut pos, section.end)
                    .unwrap()
                    .to_string(),
            );
        }
        assert!(names.contains(&"bulk-memory".to_string()));
        assert!(names.contains(&"sign-ext".to_string()));
        assert!(names.contains(&REFERENCE_TYPES.to_string()));
    }
}
