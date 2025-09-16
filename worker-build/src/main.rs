use std::{
    env::{self, VarError},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::Result;

use clap::Parser;

const OUT_DIR: &str = "build";

const SHIM_FILE: &str = include_str!("./js/shim.js");

mod install;
mod wasm_pack;

use wasm_pack::command::build::{Build, BuildOptions};

fn fix_wasm_import() -> Result<()> {
    let index_path = output_path("index.js");
    if !index_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&index_path)?;
    let updated_content = content.replace("import source wasmModule", "import wasmModule");
    fs::write(&index_path, updated_content)?;
    Ok(())
}

fn update_package_json() -> Result<()> {
    let package_json_path = output_path("package.json");

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
    let out_path = output_path("");
    if out_path.exists() {
        fs::remove_dir_all(out_path)?;
    }

    // Our tests build the bundle ourselves.
    if !cfg!(test) {
        wasm_pack_build(env::args().skip(1))?;
    }

    update_package_json()?;

    let with_coredump = env::var("COREDUMP").is_ok();
    if with_coredump {
        println!("Adding wasm coredump");
        wasm_coredump()?;
    }

    let esbuild_path = install::ensure_esbuild()?;

    let shim_template = match env::var("CUSTOM_SHIM") {
        Ok(path) => {
            let path = Path::new(&path).to_owned();
            println!("Using custom shim from {}", path.display());
            // NOTE: we fail in case that file doesnt exist or something else happens
            fs::read_to_string(path)?
        }
        Err(_) => SHIM_FILE.to_owned(),
    };

    let wait_until_response = if env::var("RUN_TO_COMPLETION").is_ok() {
        "this.ctx.waitUntil(response);"
    } else {
        ""
    };

    let shim = shim_template.replace("$WAIT_UNTIL_RESPONSE", wait_until_response);

    fs::write(output_path("shim.js"), shim)?;

    bundle(&esbuild_path)?;

    fix_wasm_import()?;

    remove_unused_files()?;

    create_shim_mjs()?;

    Ok(())
}

const INSTALL_HELP: &str = "In case you are missing the binary, you can install it using: `cargo install wasm-coredump-rewriter`";

fn wasm_coredump() -> Result<()> {
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

    let input_filename = output_path("index.wasm");

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

fn create_shim_mjs() -> Result<()> {
    let shim_content = r#"// Use index.js directly, this file provided for backwards compat
// with former shim.mjs only.
export * from '../index.js'
export { default } from '../index.js'
"#;

    fs::create_dir(output_path("worker"))?;
    fs::write(output_path("worker/shim.mjs"), shim_content)?;
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

fn wasm_pack_build<I>(args: I) -> Result<()>
where
    I: IntoIterator<Item = String>,
{
    let opts = parse_wasm_pack_opts(args)?;

    let mut build = Build::try_from_opts(opts)?;

    build.run()
}

// Bundles the snippets and worker-related code into a single file.
fn bundle(esbuild_path: &Path) -> Result<()> {
    let no_minify = !matches!(env::var("NO_MINIFY"), Err(VarError::NotPresent));
    let path = PathBuf::from(OUT_DIR).canonicalize()?;
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

fn remove_unused_files() -> Result<()> {
    std::fs::remove_file(output_path("index_bg.wasm.d.ts"))?;
    std::fs::remove_file(output_path("shim.js"))?;
    std::fs::remove_dir_all(output_path("snippets"))?;
    Ok(())
}

pub fn output_path(name: impl AsRef<str>) -> PathBuf {
    PathBuf::from(OUT_DIR).join(name.as_ref())
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

    #[test]
    fn test_wasm_pack_args_additional_arg() {
        let args = vec!["--weak-refs".to_owned()];
        let result = parse_wasm_pack_opts(args).unwrap();

        assert!(result.weak_refs);
    }
}
