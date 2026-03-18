use std::{
    env::{self, VarError},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result};
use clap::Parser;

/// Default output dir passed to the internal build pipeline.
///
/// Note: all filesystem access must be relative to the crate root discovered by
/// `Build::try_from_opts` (i.e. `Build::out_dir`), NOT the process current-dir.
const OUT_DIR: &str = "build";

const SHIM_FILE: &str = include_str!("./js/shim.js");
const SHIM_UNWIND_FILE: &str = include_str!("./js/shim-unwind.js");

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
        let index_path = output_path(&staging_dir, "index.js");
        let index_content = fs::read_to_string(&index_path)
            .with_context(|| format!("Failed to read {}", index_path.display()))?;

        let exported_fns = collect_exported_fns(&index_content);
        let do_classes = detect_do_classes(&exported_fns);

        let handlers = generate_handlers(&exported_fns, builder.panic_unwind);
        let do_class_js = generate_do_classes(&do_classes, &exported_fns, builder.panic_unwind);

        let shim = if builder.panic_unwind {
            SHIM_UNWIND_FILE
        } else {
            SHIM_FILE
        }
        .replace("$HANDLERS", &handlers)
        .replace("$DO_CLASSES", &do_class_js);
        let shim_path = output_path(&staging_dir, "shim.js");
        fs::write(&shim_path, shim)
            .with_context(|| format!("Failed to write {}", shim_path.display()))?;

        add_class_reexports(&staging_dir, &do_classes)?;

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

    // Swap staging entries into the real output directory and clean up.
    lock.finish()?;

    Ok(())
}

/// Names of wasm-bindgen internal/system exports that must NOT be wrapped as
/// Worker handler methods on the Entrypoint prototype.
static SYSTEM_FNS: &[&str] = &["__wbg_reset_state", "setPanicHook"];

/// The well-known method suffixes emitted by the `#[durable_object]` macro.
/// Each DO class exports `ClassName__DURABLE_OBJECT_INIT` plus a subset of these.
static DO_METHOD_SUFFIXES: &[&str] = &[
    "fetch",
    "alarm",
    "webSocketMessage",
    "webSocketClose",
    "webSocketError",
];

/// Collects all exported function names from the wasm-bindgen output.
fn collect_exported_fns(content: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("export function ") {
            if let Some(paren) = rest.find('(') {
                names.push(rest[..paren].trim().to_string());
            }
        } else if let Some(rest) = trimmed.strip_prefix("export {") {
            if let Some(as_pos) = rest.find(" as ") {
                let rest = &rest[as_pos + 4..];
                if let Some(brace) = rest.find('}') {
                    names.push(rest[..brace].trim().to_string());
                }
            }
        }
    }
    names
}

/// Detect Durable Object class names by finding `ClassName__DURABLE_OBJECT_INIT` exports.
fn detect_do_classes(exported_fns: &[String]) -> Vec<String> {
    exported_fns
        .iter()
        .filter_map(|name| name.strip_suffix("__DURABLE_OBJECT_INIT").map(String::from))
        .collect()
}

/// Generate `Entrypoint.prototype.*` handler methods for Worker-level exports
/// (fetch, scheduled, queue, etc.), excluding system fns and DO method exports.
///
/// In abort mode (`panic_unwind == false`), each handler is wrapped with
/// `checkReinitialize()` and a try/catch that flags `RuntimeError` as critical.
fn generate_handlers(exported_fns: &[String], panic_unwind: bool) -> String {
    let mut handlers = String::new();
    for func_name in exported_fns {
        // Skip system functions
        if SYSTEM_FNS.contains(&func_name.as_str()) {
            continue;
        }
        // Skip DO method exports (contain "__")
        if func_name.contains("__") {
            continue;
        }

        if func_name == "fetch" && env::var("RUN_TO_COMPLETION").is_ok() {
            if panic_unwind {
                handlers += "Entrypoint.prototype.fetch = async function fetch(request) {
  let response = exports.fetch(request, this.env, this.ctx);
  this.ctx.waitUntil(response);
  return response;
}
";
            } else {
                handlers += "Entrypoint.prototype.fetch = async function fetch(request) {
  checkReinitialize();
  try {
    let response = exports.fetch(request, this.env, this.ctx);
    this.ctx.waitUntil(response);
    return response;
  } catch (e) { handleMaybeCritical(e); throw e; }
}
";
            }
        } else if func_name == "fetch" || func_name == "queue" || func_name == "scheduled" {
            if panic_unwind {
                handlers += &format!(
                    "Entrypoint.prototype.{func_name} = function {func_name} (arg) {{
  return exports.{func_name}.call(this, arg, this.env, this.ctx);
}}
"
                );
            } else {
                handlers += &format!(
                    "Entrypoint.prototype.{func_name} = function {func_name} (arg) {{
  checkReinitialize();
  try {{
    return exports.{func_name}.call(this, arg, this.env, this.ctx);
  }} catch (e) {{ handleMaybeCritical(e); throw e; }}
}}
"
                );
            }
        } else if panic_unwind {
            handlers += &format!("Entrypoint.prototype.{func_name} = exports.{func_name};\n");
        } else {
            handlers += &format!(
                "Entrypoint.prototype.{func_name} = function {func_name} (...args) {{
  checkReinitialize();
  try {{
    return exports.{func_name}(...args);
  }} catch (e) {{ handleMaybeCritical(e); throw e; }}
}}
"
            );
        }
    }
    handlers
}

/// Generate thin JS class wrappers for Durable Objects and a reinit callback.
///
/// For the **unwind** shim, classes simply delegate to `exports.ClassName__method`.
/// The post-reinit hook callback re-initialises each DO with stashed state/env.
///
/// For the **abort** shim, each method also includes `checkReinitialize()` and a
/// lazy staleness check via `instanceId`.
fn generate_do_classes(
    do_classes: &[String],
    exported_fns: &[String],
    panic_unwind: bool,
) -> String {
    if do_classes.is_empty() {
        return String::new();
    }

    let mut output = String::new();

    // Global key counter shared by all DO classes.
    // Each DO instance gets a unique key used to look up the Rust struct.
    output += "let __do_next_key = 0;\n";

    // Map from key → { class_name, state, env } for reinit after wasm reset.
    output += "const __do_live = new Map();\n\n";

    for class_name in do_classes {
        let methods: Vec<&str> = DO_METHOD_SUFFIXES
            .iter()
            .filter(|m| exported_fns.contains(&format!("{class_name}__{m}")))
            .copied()
            .collect();

        output += &format!("class {class_name} {{\n");

        // Constructor: assign a key, stash state/env, call Rust init
        if panic_unwind {
            output += &format!(
                "  constructor(state, env) {{\n\
                 \x20   this.__key = __do_next_key++;\n\
                 \x20   __do_live.set(this.__key, {{ cls: \"{class_name}\", state, env }});\n\
                 \x20   exports.{class_name}__DURABLE_OBJECT_INIT(this.__key, state, env);\n\
                 \x20 }}\n"
            );
        } else {
            output += &format!(
                "  constructor(state, env) {{\n\
                 \x20   checkReinitialize();\n\
                 \x20   this.__key = __do_next_key++;\n\
                 \x20   this.__insId = instanceId;\n\
                 \x20   __do_live.set(this.__key, {{ cls: \"{class_name}\", state, env }});\n\
                 \x20   exports.{class_name}__DURABLE_OBJECT_INIT(this.__key, state, env);\n\
                 \x20 }}\n"
            );
        }

        // Methods
        for method in &methods {
            let args = match *method {
                "fetch" => "req",
                "alarm" => "",
                "webSocketMessage" => "ws, message",
                "webSocketClose" => "ws, code, reason, was_clean",
                "webSocketError" => "ws, error",
                _ => "",
            };
            let args_with_key = if args.is_empty() {
                "this.__key".to_string()
            } else {
                format!("this.__key, {args}")
            };

            if panic_unwind {
                output += &format!(
                    "  {method}({args}) {{ return exports.{class_name}__{method}({args_with_key}); }}\n"
                );
            } else {
                output += &format!(
                    "  {method}({args}) {{\n\
                     \x20   checkReinitialize();\n\
                     \x20   if (this.__insId !== instanceId) {{\n\
                     \x20     const e = __do_live.get(this.__key);\n\
                     \x20     if (e) exports[e.cls + '__DURABLE_OBJECT_INIT'](this.__key, e.state, e.env);\n\
                     \x20     this.__insId = instanceId;\n\
                     \x20   }}\n\
                     \x20   return exports.{class_name}__{method}({args_with_key});\n\
                     \x20 }}\n"
                );
            }
        }

        output += "}\n";
        output += &format!("export {{ {class_name} }};\n\n");
    }

    // Reinit callback: after wasm reset, re-init every live DO instance
    // with its stashed state/env so subsequent method calls succeed.
    if panic_unwind {
        output += "globalThis.__worker_reinit_dos = function () {\n\
                   \x20 for (const [key, e] of __do_live) {\n\
                   \x20   exports[e.cls + '__DURABLE_OBJECT_INIT'](key, e.state, e.env);\n\
                   \x20 }\n\
                   };\n";
    }

    output
}

/// Re-export non-DO classes (wasm-bindgen utility types) directly, without Proxy.
fn add_class_reexports(out_dir: &Path, do_classes: &[String]) -> Result<()> {
    let index_path = output_path(out_dir, "index.js");
    let content = fs::read_to_string(&index_path)
        .with_context(|| format!("Failed to read {}", index_path.display()))?;

    let mut class_names = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("export class ") {
            if let Some(brace_pos) = rest.find('{') {
                let class_name = rest[..brace_pos].trim();
                // Only re-export non-DO classes (DOs are now flat fn exports)
                if !do_classes.iter().any(|dc| dc == class_name) {
                    class_names.push(class_name.to_string());
                }
            }
        }
    }

    if class_names.is_empty() {
        return Ok(());
    }

    let shim_path = output_path(out_dir, "shim.js");
    let mut output = fs::read_to_string(&shim_path)
        .with_context(|| format!("Failed to read {}", shim_path.display()))?;
    for class_name in &class_names {
        use std::fmt::Write;
        writeln!(
            &mut output,
            "export const {class_name} = exports.{class_name};"
        )?;
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
