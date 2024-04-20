use std::env::var;

use worker_codegen::wit::expand_wit_source;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo::rerun-if-changed=calculator.wit");
    let source = expand_wit_source("wit/calculator.wit")?;
    let out_dir = var("OUT_DIR")?;
    let dest = format!("{}/calculator.rs", out_dir);
    std::fs::write(dest, source)?;
    Ok(())
}
