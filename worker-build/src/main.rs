use std::{
    env::{self, VarError},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result};
use clap::Parser;
use log::info;

/// Default output dir passed to the internal build pipeline.
///
/// Note: all filesystem access must be relative to the crate root discovered by
/// `Build::try_from_opts` (i.e. `Build::out_dir`), NOT the process current-dir.
const OUT_DIR: &str = "build";

const SHIM_FILE: &str = include_str!("./js/shim.js");
const EMSCRIPTEN_WRAPPER_FILE: &str = include_str!("./js/emscripten_wrapper.js");
/// Emscripten post-link flags passed to `emcc --post-link`.
///
/// These configure the emscripten JS runtime for Workers compatibility:
///   - STACK_OVERFLOW_CHECK=0: skip the `emscripten_stack_get_end` assertion
///   - ERROR_ON_UNDEFINED_SYMBOLS=0: allow unresolved imports (wasm-bindgen glue)
///   - MODULARIZE=1 + EXPORT_ES6=1: ESM factory function output
///   - ENVIRONMENT=web: hardcode ENVIRONMENT_IS_WEB=true (avoids node/shell probes)
///   - ASSERTIONS=0: strip debug assertions from emscripten runtime
const EMCC_POSTLINK_FLAGS: &[&str] = &[
    "-sSTACK_OVERFLOW_CHECK=0",
    "-sERROR_ON_UNDEFINED_SYMBOLS=0",
    "-sMODULARIZE=1",
    "-sEXPORT_ES6=1",
    "-sENVIRONMENT=web",
    "-sASSERTIONS=0",
    // Suppress "--post-link is experimental" noise — we know.
    "-Wno-experimental",
];

pub(crate) mod binary;
mod build;
mod build_lock;
mod emoji;
mod lockfile;
mod main_legacy;
mod versions;

use build::{Build, BuildOptions};
use build_lock::BuildLock;

use crate::{
    binary::{Esbuild, GetBinary},
    build::target::WasmTarget,
    build::Target,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn fix_wasm_import(out_dir: &Path) -> Result<()> {
    let index_path = output_path(out_dir, "index.js");
    let content = fs::read_to_string(&index_path)
        .with_context(|| format!("Failed to read {}", index_path.display()))?;
    let updated_content = content.replace("import source ", "import ");
    fs::write(&index_path, updated_content)
        .with_context(|| format!("Failed to write {}", index_path.display()))?;
    Ok(())
}

fn update_package_json(out_dir: &Path) -> Result<()> {
    let package_json_path = output_path(out_dir, "package.json");

    let original_content = fs::read_to_string(&package_json_path)
        .with_context(|| format!("Failed to read {}", package_json_path.display()))?;
    let mut package_json: serde_json::Value = serde_json::from_str(&original_content)?;

    package_json["files"] = serde_json::json!(["index_bg.wasm", "index.js", "index.d.ts"]);
    package_json["main"] = serde_json::Value::String("index.js".to_string());
    package_json["sideEffects"] = serde_json::json!(["./index.js"]);

    let updated_content = serde_json::to_string_pretty(&package_json)?;
    fs::write(&package_json_path, updated_content)
        .with_context(|| format!("Failed to write {}", package_json_path.display()))?;
    Ok(())
}

pub fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<_> = env::args().collect();
    if args.len() > 1 && (args[1].as_str() == "--version" || args[1].as_str() == "-v") {
        println!("{VERSION}");
        return Ok(());
    }
    let no_panic_recovery = args.iter().any(|a| a == "--no-panic-recovery");

    let wasm_pack_opts = parse_wasm_pack_opts(env::args().skip(1))?;
    let mut builder = Build::try_from_opts(wasm_pack_opts)?;

    // IMPORTANT: Build output is always relative to the crate root discovered by
    // `Build::try_from_opts`, not the process current working directory.
    let out_dir = builder.out_dir.clone();

    // Acquire the build lock: waits for any concurrent build to finish,
    // then creates a fresh .tmp staging directory with a heartbeat thread.
    let lock = BuildLock::acquire(&out_dir)?;
    let staging_dir = lock.staging_dir().to_path_buf();

    // Point the builder at the staging directory
    builder.out_dir = staging_dir.clone();

    builder.init()?;

    let is_emscripten = builder.wasm_target == WasmTarget::Emscripten;

    if is_emscripten {
        // Emscripten path: use bundler target for wasm-bindgen (emscripten mode
        // is auto-detected via the __wasm_bindgen_emscripten_marker section).
        builder.target = Target::Bundler;
        builder.run()?;

        // Run emcc --post-link: takes the wasm-bindgen output wasm and
        // library_bindgen.js, produces output.js (emscripten JS runtime +
        // wasm-bindgen glue) and output.wasm (post-linked wasm binary).
        run_emcc_postlink(&staging_dir)?;

        // Generate the thin Workers wrapper that imports the emscripten
        // module factory and wasm, then exports Workers handlers.
        let wrapper = generate_emscripten_wrapper(&staging_dir)?;
        let wrapper_path = output_path(&staging_dir, "wrapper.js");
        fs::write(&wrapper_path, &wrapper)
            .with_context(|| format!("Failed to write {}", wrapper_path.display()))?;

        // Bundle with esbuild
        let esbuild_path = Esbuild.get_binary(None)?.0;
        bundle_emscripten(&staging_dir, &esbuild_path)?;

        remove_unused_files_emscripten(&staging_dir)?;

        create_wrapper_alias(&staging_dir, false)?;
    } else {
        let supports_reset_state = builder.supports_target_module_and_reset_state()?;
        let module_target =
            supports_reset_state && !no_panic_recovery && env::var("CUSTOM_SHIM").is_err();
        if module_target {
            builder
                .extra_args
                .push("--experimental-reset-state-function".to_string());
            builder.run()?;
        } else {
            if supports_reset_state {
                // Enable once we have DO bindings to offer an alternative
                // eprintln!("Using CUSTOM_SHIM will be deprecated in a future release.");
            } else {
                eprintln!("A newer version of wasm-bindgen is available. Update to use the latest workers-rs features.");
            }
            builder.target = Target::Bundler;
            builder.run()?;
        }

        let with_coredump = env::var("COREDUMP").is_ok();
        if with_coredump {
            println!("Adding wasm coredump");
            wasm_coredump(&staging_dir)?;
        }

        if module_target {
            let shim = SHIM_FILE
                .replace("$HANDLERS", &generate_handlers(&staging_dir)?)
                .replace(
                    "$PANIC_CRITICAL_ERROR",
                    if builder.panic_unwind {
                        ""
                    } else {
                        "criticalError = true;"
                    },
                );
            let shim_path = output_path(&staging_dir, "shim.js");
            fs::write(&shim_path, shim)
                .with_context(|| format!("Failed to write {}", shim_path.display()))?;

            add_export_wrappers(&staging_dir)?;

            update_package_json(&staging_dir)?;

            let esbuild_path = Esbuild.get_binary(None)?.0;
            bundle(&staging_dir, &esbuild_path)?;

            fix_wasm_import(&staging_dir)?;

            remove_unused_files(&staging_dir)?;

            create_wrapper_alias(&staging_dir, false)?;
        } else {
            main_legacy::process(&staging_dir)?;
            create_wrapper_alias(&staging_dir, true)?;
        }
    }

    // Swap staging entries into the real output directory and clean up.
    lock.finish()?;

    Ok(())
}

fn generate_handlers(out_dir: &Path) -> Result<String> {
    let index_path = output_path(out_dir, "index.js");
    let content = fs::read_to_string(&index_path)
        .with_context(|| format!("Failed to read {}", index_path.display()))?;

    // Extract ESM function exports from the wasm-bindgen generated output.
    // This code is specialized to what wasm-bindgen outputs for ESM and is therefore
    // brittle to upstream changes. It is comprehensive to current output patterns though.
    // TODO: Convert this to Wasm binary exports analysis for entry point detection instead.
    let mut func_names = Vec::new();
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("export function") {
            if let Some(bracket_pos) = rest.find("(") {
                let func_name = rest[..bracket_pos].trim();
                // strip the exported function (we re-wrap all handlers)
                if !SYSTEM_FNS.contains(&func_name) {
                    func_names.push(func_name);
                }
            }
        } else if let Some(rest) = line.strip_prefix("export {") {
            if let Some(as_pos) = rest.find(" as ") {
                let rest = &rest[as_pos + 4..];
                if let Some(brace_pos) = rest.find("}") {
                    let func_name = rest[..brace_pos].trim();
                    if !SYSTEM_FNS.contains(&func_name) {
                        func_names.push(func_name);
                    }
                }
            }
        }
    }

    let mut handlers = String::new();
    for func_name in func_names {
        if func_name == "fetch" && env::var("RUN_TO_COMPLETION").is_ok() {
            handlers += "Entrypoint.prototype.fetch = async function fetch(request) {
  let response = exports.fetch(request, this.env, this.ctx);
  this.ctx.waitUntil(response);
  return response;
}
";
        } else if func_name == "fetch" || func_name == "queue" || func_name == "scheduled" {
            // TODO: Switch these over to https://github.com/wasm-bindgen/wasm-bindgen/pull/4757
            // once that lands.
            handlers += &format!(
                "Entrypoint.prototype.{func_name} = function {func_name} (arg) {{
  return exports.{func_name}.call(this, arg, this.env, this.ctx);
}}
"
            );
        } else {
            handlers += &format!("Entrypoint.prototype.{func_name} = exports.{func_name};\n");
        }
    }

    Ok(handlers)
}

static SYSTEM_FNS: &[&str] = &["__wbg_reset_state", "setPanicHook"];

fn add_export_wrappers(out_dir: &Path) -> Result<()> {
    let index_path = output_path(out_dir, "index.js");
    let content = fs::read_to_string(&index_path)
        .with_context(|| format!("Failed to read {}", index_path.display()))?;

    let mut class_names = Vec::new();
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("export class ") {
            if let Some(brace_pos) = rest.find("{") {
                let class_name = rest[..brace_pos].trim();
                class_names.push(class_name.to_string());
            }
        }
    }

    let shim_path = output_path(out_dir, "shim.js");
    let mut output = fs::read_to_string(&shim_path)
        .with_context(|| format!("Failed to read {}", shim_path.display()))?;
    for class_name in class_names {
        output.push_str(&format!(
            "export const {class_name} = new Proxy(exports.{class_name}, classProxyHooks);\n"
        ));
    }
    fs::write(&shim_path, output)
        .with_context(|| format!("Failed to write {}", shim_path.display()))?;
    Ok(())
}

const INSTALL_HELP: &str = "In case you are missing the binary, you can install it using: `cargo install wasm-coredump-rewriter`";

fn wasm_coredump(out_dir: &Path) -> Result<()> {
    let coredump_flags = env::var("COREDUMP_FLAGS");
    let coredump_flags: Vec<&str> = if let Ok(flags) = &coredump_flags {
        flags.split(' ').collect()
    } else {
        vec![]
    };

    let mut child = Command::new("wasm-coredump-rewriter")
        .args(coredump_flags)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| {
            anyhow::anyhow!("failed to spawn wasm-coredump-rewriter: {err}\n\n{INSTALL_HELP}.")
        })?;

    let input_filename = output_path(out_dir, "index.wasm");

    let input_bytes = {
        let mut input = File::open(input_filename.clone())
            .map_err(|err| anyhow::anyhow!("failed to open input file: {err}"))?;

        let mut input_bytes = Vec::new();
        input
            .read_to_end(&mut input_bytes)
            .map_err(|err| anyhow::anyhow!("failed to open input file: {err}"))?;

        input_bytes
    };

    {
        let child_stdin = child.stdin.as_mut().unwrap();
        child_stdin
            .write_all(&input_bytes)
            .map_err(|err| anyhow::anyhow!("failed to write input file to rewriter: {err}"))?;
        // Close stdin to finish and avoid indefinite blocking
    }

    let output = child
        .wait_with_output()
        .map_err(|err| anyhow::anyhow!("failed to get rewriter's status: {err}"))?;

    if output.status.success() {
        // Open the input file again with truncate to write the output
        let mut f = fs::OpenOptions::new()
            .truncate(true)
            .write(true)
            .open(input_filename)
            .map_err(|err| anyhow::anyhow!("failed to open output file: {err}"))?;
        f.write_all(&output.stdout)
            .map_err(|err| anyhow::anyhow!("failed to write output file: {err}"))?;

        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!(format!(
            "failed to run Wasm coredump rewriter: {stdout}\n{stderr}"
        )))
    }
}

fn create_wrapper_alias(out_dir: &Path, legacy: bool) -> Result<()> {
    let msg = if !legacy {
        "// Use index.js directly, this file provided for backwards compat
// with former shim.mjs only.
"
    } else {
        ""
    };
    let path = if !legacy {
        "../index.js"
    } else {
        "./worker/shim.mjs"
    };
    let shim_content = format!(
        "{msg}export * from '{path}';
export {{ default }} from '{path}';
"
    );

    if !legacy {
        let worker_dir = output_path(out_dir, "worker");
        fs::create_dir_all(&worker_dir)
            .with_context(|| format!("Failed to create directory {}", worker_dir.display()))?;
        let shim_path = output_path(out_dir, "worker/shim.mjs");
        fs::write(&shim_path, shim_content)
            .with_context(|| format!("Failed to write {}", shim_path.display()))?;
    } else {
        let index_path = output_path(out_dir, "index.js");
        fs::write(&index_path, shim_content)
            .with_context(|| format!("Failed to write {}", index_path.display()))?;
    }
    Ok(())
}

#[derive(Parser)]
struct BuildArgs {
    #[clap(flatten)]
    pub build_options: BuildOptions,
}

fn parse_wasm_pack_opts<I>(args: I) -> Result<BuildOptions>
where
    I: IntoIterator<Item = String>,
{
    // This is done instead of explicitly constructing
    // BuildOptions to preserve the behavior of appending
    // arbitrary arguments in `args`.
    let mut build_args = vec![
        env!("CARGO_BIN_NAME").to_owned(),
        "--no-typescript".to_owned(),
        "--target".to_owned(),
        "module".to_owned(),
        "--out-dir".to_owned(),
        OUT_DIR.to_owned(),
        "--out-name".to_owned(),
        "index".to_owned(),
    ];

    build_args.extend(args);

    let command = BuildArgs::try_parse_from(build_args)?;
    Ok(command.build_options)
}

// Bundles the snippets and worker-related code into a single file.
fn bundle(out_dir: &Path, esbuild_path: &Path) -> Result<()> {
    let no_minify = !matches!(env::var("NO_MINIFY"), Err(VarError::NotPresent));
    let path = out_dir
        .canonicalize()
        .with_context(|| format!("Failed to resolve output directory {}", out_dir.display()))?;
    let esbuild_path = esbuild_path
        .canonicalize()
        .with_context(|| format!("Failed to resolve esbuild path {}", esbuild_path.display()))?;
    let mut command = Command::new(esbuild_path);
    command.args([
        "--external:./index_bg.wasm",
        "--external:cloudflare:sockets",
        "--external:cloudflare:workers",
        "--format=esm",
        "--bundle",
        "./shim.js",
        "--outfile=index.js",
        "--allow-overwrite",
    ]);

    if !no_minify {
        command.arg("--minify");
    }

    let exit_status = command.current_dir(path).spawn()?.wait()?;

    match exit_status.success() {
        true => Ok(()),
        false => anyhow::bail!("esbuild exited with status {exit_status}"),
    }
}

fn remove_unused_files(out_dir: &Path) -> Result<()> {
    let shim_path = output_path(out_dir, "shim.js");
    std::fs::remove_file(&shim_path)
        .with_context(|| format!("Failed to remove {}", shim_path.display()))?;
    let snippets_path = output_path(out_dir, "snippets");
    if snippets_path.exists() {
        std::fs::remove_dir_all(&snippets_path)
            .with_context(|| format!("Failed to remove {}", snippets_path.display()))?;
    }
    Ok(())
}

pub fn output_path(out_dir: &Path, name: impl AsRef<str>) -> PathBuf {
    out_dir.join(name.as_ref())
}

/// Run `emcc --post-link` on the wasm-bindgen output.
///
/// Takes `index_bg.wasm` (the wasm-bindgen output) and `library_bindgen.js`
/// (the wasm-bindgen emscripten JS library) and produces `output.js`
/// (emscripten JS runtime with wasm-bindgen glue inlined) and `output.wasm`
/// (the post-linked wasm binary with emscripten runtime support).
fn run_emcc_postlink(out_dir: &Path) -> Result<()> {
    let wasm_path = output_path(out_dir, "index_bg.wasm");
    let library_path = output_path(out_dir, "library_bindgen.js");
    let output_path = output_path(out_dir, "output.js");

    anyhow::ensure!(
        wasm_path.exists(),
        "wasm-bindgen output not found at {}",
        wasm_path.display()
    );
    anyhow::ensure!(
        library_path.exists(),
        "library_bindgen.js not found at {}",
        library_path.display()
    );

    let emcc =
        which::which("emcc").context("emcc not found on PATH; required for --emscripten builds")?;

    let mut cmd = Command::new(emcc);
    cmd.arg("--post-link")
        .arg(&wasm_path)
        .arg("--js-library")
        .arg(&library_path)
        .arg("-o")
        .arg(&output_path);

    for flag in EMCC_POSTLINK_FLAGS {
        cmd.arg(flag);
    }

    let exit_status = cmd.spawn()?.wait()?;
    if !exit_status.success() {
        anyhow::bail!("emcc --post-link exited with status {exit_status}");
    }

    // Verify output files were created
    let output_wasm = output_path.with_file_name("output.wasm");
    anyhow::ensure!(
        output_path.exists(),
        "emcc --post-link did not produce {}",
        output_path.display()
    );
    anyhow::ensure!(
        output_wasm.exists(),
        "emcc --post-link did not produce {}",
        output_wasm.display()
    );

    info!(
        "emcc --post-link produced {} and {}",
        output_path.display(),
        output_wasm.display()
    );

    Ok(())
}

/// Extract exported function names from `library_bindgen.js`.
///
/// The wasm-bindgen emscripten output contains `Module.<name> = <name>;`
/// assignments inside `$initBindgen`. We scan for these to determine which
/// Workers handlers to wire up.
fn extract_exported_functions(out_dir: &Path) -> Result<Vec<String>> {
    let library_path = output_path(out_dir, "library_bindgen.js");
    let content = fs::read_to_string(&library_path)
        .with_context(|| format!("Failed to read {}", library_path.display()))?;

    let mut func_names = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Module.") {
            if let Some(eq_pos) = rest.find('=') {
                let name = rest[..eq_pos].trim();
                if !name.starts_with("__wbindgen") && !name.starts_with("__wbg") {
                    func_names.push(name.to_string());
                }
            }
        }
    }

    Ok(func_names)
}

/// Generate the Workers wrapper for the emscripten post-link output.
///
/// This is a thin ESM module that:
///   1. Imports the emscripten module factory (`output.js`)
///   2. Imports the wasm module (`output.wasm`)
///   3. Lazily instantiates the emscripten module with a custom
///      `instantiateWasm` hook (so Workers' wasm module loading works)
///   4. Exports Workers-compatible handlers based on which functions
///      the Rust crate exports via `#[wasm_bindgen]`
fn generate_emscripten_wrapper(out_dir: &Path) -> Result<String> {
    let func_names = extract_exported_functions(out_dir)?;
    info!("Detected exported functions: {:?}", func_names);

    let handlers = generate_emscripten_handlers(&func_names);
    let js = EMSCRIPTEN_WRAPPER_FILE.replace("$HANDLERS", &handlers);

    Ok(js)
}

/// Build the handler method bodies for the emscripten wrapper's default export.
fn generate_emscripten_handlers(func_names: &[String]) -> String {
    let mut handlers = String::new();

    // fetch handler — always present (falls back to 404)
    if func_names.contains(&"fetch".to_string()) {
        handlers.push_str(
            "  async fetch(request, env, ctx) {
    const mod = await getModule();
    return mod.fetch(request, env, ctx);
  },
",
        );
    } else if func_names.contains(&"handle_request".to_string()) {
        handlers.push_str(
            "  async fetch(request, env, ctx) {
    const mod = await getModule();
    const result = await mod.handle_request();
    return new Response(result, {
      headers: { 'Content-Type': 'text/html; charset=utf-8' },
    });
  },
",
        );
    } else {
        handlers.push_str(
            "  async fetch(request, env, ctx) {
    return new Response('No fetch handler exported', { status: 404 });
  },
",
        );
    }

    // scheduled handler
    if func_names.contains(&"scheduled".to_string()) {
        handlers.push_str(
            "  async scheduled(event, env, ctx) {
    const mod = await getModule();
    return mod.scheduled(event, env, ctx);
  },
",
        );
    }

    // queue handler
    if func_names.contains(&"queue".to_string()) {
        handlers.push_str(
            "  async queue(batch, env, ctx) {
    const mod = await getModule();
    return mod.queue(batch, env, ctx);
  },
",
        );
    }

    handlers
}

/// Bundle emscripten wrapper + emcc output with esbuild.
///
/// The wrapper.js imports output.js (emscripten runtime) and output.wasm.
/// esbuild inlines output.js into the bundle but keeps output.wasm external
/// since Workers load wasm modules directly.
fn bundle_emscripten(out_dir: &Path, esbuild_path: &Path) -> Result<()> {
    let path = out_dir
        .canonicalize()
        .with_context(|| format!("Failed to resolve output directory {}", out_dir.display()))?;
    let esbuild_path = esbuild_path
        .canonicalize()
        .with_context(|| format!("Failed to resolve esbuild path {}", esbuild_path.display()))?;

    let no_minify = !matches!(env::var("NO_MINIFY"), Err(env::VarError::NotPresent));

    let mut command = Command::new(esbuild_path);
    command.args([
        // Mark wasm as external — Workers handle wasm module loading
        "--external:./output.wasm",
        "--format=esm",
        "--bundle",
        "./wrapper.js",
        "--outfile=index.js",
        "--allow-overwrite",
    ]);

    if !no_minify {
        command.arg("--minify");
    }

    let exit_status = command.current_dir(path).spawn()?.wait()?;

    match exit_status.success() {
        true => Ok(()),
        false => anyhow::bail!("esbuild exited with status {exit_status}"),
    }
}

/// Remove build artifacts that aren't needed in the final output for emscripten.
///
/// After bundling, the final output contains only:
///   - `index.js` — bundled ESM entry point (esbuild output)
///   - `output.wasm` — post-linked wasm binary (external to esbuild)
///   - `worker/shim.mjs` — backwards-compat alias
///
/// Everything else is intermediate and can be removed.
fn remove_unused_files_emscripten(out_dir: &Path) -> Result<()> {
    let intermediates = [
        "wrapper.js",
        "output.js",
        "library_bindgen.js",
        "index_bg.wasm",
        "index_bg.wasm.d.ts",
        "index.d.ts",
        "package.json",
    ];

    for name in &intermediates {
        let path = output_path(out_dir, name);
        if path.exists() {
            std::fs::remove_file(&path)
                .with_context(|| format!("Failed to remove {}", path.display()))?;
        }
    }

    // Remove snippets if present
    let snippets_path = output_path(out_dir, "snippets");
    if snippets_path.exists() {
        std::fs::remove_dir_all(&snippets_path)
            .with_context(|| format!("Failed to remove {}", snippets_path.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::parse_wasm_pack_opts;
    #[test]
    fn test_wasm_pack_args_build_arg() {
        let args = vec!["--release".to_owned()];
        let result = parse_wasm_pack_opts(args);
        assert!(result.is_ok());
    }
}
