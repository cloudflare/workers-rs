//! Arguments are forwarded directly to wasm-pack

use std::{
    convert::TryInto,
    env::{self, VarError},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::Result;

use clap::Parser;
use wasm_pack::command::build::{Build, BuildOptions};

const OUT_DIR: &str = "build";
const OUT_NAME: &str = "index";
const WORKER_SUBDIR: &str = "worker";

const SHIM_TEMPLATE: &str = include_str!("./js/shim.js");

const WASM_IMPORT: &str = r#"let wasm;
export function __wbg_set_wasm(val) {
    wasm = val;
}

"#;

const WASM_IMPORT_REPLACEMENT: &str = r#"
import wasm from './glue.js';

export function getMemory() {
    return wasm.memory;
}
"#;

mod install;

pub fn main() -> Result<()> {
    // Our tests build the bundle ourselves.
    if !cfg!(test) {
        wasm_pack_build(env::args().skip(1))?;
    }

    let with_coredump = env::var("COREDUMP").is_ok();
    if with_coredump {
        println!("Adding wasm coredump");
        wasm_coredump()?;
    }

    let esbuild_path = install::ensure_esbuild()?;

    create_worker_dir()?;
    copy_generated_code_to_worker_dir()?;
    use_glue_import()?;

    write_string_to_file(worker_path("glue.js"), include_str!("./js/glue.js"))?;
    let shim = if env::var("RUN_TO_COMPLETION").is_ok() {
        SHIM_TEMPLATE.replace("$WAIT_UNTIL_RESPONSE", "this.ctx.waitUntil(response);")
    } else {
        SHIM_TEMPLATE.replace("$WAIT_UNTIL_RESPONSE", "")
    };

    write_string_to_file(worker_path("shim.js"), shim)?;

    bundle(&esbuild_path)?;

    remove_unused_js()?;

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

    let input_filename = output_path("index_bg.wasm");

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
        "bundler".to_owned(),
        "--out-dir".to_owned(),
        OUT_DIR.to_owned(),
        "--out-name".to_owned(),
        OUT_NAME.to_owned(),
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

fn create_worker_dir() -> Result<()> {
    // create a directory for our worker to live in
    let worker_dir = PathBuf::from(OUT_DIR).join(WORKER_SUBDIR);

    // remove anything that already exists
    if worker_dir.is_dir() {
        fs::remove_dir_all(&worker_dir)?
    } else if worker_dir.is_file() {
        fs::remove_file(&worker_dir)?
    };

    // create an output dir
    fs::create_dir(worker_dir)?;

    Ok(())
}

fn copy_generated_code_to_worker_dir() -> Result<()> {
    let glue_src = output_path(format!("{OUT_NAME}_bg.js"));
    let glue_dest = worker_path(format!("{OUT_NAME}_bg.js"));

    let wasm_src = output_path(format!("{OUT_NAME}_bg.wasm"));
    let wasm_dest = worker_path(format!("{OUT_NAME}.wasm"));

    // wasm-bindgen supports adding arbitrary JavaScript for a library, so we need to move that as well.
    // https://rustwasm.github.io/wasm-bindgen/reference/js-snippets.html
    let snippets_src = output_path("snippets");
    let snippets_dest = worker_path("snippets");

    for (src, dest) in [
        (glue_src, glue_dest),
        (wasm_src, wasm_dest),
        (snippets_src, snippets_dest),
    ] {
        if !src.exists() {
            continue;
        }

        fs::rename(src, dest)?;
    }

    Ok(())
}

// Replaces the wasm import with an import that instantiates the WASM modules itself.
fn use_glue_import() -> Result<()> {
    let bindgen_glue_path = worker_path(format!("{OUT_NAME}_bg.js"));
    let old_bindgen_glue = read_file_to_string(&bindgen_glue_path)?;
    let fixed_bindgen_glue = old_bindgen_glue.replace(WASM_IMPORT, WASM_IMPORT_REPLACEMENT);
    write_string_to_file(bindgen_glue_path, fixed_bindgen_glue)?;
    Ok(())
}

// Bundles the snippets and worker-related code into a single file.
fn bundle(esbuild_path: &Path) -> Result<()> {
    let no_minify = !matches!(env::var("NO_MINIFY"), Err(VarError::NotPresent));
    let path = PathBuf::from(OUT_DIR).join(WORKER_SUBDIR).canonicalize()?;
    let esbuild_path = esbuild_path.canonicalize()?;
    let mut command = Command::new(esbuild_path);
    command.args([
        "--external:./index.wasm",
        "--external:cloudflare:sockets",
        "--external:cloudflare:workers",
        "--format=esm",
        "--bundle",
        "./shim.js",
        "--outfile=shim.mjs",
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

// After bundling there's no reason why we'd want to upload our now un-used JavaScript so we'll
// delete it.
fn remove_unused_js() -> Result<()> {
    let snippets_dir = worker_path("snippets");

    if snippets_dir.exists() {
        std::fs::remove_dir_all(&snippets_dir)?;
    }

    for to_remove in [
        format!("{OUT_NAME}_bg.js"),
        "shim.js".into(),
        "glue.js".into(),
    ] {
        std::fs::remove_file(worker_path(to_remove))?;
    }

    Ok(())
}

fn read_file_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let file_size = path.as_ref().metadata()?.len().try_into()?;
    let mut file = File::open(path)?;
    let mut buf = Vec::with_capacity(file_size);
    file.read_to_end(&mut buf)?;
    String::from_utf8(buf).map_err(anyhow::Error::from)
}

fn write_string_to_file<P: AsRef<Path>>(path: P, contents: impl AsRef<str>) -> Result<()> {
    let mut file = File::create(path)?;
    file.write_all(contents.as_ref().as_bytes())?;

    Ok(())
}

pub fn worker_path(name: impl AsRef<str>) -> PathBuf {
    PathBuf::from(OUT_DIR)
        .join(WORKER_SUBDIR)
        .join(name.as_ref())
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
