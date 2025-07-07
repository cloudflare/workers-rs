use std::env::var;

use worker_codegen::wit::expand_wit_source;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo::rerun-if-changed=wit/calculator.wit");
    let source = expand_wit_source("wit/calculator.wit")?;
    let out_dir = var("OUT_DIR")?;
    let dest = format!("{out_dir}/calculator.rs");
    std::fs::write(dest, source)?;
    Ok(())
}
