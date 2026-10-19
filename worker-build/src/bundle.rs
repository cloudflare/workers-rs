use std::{
    env::{self, VarError},
    path::Path,
    sync::Arc,
};

use anyhow::{Context, Result};
use rolldown::{
    Bundler, BundlerOptions, CodeSplittingMode, IsExternal, OutputFormat, Platform,
    RawMinifyOptions,
};

use crate::build::PBAR;

/// Bundles `input` and everything it imports into the single ESM file `output`, both relative
/// to `dir`. `external` lists specifiers left as imports in the output; a trailing `*` matches
/// any suffix, as with esbuild's `--external`.
pub fn bundle(
    dir: &Path,
    input: &str,
    output: &str,
    external: &[&str],
    platform: Platform,
) -> Result<()> {
    let no_minify = !matches!(env::var("NO_MINIFY"), Err(VarError::NotPresent));
    let cwd = dir
        .canonicalize()
        .with_context(|| format!("Failed to resolve path {}", dir.display()))?;

    let external: Vec<String> = external.iter().map(|s| s.to_string()).collect();
    let is_external = IsExternal::Fn(Some(Arc::new(move |specifier, _importer, _resolved| {
        let matched = external
            .iter()
            .any(|pattern| match pattern.strip_suffix('*') {
                Some(prefix) => specifier.starts_with(prefix),
                None => specifier == pattern,
            });
        Box::pin(async move { Ok(matched) })
    })));

    let options = BundlerOptions {
        input: Some(vec![input.to_string().into()]),
        cwd: Some(cwd),
        file: Some(output.to_string()),
        format: Some(OutputFormat::Esm),
        platform: Some(platform),
        external: Some(is_external),
        code_splitting: Some(CodeSplittingMode::Bool(false)),
        minify: Some(RawMinifyOptions::Bool(!no_minify)),
        ..Default::default()
    };

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Failed to start bundler runtime")?;

    let output = runtime.block_on(async {
        let mut bundler = Bundler::new(options)?;
        let output = bundler.write().await;
        bundler.close().await?;
        output
    });

    match output {
        Ok(output) => {
            for warning in output.warnings {
                PBAR.warn(&warning.to_diagnostic().to_color_string());
            }
            Ok(())
        }
        Err(errors) => {
            let errors = errors
                .into_vec()
                .iter()
                .map(|error| error.to_diagnostic().to_color_string())
                .collect::<Vec<_>>()
                .join("\n");
            anyhow::bail!("Bundling failed:\n{errors}")
        }
    }
}
