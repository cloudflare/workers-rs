use std::{
    env::{self, VarError},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::Result;
use clap::Parser;

/// Default output dir passed to the internal build pipeline.
///
/// Note: all filesystem access must be relative to the crate root discovered by
/// `Build::try_from_opts` (i.e. `Build::out_dir`), NOT the process current-dir.
const OUT_DIR: &str = "build";

const SHIM_FILE: &str = include_str!("./js/shim.js");

pub(crate) mod binary;
mod build;
mod emoji;
mod lockfile;
mod main_legacy;
mod versions;

use build::{Build, BuildOptions};

use crate::{
    binary::{Esbuild, GetBinary},
    build::Target,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn fix_wasm_import(out_dir: &Path) -> Result<()> {
    let index_path = output_path(out_dir, "index.js");
    let content = fs::read_to_string(&index_path)?;
    let updated_content = content.replace("import source ", "import ");
    fs::write(&index_path, updated_content)?;
    Ok(())
}

fn update_package_json(out_dir: &Path) -> Result<()> {
    let package_json_path = output_path(out_dir, "package.json");

    let original_content = fs::read_to_string(&package_json_path)?;
    let mut package_json: serde_json::Value = serde_json::from_str(&original_content)?;

    package_json["files"] = serde_json::json!(["index_bg.wasm", "index.js", "index.d.ts"]);
    package_json["main"] = serde_json::Value::String("index.js".to_string());
    package_json["sideEffects"] = serde_json::json!(["./index.js"]);

    let updated_content = serde_json::to_string_pretty(&package_json)?;
    fs::write(package_json_path, updated_content)?;
    Ok(())
}

pub fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<_> = env::args().collect();
    if args.len() > 1 && (args[1].as_str() == "--version" || args[1].as_str() == "-v") {
        println!("{}", VERSION);
        return Ok(());
    }
    let no_panic_recovery = args.iter().any(|a| a == "--no-panic-recovery");

    let wasm_pack_opts = parse_wasm_pack_opts(env::args().skip(1))?;
    let mut builder = Build::try_from_opts(wasm_pack_opts)?;

    // IMPORTANT: Build output is always relative to the crate root discovered by
    // `Build::try_from_opts`, not the process current working directory.
    let out_dir = builder.out_dir.clone();

    if out_dir.is_dir() {
        fs::remove_dir_all(&out_dir)?;
    } else if out_dir.exists() {
        fs::remove_file(&out_dir)?;
    }

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
        wasm_coredump(&out_dir)?;
    }

    if module_target {
        let shim = SHIM_FILE
            .replace("$HANDLERS", &generate_handlers(&out_dir)?)
            .replace(
                "$PANIC_CRITICAL_ERROR",
                if builder.panic_unwind {
                    ""
                } else {
                    "criticalError = true;"
                },
            );
        fs::write(output_path(&out_dir, "shim.js"), shim)?;

        add_export_wrappers(&out_dir)?;

        update_package_json(&out_dir)?;

        let esbuild_path = Esbuild.get_binary(None)?.0;
        bundle(&out_dir, &esbuild_path)?;

        fix_wasm_import(&out_dir)?;

        remove_unused_files(&out_dir)?;

        create_wrapper_alias(&out_dir, false)?;
    } else {
        main_legacy::process(&out_dir)?;
        create_wrapper_alias(&out_dir, true)?;
    }

    Ok(())
}

fn generate_handlers(out_dir: &Path) -> Result<String> {
    let index_path = output_path(out_dir, "index.js");
    let content = fs::read_to_string(&index_path)?;

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
    let content = fs::read_to_string(&index_path)?;

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
    let mut output = fs::read_to_string(&shim_path)?;
    for class_name in class_names {
        output.push_str(&format!(
            "export const {class_name} = new Proxy(exports.{class_name}, classProxyHooks);\n"
        ));
    }
    fs::write(&shim_path, output)?;
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
        fs::create_dir_all(output_path(out_dir, "worker"))?;
        fs::write(output_path(out_dir, "worker/shim.mjs"), shim_content)?;
    } else {
        fs::write(output_path(out_dir, "index.js"), shim_content)?;
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
    let path = out_dir.canonicalize()?;
    let esbuild_path = esbuild_path.canonicalize()?;
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
        false => anyhow::bail!("esbuild exited with status {}", exit_status),
    }
}

fn remove_unused_files(out_dir: &Path) -> Result<()> {
    std::fs::remove_file(output_path(out_dir, "index_bg.wasm.d.ts"))?;
    std::fs::remove_file(output_path(out_dir, "shim.js"))?;
    let snippets_path = output_path(out_dir, "snippets");
    if snippets_path.exists() {
        std::fs::remove_dir_all(snippets_path)?;
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
