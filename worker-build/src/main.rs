//! Arguments are fowarded directly to wasm-pack

use std::{
    convert::TryInto,
    env,
    ffi::OsStr,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;

const OUT_DIR: &str = "build";
const OUT_NAME: &str = "index";
const WORKER_SUBDIR: &str = "worker";

mod install;

pub fn main() -> Result<()> {
    // Our tests build the bundle ourselves.
    if !cfg!(test) {
        install::ensure_wasm_pack()?;
        wasm_pack_build(env::args_os().skip(1))?;
    }

    let esbuild_path = install::ensure_esbuild()?;

    create_worker_dir()?;
    copy_generated_code_to_worker_dir()?;
    use_glue_import()?;

    write_string_to_file(worker_path("glue.js"), include_str!("./js/glue.js"))?;
    write_string_to_file(worker_path("shim.js"), include_str!("./js/shim.js"))?;

    bundle(&esbuild_path)?;

    remove_unused_js()?;

    Ok(())
}

fn wasm_pack_build<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let exit_status = Command::new("wasm-pack")
        .arg("build")
        .arg("--no-typescript")
        .args(&["--target", "bundler"])
        .args(&["--out-dir", OUT_DIR])
        .args(&["--out-name", OUT_NAME])
        .args(args)
        .spawn()?
        .wait()?;

    match exit_status.success() {
        true => Ok(()),
        false => anyhow::bail!("wasm-pack exited with status {}", exit_status),
    }
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
    let old_import = format!("import * as wasm from './{OUT_NAME}_bg.wasm'");
    let fixed_bindgen_glue = old_bindgen_glue.replace(&old_import, "import wasm from './glue.js'");
    write_string_to_file(bindgen_glue_path, fixed_bindgen_glue)?;
    Ok(())
}

// Bundles the snippets and worker-related code into a single file.
fn bundle(esbuild_path: &Path) -> Result<()> {
    let path = PathBuf::from(OUT_DIR).join(WORKER_SUBDIR).canonicalize()?;
    let esbuild_path = esbuild_path.canonicalize()?;
    let exit_status = Command::new(esbuild_path)
        .args(&[
            "--external:./index.wasm",
            "--format=esm",
            "--bundle",
            "./shim.js",
            "--outfile=shim.mjs",
        ])
        .current_dir(path)
        .spawn()?
        .wait()?;

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
        format!("{}_bg.js", OUT_NAME),
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
