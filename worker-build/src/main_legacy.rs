use std::{
    env::{self, VarError},
    fmt::Write as _,
    fs::{self, read_to_string, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};

const OUT_NAME: &str = "index";
const WORKER_SUBDIR: &str = "worker";

const SHIM_TEMPLATE: &str = include_str!("./js/shim-legacy.js");

use crate::binary::{Esbuild, GetBinary};

pub fn process(out_dir: &Path) -> Result<()> {
    let esbuild_path = Esbuild.get_binary(None)?.0;

    create_worker_dir(out_dir)?;
    copy_generated_code_to_worker_dir(out_dir)?;

    let shim_template = match env::var("CUSTOM_SHIM") {
        Ok(path) => {
            let path = Path::new(&path).to_owned();
            println!("Using custom shim from {}", path.display());
            // NOTE: we fail in case that file doesnt exist or something else happens
            read_to_string(&path)
                .with_context(|| format!("Failed to read custom shim from {}", path.display()))?
        }
        Err(_) => SHIM_TEMPLATE.to_owned(),
    };

    let wait_until_response = if env::var("RUN_TO_COMPLETION").is_ok() {
        "this.ctx.waitUntil(response);"
    } else {
        ""
    };

    let snippets_dir = worker_path(out_dir, "snippets");
    let mut snippets = Vec::new();
    let mut counter = 0;

    // wasm-bindgen outputs snippets (https://rustwasm.github.io/wasm-bindgen/reference/js-snippets.html)
    // into the snippets folder, so we recursively read what files were written here and set these up as
    // explicit imports for Wasm instantiation.
    fn get_snippets(
        path: &Path,
        path_string: String,
        counter: &mut i32,
        snippets: &mut Vec<(String, String)>,
    ) -> Result<()> {
        if path.is_dir() {
            for entry in fs::read_dir(path)
                .with_context(|| format!("Failed to read snippets directory {}", path.display()))?
            {
                let entry = entry?;
                get_snippets(
                    &entry.path(),
                    format!("{}/{}", path_string, &entry.file_name().to_string_lossy()),
                    counter,
                    snippets,
                )?;
            }
        } else if path.is_file() {
            snippets.push((format!("snippets_{counter}"), path_string));
            *counter += 1;
        }
        Ok(())
    }

    get_snippets(
        &snippets_dir,
        "./snippets".to_string(),
        &mut counter,
        &mut snippets,
    )?;

    let js_imports = snippets
        .iter()
        .fold(String::new(), |mut output, (name, path)| {
            let _ = writeln!(output, "import * as {name} from \"{path}\";");
            output
        });

    let wasm_imports = snippets
        .into_iter()
        .fold(String::new(), |mut output, (name, path)| {
            let _ = writeln!(output, "\"{path}\": {name},");
            output
        });

    let shim = shim_template
        .replace("$WAIT_UNTIL_RESPONSE", wait_until_response)
        .replace("$SNIPPET_JS_IMPORTS", &js_imports)
        .replace("$SNIPPET_WASM_IMPORTS", &wasm_imports);

    write_string_to_file(worker_path(out_dir, "shim.js"), shim)?;

    bundle(out_dir, &esbuild_path)?;

    remove_unused_js(out_dir)?;

    Ok(())
}

fn create_worker_dir(out_dir: &Path) -> Result<()> {
    // create a directory for our worker to live in
    let worker_dir = out_dir.join(WORKER_SUBDIR);

    // remove anything that already exists
    if worker_dir.is_dir() {
        fs::remove_dir_all(&worker_dir)
            .with_context(|| format!("Failed to remove directory {}", worker_dir.display()))?;
    } else if worker_dir.is_file() {
        fs::remove_file(&worker_dir)
            .with_context(|| format!("Failed to remove file {}", worker_dir.display()))?;
    };

    // create an output dir
    fs::create_dir_all(&worker_dir)
        .with_context(|| format!("Failed to create directory {}", worker_dir.display()))?;

    Ok(())
}

fn copy_generated_code_to_worker_dir(out_dir: &Path) -> Result<()> {
    let glue_src = output_path(out_dir, format!("{OUT_NAME}_bg.js"));
    let glue_dest = worker_path(out_dir, format!("{OUT_NAME}_bg.js"));

    let wasm_src = output_path(out_dir, format!("{OUT_NAME}_bg.wasm"));
    let wasm_dest = worker_path(out_dir, format!("{OUT_NAME}.wasm"));

    // wasm-bindgen supports adding arbitrary JavaScript for a library, so we need to move that as well.
    // https://rustwasm.github.io/wasm-bindgen/reference/js-snippets.html
    let snippets_src = output_path(out_dir, "snippets");
    let snippets_dest = worker_path(out_dir, "snippets");

    for (src, dest) in [
        (glue_src, glue_dest),
        (wasm_src, wasm_dest),
        (snippets_src, snippets_dest),
    ] {
        if !src.exists() {
            continue;
        }

        fs::rename(&src, &dest)
            .with_context(|| format!("Failed to move {} to {}", src.display(), dest.display()))?;
    }

    Ok(())
}

// Bundles the snippets and worker-related code into a single file.
fn bundle(out_dir: &Path, esbuild_path: &Path) -> Result<()> {
    let no_minify = !matches!(env::var("NO_MINIFY"), Err(VarError::NotPresent));
    let worker_subdir = out_dir.join(WORKER_SUBDIR);
    let path = worker_subdir
        .canonicalize()
        .with_context(|| format!("Failed to resolve path {}", worker_subdir.display()))?;
    let esbuild_path = esbuild_path
        .canonicalize()
        .with_context(|| format!("Failed to resolve esbuild path {}", esbuild_path.display()))?;
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
        false => anyhow::bail!("esbuild exited with status {exit_status}"),
    }
}

// After bundling there's no reason why we'd want to upload our now un-used JavaScript so we'll
// delete it.
fn remove_unused_js(out_dir: &Path) -> Result<()> {
    let snippets_dir = worker_path(out_dir, "snippets");

    if snippets_dir.exists() {
        std::fs::remove_dir_all(&snippets_dir)
            .with_context(|| format!("Failed to remove {}", snippets_dir.display()))?;
    }

    let bg_js = worker_path(out_dir, format!("{OUT_NAME}_bg.js"));
    std::fs::remove_file(&bg_js)
        .with_context(|| format!("Failed to remove {}", bg_js.display()))?;
    let shim_js = worker_path(out_dir, "shim.js");
    std::fs::remove_file(&shim_js)
        .with_context(|| format!("Failed to remove {}", shim_js.display()))?;
    let index_js = output_path(out_dir, "index.js");
    std::fs::remove_file(&index_js)
        .with_context(|| format!("Failed to remove {}", index_js.display()))?;

    Ok(())
}

fn write_string_to_file<P: AsRef<Path>>(path: P, contents: impl AsRef<str>) -> Result<()> {
    let path = path.as_ref();
    let mut file =
        File::create(path).with_context(|| format!("Failed to create file {}", path.display()))?;
    file.write_all(contents.as_ref().as_bytes())
        .with_context(|| format!("Failed to write file {}", path.display()))?;

    Ok(())
}

fn worker_path(out_dir: &Path, name: impl AsRef<str>) -> PathBuf {
    out_dir.join(WORKER_SUBDIR).join(name.as_ref())
}

fn output_path(out_dir: &Path, name: impl AsRef<str>) -> PathBuf {
    out_dir.join(name.as_ref())
}
